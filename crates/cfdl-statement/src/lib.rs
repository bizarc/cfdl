//! Render a pack's declared statements against engine results.
//!
//! A post-pass, like `cfdl-metrics`, and for the same reason: order, labels and
//! display signs are consumed only by readers, so they have no business in the
//! engine. Everything numeric here was already folded per period by the engine;
//! this arranges it.
//!
//! The one thing this does compute is completeness. A statement's bottom line
//! has to reconcile to the model's cash, and it only does so if every cash
//! stream landed in exactly one row. The pack loader checks that statically for
//! CATEGORIES; this checks it for the streams a run actually produced, which is
//! the half a static check cannot do — a hand-written stream carrying no
//! category at all is invisible until something runs.

use cfdl_engine::{
    Grain, Money, Results, Scalar, SeriesValue, Statement, StatementDiagnostic, StatementGrain,
    StatementReconciliation, StatementRow, StatementsSection,
};
use cfdl_pack::{StatementSpec, SubtotalSpec};
use std::collections::{BTreeMap, BTreeSet};

/// Render every statement the pack declares. `None` when it declares none.
pub fn compute(
    pack: &str,
    specs: &[StatementSpec],
    subtotals: &[SubtotalSpec],
    stream_categories: &BTreeMap<String, String>,
    waterfall_series: &BTreeSet<String>,
    results: &Results,
) -> Option<StatementsSection> {
    if specs.is_empty() {
        return None;
    }
    let series = &results.deterministic.series;
    let periods = series
        .values()
        .next()
        .map(|s| s.values.len())
        .unwrap_or_default();

    // The cash universe a statement must account for. `state.` and `domain.`
    // are excluded by prefix rather than by rule: a state is not cash, and a
    // subtotal is a fold OF the cash, so counting either would double what it
    // touches.
    //
    // WATERFALL STEPS ARE EXCLUDED BY NAME, because a prefix cannot reach them:
    // a step publishes as `stream.<waterfall>.<step>` and is indistinguishable
    // from a stream by its key alone. A waterfall runs AFTER the cash it
    // divides has been computed — it distributes free cash flow rather than
    // producing any — which is why `model.total` already leaves the steps out.
    // Counting them here made the statement disagree with the model by the
    // whole distributed amount: 4,000 on waterfall_smoke, 202.7M on the
    // monthly-grain flip, reported as W3502 beside the W3500 that named them.
    let cash_keys: Vec<&String> = series
        .keys()
        .filter(|k| k.starts_with("stream.") || k.starts_with("option."))
        .filter(|k| !waterfall_series.contains(k.as_str()))
        .collect();

    // The index every series shares, used to rebuild the timeline a coarser
    // grain needs. A post-pass has no timeline of its own.
    let index = series.values().next().map(|s| s.index.clone());

    let statements = specs
        .iter()
        .map(|spec| {
            let grain = index
                .as_ref()
                .map(|ix| Grain::from_index(ix, spec.grain.as_deref()))
                .unwrap_or_else(|| Grain {
                    calendar: String::new(),
                    start: String::new(),
                    buckets: (0..periods).map(|i| vec![i]).collect(),
                    // No index means no dates to label with; a bare ordinal is
                    // honest about that rather than inventing a calendar.
                    labels: (0..periods).map(|i| i.to_string()).collect(),
                });
            render(
                spec,
                subtotals,
                &grain,
                &cash_keys,
                stream_categories,
                results,
                periods,
            )
        })
        .collect();
    Some(StatementsSection {
        pack: pack.to_string(),
        statements,
    })
}

