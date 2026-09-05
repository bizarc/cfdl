// The walk: one period at a time, the stages in order (`docs/28` §3).
//
// For each period: the cash already settled is handed over; the state stage
// settles (fields, then the machine, then what happens — `state.rs`,
// `occurrence.rs`); this period's streams evaluate in dependency waves against
// it (`streams.rs`); the account plane applies what they moved and any machine
// write (`accounts.rs`); the waterfall stage runs where a schedule names the
// period (`distributions.rs`); the folds settle. Then `t + 1`.
//
// THE ONE EXCEPTION is the column order: a model whose window reaches past the
// period being computed cannot be walked, so its state runs to completion
// first and its streams evaluate a column at a time in the fold. The two
// orders compute the same numbers wherever both can run —
// `walk_matches_the_column_order` asserts it on the blessed corpus — and a
// stream that moves or reads a balance is refused on the column order rather
// than zeroed (`docs/42`). `evaluate_model` chooses; `walk_periods` walks.
use super::*;

/// What one evaluation of a model settles before the fold: the state stage,
/// and — under the walk — every stream's column, the refusals, the account
/// balances, the waterfall columns and the stage's journal. Under the column
/// order the stream and waterfall fields are `None` and the fold evaluates
/// them a column at a time.
pub(crate) struct Evaluated {
    pub(crate) state_values: BTreeMap<String, Vec<f64>>,
    pub(crate) event_sim: EventSim,
    pub(crate) walked_columns: Option<StreamColumns>,
    pub(crate) walked_refusals: BTreeMap<String, Vec<usize>>,
    pub(crate) account_balances: BTreeMap<String, Vec<f64>>,
    pub(crate) walked_waterfalls: Option<BTreeMap<String, Vec<f64>>>,
    pub(crate) stage_journal: Vec<JournalEntry>,
}

/// Evaluate the model: the walk where the model allows it, the column order
/// where a forward-reaching read forces it.
pub(crate) fn evaluate_model(
    ir: &Ir,
    config: &RunConfig,
    prep: &ModelPrep<'_>,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    warnings: &mut Vec<String>,
) -> Result<Evaluated, EngineError> {
    // THE WALK, WHERE THE MODEL ALLOWS IT. A period walk cannot serve a window
    // that reaches past the period being computed, so a forward-reading model
    // keeps the column order; everything else settles a period at a time, which
    // is what lets a guard read cash that has already arrived (`docs/28` §4).
    // The two orders compute the same numbers — `walk_matches_the_column_order`
    // asserts it on the blessed corpus — so this changes what a model may SAY,
    // not what any existing model reports.
    let walked_columns: Option<BTreeMap<String, Vec<f64>>>;
    let walked_refusals: BTreeMap<String, Vec<usize>>;
    let account_balances: BTreeMap<String, Vec<f64>>;
    let walked_waterfalls: Option<BTreeMap<String, Vec<f64>>>;
    let mut stage_journal: Vec<JournalEntry> = Vec::new();
    let (state_values, event_sim) = if prep.walk_ineligible.is_none() {
        let (sv, es, columns, refusals, balances, waterfalls, journal) =
            walk_periods(ir, config, prep, base_inputs, warnings);
        walked_columns = Some(columns);
        walked_refusals = refusals;
        account_balances = balances;
        walked_waterfalls = Some(waterfalls);
        stage_journal = journal;
        (sv, es)
    } else {
        walked_columns = None;
        walked_refusals = BTreeMap::new();
        account_balances = BTreeMap::new();
        walked_waterfalls = None;
        if let Some(moving) = ir.streams.iter().find(|s| s.moves.is_some()) {
            // A stream that moves or reads a balance needs the balance
            // carried period by period; the column order has no period to
            // carry it through. Refused: a zero here is a wrong number.
            return Err(EngineError::AccountsNeedTheWalk(format!(
                "Stream '{}' moves account '{}', but a forward-reaching read keeps this model on the column order, where no balance is carried: {}",
                moving.name,
                moving.moves.as_deref().unwrap_or_default(),
                prep.walk_ineligible.as_deref().unwrap_or_default()
            )));
        }
        if !ir.accounts.is_empty() {
            // A forward-reading model keeps the column order, and the column
            // order has no periods to carry a balance through. Said rather
            // than silently published as zeros.
            warnings.push(
                "This model declares accounts, but a forward-reaching read keeps it on the                  column order, where account balances are not computed."
                    .to_string(),
            );
        }
        simulate_state(ir, config, timeline, base_inputs, warnings)
    };
    Ok(Evaluated {
        state_values,
        event_sim,
        walked_columns,
        walked_refusals,
        account_balances,
        walked_waterfalls,
        stage_journal,
    })
}

