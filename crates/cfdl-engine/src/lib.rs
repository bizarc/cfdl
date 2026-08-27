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
//   timeline       the grid: dates, schedules, period arithmetic
//   ir             what the compiler hands us
//   env            the expression environment each stage evaluates in
//   state          stages 1+2, one interleaved walk — fields compute each
//                  period's candidates, events overwrite, the column settles;
//                  `prev` reads what settled (one value per path)
//   streams        stage 3 — activity, in two phases
//   distributions  stage 4 — waterfalls, allocating `available`
//   results        stage 5 — netting, rollups, metrics, statements
//   stochastic     sampling, shared by scenario and Monte Carlo runs
//
// `run_deterministic` below is the orchestrator and the only place the order
// is written down.
mod config;
pub use config::*;
mod results;
pub use results::*;
mod distributions;
use distributions::*;
mod streams;
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
    let model_hash = canonical_hash(&ir_value);
    let ir: Ir = serde_json::from_value(ir_value)?;
    compute_results(&ir, model_hash, config)
}

/// A series read where no stream value exists — the engine's backstop for the
/// compiler's `E1134_SERIES_READ_IN_LOGIC`.
///
/// The compiler refuses this on every model it sees, which is every model
/// written in CFDL. The engine also accepts IR directly — the WASM, server and
/// Python paths all do — and there the compiler's check has not run. Without
/// this the engine warns once per period and substitutes `false` or `0`,
/// publishing a full set of numbers under `status: ok`: a guard that never
/// fires, or a recurrence whose collapse `prev` carries for the rest of the
/// run (`docs/13` §7.71).
///
/// `docs/28` §4 is where this becomes an ordering rule rather than a
/// prohibition: under the period walk a guard may read a stream's settled
/// history, at or before the previous period. Same-period and forward reads
/// stay refused, so this narrows rather than disappears.
fn refuse_series_reads_in_logic(ir: &Ir) -> Result<(), EngineError> {
    let mut offences: Vec<String> = Vec::new();

    let mut check = |src: &str, site: String| {
        if let Some(func) = cfdl_expr::series_call(src) {
            offences.push(format!("{site} calls `{func}`"));
        }
    };

    for entity in &ir.entities {
        for (field, rule) in &entity.rules {
            check(
                &rule.init.src,
                format!("field '{}.{field}' in 'init'", entity.symbol),
            );
            check(
                &rule.next.src,
                format!("field '{}.{field}' in 'next'", entity.symbol),
            );
        }
    }
    for event in &ir.events {
        check(&event.when.src, format!("event '{}' guard", event.name));
        for action in &event.actions {
            if let Some(value) = &action.value {
                check(&value.src, format!("event '{}' action value", event.name));
            }
        }
    }
    for option in &ir.options {
        check(
            &option.exercise_when.src,
            format!("option '{}' election", option.name),
        );
        check(
            &option.payoff.src,
            format!("option '{}' payoff", option.name),
        );
    }

    if offences.is_empty() {
        return Ok(());
    }
    Err(EngineError::SeriesReadInLogic(format!(
        "logic cannot read a stream: {}. An event's guard and action values, a field's rule, \
         and an option's election and payoff are all evaluated before any stream has a value, \
         so the read binds nothing. Drive the logic from a field, a curve, `time.*` or \
         `inputs.*`, or read the cash from a stream, a waterfall or the results layer.",
        offences.join("; ")
    )))
}

