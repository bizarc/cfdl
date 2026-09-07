use cfdl_expr::{CompiledExpr, ExprEnv, Value as ExprValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;
// THE ENGINE, BY STAGE. Each module is one stage of evaluation; each stage
// completes before the next begins, and sees only what finished earlier.
// `fixtures/valid/evaluation_order` pins the boundaries.
//
//   config         the run: rates, scenarios, the valuation grain
//   prepare        once per model: the grid, the dependency waves, the priced
//                  closure, walk eligibility, compiled plans and openings
//   timeline       the grid: dates, schedules, period arithmetic
//   ir             what the compiler hands us
//   env            the expression environment each stage evaluates in
//   state          stages 1+2, one interleaved walk — fields compute each
//                  period's candidates, the machine moves, the column settles;
//                  `prev` reads what settled
//   occurrence     what happens: events (each occurrence) and options (an
//                  election, as often as it allows), stepped inside the state walk
//                  after the machine and writing through its stores (one value per path)
//   streams        stage 3 — activity, in two phases
//   accounts       the balance plane: openings, movements by side, the
//                  relation fold, the declared inflow
//   walk           the period loop: the stages above in order, one period at
//                  a time; the column order as its one isolated exception
//   distributions  the waterfall stage. Under the walk it runs INSIDE each
//                  period, after that period's streams (`docs/28` §3 stage 3);
//                  under the column order it stays a post-pass over all time
//   fold           from what the evaluation settled to what the results carry:
//                  column-order streams, subtotals, roll-ups, series, metrics,
//                  slices, declared metrics, the statements' inputs
//   runs           the loops: the base run, each scenario, each Monte Carlo
//                  trial — one deterministic evaluation each — and the
//                  results document they assemble
//   results        last — the results document: shapes, hashes, NPV/IRR
//   stochastic     sampling, shared by scenario and Monte Carlo runs
//
// `run_deterministic` below is the orchestrator and the only place the order
// is written down.
mod config;
pub(crate) use accounts::*;
pub use config::*;
pub(crate) use fold::*;
pub(crate) use prepare::*;
pub(crate) use runs::*;
pub(crate) use walk::*;
mod results;
pub use results::*;
mod distributions;
mod fold;
use distributions::*;
mod accounts;
mod occurrence;
mod prepare;
mod runs;
mod streams;
mod walk;
use streams::*;
mod state;
pub use state::*;
mod env;
use env::*;
mod ir;
use ir::*;
mod timeline;
pub use timeline::*;
mod stochastic;
use stochastic::*;

#[derive(Debug)]
pub enum EngineError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidDate(String),
    InvalidRunConfig(String),
    Schedule(String),
    /// A circular series read — or a read into a stream whose series names
    /// are computed at runtime. Evaluation order is an engine concept, so the
    /// check lives with the ordering rather than being restated in the
    /// compiler where the two could drift.
    SeriesCycle(String),
    /// A circular derivation among `assume` values. Same shape as
    /// `SeriesCycle`, one layer up: no order satisfies it, and the engine
    /// refuses rather than iterating.
    AssumptionCycle(String),
    /// A name that resolved to nothing. docs/03 §2: "Unknown variables are
    /// hard errors (EXPR_EVAL), not nulls." Every layer honoured that except
    /// the engine, which caught the error and substituted zero — so a
    /// mistyped `inputs.` or `time.` read produced a column of zeros and a
    /// run reporting ok.
    UnknownName(String),
    /// A series read in an event's guard or action, a field's rule, or an
    /// option's election or payoff. The compiler refuses this
    /// (`E1134_SERIES_READ_IN_LOGIC`); the engine refuses it too, because IR
    /// reaches the engine from paths the compiler never saw.
    SeriesReadInLogic(String),
    /// A stream moves or reads an account, and a forward-reaching read keeps
    /// the model on the column order, where no balance is carried. Refused
    /// rather than published as zeros (`docs/42` §3).
    AccountsNeedTheWalk(String),
    /// A field's rule failed to evaluate — a division by zero inside a
    /// recurrence, a call on an argument out of range. The value used to be
    /// substituted with zero under a warning nobody reads; a number that was
    /// never computed is not a number (`docs/13` §7.103).
    FieldEvaluationFailed(String),
    /// A stream, guard, account or option read a curve outside the effective
    /// dates the curve declares (`docs/13` §7.100). Outside them the curve has
    /// no value — not its end value held flat — so the run refuses, naming
    /// the curve, the date and the reader.
    CurveReadOutsideRange(String),
    /// A value the run supplied — an override, a scenario value, a draw, a
    /// `cfg.` path — is outside the domain of its type, the `within` the
    /// model states, or the bound the pack states on the term that reads it
    /// (`docs/13` §7.56). Refused, never adjusted: a value pulled into range
    /// would be the silent substitution this family of checks exists to end.
    InputOutOfBounds(String),
    /// An event's or option's action names a kind the engine does not
    /// execute. Only hand-written IR can carry one, and running on while the
    /// journal says `ignored` reported success for a run that did not do what
    /// it was asked (`docs/13` §7.83).
    UnknownActionKind(String),
}

impl EngineError {
    /// The diagnostic code a run failure reports under. `E5002` is kept for
    /// what it names — an IR that fails the schema — and every other failure
    /// has a code of its own, registered in `docs/08` (`docs/13` §7.93).
    pub fn code(&self) -> &'static str {
        match self {
            EngineError::Io(_) => "E5002_IR_SCHEMA_VALIDATION_FAILED",
            EngineError::Json(_) => "E5002_IR_SCHEMA_VALIDATION_FAILED",
            EngineError::InvalidDate(_) => "E5033_INVALID_RUN_CONFIG",
            EngineError::InvalidRunConfig(_) => "E5033_INVALID_RUN_CONFIG",
            EngineError::Schedule(_) => "E5034_SCHEDULE_FAILED",
            EngineError::SeriesCycle(_) => "E5035_SERIES_CYCLE",
            EngineError::AssumptionCycle(_) => "E5036_ASSUMPTION_CYCLE",
            EngineError::UnknownName(_) => "E5031_UNRESOLVED_NAME",
            EngineError::SeriesReadInLogic(_) => "E5037_SERIES_READ_IN_LOGIC",
            EngineError::AccountsNeedTheWalk(_) => "E5038_ACCOUNTS_NEED_THE_WALK",
            EngineError::FieldEvaluationFailed(_) => "E5032_FIELD_EVALUATION_FAILED",
            EngineError::UnknownActionKind(_) => "E5039_UNKNOWN_ACTION_KIND",
            EngineError::CurveReadOutsideRange(_) => "E5040_CURVE_READ_OUTSIDE_RANGE",
            EngineError::InputOutOfBounds(_) => "E5041_INPUT_OUT_OF_BOUNDS",
        }
    }
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Io(err) => write!(f, "I/O error: {err}"),
            EngineError::Json(err) => write!(f, "JSON error: {err}"),
            EngineError::SeriesCycle(msg) => write!(f, "{msg}"),
            EngineError::AssumptionCycle(msg) => write!(f, "{msg}"),
            EngineError::UnknownName(msg) => write!(f, "unresolved name: {msg}"),
            EngineError::SeriesReadInLogic(msg) => write!(f, "{msg}"),
            EngineError::FieldEvaluationFailed(msg) => write!(f, "{msg}"),
            EngineError::UnknownActionKind(msg) => write!(f, "{msg}"),
            EngineError::CurveReadOutsideRange(msg) => write!(f, "{msg}"),
            EngineError::InputOutOfBounds(msg) => write!(f, "{msg}"),
            EngineError::AccountsNeedTheWalk(msg) => write!(f, "{msg}"),
            EngineError::InvalidDate(value) => write!(f, "invalid ISO date: {value}"),
            EngineError::InvalidRunConfig(message) => write!(f, "invalid run config: {message}"),
            EngineError::Schedule(message) => write!(f, "unsupported schedule: {message}"),
        }
    }
}

impl std::error::Error for EngineError {}

impl From<std::io::Error> for EngineError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for EngineError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            arithmetic: cfdl_expr::Mode::Decimal,
            discount_rate: 0.0,
            discount_curve: None,
            rate_stated: false,
            as_of: None,
            parameter_overrides: BTreeMap::new(),
            scenarios: BTreeMap::new(),
            monte_carlo: None,
            valuation_grain: None,
        }
    }
}

pub fn run_from_file(ir_path: &Path, config: RunConfig) -> Result<Results, EngineError> {
    let raw = std::fs::read_to_string(ir_path)?;
    run_from_json_str(&raw, config)
}

pub fn run_from_json_str(raw_ir: &str, config: RunConfig) -> Result<Results, EngineError> {
    let ir_value: Value = serde_json::from_str(raw_ir)?;
    let model_hash = canonical_hash(&model_only(&ir_value));
    let ir: Ir = serde_json::from_value(ir_value)?;
    compute_results(&ir, model_hash, config)
}

