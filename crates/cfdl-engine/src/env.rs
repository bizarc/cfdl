// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

/// Evaluate a boolean guard, reporting a failure against the thing that owns it.
///
/// `subject_kind` exists because this is called from three places with three
/// different kinds of subject — a stream's `active when`, an event's `when`,
/// and an option's `exercise when` — and it used to hardcode "Stream". An
/// option that failed to evaluate was reported as a stream, sending a reader
/// looking for something that does not exist.
pub(crate) fn eval_bool_expr(
    expr: &CompiledExpr,
    env: &ExprEnv,
    subject_kind: &str,
    subject_name: &str,
    slot: &str,
    warnings: &mut Vec<String>,
) -> bool {
    match cfdl_expr::eval(expr, env) {
        Ok(ExprValue::Bool(value)) => value,
        Ok(other) => {
            warnings.push(format!(
                "{subject_kind} '{}' {} expression returned non-bool '{other:?}'; using false.",
                subject_name, slot
            ));
            false
        }
        Err(err) => {
            warnings.push(format!(
                "{subject_kind} '{}' {} evaluation failed [{}]: {}; using false.",
                subject_name, slot, err.code, err.message
            ));
            false
        }
    }
}

pub(crate) fn eval_amount_expr(
    expr: &CompiledExpr,
    env: &ExprEnv,
    stream_name: &str,
    model_currency: &str,
    warnings: &mut Vec<String>,
) -> f64 {
    match cfdl_expr::eval(expr, env) {
        Ok(value) => match value {
            ExprValue::Int(v) => v as f64,
            ExprValue::Decimal(v) => v,
            ExprValue::Money(m) => {
                if !m.currency.eq_ignore_ascii_case(model_currency) {
                    warnings.push(format!(
                        "Stream '{}' amount returned currency '{}', expected '{}'; using amount without FX conversion.",
                        stream_name, m.currency, model_currency
                    ));
                }
                m.amount
            }
            other => {
                warnings.push(format!(
                    "Stream '{}' amount returned non-numeric value '{other:?}'; using 0.",
                    stream_name
                ));
                0.0
            }
        },
        Err(err) => {
            warnings.push(format!(
                "Stream '{}' amount evaluation failed [{}]: {}; using 0.",
                stream_name, err.code, err.message
            ));
            0.0
        }
    }
}

/// Curve declarations from IR as expression-env curve defs. Dates were
/// normalized and validated by the compiler; unparseable points are skipped.
pub(crate) fn ir_curve_defs(ir: &Ir) -> BTreeMap<String, cfdl_expr::CurveDef> {
    let mut out = BTreeMap::new();
    for curve in &ir.curves {
        let points = curve
            .points
            .iter()
            .filter_map(|p| {
                Date::parse(&p.date).ok().map(|d| {
                    (
                        cfdl_expr::Date {
                            year: d.year,
                            month: d.month,
                            day: d.day,
                        },
                        p.value,
                    )
                })
            })
            .collect();
        out.insert(
            curve.name.clone(),
            cfdl_expr::CurveDef {
                interpolation: curve.interpolation.clone(),
                points,
            },
        );
    }
    out
}

/// The name of the phase covering this period, for `time.phase`.
///
/// `docs/01` §16.2 requires the expression environment to support it and the
/// course calls it "the current phase's name", so it answers with the name and
/// with null only where no declared phase covers the date. The membership test
/// is the one `state.rs` already applies to an option's `exercisable_in_phase`;
/// written once here so a phase means the same thing to a guard and to an
/// option.
///
/// DECLARATION ORDER BREAKS A TIE. Overlapping phases compile today, so two can
/// cover one period; the first declared wins, which makes the answer a stated
/// order rather than a map iteration.
pub(crate) fn phase_at(ir: &Ir, date: &Date) -> ExprValue {
    ir.phases
        .iter()
        .find(|phase| {
            Date::parse(&phase.range.start)
                .map(|start| *date >= start)
                .unwrap_or(false)
                && Date::parse(&phase.range.end)
                    .map(|end| *date <= end)
                    .unwrap_or(false)
        })
        .map(|phase| ExprValue::String(phase.name.clone()))
        .unwrap_or(ExprValue::Optional(None))
}

