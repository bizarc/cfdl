// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

/// Run a model's waterfalls, after this period's streams and states are known.
///
/// A waterfall is an author-declared priority over a pot. Steps evaluate in
/// declaration order; each takes `min(max(0, owed), remaining)` and reduces
/// what is left. The clamp is here rather than in the author's expression, so
/// the pot cannot go negative however a step is written and `= remaining`
/// means exactly what survives.
///
/// Each step becomes a stream — `stream.<waterfall>.<step>` — so a waterfall
/// adds no new kind of output and statements, metrics and the results schema
/// are untouched.
#[allow(clippy::too_many_arguments)]
/// Each entity's netted stream cash per period, rolled up by `part of`.
///
/// STREAMS ONLY: this is the cash the collateral produced, before any
/// distribution, which is what `docs/17` §4 names as a waterfall's pot. The
/// results layer builds the same fold again with waterfall payments attributed
/// to payees; that one is a report of what happened, this one is the input to
/// what happens next, and the two must stay distinct or a waterfall could read
/// its own output.
pub(crate) fn stream_cash_by_entity(
    ir: &Ir,
    stream_series: &BTreeMap<String, Vec<f64>>,
    periods: usize,
) -> BTreeMap<String, Vec<f64>> {
    let mut own: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for stream in &ir.streams {
        if let Some(values) = stream_series.get(&stream.name) {
            let slot = own
                .entry(stream.owner.symbol.clone())
                .or_insert_with(|| vec![0.0; periods]);
            for (idx, value) in values.iter().enumerate().take(periods) {
                slot[idx] += value;
            }
        }
    }
    let parent_of: BTreeMap<&str, &str> = ir
        .entities
        .iter()
        .filter_map(|e| e.parent.as_deref().map(|p| (e.symbol.as_str(), p)))
        .collect();
    let mut rollup: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for entity in &ir.entities {
        rollup
            .entry(entity.symbol.clone())
            .or_insert_with(|| vec![0.0; periods]);
    }
    for (symbol, values) in &own {
        let mut visited: BTreeSet<&str> = BTreeSet::new();
        let mut cursor: Option<&str> = Some(symbol.as_str());
        while let Some(current) = cursor {
            if !visited.insert(current) {
                break;
            }
            let slot = rollup
                .entry(current.to_string())
                .or_insert_with(|| vec![0.0; periods]);
            for (idx, value) in values.iter().enumerate().take(periods) {
                slot[idx] += value;
            }
            cursor = parent_of.get(current).copied();
        }
    }
    rollup
}