fn render(
    spec: &StatementSpec,
    subtotals: &[SubtotalSpec],
    grain: &Grain,
    cash_keys: &[&String],
    stream_categories: &BTreeMap<String, String>,
    results: &Results,
    periods: usize,
) -> Statement {
    let series = &results.deterministic.series;
    let mut rows: Vec<StatementRow> = Vec::new();
    let mut claimed: BTreeMap<&str, usize> = BTreeMap::new();

    for row in &spec.rows {
        let display_sign = match row.display.as_deref() {
            Some("positive") | Some("negative") => -1.0,
            _ => 1.0,
        };
        match row.kind.as_str() {
            "spacer" => rows.push(StatementRow {
                kind: "spacer".to_string(),
                label: String::new(),
                depth: row.depth,
                display_sign: 1.0,
                values: vec![],
                total: None,
                streams: vec![],
            }),
            "line" => {
                let mut acc = vec![0.0_f64; periods];
                let mut drawn: Vec<String> = Vec::new();
                for key in cash_keys {
                    let name = key
                        .strip_prefix("stream.")
                        .or_else(|| key.strip_prefix("option."))
                        .unwrap_or(key);
                    let by_category = stream_categories.get(name).is_some_and(|c| {
                        row.categories
                            .iter()
                            .any(|sel| sel == c || matches_prefix(sel, c))
                    });
                    let by_name = cfdl_expr::selector_matches_any(&row.streams, name);
                    if !(by_category || by_name) {
                        continue;
                    }
                    *claimed.entry(key.as_str()).or_insert(0) += 1;
                    drawn.push(name.to_string());
                    if let Some(s) = series.get(*key) {
                        for (t, v) in s.values.iter().enumerate().take(periods) {
                            if let SeriesValue::Money(m) = v {
                                acc[t] += m.amount;
                            }
                        }
                    }
                }
                let total = acc.iter().sum::<f64>();
                let acc = grain.sum(&acc);
                rows.push(StatementRow {
                    kind: "line".to_string(),
                    label: row.label.clone(),
                    depth: row.depth,
                    display_sign,
                    values: money(&acc, results),
                    total: Some(round6(total)),
                    streams: drawn,
                });
            }
            "subtotal" => {
                let id = row.subtotal.clone().unwrap_or_default();
                let (values, total) = match series.get(&id) {
                    Some(s) => {
                        let raw: Vec<f64> = s
                            .values
                            .iter()
                            .map(|v| match v {
                                SeriesValue::Money(m) => m.amount,
                                SeriesValue::Number(n) => *n,
                                SeriesValue::Null => 0.0,
                            })
                            .collect();
                        let tot = round6(raw.iter().sum::<f64>());
                        (money(&grain.sum(&raw), results), Some(tot))
                    }
                    None => (vec![], None),
                };
                let kind = "subtotal";
                rows.push(StatementRow {
                    kind: kind.to_string(),
                    label: row.label.clone(),
                    depth: row.depth,
                    display_sign,
                    values,
                    total,
                    streams: vec![],
                });
            }
            "ratio" => {
                // THE CORRECTNESS TRAP, and why this arm takes the SPECS rather
                // than the published ratio series.
                //
                // An annual coverage ratio is annual NOI over annual debt
                // service. It is NOT the mean of twelve monthly ratios, and it
                // is not any other function of them — a column of ratios cannot
                // be re-bucketed at all. So a coarse grain recomputes it from
                // its inputs, which means knowing what its inputs ARE, which is
                // in the subtotal declaration and not in the series.
                //
                // Handing this arm the ratio values and a grain would have made
                // averaging them the obvious thing to write. Handing it the
                // specs makes the wrong version unavailable.
                let id = row.subtotal.clone().unwrap_or_default();
                let spec = subtotals.iter().find(|s| s.id == id);
                //
                // The inputs must be PRESENT, not merely declared. A pack always
                // declares the spec, but a model whose streams carry no category
                // publishes no subtotal series at all — and `Grain::sum` of an
                // absent series is not an empty vector, it is one zero per
                // bucket. Recomputing from that gives a zero denominator in every
                // period and a full column of nulls, which says "undefined here"
                // about a ratio that was never computed. `dscr_smoke` is exactly
                // that model, and it is how this was caught.
                let fetch = |k: &str| -> Option<Vec<f64>> {
                    series.get(k).map(|s| {
                        s.values
                            .iter()
                            .map(|v| match v {
                                SeriesValue::Money(m) => m.amount,
                                SeriesValue::Number(n) => *n,
                                SeriesValue::Null => 0.0,
                            })
                            .collect()
                    })
                };
                let inputs =
                    spec.and_then(|s| match (s.numerator.as_ref(), s.denominator.as_ref()) {
                        (Some(n), Some(d)) => fetch(n).zip(fetch(d)),
                        _ => None,
                    });
                let values = match inputs {
                    Some((num_raw, den_raw)) => {
                        let num = grain.sum(&num_raw);
                        let den = grain.sum(&den_raw);
                        num.iter()
                            .zip(den.iter())
                            .map(|(n, d)| {
                                if d.abs() > f64::EPSILON {
                                    SeriesValue::Number(round6(n / d))
                                } else {
                                    SeriesValue::Null
                                }
                            })
                            .collect()
                    }
                    // Not a declared ratio, or its inputs were never published.
                    // Falls back to the ratio series itself, which is absent in
                    // the same cases — so the row carries no values rather than
                    // a column of manufactured ones.
                    None => series
                        .get(&id)
                        .map(|s| s.values.clone())
                        .unwrap_or_default(),
                };
                rows.push(StatementRow {
                    kind: "ratio".to_string(),
                    label: row.label.clone(),
                    depth: row.depth,
                    display_sign,
                    values,
                    // No total: summing a column of coverage ratios answers no
                    // question anyone asks.
                    total: None,
                    streams: vec![],
                });
            }
            _ => {}
        }
    }

    // --- completeness -------------------------------------------------------
    let mut diagnostics: Vec<StatementDiagnostic> = Vec::new();

    let unclaimed: Vec<&String> = cash_keys
        .iter()
        .filter(|k| !claimed.contains_key(k.as_str()))
        .copied()
        .collect();
    if !unclaimed.is_empty() {
        // Visible, not absorbed. A statement that silently omits cash is worse
        // than one that shows an ugly row, because the ugly row is the only
        // signal a reader gets that the bottom line is short.
        let mut acc = vec![0.0_f64; periods];
        let mut names: Vec<String> = Vec::new();
        for key in &unclaimed {
            names.push(
                key.strip_prefix("stream.")
                    .or_else(|| key.strip_prefix("option."))
                    .unwrap_or(key)
                    .to_string(),
            );
            if let Some(s) = series.get(*key) {
                for (t, v) in s.values.iter().enumerate().take(periods) {
                    if let SeriesValue::Money(m) = v {
                        acc[t] += m.amount;
                    }
                }
            }
        }
        let total = acc.iter().sum::<f64>();
        let acc = grain.sum(&acc);
        diagnostics.push(StatementDiagnostic {
            code: "W3500_STATEMENT_UNCLASSIFIED_STREAM".to_string(),
            message: format!(
                "{} stream(s) are in no row of statement '{}', so the bottom line is short by \
                 {:.2}: {}. Classify them, or give the statement a row that claims them.",
                names.len(),
                spec.id,
                total,
                names.join(", ")
            ),
        });
        rows.push(StatementRow {
            kind: "residual".to_string(),
            label: "Unclassified".to_string(),
            depth: 1,
            display_sign: 1.0,
            values: money(&acc, results),
            total: Some(round6(total)),
            streams: names,
        });
    }

    let doubled: Vec<&str> = claimed
        .iter()
        .filter(|(_, n)| **n > 1)
        .map(|(k, _)| *k)
        .collect();
    if !doubled.is_empty() {
        // Worse than omission: the bottom line is wrong in a direction that
        // looks plausible.
        diagnostics.push(StatementDiagnostic {
            code: "W3501_STATEMENT_STREAM_DOUBLE_COUNTED".to_string(),
            message: format!(
                "{} stream(s) appear in more than one row of statement '{}', so their cash is \
                 counted twice: {}.",
                doubled.len(),
                spec.id,
                doubled.join(", ")
            ),
        });
    }

    // --- reconciliation -----------------------------------------------------
    // The bottom line is every LINE row plus any residual — not the subtotals,
    // which are folds of those same lines and would double them.
    let bottom_line: f64 = rows
        .iter()
        .filter(|r| r.kind == "line" || r.kind == "residual")
        .filter_map(|r| r.total)
        .sum();
    let model_total = match results.deterministic.metrics.get("model.total") {
        Some(Scalar::Money(m)) => m.amount,
        _ => 0.0,
    };
    let residual = round6(bottom_line - model_total);
    // Half a cent. Not 1e-6: every row total is rounded to six decimals before
    // being summed, so a statement of N rows carries up to N * 5e-7 of pure
    // presentation rounding — about 1e-5 here, which tripped the gate on a
    // statement that reconciles exactly. The question this asks is "does the
    // statement account for the model's cash", and money that agrees to within
    // half a cent does.
    const RECONCILES_WITHIN: f64 = 0.005;
    if residual.abs() > RECONCILES_WITHIN {
        diagnostics.push(StatementDiagnostic {
            code: "W3502_STATEMENT_BOTTOM_LINE_RESIDUAL".to_string(),
            message: format!(
                "Statement '{}' totals {:.6} against model.total {:.6}, a residual of {:.6}.",
                spec.id, bottom_line, model_total, residual
            ),
        });
    }

    Statement {
        id: spec.id.clone(),
        label: spec.label.clone(),
        default: spec.default,
        grain: StatementGrain {
            calendar: grain.calendar.clone(),
            start: grain.start.clone(),
            labels: grain.labels.clone(),
        },
        rows,
        reconciliation: StatementReconciliation {
            bottom_line: round6(bottom_line),
            model_total: round6(model_total),
            residual,
        },
        diagnostics,
    }
}