/// Entity-independent environment (model/time/cfg/obs/inputs) used for event
/// and option evaluation.
pub(crate) fn build_base_env(
    ir: &Ir,
    config: &RunConfig,
    t: usize,
    date: &Date,
    base_inputs: &BTreeMap<String, f64>,
) -> ExprEnv {
    let mut env = ExprEnv::empty();
    env.mode = config.arithmetic;
    env.model.insert(
        "id".to_string(),
        ExprValue::String(ir.model.name.clone().unwrap_or_else(|| "model".to_string())),
    );
    env.model.insert(
        "base_currency".to_string(),
        ExprValue::Currency(ir.model.currency.clone()),
    );
    env.time.insert("t".to_string(), ExprValue::Int(t as i64));
    env.time.insert(
        "date".to_string(),
        ExprValue::Date(cfdl_expr::Date {
            year: date.year,
            month: date.month,
            day: date.day,
        }),
    );
    env.time.insert("phase".to_string(), phase_at(ir, date));
    // Periods per year for the model's calendar, so a hand-written model can
    // spread an annual figure without hardcoding a divisor:
    //   amount = inputs.rent_year / time.ppy
    // Packs do NOT use this — a lowering rule resolves its own periods-per-year
    // at compile time, because a rule may pay on its own interval (a monthly
    // coupon on a daily grid needs 12, not 365) and only the compiler can see
    // that. See {{model.periods_per_year}} in cfdl-compile.
    env.time.insert(
        "ppy".to_string(),
        ExprValue::Decimal(periods_per_year(&ir.time.calendar)),
    );
    // Actual calendar days in this period, so an Actual/360 or Actual/365
    // accrual can be expressed. Packs reach it through
    // {{model.accrual_divisor}}; a hand-written model may use it directly.
    env.time.insert(
        "days_in_period".to_string(),
        ExprValue::Decimal(days_in_period(&ir.time.calendar, date)),
    );
    env.curves = ir_curve_defs(ir);
    for (name, value) in base_inputs {
        env.inputs.insert(name.clone(), ExprValue::Decimal(*value));
    }
    for (key, value) in &config.parameter_overrides {
        if let Some(stripped) = key.strip_prefix("cfg.") {
            insert_cfg_value(&mut env.cfg, stripped, *value);
        } else if let Some(stripped) = key.strip_prefix("obs.") {
            insert_cfg_value(&mut env.obs, stripped, *value);
        } else if let Some(stripped) = key.strip_prefix("inputs.") {
            env.inputs
                .insert(stripped.to_string(), ExprValue::Decimal(*value));
        }
    }
    env
}

/// Render a state value for the transition log.
///
/// A lifecycle state is text; a numeric or boolean field is rendered as
/// written. The log is for reading and for asserting on, so the rendering is
/// stable rather than clever.
pub(crate) fn describe_value(value: &ExprValue) -> String {
    match value {
        ExprValue::String(text) => text.clone(),
        ExprValue::Bool(flag) => flag.to_string(),
        ExprValue::Int(n) => n.to_string(),
        ExprValue::Decimal(n) => round_amount(*n).to_string(),
        other => format!("{other:?}"),
    }
}

/// Bind every entity's LITERAL fields under its family path.
///
/// Constants only. A field carrying a rule is deliberately absent: inside
/// another rule its period-close value has not been computed, and `E1127`
/// rejects the read rather than letting it resolve to nothing.
pub(crate) fn bind_literal_fields(env: &mut ExprEnv, ir: &Ir) {
    for entity in &ir.entities {
        let Some((namespace, name)) = entity.symbol.split_once('.') else {
            continue;
        };
        let mut values: BTreeMap<String, ExprValue> = BTreeMap::new();
        for (field, raw) in &entity.fields {
            let value = match raw.parse::<f64>() {
                Ok(number) => ExprValue::Decimal(number),
                Err(_) => ExprValue::String(raw.clone()),
            };
            values.insert(field.clone(), value);
        }
        if values.is_empty() {
            continue;
        }
        match env.entity.get_mut(namespace) {
            Some(ExprValue::Map(ns_map)) => match ns_map.get_mut(name) {
                Some(ExprValue::Map(existing)) => {
                    for (field, value) in values {
                        existing.entry(field).or_insert(value);
                    }
                }
                _ => {
                    ns_map.insert(name.to_string(), ExprValue::Map(values));
                }
            },
            _ => {
                let mut ns_map = BTreeMap::new();
                ns_map.insert(name.to_string(), ExprValue::Map(values));
                env.entity
                    .insert(namespace.to_string(), ExprValue::Map(ns_map));
            }
        }
    }
}