fn compute_results(ir: &Ir, model_hash: String, config: RunConfig) -> Result<Results, EngineError> {
    refuse_series_reads_in_logic(ir)?;

    // A model may declare its own run modes. Honour a declared Monte Carlo run
    // when the run config does not ask for one, so `run monte_carlo trials N
    // seed S` in source does what it says without a separate config file.
    // An explicit run config still wins.
    let mut config = config;
    if config.monte_carlo.is_none() {
        if let Some(declared) = ir
            .runs
            .iter()
            .find(|run| run.kind == "monte_carlo" && run.trials.is_some_and(|n| n > 0))
        {
            config.monte_carlo = Some(MonteCarloRunConfig {
                trial_count: declared.trials.unwrap_or(1),
                seed: declared.seed.unwrap_or(0),
                distributions: BTreeMap::new(),
            });
        }
    }
    let config = config;

    let base_run = run_deterministic(ir, &config)?;
    let mut warnings = base_run.warnings.clone();

    let deterministic = DeterministicSection {
        status: "ok".to_string(),
        metrics: base_run.metrics.clone(),
        series: base_run.series,
        transitions: base_run.transitions.clone(),
        journal: base_run.journal.clone(),
        annual_rollup: base_run.annual_rollup,
        errors: None,
    };

    let mut scenario_summaries = Vec::new();
    for (name, scenario) in &config.scenarios {
        let mut merged_overrides = config.parameter_overrides.clone();
        for (key, value) in &scenario.parameter_overrides {
            merged_overrides.insert(key.clone(), *value);
        }
        let scenario_run = run_deterministic(
            ir,
            &RunConfig {
                // Run-wide: a scenario varies the deal's drivers and the rate it
                // is valued at, not the arithmetic every scenario shares.
                arithmetic: config.arithmetic,
                discount_rate: scenario.discount_rate.unwrap_or(config.discount_rate),
                as_of: scenario.as_of.clone().or_else(|| config.as_of.clone()),
                parameter_overrides: merged_overrides,
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
        )?;
        warnings.extend(scenario_run.warnings);
        // A scenario is a FULL deterministic run — `run_deterministic` above
        // computed every metric the base run computes. Publishing only NPV
        // threw the rest away: a stress case could not report its IRR, its
        // MoIC, or any per-stream total, and a model whose whole subject is
        // how returns move with leverage had nothing to show for the scenario
        // that varied it.
        //
        // The base run's own metrics are the same map, so scenarios and the
        // deterministic block cannot report different metric sets.
        let mut scenario_metrics = scenario_run.metrics;
        scenario_metrics.insert(
            "model.npv".to_string(),
            Scalar::Money(Money {
                amount: round_amount(scenario_run.npv),
                currency: ir.model.currency.clone(),
            }),
        );
        scenario_summaries.push(ScenarioSummary {
            name: name.clone(),
            metrics: scenario_metrics,
        });
    }

    let scenarios = if scenario_summaries.is_empty() {
        ScenarioSection {
            status: "not_run".to_string(),
            summaries: vec![],
            errors: None,
        }
    } else {
        ScenarioSection {
            status: "ok".to_string(),
            summaries: scenario_summaries,
            errors: None,
        }
    };

    let monte_carlo = if let Some(monte_carlo_config) = &config.monte_carlo {
        let mut trial_summaries = Vec::with_capacity(monte_carlo_config.trial_count as usize);
        let mut npv_values = Vec::with_capacity(monte_carlo_config.trial_count as usize);
        for trial in 0..monte_carlo_config.trial_count {
            let mut trial_overrides = config.parameter_overrides.clone();
            let mut rng_state = splitmix64(
                monte_carlo_config
                    .seed
                    .wrapping_add((trial as u64).wrapping_mul(0x9e3779b97f4a7c15)),
            );
            for (name, distribution) in &monte_carlo_config.distributions {
                let sampled = apply_clip(
                    sample_distribution(&distribution.spec, &mut rng_state),
                    distribution.clip,
                );
                trial_overrides.insert(name.clone(), sampled);
            }
            // In-language assumptions: independent, per-assumption seed
            // streams so adding one assumption never reshuffles another's
            // draws. Run-config overrides above still win on key collision.
            for (name, random) in &ir.assumptions.random {
                let key = format!("inputs.{name}");
                if trial_overrides.contains_key(&key) {
                    continue;
                }
                let spec = ir_distribution_spec(&random.dist)?;
                let mut assumption_rng = splitmix64(
                    monte_carlo_config
                        .seed
                        .wrapping_add(fnv1a(name))
                        .wrapping_add((trial as u64).wrapping_mul(0x9e3779b97f4a7c15)),
                );
                let sampled = apply_clip(
                    sample_distribution(&spec, &mut assumption_rng),
                    random.dist.clip,
                );
                trial_overrides.insert(key, sampled);
            }
            let trial_run = run_deterministic(
                ir,
                &RunConfig {
                    arithmetic: config.arithmetic,
                    discount_rate: config.discount_rate,
                    as_of: config.as_of.clone(),
                    parameter_overrides: trial_overrides,
                    scenarios: BTreeMap::new(),
                    monte_carlo: None,
                    valuation_grain: None,
                },
            )?;
            warnings.extend(trial_run.warnings);
            npv_values.push(trial_run.npv);

            let mut trial_metrics = BTreeMap::new();
            trial_metrics.insert(
                "model.npv".to_string(),
                Scalar::Money(Money {
                    amount: round_amount(trial_run.npv),
                    currency: ir.model.currency.clone(),
                }),
            );
            trial_summaries.push(MonteCarloTrialSummary {
                trial,
                metrics: trial_metrics,
            });
        }

        let aggregates = if npv_values.is_empty() {
            None
        } else {
            Some(MonteCarloAggregates {
                npv: NpvAggregate {
                    mean: round_amount(stats_mean(&npv_values)),
                    median: round_amount(stats_median(&npv_values)),
                    stddev: round_amount(stats_stddev_population(&npv_values)),
                    p_negative: round_amount(probability_negative(&npv_values)),
                },
            })
        };
        let mut metrics = BTreeMap::new();
        if let Some(aggregates_ref) = &aggregates {
            metrics.insert(
                "model.npv".to_string(),
                MetricSummary {
                    r#type: "money".to_string(),
                    mean: Scalar::Money(Money {
                        amount: aggregates_ref.npv.mean,
                        currency: ir.model.currency.clone(),
                    }),
                    stdev: Some(Scalar::Money(Money {
                        amount: aggregates_ref.npv.stddev,
                        currency: ir.model.currency.clone(),
                    })),
                    min: Some(Scalar::Money(Money {
                        amount: round_amount(
                            npv_values.iter().copied().fold(f64::INFINITY, f64::min),
                        ),
                        currency: ir.model.currency.clone(),
                    })),
                    max: Some(Scalar::Money(Money {
                        amount: round_amount(
                            npv_values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        ),
                        currency: ir.model.currency.clone(),
                    })),
                    p01: None,
                    p05: None,
                    p10: None,
                    p25: None,
                    p50: Scalar::Money(Money {
                        amount: aggregates_ref.npv.median,
                        currency: ir.model.currency.clone(),
                    }),
                    p75: None,
                    p90: None,
                    p95: None,
                    p99: None,
                },
            );
        }

        MonteCarloSection {
            status: "ok".to_string(),
            trials: monte_carlo_config.trial_count,
            seed: monte_carlo_config.seed,
            metrics,
            trial_summaries,
            aggregates,
            errors: None,
        }
    } else {
        MonteCarloSection {
            status: "not_run".to_string(),
            trials: 1,
            seed: 0,
            metrics: BTreeMap::new(),
            trial_summaries: vec![],
            aggregates: None,
            errors: None,
        }
    };

    // Hashed over the ledger only — the per-stream, per-period series. Not the
    // metrics: NPV and IRR are folds OF the ledger, so including them would
    // make the hash change for a reason the ledger did not.
    // `domain.*` is excluded on the same argument that excludes the metrics:
    // a subtotal is a fold OF the ledger, so a pack changing how it chooses to
    // subtotal must not make the hash claim the cash moved. What is hashed is
    // the cash and the states that produced it.
    //
    // THE FILTER APPLIES TO THE ROLLUP TOO. It did not, and the rollup gaining
    // kind-aware subtotals moved `ledger_hash` on fifteen goldens whose cash was
    // bit-identical — the hash asserting the ledger changed when only a fold
    // over it had. The exclusion belongs to the argument, not to the field it
    // was first written on, so it is expressed once and applied to both.
    let is_ledger = |key: &str| !key.starts_with("domain.");
    let ledger_only: BTreeMap<&String, &Series> = deterministic
        .series
        .iter()
        .filter(|(key, _)| is_ledger(key))
        .collect();
    let rollup_only: Option<BTreeMap<&String, &Series>> = deterministic
        .annual_rollup
        .as_ref()
        .map(|r| r.series.iter().filter(|(key, _)| is_ledger(key)).collect());
    let ledger_hash = canonical_hash(&serde_json::json!({
        "series": ledger_only,
        "annual_rollup": rollup_only.map(|series| serde_json::json!({ "series": series })),
    }));

    let inputs = {
        let section = InputsSection {
            resolved: base_run.resolved_inputs.clone(),
            streams: ir.stream_inputs.clone(),
            quantiles: ir.quantile_inputs.clone(),
        };
        (!section.resolved.is_empty()
            || !section.streams.is_empty()
            || !section.quantiles.is_empty())
        .then_some(section)
    };

    Ok(Results {
        results_version: "0.3".to_string(),
        model_hash,
        ledger_hash,
        engine: EngineInfo {
            name: "cfdl-engine".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build: None,
        },
        warnings,
        inputs,
        deterministic,
        scenarios,
        monte_carlo,
        domain_metrics: None,
        statements: None,
    })
}

#[derive(Debug, Clone)]
struct DeterministicRunOutput {
    warnings: Vec<String>,
    /// Evaluated `assume` values, carried out so `compute_results` can publish
    /// them without re-evaluating (which would duplicate every warning).
    resolved_inputs: BTreeMap<String, f64>,
    metrics: BTreeMap<String, Scalar>,
    series: BTreeMap<String, Series>,
    npv: f64,
    annual_rollup: Option<AnnualRollupSection>,
    transitions: Vec<TransitionRecord>,
    journal: Vec<JournalEntry>,
}

/// The distinct unresolved names a run's warnings report.
///
/// Every evaluation site formats the error's code into its warning, and
/// `ExprError`'s Display writes `[CODE] message`, so one marker finds them all
/// however the site chose to phrase the rest. Deduplicated because a name that
/// fails once fails every period.
fn unresolved_names(warnings: &[String], declared: &BTreeSet<String>) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for w in warnings {
        if !w.contains(cfdl_expr::EXPR_UNKNOWN_NAME) {
            continue;
        }
        // The message reads `... unknown variable `inputs.x`; using 0.`
        let Some(start) = w.find("unknown variable `") else {
            seen.insert(w.clone());
            continue;
        };
        let rest = &w[start + "unknown variable `".len()..];
        let Some(end) = rest.find('`') else {
            seen.insert(w.clone());
            continue;
        };
        let name = &rest[..end];
        // DECLARED SOMEWHERE IS NOT THE SAME AS BOUND HERE. An input may be
        // declared only as a Monte Carlo distribution, which leaves it unbound
        // in the deterministic pass — `run_dists_full` is exactly that model,
        // and its deterministic run is incidental to the trials it exists to
        // exercise. That is a different condition from a name nothing
        // declares, and only the second is fatal.
        if declared.contains(name) {
            continue;
        }
        seen.insert(format!("`{name}` is not declared"));
    }
    seen.into_iter().collect()
}

