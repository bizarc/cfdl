// The occurrence stage: what HAPPENS in a period (`docs/34`).
//
// An event and an option are one idea — something that happens on a date or a
// condition and changes what follows — with two shapes. An EVENT fires on each
// occurrence: a scheduled test that passes, or its condition's rising edge with
// no schedule; what it does is its actions. An OPTION is an election: it fires
// as many times as it allows (once by default), on each occurrence of its
// election inside its exercisable window — a scheduled test it passes, or its
// rising edge — or when an event forces it there; what it does is pay its
// payoff and run its actions. Both read the
// same frozen pre-state and settled cash the machine read, and both write
// through the state walk's stores — the field store, the entity state, the
// stream mask, the transition record, the journal — so declaration order is
// never semantics and a write is visible at t+1, never at t.
//
// This module holds what carries across periods for both, prepares it once,
// and steps it once per period after the machine has moved.
use super::*;
use crate::state::{StateWalk, TransitionRecord};

/// What events and options carry across periods.
#[derive(Default)]
pub(crate) struct Occurrences {
    /// Last period's value of each event's condition — the rising edge's
    /// memory, and the only memory an event has.
    event_prev_condition: Vec<bool>,
    /// Per event, which periods its schedule supplies as occurrences. `None`
    /// when it has no schedule.
    event_scheduled: Vec<Option<Vec<bool>>>,
    /// How many times each option has been exercised, against its allowance
    /// (`exercises`, default one).
    option_exercises: Vec<u32>,
    /// Last period's value of each option's election while the option was
    /// held — the rising edge's memory for an unscheduled election.
    option_prev_condition: Vec<bool>,
    /// Per option, which periods its schedule supplies as occasions to
    /// exercise. `None` when it has no schedule.
    option_scheduled: Vec<Option<Vec<bool>>>,
    /// Options an event exercised this period; decided against `exercisable
    /// in` when the options step, then cleared.
    forced_exercise: Vec<String>,
    compiled_events: Vec<Option<cfdl_expr::CompiledExpr>>,
    compiled_options: Vec<Option<(cfdl_expr::CompiledExpr, cfdl_expr::CompiledExpr)>>,
    /// Option payoff cash flows: option name -> per-period amounts. Written
    /// on exercise; every declared option is seeded with zeros at `finish`.
    option_cash: BTreeMap<String, Vec<f64>>,
    periods: usize,
}

