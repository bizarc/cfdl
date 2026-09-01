// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

/// Warn when a stream's cash settles in the projection tail.
///
/// The tail is evaluated so `series_sum` can look forward — a forward-NOI exit
/// reads a year past the sale — but it contributes nothing to cash results,
/// totals or NPV. A stream that *deliberately* runs into the tail to feed a
/// valuation is doing the right thing, and its tail values are meant to be
/// excluded; warning on those would fire on every forward-NOI model.
///
/// What is worth flagging is cash that lands there without the author asking:
/// a schedule ending on the cash horizon whose payment terms then move the
/// last settlement past it. docs/12_payment_timing.md promises a flow is never
/// silently dropped, and before this it was — the amount simply vanished.
pub(crate) fn warn_if_cash_settles_in_tail(
    stream: &IrStream,
    values: &[f64],
    cash_periods: usize,
    warnings: &mut Vec<String>,
) {
    if values.len() <= cash_periods {
        return;
    }
    if stream.schedule.net_days.is_none() && stream.schedule.net_months.is_none() {
        return;
    }
    let stranded: f64 = values[cash_periods..].iter().sum();
    if stranded.abs() < 1e-9 {
        return;
    }
    warnings.push(format!(
        "Stream '{}' settles {:.2} in the projection tail: its payment terms move cash past period {}, and the tail is computed for series lookups only, so that amount is excluded from cash results and NPV. Extend `for <n>` to cover the lag, or shorten the schedule.",
        stream.name, stranded, cash_periods
    ));
}

pub(crate) fn stream_direction_sign(stream: &IrStream, warnings: &mut Vec<String>) -> f64 {
    match stream.direction.as_str() {
        "inflow" => 1.0,
        "outflow" => -1.0,
        _ => {
            warnings.push(format!(
                "Stream '{}' has unknown direction '{}'; treating as outflow.",
                stream.name, stream.direction
            ));
            -1.0
        }
    }
}

/// Evaluate one stream over the full timeline (projection tail included).
/// `series` is Some only for phase-2 streams and enables series_sum/avg.
#[allow(clippy::too_many_arguments)]
/// A stream's compiled form: everything that does not depend on the period.
///
/// SPLIT OUT SO ONE ARITHMETIC SERVES TWO ORDERS. The engine evaluates a
/// stream's whole column today; the period walk of `docs/28` §3 needs the same
/// stream one period at a time. Compiling and scheduling once, then stepping,
/// means the two orders cannot drift into computing different numbers — the
/// column path below is a loop over `step`, not a second implementation.
///
/// A waterfall is NOT here and never will be. It is its own stage, allocating
/// cash that streams have already produced (`docs/17` §4); stream evaluation
/// does not know it exists.
pub(crate) struct StreamPlan<'a> {
    stream: &'a IrStream,
    amount_expr: cfdl_expr::CompiledExpr,
    active_expr: cfdl_expr::CompiledExpr,
    /// `accruals[pay_idx]` holds the accrual periods that settle at `pay_idx`.
    /// An accrual is never later than the period it settles in, which is what
    /// makes a per-period step well defined.
    accruals: Vec<Vec<usize>>,
    direction_sign: f64,
}

impl<'a> StreamPlan<'a> {
    /// Whether this stream's schedule anchors to a state entry — the one
    /// schedule whose membership resolves during the walk (`docs/28` §6.2).
    pub(crate) fn is_state_anchored(&self) -> bool {
        self.stream.schedule.kind == "StateEnter"
    }

    /// The anchor's entity and state, for the caller collecting entries.
    pub(crate) fn anchor(&self) -> Option<(&str, &str)> {
        match (
            self.stream.schedule.anchor_entity.as_deref(),
            self.stream.schedule.anchor_state.as_deref(),
        ) {
            (Some(entity), Some(state)) => Some((entity, state)),
            _ => None,
        }
    }