/// One stream's series-read facts, extracted before any stream evaluates.
struct StreamDeps {
    /// Calls `series_sum`/`series_avg` anywhere in its amount or guard.
    uses: bool,
    /// At least one of those calls computes its series name at runtime.
    computed: bool,
    /// The literal read patterns, as written — globs included.
    refs: Vec<String>,
}

/// Assign each stream the wave it evaluates in: 0 for streams that read no
/// series, and one past the deepest stream it reads for everything else. The
/// only rejections are the ones no order can satisfy — a circular read, and a
/// read into a stream whose series names are computed at runtime.
fn assign_waves(names: &[&str], deps: &[StreamDeps]) -> Result<Vec<usize>, EngineError> {
    // Resolve each literal pattern to the streams it names, reader -> producers.
    // Matched as SELECTORS, not exact names: `cre.unit.recoveries.*` as written
    // must find `cre.unit.recoveries.suite_100` as lowered.
    let mut edges: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); names.len()];
    for (reader, dep) in deps.iter().enumerate() {
        for pattern in &dep.refs {
            for (producer, name) in names.iter().enumerate() {
                if cfdl_expr::selector_matches(pattern, name) {
                    edges[reader].insert(producer);
                }
            }
        }
    }

    // A computed-name reader cannot be placed in the order (its edges are
    // unknowable), so it evaluates after every literally-named stream — and
    // nothing may read it, because such a read could never be ordered.
    for (reader, edge_set) in edges.iter().enumerate() {
        for &producer in edge_set {
            if deps[producer].computed {
                return Err(EngineError::SeriesCycle(format!(
                    "Stream '{}' reads series '{}', which computes its series names at \
                     runtime, so its place in the evaluation order cannot be determined. \
                     A stream with computed series names always evaluates last and cannot \
                     be read by another stream.",
                    names[reader], names[producer]
                )));
            }
        }
    }

    // Depth-first depth assignment. GRAY means "on the current chain", so
    // reaching a GRAY stream closes a genuine cycle — the one thing that has
    // no evaluation order. The engine refuses it rather than iterating toward
    // a fixed point.
    const WHITE: u8 = 0;
    const GRAY: u8 = 1;
    const BLACK: u8 = 2;
    fn depth_of(
        node: usize,
        names: &[&str],
        deps: &[StreamDeps],
        edges: &[BTreeSet<usize>],
        color: &mut [u8],
        depth: &mut [usize],
        chain: &mut Vec<usize>,
    ) -> Result<usize, EngineError> {
        if color[node] == BLACK {
            return Ok(depth[node]);
        }
        if color[node] == GRAY {
            let start = chain.iter().position(|&n| n == node).unwrap_or(0);
            let mut path: Vec<&str> = chain[start..].iter().map(|&n| names[n]).collect();
            path.push(names[node]);
            return Err(EngineError::SeriesCycle(format!(
                "cyclic series reads: {}. Each read needs the stream it names \
                 finished first, so no evaluation order exists. CFDL refuses a \
                 circular reference rather than iterating it; break the cycle by \
                 removing one of the reads.",
                path.iter()
                    .map(|n| format!("'{n}'"))
                    .collect::<Vec<_>>()
                    .join(" -> ")
            )));
        }
        color[node] = GRAY;
        chain.push(node);
        let mut deepest = 0usize;
        for &producer in &edges[node] {
            deepest = deepest.max(depth_of(producer, names, deps, edges, color, depth, chain)?);
        }
        chain.pop();
        color[node] = BLACK;
        // A reader is never wave 0 even when its reads resolve to nothing: it
        // still receives the sealed store, exactly as the old phase 2 did, so
        // an unresolved read keeps aggregating to zero under W5022 instead of
        // becoming a missing-context warning.
        depth[node] = if deps[node].uses { deepest + 1 } else { 0 };
        Ok(depth[node])
    }

    let mut color = vec![WHITE; names.len()];
    let mut depth = vec![0usize; names.len()];
    let mut chain: Vec<usize> = Vec::new();
    let mut max_literal = 0usize;
    for node in 0..names.len() {
        if deps[node].computed {
            continue;
        }
        let d = depth_of(
            node, names, deps, &edges, &mut color, &mut depth, &mut chain,
        )?;
        max_literal = max_literal.max(d);
    }
    for node in 0..names.len() {
        if deps[node].computed {
            depth[node] = max_literal + 1;
        }
    }
    Ok(depth)
}

