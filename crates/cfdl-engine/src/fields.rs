// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

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
pub(crate) fn compute_states(
    ir: &Ir,
    config: &RunConfig,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    warnings: &mut Vec<String>,
) -> BTreeMap<String, Vec<f64>> {
    let mut values: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    if ir.entities.iter().all(|e| e.rules.is_empty()) {
        return values;
    }

    // Compiled once per field, not once per field per period. This loop is the
    // only place a rule's source is evaluated and it runs `fields x periods`
    // times — `x trials` under Monte Carlo.
    struct Prepared {
        /// `<entity>.<field>` — the path the field is read by.
        name: String,
        init: CompiledExpr,
        next: CompiledExpr,
        /// Model periods on which the recurrence STEPS. `None` means every
        /// period, which is what a field with no schedule of its own means.
        ticks: Option<Vec<bool>>,
        /// The first tick. `init` is the value AT the first tick, not at model
        /// period 0 — the base case belongs to the recurrence's own clock.
        ///
        /// A quarterly schedule on a monthly book accrues at periods 2, 5, 8,
        /// where the payment index is 0, 1, 2. Stepping on the first accrual
        /// would put F(1) where the first payment reads F(0) — an off-by-one
        /// against every published schedule. With no `schedule` this is 0, so
        /// an unscheduled field steps every period.
        first_tick: usize,
    }

    let zero = cfdl_expr::compile_expr("0").expect("constant expression compiles");
    let mut prepared: Vec<Prepared> = Vec::new();

    // A FIELD'S RULE IS THE SAME RECURRENCE, evaluated in the same pass.
    //
    // There is one place a recurrence is solved and one set of rules about what
    // it can see. The name is the entity path, which is also how it is read.
    for entity in &ir.entities {
        for (field, rule) in &entity.rules {
            let name = format!("{}.{}", entity.symbol, field);
            let mut compile = |src: &str, clause: &str| match cfdl_expr::compile_expr(src) {
                Ok(compiled) => compiled,
                Err(err) => {
                    warnings.push(format!(
                        "Field '{name}' {clause} expression compile failed [{}]: {}; using 0.",
                        err.code, err.message
                    ));
                    zero.clone()
                }
            };
            let init = compile(&rule.init.src, "init");
            let next = compile(&rule.next.src, "next");
            let (ticks, first_tick) = match &rule.schedule {
                None => (None, 0),
                Some(schedule) => {
                    let mut slots = vec![Vec::new(); timeline.len()];
                    match apply_schedule_indices(schedule, timeline, &mut slots) {
                        Ok(()) => {
                            let mut ticks = vec![false; timeline.len()];
                            for accruals in &slots {
                                for &idx in accruals {
                                    ticks[idx] = true;
                                }
                            }
                            let first = ticks.iter().position(|t| *t).unwrap_or(0);
                            (Some(ticks), first)
                        }
                        Err(err) => {
                            warnings.push(format!(
                                "Field '{name}' schedule could not be resolved: {err}; stepping every period."
                            ));
                            (None, 0)
                        }
                    }
                }
            };
            values.insert(name.clone(), vec![0.0; timeline.len()]);
            prepared.push(Prepared {
                name,
                init,
                next,
                ticks,
                first_tick,
            });
        }
    }

    for (t, date) in timeline.iter().enumerate() {
        // Snapshot the previous column before writing this one, so every state
        // in this period sees the same completed history regardless of order.
        // Both spellings, for the reason `build_expr_env` gives: a field answers
        // to `asset.x.bal` and `entity.asset.x.bal` alike, and `prev` must too.
        let previous: BTreeMap<String, ExprValue> = if t == 0 {
            BTreeMap::new()
        } else {
            values
                .iter()
                .flat_map(|(name, v)| {
                    [
                        (format!("entity.{name}"), ExprValue::Decimal(v[t - 1])),
                        (name.clone(), ExprValue::Decimal(v[t - 1])),
                    ]
                })
                .collect()
        };

        for entry in &prepared {
            let name = &entry.name;
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
            // A RULE MAY READ A LITERAL FIELD. It is a constant, so there is no
            // ordering question and nothing to sequence — `amortization = 10.0`
            // means the same thing in every period.
            //
            // Rule-bearing fields stay out: their period-close value does not
            // exist yet inside another rule, which is what `E1127` rejects at
            // compile time. Binding only literals here is what makes that
            // diagnostic honest — the validator permits exactly what the engine
            // can resolve.
            bind_literal_fields(&mut env, ir);
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