    /// Recompute this plan's accruals from the entries known so far. Sound to
    /// call once per period under the walk: an entry's window opens AT the
    /// entry and extends forward, so periods already evaluated cannot gain an
    /// accrual — each recomputation agrees with every earlier one about the
    /// past, and a re-entered state re-anchors by contributing a new window.
    pub(crate) fn re_anchor(
        &mut self,
        entries: &[usize],
        timeline: &[Date],
    ) -> Result<(), EngineError> {
        self.accruals = anchored_accruals(&self.stream.schedule, entries, timeline)?;
        Ok(())
    }

    /// Does anything settle at this period?
    ///
    /// The grid is not the deal: a ten-year model with two years of activity
    /// leaves most cells empty, and a walk that steps the whole grid must be
    /// able to tell an empty cell apart from a computed zero without paying to
    /// find out.
    pub(crate) fn settles_at(&self, pay_idx: usize) -> bool {
        self.accruals.get(pay_idx).is_some_and(|a| !a.is_empty())
    }

    /// One period of this stream: what settles at `pay_idx`.
    ///
    /// Reads the same gates in the same order as the column path — the event
    /// mask first, then the stream's own `active when` — because both must
    /// pass and the second is the stream's own declaration.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn step(
        &self,
        ir: &Ir,
        config: &RunConfig,
        pay_idx: usize,
        timeline: &[Date],
        base_inputs: &BTreeMap<String, f64>,
        // The state this period sees, passed as the two pieces a stream reads
        // rather than as a finished `EventSim`. A walk supplies them while the
        // state stage is still advancing; the column order supplies them from
        // a completed one. Neither can tell the difference, which is what lets
        // one stream implementation serve both.
        entity_state: &[BTreeMap<String, BTreeMap<String, ExprValue>>],
        stream_active: &BTreeMap<String, Vec<bool>>,
        states: &BTreeMap<String, Vec<f64>>,
        series: Option<&Arc<BTreeMap<String, Vec<f64>>>>,
        warnings: &mut Vec<String>,
        activation_refused: &mut Vec<usize>,
        // The last period whose series values exist. `None` when the columns
        // are finished, which is the column order; `Some(t)` under a walk,
        // where reading past `t` would read an allocation rather than a value.
        series_available_to: Option<usize>,
    ) -> f64 {
        let event_mask = stream_active.get(&self.stream.name);
        let mut settled = 0.0_f64;
        let Some(accruals) = self.accruals.get(pay_idx) else {
            return 0.0;
        };
        for &idx in accruals {
            if let Some(mask) = event_mask {
                if !mask[idx] {
                    continue;
                }
            }
            let mut env = build_expr_env(
                ir,
                Some(self.stream),
                config,
                idx,
                &timeline[idx],
                base_inputs,
            );
            apply_entity_state(&mut env, &entity_state[idx], &self.stream.owner.symbol);
            bind_states(&mut env, states, idx);
            bind_all_entity_state(&mut env, &entity_state[idx]);
            if let Some(series) = series {
                env.series = Arc::clone(series);
            }
            env.series_available_to = series_available_to;
            let active_value = eval_bool_expr(
                &self.active_expr,
                &env,
                "Stream",
                &self.stream.name,
                "active_when",
                warnings,
            );
            if !active_value {
                if event_mask.is_some_and(|mask| mask[idx]) {
                    activation_refused.push(idx);
                }
                continue;
            }
            let amount =
                if let Some(override_value) = stream_amount_override(config, &self.stream.name) {
                    override_value
                } else {
                    eval_amount_expr(
                        &self.amount_expr,
                        &env,
                        &self.stream.name,
                        &ir.model.currency,
                        warnings,
                    )
                };
            // Evaluated against the accrual period, paid in the payment period.
            settled += amount * self.direction_sign;
        }
        settled
    }
}

