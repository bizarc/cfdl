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
    /// Every causal act, with what became of it. `docs/28` §8.
    ///
    /// Distinct from `transitions`, which records field CHANGES: an action
    /// that was declined, ignored or overridden changes nothing and so has
    /// nowhere else to appear.
    pub(crate) journal: Vec<JournalEntry>,
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

/// What a stream reads of the state a period has settled: the field values,
/// the entity state per period, and the stream-active mask.
pub(crate) type SettledState<'a> = (
    &'a BTreeMap<String, Vec<f64>>,
    &'a [BTreeMap<String, BTreeMap<String, ExprValue>>],
    &'a BTreeMap<String, Vec<bool>>,
);

/// A field's rule, compiled once and stepped every period.
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

/// The state stage, in a form that can be stepped one period at a time.
///
/// SPLIT OUT SO THE WALK CAN DRIVE IT. `docs/28` §3 settles a period in three
/// stages — state, then streams, then any scheduled waterfall — and that is
/// only expressible if the state stage can be advanced a period at a time
/// rather than run to completion first. Extracting it changes nothing on its
/// own: `simulate_state` below is a loop over `step`, so the whole-timeline
/// order and the walk share one implementation and cannot compute different
/// numbers.
///
/// It holds what accumulates ACROSS periods. Everything a single period reads
/// but does not own — the IR, the config, the timeline, the resolved inputs —
/// is passed to `step`.
/// One entity's machine, guards compiled once (`docs/28` §6.1).
struct PreparedMachine {
    lifecycle_id: String,
    /// Edges in declaration order — the order that resolves two guards
    /// holding in one period, the same rule waterfall steps follow.
    edges: Vec<PreparedEdge>,
}

struct PreparedEdge {
    from: String,
    to: String,
    /// `None` for a guard-less edge: a permission an event's write may take,
    /// never fired by the machine on its own.
    guard: Option<cfdl_expr::CompiledExpr>,
    guard_src: String,
}

/// The lifecycle state an entity OPENS in: its own `state` clause, or its
/// machine's `initial`. `None` for the many entities with neither.
pub(crate) fn opening_status(ir: &Ir, entity: &str) -> Option<String> {
    let decl = ir.entities.iter().find(|e| e.symbol == entity)?;
    decl.initial_state.clone().or_else(|| {
        decl.lifecycle
            .as_ref()
            .and_then(|id| ir.lifecycles.iter().find(|lc| &lc.id == id))
            .map(|lc| lc.initial.clone())
    })
}

/// Every period at which `entity` ENTERED `state`, opening included, from a
/// transition record. Both evaluation orders build the same answer from the
/// same records, which is what keeps a state-anchored schedule's windows
/// identical under either (`docs/28` §6.2).
pub(crate) fn entries_into(
    ir: &Ir,
    transitions: &[TransitionRecord],
    entity: &str,
    state: &str,
) -> Vec<usize> {
    let mut entries = Vec::new();
    if opening_status(ir, entity).as_deref() == Some(state) {
        entries.push(0);
    }
    for record in transitions {
        if record.entity == entity && record.to == state {
            entries.push(record.period);
        }
    }
    entries.sort_unstable();
    entries.dedup();
    entries
}

pub(crate) struct StateWalk {
    /// Each machine-bound entity's compiled machine, by entity symbol.
    machines: BTreeMap<String, PreparedMachine>,
    // Read by the walk between periods; see `entity_state_so_far`.
    values: BTreeMap<String, Vec<f64>>,
    prepared: Vec<Prepared>,
    entity_state: Vec<BTreeMap<String, BTreeMap<String, ExprValue>>>,
    current_state: BTreeMap<String, BTreeMap<String, ExprValue>>,
    current_active: BTreeMap<String, bool>,
    stream_active: BTreeMap<String, Vec<bool>>,
    option_cash: BTreeMap<String, Vec<f64>>,
    transitions: Vec<TransitionRecord>,
    journal: Vec<JournalEntry>,
    event_fired: Vec<bool>,
    option_exercised: Vec<bool>,
    forced_exercise: Vec<String>,
    compiled_events: Vec<Option<cfdl_expr::CompiledExpr>>,
    compiled_options: Vec<Option<(cfdl_expr::CompiledExpr, cfdl_expr::CompiledExpr)>>,
    periods: usize,
    /// Cash that has already settled, for logic to read.
    ///
    /// `docs/28` §4: an event's guard and a field's rule may read a stream at
    /// or before the PREVIOUS period. The walk fills this as it advances; under
    /// the column order it stays empty, because there the state stage runs
    /// before any stream has a value and there is nothing to offer.
    settled_cash: Arc<BTreeMap<String, Vec<f64>>>,
    /// Account balances settled so far, read as `prev.<account>`.
    ///
    /// `docs/28` §5.1's third use: an OC/IC-style trigger tests a reserve
    /// balance the same way a delinquency edge tests realised rent — strictly
    /// backward, so at period `t` the binding is the balance at `t - 1`, every
    /// allocation through `t - 1` included, because those stage passes have
    /// run. At period 0 there is no binding at all: before the model began is
    /// not zero, it is unavailable.
    settled_accounts: Arc<BTreeMap<String, Vec<f64>>>,
}

