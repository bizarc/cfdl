use cfdl_expr::{CompiledExpr, ExprEnv, Value as ExprValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub enum EngineError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidDate(String),
    InvalidRunConfig(String),
    Schedule(String),
    /// A cross-stream read that can never resolve. The phase split is an engine
    /// concept, so the check lives with the split rather than being restated in
    /// the compiler where the two could drift.
    PhaseReference(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Io(err) => write!(f, "I/O error: {err}"),
            EngineError::Json(err) => write!(f, "JSON error: {err}"),
            EngineError::PhaseReference(msg) => write!(f, "{msg}"),
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

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub discount_rate: f64,
    pub as_of: Option<Date>,
    pub parameter_overrides: BTreeMap<String, f64>,
    pub scenarios: BTreeMap<String, ScenarioRunConfig>,
    pub monte_carlo: Option<MonteCarloRunConfig>,
    /// The grain to value at — `"annual"`, or `None` for the model grid.
    ///
    /// Named for the `Grain` it produces, because that is what it is choosing.
    /// Two alternatives were considered and rejected: "discounting period"
    /// collides with `period`, which means a model grid period everywhere else
    /// in CFDL, and "cadence" is already taken — `pack.cadences` means which
    /// model calendars a pack's rules lower correctly on, which is a
    /// compatibility list rather than an aggregation frequency.
    ///
    /// `None` means the model grid, which is what every run did before this
    /// existed and is what keeps published NPVs unmoved.
    ///
    /// Naming a coarser grain — currently `"annual"` — sums cash into those
    /// buckets and discounts each once, which is the convention published
    /// sources use ("sum NOI by year, then discount the year"). Until this,
    /// `ppy` came from `ir.time.calendar`, so a model's CALENDAR silently
    /// decided its valuation convention: the same deal built monthly and
    /// annually returned different present values, and `mit_rentleg_plaza`
    /// records that as a 1.3% gap it attributed to the rebuild rather than to
    /// the coupling.
    pub valuation_grain: Option<String>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            discount_rate: 0.0,
            as_of: None,
            parameter_overrides: BTreeMap::new(),
            scenarios: BTreeMap::new(),
            monte_carlo: None,
            valuation_grain: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScenarioRunConfig {
    pub discount_rate: Option<f64>,
    pub as_of: Option<Date>,
    pub parameter_overrides: BTreeMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct MonteCarloRunConfig {
    pub trial_count: u32,
    pub seed: u64,
    pub distributions: BTreeMap<String, ConfiguredDistribution>,
}

/// A run-config distribution and its optional clip bounds.
///
/// Mirrors what `assume x ~ Dist(..., clip=[lo, hi])` expresses in the
/// language, so a driver declared in either place behaves identically.
#[derive(Debug, Clone)]
pub struct ConfiguredDistribution {
    pub spec: DistributionSpec,
    pub clip: Option<[f64; 2]>,
}

#[derive(Debug, Clone)]
pub enum DistributionSpec {
    Fixed { value: f64 },
    Normal { mean: f64, stddev: f64 },
    Uniform { min: f64, max: f64 },
    LogNormal { mu: f64, sigma: f64 },
    Triangular { min: f64, mode: f64, max: f64 },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunConfigFile {
    #[serde(default)]
    deterministic: DeterministicConfigFile,
    #[serde(default)]
    scenarios: BTreeMap<String, ScenarioConfigFile>,
    monte_carlo: Option<MonteCarloConfigFile>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeterministicConfigFile {
    #[serde(rename = "annual_discount_rate")]
    discount_rate: Option<f64>,
    as_of: Option<String>,
    #[serde(default)]
    parameters: BTreeMap<String, f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioConfigFile {
    #[serde(rename = "annual_discount_rate")]
    discount_rate: Option<f64>,
    as_of: Option<String>,
    #[serde(default)]
    parameters: BTreeMap<String, f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MonteCarloConfigFile {
    trial_count: u32,
    seed: u64,
    #[serde(default)]
    distributions: BTreeMap<String, DistributionConfigFile>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum DistributionConfigFile {
    Fixed {
        value: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
    Normal {
        mean: f64,
        // The language spells this `stdev`; `stddev` predates it and is kept
        // so existing run configs keep working.
        #[serde(alias = "stddev")]
        stdev: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
    Uniform {
        min: f64,
        max: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
    LogNormal {
        mu: f64,
        sigma: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
    Triangular {
        min: f64,
        mode: f64,
        max: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
}

pub fn run_from_file(ir_path: &Path, config: RunConfig) -> Result<Results, EngineError> {
    let raw = std::fs::read_to_string(ir_path)?;
    run_from_json_str(&raw, config)
}

pub fn run_config_from_json_file(
    path: &Path,
    fallback_rate: f64,
    fallback_as_of: Option<Date>,
) -> Result<RunConfig, EngineError> {
    let raw = std::fs::read_to_string(path)?;
    run_config_from_json_str(&raw, fallback_rate, fallback_as_of)
}

pub fn run_config_from_json_str(
    raw: &str,
    fallback_rate: f64,
    fallback_as_of: Option<Date>,
) -> Result<RunConfig, EngineError> {
    let config_file: RunConfigFile = serde_json::from_str(raw)?;
    run_config_from_value(config_file, fallback_rate, fallback_as_of)
}

fn run_config_from_value(
    config_file: RunConfigFile,
    fallback_rate: f64,
    fallback_as_of: Option<Date>,
) -> Result<RunConfig, EngineError> {
    let mut config = RunConfig {
        discount_rate: config_file
            .deterministic
            .discount_rate
            .unwrap_or(fallback_rate),
        as_of: fallback_as_of,
        parameter_overrides: config_file.deterministic.parameters,
        scenarios: BTreeMap::new(),
        monte_carlo: None,
        valuation_grain: None,
    };

    if let Some(as_of) = config_file.deterministic.as_of {
        config.as_of = Some(Date::parse(&as_of)?);
    }

    for (name, scenario) in config_file.scenarios {
        let scenario_as_of = match scenario.as_of {
            Some(raw) => Some(Date::parse(&raw)?),
            None => None,
        };
        config.scenarios.insert(
            name,
            ScenarioRunConfig {
                discount_rate: scenario.discount_rate,
                as_of: scenario_as_of,
                parameter_overrides: scenario.parameters,
            },
        );
    }

    if let Some(monte_carlo) = config_file.monte_carlo {
        if monte_carlo.trial_count == 0 {
            return Err(EngineError::InvalidRunConfig(
                "monte_carlo.trial_count must be >= 1".to_string(),
            ));
        }
        let mut distributions = BTreeMap::new();
        for (name, dist) in monte_carlo.distributions {
            let (spec, clip) = match dist {
                DistributionConfigFile::Fixed { value, clip } => {
                    (DistributionSpec::Fixed { value }, clip)
                }
                DistributionConfigFile::Normal { mean, stdev, clip } => {
                    if stdev < 0.0 {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' has negative stdev"
                        )));
                    }
                    (
                        DistributionSpec::Normal {
                            mean,
                            stddev: stdev,
                        },
                        clip,
                    )
                }
                DistributionConfigFile::Uniform { min, max, clip } => {
                    if min > max {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' has min > max"
                        )));
                    }
                    (DistributionSpec::Uniform { min, max }, clip)
                }
                DistributionConfigFile::LogNormal { mu, sigma, clip } => {
                    if sigma < 0.0 {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' has negative sigma"
                        )));
                    }
                    (DistributionSpec::LogNormal { mu, sigma }, clip)
                }
                DistributionConfigFile::Triangular {
                    min,
                    mode,
                    max,
                    clip,
                } => {
                    if !(min <= mode && mode <= max) {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' requires min <= mode <= max"
                        )));
                    }
                    (DistributionSpec::Triangular { min, mode, max }, clip)
                }
            };
            if let Some([lo, hi]) = clip {
                if lo > hi {
                    return Err(EngineError::InvalidRunConfig(format!(
                        "distribution '{name}' has clip lower bound above upper bound"
                    )));
                }
            }
            distributions.insert(name, ConfiguredDistribution { spec, clip });
        }
        config.monte_carlo = Some(MonteCarloRunConfig {
            trial_count: monte_carlo.trial_count,
            seed: monte_carlo.seed,
            distributions,
        });
    }

    Ok(config)
}

pub fn run_from_json_str(raw_ir: &str, config: RunConfig) -> Result<Results, EngineError> {
    let ir_value: Value = serde_json::from_str(raw_ir)?;
    let model_hash = canonical_hash(&ir_value);
    let ir: Ir = serde_json::from_value(ir_value)?;
    compute_results(&ir, model_hash, config)
}

fn compute_results(ir: &Ir, model_hash: String, config: RunConfig) -> Result<Results, EngineError> {
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
        };
        (!section.resolved.is_empty() || !section.streams.is_empty()).then_some(section)
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
}

/// Output of the discrete event/option pre-pass over the master timeline.
struct EventSim {
    /// Per period: entity symbol -> field -> value (state as of that period).
    entity_state: Vec<BTreeMap<String, BTreeMap<String, ExprValue>>>,
    /// Per stream with an event override: per-period active flag.
    stream_active: BTreeMap<String, Vec<bool>>,
    /// Option payoff cash flows: option name -> per-period amounts.
    option_cash: BTreeMap<String, Vec<f64>>,
    /// Every state change an event made, in the order it happened.
    ///
    /// Entity state was UNOBSERVABLE in results: nothing distinguished "the
    /// event fired and its target was misspelled" from "the event never fired",
    /// and a case could not assert that a transition happened at all. The
    /// audit trail is the point — if and when something occurred.
    transitions: Vec<TransitionRecord>,
}

/// One state change: when, to what, from what, and what caused it.
#[derive(Debug, Clone, Serialize)]
pub struct TransitionRecord {
    pub period: usize,
    pub date: String,
    pub entity: String,
    pub field: String,
    /// The value before. Absent when the field had none — which, for a typed
    /// entity with a lifecycle, should not happen, because it opens in its
    /// declared initial state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    pub to: String,
    /// The event that fired. A transition always has a cause.
    pub event: String,
}