/// Compile and schedule a stream, once, for either evaluation order.
pub(crate) fn plan_stream<'a>(
    ir: &Ir,
    stream: &'a IrStream,
    timeline: &[Date],
    warnings: &mut Vec<String>,
) -> Result<StreamPlan<'a>, EngineError> {
    let _ = ir;
    if let Some(lang) = &stream.amount.lang {
        if lang != "cfdl" {
            warnings.push(format!(
                "Stream '{}' amount language '{}' is unsupported; expression is treated as CEL.",
                stream.name, lang
            ));
        }
    }
    let amount_expr = match cfdl_expr::compile_expr(&stream.amount.src) {
        Ok(compiled) => compiled,
        Err(err) => {
            warnings.push(format!(
                "Stream '{}' amount expression compile failed [{}]: {}; using 0.",
                stream.name, err.code, err.message
            ));
            cfdl_expr::compile_expr("0").expect("constant expression compiles")
        }
    };
    let active_src = stream
        .active_when
        .as_ref()
        .map(|expr| expr.src.as_str())
        .unwrap_or("true");
    if let Some(expr) = &stream.active_when {
        if let Some(lang) = &expr.lang {
            if lang != "cfdl" {
                warnings.push(format!(
                    "Stream '{}' active_when language '{}' is unsupported; expression is treated as CEL.",
                    stream.name, lang
                ));
            }
        }
    }
    let active_expr = match cfdl_expr::compile_expr(active_src) {
        Ok(compiled) => compiled,
        Err(err) => {
            warnings.push(format!(
                "Stream '{}' active_when expression compile failed [{}]: {}; using true.",
                stream.name, err.code, err.message
            ));
            cfdl_expr::compile_expr("true").expect("constant expression compiles")
        }
    };

    let accruals = schedule_accruals(&stream.schedule, timeline)?;
    let direction_sign = stream_direction_sign(stream, warnings);
    Ok(StreamPlan {
        stream,
        amount_expr,
        active_expr,
        accruals,
        direction_sign,
    })
}

/// A stream's whole column: the period walk's step, run over every period.
///
/// Kept as the engine's existing entry point so the column order is unchanged
/// and provably shares its arithmetic with the walk.
#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_stream(
    ir: &Ir,
    config: &RunConfig,
    stream: &IrStream,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    event_sim: &EventSim,
    states: &BTreeMap<String, Vec<f64>>,
    series: Option<&Arc<BTreeMap<String, Vec<f64>>>>,
    warnings: &mut Vec<String>,
    // Periods where an event turned this stream ON and the stream's own
    // `active when` then refused it. Collected rather than warned so the
    // journal can say so once, with a count, instead of once per period.
    activation_refused: &mut Vec<usize>,
) -> Result<Vec<f64>, EngineError> {
    let mut plan = plan_stream(ir, stream, timeline, warnings)?;
    // Under the column order the state stage has fully settled before any
    // stream runs, so a state-anchored schedule's entries are all known here
    // — the same records the walk reads incrementally (`docs/28` §6.2).
    if plan.is_state_anchored() {
        if let Some((entity, state)) = plan.anchor() {
            let (entity, state) = (entity.to_string(), state.to_string());
            let entries = crate::state::entries_into(ir, &event_sim.transitions, &entity, &state);
            plan.re_anchor(&entries, timeline)?;
        }
    }
    let mut values = vec![0.0_f64; timeline.len()];
    for (pay_idx, slot) in values.iter_mut().enumerate() {
        *slot = plan.step(
            ir,
            config,
            pay_idx,
            timeline,
            base_inputs,
            &event_sim.entity_state,
            &event_sim.stream_active,
            states,
            series,
            warnings,
            activation_refused,
            None,
        );
    }
    Ok(values)
}

/// Fold a stream's cash-horizon slice into model totals and reporting maps.
pub(crate) fn record_stream(
    stream: &IrStream,
    values: &[f64],
    cash_periods: usize,
    model_series: &mut [f64],
    stream_totals: &mut BTreeMap<String, f64>,
    stream_series: &mut BTreeMap<String, Vec<f64>>,
) {
    let cash = &values[..cash_periods.min(values.len())];
    for (idx, value) in cash.iter().enumerate() {
        model_series[idx] += *value;
    }
    let total = cash.iter().sum::<f64>();
    stream_totals.insert(stream.name.clone(), total);
    stream_series.insert(stream.name.clone(), cash.to_vec());
}