/// What one pass over the grid settles: field values, the event record, and
/// each stream's column.
pub(crate) type WalkOutput = (
    BTreeMap<String, Vec<f64>>,
    EventSim,
    StreamColumns,
    BTreeMap<String, Vec<usize>>,
    // Each account's balance, period by period.
    BTreeMap<String, Vec<f64>>,
    // Each waterfall step's column — the stage runs inside the walk.
    BTreeMap<String, Vec<f64>>,
    // What the stage journaled: inflows and allocations, period-major.
    Vec<JournalEntry>,
);

/// THE PERIOD WALK. One period at a time: state settles, then that period's
/// streams evaluate against it.
///
/// This is `docs/28` §3, and it is the point of phase 2. The column order
/// finishes each stream over the whole timeline before starting the next, so a
/// period's state can never see cash — the state stage has already run to
/// completion by then. Here state at period `t` is settled with every earlier
/// period's cash already in the store, which is what makes `docs/28` §4's
/// backward reads expressible at all.
///
/// Nothing exercises that yet: `E1134` still refuses a series read in logic,
/// so no model can ask. Phase 3 relaxes it to refuse only FORWARD reads, and
/// this loop is what makes the refusal narrowable rather than absolute.
///
/// Waterfalls run as stage 3 OF EACH PERIOD — after that period's streams
/// settle, never interleaved with them (`docs/28` §5). The schedule stays
/// sovereign: on a period no waterfall is scheduled for, the stage accumulates
/// account inflows and does nothing else, so running the stage each period is
/// not distributing each period. What it buys is the balance law applied AT
/// each period: a distribution at `t` draws on what has accumulated, and
/// logic at `t + 1` reads a settled figure.
#[allow(clippy::too_many_arguments)]
pub(crate) fn walk_periods(
    ir: &Ir,
    config: &RunConfig,
    prep: &ModelPrep<'_>,
    base_inputs: &BTreeMap<String, f64>,
    warnings: &mut Vec<String>,
) -> WalkOutput {
    let timeline = &prep.timeline;
    let max_wave = prep.waves.iter().copied().max().unwrap_or(0);
    let mut walk = prepare_state_walk(ir, timeline, warnings);
    let mut stage = prepare_waterfall_stage(ir, timeline, warnings);
    // THE PRICED PASS (`docs/28` §7). A priced amount is a valuation over
    // cells beyond the flow; eligibility has already established that nothing
    // causal reads it back. The walk therefore skips priced streams, prices
    // them from the settled store once every causal cell exists, and runs the
    // waterfall stage after that — so a distribution at the sale period
    // allocates proceeds that exist.
    let priced: Vec<bool> = priced_closure(ir, &stream_deps(ir));
    let any_priced = priced.iter().any(|p| *p);
    let curves = ir_curve_defs(ir);
    // STATE-ANCHORED PLANS ARE WALK-LOCAL. `prep` is shared across
    // scenarios and Monte Carlo trials, and each run's entries are its own —
    // a trial where the machine breaches later anchors later. The shared
    // plan stays untouched; this walk re-anchors its own copies as entries
    // settle.
    let mut anchored: BTreeMap<usize, crate::streams::StreamPlan<'_>> = BTreeMap::new();
    for (idx, stream) in ir.streams.iter().enumerate() {
        if stream.schedule.kind == "StateEnter" {
            match plan_stream(ir, stream, timeline, warnings) {
                Ok(plan) => {
                    anchored.insert(idx, plan);
                }
                Err(err) => warnings.push(format!(
                    "Stream '{}' state-anchored schedule failed to plan: {err}; no periods scheduled.",
                    stream.name
                )),
            }
        }
    }
    let mut full: BTreeMap<String, Vec<f64>> = ir
        .streams
        .iter()
        .map(|s| (s.name.clone(), vec![0.0_f64; timeline.len()]))
        .collect();
    let mut refusals: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    // An account's balance carries across periods: the walk fills it as it
    // advances, so a distribution at `t` draws on what has accumulated and
    // logic at `t + 1` reads a settled figure.
    let mut account_balances: BTreeMap<String, Vec<f64>> = ir
        .accounts
        .iter()
        .map(|a| (a.name.clone(), vec![0.0_f64; timeline.len()]))
        .collect();
    // Each account's opening in the first period, folds included, and the
    // members every fold sums (`accounts.rs`).
    let (account_inits, members) =
        initial_balances(ir, config, prep, timeline, base_inputs, warnings);
    walk.observe_account_inits(Arc::clone(&account_inits));
    walk.observe_entity_accounts(
        ir.accounts
            .iter()
            .filter(|a| a.owner_entity.is_some() && !a.fold)
            .map(|a| a.name.clone())
            .collect(),
    );
    // What each period's machines moved on an account, kept for the deferred
    // stage of a priced model.
    let mut machine_moves_by_period: Vec<Vec<(String, String, f64)>> = Vec::new();
    let account_side = account_sides(ir);

    for t in 0..timeline.len() {
        // 0. THE CASH ALREADY SETTLED, handed over before this period's state
        //    is computed. `docs/28` §4: logic reads at or before `t - 1`, and
        //    the store holds exactly that, because periods `0..t` are done and
        //    period `t` has not started.
        walk.observe_cash(Arc::new(full.clone()));
        walk.observe_accounts(Arc::new(account_balances.clone()));

        // 1. STATE SETTLES. Fields take this period's candidates and events
        //    overwrite them — now able to test cash that has already arrived.
        walk.step(ir, config, t, &timeline[t], timeline, base_inputs, warnings);

        // 1b. STATE-ANCHORED SCHEDULES RESOLVE (`docs/28` §6.2). The entry
        //     period is settled state by the time any stream reads it, so
        //     each anchored plan's windows are recomputed from the entries
        //     known through `t` — an entry AT `t` opens its window at `t`,
        //     and a re-entered state re-anchors. Sound per period because a
        //     window only extends forward: no evaluated period can change.
        for (idx, plan) in anchored.iter_mut() {
            let Some((entity, state)) = plan.anchor() else {
                continue;
            };
            let (entity, state) = (entity.to_string(), state.to_string());
            let entries = entries_into(ir, walk.transitions_so_far(), &entity, &state);
            if let Err(err) = plan.re_anchor(&entries, timeline) {
                warnings.push(format!(
                    "Stream '{}' state-anchored schedule failed: {err}; no periods scheduled.",
                    ir.streams[*idx].name
                ));
            }
        }

        // 2. THIS PERIOD'S STREAMS, against the state just settled. The borrow
        //    of the walk ends with the period, so the next `step` may take it
        //    mutably again — sequential, not simultaneous.
        //    Each account's OPENING this period — the prior close, or the
        //    `init` in the first period — is what a stream reads as
        //    `prev.<account>` (`docs/42` §3.3).
        let mut opening_accounts: BTreeMap<String, f64> = account_balances
            .iter()
            .map(|(name, column)| {
                let opening = if t == 0 {
                    account_inits.get(name).copied().unwrap_or(0.0)
                } else {
                    column[t - 1]
                };
                (name.clone(), opening)
            })
            .collect();
        // A MACHINE'S WRITE TO AN ACCOUNT moves its opening (docs/42 §3.5):
        // `set balance = 0` on repurchase is a write-off of the whole opening
        // balance, so every stream reading `prev.balance` this period reads
        // zero. The movement is journaled by the stage below.
        let mut machine_moves: Vec<(String, String, f64)> = Vec::new();
        for (name, value, actor) in walk.take_account_writes() {
            let before = opening_accounts.get(&name).copied().unwrap_or(0.0);
            opening_accounts.insert(name.clone(), value);
            machine_moves.push((name, actor, value - before));
        }
        for (fold, of) in &members {
            let sum: f64 = of.iter().filter_map(|m| opening_accounts.get(m)).sum();
            opening_accounts.insert(fold.clone(), sum);
        }
        machine_moves_by_period.push(machine_moves.clone());
        let (field_values, entity_state, stream_active) = walk.settled();
        for wave in 0..=max_wave {
            let wave_is_active = (0..ir.streams.len()).any(|idx| {
                let plan = anchored.get(&idx).unwrap_or(&prep.plans[idx]);
                prep.waves[idx] == wave && plan.settles_at(t)
            });
            if !wave_is_active {
                continue;
            }
            let snapshot = (wave > 0).then(|| Arc::new(full.clone()));
            for (idx, stream) in ir.streams.iter().enumerate() {
                if priced[idx] {
                    continue;
                }
                let plan = anchored.get(&idx).unwrap_or(&prep.plans[idx]);
                if prep.waves[idx] != wave || !plan.settles_at(t) {
                    continue;
                }
                let mut refused: Vec<usize> = Vec::new();
                let value = plan.step(
                    ir,
                    config,
                    t,
                    timeline,
                    base_inputs,
                    entity_state,
                    stream_active,
                    field_values,
                    snapshot.as_ref(),
                    warnings,
                    &mut refused,
                    Some(t),
                    Some(&opening_accounts),
                );
                if let Some(column) = full.get_mut(&stream.name) {
                    column[t] = value;
                }
                if !refused.is_empty() {
                    refusals
                        .entry(stream.name.clone())
                        .or_default()
                        .extend(refused);
                }
            }
        }

        // 2b. WHAT THIS PERIOD'S STREAMS MOVED (`docs/42` §3.2). A cash
        //     stream's signed amount raises a liability its owner owes and
        //     lowers a receivable its owner is due; an accrual raises and a
        //     write-off lowers, whichever side. Applied to the balance by the
        //     stage below, one journal line each.
        let mut moves = account_moves_at(ir, &full, &account_side, t);
        for (name, actor, delta) in &machine_moves {
            moves
                .entry(name.clone())
                .or_default()
                .push((actor.clone(), *delta));
        }

        // 3. THE WATERFALL STAGE, over cash this period has produced and never
        //    interleaved with it. Accounts take their inflow, then each
        //    waterfall the SCHEDULE says runs allocates — from `available`,
        //    or from an account's accumulated balance. DEFERRED when the
        //    model prices: the stage must see the priced cash, and nothing in
        //    it feeds back into logic or streams (eligibility said so), so
        //    running it after the priced pass changes no causal cell.
        if !any_priced {
            let snapshot = Arc::new(full.clone());
            let (field_values, entity_state, _) = walk.settled();
            stage.step(
                ir,
                config,
                t,
                &timeline[t],
                base_inputs,
                field_values,
                entity_state,
                &curves,
                &snapshot,
                &prep.account_inflows,
                &account_inits,
                &moves,
                &mut account_balances,
                warnings,
            );
            settle_folds(&members, &mut account_balances, t);
        }
    }

    // THE PRICED PASS, then the deferred stage. Priced streams evaluate in
    // wave order against the settled store with no watermark — the forward
    // window is exactly what a valuation is allowed to see — and each column
    // joins the store as it finishes, so a priced amount may read an
    // earlier-priced one.
    if any_priced {
        let (field_values, entity_state, stream_active) = walk.settled();
        for wave in 0..=max_wave {
            for (idx, stream) in ir.streams.iter().enumerate() {
                if !priced[idx] || prep.waves[idx] != wave {
                    continue;
                }
                let snapshot = Arc::new(full.clone());
                let plan = &prep.plans[idx];
                let mut refused: Vec<usize> = Vec::new();
                let mut column = vec![0.0_f64; timeline.len()];
                for (t, slot) in column.iter_mut().enumerate() {
                    if !plan.settles_at(t) {
                        continue;
                    }
                    *slot = plan.step(
                        ir,
                        config,
                        t,
                        timeline,
                        base_inputs,
                        entity_state,
                        stream_active,
                        field_values,
                        Some(&snapshot),
                        warnings,
                        &mut refused,
                        None,
                        None,
                    );
                }
                if let Some(slot) = full.get_mut(&stream.name) {
                    *slot = column;
                }
                if !refused.is_empty() {
                    refusals
                        .entry(stream.name.clone())
                        .or_default()
                        .extend(refused);
                }
            }
        }
        for (t, date) in timeline.iter().enumerate() {
            let snapshot = Arc::new(full.clone());
            let mut moves = account_moves_at(ir, &full, &account_side, t);
            for (name, actor, delta) in machine_moves_by_period.get(t).into_iter().flatten() {
                moves
                    .entry(name.clone())
                    .or_default()
                    .push((actor.clone(), *delta));
            }
            stage.step(
                ir,
                config,
                t,
                date,
                base_inputs,
                field_values,
                entity_state,
                &curves,
                &snapshot,
                &prep.account_inflows,
                &account_inits,
                &moves,
                &mut account_balances,
                warnings,
            );
            settle_folds(&members, &mut account_balances, t);
        }
    }

    let (state_values, event_sim) = walk.finish(ir);
    let (waterfall_series, stage_journal) = stage.finish();
    (
        state_values,
        event_sim,
        full,
        refusals,
        account_balances,
        waterfall_series,
        stage_journal,
    )
}
