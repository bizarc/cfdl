// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
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

/// Evaluate events and options discretely at each time step (spec §12/§13).
/// Events latch: each fires at most once, at the first period its condition
/// is true, in declaration order. Options exercise at most once, when their
/// phase gate (if any) and `exercise when` condition hold, or when forced by
/// an `exercise option` action.
pub(crate) fn simulate_events(
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
