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
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Io(err) => write!(f, "I/O error: {err}"),
            EngineError::Json(err) => write!(f, "JSON error: {err}"),
            EngineError::InvalidDate(value) => write!(f, "invalid ISO date: {value}"),
            EngineError::InvalidRunConfig(message) => write!(f, "invalid run config: {message}"),
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
    pub distributions: BTreeMap<String, DistributionSpec>,
}

#[derive(Debug, Clone)]
pub enum DistributionSpec {
    Fixed { value: f64 },
    Normal { mean: f64, stddev: f64 },
    Uniform { min: f64, max: f64 },
}

#[derive(Debug, Deserialize)]
struct RunConfigFile {
    #[serde(default)]
    deterministic: DeterministicConfigFile,
    #[serde(default)]
    scenarios: BTreeMap<String, ScenarioConfigFile>,
    monte_carlo: Option<MonteCarloConfigFile>,
}

#[derive(Debug, Default, Deserialize)]
struct DeterministicConfigFile {
    discount_rate: Option<f64>,
    as_of: Option<String>,
    #[serde(default)]
    parameters: BTreeMap<String, f64>,
}

#[derive(Debug, Default, Deserialize)]
struct ScenarioConfigFile {
    discount_rate: Option<f64>,
    as_of: Option<String>,
    #[serde(default)]
    parameters: BTreeMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct MonteCarloConfigFile {
    trial_count: u32,
    seed: u64,
    #[serde(default)]
    distributions: BTreeMap<String, DistributionConfigFile>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DistributionConfigFile {
    Fixed { value: f64 },
    Normal { mean: f64, stddev: f64 },
    Uniform { min: f64, max: f64 },
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
            let parsed = match dist {
                DistributionConfigFile::Fixed { value } => DistributionSpec::Fixed { value },
                DistributionConfigFile::Normal { mean, stddev } => {
                    if stddev < 0.0 {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' has negative stddev"
                        )));
                    }
                    DistributionSpec::Normal { mean, stddev }
                }
                DistributionConfigFile::Uniform { min, max } => {
                    if min > max {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' has min > max"
                        )));
                    }
                    DistributionSpec::Uniform { min, max }
                }
            };
            distributions.insert(name, parsed);
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
    let base_run = run_deterministic(ir, &config)?;
    let mut warnings = base_run.warnings.clone();

    let deterministic = DeterministicSection {
        status: "ok".to_string(),
        metrics: base_run.metrics.clone(),
        series: base_run.series,
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
                let sampled = sample_distribution(distribution, &mut rng_state);
                trial_overrides.insert(name.clone(), sampled);
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
        results_version: "0.1".to_string(),
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
    })
}

#[derive(Debug, Clone)]
struct DeterministicRunOutput {
    warnings: Vec<String>,
    metrics: BTreeMap<String, Scalar>,
    series: BTreeMap<String, MoneySeries>,
    npv: f64,
}