/// Evaluate events and options discretely at each time step (spec §12/§13).
/// Events latch: each fires at most once, at the first period its condition
/// is true, in declaration order. Options exercise at most once, when their
/// phase gate (if any) and `exercise when` condition hold, or when forced by
/// an `exercise option` action.
fn simulate_events(
    ir: &Ir,
    config: &RunConfig,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    states: &BTreeMap<String, Vec<f64>>,
    warnings: &mut Vec<String>,
) -> EventSim {
    let periods = timeline.len();
    let mut entity_state: Vec<BTreeMap<String, BTreeMap<String, ExprValue>>> =
        Vec::with_capacity(periods);
    let mut current_state: BTreeMap<String, BTreeMap<String, ExprValue>> = BTreeMap::new();
    // AN ENTITY WITH A LIFECYCLE IS ALWAYS IN EXACTLY ONE STATE, from period 0.
    // Before the ontology there was no declared state space, so a status was
    // null until an event wrote one and `entity.status == "x"` was false for
    // reasons that had nothing to do with the deal.
    for entity in &ir.entities {
        if let Some(initial) = &entity.initial_state {
            current_state
                .entry(entity.symbol.clone())
                .or_default()
                .insert("status".to_string(), ExprValue::String(initial.clone()));
        }
    }
    let mut current_active: BTreeMap<String, bool> = BTreeMap::new();
    let mut stream_active: BTreeMap<String, Vec<bool>> = BTreeMap::new();
    let mut option_cash: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut transitions: Vec<TransitionRecord> = Vec::new();
    let mut event_fired = vec![false; ir.events.len()];
    let mut option_exercised = vec![false; ir.options.len()];
    let mut forced_exercise: Vec<String> = Vec::new();

    let compiled_events: Vec<Option<cfdl_expr::CompiledExpr>> = ir
        .events
        .iter()
        .map(|event| match cfdl_expr::compile_expr(&event.when.src) {
            Ok(compiled) => Some(compiled),
            Err(err) => {
                warnings.push(format!(
                    "Event '{}' trigger compile failed [{}]: {}; event disabled.",
                    event.name, err.code, err.message
                ));
                None
            }
        })
        .collect();
    let compiled_options: Vec<Option<(cfdl_expr::CompiledExpr, cfdl_expr::CompiledExpr)>> = ir
        .options
        .iter()
        .map(|option| {
            let when = cfdl_expr::compile_expr(&option.exercise_when.src);
            let payoff = cfdl_expr::compile_expr(&option.payoff.src);
            match (when, payoff) {
                (Ok(w), Ok(p)) => Some((w, p)),
                (Err(err), _) | (_, Err(err)) => {
                    warnings.push(format!(
                        "Option '{}' expression compile failed [{}]: {}; option disabled.",
                        option.name, err.code, err.message
                    ));
                    None
                }
            }
        })
        .collect();

    for (t, date) in timeline.iter().enumerate() {
        // THE SYNCHRONOUS RULE, MADE EXPLICIT.
        //
        // Every guard in this period reads the SAME frozen pre-state — the
        // entity state as it stood when the period opened. Writes accumulate
        // in `current_state` and become visible at t+1, never at t.
        //
        // That is the Esterel/SCADE discipline the engine already had by
        // accident: `env` was built once before the loop, so nothing could
        // race. It held vacuously, because guards could read no state at all.
        // Now that they can, the property has to be deliberate — otherwise the
        // value of a guard would depend on which event happened to be declared
        // first, and declaration order would become semantics.
        let pre_state = current_state.clone();
        let mut env = build_base_env(ir, config, t, date, base_inputs);
        bind_states(&mut env, states, t);
        bind_all_entity_state(&mut env, &pre_state);
        for (event_idx, event) in ir.events.iter().enumerate() {
            if event_fired[event_idx] {
                continue;
            }
            let Some(when) = &compiled_events[event_idx] else {
                continue;
            };
            if !eval_bool_expr(when, &env, "Event", &event.name, "when", warnings) {
                continue;
            }
            event_fired[event_idx] = true;
            for action in &event.actions {
                match action.kind.as_str() {
                    "SetEntityField" => {
                        let (Some(entity), Some(field), Some(value)) =
                            (&action.entity, &action.field, &action.value)
                        else {
                            warnings.push(format!(
                                "Event '{}' SetEntityField is missing fields; skipped.",
                                event.name
                            ));
                            continue;
                        };
                        match cfdl_expr::compile_expr(&value.src)
                            .and_then(|compiled| cfdl_expr::eval(&compiled, &env))
                        {
                            Ok(v) => {
                                let slot = current_state.entry(entity.symbol.clone()).or_default();
                                let before = slot.get(field).map(describe_value);
                                let after = describe_value(&v);
                                slot.insert(field.clone(), v);
                                // Recorded even when the value does not change:
                                // the log answers "did this event fire", and a
                                // set that wrote the same value still fired.
                                transitions.push(TransitionRecord {
                                    period: t,
                                    date: date.to_string(),
                                    entity: entity.symbol.clone(),
                                    field: field.clone(),
                                    from: before,
                                    to: after,
                                    event: event.name.clone(),
                                });
                            }
                            Err(err) => warnings.push(format!(
                                "Event '{}' set {}.{} failed [{}]: {}; skipped.",
                                event.name, entity.symbol, field, err.code, err.message
                            )),
                        }
                    }
                    "ActivateStream" => {
                        if let Some(stream) = &action.stream {
                            current_active.insert(stream.clone(), true);
                        }
                    }
                    "DeactivateStream" => {
                        if let Some(stream) = &action.stream {
                            current_active.insert(stream.clone(), false);
                        }
                    }
                    "ActivateContract" | "DeactivateContract" => {
                        warnings.push(format!(
                            "Event '{}': contract activation is not executed by the engine yet; action ignored.",
                            event.name
                        ));
                    }
                    "ExerciseOption" => {
                        if let Some(option) = &action.option {
                            forced_exercise.push(option.clone());
                        }
                    }
                    other => warnings.push(format!(
                        "Event '{}': unknown action kind '{other}'; ignored.",
                        event.name
                    )),
                }
            }
        }

        for (option_idx, option) in ir.options.iter().enumerate() {
            if option_exercised[option_idx] {
                continue;
            }
            let Some((when, payoff)) = &compiled_options[option_idx] else {
                continue;
            };
            // THE PHASE GATE BINDS ON A FORCED EXERCISE TOO. `exercisable in`
            // is the window the option EXISTS in — a renewal option outside its
            // window is not an option anyone holds — so an event cannot
            // exercise one that is not exercisable yet. Previously `forced`
            // short-circuited the whole test, so an `exercise option` action
            // fired outside the declared window and against a false condition.
            // What an event legitimately overrides is the option's own
            // ELECTION, which is the `exercise when` below.
            if let Some(phase_name) = &option.exercisable_in_phase {
                let in_phase = ir.phases.iter().any(|phase| {
                    phase.name == *phase_name
                        && Date::parse(&phase.range.start)
                            .map(|start| *date >= start)
                            .unwrap_or(false)
                        && Date::parse(&phase.range.end)
                            .map(|end| *date <= end)
                            .unwrap_or(false)
                });
                if !in_phase {
                    if forced_exercise.iter().any(|name| name == &option.name) {
                        warnings.push(format!(
                            "Option '{}' was forced outside its exercisable phase '{phase_name}'; not exercised.",
                            option.name
                        ));
                    }
                    continue;
                }
            }
            // An option HAS an owner, so `entity.<field>` in its guard means
            // the owner's field — the same thing it means in a stream. Events
            // have no owner and use the qualified path instead.
            let mut option_env = env.clone();
            if let Some(owner) = &option.owner {
                apply_entity_state(&mut option_env, &pre_state, &owner.symbol);
            }
            let env = &option_env;
            let forced = forced_exercise.iter().any(|name| name == &option.name);
            let triggered = forced
                || eval_bool_expr(when, env, "Option", &option.name, "exercise when", warnings);
            if !triggered {
                continue;
            }
            option_exercised[option_idx] = true;
            let mut payoff_values = vec![0.0_f64; periods];
            match cfdl_expr::eval(payoff, env) {
                Ok(ExprValue::Decimal(v)) => payoff_values[t] = v,
                Ok(ExprValue::Int(v)) => payoff_values[t] = v as f64,
                Ok(other) => warnings.push(format!(
                    "Option '{}' payoff returned non-numeric {other:?}; using 0.",
                    option.name
                )),
                Err(err) => warnings.push(format!(
                    "Option '{}' payoff failed [{}]: {}; using 0.",
                    option.name, err.code, err.message
                )),
            }
            option_cash.insert(option.name.clone(), payoff_values);
        }
        forced_exercise.clear();

        entity_state.push(current_state.clone());
        for (stream, active) in &current_active {
            stream_active
                .entry(stream.clone())
                .or_insert_with(|| vec![true; periods])[t] = *active;
        }
    }

    // AN UNEXERCISED OPTION PUBLISHES ZERO, NOT NOTHING.
    //
    // `option_cash` was only written on exercise, so an option that stayed out
    // of the money produced no series at all — a consumer could not tell "did
    // not exercise" from "does not exist", and a case could not assert a
    // NON-exercise, which is half of what an option model has to prove.
    //
    // Seeded after the loop rather than before it so `option_cash` keeps
    // meaning "exercised" while the timeline runs.
    for option in &ir.options {
        option_cash
            .entry(option.name.clone())
            .or_insert_with(|| vec![0.0; periods]);
    }

    EventSim {
        entity_state,
        stream_active,
        option_cash,
        transitions,
    }
}

