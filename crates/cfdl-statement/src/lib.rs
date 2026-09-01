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
    // The pack's recommended category vocabulary, for `W5023`.
    recommended_categories: &[String],
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

    let mut statements: Vec<Statement> = specs
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

    // `W5023` is a fact about the MODEL's vocabulary, not about any one
    // statement's rows, so it is computed once and carried on the default
    // statement rather than repeated on every statement a pack declares.
    let unrecommended = unrecommended_categories(stream_categories, recommended_categories);
    if !unrecommended.is_empty() {
        let index = statements
            .iter()
            .position(|st| st.default)
            .unwrap_or_default();
        if let Some(st) = statements.get_mut(index) {
            let mut merged = unrecommended;
            merged.append(&mut st.diagnostics);
            st.diagnostics = merged;
        }
    }

    Some(StatementsSection {
        pack: Some(pack.to_string()),
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
                    // A stream row refines WITHIN a category — it itemises a
                    // family whose members all carry the same one, which is
                    // why the pack loader can resolve the selector to that
                    // category and count the row toward completeness.
                    //
                    // So an UNCLASSIFIED stream is never claimed by name. A
                    // pack-less model whose stream happens to share a pack's
                    // spelling would otherwise be drawn onto a labelled row
                    // while every subtotal, which folds by category, ignored
                    // it — an "Operating expenses" line above a Net operating
                    // income that does not include it. `dscr_smoke` is exactly
                    // that model. Cash with no category belongs in the
                    // residual row, which is where it already went.
                    let by_name = stream_categories.contains_key(name)
                        && cfdl_expr::selector_matches_any(&row.streams, name);
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
        // A pack statement names no metrics; the clause is the model's.
        metrics: BTreeMap::new(),
    }
}