fn run_deterministic(ir: &Ir, config: &RunConfig) -> Result<DeterministicRunOutput, EngineError> {
    let timeline = timeline_dates(&ir.time.start, &ir.time.calendar, ir.time.periods as usize)?;
    let periods = timeline.len();

    let mut warnings = Vec::new();
    let mut stream_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut stream_totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut entity_totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut model_series = vec![0.0_f64; periods];

    for stream in &ir.streams {
        if let Some(lang) = &stream.amount.lang {
            if lang != "cel" {
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
                // Deterministic fallback expression.
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
                if lang != "cel" {
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

        let schedule_mask = schedule_mask(&stream.schedule, &timeline)?;
        let mut values = vec![0.0_f64; periods];
        let direction_sign = stream_direction_sign(stream, &mut warnings);
        for (idx, is_scheduled) in schedule_mask.iter().copied().enumerate() {
            if !is_scheduled {
                continue;
            }
            let env = build_expr_env(ir, stream, config, idx, &timeline[idx]);
            let active_value = eval_bool_expr(
                &active_expr,
                &env,
                &stream.name,
                "active_when",
                &mut warnings,
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
                    &mut warnings,
                )
            };
            values[idx] += amount * direction_sign;
        }
        for (idx, value) in values.iter().enumerate() {
            model_series[idx] += *value;
        }

        let total = values.iter().sum::<f64>();
        stream_totals.insert(stream.name.clone(), total);
        entity_totals
            .entry(stream.owner.symbol.clone())
            .and_modify(|sum| *sum += total)
            .or_insert(total);
        stream_series.insert(stream.name.clone(), values);
    }

    let mut series_map = BTreeMap::new();
    for (name, values) in stream_series {
        series_map.insert(
            format!("stream.{name}"),
            MoneySeries::from_values(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &ir.model.currency,
                &values,
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
    let npv = npv(&model_series, config.discount_rate);
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
    metrics.insert(
        "run.discount_rate".to_string(),
        Scalar::Number(round_amount(config.discount_rate)),
    );
    if let Some(as_of) = &config.as_of {
        metrics.insert("run.as_of".to_string(), Scalar::String(as_of.to_string()));
    }

    Ok(DeterministicRunOutput {
        warnings,
        metrics,
        series: series_map,
        npv,
    })
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

fn build_expr_env(
    ir: &Ir,
    stream: &IrStream,
    config: &RunConfig,
    t: usize,
    date: &Date,
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

    for (key, value) in &config.parameter_overrides {
        if let Some(stripped) = key.strip_prefix("cfg.") {
            insert_cfg_value(&mut env.cfg, stripped, *value);
        }
    }
    env
}

fn stream_amount_override(config: &RunConfig, stream_name: &str) -> Option<f64> {
    let legacy_key = format!("stream.{stream_name}.amount");
    if let Some(value) = config.parameter_overrides.get(&legacy_key) {
        return Some(*value);
    }
    let structured_key = format!("stream[\"{stream_name}\"].amount");
    config.parameter_overrides.get(&structured_key).copied()
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

fn schedule_mask(schedule: &IrSchedule, timeline: &[Date]) -> Result<Vec<bool>, EngineError> {
    let mut values = vec![0.0_f64; timeline.len()];
    apply_schedule(schedule, 1.0, timeline, &mut values)?;
    Ok(values.into_iter().map(|value| value.abs() > 0.0).collect())
}

fn npv(values: &[f64], rate: f64) -> f64 {
    let mut total = 0.0_f64;
    for (i, value) in values.iter().enumerate() {
        let discount = (1.0 + rate).powi(i as i32);
        total += *value / discount;
    }
    total
}

fn apply_schedule(
    schedule: &IrSchedule,
    amount: f64,
    timeline: &[Date],
    out_values: &mut [f64],
) -> Result<(), EngineError> {
    match schedule.kind.as_str() {
        "OnDate" => {
            if let Some(on) = &schedule.on {
                let target = Date::parse(on)?;
                if let Some(idx) = timeline.iter().position(|d| *d == target) {
                    out_values[idx] += amount;
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
            for (idx, date) in timeline.iter().enumerate() {
                if *date >= from_date && *date <= to_date {
                    out_values[idx] += amount;
                }
            }
        }
        _ => {}
    }
    Ok(())
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
struct IrSchedule {
    kind: String,
    on: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IrExpr {
    #[serde(default)]
    lang: Option<String>,
    src: String,
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
                    "amount": { "lang": "cel", "src": "cfg.base + time.t" },
                    "active_when": { "lang": "cel", "src": "time.t < 2" }
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
    fn supports_legacy_and_structured_stream_amount_override_keys() {
        let ir = r#"{
            "model": { "name": "demo", "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 2 },
            "streams": [
                {
                    "name": "cre.lease.base_rent",
                    "owner": { "symbol": "legal.borrower" },
                    "direction": "inflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-02-01" },
                    "amount": { "lang": "cel", "src": "10" },
                    "active_when": { "lang": "cel", "src": "true" }
                }
            ]
        }"#;

        let mut legacy = BTreeMap::new();
        legacy.insert("stream.cre.lease.base_rent.amount".to_string(), 25.0);
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
        .expect("legacy override run");

        let mut structured = BTreeMap::new();
        structured.insert("stream[\"cre.lease.base_rent\"].amount".to_string(), 25.0);
        let structured_results = run_from_json_str(
            ir,
            RunConfig {
                discount_rate: 0.0,
                as_of: None,
                parameter_overrides: structured,
                scenarios: BTreeMap::new(),
                monte_carlo: None,
            },
        )
        .expect("structured override run");

        let legacy_total = legacy_results
            .deterministic
            .metrics
            .get("stream.cre.lease.base_rent.total")
            .expect("stream metric");
        let structured_total = structured_results
            .deterministic
            .metrics
            .get("stream.cre.lease.base_rent.total")
            .expect("stream metric");
        let legacy_total = match legacy_total {
            super::Scalar::Money(money) => money.amount,
            other => panic!("expected money scalar, got {other:?}"),
        };
        let structured_total = match structured_total {
            super::Scalar::Money(money) => money.amount,
            other => panic!("expected money scalar, got {other:?}"),
        };
        assert_eq!(legacy_total, structured_total);
        assert_eq!(legacy_total, 50.0);
    }
}
