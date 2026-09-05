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
        // Which periods this waterfall runs in. The schedule is required —
        // the compiler refuses its absence (E1348) — so a waterfall without
        // one can only reach here from hand-written IR. This branch used to
        // invent every-period, which the compiler never agreed with: it
        // lowered the omission to a first-period one-shot instead, and the
        // component whose comment explained the intent was the one that lost.
        // Now one component states the rule and the other says nothing.
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
            None => {
                warnings.push(format!(
                    "Waterfall '{}' has no schedule; skipped.",
                    waterfall.name
                ));
                continue;
            }
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

            let remaining = match cfdl_expr::compile_expr(&waterfall.source.src) {
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

            allocate_steps(
                ir,
                waterfall,
                t,
                &timeline[t],
                &env,
                remaining,
                &mut out,
                warnings,
                journal,
            );
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

/// One waterfall's priority of payments, at one period.
///
/// The step loop both evaluation orders share: steps take in declaration
/// order, each `min(max(0, owed), remaining)`, and every take is journaled
/// with the pot before and after it. Returns what each step took, in step
/// order, so a caller that allocates FROM or INTO an account can move the
/// balances — the takes are the allocation, seen from either end.
#[allow(clippy::too_many_arguments)] // stage inputs, as the callers have
fn allocate_steps(
    ir: &Ir,
    waterfall: &IrWaterfall,
    t: usize,
    date: &Date,
    env: &ExprEnv,
    mut remaining: f64,
    out: &mut BTreeMap<String, Vec<f64>>,
    warnings: &mut Vec<String>,
    journal: &mut Vec<JournalEntry>,
) -> Vec<f64> {
    let mut takes_per_step: Vec<f64> = Vec::with_capacity(waterfall.steps.len());
    let mut paid: BTreeMap<String, ExprValue> = BTreeMap::new();
    let mut owed: BTreeMap<String, ExprValue> = BTreeMap::new();
    {
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
            takes_per_step.push(round_amount(takes));

            owed.insert(step.name.clone(), ExprValue::Decimal(round_amount(wants)));
            paid.insert(step.name.clone(), ExprValue::Decimal(round_amount(takes)));
            if let Some(series) = out.get_mut(&format!("{}.{}", waterfall.name, step.name)) {
                series[t] = round_amount(takes);
            }
            let short = takes + f64::EPSILON < wants;
            let mut entry = JournalEntry::new(
                t,
                &date.to_string(),
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
    takes_per_step
}

/// Each entity's netted stream cash at ONE period, rolled up by `part of`.
///
/// The walk's form of `stream_cash_by_entity`: at period `t` the columns are
/// filled through `t`, and the stage needs only that period's figure.
pub(crate) fn stream_cash_by_entity_at(
    ir: &Ir,
    stream_series: &BTreeMap<String, Vec<f64>>,
    t: usize,
) -> BTreeMap<String, f64> {
    let mut own: BTreeMap<String, f64> = BTreeMap::new();
    for stream in &ir.streams {
        if !streams::is_cash(stream) {
            continue;
        }
        if let Some(value) = stream_series.get(&stream.name).and_then(|v| v.get(t)) {
            *own.entry(stream.owner.symbol.clone()).or_insert(0.0) += value;
        }
    }
    let parent_of: BTreeMap<&str, &str> = ir
        .entities
        .iter()
        .filter_map(|e| e.parent.as_deref().map(|p| (e.symbol.as_str(), p)))
        .collect();
    let mut rollup: BTreeMap<String, f64> = BTreeMap::new();
    for entity in &ir.entities {
        rollup.entry(entity.symbol.clone()).or_insert(0.0);
    }
    for (symbol, value) in &own {
        let mut visited: BTreeSet<&str> = BTreeSet::new();
        let mut cursor: Option<&str> = Some(symbol.as_str());
        while let Some(current) = cursor {
            if !visited.insert(current) {
                break;
            }
            *rollup.entry(current.to_string()).or_insert(0.0) += value;
            cursor = parent_of.get(current).copied();
        }
    }
    rollup
}

/// A party reference, with or without its family prefix.
///
/// An account's `owner` and a step's payee are both written by the modeller,
/// one as `party.lp1` and one as `lp1` or vice versa; the party is the same.
fn party_key(reference: &str) -> &str {
    reference.strip_prefix("party.").unwrap_or(reference)
}

/// The waterfall stage of the period walk (`docs/28` §5).
///
/// Prepared once, stepped once per period AFTER that period's streams settle
/// and never interleaved with them. The SCHEDULE STAYS SOVEREIGN: on a period
/// no waterfall is scheduled for, the step accumulates account inflows and
/// does nothing else — running the stage each period is not distributing each
/// period. A quarterly or at-exit waterfall leaves the cash building in its
/// account until its date arrives.
///
/// The stage owns the whole account lifecycle at each period, in balance-law
/// order: inflow first, then allocation in and out, all at `t` —
/// `balance(t) = balance(t-1) + inflow(t) + allocated_in(t) - allocated_out(t)`.
pub(crate) struct WaterfallStage {
    /// Which periods each waterfall runs in, schedule applied once.
    /// An unusable schedule leaves an empty vec: warned at prepare, skipped.
    runs_in: Vec<Vec<bool>>,
    /// Step columns, filled as the stage steps.
    out: BTreeMap<String, Vec<f64>>,
    /// Party (bare, no family prefix) -> the one account that party owns.
    accounts_by_owner: BTreeMap<String, String>,
    journal: Vec<JournalEntry>,
}

pub(crate) fn prepare_waterfall_stage(
    ir: &Ir,
    timeline: &[Date],
    warnings: &mut Vec<String>,
) -> WaterfallStage {
    let periods = timeline.len();
    let mut runs_in: Vec<Vec<bool>> = Vec::with_capacity(ir.waterfalls.len());
    let mut out: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for waterfall in &ir.waterfalls {
        let mut hits = vec![Vec::new(); periods];
        let runs: Vec<bool> = match &waterfall.schedule {
            Some(schedule) => {
                if apply_schedule_indices(schedule, timeline, &mut hits).is_err() {
                    warnings.push(format!(
                        "Waterfall '{}' has an unusable schedule; skipped.",
                        waterfall.name
                    ));
                    runs_in.push(Vec::new());
                    continue;
                }
                hits.iter().map(|h| !h.is_empty()).collect()
            }
            // Required by the compiler (E1348); absent only in hand-written
            // IR, where inventing a cadence is what this change removes.
            None => {
                warnings.push(format!(
                    "Waterfall '{}' has no schedule; skipped.",
                    waterfall.name
                ));
                runs_in.push(Vec::new());
                continue;
            }
        };
        runs_in.push(runs);
        for step in &waterfall.steps {
            out.insert(
                format!("{}.{}", waterfall.name, step.name),
                vec![0.0; periods],
            );
        }
    }
    // A party owns at most one account, so "their account" always resolves
    // (`docs/28` §5.1). A second declaration for the same party would make the
    // rule ambiguous, so it is refused here with both accounts named.
    let mut accounts_by_owner: BTreeMap<String, String> = BTreeMap::new();
    for account in &ir.accounts {
        if let Some(owner) = &account.owner {
            let key = party_key(owner).to_string();
            if let Some(existing) = accounts_by_owner.get(&key) {
                warnings.push(format!(
                    "Party '{key}' owns accounts '{existing}' and '{}'; a party owns at most one account, and '{existing}' keeps the allocations.",
                    account.name
                ));
                continue;
            }
            accounts_by_owner.insert(key, account.name.clone());
        }
    }
    WaterfallStage {
        runs_in,
        out,
        accounts_by_owner,
        journal: Vec::new(),
    }
}

impl WaterfallStage {
    /// Settle one period: accounts take their inflow, then each scheduled
    /// waterfall allocates, in declaration order.
    #[allow(clippy::too_many_arguments)] // stage inputs, as evaluate_stream has
    pub(crate) fn step(
        &mut self,
        ir: &Ir,
        config: &RunConfig,
        t: usize,
        date: &Date,
        base_inputs: &BTreeMap<String, f64>,
        state_values: &BTreeMap<String, Vec<f64>>,
        entity_state: &[BTreeMap<String, BTreeMap<String, ExprValue>>],
        curves: &BTreeMap<String, cfdl_expr::CurveDef>,
        streams: &Arc<BTreeMap<String, Vec<f64>>>,
        account_inflows: &[Option<cfdl_expr::CompiledExpr>],
        // Each account's `init`: the balance carried into the first period.
        account_inits: &BTreeMap<String, f64>,
        // What this period's streams moved, per account: (stream, delta),
        // already signed for the account's side (`docs/42` §3.2).
        moves: &BTreeMap<String, Vec<(String, f64)>>,
        balances: &mut BTreeMap<String, Vec<f64>>,
        warnings: &mut Vec<String>,
    ) {
        let date_str = date.to_string();
        // 1. INFLOW. Carried balance plus this period's declared inflow, which
        //    reads the period's settled streams. May be negative, with no
        //    floor: an account fed a deal's whole net cash IS the cumulative
        //    position, negative through the J-curve.
        for (idx, account) in ir.accounts.iter().enumerate() {
            // A fold has no inflow and no movement of its own; the walk sums
            // its members after this stage.
            if account.fold {
                continue;
            }
            let inflow = account_inflow_at(
                ir,
                config,
                account,
                account_inflows.get(idx).and_then(|c| c.as_ref()),
                t,
                date,
                base_inputs,
                Some(streams),
                warnings,
            );
            if let Some(column) = balances.get_mut(&account.name) {
                let carried = if t == 0 {
                    account_inits.get(&account.name).copied().unwrap_or(0.0)
                } else {
                    column[t - 1]
                };
                column[t] = carried + inflow;
                // THE STREAMS THAT MOVE IT, each a journal line naming its
                // cause: this scheduled principal, from this loan, lowered
                // this balance by this much.
                if let Some(moved) = moves.get(&account.name) {
                    for (stream, delta) in moved {
                        if *delta == 0.0 {
                            continue;
                        }
                        let before = column[t];
                        column[t] += delta;
                        let mut entry = JournalEntry::new(
                            t,
                            &date_str,
                            format!("stream:{stream}"),
                            "move",
                            account.name.clone(),
                            "applied",
                        );
                        entry.amount = Some(round_amount(*delta));
                        entry.pot_before = Some(round_amount(before));
                        entry.pot_after = Some(round_amount(column[t]));
                        self.journal.push(entry);
                    }
                }
                if inflow != 0.0 {
                    let mut entry = JournalEntry::new(
                        t,
                        &date_str,
                        format!("account:{}", account.name),
                        "inflow",
                        account.name.clone(),
                        "applied",
                    );
                    entry.amount = Some(round_amount(inflow));
                    entry.pot_before = Some(round_amount(carried));
                    entry.pot_after = Some(round_amount(column[t]));
                    self.journal.push(entry);
                }
            }
        }

        // 2. ALLOCATION, on schedule.
        let available_now = stream_cash_by_entity_at(ir, streams, t);
        for (w, waterfall) in ir.waterfalls.iter().enumerate() {
            if !self.runs_in[w].get(t).copied().unwrap_or(false) {
                continue;
            }
            // COMPOSITION, the walk's form: this waterfall sees the streams and
            // every EARLIER-declared waterfall's steps, filled through `t`. A
            // walk-eligible model reads only backward, so the columns match
            // what the column order shows at the same read.
            let mut visible: BTreeMap<String, Vec<f64>> = (**streams).clone();
            for prior in &ir.waterfalls[..w] {
                for step in &prior.steps {
                    let key = format!("{}.{}", prior.name, step.name);
                    if let Some(values) = self.out.get(&key) {
                        visible.insert(key, values.clone());
                    }
                }
            }
            let mut env = build_base_env(ir, config, t, date, base_inputs);
            env.curves = curves.clone();
            env.series = Arc::new(visible);
            env.available = Some(ExprValue::Decimal(
                available_now.get(&waterfall.entity).copied().unwrap_or(0.0),
            ));
            bind_states(&mut env, state_values, t);
            if let Some(state) = entity_state.get(t) {
                bind_all_entity_state(&mut env, state);
                apply_entity_state(&mut env, state, &waterfall.entity);
            }
            // `prev.<account>` in a step's amount: the balance at the previous
            // period, strictly backward — the reserve pattern's own read
            // (`fund to target` is `target - prev.<reserve>`). This period's
            // inflow and earlier allocations are NOT in it; they are this
            // period, and this period is not settled while it is being
            // allocated. In the first period it is the `init` (`docs/42`
            // §7): an account without one opens at zero.
            for account in &ir.accounts {
                let opening = if t > 0 {
                    balances.get(&account.name).map(|column| column[t - 1])
                } else {
                    // The first period's opening is the `init` (`docs/42`
                    // §7); an account without one opens at zero.
                    Some(account_inits.get(&account.name).copied().unwrap_or(0.0))
                };
                if let Some(opening) = opening {
                    env.prev_states
                        .entry(account.name.clone())
                        .or_insert(ExprValue::Decimal(opening));
                }
            }

            // THE POT. `from <account>` hands the waterfall the ACCUMULATED
            // balance — inflow of this period included, allocations of earlier
            // waterfalls this period included. Anything else is an expression,
            // `from available` foremost. Either way the pot is floored at
            // zero: cash that is not there cannot be allocated.
            let source_account = ir
                .accounts
                .iter()
                .find(|a| a.name == waterfall.source.src.trim())
                .map(|a| a.name.clone());
            let pot = match &source_account {
                Some(name) => balances
                    .get(name)
                    .map(|column| column[t])
                    .unwrap_or(0.0)
                    .max(0.0),
                None => match cfdl_expr::compile_expr(&waterfall.source.src) {
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
                },
            };

            let takes = allocate_steps(
                ir,
                waterfall,
                t,
                date,
                &env,
                pot,
                &mut self.out,
                warnings,
                &mut self.journal,
            );

            // 3. THE BALANCES MOVE — one allocation, seen from both ends.
            //    Credits land before the source is reduced, and both before the
            //    next waterfall runs, so a later waterfall this period reads
            //    the moved balance.
            let mut total_taken = 0.0;
            for (step, take) in waterfall.steps.iter().zip(&takes) {
                total_taken += take;
                if *take == 0.0 {
                    continue;
                }
                let destination = if step.payee_is_account {
                    Some(step.payee.clone())
                } else {
                    self.accounts_by_owner.get(party_key(&step.payee)).cloned()
                };
                if let Some(name) = destination {
                    if let Some(column) = balances.get_mut(&name) {
                        let before = column[t];
                        column[t] = round_amount(before + take);
                        let mut entry = JournalEntry::new(
                            t,
                            &date_str,
                            format!("waterfall:{}", waterfall.name),
                            "allocate_in",
                            format!("{} -> account:{name}", step.name),
                            "applied",
                        );
                        entry.amount = Some(*take);
                        entry.pot_before = Some(round_amount(before));
                        entry.pot_after = Some(column[t]);
                        self.journal.push(entry);
                    }
                }
            }
            if let Some(name) = source_account {
                if total_taken > 0.0 {
                    if let Some(column) = balances.get_mut(&name) {
                        let before = column[t];
                        column[t] = round_amount(before - total_taken);
                        let mut entry = JournalEntry::new(
                            t,
                            &date_str,
                            format!("waterfall:{}", waterfall.name),
                            "allocate_out",
                            format!("account:{name}"),
                            "applied",
                        );
                        entry.amount = Some(round_amount(total_taken));
                        entry.pot_before = Some(round_amount(before));
                        entry.pot_after = Some(column[t]);
                        self.journal.push(entry);
                    }
                }
            }
        }
    }

    /// The stage's outputs: each step's column, and the journal of every
    /// allocation and inflow, in the order the run happened in.
    pub(crate) fn finish(self) -> (BTreeMap<String, Vec<f64>>, Vec<JournalEntry>) {
        (self.out, self.journal)
    }
}