/// Rows generated from a hierarchy, rather than enumerated by an author.
///
/// `docs/13` §7.55. A pack's statement lists every row it wants; a model's
/// names a STRUCTURE and a DEPTH, and the rows follow from the tree. That is
/// the difference between a presentation you can write in a page and one that
/// needs a page per property.
///
/// **An interior node is a subtotal because of where it sits.** A node whose
/// children are shown is a `subtotal`; a node whose children are cut off by
/// `depth` is a `line`, carrying all of its descendants' cash. That single rule
/// is what keeps the bottom line reconciling at EVERY depth: the lines always
/// partition the cash, whichever level you cut at.
///
/// The two structures read what already exists rather than recomputing it. An
/// entity row is the published `entity.<symbol>.net_cash_flow` rollup, and the
/// tree is the one `graph` publishes (§7.43, §7.91). A category row folds the
/// streams whose published `category` sits under its path.
pub fn generate(
    spec: &ModelStatement,
    grain: &Grain,
    cash_keys: &[&String],
    stream_categories: &BTreeMap<String, String>,
    results: &Results,
    periods: usize,
) -> Statement {
    let series = &results.deterministic.series;
    let mut rows: Vec<StatementRow> = Vec::new();
    let mut diagnostics: Vec<StatementDiagnostic> = Vec::new();
    let mut claimed: BTreeSet<String> = BTreeSet::new();
    let depth_limit = spec.depth.unwrap_or(u32::MAX);

    // THE FILTER, resolved once. A statement scoped to a slice shows the same
    // structure over less cash: the slice's matched streams, within its window.
    // Both halves have to be applied here — the slice's own `net` already has
    // them, but a statement folds the streams, not the net.
    let sliced = spec.slice.as_ref().and_then(|name| {
        results
            .slices
            .as_ref()
            .and_then(|all| all.iter().find(|s| &s.id == name))
    });
    let in_window: Vec<bool> = match sliced.and_then(|s| s.selection.window.as_ref()) {
        None => vec![true; periods],
        Some(window) => {
            let index = results
                .deterministic
                .series
                .values()
                .next()
                .map(|s| &s.index);
            let dates = index
                .map(|ix| {
                    cfdl_engine::timeline_dates(&ix.start, &ix.calendar, ix.periods as usize)
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            (0..periods)
                .map(|t| match dates.get(t) {
                    // ISO dates compare lexically, and the compiler
                    // normalises both bounds to YYYY-MM-DD.
                    Some(d) => {
                        let d = d.to_string();
                        d >= window.from && d <= window.to
                    }
                    None => false,
                })
                .collect()
        }
    };
    // A stream the slice did not match is not in this statement at all — not
    // even as a residual, because the residual answers "what did this
    // STRUCTURE miss", and a filter's exclusions are a different question that
    // the slice's own lineage already answers.
    let owned: Vec<&String>;
    let cash_keys: &[&String] = match sliced {
        None => cash_keys,
        Some(slice) => {
            owned = cash_keys
                .iter()
                .copied()
                .filter(|k| {
                    let name = k
                        .strip_prefix("stream.")
                        .or_else(|| k.strip_prefix("option."))
                        .unwrap_or(k);
                    slice.streams.iter().any(|s| s == name)
                })
                .collect();
            &owned
        }
    };

    // AUTHORED, OR GENERATED — never both. A generated statement partitions
    // the cash by construction, an authored one by the author's care, and a
    // mixture guarantees neither: an authored `line` beside generated rows
    // claims streams they already claimed, so the bottom line double-counts.
    // The compiler refuses the combination; this is the evaluator's half.
    if !spec.rows.is_empty() {
        for row in &spec.rows {
            let display_sign = match row.display.as_deref() {
                Some("positive") | Some("negative") => -1.0,
                _ => 1.0,
            };
            if row.kind == "spacer" {
                rows.push(StatementRow {
                    kind: "spacer".to_string(),
                    label: String::new(),
                    depth: row.depth,
                    display_sign: 1.0,
                    values: vec![],
                    total: None,
                    streams: vec![],
                });
                continue;
            }
            if row.kind == "ratio" {
                // TWO DECLARED SLICES. A slice is already a named selection
                // with a per-period net, so a ratio needs no row identifiers.
                let net_of = |name: &Option<String>| -> Option<Vec<f64>> {
                    let name = name.as_ref()?;
                    let all = results.slices.as_ref()?;
                    let found = all.iter().find(|s| &s.id == name)?;
                    Some(
                        found
                            .net
                            .values
                            .iter()
                            .map(|v| match v {
                                SeriesValue::Money(m) => m.amount,
                                SeriesValue::Number(n) => *n,
                                _ => 0.0,
                            })
                            .collect(),
                    )
                };
                let (Some(num), Some(den)) = (net_of(&row.numerator), net_of(&row.denominator))
                else {
                    continue;
                };
                // RECOMPUTED FROM RE-BUCKETED INPUTS, never re-bucketed
                // itself. An annual coverage ratio is annual NOI over annual
                // debt service; it is NOT the mean of twelve monthly ratios,
                // and no function of a column of ratios gives it. The pack
                // renderer takes the subtotal SPECS for exactly this reason —
                // handing an arm the ratio values and a grain makes averaging
                // them the obvious thing to write. Here the inputs are two
                // slices, so the same discipline is to divide AFTER bucketing.
                let num = grain.sum(&num);
                let den = grain.sum(&den);
                // A zero denominator publishes null, the rule a pack ratio
                // already follows: a coverage ratio with no debt service is
                // genuinely undefined, not zero.
                let values: Vec<SeriesValue> = (0..num.len().max(den.len()))
                    .map(|t| {
                        let d = den.get(t).copied().unwrap_or(0.0);
                        if d.abs() > f64::EPSILON {
                            SeriesValue::Number(round6(num.get(t).copied().unwrap_or(0.0) / d))
                        } else {
                            SeriesValue::Null
                        }
                    })
                    .collect();
                rows.push(StatementRow {
                    kind: "ratio".to_string(),
                    label: row.label.clone(),
                    depth: row.depth,
                    display_sign,
                    values,
                    // Summing a ratio means nothing, so a ratio carries no
                    // total and never reaches the bottom line.
                    total: None,
                    streams: vec![],
                });
                continue;
            }
            let mut acc = vec![0.0_f64; periods];
            let mut drawn: Vec<String> = Vec::new();
            for key in cash_keys {
                let name = key
                    .strip_prefix("stream.")
                    .or_else(|| key.strip_prefix("option."))
                    .unwrap_or(key);
                let by_category = stream_categories
                    .get(name)
                    .is_some_and(|c| row.categories.iter().any(|p| matches_prefix(p, c)));
                let by_name =
                    !row.streams.is_empty() && cfdl_expr::selector_matches_any(&row.streams, name);
                let by_slice = row.slice.as_ref().is_some_and(|slice_name| {
                    results.slices.as_ref().is_some_and(|all| {
                        all.iter()
                            .find(|s| &s.id == slice_name)
                            .is_some_and(|s| s.streams.iter().any(|n| n == name))
                    })
                });
                let by_entity = row.entity.as_ref().is_some_and(|symbol| {
                    series
                        .get(*key)
                        .and_then(|s| s.entity.as_deref())
                        .is_some_and(|owner| owner == symbol)
                });
                if !(by_category || by_name || by_slice || by_entity) {
                    continue;
                }
                // A SUBTOTAL CLAIMS NOTHING. It folds rows stated elsewhere,
                // so counting its streams would double them in the bottom line.
                if row.kind != "subtotal" {
                    claimed.insert((*key).clone());
                    drawn.push((*key).clone());
                }
                if let Some(s) = series.get(*key) {
                    for (t, v) in s.values.iter().enumerate().take(periods) {
                        if in_window[t] {
                            if let SeriesValue::Money(m) = v {
                                acc[t] += m.amount;
                            }
                        }
                    }
                }
            }
            drawn.sort();
            rows.push(StatementRow {
                kind: row.kind.clone(),
                label: row.label.clone(),
                depth: row.depth,
                display_sign,
                // `total` is the lifetime figure over the RAW periods;
                // `values` is what a reader lines up against `grain.labels`.
                total: Some(round6(acc.iter().sum())),
                values: money(&grain.sum(&acc), results),
                streams: drawn,
            });
        }
    } else {
        match spec.structure.as_str() {
            "entity" => {
                // The tree `graph` publishes: symbol -> parent.
                let entities: Vec<(&str, Option<&str>)> = results
                    .graph
                    .as_ref()
                    .map(|g| {
                        g.entities
                            .iter()
                            .map(|e| (e.symbol.as_str(), e.parent.as_deref()))
                            .collect()
                    })
                    .unwrap_or_default();
                let has_shown_child = |symbol: &str, d: u32| -> bool {
                    d + 1 < depth_limit && entities.iter().any(|(_, p)| *p == Some(symbol))
                };
                // DEPTH FIRST — a parent, then ITS subtree, then the next
                // parent. Sorting by (depth, symbol) is breadth first and reads
                // as a list rather than a hierarchy: two funds holding two
                // properties each came out as both funds followed by all the
                // properties in one flat block, with nothing saying which
                // belonged to which. A single-root fixture cannot show that,
                // which is why it survived the first cut.
                //
                // Siblings sort by symbol. Declaration order would read better
                // and is not available: the IR sorts entities by their stable
                // key, so its bytes do not depend on where a declaration sits.
                fn walk<'a>(
                    parent: Option<&str>,
                    depth: u32,
                    entities: &[(&'a str, Option<&'a str>)],
                    out: &mut Vec<(&'a str, u32)>,
                ) {
                    let mut children: Vec<&'a str> = entities
                        .iter()
                        .filter(|(_, p)| *p == parent)
                        .map(|(s, _)| *s)
                        .collect();
                    children.sort_unstable();
                    for child in children {
                        out.push((child, depth));
                        walk(Some(child), depth + 1, entities, out);
                    }
                }
                let mut ordered: Vec<(&str, u32)> = Vec::new();
                walk(None, 0, &entities, &mut ordered);
                for (symbol, d) in ordered {
                    if d >= depth_limit {
                        continue;
                    }
                    let is_subtotal = has_shown_child(symbol, d);
                    // FOLDS ITS SUBTREE rather than reading the published
                    // `entity.<symbol>.net_cash_flow` rollup. The rollup is the
                    // same number and would be cheaper, but it is computed over
                    // ALL of the entity's cash — so a statement scoped to a slice
                    // would silently ignore the filter. Folding the streams the
                    // row actually covers is the only version that stays correct
                    // when something narrows them.
                    let mut values = vec![0.0_f64; periods];
                    let mut drawn: Vec<String> = Vec::new();
                    for key in cash_keys {
                        let Some(owner) = series.get(*key).and_then(|s| s.entity.as_deref()) else {
                            continue;
                        };
                        let mut cursor = Some(owner);
                        let mut under = false;
                        while let Some(current) = cursor {
                            if current == symbol {
                                under = true;
                                break;
                            }
                            cursor = entities
                                .iter()
                                .find(|(s, _)| *s == current)
                                .and_then(|(_, p)| *p);
                        }
                        if !under {
                            continue;
                        }
                        // A LINE claims its subtree's streams; a subtotal claims
                        // nothing, because the rows beneath it will.
                        if !is_subtotal {
                            claimed.insert((*key).clone());
                            drawn.push((*key).clone());
                        }
                        if let Some(s) = series.get(*key) {
                            for (t, v) in s.values.iter().enumerate().take(periods) {
                                if in_window[t] {
                                    if let SeriesValue::Money(m) = v {
                                        values[t] += m.amount;
                                    }
                                }
                            }
                        }
                    }
                    drawn.sort();
                    rows.push(StatementRow {
                        kind: if is_subtotal { "subtotal" } else { "line" }.to_string(),
                        label: derived_label(symbol),
                        depth: d,
                        display_sign: 1.0,
                        total: Some(round6(values.iter().sum())),
                        values: money(&grain.sum(&values), results),
                        streams: drawn,
                    });
                }
            }
            "category" => {
                // The category tree, from the paths the streams declare.
                let mut nodes: BTreeSet<String> = BTreeSet::new();
                for category in stream_categories.values() {
                    let parts: Vec<&str> = category.split('.').collect();
                    for take in 1..=parts.len() {
                        if (take as u32) <= depth_limit {
                            nodes.insert(parts[..take].join("."));
                        }
                    }
                }
                // THE ROOTS HAVE A CANONICAL ORDER and it is not alphabetical.
                // `cfdl_pack::CATEGORY_ROOTS` is operating, investing, financing —
                // the order a cash flow statement is read in — and iterating the
                // set alphabetically put financing first, which is backwards for
                // any statement anyone would want. Below a root there is no
                // canonical order, so siblings sort alphabetically: arbitrary, but
                // stated and stable rather than emergent.
                let root_rank = |node: &str| -> usize {
                    let root = node.split('.').next().unwrap_or_default();
                    cfdl_pack::CATEGORY_ROOTS
                        .iter()
                        .position(|r| *r == root)
                        .unwrap_or(usize::MAX)
                };
                let mut ordered: Vec<String> = nodes.iter().cloned().collect();
                ordered.sort_by(|a, b| root_rank(a).cmp(&root_rank(b)).then(a.cmp(b)));
                for node in &ordered {
                    let d = node.matches('.').count() as u32;
                    let is_subtotal = ordered
                        .iter()
                        .any(|other| other != node && other.starts_with(&format!("{node}.")));
                    let mut acc = vec![0.0_f64; periods];
                    let mut drawn: Vec<String> = Vec::new();
                    for key in cash_keys {
                        let name = key
                            .strip_prefix("stream.")
                            .or_else(|| key.strip_prefix("option."))
                            .unwrap_or(key);
                        let under = stream_categories
                            .get(name)
                            .is_some_and(|c| c == node || c.starts_with(&format!("{node}.")));
                        if !under {
                            continue;
                        }
                        if !is_subtotal {
                            claimed.insert((*key).clone());
                            drawn.push((*key).clone());
                        }
                        if let Some(s) = series.get(*key) {
                            for (t, v) in s.values.iter().enumerate().take(periods) {
                                if in_window[t] {
                                    if let SeriesValue::Money(m) = v {
                                        acc[t] += m.amount;
                                    }
                                }
                            }
                        }
                    }
                    drawn.sort();
                    rows.push(StatementRow {
                        kind: if is_subtotal { "subtotal" } else { "line" }.to_string(),
                        label: derived_label(node),
                        depth: d,
                        display_sign: 1.0,
                        total: Some(round6(acc.iter().sum())),
                        values: money(&grain.sum(&acc), results),
                        streams: drawn,
                    });
                }
            }
            other => {
                diagnostics.push(StatementDiagnostic {
                    code: "W3503_STATEMENT_UNKNOWN_STRUCTURE".to_string(),
                    message: format!(
                        "Statement '{}' asks for structure '{other}', which this engine cannot \
                     build. Known structures: entity, category.",
                        spec.name
                    ),
                });
            }
        }
    }

    // WHAT NO ROW CLAIMED. The same completeness guarantee a pack statement
    // carries: a hierarchy covers its own tree, so a residual here means cash
    // that sits outside the structure — an uncategorised stream, or one owned
    // by no entity — and saying so is the point.
    let mut residual_acc = vec![0.0_f64; periods];
    let mut residual_streams: Vec<String> = Vec::new();
    for key in cash_keys {
        if claimed.contains(*key) {
            continue;
        }
        residual_streams.push((*key).clone());
        if let Some(s) = series.get(*key) {
            for (t, v) in s.values.iter().enumerate().take(periods) {
                if in_window[t] {
                    if let SeriesValue::Money(m) = v {
                        residual_acc[t] += m.amount;
                    }
                }
            }
        }
    }
    if !residual_streams.is_empty() {
        residual_streams.sort();
        rows.push(StatementRow {
            kind: "residual".to_string(),
            label: "Not shown by this structure".to_string(),
            depth: 0,
            display_sign: 1.0,
            total: Some(round6(residual_acc.iter().sum())),
            values: money(&grain.sum(&residual_acc), results),
            streams: residual_streams,
        });
    }

    let bottom_line: f64 = rows
        .iter()
        .filter(|r| r.kind == "line" || r.kind == "residual")
        .filter_map(|r| r.total)
        .sum();
    // WHAT THE STATEMENT IS ACCOUNTABLE FOR. An unfiltered statement must
    // account for the model's cash. A FILTERED one must account for the
    // SLICE's — reconciling it against the model would report the filter as a
    // defect, and a warning that fires on a correct model is noise, which is
    // the standard this codebase already holds ratios to.
    let (universe, universe_label) = match sliced {
        Some(slice) => match slice.metrics.get("total") {
            Some(Scalar::Money(m)) => (m.amount, format!("slice '{}'", slice.id)),
            _ => (0.0, format!("slice '{}'", slice.id)),
        },
        None => (
            match results.deterministic.metrics.get("model.total") {
                Some(Scalar::Money(m)) => m.amount,
                _ => 0.0,
            },
            "model.total".to_string(),
        ),
    };
    let residual = round6(bottom_line - universe);
    const RECONCILES_WITHIN: f64 = 0.005;
    if residual.abs() > RECONCILES_WITHIN {
        diagnostics.push(StatementDiagnostic {
            code: "W3502_STATEMENT_BOTTOM_LINE_RESIDUAL".to_string(),
            message: format!(
                "Statement '{}' totals {:.6} against {universe_label} {:.6}, a residual of {:.6}.",
                spec.name, bottom_line, universe, residual
            ),
        });
    }
    let model_total = universe;

    // THE FIGURES BESIDE THE STATEMENT. A metric is one number at the horizon
    // and every row is a series, which is why they sit in their own map rather
    // than as a row kind: a consumer that renders rows as columns of periods
    // would have nowhere to put a scalar.
    let mut metrics: BTreeMap<String, Scalar> = BTreeMap::new();
    for name in &spec.metrics {
        let key = format!("metric.{name}");
        if let Some(value) = results.deterministic.metrics.get(&key) {
            metrics.insert(name.clone(), value.clone());
        } else if let Some(value) = results.deterministic.metrics.get(name) {
            metrics.insert(name.clone(), value.clone());
        }
    }

    Statement {
        id: spec.name.clone(),
        label: spec.label.clone().unwrap_or_else(|| spec.name.clone()),
        default: false,
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
        metrics,
    }
}

/// Render a model's own statements and attach them to the results.
///
/// Separate from `compute` because the two have different inputs — a pack's
/// specs come from the registry and a model's from the IR — and the same
/// output: `StatementsSection` holds both, in declaration order with the
/// pack's first.
pub fn attach_model_statements(
    specs: &[ModelStatement],
    stream_categories: &BTreeMap<String, String>,
    waterfall_series: &BTreeSet<String>,
    results: &mut Results,
) {
    // A DEFAULT PRESENTATION, when nothing else provides one (`docs/13` §7.43).
    //
    // Without it a reader holding results sees a flat list of series keyed by
    // symbol, and has to rebuild the hierarchy the graph already publishes.
    // Every model has a shape; showing it costs a median of twelve values.
    //
    // A FALLBACK, NOT A DECLARATION. It is assembled here, at render time, and
    // never enters the IR — so it moves neither `model_hash` nor
    // `ledger_hash`, which is what lets it appear without changing what any
    // model IS. And it yields to anything declared: a pack's statements or the
    // model's own mean the presentation question is already answered.
    let owned;
    let mut is_fallback = false;
    let specs: &[ModelStatement] = if specs.is_empty() {
        if results.statements.is_some() || results.graph.is_none() {
            return;
        }
        owned = vec![ModelStatement {
            name: "by_entity".to_string(),
            label: Some("Cash by entity".to_string()),
            structure: "entity".to_string(),
            // The whole tree. A default cannot know which level a reader
            // wants, and showing every node with its children beneath it is
            // what §7.43 asked for.
            depth: None,
            grain: None,
            slice: None,
            metrics: Vec::new(),
            rows: Vec::new(),
        }];
        is_fallback = true;
        &owned
    } else {
        specs
    };
    let periods = results
        .deterministic
        .series
        .values()
        .next()
        .map(|s| s.values.len())
        .unwrap_or_default();
    let cash_keys: Vec<String> = results
        .deterministic
        .series
        .keys()
        .filter(|k| k.starts_with("stream.") || k.starts_with("option."))
        .filter(|k| !waterfall_series.contains(k.as_str()))
        .cloned()
        .collect();
    let borrowed: Vec<&String> = cash_keys.iter().collect();
    let index = results
        .deterministic
        .series
        .values()
        .next()
        .map(|s| s.index.clone());
    let mut rendered: Vec<Statement> = specs
        .iter()
        .map(|spec| {
            let grain = index
                .as_ref()
                .map(|ix| Grain::from_index(ix, spec.grain.as_deref()))
                .unwrap_or_else(|| Grain {
                    calendar: String::new(),
                    start: String::new(),
                    buckets: (0..periods).map(|i| vec![i]).collect(),
                    labels: (0..periods).map(|i| i.to_string()).collect(),
                });
            generate(spec, &grain, &borrowed, stream_categories, results, periods)
        })
        .collect();
    // The fallback IS the statement a consumer means by "the" statement, so it
    // says so. A declared statement does not: which of several is default is
    // the author's call, and nothing here can guess it.
    if is_fallback {
        for statement in &mut rendered {
            statement.default = true;
        }
    }
    match &mut results.statements {
        Some(section) => section.statements.extend(rendered),
        None => {
            results.statements = Some(StatementsSection {
                // A model statement has no pack. The field stays for the
                // pack's own statements, which is why it is optional rather
                // than carrying a sentinel that a consumer would have to know.
                pack: None,
                statements: rendered,
            })
        }
    }
}

/// A readable label for a generated row, from the name it was generated from.
///
/// `operating.revenue.base_rent` becomes "Base rent" and `asset.north` becomes
/// "North": the last path segment, underscores opened out, first letter
/// capitalised. A generated statement is meant to need no declarations, and a
/// row reading `operating.revenue.base_rent` where a pack row says "Base rental
/// revenue" is a presentation that has not been presented. An AUTHORED row
/// states its own label and never comes here.
fn derived_label(path: &str) -> String {
    let last = path.rsplit('.').next().unwrap_or(path);
    let opened = last.replace('_', " ");
    let mut chars = opened.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => opened,
    }
}

/// A model-declared presentation, as the IR carries it.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModelStatement {
    pub name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub structure: String,
    #[serde(default)]
    pub depth: Option<u32>,
    #[serde(default)]
    pub grain: Option<String>,
    #[serde(default)]
    pub slice: Option<String>,
    #[serde(default)]
    pub metrics: Vec<String>,
    #[serde(default)]
    pub rows: Vec<ModelStatementRow>,
}

