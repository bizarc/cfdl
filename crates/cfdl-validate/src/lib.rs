//! CFDL validation pass (Milestone 5).

use cfdl_parser::{Cadence, ScheduleKind, Span, Stmt};
use cfdl_resolver::{ResolveOutput, SymbolTables};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub file: String,
    pub span: Span,
}

pub fn validate(output: &ResolveOutput, symbols: &SymbolTables) -> Vec<ValidationDiagnostic> {
    let (default_file, default_span) = default_anchor(output);
    let mut diagnostics = Vec::new();

    // The model's reporting currency; every stream must agree with it.
    let model_currency: String = output
        .source_statements
        .iter()
        .find_map(|stmt| match &stmt.statement {
            Stmt::Model(model) => model.currency.clone(),
            _ => None,
        })
        .unwrap_or_else(|| "USD".to_string());
    let model_currency = model_currency.as_str();

    let mut versions = Vec::new();
    let mut models = Vec::new();
    let mut times = Vec::new();
    let mut use_packs = Vec::new();
    let mut phases = std::collections::BTreeSet::new();

    for source_stmt in &output.source_statements {
        match &source_stmt.statement {
            Stmt::Version(stmt) => versions.push((source_stmt.file.as_str(), stmt.span)),
            Stmt::Model(stmt) => models.push((source_stmt.file.as_str(), stmt.span)),
            Stmt::Time(stmt) => times.push((source_stmt.file.as_str(), stmt.span)),
            Stmt::UsePack(stmt) => use_packs.push((source_stmt.file.as_str(), stmt.span)),
            Stmt::Phase(stmt) => {
                phases.insert(stmt.name.clone());
            }
            _ => {}
        }
    }

    let anchor = (default_file.as_str(), default_span);
    push_missing_or_multiple(
        &mut diagnostics,
        RequirementSpec {
            missing_code: "E1101_MISSING_VERSION",
            missing_message: "Model is missing required 'version' statement.",
            multiple_code: "E1104_MULTIPLE_VERSION",
            multiple_message: "Model contains multiple 'version' statements.",
        },
        &versions,
        anchor,
    );
    push_missing_or_multiple(
        &mut diagnostics,
        RequirementSpec {
            missing_code: "E1102_MISSING_MODEL",
            missing_message: "Model is missing required 'model' statement.",
            multiple_code: "E1105_MULTIPLE_MODEL",
            multiple_message: "Model contains multiple 'model' statements.",
        },
        &models,
        anchor,
    );
    push_missing_or_multiple(
        &mut diagnostics,
        RequirementSpec {
            missing_code: "E1103_MISSING_TIME",
            missing_message: "Model is missing required 'time' statement.",
            multiple_code: "E1106_MULTIPLE_TIME",
            multiple_message: "Model contains multiple 'time' statements.",
        },
        &times,
        anchor,
    );
    if use_packs.len() > 1 {
        diagnostics.push(ValidationDiagnostic {
            code: "E1107_MULTIPLE_USE_PACK",
            message: "Model contains multiple 'use pack' statements.".to_string(),
            file: use_packs[1].0.to_string(),
            span: use_packs[1].1,
        });
    }
    for (file, span) in &use_packs {
        if *file != "model.cfdl" {
            diagnostics.push(ValidationDiagnostic {
                code: "E1108_USE_PACK_NOT_IN_MODEL_FILE",
                message: "The 'use pack' statement is only allowed in 'model.cfdl'.".to_string(),
                file: (*file).to_string(),
                span: *span,
            });
        }
    }
    if symbols.entities.is_empty() {
        diagnostics.push(ValidationDiagnostic {
            code: "E1109_MISSING_ENTITY",
            message: "Model must declare at least one entity.".to_string(),
            file: default_file.clone(),
            span: default_span,
        });
    }

    let Some((timeline_file, timeline)) = choose_timeline(output) else {
        sort_diagnostics(&mut diagnostics);
        return diagnostics;
    };

    // --- state declarations -------------------------------------------------
    //
    // A recurrence with an unstated base case would evaluate to zero for every
    // period, so `init` is required rather than defaulted (see
    // docs/14_state_and_recurrence.md). `prev` outside `next` has no referent —
    // there is no period -1 — and `state.<name>` inside `next` would name the
    // CURRENT period, which is the same-period edge the whole design prevents.
    // A FIELD'S RULE, held to the same two rules a state's is.
    //
    // `docs/18` §4a settles what a bare field read means inside a `next`: it is
    // rejected. Everywhere else `asset.tlb.balance` means this period's value
    // at close, and inside a rule that value does not exist yet — so rather
    // than quietly meaning the previous period, it is an error naming the
    // spelling that says so.
    //
    // Silence would be the worst answer here. A bare family path resolves
    // through the open-world `entity` root, so an unrejected read returns null
    // and evaluates to ZERO: the failure mode of `init = 100`, of the unbound
    // entity attributes, and of the bare waterfall path.
    // A rule may not read a field that MOVES — its period-close value does not
    // exist yet. A LITERAL is a constant, readable at any period, and rejecting
    // it was over-strict: the same mistake as treating a knowable value as
    // unknowable, pointing the other way.
    let mut rule_fields: BTreeSet<String> = BTreeSet::new();
    for source_stmt in &output.source_statements {
        if let Stmt::Entity(entity) = &source_stmt.statement {
            for f in &entity.fields {
                rule_fields.insert(format!("{}.{}", entity.symbol(), f.name));
            }
        }
    }

    for source_stmt in &output.source_statements {
        let Stmt::Entity(entity) = &source_stmt.statement else {
            continue;
        };
        let symbol = entity.symbol();
        for field in &entity.fields {
            for (clause, slot) in [("init", Some(&field.init)), ("next", field.next.as_ref())] {
                let Some(slot) = slot else { continue };
                if reads_moving_field(&slot.src, &rule_fields) {
                    diagnostics.push(ValidationDiagnostic {
                        code: "E1127_FIELD_RULE_READS_FIELD",
                        message: format!(
                            "Field '{symbol}.{}' reads another entity's field in '{clause}'. A field names this period's value at close, which does not exist yet inside a rule. Write `prev <entity>.<field>` for the previous period, or read it from a stream or waterfall, which see period-close values.",
                            field.name
                        ),
                        file: source_stmt.file.clone(),
                        span: slot.span,
                    });
                }
            }
            // One name, one meaning: a field cannot be both a stated fact and a
            // rule, because both bind the same path and one would silently win.
            if entity.literal_fields.iter().any(|l| l.name == field.name) {
                diagnostics.push(ValidationDiagnostic {
                    code: "E1128_FIELD_DECLARED_TWICE",
                    message: format!(
                        "Field '{symbol}.{}' is declared twice — once with '=' and once with a rule. A field is one value; state it as a fact or give it a rule, not both.",
                        field.name
                    ),
                    file: source_stmt.file.clone(),
                    span: field.span,
                });
            }
        }
    }

    let mut state_names: Vec<&str> = Vec::new();
    for source_stmt in &output.source_statements {
        let Stmt::State(state) = &source_stmt.statement else {
            continue;
        };
        if state.init.is_none() {
            diagnostics.push(ValidationDiagnostic {
                code: "E1120_STATE_MISSING_INIT",
                message: format!(
                    "State '{}' is missing required 'init'. A recurrence needs its value at period 0 stated; without it every period would evaluate to zero.",
                    state.name
                ),
                file: source_stmt.file.clone(),
                span: state.span,
            });
        }
        if state.next.is_none() {
            diagnostics.push(ValidationDiagnostic {
                code: "E1121_STATE_MISSING_NEXT",
                message: format!(
                    "State '{}' is missing required 'next'. Without it the state would hold its initial value forever, which a plain input expresses more clearly.",
                    state.name
                ),
                file: source_stmt.file.clone(),
                span: state.span,
            });
        }
        if state_names.contains(&state.name.as_str()) {
            diagnostics.push(ValidationDiagnostic {
                code: "E1122_STATE_DUPLICATE_NAME",
                message: format!("State '{}' is declared more than once.", state.name),
                file: source_stmt.file.clone(),
                span: state.span,
            });
        } else {
            state_names.push(state.name.as_str());
        }
        if let Some(init) = &state.init {
            // `state.<other>` in an `init` names a value that does not exist
            // yet. States are seeded together at period 0, so there is no order
            // in which one could already hold a value for another to read — it
            // evaluated to ZERO and said nothing, which is how an IRR-hurdle
            // model computed every vested share as nothing at all.
            //
            // The same edge `next` already rejects, one period earlier. It was
            // missed because the rule was written about same-period reads and
            // period 0 did not look like a period.
            if references_state_path(&init.src) {
                diagnostics.push(ValidationDiagnostic {
                    code: "E1126_STATE_INIT_READS_STATE",
                    message: format!(
                        "State '{}' reads another state in 'init'. Every state is seeded at period 0 together, so there is no value there to read; it would evaluate to zero. Inline the expression, or read the state from a stream or waterfall, which see period-close values.",
                        state.name
                    ),
                    file: source_stmt.file.clone(),
                    span: init.span,
                });
            }
            if references_prev(&init.src) {
                diagnostics.push(ValidationDiagnostic {
                    code: "E1123_STATE_PREV_OUTSIDE_NEXT",
                    message: format!(
                        "State '{}' uses 'prev' in 'init'. There is no period before the first, so 'init' must not depend on a previous value.",
                        state.name
                    ),
                    file: source_stmt.file.clone(),
                    span: init.span,
                });
            }
        }
        if let Some(next) = &state.next {
            if references_state_path(&next.src) {
                diagnostics.push(ValidationDiagnostic {
                    code: "E1124_STATE_SAME_PERIOD_READ",
                    message: format!(
                        "State '{}' reads 'state.<name>' inside 'next', which is the CURRENT period. Use 'prev.<name>' for another state's previous value.",
                        state.name
                    ),
                    file: source_stmt.file.clone(),
                    span: next.span,
                });
            }
        }
    }

    // Every `state.<name>` and `prev.<name>` must name a declared state.
    // Without this an undeclared reference reaches the engine, which warns and
    // substitutes zero — so a whole series silently evaluates to nothing while
    // the run still reports `status: ok`. Demonstrated before this check
    // existed; see docs/14_state_and_recurrence.md.
    for source_stmt in &output.source_statements {
        let referenced: Vec<(&str, Span)> = match &source_stmt.statement {
            Stmt::State(state) => state
                .next
                .iter()
                .flat_map(|slot| {
                    referenced_names(&slot.src, "prev.")
                        .into_iter()
                        .map(move |n| (n, slot.span))
                })
                .collect(),
            Stmt::Stream(stream) => {
                // A stream reads the CURRENT period, so `prev` has no meaning
                // there. Its env carries no `prev` map, so without this the
                // reference reaches the engine, warns, and evaluates the whole
                // stream to zero — cash silently missing, `status: ok`.
                for slot in stream.amount.iter().chain(stream.active_when.iter()) {
                    if references_prev_other_than_field(&slot.src) {
                        diagnostics.push(ValidationDiagnostic {
                            code: "E1123_STATE_PREV_OUTSIDE_NEXT",
                            message: format!(
                                "Stream '{}' uses 'prev' for a recurrence's own previous value, which means nothing outside a 'next'. A FIELD's previous value is readable here as 'prev.<entity>.<field>'; for a state's value at this period use 'state.<name>'.",
                                stream.name
                            ),
                            file: source_stmt.file.clone(),
                            span: slot.span,
                        });
                    }
                }
                stream
                    .amount
                    .iter()
                    .chain(stream.active_when.iter())
                    .flat_map(|slot| {
                        referenced_names(&slot.src, "state.")
                            .into_iter()
                            .map(move |n| (n, slot.span))
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        for (name, span) in referenced {
            if !state_names.contains(&name) {
                diagnostics.push(ValidationDiagnostic {
                    code: "E1125_STATE_UNKNOWN_REFERENCE",
                    message: format!(
                        "Reference to state '{name}', which is not declared. Declared states: {}.",
                        if state_names.is_empty() {
                            "none".to_string()
                        } else {
                            state_names.join(", ")
                        }
                    ),
                    file: source_stmt.file.clone(),
                    span,
                });
            }
        }
    }

    for source_stmt in &output.source_statements {
        match &source_stmt.statement {
            Stmt::Contract(contract) => {
                if !contract.has_term {
                    diagnostics.push(ValidationDiagnostic {
                        code: "E2001_CONTRACT_MISSING_TERM",
                        message: format!(
                            "Contract '{}' is missing required 'term'.",
                            contract.name
                        ),
                        file: source_stmt.file.clone(),
                        span: contract.span,
                    });
                }
                if !contract.has_effects {
                    diagnostics.push(ValidationDiagnostic {
                        code: "E2002_CONTRACT_MISSING_EFFECTS",
                        message: format!(
                            "Contract '{}' is missing required 'effects' block.",
                            contract.name
                        ),
                        file: source_stmt.file.clone(),
                        span: contract.span,
                    });
                }
            }
            Stmt::Stream(stream) => {
                // Cash flows are summed period by period, so a stream in a
                // different currency to the model would be added as if it were
                // the same unit — 500 USD subtracted as 500 INR. The spec
                // requires conversions to be explicit, so reject the mismatch
                // rather than produce a meaningless total.
                if let Some(declared) = stream.currency.as_deref() {
                    if !declared.eq_ignore_ascii_case(model_currency) {
                        diagnostics.push(ValidationDiagnostic {
                            code: "E2107_STREAM_CURRENCY_MISMATCH",
                            message: format!(
                                "Stream '{}' is in {} but the model reports in {}. Convert explicitly in the amount expression, or declare `model \"...\" currency {}`.",
                                stream.name, declared, model_currency, declared
                            ),
                            file: source_stmt.file.clone(),
                            span: stream.span,
                        });
                    }
                }
                if stream.amount.is_none() {
                    diagnostics.push(ValidationDiagnostic {
                        code: "E2102_STREAM_MISSING_AMOUNT",
                        message: format!("Stream '{}' is missing required 'amount'.", stream.name),
                        file: source_stmt.file.clone(),
                        span: stream.span,
                    });
                }
                if stream.schedule.is_none() {
                    diagnostics.push(ValidationDiagnostic {
                        code: "E2101_STREAM_MISSING_SCHEDULE",
                        message: format!(
                            "Stream '{}' is missing required 'schedule'.",
                            stream.name
                        ),
                        file: source_stmt.file.clone(),
                        span: stream.span,
                    });
                    continue;
                }

                let schedule = stream.schedule.as_ref().expect("checked is_some");

                // `mid` names where in its period the cash sits, and so do
                // `due` and a day rule. Two placements is not a refinement,
                // it is a contradiction, so it is rejected rather than
                // resolved by precedence.
                //
                // `net` is rejected for a different reason. Payment terms are
                // resolved on the CALENDAR — bill date, add the lag, find the
                // period the result lands in — while `mid` is a discounting
                // convention applied to whichever period the cash lands in.
                // Combining them would bill at the period end and then
                // discount as though the cash had arrived halfway through it,
                // which is not a convention anyone runs. Composing them
                // properly means billing from the midpoint and carrying the
                // lag's sub-period residual into the offset; that is a real
                // design question and it is not answered by picking one.
                if schedule.mid {
                    let clash = if schedule.due {
                        Some("`due`, which places the same cash at the start of the period")
                    } else if schedule.day_of_month.is_some() || schedule.end_of_month {
                        Some("a day rule, which places the same cash on a stated date")
                    } else if schedule.net.is_some() {
                        Some("`net` payment terms, which are resolved on the calendar rather than as a position in the period")
                    } else {
                        None
                    };
                    if let Some(clash) = clash {
                        diagnostics.push(ValidationDiagnostic {
                            code: "E2109_SCHEDULE_CONFLICTING_PLACEMENT",
                            message: format!(
                                "Stream '{}' combines `mid` with {}. A schedule states one position in its period, not two.",
                                stream.name, clash
                            ),
                            file: source_stmt.file.clone(),
                            span: schedule.span,
                        });
                    }
                }

                if let Some(day) = schedule.day_of_month {
                    if !(1..=31).contains(&day) {
                        diagnostics.push(ValidationDiagnostic {
                            code: "E2105_SCHEDULE_INVALID_DAY_OF_MONTH",
                            message: format!(
                                "Stream '{}' has invalid day-of-month {} (expected 1..31).",
                                stream.name, day
                            ),
                            file: source_stmt.file.clone(),
                            span: schedule.span,
                        });
                    }
                }

                if let (Some(from), Some(to)) = (&schedule.from, &schedule.to) {
                    if let (Some(from_date), Some(to_date)) = (parse_date(from), parse_date(to)) {
                        if from_date > to_date {
                            diagnostics.push(ValidationDiagnostic {
                                code: "E2104_SCHEDULE_INVALID_RANGE",
                                message: format!(
                                    "Stream '{}' has schedule range where 'from' is after 'to'.",
                                    stream.name
                                ),
                                file: source_stmt.file.clone(),
                                span: schedule.span,
                            });
                        }

                        if from_date < timeline.start || to_date > timeline.end {
                            diagnostics.push(ValidationDiagnostic {
                                code: "E2103_SCHEDULE_OUT_OF_BOUNDS",
                                message: format!(
                                    "Stream '{}' schedule is outside model timeline (timeline: {} to {}).",
                                    stream.name,
                                    fmt_date(timeline.start),
                                    fmt_date(timeline.end)
                                ),
                                file: source_stmt.file.clone(),
                                span: schedule.span,
                            });
                        }
                    }
                }

                match &schedule.kind {
                    ScheduleKind::PhaseEnter { phase } | ScheduleKind::EveryPhase { phase } => {
                        if !phases.contains(phase) {
                            diagnostics.push(ValidationDiagnostic {
                                code: "E2106_SCHEDULE_PHASE_NOT_FOUND",
                                message: format!(
                                    "Stream '{}' references unknown phase '{}'.",
                                    stream.name, phase
                                ),
                                file: source_stmt.file.clone(),
                                span: schedule.span,
                            });
                        }
                    }
                    ScheduleKind::OnDate => {}
                    ScheduleKind::Every => {
                        // A schedule finer than the grid cannot be
                        // represented — but not because the occurrences are
                        // lost. They ACCUMULATE: a period holds many accruals
                        // and their amounts sum, which is what makes a
                        // settlement lag work. What cannot be done is telling
                        // them apart. An accrual is stored as a model PERIOD
                        // INDEX, so occurrences inside one period share an
                        // environment, and an amount that varies with time is
                        // computed once and multiplied rather than summed
                        // across the occurrences. A constant amount is exact;
                        // anything else is silently wrong. See
                        // docs/13_feature_backlog.md 7.16.
                        if let Some(interval) = schedule.every.as_deref() {
                            if let (Some(i), Some(c)) =
                                (interval_grain(interval), cadence_grain(timeline.cadence))
                            {
                                if i < c {
                                    diagnostics.push(ValidationDiagnostic {
                                        code: "E2108_SCHEDULE_FINER_THAN_CALENDAR",
                                        message: format!(
                                            "Stream '{}' pays every {} but the model's calendar is {}. Occurrences inside one period share that period's environment and cannot be told apart, so an amount that varies over time would be computed once and multiplied. Use an interval of {} or longer, or declare a finer calendar.",
                                            stream.name,
                                            interval,
                                            cadence_name(timeline.cadence),
                                            cadence_name(timeline.cadence),
                                        ),
                                        file: source_stmt.file.clone(),
                                        span: schedule.span,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Keep deterministic output across runs and platforms.
    let _ = timeline_file;
    let _ = symbols;
    sort_diagnostics(&mut diagnostics);
    diagnostics
}

#[derive(Clone, Copy)]
struct RequirementSpec {
    missing_code: &'static str,
    missing_message: &'static str,
    multiple_code: &'static str,
    multiple_message: &'static str,
}

fn push_missing_or_multiple(
    diagnostics: &mut Vec<ValidationDiagnostic>,
    spec: RequirementSpec,
    items: &[(&str, Span)],
    anchor: (&str, Span),
) {
    match items.len() {
        0 => diagnostics.push(ValidationDiagnostic {
            code: spec.missing_code,
            message: spec.missing_message.to_string(),
            file: anchor.0.to_string(),
            span: anchor.1,
        }),
        1 => {}
        _ => diagnostics.push(ValidationDiagnostic {
            code: spec.multiple_code,
            message: spec.multiple_message.to_string(),
            file: items[1].0.to_string(),
            span: items[1].1,
        }),
    }
}

fn default_anchor(output: &ResolveOutput) -> (String, Span) {
    if let Some(first) = output.source_statements.first() {
        (first.file.clone(), statement_span(&first.statement))
    } else {
        (
            "model.cfdl".to_string(),
            Span {
                start_line: 1,
                start_col: 1,
                end_line: 1,
                end_col: 1,
            },
        )
    }
}

fn choose_timeline(output: &ResolveOutput) -> Option<(String, Timeline)> {
    for source_stmt in &output.source_statements {
        if let Stmt::Time(time) = &source_stmt.statement {
            if let Some(start) = parse_date(&time.from) {
                let cash_end = end_of_timeline(start, time.cadence, time.periods);
                let end = end_of_timeline(
                    start,
                    time.cadence,
                    time.periods.saturating_add(time.projection),
                );
                return Some((
                    source_stmt.file.clone(),
                    Timeline {
                        start,
                        end,
                        cash_end,
                        cadence: time.cadence,
                    },
                ));
            }
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Date {
    year: i32,
    month: u32,
    day: u32,
}

/// Relative coarseness, so a schedule interval can be compared against the
/// calendar cadence. Higher is coarser; a schedule must be at least as coarse
/// as the grid it is evaluated on.
fn interval_grain(interval: &str) -> Option<u8> {
    match interval {
        "day" => Some(0),
        "week" => Some(1),
        "month" => Some(2),
        "quarter" => Some(3),
        "year" => Some(4),
        _ => None,
    }
}

fn cadence_grain(cadence: Cadence) -> Option<u8> {
    match cadence {
        Cadence::Daily => Some(0),
        Cadence::Monthly => Some(2),
        Cadence::Quarterly => Some(3),
        Cadence::Annual => Some(4),
    }
}

fn cadence_name(cadence: Cadence) -> &'static str {
    match cadence {
        Cadence::Daily => "daily",
        Cadence::Monthly => "monthly",
        Cadence::Quarterly => "quarterly",
        Cadence::Annual => "annual",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Timeline {
    start: Date,
    /// Last period a schedule may legally reach — the CASH horizon plus any
    /// `project <n>` tail, because the engine evaluates streams over both.
    end: Date,
    /// Last period whose cash reaches results and NPV. Between this and `end`
    /// lies the projection tail: computed for `series_sum` lookups, excluded
    /// from cash. A flow settling there is legal but silently absent from the
    /// totals, so it warrants a warning rather than an error.
    cash_end: Date,
    cadence: Cadence,
}

fn parse_date(raw: &str) -> Option<Date> {
    let parts: Vec<&str> = raw.split('-').collect();
    match parts.as_slice() {
        [year, month] => {
            let year = year.parse::<i32>().ok()?;
            let month = month.parse::<u32>().ok()?;
            if !(1..=12).contains(&month) {
                return None;
            }
            Some(Date {
                year,
                month,
                day: 1,
            })
        }
        [year, month, day] => {
            let year = year.parse::<i32>().ok()?;
            let month = month.parse::<u32>().ok()?;
            let day = day.parse::<u32>().ok()?;
            if !(1..=12).contains(&month) {
                return None;
            }
            if day == 0 || day > days_in_month(year, month) {
                return None;
            }
            Some(Date { year, month, day })
        }
        _ => None,
    }
}

fn end_of_timeline(start: Date, cadence: Cadence, periods: u32) -> Date {
    if periods == 0 {
        return start;
    }
    match cadence {
        Cadence::Daily => add_days(start, periods.saturating_sub(1) as i32),
        Cadence::Monthly => add_months(start, periods.saturating_sub(1) as i32),
        Cadence::Quarterly => add_months(start, periods.saturating_sub(1) as i32 * 3),
        Cadence::Annual => add_months(start, periods.saturating_sub(1) as i32 * 12),
    }
}

fn add_months(date: Date, months: i32) -> Date {
    let total = (date.year * 12 + (date.month as i32 - 1)) + months;
    let year = total.div_euclid(12);
    let month = (total.rem_euclid(12) + 1) as u32;
    let max_day = days_in_month(year, month);
    Date {
        year,
        month,
        day: date.day.min(max_day),
    }
}

fn add_days(mut date: Date, mut days: i32) -> Date {
    while days > 0 {
        let dim = days_in_month(date.year, date.month);
        if date.day < dim {
            date.day += 1;
        } else if date.month == 12 {
            date.year += 1;
            date.month = 1;
            date.day = 1;
        } else {
            date.month += 1;
            date.day = 1;
        }
        days -= 1;
    }
    date
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 31,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn fmt_date(date: Date) -> String {
    format!("{:04}-{:02}-{:02}", date.year, date.month, date.day)
}

/// Whether an expression source mentions `prev`, bare or namespaced.
///
/// Source-text matching rather than an AST walk, deliberately: validation runs
/// before expressions are compiled, and every other check at this layer works
/// the same way. Word-bounded so `prev_year` and `inputs.prevailing` do not
/// match.
fn references_prev(src: &str) -> bool {
    mentions_word(src, "prev")
}

/// `prev` used for anything OTHER than a field's previous value.
///
/// A stream may read `prev.<family>.<entity>.<field>` — the close before this
/// one, which the engine has as a column and which a debt schedule needs for
/// average-balance interest. It may not read bare `prev` or `prev.<state>`,
/// which name a recurrence's own previous value and mean nothing outside one.
fn references_prev_other_than_field(src: &str) -> bool {
    let mut rest = src;
    while let Some(idx) = rest.find("prev") {
        let (before, from) = rest.split_at(idx);
        let boundary_ok = before
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != '.');
        let tail = &from[4..];
        if boundary_ok && !tail.starts_with('.') {
            return true;
        }
        if boundary_ok
            && !["asset.", "party.", "contract.", "reference."]
                .iter()
                .any(|family| tail[1..].starts_with(family))
        {
            return true;
        }
        rest = &from[4..];
    }
    false
}

/// Does this expression read a field that MOVES — one carrying a rule?
///
/// A literal field is a constant and may be read from anywhere. Only a
/// rule-bearing field has a period-close value that does not exist yet inside
/// another rule, which is what §4a of docs/18 settled.
fn reads_moving_field(src: &str, rule_fields: &BTreeSet<String>) -> bool {
    for family in ["asset", "party", "contract", "reference"] {
        let needle = format!("{family}.");
        let mut base = 0usize;
        while let Some(idx) = src[base..].find(&needle) {
            let at = base + idx;
            let before_ok = at == 0
                || !src[..at]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
            base = at + needle.len();
            if !before_ok {
                continue;
            }
            let tail = &src[base..];
            let end = tail
                .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))
                .unwrap_or(tail.len());
            let segs: Vec<&str> = tail[..end].split('.').collect();
            if segs.len() >= 2 && rule_fields.contains(&format!("{family}.{}.{}", segs[0], segs[1]))
            {
                return true;
            }
        }
    }
    false
}

/// Whether an expression source reads `state.<name>` — the CURRENT period.
fn references_state_path(src: &str) -> bool {
    src.match_indices("state.").any(|(idx, _)| {
        let before_ok = idx == 0
            || !src[..idx]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
        before_ok
    })
}

/// The `<name>` of every `<prefix><name>` in an expression source.
fn referenced_names<'a>(src: &'a str, prefix: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    for (idx, _) in src.match_indices(prefix) {
        let preceded_by_word = src[..idx]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.');
        if preceded_by_word {
            continue;
        }
        let rest = &src[idx + prefix.len()..];
        let end = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        if end > 0 {
            out.push(&rest[..end]);
        }
    }
    out
}

fn mentions_word(src: &str, word: &str) -> bool {
    src.match_indices(word).any(|(idx, _)| {
        let before = src[..idx].chars().next_back();
        let after = src[idx + word.len()..].chars().next();
        // A root, so `inputs.prev` is a different name entirely — the dot
        // has to disqualify the match as much as an alphanumeric would.
        let starts = before.is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '.'));
        // `prev.foo` is still a `prev` mention; `prev_year` is not.
        let ends = after.is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
        starts && ends
    })
}

fn statement_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Version(s) => s.span,
        Stmt::Model(s) => s.span,
        Stmt::UsePack(s) => s.span,
        Stmt::Import(s) => s.span,
        Stmt::Time(s) => s.span,
        Stmt::Phase(s) => s.span,
        Stmt::Entity(s) => s.span,
        Stmt::Assume(s) => s.span,
        Stmt::Curve(s) => s.span,
        Stmt::State(s) => s.span,
        Stmt::Contract(s) => s.span,
        Stmt::Stream(s) => s.span,
        Stmt::Event(s) => s.span,
        Stmt::Option(s) => s.span,
        Stmt::Waterfall(s) => s.span,
        Stmt::Run(s) => s.span,
    }
}

fn sort_diagnostics(diagnostics: &mut [ValidationDiagnostic]) {
    diagnostics.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.span.start_line.cmp(&b.span.start_line))
            .then(a.span.start_col.cmp(&b.span.start_col))
            .then(a.code.cmp(b.code))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full pipeline for a single-file model: lex -> parse -> resolve -> validate.
    fn diagnostics_for(src: &str) -> Vec<String> {
        let (tokens, lex_diags) = cfdl_lexer::lex(src);
        assert!(lex_diags.is_empty(), "lex diags: {lex_diags:?}");
        let parse_result = cfdl_parser::parse("model.cfdl", src, &tokens);
        assert!(
            parse_result.diagnostics.is_empty(),
            "parse diags: {:?}",
            parse_result.diagnostics
        );
        let root_module = cfdl_resolver::RootModule {
            relative_path: "model.cfdl".to_string(),
            full_path: std::path::PathBuf::from("model.cfdl"),
            ast: parse_result.ast.expect("ast"),
        };
        let output = cfdl_resolver::resolve_imports(std::path::Path::new("."), root_module)
            .expect("resolve imports");
        let symbols = cfdl_resolver::resolve_symbols(&output).expect("resolve symbols");
        validate(&output, &symbols)
            .into_iter()
            .map(|d| d.code.to_string())
            .collect()
    }

    const VALID: &str = "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 12\nentity legal borrower\nstream legal.rent on entity legal.borrower {\n  schedule every month from 2026-01 to 2026-12\n  amount = 1000\n}\n";

    #[test]
    fn clean_model_has_no_diagnostics() {
        assert!(diagnostics_for(VALID).is_empty());
    }

    #[test]
    fn missing_version_model_time() {
        let codes = diagnostics_for("entity legal borrower\n");
        assert!(
            codes.contains(&"E1101_MISSING_VERSION".to_string()),
            "{codes:?}"
        );
        assert!(codes.contains(&"E1102_MISSING_MODEL".to_string()));
        assert!(codes.contains(&"E1103_MISSING_TIME".to_string()));
    }

    #[test]
    fn multiple_version_and_model() {
        let codes = diagnostics_for(
            "version 0.1\nversion 0.1\nmodel \"a\"\nmodel \"b\"\ntime calendar monthly from 2026-01 for 2\nentity legal borrower\n",
        );
        assert!(
            codes.contains(&"E1104_MULTIPLE_VERSION".to_string()),
            "{codes:?}"
        );
        assert!(codes.contains(&"E1105_MULTIPLE_MODEL".to_string()));
    }

    #[test]
    fn missing_entity() {
        let codes =
            diagnostics_for("version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 2\n");
        assert!(
            codes.contains(&"E1109_MISSING_ENTITY".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn stream_missing_amount_and_schedule() {
        let codes = diagnostics_for(
            "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 2\nentity legal borrower\nstream legal.rent on entity legal.borrower\n",
        );
        assert!(
            codes.contains(&"E2101_STREAM_MISSING_SCHEDULE".to_string()),
            "{codes:?}"
        );
        assert!(codes.contains(&"E2102_STREAM_MISSING_AMOUNT".to_string()));
    }

    #[test]
    fn schedule_range_out_of_bounds() {
        let codes = diagnostics_for(
            "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 2\nentity legal borrower\nstream legal.rent on entity legal.borrower {\n  schedule every month from 2026-01 to 2030-12\n  amount = 1\n}\n",
        );
        assert!(
            codes.contains(&"E2103_SCHEDULE_OUT_OF_BOUNDS".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn schedule_inverted_range() {
        let codes = diagnostics_for(
            "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 12\nentity legal borrower\nstream legal.rent on entity legal.borrower {\n  schedule every month from 2026-06 to 2026-01\n  amount = 1\n}\n",
        );
        assert!(
            codes.contains(&"E2104_SCHEDULE_INVALID_RANGE".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn schedule_invalid_day_of_month() {
        let codes = diagnostics_for(
            "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 12\nentity legal borrower\nstream legal.rent on entity legal.borrower {\n  schedule every month on day 42 from 2026-01 to 2026-12\n  amount = 1\n}\n",
        );
        assert!(
            codes.contains(&"E2105_SCHEDULE_INVALID_DAY_OF_MONTH".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn schedule_unknown_phase() {
        let codes = diagnostics_for(
            "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 12\nentity legal borrower\nstream legal.rent on entity legal.borrower {\n  schedule on phase_enter(\"nope\")\n  amount = 1\n}\n",
        );
        assert!(
            codes.contains(&"E2106_SCHEDULE_PHASE_NOT_FOUND".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn contract_missing_term() {
        let codes = diagnostics_for(
            "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 12\nentity legal borrower\ncontract cre.lease on entity legal.borrower {\n}\n",
        );
        assert!(
            codes.contains(&"E2001_CONTRACT_MISSING_TERM".to_string()),
            "{codes:?}"
        );
    }
}
