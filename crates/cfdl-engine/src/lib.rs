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
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Io(err) => write!(f, "I/O error: {err}"),
            EngineError::Json(err) => write!(f, "JSON error: {err}"),
            EngineError::InvalidDate(value) => write!(f, "invalid ISO date: {value}"),
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
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            discount_rate: 0.0,
            as_of: None,
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

fn compute_results(ir: &Ir, model_hash: String, config: RunConfig) -> Result<Results, EngineError> {
    let timeline = timeline_dates(&ir.time.start, &ir.time.calendar, ir.time.periods as usize)?;
    let periods = timeline.len();

    let mut warnings = Vec::new();
    let mut stream_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut stream_totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut entity_totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut model_series = vec![0.0_f64; periods];

    for stream in &ir.streams {
        let amount = parse_cel_decimal(&stream.amount.src).unwrap_or_else(|| {
            warnings.push(format!(
                "Stream '{}' amount '{}' is not a numeric literal; using 0.",
                stream.name, stream.amount.src
            ));
            0.0
        });
        let signed_amount = match stream.direction.as_str() {
            "inflow" => amount,
            "outflow" => -amount,
            _ => {
                warnings.push(format!(
                    "Stream '{}' has unknown direction '{}'; treating as outflow.",
                    stream.name, stream.direction
                ));
                -amount
            }
        };

        let mut values = vec![0.0_f64; periods];
        apply_schedule(&stream.schedule, signed_amount, &timeline, &mut values)?;
        for (idx, value) in values.iter().enumerate() {
            model_series[idx] += *value;
        }

        let total = values.iter().sum::<f64>();
        stream_totals.insert(stream.name.clone(), round_amount(total));
        entity_totals
            .entry(stream.owner.symbol.clone())
            .and_modify(|sum| *sum = round_amount(*sum + total))
            .or_insert_with(|| round_amount(total));
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
    if let Some(as_of) = config.as_of {
        metrics.insert("run.as_of".to_string(), Scalar::String(as_of.to_string()));
    }

    Ok(Results {
        results_version: "0.1".to_string(),
        model_hash,
        engine: EngineInfo {
            name: "cfdl-engine".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build: None,
        },
        warnings,
        deterministic: DeterministicSection {
            status: "ok".to_string(),
            metrics,
            series: series_map,
            errors: None,
        },
        monte_carlo: MonteCarloSection {
            status: "not_run".to_string(),
            trials: 1,
            seed: 0,
            metrics: BTreeMap::new(),
            errors: None,
        },
    })
}

fn parse_cel_decimal(src: &str) -> Option<f64> {
    src.trim().parse::<f64>().ok()
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
    src: String,
}

#[derive(Debug, Serialize)]
pub struct Results {
    pub results_version: String,
    pub model_hash: String,
    pub engine: EngineInfo,
    pub warnings: Vec<String>,
    pub deterministic: DeterministicSection,
    pub monte_carlo: MonteCarloSection,
}

#[derive(Debug, Serialize)]
pub struct EngineInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeterministicSection {
    pub status: String,
    pub metrics: BTreeMap<String, Scalar>,
    pub series: BTreeMap<String, MoneySeries>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RuntimeError>>,
}

#[derive(Debug, Serialize)]
pub struct MonteCarloSection {
    pub status: String,
    pub trials: u32,
    pub seed: u64,
    pub metrics: BTreeMap<String, MetricSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RuntimeError>>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Scalar {
    Number(f64),
    Money(Money),
    String(String),
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct SeriesIndex {
    pub calendar: String,
    pub start: String,
    pub periods: u32,
}

#[derive(Debug, Serialize)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::{run_from_json_str, RunConfig};

    #[test]
    fn deterministic_output_for_identical_input() {
        let ir = r#"{
            "model": { "currency": "USD" },
            "time": { "calendar": "monthly", "start": "2026-01-01", "periods": 3 },
            "streams": [
                {
                    "name": "rent",
                    "owner": { "symbol": "legal.borrower" },
                    "direction": "outflow",
                    "schedule": { "kind": "Every", "from": "2026-01-01", "to": "2026-03-01" },
                    "amount": { "src": "100" }
                }
            ]
        }"#;

        let first = run_from_json_str(
            ir,
            RunConfig {
                discount_rate: 0.05,
                as_of: None,
            },
        )
        .unwrap();
        let second = run_from_json_str(
            ir,
            RunConfig {
                discount_rate: 0.05,
                as_of: None,
            },
        )
        .unwrap();
        let a = serde_json::to_string(&first).unwrap();
        let b = serde_json::to_string(&second).unwrap();
        assert_eq!(a, b);
    }
}
