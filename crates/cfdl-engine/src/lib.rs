use cfdl_expr::{CompiledExpr, ExprEnv, Value as ExprValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug)]
pub enum EngineError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidDate(String),
    InvalidRunConfig(String),
    Schedule(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Io(err) => write!(f, "I/O error: {err}"),
            EngineError::Json(err) => write!(f, "JSON error: {err}"),
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
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            discount_rate: 0.0,
            as_of: None,
            parameter_overrides: BTreeMap::new(),
            scenarios: BTreeMap::new(),
            monte_carlo: None,
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
            },
        )?;
        warnings.extend(scenario_run.warnings);
        let mut scenario_metrics = BTreeMap::new();
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

    Ok(Results {
        results_version: "0.2".to_string(),
        model_hash,
        engine: EngineInfo {
            name: "cfdl-engine".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build: None,
        },
        warnings,
        deterministic,
        scenarios,
        monte_carlo,
        domain_metrics: None,
    })
}

#[derive(Debug, Clone)]
struct DeterministicRunOutput {
    warnings: Vec<String>,
    metrics: BTreeMap<String, Scalar>,
    series: BTreeMap<String, MoneySeries>,
    npv: f64,
    annual_rollup: Option<AnnualRollupSection>,
}

/// Output of the discrete event/option pre-pass over the master timeline.
struct EventSim {
    /// Per period: entity symbol -> field -> value (state as of that period).
    entity_state: Vec<BTreeMap<String, BTreeMap<String, ExprValue>>>,
    /// Per stream with an event override: per-period active flag.
    stream_active: BTreeMap<String, Vec<bool>>,
    /// Option payoff cash flows: option name -> per-period amounts.
    option_cash: BTreeMap<String, Vec<f64>>,
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
    warnings: &mut Vec<String>,
) -> EventSim {
    let periods = timeline.len();
    let mut entity_state: Vec<BTreeMap<String, BTreeMap<String, ExprValue>>> =
        Vec::with_capacity(periods);
    let mut current_state: BTreeMap<String, BTreeMap<String, ExprValue>> = BTreeMap::new();
    let mut current_active: BTreeMap<String, bool> = BTreeMap::new();
    let mut stream_active: BTreeMap<String, Vec<bool>> = BTreeMap::new();
    let mut option_cash: BTreeMap<String, Vec<f64>> = BTreeMap::new();
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
        let env = build_base_env(ir, config, t, date, base_inputs);
        for (event_idx, event) in ir.events.iter().enumerate() {
            if event_fired[event_idx] {
                continue;
            }
            let Some(when) = &compiled_events[event_idx] else {
                continue;
            };
            if !eval_bool_expr(when, &env, &event.name, "event when", warnings) {
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
                                current_state
                                    .entry(entity.symbol.clone())
                                    .or_default()
                                    .insert(field.clone(), v);
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
            let forced = forced_exercise.iter().any(|name| name == &option.name);
            let triggered = if forced {
                true
            } else {
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
                        continue;
                    }
                }
                eval_bool_expr(when, &env, &option.name, "exercise when", warnings)
            };
            if !triggered {
                continue;
            }
            option_exercised[option_idx] = true;
            let mut payoff_values = vec![0.0_f64; periods];
            match cfdl_expr::eval(payoff, &env) {
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

    EventSim {
        entity_state,
        stream_active,
        option_cash,
    }
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
    let event_sim = simulate_events(ir, config, &timeline, &base_inputs, &mut warnings);
    let mut stream_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut stream_totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut entity_totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut model_series = vec![0.0_f64; cash_periods];
    // Each stream's series paired with where in its period the cash falls;
    // valuation needs both, while reported cash uses model_series alone.
    let mut valued_streams: Vec<(Vec<f64>, f64)> = Vec::new();

    // Phase 1: streams without series references. Their FULL (projection-
    // inclusive) values feed the series store for phase 2.
    let mut full_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut phase2: Vec<&IrStream> = Vec::new();
    for stream in &ir.streams {
        let phase2_stream = cfdl_expr::compile_expr(&stream.amount.src)
            .map(|compiled| cfdl_expr::uses_series(&compiled))
            .unwrap_or(false);
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
            None,
            &mut warnings,
        )?;
        valued_streams.push((
            values[..cash_periods.min(values.len())].to_vec(),
            discount_offset(&stream.schedule),
        ));
        record_stream(
            stream,
            &values,
            cash_periods,
            &mut model_series,
            &mut stream_totals,
            &mut entity_totals,
            &mut stream_series,
        );
        full_series.insert(stream.name.clone(), values);
    }

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
            Some(&full_series),
            &mut warnings,
        )?;
        valued_streams.push((
            values[..cash_periods.min(values.len())].to_vec(),
            discount_offset(&stream.schedule),
        ));
        record_stream(
            stream,
            &values,
            cash_periods,
            &mut model_series,
            &mut stream_totals,
            &mut entity_totals,
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
    }

    let mut series_map = BTreeMap::new();
    for (name, values) in &stream_series {
        series_map.insert(
            format!("stream.{name}"),
            MoneySeries::from_values(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &ir.model.currency,
                values,
            ),
        );
    }
    series_map.insert(
        "model.net_cash_flow".to_string(),
        MoneySeries::from_values(
            &ir.time.calendar,
            &ir.time.start,
            periods as u32,
            &ir.model.currency,
            &model_series,
        ),
    );

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
    for (entity_symbol, total) in entity_totals {
        metrics.insert(
            format!("entity.{entity_symbol}.total"),
            Scalar::Money(Money {
                amount: round_amount(total),
                currency: ir.model.currency.clone(),
            }),
        );
    }

    let model_total = model_series.iter().sum::<f64>();
    let ppy = periods_per_year(&ir.time.calendar);
    let per_period_rate = (1.0 + config.discount_rate).powf(1.0 / ppy) - 1.0;
    let npv = npv_with_offsets(&valued_streams, per_period_rate);
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
    let total_inflows: f64 = model_series.iter().filter(|v| **v > 0.0).sum();
    let total_outflows: f64 = -model_series.iter().filter(|v| **v < 0.0).sum::<f64>();
    if total_outflows > 0.0 && total_inflows > 0.0 {
        metrics.insert(
            "model.moic".to_string(),
            Scalar::Number(round_amount(total_inflows / total_outflows)),
        );
    }
    // Payback: first period at which cumulative net cash flow becomes
    // non-negative, given the model starts cash-negative. Omitted otherwise.
    if model_series.first().copied().unwrap_or(0.0) < 0.0 {
        let mut cumulative = 0.0_f64;
        let mut payback: Option<usize> = None;
        for (idx, value) in model_series.iter().enumerate() {
            cumulative += *value;
            if cumulative >= 0.0 {
                payback = Some(idx);
                break;
            }
        }
        if let Some(period) = payback {
            metrics.insert(
                "model.payback_periods".to_string(),
                Scalar::Number(period as f64),
            );
            metrics.insert(
                "model.payback_years".to_string(),
                Scalar::Number(round_amount(period as f64 / ppy)),
            );
        }
    }
    // WAL: inflow-weighted average life in years.
    if total_inflows > 0.0 {
        let weighted: f64 = model_series
            .iter()
            .enumerate()
            .filter(|(_, v)| **v > 0.0)
            .map(|(idx, v)| (idx as f64 / ppy) * *v)
            .sum();
        metrics.insert(
            "model.wal_years".to_string(),
            Scalar::Number(round_amount(weighted / total_inflows)),
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
        ))
    };

    Ok(DeterministicRunOutput {
        warnings,
        metrics,
        series: series_map,
        npv,
        annual_rollup,
    })
}