#[allow(clippy::too_many_arguments)] // same shape as evaluate_stream: one call site, stage inputs
pub(crate) fn run_waterfalls(
    ir: &Ir,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    state_values: &BTreeMap<String, Vec<f64>>,
    entity_state: &[BTreeMap<String, BTreeMap<String, ExprValue>>],
    curves: &BTreeMap<String, cfdl_expr::CurveDef>,
    stream_series: &BTreeMap<String, Vec<f64>>,
    available_by_entity: &BTreeMap<String, Vec<f64>>,
    config: &RunConfig,
    warnings: &mut Vec<String>,
    // One row per step that ran, carrying the pot before and after it took.
    // A payee that got less than it was owed is then visible as a short pot
    // rather than inferable from the amount alone (`docs/28` §8).
    journal: &mut Vec<JournalEntry>,
) -> BTreeMap<String, Vec<f64>> {
    let periods = timeline.len();
    let mut out: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    // COMPOSITION: a waterfall may draw on what an EARLIER one paid.
    //
    // A fund's carry is a fund waterfall's output and a firm's carry pool is
    // its input, so the second reads the first through `series_sum`. Waterfalls
    // run in declaration order and each one's steps join this map as it
    // finishes, which makes the dependency an order rather than a graph — the
    // same rule steps inside a waterfall already follow.
    let mut visible: BTreeMap<String, Vec<f64>> = stream_series.clone();
    for waterfall in &ir.waterfalls {
        let shared = Arc::new(visible.clone());
        // Which periods this waterfall runs in. No schedule means every
        // period, the cadence a distribution date usually has.
        let mut hits = vec![Vec::new(); periods];
        let runs_in: Vec<bool> = match &waterfall.schedule {
            Some(schedule) => {
                if apply_schedule_indices(schedule, timeline, &mut hits).is_err() {
                    warnings.push(format!(
                        "Waterfall '{}' has an unusable schedule; skipped.",
                        waterfall.name
                    ));
                    continue;
                }
                hits.iter().map(|h| !h.is_empty()).collect()
            }
            None => vec![true; periods],
        };

        for step in &waterfall.steps {
            out.insert(
                format!("{}.{}", waterfall.name, step.name),
                vec![0.0; periods],
            );
        }

        for (t, runs) in runs_in.iter().enumerate().take(periods) {
            if !runs {
                continue;
            }
            let date = &timeline[t];
            let mut env = build_base_env(ir, config, t, date, base_inputs);
            env.curves = curves.clone();
            env.series = Arc::clone(&shared);
            // The pot, by name. `available` is the netted stream cash of the
            // entity this waterfall hangs on, children rolled up — supplied
            // the way `remaining` is, so no model declares a field for it.
            env.available = Some(ExprValue::Decimal(
                available_by_entity
                    .get(&waterfall.entity)
                    .and_then(|v| v.get(t))
                    .copied()
                    .unwrap_or(0.0),
            ));
            bind_states(&mut env, state_values, t);
            if let Some(state) = entity_state.get(t) {
                // Every entity by path — `entity.asset.class_a.balance` —
                // because a waterfall reads the balances of the things it pays,
                // not only of the one it hangs on.
                bind_all_entity_state(&mut env, state);
                // And the waterfall's own entity as `entity.state.*`, the
                // shorthand a stream on that entity already has.
                apply_entity_state(&mut env, state, &waterfall.entity);
            }

            let mut remaining = match cfdl_expr::compile_expr(&waterfall.source.src) {
                Ok(compiled) => eval_amount_expr(
                    &compiled,
                    &env,
                    &waterfall.name,
                    &ir.model.currency,
                    warnings,
                )
                .max(0.0),
                Err(err) => {
                    warnings.push(format!(
                        "Waterfall '{}' pot failed to compile [{}]: {}; treated as zero.",
                        waterfall.name, err.code, err.message
                    ));
                    0.0
                }
            };

            let mut paid: BTreeMap<String, ExprValue> = BTreeMap::new();
            let mut owed: BTreeMap<String, ExprValue> = BTreeMap::new();
            for step in &waterfall.steps {
                let pot_before = remaining;
                let mut step_env = env.clone();
                step_env.remaining = Some(ExprValue::Decimal(remaining));
                step_env.paid = paid.clone();
                step_env.owed = owed.clone();

                let wants = match cfdl_expr::compile_expr(&step.amount.src) {
                    Ok(compiled) => eval_amount_expr(
                        &compiled,
                        &step_env,
                        &format!("{}.{}", waterfall.name, step.name),
                        &ir.model.currency,
                        warnings,
                    ),
                    Err(err) => {
                        warnings.push(format!(
                            "Waterfall '{}' step '{}' failed to compile [{}]: {}; pays nothing.",
                            waterfall.name, step.name, err.code, err.message
                        ));
                        0.0
                    }
                }
                .max(0.0);
                let takes = wants.min(remaining);
                remaining = round_amount(remaining - takes);

                owed.insert(step.name.clone(), ExprValue::Decimal(round_amount(wants)));
                paid.insert(step.name.clone(), ExprValue::Decimal(round_amount(takes)));
                if let Some(series) = out.get_mut(&format!("{}.{}", waterfall.name, step.name)) {
                    series[t] = round_amount(takes);
                }
                let short = takes + f64::EPSILON < wants;
                let mut entry = JournalEntry::new(
                    t,
                    &timeline[t].to_string(),
                    format!("waterfall:{}", waterfall.name),
                    "pay",
                    format!("{} -> {}", step.name, step.payee),
                    if short { "overridden" } else { "applied" },
                );
                entry.amount = Some(round_amount(takes));
                entry.pot_before = Some(round_amount(pot_before));
                entry.pot_after = Some(remaining);
                if short {
                    entry.note = Some(format!(
                        "the pot was short: the step was owed {} and took {}",
                        round_amount(wants),
                        round_amount(takes)
                    ));
                }
                journal.push(entry);
            }
        }
        // This waterfall's steps become visible to the next one.
        for step in &waterfall.steps {
            let key = format!("{}.{}", waterfall.name, step.name);
            if let Some(values) = out.get(&key) {
                visible.insert(key, values.clone());
            }
        }
    }
    out
}