fn run_deterministic(ir: &Ir, config: &RunConfig) -> Result<DeterministicRunOutput, EngineError> {
    // Cash horizon vs full evaluation window: the projection tail
    // (`time ... project <n>`) is computed so series_sum/series_avg can read
    // past the horizon (e.g. forward NOI at exit), but contributes nothing to
    // cash results, totals, or NPV.
    let cash_periods = ir.time.periods as usize;
    let total_periods = cash_periods + ir.time.projection as usize;
    let timeline = timeline_dates(&ir.time.start, &ir.time.calendar, total_periods)?;
    let periods = cash_periods;

    let mut warnings = Vec::new();
    let base_inputs = assumption_inputs(ir, &mut warnings)?;
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
    let (state_values, event_sim) =
        simulate_state(ir, config, &timeline, &base_inputs, &mut warnings);
    let transitions = event_sim.transitions.clone();
    // The journal opens with what the state stage did and grows as each later
    // stage acts, so its order is the order the run happened in.
    let mut journal = event_sim.journal.clone();
    // THE MASK DEFAULTS TO TRUE, so "the mask is on" does not mean an event
    // turned it on — a stream nothing ever touched has an all-true mask. Only
    // a period at or after an actual `activate stream` can be an activation
    // that `active when` then refused, so the first such period per stream is
    // the threshold the streams stage measures against.
    let first_activation: BTreeMap<String, usize> = journal
        .iter()
        .filter(|entry| entry.action == "activate_stream" && entry.outcome == "applied")
        .fold(BTreeMap::new(), |mut acc, entry| {
            let slot = acc.entry(entry.target.clone()).or_insert(entry.period);
            *slot = (*slot).min(entry.period);
            acc
        });
    let mut stream_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    // Each stream's placement in its period, published on the series so a
    // consumer holding results.json can recompute the time-weighted metrics
    // the engine reported. `stream_series` is keyed by bare name; so is this.
    let mut stream_offsets: BTreeMap<String, f64> = BTreeMap::new();
    let mut stream_totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut model_series = vec![0.0_f64; cash_periods];
    // Each stream's series paired with where in its period the cash falls;
    // valuation needs both, while reported cash uses model_series alone.
    let mut valued_streams: Vec<(Vec<f64>, f64)> = Vec::new();

    // --- Streams: dependency-ordered waves ---------------------------------
    //
    // Wave 0 is every stream that reads no series. A reader's wave is one past
    // the deepest stream it reads, so every wave evaluates against a store in
    // which everything it references is already finished. The graph is the one
    // the model states: `series_references` extracts each read as written and
    // `selector_matches` resolves it to the streams it names — the same edges
    // the old two-phase guard walked to REJECT any chain of depth two. Sorting
    // them instead gives exactly acyclicity: a genuine circular read is the
    // only rejection left, because a cycle has no evaluation order and the
    // engine does not iterate to convergence (docs/14 §5 — no fixed-point
    // solver). History that shaped this: the guard matched references as
    // SELECTORS, not exact names, because `cre.exit_forward` reads
    // `series_sum("cre.unit.recoveries.*", ...)` and an exact lookup of the
    // pattern found nothing — measured on `mit_rentleg_plaza`, an exit price
    // $116,440 lower with no diagnostic.
    let mut deps: Vec<StreamDeps> = Vec::with_capacity(ir.streams.len());
    for stream in &ir.streams {
        // A STREAM READS SERIES IF *ANY* OF ITS EXPRESSIONS DOES, not just its
        // amount. `active when series_sum(...) > 0` on a stream whose amount
        // happens not to use one was once handed an empty series map, and its
        // guard then failed — warned, evaluated FALSE, and the stream silently
        // produced nothing at all. An expression that fails to compile
        // contributes nothing here; `evaluate_stream` warns about it later.
        let probe = |src: &str| -> (bool, bool) {
            cfdl_expr::compile_expr(src)
                .map(|c| {
                    (
                        cfdl_expr::uses_series(&c),
                        cfdl_expr::has_computed_series_name(&c),
                    )
                })
                .unwrap_or((false, false))
        };
        let (mut uses, mut computed) = probe(&stream.amount.src);
        let mut refs = cfdl_expr::series_references(&stream.amount.src);
        if let Some(guard) = &stream.active_when {
            let (guard_uses, guard_computed) = probe(&guard.src);
            uses |= guard_uses;
            computed |= guard_computed;
            refs.extend(cfdl_expr::series_references(&guard.src));
        }
        deps.push(StreamDeps {
            uses,
            computed,
            refs,
        });
    }
    let stream_names: Vec<&str> = ir.streams.iter().map(|s| s.name.as_str()).collect();
    let waves = assign_waves(&stream_names, &deps)?;
    let max_wave = waves.iter().copied().max().unwrap_or(0);

    let mut full_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for wave in 0..=max_wave {
        // Wave 0 reads nothing and gets no store, and every later wave gets a
        // snapshot sealed when the wave before it finished — wrapped once per
        // wave, not per accrual, so every env in the wave shares one map.
        let snapshot = (wave > 0).then(|| Arc::new(full_series.clone()));
        for (idx, stream) in ir.streams.iter().enumerate() {
            if waves[idx] != wave {
                continue;
            }
            let mut activation_refused: Vec<usize> = Vec::new();
            let values = evaluate_stream(
                ir,
                config,
                stream,
                &timeline,
                &base_inputs,
                &event_sim,
                &state_values,
                snapshot.as_ref(),
                &mut warnings,
                &mut activation_refused,
            )?;
            // ONE ROW PER STREAM, at the first period the refusal bit. An
            // activation persists forward, so a per-period row would repeat
            // the same fact for the rest of the run.
            let activated_at = first_activation.get(&stream.name).copied();
            let activation_refused: Vec<usize> = match activated_at {
                Some(from) => activation_refused
                    .into_iter()
                    .filter(|idx| *idx >= from)
                    .collect(),
                None => Vec::new(),
            };
            if let Some(&first) = activation_refused.first() {
                let count = activation_refused.len();
                journal.push(
                    JournalEntry::new(
                        first,
                        &timeline[first].to_string(),
                        format!("stream:{}", stream.name),
                        "activate_stream",
                        stream.name.clone(),
                        "overridden",
                    )
                    .with_note(format!(
                        "an event activated this stream and its own `active when` was \
                         false for {count} scheduled period(s) from this one; both \
                         gates must pass, so the activation did not turn it on"
                    )),
                );
            }
            warn_if_cash_settles_in_tail(stream, &values, cash_periods, &mut warnings);
            let offset = discount_offset(&stream.schedule, &ir.time.calendar);
            stream_offsets.insert(stream.name.clone(), offset);
            valued_streams.push((values[..cash_periods.min(values.len())].to_vec(), offset));
            record_stream(
                stream,
                &values,
                cash_periods,
                &mut model_series,
                &mut stream_totals,
                &mut stream_series,
            );
            // The FULL (projection-inclusive) values feed later waves.
            full_series.insert(stream.name.clone(), values);
        }
        if wave == 0 {
            // A NAME THAT PRODUCES NOTHING READS AS ZERO, and said nothing at
            // all until now. Same reasoning as the cycle check — a read that
            // can never resolve reported a plausible number — with a softer
            // verdict, because a literal name matching nothing is a pack idiom
            // as well as a typo.
            check_series_names(ir, &mut warnings);
        }
    }

    for (name, values) in &event_sim.option_cash {
        let cash = &values[..cash_periods.min(values.len())];
        for (idx, value) in cash.iter().enumerate() {
            model_series[idx] += *value;
        }
        valued_streams.push((cash.to_vec(), 0.0));
        let total = cash.iter().sum::<f64>();
        stream_totals.insert(format!("option.{name}"), total);
        stream_series.insert(format!("option.{name}"), cash.to_vec());
        // Option exercise cash settles on its exercise date, so it sits at the
        // period's open — matching the 0.0 pushed into valued_streams above.
        stream_offsets.insert(format!("option.{name}"), 0.0);
    }

    // --- Subtotals: the fold layer ------------------------------------------
    //
    // Evaluated after every stream and every option, so the ledger is complete,
    // and in the IR's array order, which is the dependency order the pack
    // declared. A reference can only reach something already computed, which is
    // what makes a cycle unexpressible rather than merely rejected.
    //
    // These live in their OWN maps and are never merged into `stream_series`.
    // That is the same construction the `state.` prefix relies on below, and it
    // is load-bearing: `model_series` was summed from streams alone,
    // `valued_streams` drives NPV and IRR, and `build_annual_rollup` iterates
    // `stream_series`. A subtotal is a fold OF the cash, so counting it as cash
    // would double every number it touches.
    let stream_category: BTreeMap<&str, &str> = ir
        .streams
        .iter()
        .filter_map(|s| s.category.as_deref().map(|c| (s.name.as_str(), c)))
        .collect();
    let mut subtotal_money: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut subtotal_ratio: BTreeMap<String, Vec<Option<f64>>> = BTreeMap::new();

    for spec in &ir.subtotals {
        match spec.op.as_str() {
            "sum" | "negated_sum" => {
                let sign = if spec.op == "negated_sum" { -1.0 } else { 1.0 };
                let mut acc = vec![0.0_f64; cash_periods];
                for (name, values) in &stream_series {
                    // A stream is folded if its CATEGORY is selected, or if it
                    // is named outright. Category first: it is what the pack
                    // meant, and it keeps a subtotal correct when the pack
                    // grows a contract nobody thought to add here.
                    let by_category = stream_category
                        .get(name.as_str())
                        .is_some_and(|c| cfdl_expr::selector_matches_any(&spec.categories, c));
                    let by_name = cfdl_expr::selector_matches_any(&spec.streams, name);
                    if !(by_category || by_name) {
                        continue;
                    }
                    for (t, v) in values.iter().take(cash_periods).enumerate() {
                        acc[t] += sign * v;
                    }
                }
                for referenced in &spec.subtotals {
                    if let Some(src) = subtotal_money.get(referenced) {
                        for (t, v) in src.iter().enumerate() {
                            acc[t] += sign * v;
                        }
                    }
                }
                // Rounded HERE, not just on the way out, so the ratio below
                // divides the same numbers that get published. Two reasons.
                //
                // A fold of signed cash whose flows cancel leaves a residue —
                // about 2e-12 — rather than an exact zero. Dividing that by a
                // real denominator yields ~2.6e-17, whose last bits differ by
                // platform: that shipped, and Windows disagreed with Linux and
                // macOS on one golden.
                //
                // And it makes the published rows self-consistent: a reader can
                // divide the published NOI by the published debt service and
                // get the published coverage ratio, instead of a number that
                // only reconciles against intermediates nobody can see.
                for v in acc.iter_mut() {
                    *v = round_amount(*v);
                }
                subtotal_money.insert(spec.id.clone(), acc);
            }
            // A RUNNING TOTAL, which a per-period fold cannot express.
            //
            // Percent-of-pool-outstanding is cumulative principal over the
            // original balance, and that shape appears wherever a stock is
            // derived from a flow: principal paid to date, cumulative capital
            // called, drawn-to-date on a facility. Every other op answers "what
            // happened in this period"; this one answers "how much so far".
            //
            // Built from the per-period fold rather than beside it, so a
            // cumulative subtotal and the periodic one it accumulates cannot
            // disagree about what they are summing.
            "cumulative" | "negated_cumulative" => {
                let sign = if spec.op == "negated_cumulative" {
                    -1.0
                } else {
                    1.0
                };
                let mut acc = vec![0.0_f64; cash_periods];
                for (name, values) in &stream_series {
                    let by_category = stream_category
                        .get(name.as_str())
                        .is_some_and(|c| cfdl_expr::selector_matches_any(&spec.categories, c));
                    let by_name = cfdl_expr::selector_matches_any(&spec.streams, name);
                    if !(by_category || by_name) {
                        continue;
                    }
                    for (t, v) in values.iter().take(cash_periods).enumerate() {
                        acc[t] += sign * v;
                    }
                }
                for referenced in &spec.subtotals {
                    if let Some(src) = subtotal_money.get(referenced) {
                        for (t, v) in src.iter().enumerate() {
                            acc[t] += sign * v;
                        }
                    }
                }
                let mut running = 0.0;
                for v in acc.iter_mut() {
                    running += *v;
                    *v = round_amount(running);
                }
                subtotal_money.insert(spec.id.clone(), acc);
            }
            "ratio" => {
                let (Some(num_id), Some(den_id)) = (&spec.numerator, &spec.denominator) else {
                    continue;
                };
                let (Some(num), Some(den)) =
                    (subtotal_money.get(num_id), subtotal_money.get(den_id))
                else {
                    continue;
                };
                // A zero denominator publishes `null` and says nothing else.
                // It is not a warning: a coverage ratio is genuinely undefined
                // once a loan matures, and HUD's does at year 14 of 29 — that
                // is the model being right, not a problem. A warning firing on
                // correct models is noise, and it would fail every benchmark,
                // since tools/benchmark-runner.py treats any warning as a
                // failure.
                //
                // Nothing is discarded silently either, which is the standard
                // that would otherwise argue for a warning: the null is IN the
                // series, per period, so a reader sees exactly which periods
                // are undefined and a consumer cannot mistake one for zero.
                let values: Vec<Option<f64>> = (0..cash_periods)
                    .map(|t| {
                        let d = den.get(t).copied().unwrap_or(0.0);
                        (d.abs() > f64::EPSILON).then(|| num.get(t).copied().unwrap_or(0.0) / d)
                    })
                    .collect();
                subtotal_ratio.insert(spec.id.clone(), values);
            }
            _ => {}
        }
    }

    // Waterfalls run last: a priority of payments allocates cash that this
    // period's streams and states have already produced.
    // Computed from streams alone, before any distribution: the quantity a
    // waterfall's `available` binding reads.
    let available_by_entity = stream_cash_by_entity(ir, &stream_series, cash_periods);
    let waterfall_series = run_waterfalls(
        ir,
        &timeline,
        &base_inputs,
        &state_values,
        &event_sim.entity_state,
        &ir_curve_defs(ir),
        &stream_series,
        &available_by_entity,
        config,
        &mut warnings,
        &mut journal,
    );

    let mut series_map = BTreeMap::new();
    for (name, values) in &stream_series {
        series_map.insert(
            format!("stream.{name}"),
            Series::from_values(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &ir.model.currency,
                stream_offsets.get(name).copied(),
                values,
            ),
        );
    }
    // Each waterfall step is a stream, so a priority of payments publishes
    // under the same prefix everything else pays under and needs no special
    // handling downstream.
    for (name, values) in &waterfall_series {
        series_map.insert(
            format!("stream.{name}"),
            Series::from_values(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &ir.model.currency,
                None,
                values,
            ),
        );
    }
    // States and fields, published for inspection and never counted as cash.
    //
    // WHAT KEEPS THEM OUT IS THE COLLECTION THEY ARE NOT IN, not the name they
    // publish under. Every cash consumer reads a stream collection: the WAL and
    // payback weightings look up `stream.<name>` keys, the annual rollup
    // iterates `stream_series`, and `model_series` and `valued_streams` were
    // summed above from streams alone, before this map is written. A value that
    // never entered those cannot reach model.total, model.npv, the IRR or any
    // domain metric.
    //
    // This comment used to say the `state.` prefix was the guard, which stopped
    // being true when a field started publishing under its owning entity —
    // `asset.pool.balance` carries no prefix and is out anyway. Preserving the
    // prefix while breaking the collection boundary would keep the sentence
    // true and the invariant lost, so the boundary is what this names.
    for (name, values) in &state_values {
        // A FIELD PUBLISHES UNDER THE THING THAT OWNS IT. `state.` names a
        // model-level state, and a field is not one — it is `asset.pool.balance`
        // in the model and reads the same in the results.
        let key = if name.matches('.').count() == 2 {
            name.clone()
        } else {
            format!("state.{name}")
        };
        series_map.insert(
            key,
            Series::from_plain(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &values[..periods.min(values.len())],
            ),
        );
    }
    // Subtotals, under their own `domain.` prefix. Money keeps a currency and
    // no offset — a fold spans streams that may settle at different points, so
    // there is no single placement to claim. Ratios are plain numbers, and
    // `null` where the denominator vanishes.
    for (id, values) in &subtotal_money {
        series_map.insert(
            id.clone(),
            Series::from_values(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &ir.model.currency,
                None,
                &values[..periods.min(values.len())],
            ),
        );
    }
    for (id, values) in &subtotal_ratio {
        series_map.insert(
            id.clone(),
            Series::from_optional(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &values[..periods.min(values.len())],
            ),
        );
    }
    series_map.insert(
        "model.net_cash_flow".to_string(),
        Series::from_values(
            &ir.time.calendar,
            &ir.time.start,
            periods as u32,
            &ir.model.currency,
            None,
            &model_series,
        ),
    );

    // ------------------------------------------------------------------
    // Per-entity cash, AGGREGATED BY RELATION rather than by string glob.
    //
    // A cross-stream read matches series by NAME (`series_sum("cre.rent.*")`),
    // which works only when the modeller encoded the hierarchy into the names.
    // The `part_of` relation says it directly, so a building's cash is its
    // units' cash because they are its units — not because someone prefixed
    // them consistently.
    //
    // AN ENTITY WITH NO CHILDREN IS UNAFFECTED: its series is its own streams,
    // which is the pool that models collective behavior directly. The grain
    // stays the modeller's choice.
    //
    // Like a subtotal, this is a fold OF the cash and never counts AS cash: it
    // is excluded from model.net_cash_flow, model.total and NPV, because
    // counting a parent and its children would double what it touches.
    // ------------------------------------------------------------------
    let mut entity_own: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut add_owned = |symbol: &str, values: &[f64]| {
        let slot = entity_own
            .entry(symbol.to_string())
            .or_insert_with(|| vec![0.0; periods]);
        for (idx, value) in values.iter().enumerate().take(periods) {
            slot[idx] += value;
        }
    };
    for stream in &ir.streams {
        if let Some(values) = stream_series.get(&stream.name) {
            add_owned(&stream.owner.symbol, values);
        }
    }
    // A waterfall step's cash belongs to whoever it pays. Without this a
    // priority of payments would move money and no entity's total would show
    // it — the payee is named in the step and would have gone unread.
    for waterfall in &ir.waterfalls {
        for step in &waterfall.steps {
            if let Some(values) = waterfall_series.get(&format!("{}.{}", waterfall.name, step.name))
            {
                add_owned(&step.payee, values);
            }
        }
    }
    // An option is a contract, so its payoff belongs to the asset it is
    // written on — which is why options gained an owner.
    for option in &ir.options {
        if let (Some(owner), Some(values)) = (
            option.owner.as_ref(),
            stream_series.get(&format!("option.{}", option.name)),
        ) {
            add_owned(&owner.symbol, values);
        }
    }

    let parent_of: BTreeMap<&str, &str> = ir
        .entities
        .iter()
        .filter_map(|e| e.parent.as_deref().map(|p| (e.symbol.as_str(), p)))
        .collect();
    let mut entity_rollup: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for entity in &ir.entities {
        entity_rollup
            .entry(entity.symbol.clone())
            .or_insert_with(|| vec![0.0; periods]);
    }
    for (symbol, own) in &entity_own {
        // Walk from the owner up to the root, adding its cash to every
        // ancestor. `visited` bounds the walk even though a cycle is rejected
        // at compile time — this reads IR that may not have come from there.
        let mut visited: BTreeSet<&str> = BTreeSet::new();
        let mut cursor: Option<&str> = Some(symbol.as_str());
        while let Some(current) = cursor {
            if !visited.insert(current) {
                break;
            }
            let slot = entity_rollup
                .entry(current.to_string())
                .or_insert_with(|| vec![0.0; periods]);
            for (idx, value) in own.iter().enumerate().take(periods) {
                slot[idx] += value;
            }
            cursor = parent_of.get(current).copied();
        }
    }
    for (symbol, values) in &entity_rollup {
        series_map.insert(
            format!("entity.{symbol}.net_cash_flow"),
            Series::from_values(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &ir.model.currency,
                None,
                values,
            ),
        );
    }

    let mut metrics = BTreeMap::new();
    for (stream_name, total) in stream_totals {
        metrics.insert(
            format!("stream.{stream_name}.total"),
            Scalar::Money(Money {
                amount: round_amount(total),
                currency: ir.model.currency.clone(),
            }),
        );
    }
    // Rolled up, so the lifetime total agrees with the series above rather
    // than disagreeing with it for any entity that has children.
    for (entity_symbol, values) in &entity_rollup {
        metrics.insert(
            format!("entity.{entity_symbol}.total"),
            Scalar::Money(Money {
                amount: round_amount(values.iter().sum::<f64>()),
                currency: ir.model.currency.clone(),
            }),
        );
    }

    let model_total = model_series.iter().sum::<f64>();
    let ppy = periods_per_year(&ir.time.calendar);
    let per_period_rate = (1.0 + config.discount_rate).powf(1.0 / ppy) - 1.0;
    // The identity grain keeps the original path, byte for byte. Regrouping the
    // sum changes its last bit (measured at 1 ULP), and no published NPV should
    // move because a capability was added that nobody asked for yet.
    let npv = match config.valuation_grain.as_deref() {
        Some("annual") => {
            let grain = Grain::calendar_year(&timeline[..cash_periods.min(timeline.len())]);
            // One bucket is one year, so the rate for a bucket is the ANNUAL
            // rate — not the per-period rate the grid would use.
            npv_at_grain(&valued_streams, config.discount_rate, &grain)
        }
        _ => npv_with_offsets(&valued_streams, per_period_rate),
    };
    metrics.insert(
        "model.total".to_string(),
        Scalar::Money(Money {
            amount: round_amount(model_total),
            currency: ir.model.currency.clone(),
        }),
    );
    metrics.insert(
        "model.npv".to_string(),
        Scalar::Money(Money {
            amount: round_amount(npv),
            currency: ir.model.currency.clone(),
        }),
    );
    if let Some(pp_irr) = irr_with_offsets(&valued_streams) {
        let annual_irr = (1.0 + pp_irr).powf(ppy) - 1.0;
        metrics.insert(
            "model.irr".to_string(),
            Scalar::Number(round_amount(annual_irr)),
        );
    }
    // Engine-universal return metrics: MOIC, payback
    // period, WAL. Domain metrics live in pack metrics.toml files.
    //
    // WAL and payback are measured on the SAME TIME AXIS as discounting: a
    // flow's position is (period + offset), the exponent npv_with_offsets
    // uses. See docs/12_payment_timing.md. So an ordinary annuity's first
    // monthly collection is at 1/12 of a year, not 0 — which is the market
    // definition, and what a prospectus means by "the number of years from
    // the closing date to the related distribution date".
    //
    // Streams net only WITHIN an offset. Two flows in the same period at
    // different points in it are not the same cash at the same moment, so a
    // purchase settling on its date cannot cancel that period's collections.
    // Bucketing by offset and summing inside each bucket reduces exactly to
    // the old net-series computation whenever every stream shares an offset.
    let mut by_offset: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
    for (values, offset) in &valued_streams {
        // f64 is not Ord and these are exact fractions, so quantise to key.
        let key = (offset * 1e9).round() as i64;
        let bucket = by_offset
            .entry(key)
            .or_insert_with(|| vec![0.0; cash_periods]);
        for (idx, value) in values.iter().enumerate() {
            if idx < bucket.len() {
                bucket[idx] += *value;
            }
        }
    }
    // MOIC keeps the whole-model net series: it is a ratio of cash in to cash
    // out over the life, and where inside a period the cash sits does not
    // change how much of it there is. Only the time-weighted metrics below
    // need the offset, so they compute their own totals.
    let total_inflows: f64 = model_series.iter().filter(|v| **v > 0.0).sum();
    let total_outflows: f64 = -model_series.iter().filter(|v| **v < 0.0).sum::<f64>();
    if total_outflows > 0.0 && total_inflows > 0.0 {
        metrics.insert(
            "model.moic".to_string(),
            Scalar::Number(round_amount(total_inflows / total_outflows)),
        );
    }
    // Payback: the first INSTANT at which cumulative net cash flow becomes
    // non-negative, given the model starts cash-negative. Omitted otherwise.
    //
    // Instants, not periods: cash is ordered by (period + offset), so an
    // outlay settling on its date at period 0 precedes collections that fall
    // at the end of that same period. `payback_periods` stays a whole period
    // index, because that is what it names.
    if model_series.first().copied().unwrap_or(0.0) < 0.0 {
        let mut instants: Vec<(f64, usize, f64)> = Vec::new();
        for (key, values) in &by_offset {
            let offset = *key as f64 / 1e9;
            for (idx, value) in values.iter().enumerate() {
                if *value != 0.0 {
                    instants.push((idx as f64 + offset, idx, *value));
                }
            }
        }
        instants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        let mut cumulative = 0.0_f64;
        let mut payback: Option<(f64, usize)> = None;
        for (position, idx, value) in &instants {
            cumulative += *value;
            if cumulative >= 0.0 {
                payback = Some((*position, *idx));
                break;
            }
        }
        if let Some((position, period)) = payback {
            metrics.insert(
                "model.payback_periods".to_string(),
                Scalar::Number(period as f64),
            );
            metrics.insert(
                "model.payback_years".to_string(),
                Scalar::Number(round_amount(position / ppy)),
            );
        }
    }
    // WAL: net-inflow-weighted average life in years, on the discounting axis.
    let mut wal_weighted = 0.0_f64;
    let mut wal_inflows = 0.0_f64;
    for (key, values) in &by_offset {
        let offset = *key as f64 / 1e9;
        for (idx, value) in values.iter().enumerate() {
            if *value > 0.0 {
                wal_weighted += ((idx as f64 + offset) / ppy) * *value;
                wal_inflows += *value;
            }
        }
    }
    if wal_inflows > 0.0 {
        metrics.insert(
            "model.wal_years".to_string(),
            Scalar::Number(round_amount(wal_weighted / wal_inflows)),
        );
    }
    metrics.insert(
        "run.annual_discount_rate".to_string(),
        Scalar::Number(round_amount(config.discount_rate)),
    );
    // Published for downstream metric evaluation (e.g. cfdl-metrics
    // `wal_years`, which needs to convert period indices to years).
    metrics.insert("run.periods_per_year".to_string(), Scalar::Number(ppy));
    if let Some(as_of) = &config.as_of {
        metrics.insert("run.as_of".to_string(), Scalar::String(as_of.to_string()));
    }

    let annual_rollup = if ir.time.calendar == "annual" {
        None
    } else {
        Some(build_annual_rollup(
            &timeline[..cash_periods],
            &stream_series,
            &model_series,
            &ir.model.currency,
            &subtotal_money,
            &ir.subtotals,
        ))
    };

    // A NAME THAT RESOLVED TO NOTHING IS FATAL, per docs/03 §2. Detected here,
    // once, rather than at each of the six evaluation sites: those return a
    // number deep inside a per-period loop, and the useful message names the
    // DISTINCT unresolved names rather than repeating one of them per period.
    //
    // `inputs.` cannot be checked at compile time — an input may be supplied
    // entirely by the run configuration, as `run_dists_full` does — so this is
    // the first layer that knows every source. `time.` is closed and is caught
    // earlier, by E1133.
    let mut declared: BTreeSet<String> = BTreeSet::new();
    for name in base_inputs.keys() {
        declared.insert(format!("inputs.{name}"));
    }
    if let Some(mc) = &config.monte_carlo {
        for name in mc.distributions.keys() {
            declared.insert(if name.starts_with("inputs.") {
                name.clone()
            } else {
                format!("inputs.{name}")
            });
        }
    }
    for scenario in config.scenarios.values() {
        for name in scenario.parameter_overrides.keys() {
            declared.insert(name.clone());
        }
    }
    let unresolved = unresolved_names(&warnings, &declared);
    if !unresolved.is_empty() {
        return Err(EngineError::UnknownName(format!(
            "{} — each read as zero. Declare it, supply it in the run configuration, or correct the name.",
            unresolved.join("; ")
        )));
    }

    Ok(DeterministicRunOutput {
        journal,
        warnings,
        resolved_inputs: base_inputs,
        metrics,
        series: series_map,
        npv,
        annual_rollup,
        transitions,
    })
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod unresolved_name_tests {
    use super::unresolved_names;
    use std::collections::BTreeSet;

    fn warn(name: &str) -> String {
        format!(
            "Stream 'x' amount evaluation failed [{}]: unknown variable `{name}`; using 0.",
            cfdl_expr::EXPR_UNKNOWN_NAME
        )
    }

    #[test]
    fn a_name_nothing_declares_is_fatal() {
        let found = unresolved_names(&[warn("inputs.typo")], &BTreeSet::new());
        assert_eq!(found, vec!["`inputs.typo` is not declared".to_string()]);
    }

    #[test]
    fn a_declared_name_merely_unbound_here_is_not() {
        // An input declared only as a Monte Carlo distribution is unbound in
        // the deterministic pass. `run_dists_full` is that model, and its
        // deterministic run is incidental to the trials it exercises.
        let declared: BTreeSet<String> = ["inputs.n".to_string()].into_iter().collect();
        assert!(unresolved_names(&[warn("inputs.n")], &declared).is_empty());
    }

    #[test]
    fn one_entry_per_distinct_name_however_many_periods() {
        let warnings = vec![warn("inputs.a"), warn("inputs.a"), warn("inputs.b")];
        assert_eq!(unresolved_names(&warnings, &BTreeSet::new()).len(), 2);
    }

    #[test]
    fn an_ordinary_evaluation_failure_is_left_alone() {
        let w = "Stream 'x' amount evaluation failed [EXPR_EVAL]: division by zero; using 0.";
        assert!(unresolved_names(&[w.to_string()], &BTreeSet::new()).is_empty());
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
    /// change was a real behavioural difference or a run-to-run wobble, and a
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
    fn the_same_cash_values_the_same_at_one_convention_whatever_grain_it_was_modelled_on() {
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
                      "category": "financing.debt_service",
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
             "categories": ["financing.debt_service"]},
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

    /// AN ACTION THE ENGINE DOES NOT EXECUTE IS JOURNALED AS `ignored`.
    ///
    /// `activate contract` parses and lowers, and the engine has no contract
    /// runtime yet. It cannot be reached from a model: a contract carries only
    /// its type, so `deactivate contract cre.lease` is
    /// `E1303_UNRESOLVED_CONTRACT_REF` — there is no instance to name (backlog
    /// 7.63, which therefore sequences before 7.40i's runtime). Hand-written IR
    /// is the only way in, and the only way to test that the results say what
    /// happened rather than staying silent.
    #[test]
    fn a_contract_action_is_journaled_as_ignored() {
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

        let results =
            run_from_json_str(ir, RunConfig::default()).expect("an ignored action is not an error");
        let row = results
            .deterministic
            .journal
            .iter()
            .find(|entry| entry.action == "deactivate_contract")
            .expect("the action must appear in the journal even though it did nothing");
        assert_eq!(row.outcome, "ignored");
        assert_eq!(row.target, "cre.lease");
        assert!(
            row.note
                .as_deref()
                .is_some_and(|n| n.contains("contract runtime")),
            "the row must say why it did nothing: {:?}",
            row.note
        );
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
            message.contains("event 'vacate' guard") && message.contains("series_sum"),
            "message should name the site and the call: {message}"
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

        // Legacy and bracket key forms must not be accepted
        let mut legacy = BTreeMap::new();
        legacy.insert("stream.cre.lease.base_rent.amount".to_string(), 99.0);
        let legacy_results = run_from_json_str(
            ir,
            RunConfig {
                arithmetic: cfdl_expr::Mode::Decimal,
                discount_rate: 0.0,
                as_of: None,
                parameter_overrides: legacy,
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
        )
        .expect("run with legacy key");
        let legacy_total = legacy_results
            .deterministic
            .metrics
            .get("stream.cre.lease.base_rent.total")
            .and_then(|s| match s {
                super::Scalar::Money(m) => Some(m.amount),
                _ => None,
            })
            .unwrap_or(0.0);
        // Default amount 10 per period, 2 periods => 20 when legacy key is ignored
        assert!(
            (legacy_total - 20.0).abs() < 1e-9,
            "legacy key must be ignored"
        );

        let mut bracket = BTreeMap::new();
        bracket.insert("stream[\"cre.lease.base_rent\"].amount".to_string(), 99.0);
        let bracket_results = run_from_json_str(
            ir,
            RunConfig {
                arithmetic: cfdl_expr::Mode::Decimal,
                discount_rate: 0.0,
                as_of: None,
                parameter_overrides: bracket,
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
        )
        .expect("run with bracket key");
        let bracket_total = bracket_results
            .deterministic
            .metrics
            .get("stream.cre.lease.base_rent.total")
            .and_then(|s| match s {
                super::Scalar::Money(m) => Some(m.amount),
                _ => None,
            })
            .unwrap_or(0.0);
        assert!(
            (bracket_total - 20.0).abs() < 1e-9,
            "bracket key must be ignored"
        );
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
        let out = run_deterministic(&ir, &RunConfig::default()).expect("resolves");
        assert_eq!(out.resolved_inputs["net_sf"], 8500.0);
        assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    }

    /// Name order would have resolved this one by luck; dependency order
    /// resolves it because it is correct.
    #[test]
    fn order_follows_dependencies_not_names() {
        // `alpha` reads `zulu`, so the alphabetical walk meets it first.
        let ir = ir_with(&[("alpha", "inputs.zulu * 3.0"), ("zulu", "7.0")]);
        let out = run_deterministic(&ir, &RunConfig::default()).expect("resolves");
        assert_eq!(out.resolved_inputs["alpha"], 21.0);
    }

    #[test]
    fn a_circular_derivation_is_refused_with_its_path() {
        let ir = ir_with(&[
            ("gross_sf", "inputs.net_sf / 2.0"),
            ("net_sf", "inputs.gross_sf * 0.85"),
        ]);
        let err = run_deterministic(&ir, &RunConfig::default()).expect_err("no order exists");
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
        let out = run_deterministic(&ir, &RunConfig::default()).expect("resolves");
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
        let out = run_deterministic(&ir, &RunConfig::default()).expect("chain evaluates");
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
        let err = run_deterministic(&ir, &RunConfig::default())
            .expect_err("a circular read has no order");
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