/// Bind `state.<name>` to each declared state's value AT period `idx`.
///
/// Extracted so a stream and an option bind the SAME period by construction
/// rather than by two copies agreeing. `prev_states`/`prev_self` are left
/// empty, so `prev` is not merely rejected outside a recurrence — it is not
/// there to be found. See docs/14_state_and_recurrence.md.
pub(crate) fn bind_states(env: &mut ExprEnv, states: &BTreeMap<String, Vec<f64>>, idx: usize) {
    env.states = states
        .iter()
        .filter_map(|(name, values)| {
            values
                .get(idx)
                .map(|v| (name.clone(), ExprValue::Decimal(*v)))
        })
        .collect();

    // A STREAM READS A FIELD'S PREVIOUS PERIOD TOO.
    //
    // `_open` states existed because a stream could not: a debt schedule
    // charging interest on the average of a period's opening and closing
    // balance had to declare the quantity twice, and the second was never a
    // quantity — it was this accessor, missing. Only field paths are bound, so
    // bare `prev` still means nothing outside a rule.
    //
    // BOTH SPELLINGS, because a CURRENT-period read already has both:
    // `asset.tlb.balance` IS `entity.asset.tlb.balance`, and binding one end of
    // that alias and not the other is not a rule anybody could learn. A PACK
    // LOWERING RULE IS WHERE IT BIT. A rule writes `field.<name>` and lowering
    // rewrites that to the `entity.` long form deliberately — the bare alias
    // covers the four declared families only, and a rule may sit on any entity.
    // So `prev.field.<name>` — the average-balance accessor this block exists
    // to provide — arrived here as `prev.entity.asset.x.bal`, matched nothing,
    // and evaluated as a SILENT ZERO with a warning rather than an error. No
    // shipped pack exercised it; the credit pack reaches the same value by
    // declaring a second, lagged field.
    if idx > 0 {
        env.prev_states = states
            .iter()
            .filter(|(name, _)| name.matches('.').count() == 2)
            .filter_map(|(name, values)| values.get(idx - 1).map(|v| (name, *v)))
            .flat_map(|(name, value)| {
                [
                    (format!("entity.{name}"), ExprValue::Decimal(value)),
                    (name.clone(), ExprValue::Decimal(value)),
                ]
            })
            .collect();
    }

    // A FIELD RULE IS READ WHERE THE FIELD IS, not under `state.`.
    //
    // `compute_states` keys a rule by its entity path, so `asset.tlb.balance`
    // arrives here as one name. Binding it into the entity map is what makes
    // the computed value answer to the same spelling as a stated one — a
    // reader should not have to know whether a field holds or moves to know
    // how to read it.
    for (name, values) in states {
        let Some(value) = values.get(idx) else {
            continue;
        };
        let mut parts = name.split('.');
        let (Some(family), Some(entity), Some(field)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if parts.next().is_some() {
            continue;
        }
        let ExprValue::Map(family_map) = env
            .entity
            .entry(family.to_string())
            .or_insert_with(|| ExprValue::Map(BTreeMap::new()))
        else {
            continue;
        };
        let ExprValue::Map(entity_map) = family_map
            .entry(entity.to_string())
            .or_insert_with(|| ExprValue::Map(BTreeMap::new()))
        else {
            continue;
        };
        entity_map.insert(field.to_string(), ExprValue::Decimal(*value));
    }
}

/// Bind every entity's state under its qualified path — `entity.asset.tower.status`.
///
/// A stream reads `entity.<field>` relative to its owner, which works because a
/// stream HAS one. An event does not, so the qualified form is how an event
/// guard names the thing it is asking about. An option has an owner (it is a
/// contract), so it gets both.
pub(crate) fn bind_all_entity_state(
    env: &mut ExprEnv,
    state_by_entity: &BTreeMap<String, BTreeMap<String, ExprValue>>,
) {
    for (symbol, fields) in state_by_entity {
        let Some((namespace, name)) = symbol.split_once('.') else {
            continue;
        };
        // MERGE, DO NOT REPLACE. An entity's field values are bound first and
        // its lifecycle state second; overwriting the map here dropped the
        // fields, so an event guard reading one saw a key that was not there.
        // Under the open-world `entity` root that resolved to NULL rather than
        // failing — the guard compared null to a number, went false forever,
        // and the event never fired.
        match env.entity.get_mut(namespace) {
            Some(ExprValue::Map(ns_map)) => match ns_map.get_mut(name) {
                Some(ExprValue::Map(existing)) => {
                    for (field, value) in fields {
                        existing.insert(field.clone(), value.clone());
                    }
                }
                _ => {
                    ns_map.insert(name.to_string(), ExprValue::Map(fields.clone()));
                }
            },
            _ => {
                let mut ns_map = BTreeMap::new();
                ns_map.insert(name.to_string(), ExprValue::Map(fields.clone()));
                env.entity
                    .insert(namespace.to_string(), ExprValue::Map(ns_map));
            }
        }
    }
}

/// Expose an entity's event-driven state to expressions, both as
/// `entity.state.<field>` and directly as `entity.<field>` (spec §12.3).
pub(crate) fn apply_entity_state(
    env: &mut ExprEnv,
    state_by_entity: &BTreeMap<String, BTreeMap<String, ExprValue>>,
    owner_symbol: &str,
) {
    let Some(state) = state_by_entity.get(owner_symbol) else {
        return;
    };
    for (field, value) in state {
        env.entity.insert(field.clone(), value.clone());
    }
    // `entity.state.<field>` IS GONE. It was a second store under a second
    // name: an event's write lived there while the field's rule lived under the
    // field's own path, so one name answered differently depending on who
    // asked. An entity's state IS its fields, and lifecycle status is one of
    // them.
}

pub(crate) fn build_expr_env(
    ir: &Ir,
    stream: Option<&IrStream>,
    config: &RunConfig,
    t: usize,
    date: &Date,
    base_inputs: &BTreeMap<String, f64>,
) -> ExprEnv {
    let mut env = ExprEnv::empty();
    env.mode = config.arithmetic;
    env.model.insert(
        "id".to_string(),
        ExprValue::String(ir.model.name.clone().unwrap_or_else(|| "model".to_string())),
    );
    env.model.insert(
        "base_currency".to_string(),
        ExprValue::Currency(ir.model.currency.clone()),
    );

    env.time.insert("t".to_string(), ExprValue::Int(t as i64));
    env.time.insert(
        "date".to_string(),
        ExprValue::Date(cfdl_expr::Date {
            year: date.year,
            month: date.month,
            day: date.day,
        }),
    );
    env.time.insert("phase".to_string(), phase_at(ir, date));
    // Periods per year for the model's calendar, so a hand-written model can
    // spread an annual figure without hardcoding a divisor:
    //   amount = inputs.rent_year / time.ppy
    // Packs do NOT use this — a lowering rule resolves its own periods-per-year
    // at compile time, because a rule may pay on its own interval (a monthly
    // coupon on a daily grid needs 12, not 365) and only the compiler can see
    // that. See {{model.periods_per_year}} in cfdl-compile.
    env.time.insert(
        "ppy".to_string(),
        ExprValue::Decimal(periods_per_year(&ir.time.calendar)),
    );
    // Actual calendar days in this period, so an Actual/360 or Actual/365
    // accrual can be expressed. Packs reach it through
    // {{model.accrual_divisor}}; a hand-written model may use it directly.
    env.time.insert(
        "days_in_period".to_string(),
        ExprValue::Decimal(days_in_period(&ir.time.calendar, date)),
    );

    env.entity.insert(
        "id".to_string(),
        ExprValue::String(stream.map_or_else(String::new, |s| s.owner.symbol.clone())),
    );
    env.entity.insert(
        "name".to_string(),
        ExprValue::String(stream.map_or_else(String::new, |s| s.owner.symbol.clone())),
    );
    // No `entity.state` map — see apply_entity_state.
    env.curves = ir_curve_defs(ir);

    for (name, value) in base_inputs {
        env.inputs.insert(name.clone(), ExprValue::Decimal(*value));
    }
    for (key, value) in &config.parameter_overrides {
        if let Some(stripped) = key.strip_prefix("cfg.") {
            insert_cfg_value(&mut env.cfg, stripped, *value);
        } else if let Some(stripped) = key.strip_prefix("obs.") {
            insert_cfg_value(&mut env.obs, stripped, *value);
        } else if let Some(stripped) = key.strip_prefix("inputs.") {
            env.inputs
                .insert(stripped.to_string(), ExprValue::Decimal(*value));
        }
    }
    env
}

/// Resolve `assume` values from IR: constants evaluate their expression;
/// random assumptions contribute their deterministic central value (Monte
/// Carlo trials override these via `inputs.<name>` parameter overrides).
pub(crate) fn assumption_inputs(
    ir: &Ir,
    warnings: &mut Vec<String>,
) -> Result<BTreeMap<String, f64>, EngineError> {
    let mut inputs = BTreeMap::new();
    let empty_env = ExprEnv::empty();
    for (name, constant) in &ir.assumptions.constants {
        match cfdl_expr::compile_expr(&constant.expr.src)
            .and_then(|compiled| cfdl_expr::eval(&compiled, &empty_env))
        {
            Ok(ExprValue::Decimal(v)) => {
                inputs.insert(name.clone(), v);
            }
            Ok(ExprValue::Int(v)) => {
                inputs.insert(name.clone(), v as f64);
            }
            Ok(other) => warnings.push(format!(
                "Assumption '{name}' evaluated to non-numeric {other:?}; ignoring."
            )),
            Err(err) => warnings.push(format!(
                "Assumption '{name}' failed to evaluate [{}]: {}; ignoring.",
                err.code, err.message
            )),
        }
    }
    for (name, random) in &ir.assumptions.random {
        let spec = ir_distribution_spec(&random.dist)?;
        inputs.insert(
            name.clone(),
            apply_clip(central_value(&spec), random.dist.clip),
        );
    }
    Ok(inputs)
}

pub(crate) fn stream_amount_override(config: &RunConfig, stream_name: &str) -> Option<f64> {
    let key = format!("stream.{stream_name}:amount");
    config.parameter_overrides.get(&key).copied()
}

pub(crate) fn insert_cfg_value(map: &mut BTreeMap<String, ExprValue>, path: &str, value: f64) {
    let mut segments = path.split('.').filter(|part| !part.is_empty());
    let Some(first) = segments.next() else {
        return;
    };
    let rest = segments.collect::<Vec<_>>();
    if rest.is_empty() {
        map.insert(first.to_string(), ExprValue::Decimal(value));
        return;
    }
    let entry = map
        .entry(first.to_string())
        .or_insert_with(|| ExprValue::Map(BTreeMap::new()));
    insert_cfg_into_value(entry, &rest, value);
}

pub(crate) fn insert_cfg_into_value(slot: &mut ExprValue, path: &[&str], value: f64) {
    if path.is_empty() {
        *slot = ExprValue::Decimal(value);
        return;
    }
    if !matches!(slot, ExprValue::Map(_)) {
        *slot = ExprValue::Map(BTreeMap::new());
    }
    let ExprValue::Map(map) = slot else {
        return;
    };
    let head = path[0];
    let tail = &path[1..];
    if tail.is_empty() {
        map.insert(head.to_string(), ExprValue::Decimal(value));
        return;
    }
    let entry = map
        .entry(head.to_string())
        .or_insert_with(|| ExprValue::Map(BTreeMap::new()));
    insert_cfg_into_value(entry, tail, value);
}
