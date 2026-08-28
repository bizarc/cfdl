//! `explain`: a series + period -> the journal slice that produced the number.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainParams {
    /// The results document (docs/06), inline. Alternative to `results_path`.
    #[serde(default)]
    pub results: Option<serde_json::Value>,
    /// Path to a results JSON file.
    #[serde(default)]
    pub results_path: Option<String>,
    /// A series key from `deterministic.series` (e.g. `domain.cre.noi`,
    /// `stream.rent`, `account.reserve`, `model.net_cash_flow`).
    pub series: String,
    /// The 0-based grid period to explain.
    pub period: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ExplainResult {
    pub series: String,
    pub period: u64,
    /// The period's date, when the journal names it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// The value at that period. Null means the series is genuinely undefined
    /// there (e.g. a coverage ratio with no debt service), not zero.
    pub value: Option<f64>,
    /// The neighboring values, for reading a step or a carry at a glance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<f64>,
    /// Journal entries for this period whose actor or target names the series
    /// (matched on the full key and on its last segment).
    pub journal: Vec<serde_json::Value>,
    /// How many journal entries the period has in total — when larger than
    /// `journal.len()`, other actions also ran that period.
    pub period_entries_total: u64,
    pub note: String,
}

pub fn explain(
    params: &ExplainParams,
    defaults: &super::Defaults,
) -> Result<ExplainResult, String> {
    let _ = defaults;
    let results = super::load_results(params.results.clone(), params.results_path.as_deref())?;
    let Some(series_map) = results["deterministic"]["series"].as_object() else {
        return Err("results carry no `deterministic.series`".to_string());
    };
    let Some(series) = series_map.get(&params.series) else {
        let mut available: Vec<&str> = series_map.keys().map(String::as_str).collect();
        available.truncate(40);
        return Err(format!(
            "no series '{}' in results; available: {}",
            params.series,
            available.join(", ")
        ));
    };
    let values = series["values"].as_array().cloned().unwrap_or_default();
    let t = params.period as usize;
    if t >= values.len() {
        return Err(format!(
            "period {} is beyond the {}-period timeline",
            params.period,
            values.len()
        ));
    }
    let value = super::scalar(&values[t]);
    let prev = t
        .checked_sub(1)
        .and_then(|p| values.get(p))
        .and_then(super::scalar);
    let next = values.get(t + 1).and_then(super::scalar);

    // The journal is flat — one row per act, with a kind-qualified actor and a
    // free-form target ("top_up -> reserve"). Match on the full series key and
    // on its last segment, which is how waterfall steps and accounts appear.
    let last_segment = params.series.rsplit('.').next().unwrap_or(&params.series);
    let mut matched = Vec::new();
    let mut period_entries_total = 0u64;
    let mut date = None;
    for entry in results["deterministic"]["journal"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if entry["period"].as_u64() != Some(params.period) {
            continue;
        }
        period_entries_total += 1;
        if date.is_none() {
            date = entry["date"].as_str().map(str::to_string);
        }
        let names = |field: &str| entry[field].as_str().unwrap_or("");
        let text = format!("{} {}", names("actor"), names("target"));
        if text.contains(&params.series) || text.contains(last_segment) {
            matched.push(entry.clone());
        }
    }

    let note = if matched.is_empty() && period_entries_total > 0 {
        format!(
            "no journal entry names '{}' in period {}; {} other entries ran that period \
             (the value may be a schedule/expression evaluation, which the journal does not row)",
            params.series, params.period, period_entries_total
        )
    } else if period_entries_total == 0 {
        "the run journaled no actions this period; the value comes from stream \
         evaluation on its schedule"
            .to_string()
    } else {
        format!(
            "{} of {} journal entries this period name the series",
            matched.len(),
            period_entries_total
        )
    };

    Ok(ExplainResult {
        series: params.series.clone(),
        period: params.period,
        date,
        value,
        prev,
        next,
        journal: matched,
        period_entries_total,
        note,
    })
}