/// Both evaluation orders over one model, for comparison.
///
/// The collapse property of phase 2 is a claim that the walk computes what the
/// column order computes. This runs both and hands back each stream's column
/// from each, so a test can assert it over the whole blessed corpus rather
/// than on reasoning.
///
/// `Ok(None)` when the model reads forward and the walk cannot run it — not
/// wrong there, inapplicable, so the caller skips rather than fails.
pub fn compare_evaluation_orders(
    raw_ir: &str,
    config: RunConfig,
) -> Result<Option<(StreamColumns, StreamColumns)>, EngineError> {
    let ir: Ir = serde_json::from_str(raw_ir)?;
    let deps = stream_deps(&ir);
    if walk_ineligible_reason(&ir, &deps).is_some() {
        return Ok(None);
    }
    let mut warnings = Vec::new();
    let prep = prepare_model(&ir, &mut warnings)?;
    let timeline = prep.timeline.clone();
    let base_inputs = assumption_inputs(&ir, &mut warnings)?;
    refuse_out_of_bounds_inputs(&ir, &config, &base_inputs)?;

    // The column order: all state first, then each stream over the whole
    // timeline, in dependency waves.
    let (state_values, event_sim) =
        simulate_state(&ir, &config, &timeline, &base_inputs, &mut warnings);
    let mut column: StreamColumns = BTreeMap::new();
    let max_wave = prep.waves.iter().copied().max().unwrap_or(0);
    for wave in 0..=max_wave {
        let snapshot = (wave > 0).then(|| Arc::new(column.clone()));
        for (idx, stream) in ir.streams.iter().enumerate() {
            if prep.waves[idx] != wave {
                continue;
            }
            let mut refused = Vec::new();
            let values = evaluate_stream(
                &ir,
                &config,
                stream,
                &timeline,
                &base_inputs,
                &event_sim,
                &state_values,
                snapshot.as_ref(),
                &mut warnings,
                &mut refused,
            )?;
            column.insert(stream.name.clone(), values);
        }
    }

    // The walk: state settles, then that period's streams, one period at a
    // time.
    let mut walk_warnings = Vec::new();
    let (_state_values, _event_sim, walked, _refusals, _balances, _waterfalls, _stage_journal) =
        walk_periods(&ir, &config, &prep, &base_inputs, &mut walk_warnings);
    Ok(Some((column, walked)))
}

/// Each stream's column, keyed by stream name.
pub type StreamColumns = BTreeMap<String, Vec<f64>>;

/// The streams whose amounts are PRICED — forward windows served by the
/// priced pass (`docs/28` §7). Public so the corpus test keeps the inventory
/// deliberate: a new priced amount is added there by hand, not by blessing.
pub fn priced_streams(raw_ir: &str) -> Result<Vec<String>, EngineError> {
    let ir: Ir = serde_json::from_str(raw_ir)?;
    let deps = stream_deps(&ir);
    Ok(ir
        .streams
        .iter()
        .zip(&deps)
        .filter(|(_, dep)| dep.amount_reads_forward)
        .map(|(stream, _)| stream.name.clone())
        .collect())
}

pub fn walk_eligibility(raw_ir: &str) -> Result<Option<String>, EngineError> {
    let ir: Ir = serde_json::from_str(raw_ir)?;
    let deps = stream_deps(&ir);
    Ok(walk_ineligible_reason(&ir, &deps))
}