/// `operating.*` reaches `operating.revenue.base_rent`. The same path-segment
/// rule the one selector dialect uses; spelled out here because a row's
/// categories are usually exact and the glob is the exception.
fn matches_prefix(selector: &str, category: &str) -> bool {
    cfdl_expr::selector_matches(selector, category)
}

fn money(values: &[f64], results: &Results) -> Vec<SeriesValue> {
    let currency = results
        .deterministic
        .metrics
        .values()
        .find_map(|s| match s {
            Scalar::Money(m) => Some(m.currency.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "USD".to_string());
    values
        .iter()
        .map(|v| {
            SeriesValue::Money(Money {
                amount: round6(*v),
                currency: currency.clone(),
            })
        })
        .collect()
}

fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

/// Stream name -> category, read from the IR the run was produced from.
///
/// The engine does not republish a stream's category, so a caller passes it in.
/// Kept as a free function so every host builds it the same way.
/// Every series key a waterfall publishes, as `stream.<waterfall>.<step>`.
///
/// Read from the IR because the results cannot tell one apart: a step's series
/// is keyed exactly like a stream's, carries no category, and so would land in
/// a statement's residual row — which is cash the model never counted.
pub fn waterfall_series(ir: &serde_json::Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(waterfalls) = ir.get("waterfalls").and_then(|w| w.as_array()) {
        for w in waterfalls {
            let Some(name) = w.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(steps) = w.get("steps").and_then(|s| s.as_array()) else {
                continue;
            };
            for step in steps {
                if let Some(step_name) = step.get("name").and_then(|v| v.as_str()) {
                    out.insert(format!("stream.{name}.{step_name}"));
                }
            }
        }
    }
    out
}

pub fn stream_categories(ir: &serde_json::Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(streams) = ir.get("streams").and_then(|s| s.as_array()) {
        for s in streams {
            if let (Some(name), Some(cat)) = (
                s.get("name").and_then(|v| v.as_str()),
                s.get("category").and_then(|v| v.as_str()),
            ) {
                out.insert(name.to_string(), cat.to_string());
            }
        }
    }
    out
}