/// One authored row, as the IR carries it.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModelStatementRow {
    pub kind: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub depth: u32,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub streams: Vec<String>,
    #[serde(default)]
    pub slice: Option<String>,
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub numerator: Option<String>,
    #[serde(default)]
    pub denominator: Option<String>,
    #[serde(default)]
    pub display: Option<String>,
}

/// The model statements an IR carries, if any.
pub fn model_statements(ir: &serde_json::Value) -> Vec<ModelStatement> {
    ir.get("views")
        .and_then(|v| v.get("statements"))
        .and_then(|v| serde_json::from_value::<Vec<ModelStatement>>(v.clone()).ok())
        .unwrap_or_default()
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

/// `W5023` — categories the active pack does not recommend.
///
/// The three roots are the only gate (docs/35): a model may name a leaf the
/// pack never enumerated, and it folds exactly as a listed one does. What the
/// pack's list still carries is the domain's conventional spelling, and this is
/// where that is spent — beside `W3500`, because the consequence of an
/// unrecommended category IS a presentation one: no row of the pack's statement
/// claims it, so it lands in the residual.
///
/// Reported once per distinct category. Thirteen expense lines sharing one
/// misspelling are one mistake.
fn unrecommended_categories(
    stream_categories: &BTreeMap<String, String>,
    recommended: &[String],
) -> Vec<StatementDiagnostic> {
    if recommended.is_empty() {
        return Vec::new();
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for category in stream_categories.values() {
        if category.is_empty() || recommended.iter().any(|c| c == category) {
            continue;
        }
        if !seen.insert(category.as_str()) {
            continue;
        }
        // One edit apart, the bar the compiler already uses for a misspelled
        // term. A looser bar suggests confidently and wrongly.
        let advice = match recommended
            .iter()
            .find(|candidate| edit_distance_at_most_one(candidate, category))
        {
            Some(candidate) => format!(" Did you mean '{candidate}'?"),
            None => String::new(),
        };
        out.push(StatementDiagnostic {
            code: "W5023_UNRECOGNISED_PACK_CATEGORY".to_string(),
            message: format!(
                "Category '{category}' is not one this pack recommends. It is valid and \
                 folds correctly — the three roots are the only gate — but no row of a \
                 pack statement claims it, so it reports in the residual.{advice}"
            ),
        });
    }
    out
}

/// One insertion, deletion or substitution apart.
fn edit_distance_at_most_one(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    if a.len().abs_diff(b.len()) > 1 {
        return false;
    }
    let (mut i, mut j, mut edits) = (0usize, 0usize, 0u8);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            i += 1;
            j += 1;
            continue;
        }
        edits += 1;
        if edits > 1 {
            return false;
        }
        match a.len().cmp(&b.len()) {
            std::cmp::Ordering::Greater => i += 1,
            std::cmp::Ordering::Less => j += 1,
            std::cmp::Ordering::Equal => {
                i += 1;
                j += 1;
            }
        }
    }
    edits + u8::from(i < a.len() || j < b.len()) <= 1
}
