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
//                  election, at most once), stepped inside the state walk
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
pub(crate) use walk::*;
mod results;
pub use results::*;
mod distributions;
mod fold;
use distributions::*;
mod accounts;
mod occurrence;
mod prepare;
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

/// The document without its VIEWS — what `model_hash` identifies.
///
/// A slice filters and a statement organizes. Neither produces cash, so two
/// users who look at identical results differently are running the same model,
/// and the hash has to say so. A metric is NOT dropped: it is a figure the
/// model claims, asserted by every benchmark's `expected_metrics.json`.
///
/// One key, dropped at the ONE site that hashes. `is_ledger` below is the
/// cautionary case — that rule was written onto one field, missed a second,
/// and moved `ledger_hash` on fifteen goldens whose cash was bit-identical.
/// A single site cannot repeat that, and anything added under `views` is
/// outside the model's identity without this function changing.
fn model_only(document: &Value) -> Value {
    let mut model = document.clone();
    if let Some(object) = model.as_object_mut() {
        object.remove("views");
    }
    model
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

    // ONCE PER MODEL, not once per run: the base run, every scenario and
    // every Monte Carlo trial share one grid, one set of compiled expressions
    // and one set of schedules.
    let mut prep_warnings: Vec<String> = Vec::new();
    let prep = prepare_model(ir, &mut prep_warnings)?;
    let base_run = run_deterministic(ir, &config, &prep)?;
    // Preparation happens once, so its warnings are emitted once — they belong
    // to the run that publishes them rather than being repeated per trial.
    let mut warnings = prep_warnings.clone();
    warnings.extend(base_run.warnings.clone());

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
            &prep,
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
        // ACT IDENTITY -> the period it first occurred, one entry per trial that
        // saw it. Keyed rather than accumulated per trial, because §7.18's
        // objection to a per-trial log is its size: this map is bounded by the
        // model's acts, not by the trial count.
        let mut journal_firsts: BTreeMap<(String, String, String, String), Vec<usize>> =
            BTreeMap::new();
        let mut npv_values = Vec::with_capacity(monte_carlo_config.trial_count as usize);
        // METRIC NAME -> its samples across the trials. Bounded by the model's
        // metric names rather than by the trial count, which is the same size
        // argument §7.18 made about the journal — and the reason a scalar map
        // is carried per trial where a series map is not.
        let mut metric_samples: BTreeMap<String, MetricSamples> = BTreeMap::new();
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
                &prep,
            )?;
            warnings.extend(trial_run.warnings);
            npv_values.push(trial_run.npv);

            // FIRST occurrence per act in THIS trial, so a repeating act
            // contributes one period rather than one per period.
            let mut seen_this_trial: BTreeMap<(String, String, String, String), usize> =
                BTreeMap::new();
            for entry in &trial_run.journal {
                let key = (
                    entry.actor.clone(),
                    entry.action.clone(),
                    entry.target.clone(),
                    entry.outcome.clone(),
                );
                seen_this_trial.entry(key).or_insert(entry.period);
            }
            for (key, period) in seen_this_trial {
                journal_firsts.entry(key).or_default().push(period);
            }

            // EVERY metric the trial computed, not NPV alone (`docs/13`
            // §7.87). A trial IS a complete deterministic run, so this map
            // already holds `model.npv` beside `model.irr`, `model.moic`,
            // every `stream.*.total` and `entity.*.total`, each `domain.*` KPI
            // and every metric the model declared. The scenario path one
            // function up has carried the whole map since it was written; the
            // trial loop had `trial_run.metrics` in scope and built a fresh
            // one-entry map instead, so the figure a stochastic case exists to
            // assert existed in no trial.
            //
            // `entity.*.total` is the join the published entity graph was
            // built for: a trial row keys `entity.asset.tower.total`, and
            // `graph.entities[].symbol` is `asset.tower`, so a per-entity
            // distribution is now readable from results alone — the ownership
            // axis of §7.43, worn stochastically.
            let trial_metrics = trial_run.metrics;
            for (name, value) in &trial_metrics {
                metric_samples
                    .entry(name.clone())
                    .or_default()
                    .observe(value);
            }
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
        // ONE SUMMARY PER METRIC THE TRIALS PUBLISHED (`docs/13` §7.87).
        //
        // This used to be a single hand-built entry for `model.npv`, which is
        // why a declared metric, a MoIC, an IRR or any `domain.*` KPI had no
        // distribution at all. `model.npv` is not special-cased any more — it
        // arrives through the same accumulator as every other name, and comes
        // out with the mean, stdev, min, max and p50 it always had, now with
        // the tails beside them.
        //
        // `MonteCarloAggregates` below still carries the NPV's own four
        // figures, `p_negative` among them: that is the loss probability, a
        // question about a threshold rather than a quantile, and it stays
        // where consumers already read it.
        let metrics: BTreeMap<String, MetricSummary> = metric_samples
            .iter()
            .filter_map(|(name, samples)| Some((name.clone(), samples.summarise()?)))
            .collect();

        let trials_run = monte_carlo_config.trial_count.max(1) as f64;
        let journal_summary: Vec<JournalTrialSummary> = journal_firsts
            .into_iter()
            .map(|((actor, action, target, outcome), mut periods)| {
                periods.sort_unstable();
                let occurred = periods.len();
                JournalTrialSummary {
                    actor,
                    action,
                    target,
                    outcome,
                    trials_occurred: occurred as u32,
                    share: round_share(occurred as f64 / trials_run),
                    first_period: period_distribution(&periods),
                }
            })
            .collect();

        MonteCarloSection {
            status: "ok".to_string(),
            trials: monte_carlo_config.trial_count,
            seed: monte_carlo_config.seed,
            metrics,
            trial_summaries,
            journal: journal_summary,
            aggregates,
            errors: None,
        }
    } else {
        MonteCarloSection {
            status: "not_run".to_string(),
            journal: Vec::new(),
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
    // A LEDGER HAS JOURNAL ENTRIES IN IT. That is what a ledger is, and the
    // trace was in neither hash: two runs with identical series and different
    // journals hashed identically, so the record of what the model DID — the
    // thing `explain` walks — was outside the guarantee. `transitions` joins
    // it on the same argument: which fields changed, and when.
    //
    // THE RUN CONFIGURATION IS DELIBERATELY NOT HERE. Folding it in would give
    // three prepayment speeds over one model three different hashes, and no
    // way to tell whether the cash moved or only a setting. Comparing results
    // across runs is the point; the configuration is an input, and this is the
    // output hash.
    let ledger_hash = canonical_hash(&serde_json::json!({
        "series": ledger_only,
        "annual_rollup": rollup_only.map(|series| serde_json::json!({ "series": series })),
        "journal": deterministic.journal,
        "transitions": deterministic.transitions,
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

    // THE MODEL'S GRAPH, republished from the IR so results stand alone
    // (docs/13 §7.43, §7.91): symbol, family, type, the stable id a layer
    // above assigned, and the part_of parent. What a consumer needed the IR
    // for — attributing a stream to a thing — it now has beside the values.
    // THE CONTRACTS BESIDE THE ENTITIES (docs/40 stage 4): each resolved to
    // its type and master by the compiler, with the streams its pack rules
    // lowered — read off each stream's provenance, which names the contract.
    let graph = (!ir.entities.is_empty() || !ir.contracts.is_empty()).then(|| ResultsGraph {
        entities: ir
            .entities
            .iter()
            .map(|e| GraphEntity {
                symbol: e.symbol.clone(),
                family: e.symbol.split('.').next().unwrap_or_default().to_string(),
                type_id: e.type_id.clone(),
                id: e.fields.get("id").cloned(),
                parent: e.parent.clone(),
            })
            .collect(),
        contracts: ir
            .contracts
            .iter()
            .map(|c| GraphContract {
                name: c.name.clone(),
                type_id: c.type_id.clone(),
                master: c.master.clone(),
                contract_name: c.contract_name.clone(),
                instance: c.instance.clone(),
                subject: c.subject.symbol.clone(),
                parties: c
                    .parties
                    .iter()
                    .map(|p| GraphParty {
                        role: p.role.clone(),
                        master_role: p.master_role.clone(),
                        entity: p.entity.symbol.clone(),
                    })
                    .collect(),
                streams: ir
                    .streams
                    .iter()
                    .filter(|s| lowering_contract(s) == Some(c.name.as_str()))
                    .map(|s| s.name.clone())
                    .chain(ir.waterfalls.iter().flat_map(|w| {
                        w.steps
                            .iter()
                            .filter(|step| step.contract.as_deref() == Some(c.name.as_str()))
                            .map(move |step| format!("{}.{}", w.name, step.name))
                    }))
                    .collect(),
            })
            .collect(),
    });

    Ok(Results {
        results_version: "0.13".to_string(),
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
        graph,
        slices: (!base_run.slices.is_empty()).then(|| base_run.slices.clone()),
    })
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

    /// AN ACTION KIND THE ENGINE DOES NOT KNOW IS JOURNALED AS `ignored`.
    ///
    /// `DeactivateContract` is the case in hand: the action was retired from
    /// the language (`docs/13` §7.73 — a contract is a collection of streams,
    /// and one switch cannot say what forbearance says), so no compiler emits
    /// this kind any more. IR that still carries it must not run silently
    /// wrong; the engine names it in `warnings` and journals it, which is what
    /// this pins. Hand-written IR is the only way in, and the only way to test
    /// that the results say what happened rather than staying silent.
    #[test]
    fn an_unknown_action_kind_is_journaled_as_ignored() {
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
            // The catch-all journals the kind as the IR spelled it, since it
            // has no vocabulary of its own for a kind it does not know.
            .find(|entry| entry.action == "DeactivateContract")
            .expect("the action must appear in the journal even though it did nothing");
        assert_eq!(row.outcome, "ignored");
        assert_eq!(row.target, "");
        assert!(
            row.note
                .as_deref()
                .is_some_and(|n| n.contains("unknown action kind")),
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
        let out = run_deterministic(
            &ir,
            &RunConfig::default(),
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
            &RunConfig::default(),
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