/// Every literal series name a model reads must name something the model
/// produces.
///
/// `series_aggregate` returns 0 for a name that matches nothing, deliberately:
/// a selector like `credit.pool.prepay.*` must contribute nothing when no
/// contract lowered a prepayment stream. That default is right for a selector
/// and wrong for a spelled-out name, which can only be a name that no longer
/// exists or never did — and zero is the one answer a reader will not question.
///
/// Selectors are therefore left alone and literals are checked. The universe is
/// every stream in the IR (a pack's lowered streams are already among them) and
/// every waterfall step, which publishes as `<waterfall>.<step>`.
///
/// WHY THIS WARNS RATHER THAN FAILS. A literal name matching nothing is a pack
/// idiom, not only a typo. `cre.exit` sums nine NOI components by name —
/// `cre.unit.base_rent.*` and `cre.rollover.rent.*` as selectors, but
/// `cre.lease.base_rent` and `cre.ops.revenue` as literals, because a single
/// lease lowers to an unsuffixed stream — and a property with no such contract
/// must contribute nothing rather than fail. Refusing literals outright broke
/// four goldens on exactly that. The visible warning is the fix that survives
/// the idiom; refusing it would need the convention settled first.
pub(crate) fn check_series_names(ir: &Ir, warnings: &mut Vec<String>) {
    let mut known: BTreeSet<String> = ir.streams.iter().map(|s| s.name.clone()).collect();
    for waterfall in &ir.waterfalls {
        for step in &waterfall.steps {
            known.insert(format!("{}.{}", waterfall.name, step.name));
        }
    }

    let mut sources: Vec<(String, &str)> = Vec::new();
    for stream in &ir.streams {
        // A PACK'S OWN EXPRESSION IS NOT THE MODELLER'S TO FIX. `cre.exit`
        // names nine NOI components and a given property declares some of
        // them; the unmatched ones are the idiom working, and warning about
        // them on every CRE model would teach a reader to ignore the warning
        // that matters.
        if stream
            .provenance
            .as_ref()
            .is_some_and(|p| p.generated_by.is_some())
        {
            continue;
        }
        sources.push((
            format!("stream '{}'", stream.name),
            stream.amount.src.as_str(),
        ));
        if let Some(guard) = &stream.active_when {
            sources.push((format!("stream '{}'", stream.name), guard.src.as_str()));
        }
    }
    for waterfall in &ir.waterfalls {
        sources.push((
            format!("waterfall '{}'", waterfall.name),
            waterfall.source.src.as_str(),
        ));
        for step in &waterfall.steps {
            sources.push((
                format!("waterfall '{}' step '{}'", waterfall.name, step.name),
                step.amount.src.as_str(),
            ));
        }
    }
    for entity in &ir.entities {
        for (field, rule) in &entity.rules {
            for src in [rule.init.src.as_str(), rule.next.src.as_str()] {
                sources.push((format!("field '{}.{field}'", entity.symbol), src));
            }
        }
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    for (where_, src) in sources {
        for referenced in series_references(src) {
            if referenced.ends_with(".*") || known.contains(&referenced) {
                continue;
            }
            if !seen.insert(format!("{where_}|{referenced}")) {
                continue;
            }
            warnings.push(format!(
                "W5022_UNKNOWN_SERIES_REFERENCE: in {where_}, a series reduction \
                 (`series_sum`, `series_avg`, `series_min`, `series_max`, `series_prod`, \
                 `series_count`) names series '{referenced}', which no stream, contract or \
                 waterfall step in this model produces. It reduces over nothing, so anything \
                 reading it is reading nothing. Check the spelling; a selector ending in `.*` \
                 states that matching nothing is intended."
            ));
        }
    }
}

pub(crate) use cfdl_expr::series_references;