fn run_deterministic(
    ir: &Ir,
    config: &RunConfig,
    prep: &ModelPrep<'_>,
) -> Result<DeterministicRunOutput, EngineError> {
    // The grid, the compiled expressions and the schedules come from the
    // preparation, which happens once per model rather than once per run.
    let timeline = prep.timeline.clone();

    let mut warnings = Vec::new();
    let base_inputs = assumption_inputs(ir, &mut warnings)?;
    // EVERY VALUE THE RUN SUPPLIED IS CHECKED HERE, once, before anything
    // reads it: the type's domain and the model's `within` on each
    // assumption, and the pack's bound on each term that defers to the run.
    refuse_out_of_bounds_inputs(ir, config, &base_inputs)?;
    // States are recurrences: every period is computed from the completed
    // previous one, so the whole column exists before anything reads it.
    //
    // THIS RUNS BEFORE EVENTS AND OPTIONS, which is the fix for a defect that
    // made options nearly useless: an `exercise when` could not read
    // `state.<name>` because no state existed yet when it was evaluated, and
    // the failure was silent — a warning and `false`, so the option quietly
    // never exercised and its value vanished.
    //
    // The reorder is sound because the dependency graph is a strict DAG. A
    // state's `next` reads only `prev`, curves, inputs and time — never a
    // stream, never an event, never an option — so nothing an event or option
    // does can reach back into a state.
    // THE WALK, OR THE COLUMN ORDER WHERE A FORWARD READ FORCES IT
    // (`walk.rs`): state, occurrences, streams and the stage settle one period
    // at a time, or — for a model whose window reaches past the period being
    // computed — state runs to completion first and the streams evaluate a
    // column at a time below.
    let evaluated = evaluate_model(ir, config, prep, &timeline, &base_inputs, &mut warnings)?;
    // THE FOLD (`fold.rs`): streams under the column order where the walk did
    // not run them, subtotals, rollups, series, metrics, slices, declared
    // metrics and the statements — everything the results carry.
    fold_results(
        ir,
        config,
        prep,
        &timeline,
        base_inputs,
        evaluated,
        warnings,
    )
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    /// A minimal one-stream IR, with the amount parameterized so a test can
    /// change the model without changing anything else about the run.
    #[cfg(test)]
    fn probe_ir(amount: &str) -> String {
        format!(
            r#"{{
              "model": {{"name": "hash-probe", "currency": "USD"}},
              "time": {{"calendar": "annual", "start": "2026-01-01", "periods": 3}},
              "streams": [{{
                "id": "s1", "name": "probe.rent",
                "owner": {{"symbol": "legal.co"}},
                "direction": "inflow", "currency": "USD",
                "schedule": {{"kind": "Every", "every": "annual",
                             "from": "2026-01-01", "to": "2028-01-01"}},
                "amount": {{"lang": "cfdl", "src": "{amount}"}},
                "active_when": {{"lang": "cfdl", "src": "true"}}
              }}]
            }}"#
        )
    }

    /// The property `ledger_hash` exists to make testable: identical inputs on
    /// an identical engine reproduce an identical ledger.
    ///
    /// Worth stating as a test rather than trusting the golden suite to notice.
    /// A golden diff says "this document changed"; it cannot say whether the
    /// change was a real behavioral difference or a run-to-run wobble, and a
    /// wobble would surface as a flapping test rather than as the defect it is.
    /// The property the whole decoupling exists for: the same cash, modeled
    /// at two different grains, values the same when valued at one convention.
    ///
    /// Before this, `ppy` came from `ir.time.calendar`, so a model's CALENDAR
    /// decided its valuation convention. `benchmarks/cre/mit_rentleg_plaza`
    /// records the consequence — a monthly rebuild "discounting at
    /// (1.12)^(1/12)-1 gives ~$2,323,050, about +1.3%" — and attributes it to
    /// the rebuild. It is not the rebuild. Summing a year's cash and then
    /// discounting at the annual rate is the same arithmetic whichever grain
    /// the cash was modeled on, and this asserts exactly that.
    #[test]
    fn the_same_cash_values_the_same_at_one_convention_whatever_grain_it_was_modeled_on() {
        use super::*;
        let annual_line: Vec<Date> = (0..3)
            .map(|i| Date {
                year: 2026 + i,
                month: 1,
                day: 1,
            })
            .collect();
        let monthly_line: Vec<Date> = (0..36)
            .map(|i| Date {
                year: 2026 + i / 12,
                month: 1 + (i % 12) as u32,
                day: 1,
            })
            .collect();

        // 1,200 a year, one way as a single annual payment and the other as
        // twelve monthly ones. Same cash, same years.
        let annual_streams = vec![(vec![1200.0, 1200.0, 1200.0], 1.0)];
        let monthly_streams = vec![(vec![100.0; 36], 1.0)];

        let rate = 0.12;
        let annual_grain_from_annual = Grain::calendar_year(&annual_line);
        let annual_grain_from_monthly = Grain::calendar_year(&monthly_line);

        let a = npv_at_grain(&annual_streams, rate, &annual_grain_from_annual);
        let b = npv_at_grain(&monthly_streams, rate, &annual_grain_from_monthly);
        assert!(
            (a - b).abs() < 1e-9,
            "valued at the same annual convention these must agree: {a} vs {b}"
        );

        // And the coupling this replaces would NOT have agreed: discounting the
        // monthly model per period is a materially different number.
        let coupled = npv_with_offsets(&monthly_streams, (1.0 + rate).powf(1.0 / 12.0) - 1.0);
        assert!(
            (coupled - a).abs() > 10.0,
            "the old per-period path differs materially: {coupled} vs {a}"
        );
    }

    /// At model grain the new path must agree with the old one to within float
    /// reassociation — and no further.
    ///
    /// The first version of this test asserted bit-equality and failed at 1 ULP
    /// (339.00849393939615 vs 339.0084939393961). That is not a defect in
    /// either path: addition is not associative, and grouping by
    /// `(bucket, offset)` sums in a different order than accumulating stream by
    /// stream. The consequence is recorded rather than papered over — the
    /// identity grain keeps using `npv_with_offsets`, so no published NPV moves.
    ///
    /// Mixed offsets are the case a naive bucketing would break, so they are
    /// the case tested.
    #[test]
    fn npv_at_model_grain_agrees_with_the_per_stream_accumulation() {
        use super::*;
        let timeline: Vec<Date> = (0..6)
            .map(|i| Date {
                year: 2026 + i / 12,
                month: 1 + (i % 12) as u32,
                day: 1,
            })
            .collect();
        let identity = Grain::identity(&timeline, "monthly", "2026-01-01");

        // Deliberately mixed offsets: an ordinary annuity at 1.0 alongside a
        // one-shot settling at the period's open. Collapsing the offset
        // dimension would change this and not the single-offset case.
        let streams = vec![
            (vec![100.0, 100.0, 100.0, 100.0, 100.0, 100.0], 1.0),
            (vec![-500.0, 0.0, 0.0, 0.0, 0.0, 0.0], 0.0),
            (vec![0.0, 0.0, 250.0, 0.0, 0.0, 0.0], 0.5),
        ];
        for rate in [0.0, 0.004074, 0.05, 0.25] {
            let old = npv_with_offsets(&streams, rate);
            let new = npv_at_grain(&streams, rate, &identity);
            let tolerance = old.abs().max(1.0) * 1e-12;
            assert!(
                (old - new).abs() <= tolerance,
                "at model grain the two must agree to within reassociation \
                 (rate {rate}): {old} vs {new}"
            );
        }
    }

    /// Summing into a coarser bucket and discounting once is NOT the same as
    /// discounting each period — which is the entire point, and the reason a
    /// model's calendar must stop deciding its valuation convention.
    #[test]
    fn a_coarser_grain_changes_the_valuation_and_that_is_the_point() {
        use super::*;
        let timeline: Vec<Date> = (0..12)
            .map(|i| Date {
                year: 2026,
                month: 1 + i as u32,
                day: 1,
            })
            .collect();
        let annual = Grain::calendar_year(&timeline);
        assert_eq!(
            annual.buckets.len(),
            1,
            "twelve months of one year is one bucket"
        );

        let streams = vec![(vec![100.0; 12], 1.0)];
        let monthly_rate = 0.01;
        let per_period = npv_with_offsets(&streams, monthly_rate);
        let at_annual = npv_at_grain(&streams, monthly_rate, &annual);
        assert!(
            (per_period - at_annual).abs() > 1.0,
            "discounting twelve times differs from discounting one bucket once: \
             {per_period} vs {at_annual}"
        );
    }

    #[test]
    fn ledger_hash_is_reproducible_and_moves_only_with_the_ledger() {
        use super::*;
        let run = |src: &str, rate: f64| -> (String, String, f64) {
            let config = RunConfig {
                discount_rate: rate,
                rate_stated: true,
                discount_curve: None,
                ..RunConfig::default()
            };
            let results = run_from_json_str(src, config).expect("run");
            let npv = match results.deterministic.metrics.get("model.npv") {
                Some(Scalar::Money(m)) => m.amount,
                other => panic!("expected money npv, got {other:?}"),
            };
            (results.model_hash, results.ledger_hash, npv)
        };

        let (m1, l1, npv1) = run(&probe_ir("100"), 0.10);
        let (m2, l2, npv2) = run(&probe_ir("100"), 0.10);
        assert_eq!(m1, m2, "same source must hash the same");
        assert_eq!(l1, l2, "same run twice must reproduce the ledger exactly");
        assert_eq!(npv1, npv2);

        // The discount rate must NOT move the ledger. The ledger is cash before
        // discounting; the rate belongs to a fold over it. If this ever fails,
        // discounting has leaked into the ledger.
        let (_, l_rate, npv_rate) = run(&probe_ir("100"), 0.25);
        assert_eq!(l1, l_rate, "the discount rate is not part of the ledger");
        assert_ne!(npv1, npv_rate, "but it is part of the valuation");

        // A change to the model's cash must move it.
        let (m_amt, l_amt, _) = run(&probe_ir("101"), 0.10);
        assert_ne!(m1, m_amt);
        assert_ne!(l1, l_amt, "a different ledger must hash differently");
    }

    /// A VIEW IS NOT THE MODEL, AND NOT THE RESULT.
    ///
    /// `docs/13` §7.55. Two users who look at identical results differently
    /// are running the same model: adding a slice or a statement must move
    /// neither hash. A metric is the other side of the same rule — it is a
    /// figure the model CLAIMS, asserted by every benchmark's
    /// `expected_metrics.json`, so it belongs to the model's identity and not
    /// to the ledger's.
    #[test]
    fn a_view_changes_neither_identity_but_a_metric_changes_the_model() {
        use super::*;
        let ir = |extra: &str, views: &str| {
            format!(
                r#"{{
                  "model": {{"name": "view-probe", "currency": "USD"}},
                  "time": {{"calendar": "annual", "start": "2026-01-01", "periods": 3}},
                  "entities": [{{"id": "e1", "symbol": "asset.co", "type": "Asset.Financial",
                                "fields": {{}}, "state": {{}}}}],
                  {extra}
                  {views}
                  "streams": [
                    {{"id": "s1", "name": "probe.rent",
                      "owner": {{"symbol": "asset.co"}},
                      "direction": "inflow", "currency": "USD",
                      "category": "operating.revenue.base_rent",
                      "schedule": {{"kind": "Every", "every": "annual",
                                   "from": "2026-01-01", "to": "2028-01-01"}},
                      "amount": {{"lang": "cfdl", "src": "100"}},
                      "active_when": {{"lang": "cfdl", "src": "true"}}}}
                  ]
                }}"#
            )
        };
        let run = |src: String| run_from_json_str(&src, RunConfig::default()).expect("run");

        let bare = run(ir("", ""));
        let viewed = run(ir(
            "",
            r#""views": {"slices": [{"name": "only_co", "entities": ["asset.co"],
                                    "provenance": {"source_file": "m.cfdl"}}]},"#,
        ));
        let measured = run(ir(
            r#""metrics": [{"name": "rent", "expr": {"lang": "cfdl",
                            "src": "series_sum(\"probe.rent\", 0, 2)"}}],"#,
            "",
        ));

        // The view really was evaluated, so a passing assertion below means
        // the exclusion worked rather than that nothing was there to exclude.
        assert!(
            viewed.slices.as_ref().is_some_and(|s| !s.is_empty()),
            "the slice must actually have been computed"
        );
        assert!(measured.deterministic.metrics.contains_key("metric.rent"));

        assert_eq!(
            bare.model_hash, viewed.model_hash,
            "a slice is a lens on the result, not part of the model"
        );
        assert_eq!(
            bare.ledger_hash, viewed.ledger_hash,
            "a slice changes no cash"
        );
        assert_ne!(
            bare.model_hash, measured.model_hash,
            "a declared metric is a figure the model claims"
        );
        assert_eq!(
            bare.ledger_hash, measured.ledger_hash,
            "a metric is a fold OF the ledger, so it does not change it"
        );
    }

    /// A SUBTOTAL is a fold OF the ledger, so declaring one must not make the
    /// hash claim the cash moved.
    ///
    /// `deterministic.series` was filtered for `domain.*` from the start, and
    /// this looked settled because of it. It was not: the annual rollup went
    /// into the same hash UNFILTERED, so the moment the rollup gained kind-aware
    /// subtotals, `ledger_hash` moved on fifteen goldens whose cash was
    /// bit-identical. The filter had been written onto one field rather than
    /// onto the argument that justifies it.
    ///
    /// Monthly on purpose — an annual model publishes no rollup at all, and
    /// would have passed this test throughout the window when it was broken.
    #[test]
    fn a_fold_over_the_ledger_is_not_part_of_the_ledger() {
        use super::*;
        let ir = |subtotals: &str| {
            format!(
                r#"{{
                  "model": {{"name": "fold-probe", "currency": "USD"}},
                  "time": {{"calendar": "monthly", "start": "2026-01-01", "periods": 24}},
                  "subtotals": [{subtotals}],
                  "streams": [
                    {{"id": "s1", "name": "probe.rent",
                      "owner": {{"symbol": "legal.co"}},
                      "direction": "inflow", "currency": "USD",
                      "category": "operating.revenue.base_rent",
                      "schedule": {{"kind": "Every", "every": "monthly",
                                   "from": "2026-01-01", "to": "2027-12-01"}},
                      "amount": {{"lang": "cfdl", "src": "30000"}},
                      "active_when": {{"lang": "cfdl", "src": "true"}}}},
                    {{"id": "s2", "name": "probe.debt",
                      "owner": {{"symbol": "legal.co"}},
                      "direction": "outflow", "currency": "USD",
                      "category": "financing.debt.service",
                      "schedule": {{"kind": "Every", "every": "monthly",
                                   "from": "2026-01-01", "to": "2027-12-01"}},
                      "amount": {{"lang": "cfdl", "src": "15000"}},
                      "active_when": {{"lang": "cfdl", "src": "true"}}}}
                  ]
                }}"#
            )
        };
        let run = |src: String| run_from_json_str(&src, RunConfig::default()).expect("run");

        let bare = run(ir(""));
        let folded = run(ir(r#"
            {"id": "domain.p.noi", "kind": "money", "op": "sum",
             "categories": ["operating.*"]},
            {"id": "domain.p.ds", "kind": "money", "op": "negated_sum",
             "categories": ["financing.debt.service"]},
            {"id": "domain.p.dscr", "kind": "number", "op": "ratio",
             "numerator": "domain.p.noi", "denominator": "domain.p.ds"}
        "#));

        // The folds really were computed and published, in both places — so a
        // passing hash assertion below means the filter worked, not that there
        // was nothing to filter.
        assert!(folded.deterministic.series.contains_key("domain.p.dscr"));
        let rollup = folded
            .deterministic
            .annual_rollup
            .as_ref()
            .expect("a monthly model publishes an annual rollup");
        assert!(rollup.series.contains_key("domain.p.dscr"));
        assert!(bare.deterministic.annual_rollup.is_some());

        assert_eq!(
            bare.ledger_hash, folded.ledger_hash,
            "declaring a subtotal folds the ledger; it does not change it"
        );
    }

    #[test]
    fn wal_nets_within_an_offset_but_not_across_one() {
        use super::*;
        // Two flows in the SAME period at DIFFERENT points in it are not the
        // same cash at the same moment, so they must not cancel. This is what
        // separates the bucketed WAL from summing the net series first: a
        // purchase settling on its date (offset 0) does not annihilate that
        // period's collections (offset 1), which are a full period later.
        let ppy = 12.0;
        let wal = |streams: &[(Vec<f64>, f64)]| -> Option<f64> {
            let mut by_offset: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
            for (values, offset) in streams {
                let bucket = by_offset
                    .entry((offset * 1e9).round() as i64)
                    .or_insert_with(|| vec![0.0; values.len()]);
                for (idx, value) in values.iter().enumerate() {
                    bucket[idx] += *value;
                }
            }
            let (mut w, mut t) = (0.0_f64, 0.0_f64);
            for (key, values) in &by_offset {
                let offset = *key as f64 / 1e9;
                for (idx, value) in values.iter().enumerate() {
                    if *value > 0.0 {
                        w += ((idx as f64 + offset) / ppy) * *value;
                        t += *value;
                    }
                }
            }
            (t > 0.0).then(|| w / t)
        };

        // Different offsets: the inflow survives at its own instant, 1/12.
        let across = wal(&[(vec![-100.0], 0.0), (vec![100.0], 1.0)]).expect("survives");
        assert!((across - 1.0 / 12.0).abs() < 1e-12, "across = {across}");

        // Same offset: they are the same cash at the same moment and cancel,
        // leaving nothing positive at all.
        let within = wal(&[(vec![-100.0], 1.0), (vec![100.0], 1.0)]);
        assert_eq!(within, None);
    }

    use super::{run_from_json_str, RunConfig};
    use std::collections::BTreeMap;

    #[test]
    fn deterministic_output_for_identical_input() {
        let ir = r#"{
            "model": { "name": "demo", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 3 },
            "streams": [
                {
                    "name": "rent",
                    "owner": { "symbol": "legal.borrower" },
                    "direction": "outflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-03-01" },
                    "amount": { "lang": "cfdl", "src": "cfg.base + time.t" },
                    "active_when": { "lang": "cfdl", "src": "time.t < 2" }
                }
            ]
        }"#;
        let mut overrides = BTreeMap::new();
        overrides.insert("cfg.base".to_string(), 100.0);

        let first = run_from_json_str(
            ir,
            RunConfig {
                arithmetic: cfdl_expr::Mode::Decimal,
                discount_rate: 0.05,
                rate_stated: true,
                discount_curve: None,
                as_of: None,
                parameter_overrides: overrides.clone(),
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
        )
        .unwrap();
        let second = run_from_json_str(
            ir,
            RunConfig {
                arithmetic: cfdl_expr::Mode::Decimal,
                discount_rate: 0.05,
                rate_stated: true,
                discount_curve: None,
                as_of: None,
                parameter_overrides: overrides,
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
        )
        .unwrap();
        let a = serde_json::to_string(&first).unwrap();
        let b = serde_json::to_string(&second).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn obs_map_flows_into_cel_context() {
        let ir = r#"{
            "model": { "name": "obs_test", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 2 },
            "streams": [
                {
                    "name": "test.payment",
                    "owner": { "symbol": "legal.borrower" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cfdl", "src": "obs.rate" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;

        let mut overrides = BTreeMap::new();
        overrides.insert("obs.rate".to_string(), 500.0);

        let results = run_from_json_str(
            ir,
            RunConfig {
                arithmetic: cfdl_expr::Mode::Decimal,
                discount_rate: 0.0,
                rate_stated: true,
                discount_curve: None,
                as_of: None,
                parameter_overrides: overrides,
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
        )
        .expect("obs_map_flows run");

        let total = results
            .deterministic
            .metrics
            .get("stream.test.payment.total")
            .expect("stream metric");
        let amount = match total {
            super::Scalar::Money(m) => m.amount,
            other => panic!("expected money scalar, got {other:?}"),
        };
        // 500 per period × 2 periods = 1000
        assert!(
            (amount - 1000.0).abs() < 1e-9,
            "expected 1000.0, got {amount}"
        );
    }

    fn assert_money(m: &BTreeMap<String, super::Scalar>, key: &str, expected: f64) {
        let amount = match m
            .get(key)
            .unwrap_or_else(|| panic!("missing metric: {key}"))
        {
            super::Scalar::Money(v) => v.amount,
            other => panic!("expected Money for {key}, got {other:?}"),
        };
        assert!(
            (amount - expected).abs() < 1e-9,
            "{key}: expected {expected}, got {amount}"
        );
    }

    /// AN ACTION KIND THE ENGINE DOES NOT KNOW REFUSES THE RUN (`docs/13`
    /// §7.83). `DeactivateContract` is the case in hand: retired from the
    /// language (§7.73), so no compiler emits it, and IR that still carries
    /// it asks for something this engine cannot do. It used to be journaled
    /// as `ignored` under a run reporting ok; the run now says no, and names
    /// the host and the kind.
    #[test]
    fn an_unknown_action_kind_refuses_the_run() {
        let ir = r#"{
            "model": { "name": "contract_action", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 2 },
            "entities": [ { "symbol": "asset.a", "rules": {} } ],
            "events": [
                {
                    "name": "terminate",
                    "when": { "lang": "cfdl", "src": "time.t >= 1" },
                    "actions": [ { "kind": "DeactivateContract", "contract": "cre.lease" } ]
                }
            ],
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "asset.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cfdl", "src": "100.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;
        let err = run_from_json_str(ir, RunConfig::default())
            .expect_err("an action kind the engine cannot execute must refuse the run");
        assert!(
            matches!(err, super::EngineError::UnknownActionKind(_)),
            "{err}"
        );
        assert_eq!(err.code(), "E5039_UNKNOWN_ACTION_KIND");
        let msg = err.to_string();
        assert!(
            msg.contains("event 'terminate'") && msg.contains("DeactivateContract"),
            "{msg}"
        );
    }

    /// A FIELD WHOSE RULE FAILS IS FATAL, NAMED WITH ITS PERIOD (`docs/13`
    /// §7.103). `pmt` with no payments left is the case that found it — a
    /// recurrence stepping past its own maturity — and it used to be a panic
    /// out of the decimal library, then a zero under a warning.
    #[test]
    fn a_failed_field_rule_refuses_the_run_naming_the_period() {
        let ir = r#"{
            "model": { "name": "field_failure", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 3 },
            "entities": [ { "symbol": "asset.a", "rules": {
                "left": { "init": { "lang": "cfdl", "src": "1.0" },
                          "next": { "lang": "cfdl", "src": "pmt(0.05, 2.0 - time.t, 1.0)" } }
            } } ],
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "asset.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-03-01" },
                    "amount": { "lang": "cfdl", "src": "100.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;
        let err = run_from_json_str(ir, RunConfig::default())
            .expect_err("a division by zero inside a recurrence is not a zero");
        assert!(
            matches!(err, super::EngineError::FieldEvaluationFailed(_)),
            "{err}"
        );
        let msg = err.to_string();
        assert!(
            msg.contains("asset.a.left") && msg.contains("period 2"),
            "{msg}"
        );
    }

    /// A DISCOUNT CURVE (`docs/13` §7.4). The IR below is Damodaran's FCFF
    /// Simple Ginzu workbook, sheet "Valuation output": row 9 is the ten
    /// years of FCFF, row 12 the cost of capital converging 7.055% -> 8.81%,
    /// and B20 the PV of those ten years, 16394.53892909532. Discounting
    /// along the curve reproduces B20; a flat curve reproduces the scalar
    /// rate to the last bit; stating both, or a curve the model does not
    /// declare, is refused.
    fn damodaran_ir(curve_points: &str) -> String {
        format!(
            r#"{{
            "model": {{ "name": "ginzu", "currency": "USD" }},
            "time": {{ "calendar": "annual", "start": "2026-01-01", "periods": 10 }},
            "entities": [ {{ "symbol": "asset.firm", "rules": {{}} }} ],
            "curves": [ {{ "name": "cost_of_capital", "interpolation": "step", "points": [{curve_points}] }} ],
            "streams": [
                {{
                    "name": "firm.fcff",
                    "owner": {{ "symbol": "asset.firm" }},
                    "direction": "inflow",
                    "schedule": {{ "kind": "Every", "every": "annual", "from": "2026-01-01", "to": "2035-01-01" }},
                    "amount": {{ "lang": "cfdl", "src": "if(time.t == 0.0, 1982.696720220315, if(time.t == 1.0, 2081.831556231332, if(time.t == 2.0, 2185.9231340428987, if(time.t == 3.0, 2295.219290745043, if(time.t == 4.0, 2423.6376504458894, if(time.t == 5.0, 2495.633211640767, if(time.t == 6.0, 2566.7934322235305, if(time.t == 7.0, 2636.8891298876333, if(time.t == 8.0, 2705.683167881936, 2755.7086026919724)))))))))" }},
                    "active_when": {{ "lang": "cfdl", "src": "true" }}
                }}
            ]
        }}"#
        )
    }

    fn npv_of(results: &super::Results) -> f64 {
        match results.deterministic.metrics.get("model.npv") {
            Some(super::Scalar::Money(m)) => m.amount,
            other => panic!("model.npv: {other:?}"),
        }
    }

    #[test]
    fn a_discount_curve_reproduces_the_workbooks_cumulated_discount_factors() {
        let ir = damodaran_ir(
            r#"{ "date": "2026-01-01", "value": 0.0705501574064654 },
               { "date": "2031-01-01", "value": 0.07406012592517232 },
               { "date": "2032-01-01", "value": 0.07757009444387923 },
               { "date": "2033-01-01", "value": 0.08108006296258614 },
               { "date": "2034-01-01", "value": 0.08459003148129306 },
               { "date": "2035-01-01", "value": 0.0881 }"#,
        );
        let config = RunConfig {
            discount_curve: Some("cost_of_capital".to_string()),
            rate_stated: true,
            ..Default::default()
        };
        let results = run_from_json_str(&ir, config).expect("run");
        let npv = npv_of(&results);
        assert!(
            (npv - 16394.53892909532).abs() < 1e-5,
            "PV of the ten years along the curve: {npv}"
        );
        assert!(matches!(
            results
                .deterministic
                .metrics
                .get("run.annual_discount_curve"),
            Some(super::Scalar::String(name)) if name == "cost_of_capital"
        ));
        assert!(!results
            .deterministic
            .metrics
            .contains_key("run.annual_discount_rate"));
    }

    #[test]
    fn a_flat_discount_curve_is_the_scalar_rate_to_the_last_bit() {
        let ir = damodaran_ir(r#"{ "date": "2026-01-01", "value": 0.0705501574064654 }"#);
        let along = run_from_json_str(
            &ir,
            RunConfig {
                discount_curve: Some("cost_of_capital".to_string()),
                rate_stated: true,
                ..Default::default()
            },
        )
        .expect("curve run");
        let flat = run_from_json_str(
            &ir,
            RunConfig {
                discount_rate: 0.0705501574064654,
                rate_stated: true,
                ..Default::default()
            },
        )
        .expect("scalar run");
        assert_eq!(npv_of(&along).to_bits(), npv_of(&flat).to_bits());
    }

    #[test]
    fn a_discount_curve_the_model_does_not_declare_is_refused() {
        let ir = damodaran_ir(r#"{ "date": "2026-01-01", "value": 0.07 }"#);
        let err = run_from_json_str(
            &ir,
            RunConfig {
                discount_curve: Some("wacc".to_string()),
                rate_stated: true,
                ..Default::default()
            },
        )
        .expect_err("no such curve");
        assert_eq!(err.code(), "E5033_INVALID_RUN_CONFIG");
        assert!(err.to_string().contains("cost_of_capital"), "{err}");
    }

    #[test]
    fn a_run_config_stating_both_a_rate_and_a_curve_is_refused() {
        let raw = r#"{ "deterministic": { "annual_discount_rate": 0.1, "annual_discount_curve": "wacc" } }"#;
        let err = super::run_config_from_json_str(raw, None, None).expect_err("both stated");
        assert_eq!(err.code(), "E5033_INVALID_RUN_CONFIG");
        let raw = r#"{ "deterministic": { "annual_discount_curve": "wacc" }, "scenarios": { "up": { "annual_discount_rate": 0.12 } } }"#;
        let config = super::run_config_from_json_str(raw, Some(0.05), None)
            .expect("curve wins over the fallback");
        assert_eq!(config.discount_curve.as_deref(), Some("wacc"));
        assert!(config.rate_stated);
        assert_eq!(config.scenarios["up"].discount_rate, Some(0.12));
    }

    /// A value the run supplies is checked where it arrives (`docs/13`
    /// §7.56): the type's domain, the model's `within`, and the pack's bound
    /// on a term that defers to the run. Refused, never adjusted.
    #[test]
    fn a_supplied_value_outside_its_bound_is_refused() {
        let ir = r#"{
            "model": { "name": "bounds", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 2 },
            "entities": [ { "symbol": "asset.a", "rules": {} } ],
            "assumptions": {
                "constants": {
                    "share": { "expr": { "lang": "cfdl", "src": "0.9" }, "type": "Fraction" },
                    "cap_rate": { "expr": { "lang": "cfdl", "src": "0.065" }, "type": "Rate", "within": [0.04, 0.10] },
                    "speed": { "expr": { "lang": "cfdl", "src": "1.5" } }
                },
                "random": {
                    "growth": { "dist": { "kind": "Normal", "params": { "mean": 0.03, "stdev": 0.01 } }, "type": "Rate", "within": [-0.05, 0.10] }
                }
            },
            "contracts": [
                {
                    "name": "credit.loan.x",
                    "type": "Credit.Contract.Loan",
                    "subject": { "symbol": "asset.a" },
                    "term_bounds": {
                        "psa_speed": { "reads": "inputs.speed", "min": 0.0, "max": 10.0, "code": "E9016_CREDIT_INVALID_PSA_SPEED" },
                        "abs_speed": { "reads": "cfg.abs", "min": 0.0, "max": 1.0, "code": "E9018_CREDIT_INVALID_ABS_SPEED" }
                    }
                }
            ],
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "asset.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "every": "monthly", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cfdl", "src": "100.0 * inputs.share * inputs.cap_rate * inputs.speed * inputs.growth * cfg.abs" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;
        let run = |keys: &[(&str, f64)]| {
            let mut overrides = BTreeMap::new();
            overrides.insert("cfg.abs".to_string(), 0.02);
            for (k, v) in keys {
                overrides.insert(k.to_string(), *v);
            }
            run_from_json_str(
                ir,
                RunConfig {
                    parameter_overrides: overrides,
                    ..Default::default()
                },
            )
        };
        run(&[]).expect("every value inside its bound");
        run(&[
            ("inputs.share", 1.0),
            ("inputs.cap_rate", 0.10),
            ("inputs.speed", 10.0),
        ])
        .expect("the bounds are inclusive");
        // The type's domain.
        let err = run(&[("inputs.share", 1.3)]).expect_err("a fraction above one");
        assert_eq!(err.code(), "E5041_INPUT_OUT_OF_BOUNDS");
        assert!(
            err.to_string().contains("`inputs.share` is 1.3")
                && err.to_string().contains("domain of a fraction (0 to 1)"),
            "{err}"
        );
        // The model's within.
        let err = run(&[("inputs.cap_rate", 0.12)]).expect_err("a rate past its within");
        assert!(err.to_string().contains("within [0.04, 0.1]"), "{err}");
        // The pack's bound on a term deferred to an input …
        let err = run(&[("inputs.speed", 50.0)]).expect_err("psa past the pack's bound");
        let msg = err.to_string();
        assert!(
            msg.contains("term `psa_speed` reads `inputs.speed` = 50")
                && msg.contains("0 or more and 10 or less")
                && msg.contains("E9016_CREDIT_INVALID_PSA_SPEED"),
            "{msg}"
        );
        // … and to the run configuration's other channel.
        let err = run(&[("cfg.abs", 2.0)]).expect_err("abs past the pack's bound");
        assert!(err.to_string().contains("reads `cfg.abs` = 2"), "{err}");
        // A random assumption's central value is checked like any other.
        let err = run(&[("inputs.growth", 0.5)]).expect_err("growth past its within");
        assert!(err.to_string().contains("`inputs.growth` is 0.5"), "{err}");
    }

    /// AN OVERRIDE THAT MATCHES NOTHING IS REFUSED (`docs/13` §7.51, §7.116):
    /// a key with no prefix, a misspelled input, a stream nobody declared, in
    /// the deterministic block, a scenario or the Monte Carlo distributions.
    /// A key that resolves — a declared assumption, an input the model reads
    /// without declaring, a `cfg.` path some expression reads — is accepted.
    #[test]
    fn an_override_that_matches_nothing_is_refused_naming_the_nearest_input() {
        let ir = r#"{
            "model": { "name": "overrides", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 2 },
            "entities": [ { "symbol": "asset.a", "rules": {} } ],
            "assumptions": { "constants": { "cpr": { "expr": { "lang": "cfdl", "src": "0.1" } } } },
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "asset.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "every": "monthly", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cfdl", "src": "100.0 * inputs.cpr * cfg.stress.factor + inputs.undeclared" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;
        let run = |keys: &[(&str, f64)]| {
            let mut overrides = BTreeMap::new();
            for (k, v) in keys {
                overrides.insert(k.to_string(), *v);
            }
            run_from_json_str(
                ir,
                RunConfig {
                    parameter_overrides: overrides,
                    ..Default::default()
                },
            )
        };
        // Resolving keys: a declared assumption, an input only the model reads,
        // a cfg path an expression reads, a declared stream.
        run(&[
            ("inputs.cpr", 0.2),
            ("inputs.undeclared", 1.0),
            ("cfg.stress.factor", 2.0),
            ("stream.ops.revenue:amount", 5.0),
        ])
        .expect("every key resolves");
        // A bare key: refused, with the prefix it wanted.
        let err = run(&[
            ("cpr", 0.2),
            ("inputs.undeclared", 1.0),
            ("cfg.stress.factor", 1.0),
        ])
        .expect_err("bare key");
        assert_eq!(err.code(), "E5033_INVALID_RUN_CONFIG");
        let msg = err.to_string();
        assert!(
            msg.contains("deterministic block sets `cpr`")
                && msg.contains("did you mean `inputs.cpr`"),
            "{msg}"
        );
        // A misspelled input: refused, with the nearest declared name.
        let err = run(&[
            ("inputs.cprr", 0.2),
            ("inputs.undeclared", 1.0),
            ("cfg.stress.factor", 1.0),
        ])
        .expect_err("misspelled input");
        assert!(
            err.to_string().contains("did you mean `inputs.cpr`"),
            "{err}"
        );
        // A stream nobody declared, and a cfg path nothing reads.
        let err = run(&[
            ("stream.ops.cost:amount", 1.0),
            ("inputs.undeclared", 1.0),
            ("cfg.stress.factor", 1.0),
        ])
        .expect_err("unknown stream");
        assert!(
            err.to_string().contains("`stream.ops.cost:amount`"),
            "{err}"
        );
        let err = run(&[
            ("cfg.other", 1.0),
            ("inputs.undeclared", 1.0),
            ("cfg.stress.factor", 1.0),
        ])
        .expect_err("unread cfg path");
        assert!(err.to_string().contains("`cfg.other`"), "{err}");
        // A scenario's key is checked too, and named by the scenario.
        let mut scenarios = BTreeMap::new();
        let mut bad = BTreeMap::new();
        bad.insert("cpr".to_string(), 0.3);
        scenarios.insert(
            "stress".to_string(),
            super::ScenarioRunConfig {
                discount_rate: None,
                discount_curve: None,
                as_of: None,
                parameter_overrides: bad,
            },
        );
        let mut base = BTreeMap::new();
        base.insert("inputs.undeclared".to_string(), 1.0);
        base.insert("cfg.stress.factor".to_string(), 1.0);
        let err = run_from_json_str(
            ir,
            RunConfig {
                parameter_overrides: base,
                scenarios,
                ..Default::default()
            },
        )
        .expect_err("scenario key");
        assert!(
            err.to_string().contains("scenario 'stress' sets `cpr`"),
            "{err}"
        );
    }

    /// NO RATE, NO NPV (`docs/13` §7.46): a run nobody gave a discount rate
    /// publishes neither `model.npv` nor `run.annual_discount_rate`, and says
    /// why; a run that states one, even zero, publishes both.
    #[test]
    fn a_run_without_a_stated_rate_publishes_no_npv() {
        let ir = r#"{
            "model": { "name": "no_rate", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 2 },
            "entities": [ { "symbol": "asset.a", "rules": {} } ],
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "asset.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cfdl", "src": "100.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;
        let unstated = run_from_json_str(ir, RunConfig::default()).expect("run");
        assert!(!unstated.deterministic.metrics.contains_key("model.npv"));
        assert!(!unstated
            .deterministic
            .metrics
            .contains_key("run.annual_discount_rate"));
        assert!(unstated.deterministic.metrics.contains_key("model.total"));
        assert!(unstated
            .warnings
            .iter()
            .any(|w| w.contains("No discount rate was stated")));
        let stated = run_from_json_str(
            ir,
            RunConfig {
                discount_rate: 0.0,
                rate_stated: true,
                discount_curve: None,
                ..RunConfig::default()
            },
        )
        .expect("run");
        assert!(stated.deterministic.metrics.contains_key("model.npv"));
    }

    /// A DECLARED RUN MODE IS PICKED UP ONLY WHEN IT IS A USABLE ONE.
    ///
    /// `compute_results` reads `ir.runs` for a Monte Carlo run when the run
    /// config asks for none. Both halves of that condition were untestable
    /// from source: the parser now refuses `trials 0`
    /// (`invalid/run_monte_carlo_zero_trials`), and no grammar puts a trial
    /// count on a `run deterministic`. Hand-written IR can do both, which is
    /// the only place the guard is reachable — and mutation testing found it
    /// by surviving `> 0` → `>= 0` and `&&` → `||` with nothing to tell them
    /// apart (`docs/30`).
    #[test]
    fn a_declared_run_needs_a_kind_and_a_positive_trial_count() {
        fn ir_with_run(kind: &str, trials: u64) -> String {
            format!(
                r#"{{
                "model": {{ "name": "declared_run", "currency": "USD" }},
                "time": {{ "calendar": "monthly", "start": "2026-01-01", "periods": 2 }},
                "entities": [ {{ "symbol": "asset.a", "rules": {{}} }} ],
                "runs": [ {{ "kind": "{kind}", "trials": {trials}, "seed": 7 }} ],
                "streams": [
                    {{
                        "name": "ops.revenue",
                        "owner": {{ "symbol": "asset.a" }},
                        "direction": "inflow",
                        "schedule": {{ "kind": "Every", "from": "2026-01-01", "to": "2026-02-01" }},
                        "amount": {{ "lang": "cfdl", "src": "100.0" }},
                        "active_when": {{ "lang": "cfdl", "src": "true" }}
                    }}
                ]
            }}"#
            )
        }

        // The usable case: honoured, with the declared trial count and seed.
        let results = run_from_json_str(&ir_with_run("monte_carlo", 4), RunConfig::default())
            .expect("a declared monte_carlo run is honoured");
        assert_eq!(
            results.monte_carlo.status, "ok",
            "a declared monte_carlo run should actually run"
        );
        assert_eq!(
            results.monte_carlo.trials, 4,
            "the declared trial count is what runs"
        );

        // Zero trials is not a run. `>= 0` would set one up with no trials.
        let results = run_from_json_str(&ir_with_run("monte_carlo", 0), RunConfig::default())
            .expect("zero trials runs deterministically, not as an error");
        assert_eq!(
            results.monte_carlo.status, "not_run",
            "a monte_carlo run of zero trials is not a run and must not be set up"
        );

        // A trial count on a run that is not Monte Carlo is not a Monte Carlo
        // run. `||` would treat this one as if it were.
        let results = run_from_json_str(&ir_with_run("deterministic", 4), RunConfig::default())
            .expect("a deterministic run with a stray trial count still runs");
        assert_eq!(
            results.monte_carlo.status, "not_run",
            "only a run whose kind is monte_carlo may set one up"
        );
    }

    /// The IR the compiler will no longer emit, run directly.
    ///
    /// Narrowed in phase 3: this guard reads the CURRENT period, which stays
    /// refused. Settled history — `time.t - 1` and earlier — is now legal, and
    /// `fixtures/valid/logic_reads_settled_cash` is where that is pinned.
    ///
    /// `E1134_SERIES_READ_IN_LOGIC` refuses this in every model written in
    /// CFDL, which is why no fixture can carry it: the compiler stops first.
    /// The engine still accepts IR from the WASM, server and Python paths,
    /// where nothing has validated it — so the backstop is only reachable, and
    /// only testable, from hand-written IR.
    ///
    /// Without it the engine warns once per period and substitutes `false`,
    /// publishing a run that reports ok with an event that never fired
    /// (`docs/13` §7.71).
    #[test]
    fn a_guard_reading_a_series_is_refused_at_ir_load() {
        let ir = r#"{
            "model": { "name": "guard_reads_series", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 3 },
            "entities": [ { "symbol": "asset.a", "rules": {} } ],
            "events": [
                {
                    "name": "vacate",
                    "when": { "lang": "cfdl", "src": "series_sum(\"ops.revenue\", time.t, time.t) < 50" },
                    "actions": []
                }
            ],
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "asset.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-03-01" },
                    "amount": { "lang": "cfdl", "src": "100.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;

        let err = run_from_json_str(ir, RunConfig::default())
            .expect_err("a guard reading a series must be refused, not warned about");
        let message = err.to_string();
        assert!(
            matches!(err, super::EngineError::SeriesReadInLogic(_)),
            "expected SeriesReadInLogic, got: {message}"
        );
        // The message must name WHERE, or it sends the reader hunting.
        assert!(
            message.contains("event 'vacate' guard") && message.contains("ops.revenue"),
            "message should name the site and the read: {message}"
        );
    }

    /// The same read in a field's rule, which fails differently and worse: the
    /// substituted zero nulls the whole expression and `prev` carries it.
    #[test]
    fn a_recurrence_reading_a_series_is_refused_at_ir_load() {
        let ir = r#"{
            "model": { "name": "rule_reads_series", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 3 },
            "entities": [
                {
                    "symbol": "asset.a",
                    "rules": {
                        "occupancy": {
                            "init": { "lang": "cfdl", "src": "0.8" },
                            "next": { "lang": "cfdl", "src": "prev + series_sum(\"ops.revenue\", time.t, time.t)" }
                        }
                    }
                }
            ],
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "asset.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-03-01" },
                    "amount": { "lang": "cfdl", "src": "100.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;

        let err = run_from_json_str(ir, RunConfig::default())
            .expect_err("a rule reading a series must be refused");
        let message = err.to_string();
        assert!(
            message.contains("field 'asset.a.occupancy' in 'next'"),
            "message should name the field and the clause: {message}"
        );
    }

    /// The backstop must not refuse what is legal: a STREAM reading another
    /// stream is the language's ordinary cross-stream read, and the whole
    /// dependency-wave design exists to serve it.
    #[test]
    fn a_stream_reading_a_series_is_untouched() {
        let ir = r#"{
            "model": { "name": "stream_reads_series", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 3 },
            "entities": [ { "symbol": "asset.a", "rules": {} } ],
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "asset.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-03-01" },
                    "amount": { "lang": "cfdl", "src": "100.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                },
                {
                    "name": "ops.fee",
                    "owner": { "symbol": "asset.a" },
                    "direction": "outflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-03-01" },
                    "amount": { "lang": "cfdl", "src": "series_sum(\"ops.revenue\", time.t, time.t) * 0.1" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;

        run_from_json_str(ir, RunConfig::default())
            .expect("a stream reading another stream is legal and must stay legal");
    }

    #[test]
    fn multi_stream_period_aggregation() {
        // Three concurrent streams: two ops (inflow/outflow) active periods 0–1,
        // one exit event at period 2. Verifies:
        //   - per-period net = inflow - outflow (sign handling correct)
        //   - stream totals accumulate correctly across periods
        //   - a terminal stream fires exactly once at the right period
        //   - model total = sum of all stream contributions
        let ir = r#"{
            "model": { "name": "agg_test", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 3 },
            "streams": [
                {
                    "name": "ops.revenue",
                    "owner": { "symbol": "entity.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cfdl", "src": "3000.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                },
                {
                    "name": "ops.expense",
                    "owner": { "symbol": "entity.a" },
                    "direction": "outflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cfdl", "src": "1000.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                },
                {
                    "name": "exit.proceeds",
                    "owner": { "symbol": "entity.a" },
                    "direction": "inflow",
                    "schedule": { "kind": "OnDate", "on": "2026-03-01" },
                    "amount": { "lang": "cfdl", "src": "50000.0" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;

        let results = run_from_json_str(
            ir,
            RunConfig {
                arithmetic: cfdl_expr::Mode::Decimal,
                discount_rate: 0.0,
                rate_stated: true,
                discount_curve: None,
                as_of: None,
                parameter_overrides: BTreeMap::new(),
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
        )
        .expect("aggregation run");

        // A published series entry is Money or a bare number; these keys are all
        // cash, so unwrapping here asserts the denomination as well as the value.
        // A `state.` series would fail this, which is the point.
        fn cash(value: &super::SeriesValue) -> f64 {
            value.money_amount().expect("cash series entry")
        }

        let m = &results.deterministic.metrics;
        let s = &results.deterministic.series;

        // --- Totals (scalar metrics) ---
        assert_money(m, "stream.ops.revenue.total", 6000.0); // 3000 x 2 periods
        assert_money(m, "stream.ops.expense.total", -2000.0); // -1000 x 2 periods
        assert_money(m, "stream.exit.proceeds.total", 50000.0); // single event
        assert_money(m, "model.total", 54000.0); // 6000 - 2000 + 50000

        // --- Per-stream monthly series (the T-12 / pro-forma interface) ---
        // Revenue: active periods 0 and 1, zero at period 2
        let rev = &s["stream.ops.revenue"].values;
        assert_eq!(rev.len(), 3);
        assert!((cash(&rev[0]) - 3000.0).abs() < 1e-9, "revenue[0]");
        assert!((cash(&rev[1]) - 3000.0).abs() < 1e-9, "revenue[1]");
        assert!((cash(&rev[2])).abs() < 1e-9, "revenue[2] should be 0");

        // Expense: outflow sign, active periods 0 and 1, zero at period 2
        let exp = &s["stream.ops.expense"].values;
        assert_eq!(exp.len(), 3);
        assert!((cash(&exp[0]) - (-1000.0)).abs() < 1e-9, "expense[0]");
        assert!((cash(&exp[1]) - (-1000.0)).abs() < 1e-9, "expense[1]");
        assert!((cash(&exp[2])).abs() < 1e-9, "expense[2] should be 0");

        // Exit: zero for first two periods, fires only at period 2
        let exit = &s["stream.exit.proceeds"].values;
        assert_eq!(exit.len(), 3);
        assert!((cash(&exit[0])).abs() < 1e-9, "exit[0] should be 0");
        assert!((cash(&exit[1])).abs() < 1e-9, "exit[1] should be 0");
        assert!((cash(&exit[2]) - 50000.0).abs() < 1e-9, "exit[2]");

        // --- Aggregate net cash flow series ---
        // Period 0: 3000 - 1000 = 2000; Period 1: same; Period 2: 50000 (exit only)
        let net = &s["model.net_cash_flow"].values;
        assert_eq!(net.len(), 3);
        assert!((cash(&net[0]) - 2000.0).abs() < 1e-9, "net[0]");
        assert!((cash(&net[1]) - 2000.0).abs() < 1e-9, "net[1]");
        assert!((cash(&net[2]) - 50000.0).abs() < 1e-9, "net[2]");
    }

    #[test]
    fn supports_colon_boundary_stream_amount_override_key() {
        let ir = r#"{
            "model": { "name": "demo", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 2 },
            "streams": [
                {
                    "name": "cre.lease.base_rent",
                    "owner": { "symbol": "legal.borrower" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cfdl", "src": "10" },
                    "active_when": { "lang": "cfdl", "src": "true" }
                }
            ]
        }"#;

        let mut overrides = BTreeMap::new();
        overrides.insert("stream.cre.lease.base_rent:amount".to_string(), 25.0);
        let results = run_from_json_str(
            ir,
            RunConfig {
                arithmetic: cfdl_expr::Mode::Decimal,
                discount_rate: 0.0,
                rate_stated: true,
                discount_curve: None,
                as_of: None,
                parameter_overrides: overrides,
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
        )
        .expect("colon-boundary override run");

        let total = results
            .deterministic
            .metrics
            .get("stream.cre.lease.base_rent.total")
            .expect("stream metric");
        let total = match total {
            super::Scalar::Money(money) => money.amount,
            other => panic!("expected money scalar, got {other:?}"),
        };
        // Override 25 per period, 2 periods => total 50
        assert!((total - 50.0).abs() < 1e-9);

        // Legacy and bracket key forms are REFUSED, not ignored (`docs/13`
        // §7.51): a key that matches nothing would leave the run unchanged and
        // reported as ok, and the refusal names the spelling it wanted.
        for legacy_key in [
            "stream.cre.lease.base_rent.amount",
            "stream[\"cre.lease.base_rent\"].amount",
        ] {
            let mut legacy = BTreeMap::new();
            legacy.insert(legacy_key.to_string(), 99.0);
            let err = run_from_json_str(
                ir,
                RunConfig {
                    parameter_overrides: legacy,
                    rate_stated: true,
                    ..Default::default()
                },
            )
            .expect_err("a key that matches nothing is refused");
            assert_eq!(err.code(), "E5033_INVALID_RUN_CONFIG");
            assert!(
                err.to_string()
                    .contains("stream.cre.lease.base_rent:amount"),
                "{err}"
            );
        }
    }

    #[test]
    fn irr_simple_two_period() {
        // Invest $1000, receive $1100 one period later → IRR = 10%
        let result = super::irr_with_offsets(&[(vec![-1000.0, 1100.0], 0.0)])
            .expect("IRR should be defined");
        assert!(
            (result - 0.10).abs() < 1e-6,
            "expected IRR ≈ 0.10, got {result}"
        );
    }

    #[test]
    fn irr_undefined_all_positive() {
        // No sign change → IRR undefined
        assert!(super::irr_with_offsets(&[(vec![100.0, 200.0], 0.0)]).is_none());
    }
}

#[cfg(test)]
mod assumption_order_tests {
    use super::*;

    fn ir_with(assumes: &[(&str, &str)]) -> Ir {
        let constants: serde_json::Map<String, serde_json::Value> = assumes
            .iter()
            .map(|(name, src)| {
                (
                    (*name).to_string(),
                    serde_json::json!({ "expr": { "lang": "cfdl", "src": src } }),
                )
            })
            .collect();
        let ir_json = serde_json::json!({
            "model": { "name": "m", "currency": "USD" },
            "time": { "calendar": "annual", "start": "2026-01-01", "periods": 2 },
            "entities": [{ "symbol": "asset.co" }],
            "assumptions": { "constants": constants },
            "streams": [{
                "name": "base.rent",
                "owner": { "symbol": "asset.co" },
                "direction": "inflow",
                "currency": "USD",
                "amount": { "lang": "cfdl", "src": "1.0" },
                "schedule": { "kind": "Every", "every": "annual",
                              "from": "2026-01-01", "to": "2027-01-01" }
            }]
        });
        serde_json::from_value(ir_json).expect("ir parses")
    }

    /// A derived assumption is ordinary modeling. Evaluated in name order
    /// alone, `net_sf` read an empty environment and resolved to nothing.
    #[test]
    fn an_assumption_may_be_derived_from_another() {
        let ir = ir_with(&[
            ("gross_sf", "10000.0"),
            ("efficiency", "0.85"),
            ("net_sf", "inputs.gross_sf * inputs.efficiency"),
        ]);
        let out = run_deterministic(
            &ir,
            &RunConfig {
                rate_stated: true,
                discount_curve: None,
                ..RunConfig::default()
            },
            &prepare_model(&ir, &mut Vec::new()).expect("prepares"),
        )
        .expect("resolves");
        assert_eq!(out.resolved_inputs["net_sf"], 8500.0);
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    }

    /// Name order would have resolved this one by luck; dependency order
    /// resolves it because it is correct.
    #[test]
    fn order_follows_dependencies_not_names() {
        // `alpha` reads `zulu`, so the alphabetical walk meets it first.
        let ir = ir_with(&[("alpha", "inputs.zulu * 3.0"), ("zulu", "7.0")]);
        let out = run_deterministic(
            &ir,
            &RunConfig::default(),
            &prepare_model(&ir, &mut Vec::new()).expect("prepares"),
        )
        .expect("resolves");
        assert_eq!(out.resolved_inputs["alpha"], 21.0);
    }

    #[test]
    fn a_circular_derivation_is_refused_with_its_path() {
        let ir = ir_with(&[
            ("gross_sf", "inputs.net_sf / 2.0"),
            ("net_sf", "inputs.gross_sf * 0.85"),
        ]);
        let err = run_deterministic(
            &ir,
            &RunConfig::default(),
            &prepare_model(&ir, &mut Vec::new()).expect("prepares"),
        )
        .expect_err("no order exists");
        match err {
            EngineError::AssumptionCycle(msg) => {
                assert!(msg.contains("cyclic assumptions"), "{msg}");
                assert!(msg.contains("'gross_sf'"), "{msg}");
                assert!(msg.contains("'net_sf'"), "{msg}");
            }
            other => panic!("expected AssumptionCycle, got {other:?}"),
        }
    }

    /// A name that is not an assumption is not an edge — it comes from the run
    /// configuration, or from nowhere, and the unresolved-name gate speaks for
    /// the latter.
    #[test]
    fn a_non_assumption_name_is_not_a_dependency() {
        let ir = ir_with(&[("net_sf", "100.0"), ("unused", "5.0")]);
        let out = run_deterministic(
            &ir,
            &RunConfig::default(),
            &prepare_model(&ir, &mut Vec::new()).expect("prepares"),
        )
        .expect("resolves");
        assert_eq!(out.resolved_inputs["net_sf"], 100.0);
    }
}

#[cfg(test)]
mod series_wave_tests {
    use super::*;

    #[test]
    fn extracts_literal_series_names() {
        assert_eq!(
            series_references(r#"series_sum("base.revenue", 0, time.t) * 0.1"#),
            vec!["base.revenue"]
        );
        assert_eq!(
            series_references(r#"series_avg( "a.b" , 0, 1) + series_sum("c.d", 0, 1)"#),
            vec!["c.d", "a.b"]
        );
        // A computed name is not addressed here; the runtime still returns 0
        // for an unmatched name, which is right for a stream that never lowered.
        assert!(series_references("series_sum(name_var, 0, 1)").is_empty());
        assert!(series_references("amount * 2").is_empty());
    }

    fn dep(uses: bool, computed: bool, refs: &[&str]) -> StreamDeps {
        StreamDeps {
            uses,
            computed,
            refs: refs.iter().map(|r| r.to_string()).collect(),
            reads_forward: false,
            amount_reads_forward: false,
        }
    }

    #[test]
    fn waves_are_dependency_depth() {
        // base -> mid -> top, plus a reader of nothing that still leaves wave 0.
        let names = ["base", "mid", "orphan_reader", "top"];
        let deps = vec![
            dep(false, false, &[]),
            dep(true, false, &["base"]),
            dep(true, false, &["no.such.stream"]),
            dep(true, false, &["mid"]),
        ];
        assert_eq!(assign_waves(&names, &deps).unwrap(), vec![0, 1, 1, 2]);
    }

    #[test]
    fn glob_references_resolve_to_every_member_of_the_family() {
        let names = ["fam.a", "fam.b", "reader"];
        let deps = vec![
            dep(false, false, &[]),
            dep(true, false, &["fam.a"]),
            dep(true, false, &["fam.*"]),
        ];
        // `fam.*` reaches fam.b, which reads fam.a — so the reader is wave 2.
        assert_eq!(assign_waves(&names, &deps).unwrap(), vec![0, 1, 2]);
    }

    #[test]
    fn a_self_read_is_a_cycle() {
        let names = ["x"];
        let deps = vec![dep(true, false, &["x"])];
        let err = assign_waves(&names, &deps).unwrap_err();
        match err {
            EngineError::SeriesCycle(msg) => {
                assert!(msg.contains("'x' -> 'x'"), "{msg}");
            }
            other => panic!("expected SeriesCycle, got {other:?}"),
        }
    }

    #[test]
    fn a_computed_name_reader_evaluates_last_and_cannot_be_read() {
        let names = ["base", "literal_reader", "runtime_reader"];
        let deps = vec![
            dep(false, false, &[]),
            dep(true, false, &["base"]),
            dep(true, true, &[]),
        ];
        assert_eq!(assign_waves(&names, &deps).unwrap(), vec![0, 1, 2]);

        let deps_with_read_into = vec![
            dep(false, false, &[]),
            dep(true, false, &["runtime_reader"]),
            dep(true, true, &[]),
        ];
        let err = assign_waves(&names, &deps_with_read_into).unwrap_err();
        match err {
            EngineError::SeriesCycle(msg) => {
                assert!(msg.contains("computes its series names at"), "{msg}");
                assert!(msg.contains("runtime_reader"), "{msg}");
            }
            other => panic!("expected SeriesCycle, got {other:?}"),
        }
    }

    fn chain_ir(b_reads: &str) -> Ir {
        let ir_json = serde_json::json!({
            "model": { "name": "m", "currency": "USD" },
            "time": { "calendar": "annual", "start": "2026-01-01", "periods": 3 },
            "entities": [{ "symbol": "asset.co" }],
            "streams": [
                {
                    "name": "base.revenue",
                    "owner": { "symbol": "asset.co" },
                    "direction": "inflow",
                    "currency": "USD",
                    "amount": { "lang": "cfdl", "src": "100" },
                    "schedule": { "kind": "Every", "every": "annual",
                                  "from": "2026-01-01", "to": "2028-01-01" }
                },
                {
                    "name": "derived.a",
                    "owner": { "symbol": "asset.co" },
                    "direction": "inflow",
                    "currency": "USD",
                    "amount": { "lang": "cfdl",
                                "src": "series_sum(\"base.revenue\", 0, time.t)" },
                    "schedule": { "kind": "Every", "every": "annual",
                                  "from": "2026-01-01", "to": "2028-01-01" }
                },
                {
                    "name": "derived.b",
                    "owner": { "symbol": "asset.co" },
                    "direction": "inflow",
                    "currency": "USD",
                    "amount": { "lang": "cfdl",
                                "src": format!("series_sum(\"{b_reads}\", 0, time.t)") },
                    "schedule": { "kind": "Every", "every": "annual",
                                  "from": "2026-01-01", "to": "2028-01-01" }
                }
            ]
        });
        serde_json::from_value(ir_json).expect("ir parses")
    }

    /// The chain the two-phase engine refused outright: a stream reading a
    /// stream that itself reads one. Waves order it — and the numbers prove
    /// `derived.b` saw `derived.a` FINISHED, not the empty store the sealed
    /// design handed phase 2.
    #[test]
    fn a_depth_two_chain_evaluates_in_order() {
        let ir = chain_ir("derived.a");
        let out = run_deterministic(
            &ir,
            &RunConfig {
                rate_stated: true,
                discount_curve: None,
                ..RunConfig::default()
            },
            &prepare_model(&ir, &mut Vec::new()).expect("prepares"),
        )
        .expect("chain evaluates");
        // base = [100, 100, 100]; a = cumsum(base) = [100, 200, 300];
        // b = cumsum(a) = [100, 300, 600]. b's total is 1000 only if a was
        // complete when b evaluated.
        let total = |name: &str| -> f64 {
            out.series[name]
                .values
                .iter()
                .filter_map(|v| v.money_amount())
                .sum()
        };
        assert_eq!(total("stream.derived.a"), 600.0);
        assert_eq!(total("stream.derived.b"), 1000.0);
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    }

    /// Two streams reading each other have no evaluation order at all — the
    /// one rejection waves keep, named as the actual cycle.
    #[test]
    fn a_genuine_cycle_is_an_error_naming_the_path() {
        let mut ir = chain_ir("derived.a");
        // Rewire: derived.a reads derived.b, closing the loop.
        ir.streams[1].amount.src = "series_sum(\"derived.b\", 0, time.t)".to_string();
        // REFUSED AT PREPARATION, which is where the wave ordering is now
        // computed — once per model rather than once per run, so a Monte Carlo
        // does not rediscover the same cycle on every trial.
        let Err(err) = prepare_model(&ir, &mut Vec::new()) else {
            panic!("a circular read has no order");
        };
        match err {
            EngineError::SeriesCycle(msg) => {
                assert!(msg.contains("cyclic series reads"), "{msg}");
                assert!(msg.contains("'derived.a'"), "{msg}");
                assert!(msg.contains("'derived.b'"), "{msg}");
            }
            other => panic!("expected SeriesCycle, got {other:?}"),
        }
    }
}