/// Compile every event's condition and every option's election and payoff,
/// and resolve each scheduled event's occurrences on the timeline.
pub(crate) fn prepare_occurrences(
    ir: &Ir,
    timeline: &[Date],
    warnings: &mut Vec<String>,
) -> Occurrences {
    let periods = timeline.len();
    let option_cash: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    // WAS `event_fired`, the latch. It is now last period's CONDITION, which
    // is what a rising edge compares against (`docs/34` D1). The latch was a
    // hidden state — "has fired" — living outside the machine, unjournaled and
    // undeclarable; this carries no memory of its own beyond one period.
    let event_prev_condition = vec![false; ir.events.len()];
    // Which periods a scheduled event is TESTED at. `None` for an event with
    // no schedule, whose occurrences come from its condition's own dynamics.
    let event_scheduled: Vec<Option<Vec<bool>>> = ir
        .events
        .iter()
        .map(|event| {
            let schedule = event.schedule.as_ref()?;
            let mut slots: Vec<Vec<usize>> = vec![Vec::new(); timeline.len()];
            match crate::timeline::apply_schedule_indices(schedule, timeline, &mut slots) {
                Ok(()) => Some(slots.iter().map(|s| !s.is_empty()).collect()),
                Err(err) => {
                    warnings.push(format!(
                        "Event '{}' schedule failed: {err}; event disabled.",
                        event.name
                    ));
                    // Every period false: a schedule that could not be read
                    // supplies no occurrences, rather than silently becoming
                    // an unscheduled event that fires on its condition.
                    Some(vec![false; timeline.len()])
                }
            }
        })
        .collect();
    let option_exercises = vec![0_u32; ir.options.len()];
    let option_prev_condition = vec![false; ir.options.len()];
    let option_scheduled: Vec<Option<Vec<bool>>> = ir
        .options
        .iter()
        .map(|option| {
            let schedule = option.schedule.as_ref()?;
            let mut slots: Vec<Vec<usize>> = vec![Vec::new(); timeline.len()];
            match crate::timeline::apply_schedule_indices(schedule, timeline, &mut slots) {
                Ok(()) => Some(slots.iter().map(|s| !s.is_empty()).collect()),
                Err(err) => {
                    warnings.push(format!(
                        "Option '{}' schedule failed: {err}; option disabled.",
                        option.name
                    ));
                    Some(vec![false; timeline.len()])
                }
            }
        })
        .collect();
    let forced_exercise: Vec<String> = Vec::new();

    let compiled_events: Vec<Option<cfdl_expr::CompiledExpr>> = ir
        .events
        .iter()
        .map(|event| {
            // An event with no `when` is purely scheduled: there is nothing to
            // compile, and every scheduled occurrence fires.
            let when = event.when.as_ref()?;
            match cfdl_expr::compile_expr(&when.src) {
                Ok(compiled) => Some(compiled),
                Err(err) => {
                    warnings.push(format!(
                        "Event '{}' trigger compile failed [{}]: {}; event disabled.",
                        event.name, err.code, err.message
                    ));
                    None
                }
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

    Occurrences {
        event_prev_condition,
        event_scheduled,
        option_exercises,
        option_prev_condition,
        option_scheduled,
        forced_exercise,
        compiled_events,
        compiled_options,
        option_cash,
        periods,
    }
}

impl Occurrences {
    /// What happens this period: every event in declaration order, then every
    /// option, against `env` — the frozen pre-state and settled cash the
    /// machine read — writing through `walk`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step(
        &mut self,
        ir: &Ir,
        t: usize,
        date: &Date,
        env: &ExprEnv,
        pre_state: &BTreeMap<String, BTreeMap<String, ExprValue>>,
        walk: &mut StateWalk,
        warnings: &mut Vec<String>,
    ) {
        for (event_idx, event) in ir.events.iter().enumerate() {
            // AN EVENT IS SOMETHING THAT HAPPENS, and nothing restricts it to
            // happening once (`docs/34` D1). A schedule SUPPLIES occurrences
            // and `when` FILTERS them; with no schedule, an occurrence is the
            // condition's rising edge — true having been false, re-arming when
            // it falls.
            //
            // A condition that fails to compile disables the event, which is
            // the shipped behavior: `None` here means no occurrence, never an
            // unconditional one.
            let condition = match &self.compiled_events[event_idx] {
                Some(when) => eval_bool_expr(when, env, "Event", &event.name, "when", warnings),
                // No `when` at all: every scheduled occurrence fires. An event
                // that failed to compile is a different case and is caught by
                // the `when.is_some()` test below.
                None => event.when.is_none(),
            };
            let disabled = event.when.is_some() && self.compiled_events[event_idx].is_none();
            let fires = match &self.event_scheduled[event_idx] {
                // Scheduled: the calendar says WHEN it is tested, the
                // condition says whether it fires. Four consecutive failing
                // quarterly tests are four breach events, because the model
                // declared quarterly testing.
                Some(mask) => mask.get(t).copied().unwrap_or(false) && condition,
                // Unscheduled: the condition's own rising edge.
                None => condition && !self.event_prev_condition[event_idx],
            };
            // Recorded whatever happened, so the edge is measured against the
            // condition and not against whether the event fired.
            self.event_prev_condition[event_idx] = condition;
            if disabled || !fires {
                continue;
            }
            self.run_actions(
                &format!("Event '{}'", event.name),
                &format!("event:{}", event.name),
                &event.name,
                &event.actions,
                t,
                date,
                env,
                walk,
                warnings,
            );
        }

        for (option_idx, option) in ir.options.iter().enumerate() {
            // AN OPTION IS EXERCISED AS MANY TIMES AS IT ALLOWS, once by
            // default: a lease with two renewals is exercised twice. Each
            // exercise is an OCCURRENCE in the event's sense (`docs/34` D1):
            // a scheduled test the election passes, or, unscheduled, the
            // election's rising edge — true having been false — so a right
            // that stays in the money is not re-exercised every period.
            let allowed = option.exercises.unwrap_or(1);
            if self.option_exercises[option_idx] >= allowed {
                continue;
            }
            let Some((when, payoff)) = &self.compiled_options[option_idx] else {
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
            //
            // Outside the window the election is not OBSERVED either: its edge
            // memory stays false, so a right whose condition already holds when
            // the window opens is exercised as the window opens.
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
                    if self.forced_exercise.iter().any(|name| name == &option.name) {
                        warnings.push(format!(
                            "Option '{}' was forced outside its exercisable phase '{phase_name}'; not exercised.",
                            option.name
                        ));
                        // An option outside its window is not one anyone holds,
                        // so an event cannot exercise it. The request was
                        // journaled as made; this is what became of it.
                        walk.journal.push(
                            JournalEntry::new(
                                t,
                                &date.to_string(),
                                format!("option:{}", option.name),
                                "exercise_option",
                                option.name.clone(),
                                "declined",
                            )
                            .with_note(format!(
                                "forced outside its exercisable phase '{phase_name}'; an option outside its window is not one anyone holds"
                            )),
                        );
                    }
                    continue;
                }
            }
            // An option HAS an owner, so `entity.<field>` in its guard means
            // the owner's field — the same thing it means in a stream. Events
            // have no owner and use the qualified path instead.
            let mut option_env = env.clone();
            if let Some(owner) = &option.owner {
                apply_entity_state(&mut option_env, pre_state, &owner.symbol);
            }
            let env = &option_env;
            let forced = self.forced_exercise.iter().any(|name| name == &option.name);
            let condition =
                eval_bool_expr(when, env, "Option", &option.name, "exercise when", warnings);
            let occurrence = match &self.option_scheduled[option_idx] {
                // Scheduled: the calendar says WHEN the right may be exercised
                // (a Bermudan election), the election says whether it is.
                Some(mask) => mask.get(t).copied().unwrap_or(false) && condition,
                // Unscheduled: the election's own rising edge.
                None => condition && !self.option_prev_condition[option_idx],
            };
            self.option_prev_condition[option_idx] = condition;
            if !(forced || occurrence) {
                continue;
            }
            self.option_exercises[option_idx] += 1;
            walk.journal.push(
                JournalEntry::new(
                    t,
                    &date.to_string(),
                    format!("option:{}", option.name),
                    "exercise_option",
                    option.name.clone(),
                    "applied",
                )
                .with_note(match (forced, allowed) {
                    (true, 1) => "forced by an event, inside its exercisable window".to_string(),
                    (true, n) => format!(
                        "forced by an event, inside its exercisable window; exercise {} of {n}",
                        self.option_exercises[option_idx]
                    ),
                    (false, 1) => "its own `exercise when` held".to_string(),
                    (false, n) => format!(
                        "its own `exercise when` held; exercise {} of {n}",
                        self.option_exercises[option_idx]
                    ),
                }),
            );
            // The payoff ACCUMULATES: a right exercised twice pays twice, and
            // two exercises in one period (forced and elected) would pay once
            // each.
            let payoff_values = self
                .option_cash
                .entry(option.name.clone())
                .or_insert_with(|| vec![0.0_f64; self.periods]);
            match cfdl_expr::eval(payoff, env) {
                Ok(ExprValue::Decimal(v)) => payoff_values[t] += v,
                Ok(ExprValue::Int(v)) => payoff_values[t] += v as f64,
                Ok(other) => warnings.push(format!(
                    "Option '{}' payoff returned non-numeric {other:?}; using 0.",
                    option.name
                )),
                Err(err) => warnings.push(format!(
                    "Option '{}' payoff failed [{}]: {}; using 0.",
                    option.name, err.code, err.message
                )),
            }
            // WHAT THE EXERCISE DOES beyond paying — a prepayment ends the
            // loan, a renewal extends the lease — through the vocabulary an
            // event uses and the same stores, visible at t+1. An `exercise
            // option` here reaches an option declared after this one in the
            // same period, as an event's does.
            if !option.actions.is_empty() {
                self.run_actions(
                    &format!("Option '{}'", option.name),
                    &format!("option:{}", option.name),
                    &option.name,
                    &option.actions,
                    t,
                    date,
                    env,
                    walk,
                    warnings,
                );
            }
        }
        self.forced_exercise.clear();
    }

    /// Run a host's actions — an event's on firing, an option's on exercise —
    /// against `env`, writing through the walk's stores. `label` names the
    /// host for warnings ("Event 'x'"), `source` is the journal source
    /// ("event:x", "option:x"), `cause` is what a transition records.
    #[allow(clippy::too_many_arguments)]
    fn run_actions(
        &mut self,
        label: &str,
        source: &str,
        cause: &str,
        actions: &[IrAction],
        t: usize,
        date: &Date,
        env: &ExprEnv,
        walk: &mut StateWalk,
        warnings: &mut Vec<String>,
    ) {
        for action in actions {
            match action.kind.as_str() {
                "SetEntityField" => {
                    let (Some(entity), Some(field), Some(value)) =
                        (&action.entity, &action.field, &action.value)
                    else {
                        warnings.push(format!(
                            "{label} SetEntityField is missing fields; skipped."
                        ));
                        continue;
                    };
                    match cfdl_expr::compile_expr(&value.src)
                        .and_then(|compiled| cfdl_expr::eval(&compiled, env))
                    {
                        Ok(v) => {
                            let rule_key = format!("{}.{}", entity.symbol, field);
                            if let Some(series) = walk.values.get_mut(&rule_key) {
                                // ONE VALUE PER PATH: the write settles the
                                // field store, and the recurrence resumes
                                // from it next period. It does NOT enter
                                // the entity-state record — that would be
                                // a second copy, free to go stale.
                                let before = Some(describe_value(&ExprValue::Decimal(series[t])));
                                let after = describe_value(&v);
                                match &v {
                                    ExprValue::Decimal(d) => series[t] = *d,
                                    ExprValue::Int(i) => series[t] = *i as f64,
                                    other => {
                                        warnings.push(format!(
                                            "{label} set {} to non-numeric {:?}; store unchanged.",
                                            rule_key, other
                                        ));
                                    }
                                }
                                walk.transitions.push(TransitionRecord {
                                    period: t,
                                    date: date.to_string(),
                                    entity: entity.symbol.clone(),
                                    field: field.clone(),
                                    from: before.clone(),
                                    to: after.clone(),
                                    event: cause.to_string(),
                                });
                                walk.journal.push(
                                    JournalEntry::new(
                                        t,
                                        &date.to_string(),
                                        source.to_string(),
                                        "set",
                                        rule_key.clone(),
                                        "applied",
                                    )
                                    .with_change(before, after),
                                );
                                continue;
                            }
                            // AN EVENT'S WRITE IS VALIDATED AGAINST THE
                            // MACHINE (`docs/28` §6.1 rule 3). The
                            // from-state is the status as it stands NOW
                            // in the period — a machine move this period
                            // included — and an absent edge is a refusal
                            // with the edge named, not a silent
                            // overwrite. An edge-less machine stays
                            // unconstrained, `permits()`'s shipped rule;
                            // any declared edge suffices whether guarded
                            // or not, because a guard-less edge is
                            // exactly a permission for a write like this.
                            if field == "status" {
                                if let Some(machine) = walk.machines.get(&entity.symbol) {
                                    if !machine.edges.is_empty() {
                                        let from = walk
                                            .current_state
                                            .get(&entity.symbol)
                                            .and_then(|f| f.get("status"))
                                            .and_then(|v| match v {
                                                ExprValue::String(s) => Some(s.clone()),
                                                _ => None,
                                            })
                                            .unwrap_or_default();
                                        let to = match &v {
                                            ExprValue::String(s) => s.clone(),
                                            other => describe_value(other),
                                        };
                                        let permitted = machine
                                            .edges
                                            .iter()
                                            .any(|e| e.from == from && e.to == to);
                                        if !permitted {
                                            warnings.push(format!(
                                                "{label} would move '{}' {from} -> {to}, an edge lifecycle '{}' does not declare; the write is refused.",
                                                entity.symbol, machine.lifecycle_id
                                            ));
                                            walk.journal.push(
                                                JournalEntry::new(
                                                    t,
                                                    &date.to_string(),
                                                    source.to_string(),
                                                    "set",
                                                    format!("{}.status", entity.symbol),
                                                    "declined",
                                                )
                                                .with_note(format!(
                                                    "{from} -> {to} is not a declared edge of lifecycle '{}'",
                                                    machine.lifecycle_id
                                                )),
                                            );
                                            continue;
                                        }
                                    }
                                }
                            }
                            let slot = walk.current_state.entry(entity.symbol.clone()).or_default();
                            let before = slot.get(field).map(describe_value);
                            let after = describe_value(&v);
                            slot.insert(field.clone(), v);
                            // Recorded even when the value does not change:
                            // the log answers "did this event fire", and a
                            // set that wrote the same value still fired.
                            walk.transitions.push(TransitionRecord {
                                period: t,
                                date: date.to_string(),
                                entity: entity.symbol.clone(),
                                field: field.clone(),
                                from: before.clone(),
                                to: after.clone(),
                                event: cause.to_string(),
                            });
                            let mut entry = JournalEntry::new(
                                t,
                                &date.to_string(),
                                source.to_string(),
                                "set",
                                format!("{}.{}", entity.symbol, field),
                                "applied",
                            )
                            .with_change(before.clone(), after.clone());
                            // A `status` write that MOVES the entity is an
                            // arrival like any other, and runs the target
                            // state's entry actions and the taken edge's
                            // (`docs/34` D2, D6). Without this the same
                            // arrival would mean two different things
                            // depending on what caused it.
                            if field == "status" && before.as_deref() != Some(after.as_str()) {
                                let plan = walk.machines.get(&entity.symbol).map(|m| {
                                    (
                                        m.lifecycle_id.clone(),
                                        m.edges.iter().position(|e| {
                                            Some(e.from.as_str()) == before.as_deref()
                                                && e.to == after
                                        }),
                                    )
                                });
                                if let Some((lifecycle_id, edge_idx)) = plan {
                                    entry.children = walk.run_arrival_actions(
                                        &entity.symbol,
                                        &lifecycle_id,
                                        &after,
                                        edge_idx,
                                        t,
                                        date,
                                        env,
                                        warnings,
                                    );
                                }
                            }
                            walk.journal.push(entry);
                        }
                        Err(err) => {
                            warnings.push(format!(
                                "{label} set {}.{} failed [{}]: {}; skipped.",
                                entity.symbol, field, err.code, err.message
                            ));
                            walk.journal.push(
                                JournalEntry::new(
                                    t,
                                    &date.to_string(),
                                    source.to_string(),
                                    "set",
                                    format!("{}.{}", entity.symbol, field),
                                    "failed",
                                )
                                .with_note(format!("[{}] {}", err.code, err.message)),
                            );
                        }
                    }
                }
                "ActivateStream" => {
                    if let Some(stream) = &action.stream {
                        walk.current_active.insert(stream.clone(), true);
                        // `applied` HERE MEANS THE MASK MOVED, not that the
                        // stream will pay: the stream's own `active when`
                        // is a second gate, and `streams.rs` rewrites this
                        // row to `overridden` for the periods it refuses.
                        walk.journal.push(JournalEntry::new(
                            t,
                            &date.to_string(),
                            source.to_string(),
                            "activate_stream",
                            stream.clone(),
                            "applied",
                        ));
                    }
                }
                "DeactivateStream" => {
                    if let Some(stream) = &action.stream {
                        walk.current_active.insert(stream.clone(), false);
                        walk.journal.push(JournalEntry::new(
                            t,
                            &date.to_string(),
                            source.to_string(),
                            "deactivate_stream",
                            stream.clone(),
                            "applied",
                        ));
                    }
                }
                "ExerciseOption" => {
                    if let Some(option) = &action.option {
                        self.forced_exercise.push(option.clone());
                        // Whether it is HELD is decided below, against
                        // `exercisable in`; this row records the request.
                        walk.journal.push(JournalEntry::new(
                            t,
                            &date.to_string(),
                            source.to_string(),
                            "exercise_option",
                            option.clone(),
                            "applied",
                        ));
                    }
                }
                other => {
                    warnings.push(format!("{label}: unknown action kind '{other}'; ignored."));
                    walk.journal.push(
                        JournalEntry::new(
                            t,
                            &date.to_string(),
                            source.to_string(),
                            other,
                            String::new(),
                            "ignored",
                        )
                        .with_note("unknown action kind"),
                    );
                }
            }
        }
    }

    /// Publish the option cash. AN UNEXERCISED OPTION PUBLISHES ZERO, NOT
    /// NOTHING: `option_cash` is only written on exercise, so an option that
    /// stayed out of the money would otherwise produce no series at all — a
    /// consumer could not tell "did not exercise" from "does not exist", and a
    /// case could not assert a NON-exercise, which is half of what an option
    /// model has to prove. Seeded here rather than before the run so the map
    /// keeps meaning "exercised" while the timeline runs.
    pub(crate) fn finish(mut self, ir: &Ir) -> BTreeMap<String, Vec<f64>> {
        for option in &ir.options {
            self.option_cash
                .entry(option.name.clone())
                .or_insert_with(|| vec![0.0; self.periods]);
        }
        self.option_cash
    }
}
