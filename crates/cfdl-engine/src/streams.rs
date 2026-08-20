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
) -> Result<Vec<f64>, EngineError> {
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

    let schedule_accruals = schedule_accruals(&stream.schedule, timeline)?;
    let event_mask = event_sim.stream_active.get(&stream.name);
    let mut values = vec![0.0_f64; timeline.len()];
    let direction_sign = stream_direction_sign(stream, warnings);
    // A period may receive several accruals — under net-30 both February and
    // March settle in March — so their amounts sum into that period.
    for (pay_idx, accruals) in schedule_accruals.iter().enumerate() {
        for &idx in accruals {
            if let Some(mask) = event_mask {
                if !mask[idx] {
                    continue;
                }
            }
            let mut env =
                build_expr_env(ir, Some(stream), config, idx, &timeline[idx], base_inputs);
            apply_entity_state(&mut env, &event_sim.entity_state[idx], &stream.owner.symbol);
            bind_states(&mut env, states, idx);
            // AN EVENT WRITES THE FIELD, so a stream must see what it wrote.
            //
            // Field values are bound first and event writes merged over them —
            // the order a waterfall already uses. Without this a stream read
            // the rule's value and a waterfall read the event's, so one field
            // name answered differently depending on who asked.
            bind_all_entity_state(&mut env, &event_sim.entity_state[idx]);
            if let Some(series) = series {
                env.series = Arc::clone(series);
            }
            let active_value = eval_bool_expr(
                &active_expr,
                &env,
                "Stream",
                &stream.name,
                "active_when",
                warnings,
            );
            if !active_value {
                continue;
            }
            let amount = if let Some(override_value) = stream_amount_override(config, &stream.name)
            {
                override_value
            } else {
                eval_amount_expr(
                    &amount_expr,
                    &env,
                    &stream.name,
                    &ir.model.currency,
                    warnings,
                )
            };
            // Evaluated against the accrual period, paid in the payment period.
            values[pay_idx] += amount * direction_sign;
        }
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

/// Expose an entity's event-driven state to expressions, both as
/// `entity.state.<field>` and directly as `entity.<field>` (spec §12.3).
/// Bind `state.<name>` to each declared state's value AT period `idx`.
///
/// Extracted so a stream and an option bind the SAME period by construction
/// rather than by two copies agreeing. `prev_states`/`prev_self` are left
/// empty, so `prev` is not merely rejected outside a recurrence — it is not
/// there to be found. See docs/14_state_and_recurrence.md.
/// The series names an expression reads, as written.
///
/// Only literal first arguments — `series_sum("a.b", ...)` — which is what a
/// cross-stream read is. A computed name is not addressed here and is left to
/// the runtime, where it still returns 0 for an unmatched name.
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
                "W5022_UNKNOWN_SERIES_REFERENCE: in {where_}, `series_sum`/`series_avg` names \
                 series '{referenced}', which no stream, contract or waterfall step in this \
                 model produces. It aggregates to zero, so anything reading it is reading \
                 nothing. Check the spelling; a selector ending in `.*` states that matching \
                 nothing is intended."
            ));
        }
    }
}

pub(crate) fn series_references(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    for func in ["series_sum", "series_avg"] {
        let mut from = 0usize;
        while let Some(rel) = src[from..].find(func) {
            let after = from + rel + func.len();
            from = after;
            let mut i = after;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'(' {
                continue;
            }
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'"' {
                continue;
            }
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            if i <= bytes.len() {
                out.push(src[start..i].to_string());
            }
        }
    }
    out
}
