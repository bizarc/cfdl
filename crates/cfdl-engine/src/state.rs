// The state stage: fields and events, one interleaved walk per period.
//
// ONE VALUE PER PATH. A field's recurrence computes the period's candidate;
// an event's write OVERWRITES it; the settled value is what readers see, what
// results publish, and what `prev` reads next period — so the law of motion
// resumes from the intervention. A partial liquidation reduces the balance and
// the next period amortizes from the reduced balance, not the contractual one.
//
// Rule-bearing fields therefore live in the value store and nowhere else: an
// event's write to one goes into the store, not into the entity-state record,
// so there is no second copy to go stale. The record keeps what it always
// held — declared attributes, lifecycle status, and writes to plain fields.
use super::*;

/// Output of the discrete event/option pre-pass over the master timeline.
pub(crate) struct EventSim {
    /// Per period: entity symbol -> field -> value (state as of that period).
    pub(crate) entity_state: Vec<BTreeMap<String, BTreeMap<String, ExprValue>>>,
    /// Per stream with an event override: per-period active flag.
    pub(crate) stream_active: BTreeMap<String, Vec<bool>>,
    /// Option payoff cash flows: option name -> per-period amounts.
    pub(crate) option_cash: BTreeMap<String, Vec<f64>>,
    /// Every state change an event made, in the order it happened.
    ///
    /// Entity state was UNOBSERVABLE in results: nothing distinguished "the
    /// event fired and its target was misspelled" from "the event never fired",
    /// and a case could not assert that a transition happened at all. The
    /// audit trail is the point — if and when something occurred.
    pub(crate) transitions: Vec<TransitionRecord>,
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
///
/// Fields and events run as ONE interleaved walk per period. Within a period:
/// every field's candidate is computed from the settled previous column; every
/// guard then reads the SAME frozen pre-state plus this period's candidates
/// (the synchronous rule — declaration order is not semantics); writes settle
/// the column. `prev` at t+1 reads what settled.
pub(crate) fn simulate_state(
    ir: &Ir,
    config: &RunConfig,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    warnings: &mut Vec<String>,
) -> (BTreeMap<String, Vec<f64>>, EventSim) {
    // No early exit on "no rules": a model with events and no fields still
    // needs the walk, which the fields-only version of this pass could skip.
    let mut values: BTreeMap<String, Vec<f64>> = BTreeMap::new();

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

    let periods = timeline.len();
    let mut entity_state: Vec<BTreeMap<String, BTreeMap<String, ExprValue>>> =
        Vec::with_capacity(periods);
    let mut current_state: BTreeMap<String, BTreeMap<String, ExprValue>> = BTreeMap::new();
    // AN ENTITY WITH A LIFECYCLE IS ALWAYS IN EXACTLY ONE STATE, from period 0.
    // Before the ontology there was no declared state space, so a status was
    // null until an event wrote one and `entity.status == "x"` was false for
    // reasons that had nothing to do with the deal.
    for entity in &ir.entities {
        // Declared attributes are the entity's properties from period 0. An
        // event may later write over one, which is why they seed the same map
        // rather than sitting beside it: a model asks an entity what it is,
        // and does not care whether the answer was declared or assigned.
        for (name, raw) in &entity.fields {
            let value = match raw.parse::<f64>() {
                Ok(number) => ExprValue::Decimal(number),
                Err(_) => ExprValue::String(raw.clone()),
            };
            current_state
                .entry(entity.symbol.clone())
                .or_default()
                .insert(name.clone(), value);
        }
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
        // --- fields: this period's candidates, from the settled prior column ---

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

        // --- events and options: read the candidates, settle the column ---

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
        bind_states(&mut env, &values, t);
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
                                let rule_key = format!("{}.{}", entity.symbol, field);
                                if let Some(series) = values.get_mut(&rule_key) {
                                    // ONE VALUE PER PATH: the write settles the
                                    // field store, and the recurrence resumes
                                    // from it next period. It does NOT enter
                                    // the entity-state record — that would be
                                    // a second copy, free to go stale.
                                    let before =
                                        Some(describe_value(&ExprValue::Decimal(series[t])));
                                    let after = describe_value(&v);
                                    match &v {
                                        ExprValue::Decimal(d) => series[t] = *d,
                                        ExprValue::Int(i) => series[t] = *i as f64,
                                        other => {
                                            warnings.push(format!(
                                                "Event '{}' set {} to non-numeric {:?}; store unchanged.",
                                                event.name, rule_key, other
                                            ));
                                        }
                                    }
                                    transitions.push(TransitionRecord {
                                        period: t,
                                        date: date.to_string(),
                                        entity: entity.symbol.clone(),
                                        field: field.clone(),
                                        from: before,
                                        to: after,
                                        event: event.name.clone(),
                                    });
                                    continue;
                                }
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

    (
        values,
        EventSim {
            entity_state,
            stream_active,
            option_cash,
            transitions,
        },
    )
}