impl StateWalk {
    /// Hand the walk the cash settled so far, before stepping the next period.
    pub(crate) fn observe_cash(&mut self, cash: Arc<BTreeMap<String, Vec<f64>>>) {
        self.settled_cash = cash;
    }

    /// Hand the walk the account balances settled so far, likewise.
    pub(crate) fn observe_accounts(&mut self, balances: Arc<BTreeMap<String, Vec<f64>>>) {
        self.settled_accounts = balances;
    }

    /// The transition record as settled so far, for state-anchored schedules.
    pub(crate) fn transitions_so_far(&self) -> &[TransitionRecord] {
        &self.transitions
    }

    /// The state settled so far, as the two pieces a stream reads.
    ///
    /// Handed out BETWEEN periods, never during one: the state stage settles a
    /// period completely before the streams stage evaluates it, so the two
    /// borrows are sequential and the seam needs no interior mutability to be
    /// two-way.
    pub(crate) fn settled(&self) -> SettledState<'_> {
        (&self.values, &self.entity_state, &self.stream_active)
    }

    /// Settle one period: this period's field candidates, then the events and
    /// options that may overwrite them, then the column.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step(
        &mut self,
        ir: &Ir,
        config: &RunConfig,
        t: usize,
        date: &Date,
        timeline: &[Date],
        base_inputs: &BTreeMap<String, f64>,
        warnings: &mut Vec<String>,
    ) {
        let _ = timeline;
        // --- fields: this period's candidates, from the settled prior column ---

        // Snapshot the previous column before writing this one, so every state
        // in this period sees the same completed history regardless of order.
        // Both spellings, for the reason `build_expr_env` gives: a field answers
        // to `asset.x.bal` and `entity.asset.x.bal` alike, and `prev` must too.
        let mut previous: BTreeMap<String, ExprValue> = if t == 0 {
            BTreeMap::new()
        } else {
            self.values
                .iter()
                .flat_map(|(name, v)| {
                    [
                        (format!("entity.{name}"), ExprValue::Decimal(v[t - 1])),
                        (name.clone(), ExprValue::Decimal(v[t - 1])),
                    ]
                })
                .collect()
        };
        // `prev.<account>` is the balance at the previous period, settled
        // history the way a field's prior value is. A field keeps its name on
        // collision — an account cannot shadow declared state.
        if t > 0 {
            for (name, column) in self.settled_accounts.iter() {
                previous
                    .entry(name.clone())
                    .or_insert(ExprValue::Decimal(column[t - 1]));
            }
        }

        for entry in &self.prepared {
            let name = &entry.name;
            // Between ticks, and outside the schedule's window, a state HOLDS.
            // It does not fall to zero — that is what separates a schedule from
            // `active when`, and why `active when` is deliberately absent here.
            // See docs/14_state_and_recurrence.md.
            let steps = t > entry.first_tick && entry.ticks.as_ref().is_none_or(|ticks| ticks[t]);
            if t > 0 && !steps {
                if let Some(slot) = self.values.get_mut(name) {
                    slot[t] = slot[t - 1];
                }
                continue;
            }

            let mut env = build_expr_env(ir, None, config, t, date, base_inputs);
            // SETTLED HISTORY ONLY. The watermark is the previous period, so a
            // guard reaching this period or later is refused by the engine
            // rather than reading a cell the walk has not computed — the same
            // discipline `E1134` applies at compile time.
            env.series = Arc::clone(&self.settled_cash);
            env.series_available_to = Some(t.saturating_sub(1));
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
                    if let Some(slot) = self.values.get_mut(name) {
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
        // in `self.current_state` and become visible at t+1, never at t.
        //
        // That is the Esterel/SCADE discipline the engine already had by
        // accident: `env` was built once before the loop, so nothing could
        // race. It held vacuously, because guards could read no state at all.
        // Now that they can, the property has to be deliberate — otherwise the
        // value of a guard would depend on which event happened to be declared
        // first, and declaration order would become semantics.
        let pre_state = self.current_state.clone();
        let mut env = build_base_env(ir, config, t, date, base_inputs);
        bind_states(&mut env, &self.values, t);
        bind_all_entity_state(&mut env, &pre_state);
        // AN EVENT'S GUARD AND AN OPTION'S ELECTION READ SETTLED CASH, at or
        // before the previous period (`docs/28` §4). The watermark is `t - 1`,
        // so a read reaching this period is refused by the engine rather than
        // finding a cell the walk has not computed.
        env.series = Arc::clone(&self.settled_cash);
        env.series_available_to = Some(t.saturating_sub(1));
        // A guard reads `prev.<account>` the way a rule does: the balance at
        // `t - 1`, and no binding at all in the first period — before the
        // model began is not zero, it is unavailable.
        if t > 0 {
            for (name, column) in self.settled_accounts.iter() {
                env.prev_states
                    .entry(name.clone())
                    .or_insert(ExprValue::Decimal(column[t - 1]));
            }
        }
        // THE MACHINE MOVES FIRST (`docs/28` §6.1). Each machined entity's
        // outgoing edges are evaluated in declaration order against the SAME
        // frozen pre-state and settled cash the events read; the first guard
        // that holds takes its edge, and at most one transition per entity
        // per period is taken. There is no latch — the from-state gate is the
        // memory, and a self-edge is a real transition. The write is visible
        // to this period's streams, the timing every event write has.
        let machine_moves: Vec<(String, String, String, String, String)> = self
            .machines
            .iter()
            .filter_map(|(symbol, machine)| {
                let from = match pre_state.get(symbol).and_then(|f| f.get("status")) {
                    Some(ExprValue::String(state)) => state.clone(),
                    _ => return None,
                };
                for edge in &machine.edges {
                    if edge.from != from {
                        continue;
                    }
                    let Some(guard) = &edge.guard else {
                        // A guard-less edge is a permission, never fired here.
                        continue;
                    };
                    if !eval_bool_expr(
                        guard,
                        &env,
                        "Lifecycle",
                        &format!("{} edge {} -> {}", machine.lifecycle_id, edge.from, edge.to),
                        "when",
                        warnings,
                    ) {
                        continue;
                    }
                    // The values the guard read, for the journal (§6.1 rule
                    // 4): each series it names at the settled watermark, and
                    // the period itself.
                    let mut reads: Vec<String> = vec![format!("time.t={t}")];
                    for series in cfdl_expr::series_references(&edge.guard_src) {
                        let cell = (t > 0)
                            .then(|| {
                                self.settled_cash
                                    .get(&series)
                                    .and_then(|col| col.get(t - 1))
                            })
                            .flatten();
                        match cell {
                            Some(v) => reads.push(format!("{series}[t-1]={v}")),
                            None => reads.push(format!("{series}[t-1]=unavailable")),
                        }
                    }
                    return Some((
                        symbol.clone(),
                        machine.lifecycle_id.clone(),
                        edge.from.clone(),
                        edge.to.clone(),
                        format!("when {} — read {}", edge.guard_src, reads.join(", ")),
                    ));
                }
                None
            })
            .collect();
        for (symbol, lifecycle_id, from, to, note) in machine_moves {
            self.current_state
                .entry(symbol.clone())
                .or_default()
                .insert("status".to_string(), ExprValue::String(to.clone()));
            self.transitions.push(TransitionRecord {
                period: t,
                date: date.to_string(),
                entity: symbol.clone(),
                field: "status".to_string(),
                from: Some(from.clone()),
                to: to.clone(),
                event: format!("lifecycle:{lifecycle_id}"),
            });
            self.journal.push(
                JournalEntry::new(
                    t,
                    &date.to_string(),
                    format!("lifecycle:{lifecycle_id}"),
                    "transition",
                    format!("{symbol}: {from} -> {to}"),
                    "applied",
                )
                .with_note(note),
            );
        }

        for (event_idx, event) in ir.events.iter().enumerate() {
            if self.event_fired[event_idx] {
                continue;
            }
            let Some(when) = &self.compiled_events[event_idx] else {
                continue;
            };
            if !eval_bool_expr(when, &env, "Event", &event.name, "when", warnings) {
                continue;
            }
            self.event_fired[event_idx] = true;
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
                                if let Some(series) = self.values.get_mut(&rule_key) {
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
                                    self.transitions.push(TransitionRecord {
                                        period: t,
                                        date: date.to_string(),
                                        entity: entity.symbol.clone(),
                                        field: field.clone(),
                                        from: before.clone(),
                                        to: after.clone(),
                                        event: event.name.clone(),
                                    });
                                    self.journal.push(
                                        JournalEntry::new(
                                            t,
                                            &date.to_string(),
                                            format!("event:{}", event.name),
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
                                    if let Some(machine) = self.machines.get(&entity.symbol) {
                                        if !machine.edges.is_empty() {
                                            let from = self
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
                                                    "Event '{}' would move '{}' {from} -> {to}, an edge lifecycle '{}' does not declare; the write is refused.",
                                                    event.name, entity.symbol, machine.lifecycle_id
                                                ));
                                                self.journal.push(
                                                    JournalEntry::new(
                                                        t,
                                                        &date.to_string(),
                                                        format!("event:{}", event.name),
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
                                let slot =
                                    self.current_state.entry(entity.symbol.clone()).or_default();
                                let before = slot.get(field).map(describe_value);
                                let after = describe_value(&v);
                                slot.insert(field.clone(), v);
                                // Recorded even when the value does not change:
                                // the log answers "did this event fire", and a
                                // set that wrote the same value still fired.
                                self.transitions.push(TransitionRecord {
                                    period: t,
                                    date: date.to_string(),
                                    entity: entity.symbol.clone(),
                                    field: field.clone(),
                                    from: before.clone(),
                                    to: after.clone(),
                                    event: event.name.clone(),
                                });
                                self.journal.push(
                                    JournalEntry::new(
                                        t,
                                        &date.to_string(),
                                        format!("event:{}", event.name),
                                        "set",
                                        format!("{}.{}", entity.symbol, field),
                                        "applied",
                                    )
                                    .with_change(before, after),
                                );
                            }
                            Err(err) => {
                                warnings.push(format!(
                                    "Event '{}' set {}.{} failed [{}]: {}; skipped.",
                                    event.name, entity.symbol, field, err.code, err.message
                                ));
                                self.journal.push(
                                    JournalEntry::new(
                                        t,
                                        &date.to_string(),
                                        format!("event:{}", event.name),
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
                            self.current_active.insert(stream.clone(), true);
                            // `applied` HERE MEANS THE MASK MOVED, not that the
                            // stream will pay: the stream's own `active when`
                            // is a second gate, and `streams.rs` rewrites this
                            // row to `overridden` for the periods it refuses.
                            self.journal.push(JournalEntry::new(
                                t,
                                &date.to_string(),
                                format!("event:{}", event.name),
                                "activate_stream",
                                stream.clone(),
                                "applied",
                            ));
                        }
                    }
                    "DeactivateStream" => {
                        if let Some(stream) = &action.stream {
                            self.current_active.insert(stream.clone(), false);
                            self.journal.push(JournalEntry::new(
                                t,
                                &date.to_string(),
                                format!("event:{}", event.name),
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
                            self.journal.push(JournalEntry::new(
                                t,
                                &date.to_string(),
                                format!("event:{}", event.name),
                                "exercise_option",
                                option.clone(),
                                "applied",
                            ));
                        }
                    }
                    other => {
                        warnings.push(format!(
                            "Event '{}': unknown action kind '{other}'; ignored.",
                            event.name
                        ));
                        self.journal.push(
                            JournalEntry::new(
                                t,
                                &date.to_string(),
                                format!("event:{}", event.name),
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

        for (option_idx, option) in ir.options.iter().enumerate() {
            if self.option_exercised[option_idx] {
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
                        self.journal.push(
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
                apply_entity_state(&mut option_env, &pre_state, &owner.symbol);
            }
            let env = &option_env;
            let forced = self.forced_exercise.iter().any(|name| name == &option.name);
            let triggered = forced
                || eval_bool_expr(when, env, "Option", &option.name, "exercise when", warnings);
            if !triggered {
                continue;
            }
            self.option_exercised[option_idx] = true;
            self.journal.push(
                JournalEntry::new(
                    t,
                    &date.to_string(),
                    format!("option:{}", option.name),
                    "exercise_option",
                    option.name.clone(),
                    "applied",
                )
                .with_note(if forced {
                    "forced by an event, inside its exercisable window"
                } else {
                    "its own `exercise when` held"
                }),
            );
            let mut payoff_values = vec![0.0_f64; self.periods];
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
            self.option_cash.insert(option.name.clone(), payoff_values);
        }
        self.forced_exercise.clear();

        self.entity_state.push(self.current_state.clone());
        for (stream, active) in &self.current_active {
            self.stream_active
                .entry(stream.clone())
                .or_insert_with(|| vec![true; self.periods])[t] = *active;
        }
    }
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
/// Compile and lay out the state stage, ready to be stepped.
pub(crate) fn prepare_state_walk(
    ir: &Ir,
    timeline: &[Date],
    warnings: &mut Vec<String>,
) -> StateWalk {
    // No early exit on "no rules": a model with events and no fields still
    // needs the walk, which the fields-only version of this pass could skip.
    let mut values: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    // Compiled once per field, not once per field per period. This loop is the
    // only place a rule's source is evaluated and it runs `fields x periods`
    // times — `x trials` under Monte Carlo.

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
    let entity_state: Vec<BTreeMap<String, BTreeMap<String, ExprValue>>> =
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
        // The entity's own `state` wins; the machine's `initial` is the
        // fallback — every machine opens somewhere, so a bound entity is
        // never status-less.
        let opening = entity.initial_state.clone().or_else(|| {
            entity
                .lifecycle
                .as_ref()
                .and_then(|id| ir.lifecycles.iter().find(|lc| &lc.id == id))
                .map(|lc| lc.initial.clone())
        });
        if let Some(initial) = opening {
            current_state
                .entry(entity.symbol.clone())
                .or_default()
                .insert("status".to_string(), ExprValue::String(initial));
        }
    }

    // THE MACHINES, GUARDS COMPILED ONCE. Only entities that bind one get an
    // entry, so the per-period pass is over exactly the machined entities.
    let mut machines: BTreeMap<String, PreparedMachine> = BTreeMap::new();
    for entity in &ir.entities {
        let Some(machine) = entity
            .lifecycle
            .as_ref()
            .and_then(|id| ir.lifecycles.iter().find(|lc| &lc.id == id))
        else {
            continue;
        };
        let edges = machine
            .edges
            .iter()
            .map(|edge| PreparedEdge {
                from: edge.from.clone(),
                to: edge.to.clone(),
                guard: edge.guard.as_ref().and_then(|g| {
                    match cfdl_expr::compile_expr(&g.src) {
                        Ok(compiled) => Some(compiled),
                        Err(err) => {
                            warnings.push(format!(
                                "Lifecycle '{}' edge '{} -> {}' guard compile failed [{}]: {}; edge disabled.",
                                machine.id, edge.from, edge.to, err.code, err.message
                            ));
                            None
                        }
                    }
                }),
                guard_src: edge.guard.as_ref().map(|g| g.src.clone()).unwrap_or_default(),
            })
            .collect();
        machines.insert(
            entity.symbol.clone(),
            PreparedMachine {
                lifecycle_id: machine.id.clone(),
                edges,
            },
        );
    }
    let current_active: BTreeMap<String, bool> = BTreeMap::new();
    let stream_active: BTreeMap<String, Vec<bool>> = BTreeMap::new();
    let option_cash: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let transitions: Vec<TransitionRecord> = Vec::new();
    let journal: Vec<JournalEntry> = Vec::new();
    let event_fired = vec![false; ir.events.len()];
    let option_exercised = vec![false; ir.options.len()];
    let forced_exercise: Vec<String> = Vec::new();

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

    StateWalk {
        machines,
        values,
        prepared,
        entity_state,
        current_state,
        current_active,
        stream_active,
        option_cash,
        transitions,
        journal,
        event_fired,
        option_exercised,
        forced_exercise,
        compiled_events,
        compiled_options,
        periods,
        settled_cash: Arc::default(),
        settled_accounts: Arc::default(),
    }
}

/// Settle every period, then publish. The whole-timeline order.
pub(crate) fn simulate_state(
    ir: &Ir,
    config: &RunConfig,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    warnings: &mut Vec<String>,
) -> (BTreeMap<String, Vec<f64>>, EventSim) {
    let mut walk = prepare_state_walk(ir, timeline, warnings);
    for (t, date) in timeline.iter().enumerate() {
        walk.step(ir, config, t, date, timeline, base_inputs, warnings);
    }
    walk.finish(ir)
}

impl StateWalk {
    /// Publish what the walk settled.
    pub(crate) fn finish(self, ir: &Ir) -> (BTreeMap<String, Vec<f64>>, EventSim) {
        let StateWalk {
            values,
            entity_state,
            stream_active,
            mut option_cash,
            transitions,
            journal,
            periods,
            ..
        } = self;

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
                journal,
            },
        )
    }
}