/// Aggregate per-period series values by calendar year.
///
/// Each distinct calendar year present in `timeline` becomes one entry in the
/// output `MoneySeries`.  Values for all periods that fall within a given year
/// are summed.  The resulting index uses `calendar = "annual"` and `start =
/// "{first_year}-01-01"`.
fn build_annual_rollup(
    timeline: &[Date],
    stream_series: &BTreeMap<String, Vec<f64>>,
    model_series: &[f64],
    currency: &str,
) -> AnnualRollupSection {
    // Collect the ordered, distinct calendar years.
    let mut seen = std::collections::BTreeSet::new();
    let mut years: Vec<i32> = Vec::new();
    for date in timeline {
        if seen.insert(date.year) {
            years.push(date.year);
        }
    }

    let n_years = years.len() as u32;
    let start = format!("{:04}-01-01", years[0]);

    // Sum a flat slice of per-period floats into one value per calendar year.
    let aggregate = |values: &[f64]| -> Vec<f64> {
        years
            .iter()
            .map(|&yr| {
                timeline
                    .iter()
                    .zip(values.iter())
                    .filter_map(|(d, v)| if d.year == yr { Some(*v) } else { None })
                    .sum::<f64>()
            })
            .collect()
    };

    let mut rollup = BTreeMap::new();

    rollup.insert(
        "model.net_cash_flow".to_string(),
        MoneySeries::from_values(
            "annual",
            &start,
            n_years,
            currency,
            &aggregate(model_series),
        ),
    );

    for (name, values) in stream_series {
        rollup.insert(
            format!("stream.{name}"),
            MoneySeries::from_values("annual", &start, n_years, currency, &aggregate(values)),
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
    series: Option<&BTreeMap<String, Vec<f64>>>,
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
    for (pay_idx, accrual) in schedule_accruals.iter().copied().enumerate() {
        let Some(idx) = accrual else {
            continue;
        };
        if let Some(mask) = event_mask {
            if !mask[idx] {
                continue;
            }
        }
        let mut env = build_expr_env(ir, stream, config, idx, &timeline[idx], base_inputs);
        apply_entity_state(&mut env, &event_sim.entity_state[idx], &stream.owner.symbol);
        if let Some(series) = series {
            env.series = series.clone();
        }
        let active_value =
            eval_bool_expr(&active_expr, &env, &stream.name, "active_when", warnings);
        if !active_value {
            continue;
        }
        let amount = if let Some(override_value) = stream_amount_override(config, &stream.name) {
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
        // Evaluated against the accrual period, settled in the payment period.
        values[pay_idx] += amount * direction_sign;
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
    entity_totals: &mut BTreeMap<String, f64>,
    stream_series: &mut BTreeMap<String, Vec<f64>>,
) {
    let cash = &values[..cash_periods.min(values.len())];
    for (idx, value) in cash.iter().enumerate() {
        model_series[idx] += *value;
    }
    let total = cash.iter().sum::<f64>();
    stream_totals.insert(stream.name.clone(), total);
    entity_totals
        .entry(stream.owner.symbol.clone())
        .and_modify(|sum| *sum += total)
        .or_insert(total);
    stream_series.insert(stream.name.clone(), cash.to_vec());
}

fn eval_bool_expr(
    expr: &CompiledExpr,
    env: &ExprEnv,
    stream_name: &str,
    slot: &str,
    warnings: &mut Vec<String>,
) -> bool {
    match cfdl_expr::eval(expr, env) {
        Ok(ExprValue::Bool(value)) => value,
        Ok(other) => {
            warnings.push(format!(
                "Stream '{}' {} expression returned non-bool '{other:?}'; using false.",
                stream_name, slot
            ));
            false
        }
        Err(err) => {
            warnings.push(format!(
                "Stream '{}' {} evaluation failed [{}]: {}; using false.",
                stream_name, slot, err.code, err.message
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

fn build_expr_env(
    ir: &Ir,
    stream: &IrStream,
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

    env.entity.insert(
        "id".to_string(),
        ExprValue::String(stream.owner.symbol.clone()),
    );
    env.entity.insert(
        "name".to_string(),
        ExprValue::String(stream.owner.symbol.clone()),
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
) -> Result<Vec<Option<usize>>, EngineError> {
    let mut out = vec![None; timeline.len()];
    apply_schedule_indices(schedule, timeline, &mut out)?;
    Ok(out)
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
fn discount_offset(schedule: &IrSchedule) -> f64 {
    // A one-shot flow happens on its stated date, not at the end of the
    // period containing it: a purchase on 2026-01 is settled then, so it is
    // not discounted for a period it never waited through.
    if schedule.kind == "OnDate" || schedule.due {
        return 0.0;
    }
    match schedule.on_rule.as_ref() {
        // `on day <n>`: n days into a nominal 30-day period.
        Some(rule) if rule.kind == "DayOfMonth" => (rule.day.clamp(1, 31) as f64 / 30.0).min(1.0),
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
fn apply_schedule_indices(
    schedule: &IrSchedule,
    timeline: &[Date],
    out: &mut [Option<usize>],
) -> Result<(), EngineError> {
    let roll = schedule_roll(schedule)?;
    match schedule.kind.as_str() {
        "OnDate" => {
            if let Some(on) = &schedule.on {
                let target = roll_date(&Date::parse(on)?, roll);
                if let Some(idx) = timeline.iter().position(|d| *d == target) {
                    out[idx] = Some(idx);
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
                let placed = place_in_interval(&timeline[pay_idx], schedule.on_rule.as_ref());
                let rolled = roll_date(&placed, roll);
                let settled = period_index(timeline, &rolled).unwrap_or(pay_idx);
                out[settled] = Some(accrual_idx);
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
            out[idx] = None;
        }
    }
    for raw in &schedule.also_dates {
        let target = roll_date(&Date::parse(raw)?, roll);
        if let Some(idx) = timeline.iter().position(|d| *d == target) {
            out[idx] = Some(idx);
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

    fn add_days(&self, days: i32) -> Self {
        if days <= 0 {
            return self.clone();
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
    /// Run modes the model declares for itself. A `run monte_carlo trials N
    /// seed S` in source used to be parsed, lowered, and then dropped here, so
    /// the model asked for trials and got a single deterministic pass.
    #[serde(default)]
    runs: Vec<IrRun>,
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
    schedule: IrSchedule,
    amount: IrExpr,
    #[serde(default)]
    active_when: Option<IrExpr>,
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
    pub engine: EngineInfo,
    pub warnings: Vec<String>,
    pub deterministic: DeterministicSection,
    pub scenarios: ScenarioSection,
    pub monte_carlo: MonteCarloSection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_metrics: Option<DomainMetrics>,
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
    pub series: BTreeMap<String, MoneySeries>,
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
    pub series: BTreeMap<String, MoneySeries>,
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

#[derive(Debug, Clone, Serialize)]
pub struct MoneySeries {
    pub index: SeriesIndex,
    pub values: Vec<Money>,
}

impl MoneySeries {
    fn from_values(
        calendar: &str,
        start: &str,
        periods: u32,
        currency: &str,
        values: &[f64],
    ) -> Self {
        Self {
            index: SeriesIndex {
                calendar: calendar.to_string(),
                start: start.to_string(),
                periods,
            },
            values: values
                .iter()
                .map(|amount| Money {
                    amount: round_amount(*amount),
                    currency: currency.to_string(),
                })
                .collect(),
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
            },
        )
        .expect("aggregation run");

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
        assert!((rev[0].amount - 3000.0).abs() < 1e-9, "revenue[0]");
        assert!((rev[1].amount - 3000.0).abs() < 1e-9, "revenue[1]");
        assert!((rev[2].amount).abs() < 1e-9, "revenue[2] should be 0");

        // Expense: outflow sign, active periods 0 and 1, zero at period 2
        let exp = &s["stream.ops.expense"].values;
        assert_eq!(exp.len(), 3);
        assert!((exp[0].amount - (-1000.0)).abs() < 1e-9, "expense[0]");
        assert!((exp[1].amount - (-1000.0)).abs() < 1e-9, "expense[1]");
        assert!((exp[2].amount).abs() < 1e-9, "expense[2] should be 0");

        // Exit: zero for first two periods, fires only at period 2
        let exit = &s["stream.exit.proceeds"].values;
        assert_eq!(exit.len(), 3);
        assert!((exit[0].amount).abs() < 1e-9, "exit[0] should be 0");
        assert!((exit[1].amount).abs() < 1e-9, "exit[1] should be 0");
        assert!((exit[2].amount - 50000.0).abs() < 1e-9, "exit[2]");

        // --- Aggregate net cash flow series ---
        // Period 0: 3000 - 1000 = 2000; Period 1: same; Period 2: 50000 (exit only)
        let net = &s["model.net_cash_flow"].values;
        assert_eq!(net.len(), 3);
        assert!((net[0].amount - 2000.0).abs() < 1e-9, "net[0]");
        assert!((net[1].amount - 2000.0).abs() < 1e-9, "net[1]");
        assert!((net[2].amount - 50000.0).abs() < 1e-9, "net[2]");
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