/// Warn when a stream's cash settles in the projection tail.
///
/// The tail is evaluated so `series_sum` can look forward — a forward-NOI exit
/// reads a year past the sale — but it contributes nothing to cash results,
/// totals or NPV. A stream that *deliberately* runs into the tail to feed a
/// valuation is doing the right thing, and its tail values are meant to be
/// excluded; warning on those would fire on every forward-NOI model.
///
/// What is worth flagging is cash that lands there without the author asking:
/// a schedule ending on the cash horizon whose payment terms then move the
/// last settlement past it. docs/12_payment_timing.md promises a flow is never
/// silently dropped, and before this it was — the amount simply vanished.
fn warn_if_cash_settles_in_tail(
    stream: &IrStream,
    values: &[f64],
    cash_periods: usize,
    warnings: &mut Vec<String>,
) {
    if values.len() <= cash_periods {
        return;
    }
    if stream.schedule.net_days.is_none() && stream.schedule.net_months.is_none() {
        return;
    }
    let stranded: f64 = values[cash_periods..].iter().sum();
    if stranded.abs() < 1e-9 {
        return;
    }
    warnings.push(format!(
        "Stream '{}' settles {:.2} in the projection tail: its payment terms move cash past period {}, and the tail is computed for series lookups only, so that amount is excluded from cash results and NPV. Extend `for <n>` to cover the lag, or shorten the schedule.",
        stream.name, stranded, cash_periods
    ));
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
    let state_values = compute_states(ir, config, &timeline, &base_inputs, &mut warnings);
    let event_sim = simulate_events(
        ir,
        config,
        &timeline,
        &base_inputs,
        &state_values,
        &mut warnings,
    );
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

    // Phase 1: streams without series references. Their FULL (projection-
    // inclusive) values feed the series store for phase 2.
    let mut full_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut phase2: Vec<&IrStream> = Vec::new();
    for stream in &ir.streams {
        // A STREAM IS PHASE 2 IF *ANY* OF ITS EXPRESSIONS READS A SERIES, not
        // just its amount. `active when series_sum(...) > 0` on a stream whose
        // amount happens not to use one was classified phase 1, handed an empty
        // series map, and its guard then failed — warned, evaluated FALSE, and
        // the stream silently produced nothing at all.
        let uses = |src: &str| {
            cfdl_expr::compile_expr(src)
                .map(|compiled| cfdl_expr::uses_series(&compiled))
                .unwrap_or(false)
        };
        let phase2_stream = uses(&stream.amount.src)
            || stream
                .active_when
                .as_ref()
                .is_some_and(|guard| uses(&guard.src));
        if phase2_stream {
            phase2.push(stream);
            continue;
        }
        let values = evaluate_stream(
            ir,
            config,
            stream,
            &timeline,
            &base_inputs,
            &event_sim,
            &state_values,
            None,
            &mut warnings,
        )?;
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
        full_series.insert(stream.name.clone(), values);
    }

    // A PHASE-2 STREAM CANNOT READ ANOTHER PHASE-2 STREAM, and saying so is
    // the difference between a wrong number and a diagnostic.
    //
    // `full_series` is sealed here and never grows, so a phase-2 stream naming
    // another one matches nothing — and `series_aggregate` returns 0 for an
    // unmatched name, deliberately, because a pack rule that lowered no stream
    // should contribute nothing. That default is right for an absent stream and
    // wrong for a present one: the reference can NEVER work, and it reported a
    // plausible zero instead of saying so.
    let phase2_names: BTreeSet<&str> = phase2.iter().map(|s| s.name.as_str()).collect();
    for stream in &phase2 {
        let mut sources = vec![stream.amount.src.as_str()];
        if let Some(guard) = &stream.active_when {
            sources.push(guard.src.as_str());
        }
        for src in sources {
            for referenced in series_references(src) {
                if let Some(other) = phase2_names.get(referenced.as_str()) {
                    return Err(EngineError::PhaseReference(format!(
                        "Stream '{}' reads series '{other}', which itself reads a series. A \
                         cross-stream read can only see streams that read none, so this would \
                         always aggregate to zero.",
                        stream.name
                    )));
                }
            }
        }
    }

    // Wrapped once, not per accrual: every phase-2 env shares this one map.
    let shared_series = Arc::new(full_series);

    // Phase 2: streams calling series_sum/series_avg read phase-1 series
    // (and only those — no phase-2 -> phase-2 references, so no cycles).
    for stream in phase2 {
        let values = evaluate_stream(
            ir,
            config,
            stream,
            &timeline,
            &base_inputs,
            &event_sim,
            &state_values,
            Some(&shared_series),
            &mut warnings,
        )?;
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
    // States, published for inspection. The `state.` prefix keeps them out of
    // reach of every cash consumer by construction: the WAL and payback
    // weightings look up `stream.<name>` keys, the annual rollup iterates
    // `stream_series`, and `model_series` was summed above from streams alone.
    // A state never enters model.total, model.npv or any domain metric.
    for (name, values) in &state_values {
        series_map.insert(
            format!("state.{name}"),
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
    // which is the pool that models collective behaviour directly. The grain
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

    let transitions = event_sim.transitions.clone();
    Ok(DeterministicRunOutput {
        warnings,
        resolved_inputs: base_inputs,
        metrics,
        series: series_map,
        npv,
        annual_rollup,
        transitions,
    })
}

/// Aggregate per-period series values by calendar year.
///
/// Each distinct calendar year present in `timeline` becomes one entry in the
/// output `Series`.  Values for all periods that fall within a given year
/// are summed.  The resulting index uses `calendar = "annual"` and `start =
/// "{first_year}-01-01"`.
/// A bucketing of the model grid into report periods.
///
/// "Grain" rather than a coined term: it is what analytics tooling already
/// calls this — Superset and Looker both offer a Time Grain of
/// day/week/month/quarter/year, meaning exactly "what one row represents".
/// Not to be confused with `pack.cadences`, which is a different thing wearing
/// a similar word: the list of model calendars a pack's rules lower correctly
/// on, rather than a frequency to aggregate at.
///
/// The model grain and the annual rollup were always two bucketings of one
/// mechanism; only one of them was written down. Making it a type means a
/// quarterly statement, an annual rollup and a valuation at a different
/// convention are the same operation with a different partition, rather than
/// three pieces of code that must be kept agreeing.
///
/// `buckets[i]` holds the model-period indices that fall in report period `i`.
/// The identity bucketing — one model period per bucket — is what everything
/// defaults to, which is why nothing moves until something opts in.
#[derive(Debug, Clone)]
pub struct Grain {
    pub calendar: String,
    pub start: String,
    pub buckets: Vec<Vec<usize>>,
    /// One label per bucket, built HERE because this is the last place the
    /// dates exist. A statement is a post-pass with only a `SeriesIndex`, and a
    /// coarse grain's buckets are opaque indices — nothing downstream can say
    /// which year bucket 3 is without rebuilding the timeline again.
    pub labels: Vec<String>,
}

/// Format one bucket's opening date for the calendar it is bucketed at.
fn bucket_label(date: &Date, calendar: &str) -> String {
    match calendar {
        "annual" => format!("{:04}", date.year),
        "quarterly" => format!("{:04}-Q{}", date.year, (date.month - 1) / 3 + 1),
        "daily" => format!("{:04}-{:02}-{:02}", date.year, date.month, date.day),
        // monthly, and anything unrecognised: a year-month is never wrong,
        // only less precise than it could be.
        _ => format!("{:04}-{:02}", date.year, date.month),
    }
}

impl Grain {
    /// One bucket per model period: the grid reporting on itself.
    pub fn identity(timeline: &[Date], calendar: &str, start: &str) -> Self {
        Self {
            calendar: calendar.to_string(),
            start: start.to_string(),
            buckets: (0..timeline.len()).map(|i| vec![i]).collect(),
            labels: timeline.iter().map(|d| bucket_label(d, calendar)).collect(),
        }
    }

    /// One bucket per distinct CALENDAR year — not per model year. A mid-year
    /// start therefore produces a short first bucket, which is what the annual
    /// rollup has always done and what a fiscal reader expects.
    pub fn calendar_year(timeline: &[Date]) -> Self {
        let mut years: Vec<i32> = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for d in timeline {
            if seen.insert(d.year) {
                years.push(d.year);
            }
        }
        let buckets = years
            .iter()
            .map(|&yr| {
                timeline
                    .iter()
                    .enumerate()
                    .filter_map(|(i, d)| (d.year == yr).then_some(i))
                    .collect()
            })
            .collect();
        Self {
            calendar: "annual".to_string(),
            start: years
                .first()
                .map(|y| format!("{y:04}-01-01"))
                .unwrap_or_default(),
            buckets,
            labels: years.iter().map(|y| format!("{y:04}")).collect(),
        }
    }

    /// Build a grain from what a results document already carries.
    ///
    /// A statement is a post-pass and has no timeline — only a `SeriesIndex`.
    /// Reconstructing the dates from `(calendar, start, periods)` is exact,
    /// because that triple is what generated them in the first place.
    ///
    /// `name` is the grain a declaration asked for. `None` or `"period"` gives
    /// the identity bucketing, which is what everything defaults to.
    pub fn from_index(index: &SeriesIndex, name: Option<&str>) -> Self {
        let timeline = timeline_dates(&index.start, &index.calendar, index.periods as usize)
            .unwrap_or_default();
        match name {
            Some("annual") if !timeline.is_empty() => Grain::calendar_year(&timeline),
            _ => Grain::identity(&timeline, &index.calendar, &index.start),
        }
    }

    pub fn is_identity(&self) -> bool {
        self.buckets.iter().all(|b| b.len() <= 1)
    }

    /// Sum a per-period series into this grain's buckets.
    ///
    /// Money buckets by summation. A RATIO does not, and must never be routed
    /// through here: the mean of twelve monthly coverage ratios is not the
    /// annual coverage ratio. A ratio is recomputed from its re-bucketed
    /// numerator and denominator — see `rebucket_subtotals`, whose signature
    /// takes the SPECS rather than the values so that computing the wrong
    /// thing is not the path of least resistance.
    pub fn sum(&self, values: &[f64]) -> Vec<f64> {
        self.buckets
            .iter()
            .map(|b| b.iter().filter_map(|&i| values.get(i)).sum())
            .collect()
    }
}

fn build_annual_rollup(
    timeline: &[Date],
    stream_series: &BTreeMap<String, Vec<f64>>,
    model_series: &[f64],
    currency: &str,
    subtotal_money: &BTreeMap<String, Vec<f64>>,
    subtotal_specs: &[IrSubtotal],
) -> AnnualRollupSection {
    // One caller of Grain rather than its own bucketing. The rollup and a
    // coarser statement ask the same question of the same partition; keeping
    // two implementations of that meant keeping them agreeing.
    //
    // The grain is constructed HERE rather than passed, and the function stays
    // `build_annual_rollup` rather than becoming a general `build_rollup`. It
    // returns `AnnualRollupSection`, and the published schema pins
    // `deterministic.annual_rollup` to `calendar: "annual"` — so a version that
    // accepted any grain could emit quarterly data under a field called
    // `annual_rollup`, which is a worse trade than one saved constructor call.
    //
    // Generality belongs where the grain genuinely varies per output: the
    // statement and valuation paths. If the rollup ever becomes "at whatever
    // grain you asked for", the field name and the schema move with it, and
    // that is a deliberate contract change rather than a rename.
    let grain = Grain::calendar_year(timeline);
    let n_years = grain.buckets.len() as u32;
    let start = grain.start.clone();
    let aggregate = |values: &[f64]| -> Vec<f64> { grain.sum(values) };

    let mut rollup = BTreeMap::new();

    rollup.insert(
        "model.net_cash_flow".to_string(),
        Series::from_values(
            "annual",
            &start,
            n_years,
            currency,
            None,
            &aggregate(model_series),
        ),
    );

    for (name, values) in stream_series {
        rollup.insert(
            format!("stream.{name}"),
            Series::from_values(
                "annual",
                &start,
                n_years,
                currency,
                None,
                &aggregate(values),
            ),
        );
    }

    // Subtotals roll up BY KIND, and that distinction is the whole reason this
    // takes the specs rather than the two value maps.
    //
    // Money folds. A ratio does not: the mean of twelve monthly coverage ratios
    // is not the annual coverage ratio, and the annual ratio is not recoverable
    // from the monthly column at all. So it is recomputed from its numerator and
    // denominator AFTER those have been rolled up — which is only possible
    // because the declaration says what they are.
    //
    // Deliberately keyed off `subtotal_money` for the inputs rather than off the
    // published ratio series, for the same reason cfdl-statement takes specs:
    // given a column of ratios and a grain, averaging them is the obvious thing
    // to write, and it is wrong.
    for (id, values) in subtotal_money {
        rollup.insert(
            id.clone(),
            Series::from_values(
                "annual",
                &start,
                n_years,
                currency,
                None,
                &aggregate(values),
            ),
        );
    }
    for spec in subtotal_specs {
        if spec.op != "ratio" {
            continue;
        }
        let (Some(num_id), Some(den_id)) = (&spec.numerator, &spec.denominator) else {
            continue;
        };
        let (Some(num), Some(den)) = (subtotal_money.get(num_id), subtotal_money.get(den_id))
        else {
            continue;
        };
        let (num, den) = (aggregate(num), aggregate(den));
        let values: Vec<Option<f64>> = num
            .iter()
            .zip(den.iter())
            .map(|(n, d)| (d.abs() > f64::EPSILON).then(|| round_amount(n / d)))
            .collect();
        rollup.insert(
            spec.id.clone(),
            Series::from_optional("annual", &start, n_years, &values),
        );
    }

    AnnualRollupSection { series: rollup }
}

fn stream_direction_sign(stream: &IrStream, warnings: &mut Vec<String>) -> f64 {
    match stream.direction.as_str() {
        "inflow" => 1.0,
        "outflow" => -1.0,
        _ => {
            warnings.push(format!(
                "Stream '{}' has unknown direction '{}'; treating as outflow.",
                stream.name, stream.direction
            ));
            -1.0
        }
    }
}

/// Evaluate one stream over the full timeline (projection tail included).
/// `series` is Some only for phase-2 streams and enables series_sum/avg.
#[allow(clippy::too_many_arguments)]
fn evaluate_stream(
    ir: &Ir,
    config: &RunConfig,
    stream: &IrStream,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    event_sim: &EventSim,
    states: &BTreeMap<String, Vec<f64>>,
    series: Option<&Arc<BTreeMap<String, Vec<f64>>>>,
    warnings: &mut Vec<String>,
) -> Result<Vec<f64>, EngineError> {
    if let Some(lang) = &stream.amount.lang {
        if lang != "cfdl" {
            warnings.push(format!(
                "Stream '{}' amount language '{}' is unsupported; expression is treated as CEL.",
                stream.name, lang
            ));
        }
    }
    let amount_expr = match cfdl_expr::compile_expr(&stream.amount.src) {
        Ok(compiled) => compiled,
        Err(err) => {
            warnings.push(format!(
                "Stream '{}' amount expression compile failed [{}]: {}; using 0.",
                stream.name, err.code, err.message
            ));
            cfdl_expr::compile_expr("0").expect("constant expression compiles")
        }
    };
    let active_src = stream
        .active_when
        .as_ref()
        .map(|expr| expr.src.as_str())
        .unwrap_or("true");
    if let Some(expr) = &stream.active_when {
        if let Some(lang) = &expr.lang {
            if lang != "cfdl" {
                warnings.push(format!(
                    "Stream '{}' active_when language '{}' is unsupported; expression is treated as CEL.",
                    stream.name, lang
                ));
            }
        }
    }
    let active_expr = match cfdl_expr::compile_expr(active_src) {
        Ok(compiled) => compiled,
        Err(err) => {
            warnings.push(format!(
                "Stream '{}' active_when expression compile failed [{}]: {}; using true.",
                stream.name, err.code, err.message
            ));
            cfdl_expr::compile_expr("true").expect("constant expression compiles")
        }
    };

    let schedule_accruals = schedule_accruals(&stream.schedule, timeline)?;
    let event_mask = event_sim.stream_active.get(&stream.name);
    let mut values = vec![0.0_f64; timeline.len()];
    let direction_sign = stream_direction_sign(stream, warnings);
    // A period may receive several accruals — under net-30 both February and
    // March settle in March — so their amounts sum into that period.
    for (pay_idx, accruals) in schedule_accruals.iter().enumerate() {
        for &idx in accruals {
            if let Some(mask) = event_mask {
                if !mask[idx] {
                    continue;
                }
            }
            let mut env =
                build_expr_env(ir, Some(stream), config, idx, &timeline[idx], base_inputs);
            apply_entity_state(&mut env, &event_sim.entity_state[idx], &stream.owner.symbol);
            bind_states(&mut env, states, idx);
            if let Some(series) = series {
                env.series = Arc::clone(series);
            }
            let active_value = eval_bool_expr(
                &active_expr,
                &env,
                "Stream",
                &stream.name,
                "active_when",
                warnings,
            );
            if !active_value {
                continue;
            }
            let amount = if let Some(override_value) = stream_amount_override(config, &stream.name)
            {
                override_value
            } else {
                eval_amount_expr(
                    &amount_expr,
                    &env,
                    &stream.name,
                    &ir.model.currency,
                    warnings,
                )
            };
            // Evaluated against the accrual period, paid in the payment period.
            values[pay_idx] += amount * direction_sign;
        }
    }
    Ok(values)
}

/// Fold a stream's cash-horizon slice into model totals and reporting maps.
fn record_stream(
    stream: &IrStream,
    values: &[f64],
    cash_periods: usize,
    model_series: &mut [f64],
    stream_totals: &mut BTreeMap<String, f64>,
    stream_series: &mut BTreeMap<String, Vec<f64>>,
) {
    let cash = &values[..cash_periods.min(values.len())];
    for (idx, value) in cash.iter().enumerate() {
        model_series[idx] += *value;
    }
    let total = cash.iter().sum::<f64>();
    stream_totals.insert(stream.name.clone(), total);
    stream_series.insert(stream.name.clone(), cash.to_vec());
}

/// Evaluate a boolean guard, reporting a failure against the thing that owns it.
///
/// `subject_kind` exists because this is called from three places with three
/// different kinds of subject — a stream's `active when`, an event's `when`,
/// and an option's `exercise when` — and it used to hardcode "Stream". An
/// option that failed to evaluate was reported as a stream, sending a reader
/// looking for something that does not exist.
fn eval_bool_expr(
    expr: &CompiledExpr,
    env: &ExprEnv,
    subject_kind: &str,
    subject_name: &str,
    slot: &str,
    warnings: &mut Vec<String>,
) -> bool {
    match cfdl_expr::eval(expr, env) {
        Ok(ExprValue::Bool(value)) => value,
        Ok(other) => {
            warnings.push(format!(
                "{subject_kind} '{}' {} expression returned non-bool '{other:?}'; using false.",
                subject_name, slot
            ));
            false
        }
        Err(err) => {
            warnings.push(format!(
                "{subject_kind} '{}' {} evaluation failed [{}]: {}; using false.",
                subject_name, slot, err.code, err.message
            ));
            false
        }
    }
}

fn eval_amount_expr(
    expr: &CompiledExpr,
    env: &ExprEnv,
    stream_name: &str,
    model_currency: &str,
    warnings: &mut Vec<String>,
) -> f64 {
    match cfdl_expr::eval(expr, env) {
        Ok(value) => match value {
            ExprValue::Int(v) => v as f64,
            ExprValue::Decimal(v) => v,
            ExprValue::Money(m) => {
                if !m.currency.eq_ignore_ascii_case(model_currency) {
                    warnings.push(format!(
                        "Stream '{}' amount returned currency '{}', expected '{}'; using amount without FX conversion.",
                        stream_name, m.currency, model_currency
                    ));
                }
                m.amount
            }
            other => {
                warnings.push(format!(
                    "Stream '{}' amount returned non-numeric value '{other:?}'; using 0.",
                    stream_name
                ));
                0.0
            }
        },
        Err(err) => {
            warnings.push(format!(
                "Stream '{}' amount evaluation failed [{}]: {}; using 0.",
                stream_name, err.code, err.message
            ));
            0.0
        }
    }
}

/// Curve declarations from IR as expression-env curve defs. Dates were
/// normalized and validated by the compiler; unparseable points are skipped.
fn ir_curve_defs(ir: &Ir) -> BTreeMap<String, cfdl_expr::CurveDef> {
    let mut out = BTreeMap::new();
    for curve in &ir.curves {
        let points = curve
            .points
            .iter()
            .filter_map(|p| {
                Date::parse(&p.date).ok().map(|d| {
                    (
                        cfdl_expr::Date {
                            year: d.year,
                            month: d.month,
                            day: d.day,
                        },
                        p.value,
                    )
                })
            })
            .collect();
        out.insert(
            curve.name.clone(),
            cfdl_expr::CurveDef {
                interpolation: curve.interpolation.clone(),
                points,
            },
        );
    }
    out
}

/// Entity-independent environment (model/time/cfg/obs/inputs) used for event
/// and option evaluation.
fn build_base_env(
    ir: &Ir,
    config: &RunConfig,
    t: usize,
    date: &Date,
    base_inputs: &BTreeMap<String, f64>,
) -> ExprEnv {
    let mut env = ExprEnv::empty();
    env.model.insert(
        "id".to_string(),
        ExprValue::String(ir.model.name.clone().unwrap_or_else(|| "model".to_string())),
    );
    env.model.insert(
        "base_currency".to_string(),
        ExprValue::Currency(ir.model.currency.clone()),
    );
    env.time.insert("t".to_string(), ExprValue::Int(t as i64));
    env.time.insert(
        "date".to_string(),
        ExprValue::Date(cfdl_expr::Date {
            year: date.year,
            month: date.month,
            day: date.day,
        }),
    );
    env.time
        .insert("phase".to_string(), ExprValue::Optional(None));
    // Periods per year for the model's calendar, so a hand-written model can
    // spread an annual figure without hardcoding a divisor:
    //   amount = inputs.rent_year / time.ppy
    // Packs do NOT use this — a lowering rule resolves its own periods-per-year
    // at compile time, because a rule may pay on its own interval (a monthly
    // coupon on a daily grid needs 12, not 365) and only the compiler can see
    // that. See {{model.periods_per_year}} in cfdl-compile.
    env.time.insert(
        "ppy".to_string(),
        ExprValue::Decimal(periods_per_year(&ir.time.calendar)),
    );
    // Actual calendar days in this period, so an Actual/360 or Actual/365
    // accrual can be expressed. Packs reach it through
    // {{model.accrual_divisor}}; a hand-written model may use it directly.
    env.time.insert(
        "days_in_period".to_string(),
        ExprValue::Decimal(days_in_period(&ir.time.calendar, date)),
    );
    env.curves = ir_curve_defs(ir);
    for (name, value) in base_inputs {
        env.inputs.insert(name.clone(), ExprValue::Decimal(*value));
    }
    for (key, value) in &config.parameter_overrides {
        if let Some(stripped) = key.strip_prefix("cfg.") {
            insert_cfg_value(&mut env.cfg, stripped, *value);
        } else if let Some(stripped) = key.strip_prefix("obs.") {
            insert_cfg_value(&mut env.obs, stripped, *value);
        } else if let Some(stripped) = key.strip_prefix("inputs.") {
            env.inputs
                .insert(stripped.to_string(), ExprValue::Decimal(*value));
        }
    }
    env
}

/// Expose an entity's event-driven state to expressions, both as
/// `entity.state.<field>` and directly as `entity.<field>` (spec §12.3).
/// Bind `state.<name>` to each declared state's value AT period `idx`.
///
/// Extracted so a stream and an option bind the SAME period by construction
/// rather than by two copies agreeing. `prev_states`/`prev_self` are left
/// empty, so `prev` is not merely rejected outside a recurrence — it is not
/// there to be found. See docs/14_state_and_recurrence.md.
/// The series names an expression reads, as written.
///
/// Only literal first arguments — `series_sum("a.b", ...)` — which is what a
/// cross-stream read is. A computed name is not addressed here and is left to
/// the runtime, where it still returns 0 for an unmatched name.
fn series_references(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    for func in ["series_sum", "series_avg"] {
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(func) {
            let after = from + rel + func.len();
            from = after;
            let mut i = after;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'(' {
                continue;
            }
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'"' {
                continue;
            }
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            if i <= bytes.len() {
                out.push(src[start..i].to_string());
            }
        }
    }
    out
}

/// Render a state value for the transition log.
///
/// A lifecycle state is text; a numeric or boolean field is rendered as
/// written. The log is for reading and for asserting on, so the rendering is
/// stable rather than clever.
fn describe_value(value: &ExprValue) -> String {
    match value {
        ExprValue::String(text) => text.clone(),
        ExprValue::Bool(flag) => flag.to_string(),
        ExprValue::Int(n) => n.to_string(),
        ExprValue::Decimal(n) => round_amount(*n).to_string(),
        other => format!("{other:?}"),
    }
}

fn bind_states(env: &mut ExprEnv, states: &BTreeMap<String, Vec<f64>>, idx: usize) {
    env.states = states
        .iter()
        .filter_map(|(name, values)| {
            values
                .get(idx)
                .map(|v| (name.clone(), ExprValue::Decimal(*v)))
        })
        .collect();
}

/// Bind every entity's state under its qualified path — `entity.asset.tower.status`.
///
/// A stream reads `entity.<field>` relative to its owner, which works because a
/// stream HAS one. An event does not, so the qualified form is how an event
/// guard names the thing it is asking about. An option has an owner (it is a
/// contract), so it gets both.
fn bind_all_entity_state(
    env: &mut ExprEnv,
    state_by_entity: &BTreeMap<String, BTreeMap<String, ExprValue>>,
) {
    for (symbol, fields) in state_by_entity {
        let Some((namespace, name)) = symbol.split_once('.') else {
            continue;
        };
        let inner = ExprValue::Map(fields.clone());
        match env.entity.get_mut(namespace) {
            Some(ExprValue::Map(ns_map)) => {
                ns_map.insert(name.to_string(), inner);
            }
            _ => {
                let mut ns_map = BTreeMap::new();
                ns_map.insert(name.to_string(), inner);
                env.entity
                    .insert(namespace.to_string(), ExprValue::Map(ns_map));
            }
        }
    }
}

fn apply_entity_state(
    env: &mut ExprEnv,
    state_by_entity: &BTreeMap<String, BTreeMap<String, ExprValue>>,
    owner_symbol: &str,
) {
    let Some(state) = state_by_entity.get(owner_symbol) else {
        return;
    };
    for (field, value) in state {
        env.entity.insert(field.clone(), value.clone());
    }
    env.entity
        .insert("state".to_string(), ExprValue::Map(state.clone()));
}

/// Evaluate every declared state over the whole evaluation window.
///
/// One pass per period, all states together, before any stream is touched.
/// Period 0 takes `init`; every later period takes `next` with `prev` bound to
/// the state's own previous value and `prev.<name>` to another state's.
///
/// THE INVARIANT LIVES HERE. The env handed to `next` carries `prev_states`
/// and `prev_self` and leaves `states` EMPTY, so a `state.<name>` read inside a
/// recurrence finds nothing rather than being rejected by a check. Every value
/// a state can see comes from the completed `t-1` column, so no reference can
/// close a cycle and declaration order carries no meaning — states may even
/// reference each other mutually. See docs/14_state_and_recurrence.md.
///
/// The window includes the projection tail so a stream reading forward finds
/// states populated.
fn compute_states(
    ir: &Ir,
    config: &RunConfig,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    warnings: &mut Vec<String>,
) -> BTreeMap<String, Vec<f64>> {
    let mut values: BTreeMap<String, Vec<f64>> = ir
        .states
        .iter()
        .map(|st| (st.name.clone(), vec![0.0; timeline.len()]))
        .collect();
    if ir.states.is_empty() {
        return values;
    }

    // Compiled once per state, not once per state per period. This loop is the
    // only place a state's source is evaluated and it runs `states x periods`
    // times — `x trials` under Monte Carlo.
    struct Prepared<'a> {
        state: &'a IrState,
        init: CompiledExpr,
        next: CompiledExpr,
        /// Model periods on which the recurrence STEPS. `None` means every
        /// period, which is what a state with no `schedule` clause means.
        ticks: Option<Vec<bool>>,
        /// The first tick. `init` is the value AT the first tick, not at model
        /// period 0 — the base case belongs to the recurrence's own clock.
        ///
        /// A quarterly schedule on a monthly book accrues at periods 2, 5, 8,
        /// where the payment index is 0, 1, 2. Stepping on the first accrual
        /// would put F(1) where the first payment reads F(0) — an off-by-one
        /// against every published schedule. With no `schedule` this is 0, so
        /// an unscheduled state behaves exactly as it always has.
        first_tick: usize,
    }

    let zero = cfdl_expr::compile_expr("0").expect("constant expression compiles");
    let mut prepared: Vec<Prepared> = Vec::with_capacity(ir.states.len());
    for state in &ir.states {
        let mut compile = |src: &str, clause: &str| match cfdl_expr::compile_expr(src) {
            Ok(compiled) => compiled,
            Err(err) => {
                warnings.push(format!(
                    "State '{}' {clause} expression compile failed [{}]: {}; using 0.",
                    state.name, err.code, err.message
                ));
                zero.clone()
            }
        };
        let init = compile(&state.init.src, "init");
        let next = compile(&state.next.src, "next");

        // A state's clock is its own, exactly as a stream's is. Without this a
        // pool carried on a daily book but paying monthly would compound its
        // hazard 365 times a year instead of 12.
        let ticks = match &state.schedule {
            None => None,
            Some(schedule) => {
                let mut slots = vec![Vec::new(); timeline.len()];
                match apply_schedule_indices(schedule, timeline, &mut slots) {
                    // The ACCRUAL periods, not the settlement periods.
                    // `apply_schedule_indices` files each accrual under the
                    // period its cash settles in, and a stream's amount is
                    // evaluated at the accrual — which is also where
                    // `{{time.elapsed_periods}}` is counted. Ticking on
                    // settlement instead puts the recurrence a whole payment
                    // interval away from the index that reads it.
                    Ok(()) => {
                        let mut ticks = vec![false; timeline.len()];
                        for accruals in &slots {
                            for &idx in accruals {
                                ticks[idx] = true;
                            }
                        }
                        Some(ticks)
                    }
                    Err(err) => {
                        warnings.push(format!(
                            "State '{}' schedule could not be resolved: {err}; stepping every period.",
                            state.name
                        ));
                        None
                    }
                }
            }
        };
        let first_tick = ticks
            .as_ref()
            .and_then(|t: &Vec<bool>| t.iter().position(|on| *on))
            .unwrap_or(0);
        prepared.push(Prepared {
            state,
            init,
            next,
            ticks,
            first_tick,
        });
    }

    for (t, date) in timeline.iter().enumerate() {
        // Snapshot the previous column before writing this one, so every state
        // in this period sees the same completed history regardless of order.
        let previous: BTreeMap<String, ExprValue> = if t == 0 {
            BTreeMap::new()
        } else {
            values
                .iter()
                .map(|(name, v)| (name.clone(), ExprValue::Decimal(v[t - 1])))
                .collect()
        };

        for entry in &prepared {
            let name = &entry.state.name;
            // Between ticks, and outside the schedule's window, a state HOLDS.
            // It does not fall to zero — that is what separates a schedule from
            // `active when`, and why `active when` is deliberately absent here.
            // See docs/14_state_and_recurrence.md.
            let steps = t > entry.first_tick && entry.ticks.as_ref().is_none_or(|ticks| ticks[t]);
            if t > 0 && !steps {
                if let Some(slot) = values.get_mut(name) {
                    slot[t] = slot[t - 1];
                }
                continue;
            }

            let mut env = build_expr_env(ir, None, config, t, date, base_inputs);
            let (compiled, clause) = if t == 0 {
                (&entry.init, "init")
            } else {
                env.prev_states = previous.clone();
                env.prev_self = previous.get(name).cloned();
                (&entry.next, "next")
            };
            match cfdl_expr::eval(compiled, &env) {
                Ok(ExprValue::Decimal(d)) => {
                    if let Some(slot) = values.get_mut(name) {
                        slot[t] = d;
                    }
                }
                Ok(other) => warnings.push(format!(
                    "State '{name}' {clause} evaluated to {other:?}, which is not a number; using 0."
                )),
                Err(err) => warnings.push(format!(
                    "State '{name}' {clause} evaluation failed: {err}; using 0."
                )),
            }
        }
    }
    values
}

fn build_expr_env(
    ir: &Ir,
    stream: Option<&IrStream>,
    config: &RunConfig,
    t: usize,
    date: &Date,
    base_inputs: &BTreeMap<String, f64>,
) -> ExprEnv {
    let mut env = ExprEnv::empty();
    env.model.insert(
        "id".to_string(),
        ExprValue::String(ir.model.name.clone().unwrap_or_else(|| "model".to_string())),
    );
    env.model.insert(
        "base_currency".to_string(),
        ExprValue::Currency(ir.model.currency.clone()),
    );

    env.time.insert("t".to_string(), ExprValue::Int(t as i64));
    env.time.insert(
        "date".to_string(),
        ExprValue::Date(cfdl_expr::Date {
            year: date.year,
            month: date.month,
            day: date.day,
        }),
    );
    env.time
        .insert("phase".to_string(), ExprValue::Optional(None));
    // Periods per year for the model's calendar, so a hand-written model can
    // spread an annual figure without hardcoding a divisor:
    //   amount = inputs.rent_year / time.ppy
    // Packs do NOT use this — a lowering rule resolves its own periods-per-year
    // at compile time, because a rule may pay on its own interval (a monthly
    // coupon on a daily grid needs 12, not 365) and only the compiler can see
    // that. See {{model.periods_per_year}} in cfdl-compile.
    env.time.insert(
        "ppy".to_string(),
        ExprValue::Decimal(periods_per_year(&ir.time.calendar)),
    );
    // Actual calendar days in this period, so an Actual/360 or Actual/365
    // accrual can be expressed. Packs reach it through
    // {{model.accrual_divisor}}; a hand-written model may use it directly.
    env.time.insert(
        "days_in_period".to_string(),
        ExprValue::Decimal(days_in_period(&ir.time.calendar, date)),
    );

    env.entity.insert(
        "id".to_string(),
        ExprValue::String(stream.map_or_else(String::new, |s| s.owner.symbol.clone())),
    );
    env.entity.insert(
        "name".to_string(),
        ExprValue::String(stream.map_or_else(String::new, |s| s.owner.symbol.clone())),
    );
    env.entity
        .insert("state".to_string(), ExprValue::Map(BTreeMap::new()));
    env.curves = ir_curve_defs(ir);

    for (name, value) in base_inputs {
        env.inputs.insert(name.clone(), ExprValue::Decimal(*value));
    }
    for (key, value) in &config.parameter_overrides {
        if let Some(stripped) = key.strip_prefix("cfg.") {
            insert_cfg_value(&mut env.cfg, stripped, *value);
        } else if let Some(stripped) = key.strip_prefix("obs.") {
            insert_cfg_value(&mut env.obs, stripped, *value);
        } else if let Some(stripped) = key.strip_prefix("inputs.") {
            env.inputs
                .insert(stripped.to_string(), ExprValue::Decimal(*value));
        }
    }
    env
}

/// Resolve `assume` values from IR: constants evaluate their expression;
/// random assumptions contribute their deterministic central value (Monte
/// Carlo trials override these via `inputs.<name>` parameter overrides).
fn assumption_inputs(
    ir: &Ir,
    warnings: &mut Vec<String>,
) -> Result<BTreeMap<String, f64>, EngineError> {
    let mut inputs = BTreeMap::new();
    let empty_env = ExprEnv::empty();
    for (name, constant) in &ir.assumptions.constants {
        match cfdl_expr::compile_expr(&constant.expr.src)
            .and_then(|compiled| cfdl_expr::eval(&compiled, &empty_env))
        {
            Ok(ExprValue::Decimal(v)) => {
                inputs.insert(name.clone(), v);
            }
            Ok(ExprValue::Int(v)) => {
                inputs.insert(name.clone(), v as f64);
            }
            Ok(other) => warnings.push(format!(
                "Assumption '{name}' evaluated to non-numeric {other:?}; ignoring."
            )),
            Err(err) => warnings.push(format!(
                "Assumption '{name}' failed to evaluate [{}]: {}; ignoring.",
                err.code, err.message
            )),
        }
    }
    for (name, random) in &ir.assumptions.random {
        let spec = ir_distribution_spec(&random.dist)?;
        inputs.insert(
            name.clone(),
            apply_clip(central_value(&spec), random.dist.clip),
        );
    }
    Ok(inputs)
}

fn stream_amount_override(config: &RunConfig, stream_name: &str) -> Option<f64> {
    let key = format!("stream.{stream_name}:amount");
    config.parameter_overrides.get(&key).copied()
}

fn insert_cfg_value(map: &mut BTreeMap<String, ExprValue>, path: &str, value: f64) {
    let mut segments = path.split('.').filter(|part| !part.is_empty());
    let Some(first) = segments.next() else {
        return;
    };
    let rest = segments.collect::<Vec<_>>();
    if rest.is_empty() {
        map.insert(first.to_string(), ExprValue::Decimal(value));
        return;
    }
    let entry = map
        .entry(first.to_string())
        .or_insert_with(|| ExprValue::Map(BTreeMap::new()));
    insert_cfg_into_value(entry, &rest, value);
}

fn insert_cfg_into_value(slot: &mut ExprValue, path: &[&str], value: f64) {
    if path.is_empty() {
        *slot = ExprValue::Decimal(value);
        return;
    }
    if !matches!(slot, ExprValue::Map(_)) {
        *slot = ExprValue::Map(BTreeMap::new());
    }
    let ExprValue::Map(map) = slot else {
        return;
    };
    let head = path[0];
    let tail = &path[1..];
    if tail.is_empty() {
        map.insert(head.to_string(), ExprValue::Decimal(value));
        return;
    }
    let entry = map
        .entry(head.to_string())
        .or_insert_with(|| ExprValue::Map(BTreeMap::new()));
    insert_cfg_into_value(entry, tail, value);
}

/// For each timeline period, the period whose amount is paid there.
///
/// `None` means no payment. For an annuity due the two indices coincide. For
/// an ordinary annuity the payment lands one interval after the period that
/// earned it, so the amount must still be evaluated against the earning
/// period — `time.t` inside the amount expression refers to accrual, not to
/// settlement.
fn schedule_accruals(
    schedule: &IrSchedule,
    timeline: &[Date],
) -> Result<Vec<Vec<usize>>, EngineError> {
    let mut out = vec![Vec::new(); timeline.len()];
    apply_schedule_indices(schedule, timeline, &mut out)?;
    Ok(out)
}

/// Calendar days in the period beginning `date` on `calendar`.
///
/// Derived from the calendar rather than the timeline so it is available
/// wherever the expression environment is built, including the projection
/// tail. Actual days, not a nominal 30 — that is the whole point: an
/// Actual/360 accrual pays more in a 31-day month than a 30-day one.
fn days_in_period(calendar: &str, date: &Date) -> f64 {
    let next = match calendar {
        "daily" => return 1.0,
        "monthly" => date.add_months(1),
        "quarterly" => date.add_months(3),
        "annual" => date.add_months(12),
        _ => date.add_months(1),
    };
    days_between(date, &next).max(1) as f64
}

fn periods_per_year(calendar: &str) -> f64 {
    match calendar {
        "daily" => 365.0,
        "monthly" => 12.0,
        "quarterly" => 4.0,
        "annual" => 1.0,
        _ => 1.0,
    }
}

/// How far through its period a payment falls, per docs/12_payment_timing.md.
///
/// A payment belongs to the period that earned it; this says where inside that
/// period the cash sits, and so how far it is discounted. One mechanism covers
/// every case — an annuity due sits at the start, an ordinary annuity at the
/// end, and a day rule at its own point in between.
fn discount_offset(schedule: &IrSchedule, calendar: &str) -> f64 {
    // A one-shot flow happens on its stated date, not at the end of the
    // period containing it: a purchase on 2026-01 is settled then, so it is
    // not discounted for a period it never waited through.
    //
    // Unless it says otherwise. That default is right for an acquisition and
    // wrong for a disposal: a reversion is taken at the END of the holding
    // period, so a year-5 sale must discount five periods rather than four.
    // The date names the period; `at_period_end` says where in it the cash
    // falls. On a monthly model the gap is one month; on an annual one it is a
    // whole year, and 9% of the reversion at 12%.
    if schedule.kind == "OnDate" {
        // Three placements, same axis: the stated date's period opens (the
        // default, right for an acquisition), closes (`at_period_end`, right
        // for a disposal), or is treated as arriving evenly across it (`mid`).
        return if schedule.mid {
            0.5
        } else if schedule.at_period_end {
            1.0
        } else {
            0.0
        };
    }
    if schedule.due {
        return 0.0;
    }
    if schedule.mid {
        return 0.5;
    }
    match schedule.on_rule.as_ref() {
        // `on day <n>`: n days into the period, so the divisor is the period's
        // own length. It was a literal 30 until an annual model wanted the
        // mid-period convention and got it from `on day 15` — which is right
        // only on a monthly grid. On a quarterly or annual calendar `day 15`
        // is 15 days into a quarter or a year, not half of one, and on a daily
        // calendar the payment date IS the period, so it clamps to its end.
        Some(rule) if rule.kind == "DayOfMonth" => {
            (rule.day.clamp(1, 31) as f64 / (365.0 / periods_per_year(calendar))).min(1.0)
        }
        // End of month is the period end, same as the default.
        _ => 1.0,
    }
}

/// Present value of streams that each carry their own position in period.
///
/// `v / (1+r)^(t + offset)` factorises to `[v / (1+r)^offset] / (1+r)^t`, so a
/// stream's offset is a constant scale on its whole series.
fn npv_with_offsets(streams: &[(Vec<f64>, f64)], rate: f64) -> f64 {
    let mut total = 0.0_f64;
    for (values, offset) in streams {
        let scale = (1.0 + rate).powf(-offset);
        for (i, value) in values.iter().enumerate() {
            total += value * scale / (1.0 + rate).powi(i as i32);
        }
    }
    total
}

/// Present value at a stated GRAIN: sum the cash into the grain's buckets
/// first, then discount each bucket once.
///
/// This is the order practitioners use — sum NOI by year, then discount the
/// year — and the order matters. `npv_with_offsets` above discounts each
/// stream-period individually and accumulates, which is the same answer only
/// when the grain IS the model grid.
///
/// Grouping is by `(bucket, offset)`, not by bucket alone. A discount factor
/// depends only on position and offset, so summing within a `(bucket, offset)`
/// group and discounting once is MATHEMATICALLY equal to the per-stream
/// accumulation at model grain, including for models whose streams settle at
/// different points in a period. Collapsing the offset dimension would change
/// every mixed-offset model, which is why it is not collapsed.
///
/// Mathematically equal is not bit-equal: float addition is not associative,
/// and regrouping the sum moves the last bit — measured at 1 ULP on a mixed-
/// offset probe. So the identity grain does NOT route through here. The default
/// path stays `npv_with_offsets` exactly as it was, and this function serves
/// callers that ask for a different grain. That keeps the promise that nothing
/// moves until something opts in, rather than re-blessing every NPV in the
/// golden suite for a change of summation order.
///
/// At a coarser grain the sub-bucket offsets do collapse, which is exactly what
/// an annual convention asserts: MIT OCW 11.431J's own footnote says "assumes
/// first cash flow occurs 1 year from present".
fn npv_at_grain(streams: &[(Vec<f64>, f64)], rate_per_bucket: f64, grain: &Grain) -> f64 {
    // (bucket index, quantised offset) -> summed cash. The quantisation mirrors
    // `by_offset` used for WAL and payback, so one convention describes both.
    let mut grouped: BTreeMap<(usize, i64), f64> = BTreeMap::new();
    for (values, offset) in streams {
        let key_offset = (offset * 1e9).round() as i64;
        for (bucket_idx, members) in grain.buckets.iter().enumerate() {
            let mut sum = 0.0_f64;
            for &i in members {
                if let Some(v) = values.get(i) {
                    sum += *v;
                }
            }
            if sum != 0.0 {
                *grouped.entry((bucket_idx, key_offset)).or_insert(0.0) += sum;
            }
        }
    }
    let mut total = 0.0_f64;
    for ((bucket_idx, key_offset), sum) in grouped {
        let offset = key_offset as f64 / 1e9;
        total += sum / (1.0 + rate_per_bucket).powf(bucket_idx as f64 + offset);
    }
    total
}

/// IRR over offset-carrying streams: the rate at which their present value is
/// zero. Bisection, because the basis is rebuilt for each candidate rate.
fn irr_with_offsets(streams: &[(Vec<f64>, f64)]) -> Option<f64> {
    let f = |r: f64| npv_with_offsets(streams, r);
    let (mut lo, mut hi) = (-0.9999_f64, 10.0_f64);
    let (mut f_lo, f_hi) = (f(lo), f(hi));
    if f_lo.is_nan() || f_hi.is_nan() || f_lo * f_hi > 0.0 {
        return None;
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let f_mid = f(mid);
        if f_mid.abs() < 1e-10 {
            return Some(mid);
        }
        if f_lo * f_mid < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            f_lo = f_mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Advance a date by one interval.
fn step_once(d: &Date, interval: &str) -> Date {
    match interval {
        "daily" => d.add_days(1),
        "weekly" => d.add_days(7),
        "quarterly" => d.add_months(3),
        "annual" => d.add_months(12),
        _ => d.add_months(1),
    }
}

/// Occurrence dates from `from`, stepping by `interval`, up to and including `to`.
fn occurrences(from: &Date, to: &Date, interval: &str) -> Result<Vec<Date>, EngineError> {
    // Guard against a zero step producing an unbounded loop.
    let step_months = match interval {
        "monthly" => Some(1),
        "quarterly" => Some(3),
        "annual" => Some(12),
        "daily" | "weekly" => None,
        other => {
            return Err(EngineError::Schedule(format!(
                "unsupported schedule interval: {other}"
            )))
        }
    };
    let step_days = match interval {
        "daily" => Some(1),
        "weekly" => Some(7),
        _ => None,
    };

    let mut out = Vec::new();
    let advance = |d: &Date| match (step_months, step_days) {
        (Some(m), _) => d.add_months(m),
        (_, Some(days)) => d.add_days(days),
        _ => d.clone(),
    };
    let mut cursor = from.clone();
    let last = to.clone();
    // A monthly stream over a century is ~1200 occurrences; this ceiling only
    // exists so a malformed range cannot spin.
    let limit = 100_000;
    while cursor <= last && out.len() < limit {
        out.push(cursor.clone());
        let next = advance(&cursor);
        if next == cursor {
            break;
        }
        cursor = next;
    }
    Ok(out)
}

/// Move an occurrence within its own interval per `on day <n>` / `on eom`.
fn place_in_interval(occurrence: &Date, on_rule: Option<&IrOnRule>) -> Date {
    match on_rule {
        Some(rule) if rule.kind == "EndOfMonth" => Date {
            year: occurrence.year,
            month: occurrence.month,
            day: days_in_month(occurrence.year, occurrence.month),
        },
        Some(rule) if rule.kind == "DayOfMonth" => Date {
            year: occurrence.year,
            month: occurrence.month,
            // Clamp so day 31 in a 30-day month lands on the 30th rather than
            // rolling into the next period.
            day: (rule.day.max(1) as u32).min(days_in_month(occurrence.year, occurrence.month)),
        },
        _ => occurrence.clone(),
    }
}

/// The timeline bucket containing `date`: the last period starting at or
/// before it. An occurrence mid-period belongs to that period, so an exact
/// match is not required.
fn period_index(timeline: &[Date], date: &Date) -> Option<usize> {
    if timeline.first().is_some_and(|first| date < first) {
        return None;
    }
    timeline.iter().rposition(|d| d <= date)
}

/// Populate `out[payment] = Some(accrual)` for every occurrence.
///
/// Mirrors `apply_schedule`, but records which period earned each payment
/// rather than accumulating an amount. `OnDate` and the explicit `also`/
/// `except` dates are point events, so their accrual and payment periods are
/// the same.
/// The last day of the period at `idx` — the day before the next one starts.
///
/// A period's spacing is taken from its neighbours rather than the calendar,
/// so this works on any cadence. The final period borrows the spacing of the
/// one before it.
fn period_end(timeline: &[Date], idx: usize) -> Date {
    match timeline.get(idx + 1) {
        Some(next) => next.add_days(-1).max(timeline[idx].clone()),
        None => {
            let span = timeline
                .get(idx.wrapping_sub(1))
                .map(|prev| days_between(prev, &timeline[idx]))
                .unwrap_or(30)
                .max(1);
            timeline[idx].add_days(span - 1)
        }
    }
}

/// Whole days from `from` to `to`, negative if `to` precedes it.
fn days_between(from: &Date, to: &Date) -> i32 {
    (to.to_epoch_days() - from.to_epoch_days()) as i32
}

/// Whether a date falls past the last period the timeline models.
///
/// The final period spans from its start date up to the next one that would
/// have followed, so a date inside that span still belongs to it. Spacing is
/// taken from the last two periods rather than the calendar, which keeps this
/// independent of the cadence.
fn beyond_timeline(timeline: &[Date], date: &Date) -> bool {
    let (Some(last), Some(prev)) = (
        timeline.last(),
        timeline.get(timeline.len().wrapping_sub(2)),
    ) else {
        return false;
    };
    let span_days = days_between(prev, last).max(1);
    *date >= last.add_days(span_days)
}

fn apply_schedule_indices(
    schedule: &IrSchedule,
    timeline: &[Date],
    out: &mut [Vec<usize>],
) -> Result<(), EngineError> {
    let roll = schedule_roll(schedule)?;
    match schedule.kind.as_str() {
        "OnDate" => {
            if let Some(on) = &schedule.on {
                let target = roll_date(&Date::parse(on)?, roll);
                if let Some(idx) = timeline.iter().position(|d| *d == target) {
                    out[idx].push(idx);
                }
            }
        }
        "Every" => {
            let default_from = timeline
                .first()
                .map(|d| d.to_string())
                .unwrap_or_else(|| "1970-01-01".to_string());
            let default_to = timeline
                .last()
                .map(|d| d.to_string())
                .unwrap_or_else(|| "1970-01-01".to_string());
            let from = schedule.from.as_deref().unwrap_or(default_from.as_str());
            let to = schedule.to.as_deref().unwrap_or(default_to.as_str());
            let from_date = Date::parse(from)?;
            let to_date = Date::parse(to)?;
            let interval = schedule.every.as_deref().unwrap_or("monthly");

            // Accruals run over [from, to]; each settles at its own end for an
            // ordinary annuity, or at its start for an annuity due.
            let starts = occurrences(&from_date, &to_date, interval)?;
            for (k, start) in starts.iter().enumerate() {
                let accrual_idx = match period_index(timeline, start) {
                    Some(i) => i,
                    None => continue,
                };
                // An annuity due pays as its interval opens. An ordinary
                // annuity pays as it closes — the last calendar period the
                // interval covers, which for an annual interval on a monthly
                // grid is its twelfth month, not its first.
                let pay_idx = if schedule.due {
                    accrual_idx
                } else {
                    let next = starts
                        .get(k + 1)
                        .cloned()
                        .unwrap_or_else(|| step_once(start, interval));
                    // The interval closes in the last period before the
                    // next one opens. Taking that directly, rather than
                    // stepping back from the next interval's index, keeps the
                    // final interval correct when it closes at or past the
                    // end of the timeline.
                    timeline
                        .iter()
                        .rposition(|d| *d < next)
                        .unwrap_or(accrual_idx)
                };
                // The billing date, then the payment terms, then the roll.
                // Order matters: the due date is N days after billing, and it
                // is that due date which moves off a weekend — not the bill.
                //
                // Billing happens when the period closes, not when it opens:
                // January's electricity is invoiced at the end of January, so
                // net-30 falls in early March, not late January. A day rule
                // (`on day 15`, `on eom`) names the billing date explicitly
                // and overrides that.
                let has_terms = schedule.net_days.is_some_and(|d| d != 0)
                    || schedule.net_months.is_some_and(|m| m != 0);
                let billed = match (has_terms, schedule.on_rule.as_ref()) {
                    (true, None) => period_end(timeline, pay_idx),
                    _ => place_in_interval(&timeline[pay_idx], schedule.on_rule.as_ref()),
                };
                // Months step by the calendar, not by 30 days: a six-month lag
                // is six months, and the two diverge once billing is not at a
                // month end.
                let due = match (schedule.net_days, schedule.net_months) {
                    (_, Some(m)) if m != 0 => billed.add_months(m as i32),
                    (Some(d), _) if d != 0 => billed.add_days(d as i32),
                    _ => billed,
                };
                let rolled = roll_date(&due, roll);

                // period_index clamps a date past the end to the last period,
                // which would pile deferred cash into the final bucket and
                // overstate it. A payment that falls outside the modelled
                // horizon is a modelling error, not a rounding one.
                if beyond_timeline(timeline, &rolled) {
                    return Err(EngineError::Schedule(format!(
                        "a payment accruing in period {} settles on {} under these payment terms, past the end of the model timeline. Extend the timeline so the cash has a period to land in.",
                        accrual_idx + 1,
                        rolled
                    )));
                }
                let settled = period_index(timeline, &rolled).unwrap_or(pay_idx);
                out[settled].push(accrual_idx);
            }
        }
        other => {
            return Err(EngineError::Schedule(format!(
                "unsupported schedule kind: {other}"
            )));
        }
    }
    for raw in &schedule.except_dates {
        let target = roll_date(&Date::parse(raw)?, roll);
        if let Some(idx) = timeline.iter().position(|d| *d == target) {
            out[idx].clear();
        }
    }
    for raw in &schedule.also_dates {
        let target = roll_date(&Date::parse(raw)?, roll);
        if let Some(idx) = timeline.iter().position(|d| *d == target) {
            out[idx].push(idx);
        }
    }
    Ok(())
}

/// Resolve the schedule's business-day roll, if any. A convention without a
/// calendar defaults to the weekend-only calendar; a calendar without a
/// convention defaults to `following`.
fn schedule_roll(
    schedule: &IrSchedule,
) -> Result<Option<(cfdl_calc::RollConvention, cfdl_calc::HolidayCalendar)>, EngineError> {
    if schedule.convention.is_none() && schedule.calendar.is_none() {
        return Ok(None);
    }
    let convention = match schedule.convention.as_deref() {
        None => cfdl_calc::RollConvention::Following,
        Some(raw) => cfdl_calc::RollConvention::parse(raw)
            .ok_or_else(|| EngineError::Schedule(format!("unknown roll convention: {raw}")))?,
    };
    let calendar = match schedule.calendar.as_deref() {
        None => cfdl_calc::HolidayCalendar::Weekend,
        Some(raw) => cfdl_calc::HolidayCalendar::parse(raw)
            .ok_or_else(|| EngineError::Schedule(format!("unknown holiday calendar: {raw}")))?,
    };
    Ok(Some((convention, calendar)))
}

fn roll_date(
    date: &Date,
    roll: Option<(cfdl_calc::RollConvention, cfdl_calc::HolidayCalendar)>,
) -> Date {
    let Some((convention, calendar)) = roll else {
        return date.clone();
    };
    let Some(calc) = cfdl_calc::CalcDate::new(date.year, date.month, date.day) else {
        return date.clone();
    };
    let rolled = calendar.roll(&calc, convention);
    Date {
        year: rolled.year(),
        month: rolled.month(),
        day: rolled.day(),
    }
}

fn timeline_dates(start: &str, calendar: &str, periods: usize) -> Result<Vec<Date>, EngineError> {
    let start = Date::parse(start)?;
    let mut out = Vec::with_capacity(periods);
    for idx in 0..periods {
        let date = match calendar {
            "daily" => start.add_days(idx as i32),
            "monthly" => start.add_months(idx as i32),
            "quarterly" => start.add_months((idx as i32) * 3),
            "annual" => start.add_months((idx as i32) * 12),
            _ => start.add_months(idx as i32),
        };
        out.push(date);
    }
    Ok(out)
}

fn round_amount(value: f64) -> f64 {
    // Single global rounding policy for deterministic numeric outputs.
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn canonical_hash(value: &Value) -> String {
    let canonical = canonical_json(value);
    let digest = Sha256::digest(canonical.as_bytes());
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(v) => v.to_string(),
        Value::String(v) => serde_json::to_string(v).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(values) => {
            let inner = values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{inner}]")
        }
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut parts = Vec::with_capacity(keys.len());
            for key in keys {
                let key_json = serde_json::to_string(&key).unwrap_or_else(|_| "\"\"".to_string());
                let value_json = canonical_json(&map[&key]);
                parts.push(format!("{key_json}:{value_json}"));
            }
            format!("{{{}}}", parts.join(","))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Date {
    year: i32,
    month: u32,
    day: u32,
}

impl Date {
    pub fn parse(value: &str) -> Result<Self, EngineError> {
        let parts = value.split('-').collect::<Vec<_>>();
        match parts.as_slice() {
            [y, m, d] => {
                let year = y
                    .parse::<i32>()
                    .map_err(|_| EngineError::InvalidDate(value.to_string()))?;
                let month = m
                    .parse::<u32>()
                    .map_err(|_| EngineError::InvalidDate(value.to_string()))?;
                let day = d
                    .parse::<u32>()
                    .map_err(|_| EngineError::InvalidDate(value.to_string()))?;
                if month == 0 || month > 12 {
                    return Err(EngineError::InvalidDate(value.to_string()));
                }
                if day == 0 || day > days_in_month(year, month) {
                    return Err(EngineError::InvalidDate(value.to_string()));
                }
                Ok(Self { year, month, day })
            }
            _ => Err(EngineError::InvalidDate(value.to_string())),
        }
    }

    /// Days since 1970-01-01, negative before it.
    ///
    /// Hinnant's civil-from-days, the standard branch-free formulation. Used
    /// for date differences; `add_days` still walks day by day, which is fine
    /// for the small offsets payment terms produce.
    fn to_epoch_days(&self) -> i64 {
        let y = if self.month <= 2 {
            self.year - 1
        } else {
            self.year
        } as i64;
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = self.month as i64;
        let d = self.day as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    fn add_days(&self, days: i32) -> Self {
        if days == 0 {
            return self.clone();
        }
        // Stepping backwards used to be a silent no-op, so any caller asking
        // for a day before a date got the date itself.
        if days < 0 {
            let mut out = self.clone();
            for _ in 0..(-days) {
                if out.day > 1 {
                    out.day -= 1;
                } else {
                    if out.month == 1 {
                        out.month = 12;
                        out.year -= 1;
                    } else {
                        out.month -= 1;
                    }
                    out.day = days_in_month(out.year, out.month);
                }
            }
            return out;
        }
        let mut out = self.clone();
        for _ in 0..days {
            let dim = days_in_month(out.year, out.month);
            if out.day < dim {
                out.day += 1;
            } else {
                out.day = 1;
                if out.month == 12 {
                    out.month = 1;
                    out.year += 1;
                } else {
                    out.month += 1;
                }
            }
        }
        out
    }

    fn add_months(&self, months: i32) -> Self {
        let total_months = self.year * 12 + (self.month as i32 - 1) + months;
        let year = total_months.div_euclid(12);
        let month = total_months.rem_euclid(12) as u32 + 1;
        let day = self.day.min(days_in_month(year, month));
        Self { year, month, day }
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[derive(Debug, Deserialize)]
struct Ir {
    model: IrModel,
    time: IrTime,
    #[serde(default)]
    streams: Vec<IrStream>,
    /// Per-stream record of what each pack rule consumed. Deserialized as
    /// opaque JSON and republished verbatim: the engine has no use for it, and
    /// giving it a typed shape here would mean maintaining that shape in two
    /// crates for no gain.
    #[serde(default)]
    stream_inputs: Vec<serde_json::Value>,
    /// Per-period subtotals, in dependency order. The compiler has already
    /// rejected forward references, so a lookup here always finds something
    /// already computed.
    #[serde(default)]
    subtotals: Vec<IrSubtotal>,
    #[serde(default)]
    assumptions: IrAssumptions,
    #[serde(default)]
    events: Vec<IrEvent>,
    #[serde(default)]
    options: Vec<IrOption>,
    #[serde(default)]
    phases: Vec<IrPhase>,
    #[serde(default)]
    curves: Vec<IrCurve>,
    #[serde(default)]
    states: Vec<IrState>,
    /// Declared entities. Read so an entity's lifecycle STARTS where the model
    /// says rather than at null — the totality the ontology exists to give.
    #[serde(default)]
    entities: Vec<IrEntityDecl>,
    /// Run modes the model declares for itself. A `run monte_carlo trials N
    /// seed S` in source used to be parsed, lowered, and then dropped here, so
    /// the model asked for trials and got a single deterministic pass.
    #[serde(default)]
    runs: Vec<IrRun>,
}

#[derive(Debug, Deserialize)]
struct IrState {
    name: String,
    init: IrExpr,
    next: IrExpr,
    /// When the recurrence steps. Absent means every model period.
    #[serde(default)]
    schedule: Option<IrSchedule>,
}

#[derive(Debug, Deserialize)]
struct IrCurve {
    name: String,
    /// "step" (flat-forward) or "linear".
    #[serde(default = "default_interpolation")]
    interpolation: String,
    points: Vec<IrCurvePoint>,
}

fn default_interpolation() -> String {
    "step".to_string()
}

#[derive(Debug, Deserialize)]
struct IrCurvePoint {
    date: String,
    value: f64,
}

#[derive(Debug, Deserialize)]
struct IrEvent {
    name: String,
    when: IrExpr,
    #[serde(default)]
    actions: Vec<IrAction>,
}

#[derive(Debug, Deserialize)]
struct IrAction {
    kind: String,
    #[serde(default)]
    entity: Option<IrEntityRef>,
    #[serde(default)]
    field: Option<String>,
    #[serde(default)]
    value: Option<IrExpr>,
    #[serde(default)]
    stream: Option<String>,
    #[serde(default)]
    #[allow(dead_code)] // deserialized for contract actions, which warn-and-skip for now
    contract: Option<String>,
    #[serde(default)]
    option: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IrOption {
    name: String,
    exercise_when: IrExpr,
    payoff: IrExpr,
    #[serde(default)]
    exercisable_in_phase: Option<String>,
    /// The asset the option is written on. An option is a contract, so it has
    /// one; with it, `entity.<field>` in a guard means the same thing it means
    /// in a stream.
    #[serde(default)]
    owner: Option<IrEntityRef>,
}

#[derive(Debug, Deserialize)]
struct IrEntityDecl {
    symbol: String,
    /// The lifecycle state this entity opens in. `None` when its type declares
    /// no lifecycle, which is most entities.
    #[serde(default)]
    initial_state: Option<String>,
    /// The entity this one is part of, when the model groups it. Absent for
    /// most entities: hierarchy is available at every grain and required at
    /// none.
    #[serde(default)]
    parent: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IrPhase {
    name: String,
    range: IrDateRange,
}

#[derive(Debug, Deserialize)]
struct IrDateRange {
    start: String,
    end: String,
}

#[derive(Debug, Deserialize)]
struct IrRun {
    kind: String,
    #[serde(default)]
    trials: Option<u32>,
    #[serde(default)]
    seed: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct IrAssumptions {
    #[serde(default)]
    constants: BTreeMap<String, IrAssumeConstant>,
    #[serde(default)]
    random: BTreeMap<String, IrAssumeRandom>,
}

#[derive(Debug, Deserialize)]
struct IrAssumeConstant {
    expr: IrExpr,
}

#[derive(Debug, Deserialize)]
struct IrAssumeRandom {
    dist: IrDistribution,
}

#[derive(Debug, Deserialize)]
struct IrDistribution {
    kind: String,
    #[serde(default)]
    params: BTreeMap<String, f64>,
    #[serde(default)]
    clip: Option<[f64; 2]>,
}

#[derive(Debug, Deserialize)]
struct IrModel {
    #[serde(default)]
    name: Option<String>,
    currency: String,
}

#[derive(Debug, Deserialize)]
struct IrTime {
    calendar: String,
    start: String,
    periods: u32,
    #[serde(default)]
    projection: u32,
}

#[derive(Debug, Deserialize)]
struct IrStream {
    name: String,
    owner: IrEntityRef,
    direction: String,
    /// What this stream is, economically. The fold layer aggregates on this
    /// rather than on the name — the one field of stream metadata the engine
    /// genuinely needs, read once per stream rather than per period.
    #[serde(default)]
    category: Option<String>,
    schedule: IrSchedule,
    amount: IrExpr,
    #[serde(default)]
    active_when: Option<IrExpr>,
}

#[derive(Debug, Deserialize)]
struct IrSubtotal {
    id: String,
    // `kind` is in the IR but not read here: `op` already determines the shape
    // (a sum is money, a ratio is a number), and the pack loader has rejected
    // any spec where the two disagree. Deserializing a field only to ignore it
    // would be the kind of accepted-and-discarded the repo rejects elsewhere.
    op: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    streams: Vec<String>,
    #[serde(default)]
    subtotals: Vec<String>,
    #[serde(default)]
    numerator: Option<String>,
    #[serde(default)]
    denominator: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IrEntityRef {
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct IrOnRule {
    kind: String,
    #[serde(default)]
    day: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct IrSchedule {
    kind: String,
    on: Option<String>,
    #[serde(default)]
    every: Option<String>,
    /// Annuity due: payment at the start of each interval. Absent means an
    /// ordinary annuity — the interval elapses, then payment falls.
    #[serde(default)]
    due: bool,
    /// A one-shot flow that settles at the END of its period.
    #[serde(default)]
    at_period_end: bool,
    /// Mid-period convention: cash treated as arriving halfway through the
    /// period it was earned in. A discounting convention rather than a date,
    /// so it is 0.5 of a period on every calendar.
    #[serde(default)]
    mid: bool,
    /// How long after a flow is earned its cash moves. Absent means the cash
    /// lands in the period that earned it.
    #[serde(default)]
    net_days: Option<i64>,
    #[serde(default)]
    net_months: Option<i64>,
    from: Option<String>,
    to: Option<String>,
    /// Places an occurrence within its interval (`on day <n>` / `on eom`).
    /// Previously not even deserialized, so the compiler emitted it and the
    /// engine dropped it — `on day 15` had no effect on any cash flow.
    #[serde(default)]
    on_rule: Option<IrOnRule>,
    #[serde(default)]
    phase: Option<String>,
    #[serde(default)]
    convention: Option<String>,
    #[serde(default)]
    calendar: Option<String>,
    #[serde(default)]
    except_dates: Vec<String>,
    #[serde(default)]
    also_dates: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct IrExpr {
    #[serde(default)]
    lang: Option<String>,
    src: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainMetrics {
    pub pack: String,
    pub metrics: BTreeMap<String, Scalar>,
    pub lineage: BTreeMap<String, MetricLineage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricLineage {
    pub numerator_streams: Vec<String>,
    pub denominator_streams: Vec<String>,
    pub formula: String,
}

#[derive(Debug, Serialize)]
pub struct Results {
    pub results_version: String,
    pub model_hash: String,
    /// Content hash of the deterministic ledger — the per-stream, per-period
    /// series this run produced.
    ///
    /// Together with `model_hash`, `engine` and the run config in
    /// `deterministic.metrics`, this closes the chain: identical inputs on an
    /// identical engine must reproduce an identical `ledger_hash`. If they do
    /// not, something is nondeterministic, and the golden suite would otherwise
    /// report that as a flapping test rather than as the defect it is.
    ///
    /// It hashes the LEDGER, not the inputs, deliberately. "Did the inputs
    /// change" is already answerable from `model_hash`; what nothing answered
    /// before is "did the output change", which is the question a reviewer
    /// staring at a re-blessed golden actually has.
    pub ledger_hash: String,
    pub engine: EngineInfo,
    pub warnings: Vec<String>,
    /// Resolved assumptions and the contract terms each lowered stream
    /// consumed. Absent when the model declares neither.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<InputsSection>,
    pub deterministic: DeterministicSection,
    pub scenarios: ScenarioSection,
    pub monte_carlo: MonteCarloSection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_metrics: Option<DomainMetrics>,
    /// Rendered statements. Present only when the active pack declares one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statements: Option<StatementsSection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementsSection {
    pub pack: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Statement {
    pub id: String,
    pub label: String,
    pub default: bool,
    /// The grain this statement reports at, and the period labels that go with
    /// it. Published because a consumer CANNOT derive it: an annual statement
    /// over a monthly model has ten values where the model has 120, and nothing
    /// else in the document says which ten periods those are. The playground
    /// needs it to label a column; so does anyone rendering the JSON.
    pub grain: StatementGrain,
    pub rows: Vec<StatementRow>,
    pub reconciliation: StatementReconciliation,
    /// Completeness findings. Empty is the healthy case.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<StatementDiagnostic>,
}

/// How a statement's columns are bucketed, and what to call them.
#[derive(Debug, Clone, Serialize)]
pub struct StatementGrain {
    /// `monthly` | `quarterly` | `annual` | whatever the model grid is.
    pub calendar: String,
    /// First bucket's start date.
    pub start: String,
    /// One label per column, ready to render.
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementRow {
    /// `line` | `subtotal` | `ratio` | `spacer` | `residual`.
    pub kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    pub depth: u32,
    /// How to RENDER the sign: +1 shows the value as stored, -1 flips it for
    /// display only. `values` is always the signed arithmetic quantity, so a
    /// consumer that ignores this still adds up correctly.
    pub display_sign: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<SeriesValue>,
    /// Lifetime total of the row. Absent for a ratio, where summing means
    /// nothing, and for a spacer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    /// The streams this row drew from. Present on `line` and `residual` rows;
    /// it is what makes a published figure traceable without the ledger.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub streams: Vec<String>,
}

/// Does the statement add up to the model's cash?
///
/// Published always and asserted rather than corrected. A statement whose
/// bottom line quietly differs from `model.total` is the failure this exists to
/// make visible.
#[derive(Debug, Clone, Serialize)]
pub struct StatementReconciliation {
    pub bottom_line: f64,
    pub model_total: f64,
    pub residual: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementDiagnostic {
    pub code: String,
    pub message: String,
}

/// The top of the audit chain: what went in, above the line items.
#[derive(Debug, Clone, Serialize)]
pub struct InputsSection {
    /// Evaluated `assume` values, as `inputs.<name>` resolves them.
    ///
    /// In a deterministic run a random assumption resolves to its clipped
    /// central value, not to a draw — publishing it here is what stops that
    /// being invisible.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub resolved: BTreeMap<String, f64>,
    /// Per-stream record of the contract terms a pack rule consumed. Passed
    /// through from the IR verbatim, so `IrStream` and the per-period
    /// evaluation path are untouched by it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub streams: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct EngineInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeterministicSection {
    pub status: String,
    pub metrics: BTreeMap<String, Scalar>,
    pub series: BTreeMap<String, Series>,
    /// Every state change an event made, in the order it happened. Omitted when
    /// the model has none, so a model without events is unchanged.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<TransitionRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annual_rollup: Option<AnnualRollupSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RuntimeError>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloSection {
    pub status: String,
    pub trials: u32,
    pub seed: u64,
    pub metrics: BTreeMap<String, MetricSummary>,
    pub trial_summaries: Vec<MonteCarloTrialSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregates: Option<MonteCarloAggregates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RuntimeError>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioSection {
    pub status: String,
    pub summaries: Vec<ScenarioSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RuntimeError>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioSummary {
    pub name: String,
    pub metrics: BTreeMap<String, Scalar>,
}

/// Calendar-year aggregates of all per-period series.
/// Omitted when the model frequency is already "annual".
#[derive(Debug, Clone, Serialize)]
pub struct AnnualRollupSection {
    pub series: BTreeMap<String, Series>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloTrialSummary {
    pub trial: u32,
    pub metrics: BTreeMap<String, Scalar>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloAggregates {
    pub npv: NpvAggregate,
}

#[derive(Debug, Clone, Serialize)]
pub struct NpvAggregate {
    pub mean: f64,
    pub median: f64,
    pub stddev: f64,
    pub p_negative: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Scalar {
    Number(f64),
    Money(Money),
    String(String),
}

/// One period's value on a published series.
///
/// Cash carries a currency; a declared `state` does not — it is an index, a
/// factor, a count. Publishing a state as `Money` would assert a denomination
/// it does not have, and would make it look summable alongside cash. The
/// results schema has always permitted a bare number here.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SeriesValue {
    Money(Money),
    Number(f64),
    /// A period where the value is genuinely undefined — a coverage ratio in a
    /// period with no debt service. Published as JSON `null`, which the results
    /// schema has always permitted.
    ///
    /// Not zero: a coverage ratio of "no debt" is not a coverage ratio of zero,
    /// and a consumer that averaged the series would be badly misled. Not an
    /// omission either, because a shortened series breaks index alignment.
    Null,
}

impl SeriesValue {
    /// The cash amount, or `None` for a series that is not money. Callers that
    /// weight or sum cash use this, so a state cannot silently contribute.
    pub fn money_amount(&self) -> Option<f64> {
        match self {
            SeriesValue::Money(m) => Some(m.amount),
            SeriesValue::Number(_) | SeriesValue::Null => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Series {
    pub index: SeriesIndex,
    /// Where in each period this series' cash falls, per
    /// docs/12_payment_timing.md — the same offset used to discount it, and
    /// the axis `model.wal_years` is measured on. Absent on aggregates
    /// (`model.net_cash_flow`, the annual rollup), which sum streams whose
    /// placements differ and so have no single position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<f64>,
    pub values: Vec<SeriesValue>,
}

impl Series {
    fn from_values(
        calendar: &str,
        start: &str,
        periods: u32,
        currency: &str,
        offset: Option<f64>,
        values: &[f64],
    ) -> Self {
        Self {
            index: SeriesIndex {
                calendar: calendar.to_string(),
                start: start.to_string(),
                periods,
            },
            offset,
            values: values
                .iter()
                .map(|amount| {
                    SeriesValue::Money(Money {
                        amount: round_amount(*amount),
                        currency: currency.to_string(),
                    })
                })
                .collect(),
        }
    }

    /// A dimensionless series: a declared `state`, published so a recurrence
    /// can be inspected rather than only its effect on cash. No currency and
    /// no offset — a state is not paid, so it does not sit anywhere in its
    /// period.
    /// A plain-number series where some periods are genuinely undefined.
    /// `None` publishes as JSON `null`, which the results schema permits.
    ///
    /// Rounded like every other published number. That is not cosmetic: a
    /// ratio's numerator is a fold of signed cash, so a period whose flows
    /// cancel leaves a residue rather than an exact zero — around 2e-12 in
    /// practice — and dividing that by a real denominator publishes something
    /// like 2.655e-17. Whose last bits differ by platform: this shipped, and
    /// the Windows runner disagreed with Linux and macOS on one golden while
    /// both of those agreed with each other.
    ///
    /// `round_amount` is described at its definition as the single global
    /// rounding policy for deterministic numeric outputs. Skipping it here was
    /// the defect; nothing else published bypasses it.
    fn from_optional(calendar: &str, start: &str, periods: u32, values: &[Option<f64>]) -> Self {
        Self {
            index: SeriesIndex {
                calendar: calendar.to_string(),
                start: start.to_string(),
                periods,
            },
            offset: None,
            values: values
                .iter()
                .map(|v| match v {
                    Some(x) => SeriesValue::Number(round_amount(*x)),
                    None => SeriesValue::Null,
                })
                .collect(),
        }
    }

    fn from_plain(calendar: &str, start: &str, periods: u32, values: &[f64]) -> Self {
        Self {
            index: SeriesIndex {
                calendar: calendar.to_string(),
                start: start.to_string(),
                periods,
            },
            offset: None,
            values: values.iter().map(|v| SeriesValue::Number(*v)).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SeriesIndex {
    pub calendar: String,
    pub start: String,
    pub periods: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricSummary {
    pub r#type: String,
    pub mean: Scalar,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdev: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p01: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p05: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p10: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p25: Option<Scalar>,
    pub p50: Scalar,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p75: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p90: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p99: Option<Scalar>,
}

fn splitmix64(state: u64) -> u64 {
    let mut z = state.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

fn next_uniform_open_closed(seed: &mut u64) -> f64 {
    let bits = splitmix64(*seed);
    *seed = bits;
    ((bits as f64) + 1.0) / ((u64::MAX as f64) + 1.0)
}

fn sample_distribution(distribution: &DistributionSpec, seed: &mut u64) -> f64 {
    match distribution {
        DistributionSpec::Fixed { value } => *value,
        DistributionSpec::Uniform { min, max } => {
            if (*max - *min).abs() < f64::EPSILON {
                *min
            } else {
                min + ((*max - *min) * next_uniform_open_closed(seed))
            }
        }
        DistributionSpec::Normal { mean, stddev } => {
            if *stddev == 0.0 {
                *mean
            } else {
                let u1 = next_uniform_open_closed(seed);
                let u2 = next_uniform_open_closed(seed);
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                mean + stddev * z0
            }
        }
        DistributionSpec::LogNormal { mu, sigma } => {
            if *sigma == 0.0 {
                mu.exp()
            } else {
                let u1 = next_uniform_open_closed(seed);
                let u2 = next_uniform_open_closed(seed);
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                (mu + sigma * z0).exp()
            }
        }
        DistributionSpec::Triangular { min, mode, max } => {
            if (*max - *min).abs() < f64::EPSILON {
                *min
            } else {
                // Inverse-CDF sampling.
                let u = next_uniform_open_closed(seed);
                let fc = (mode - min) / (max - min);
                if u < fc {
                    min + (u * (max - min) * (mode - min)).sqrt()
                } else {
                    max - ((1.0 - u) * (max - min) * (max - mode)).sqrt()
                }
            }
        }
    }
}

/// FNV-1a hash for per-assumption seed derivation: adding or removing one
/// assumption never reshuffles another assumption's draws.
fn fnv1a(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in name.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn ir_distribution_spec(dist: &IrDistribution) -> Result<DistributionSpec, EngineError> {
    let get =
        |names: &[&str]| -> Option<f64> { names.iter().find_map(|n| dist.params.get(*n).copied()) };
    let missing = |what: &str| {
        EngineError::InvalidRunConfig(format!(
            "assumption distribution {} missing parameter '{}'",
            dist.kind, what
        ))
    };
    match dist.kind.as_str() {
        "Normal" => Ok(DistributionSpec::Normal {
            mean: get(&["mean"]).ok_or_else(|| missing("mean"))?,
            stddev: get(&["stdev", "stddev"]).ok_or_else(|| missing("stdev"))?,
        }),
        "LogNormal" => Ok(DistributionSpec::LogNormal {
            mu: get(&["mu"]).ok_or_else(|| missing("mu"))?,
            sigma: get(&["sigma"]).ok_or_else(|| missing("sigma"))?,
        }),
        "Uniform" => Ok(DistributionSpec::Uniform {
            min: get(&["min"]).ok_or_else(|| missing("min"))?,
            max: get(&["max"]).ok_or_else(|| missing("max"))?,
        }),
        "Triangular" => Ok(DistributionSpec::Triangular {
            min: get(&["min"]).ok_or_else(|| missing("min"))?,
            mode: get(&["mode"]).ok_or_else(|| missing("mode"))?,
            max: get(&["max"]).ok_or_else(|| missing("max"))?,
        }),
        other => Err(EngineError::InvalidRunConfig(format!(
            "unknown assumption distribution kind: {other}"
        ))),
    }
}

fn apply_clip(value: f64, clip: Option<[f64; 2]>) -> f64 {
    match clip {
        Some([lo, hi]) => value.clamp(lo, hi),
        None => value,
    }
}

/// Deterministic central value of a distribution, used outside Monte Carlo:
/// Normal -> mean; LogNormal -> exp(mu + sigma^2/2) (the distribution mean);
/// Uniform -> midpoint; Triangular -> (min + mode + max) / 3.
fn central_value(spec: &DistributionSpec) -> f64 {
    match spec {
        DistributionSpec::Fixed { value } => *value,
        DistributionSpec::Normal { mean, .. } => *mean,
        DistributionSpec::LogNormal { mu, sigma } => (mu + sigma * sigma / 2.0).exp(),
        DistributionSpec::Uniform { min, max } => (min + max) / 2.0,
        DistributionSpec::Triangular { min, mode, max } => (min + mode + max) / 3.0,
    }
}

fn stats_mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn stats_median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

fn stats_stddev_population(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = stats_mean(values);
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt()
}

fn probability_negative(values: &[f64]) -> f64 {
    let negatives = values.iter().filter(|value| **value < 0.0).count();
    negatives as f64 / values.len() as f64
}

#[cfg(test)]
mod tests {
    /// A minimal one-stream IR, with the amount parameterised so a test can
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
    /// The property the whole decoupling exists for: the same cash, modelled
    /// at two different grains, values the same when valued at one convention.
    ///
    /// Before this, `ppy` came from `ir.time.calendar`, so a model's CALENDAR
    /// decided its valuation convention. `benchmarks/cre/mit_rentleg_plaza`
    /// records the consequence — a monthly rebuild "discounting at
    /// (1.12)^(1/12)-1 gives ~$2,323,050, about +1.3%" — and attributes it to
    /// the rebuild. It is not the rebuild. Summing a year's cash and then
    /// discounting at the annual rate is the same arithmetic whichever grain
    /// the cash was modelled on, and this asserts exactly that.
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
mod phase_reference_tests {
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

    /// A phase-2 stream reading another phase-2 stream can never resolve —
    /// `full_series` is sealed before phase 2 runs and never grows — so it
    /// would always aggregate to zero. That is a wrong number reported as a
    /// plausible one, which is the failure this rejects.
    #[test]
    fn phase2_reading_phase2_is_an_error_not_a_zero() {
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
                                "src": "series_sum(\"derived.a\", 0, time.t)" },
                    "schedule": { "kind": "Every", "every": "annual",
                                  "from": "2026-01-01", "to": "2028-01-01" }
                }
            ]
        });
        let ir: Ir = serde_json::from_value(ir_json).expect("ir parses");
        let err = run_deterministic(&ir, &RunConfig::default())
            .expect_err("a read that can never resolve is an error");
        match err {
            EngineError::PhaseReference(msg) => {
                assert!(msg.contains("derived.b"), "{msg}");
                assert!(msg.contains("derived.a"), "{msg}");
                assert!(msg.contains("always aggregate to zero"), "{msg}");
            }
            other => panic!("expected PhaseReference, got {other:?}"),
        }
    }
}
