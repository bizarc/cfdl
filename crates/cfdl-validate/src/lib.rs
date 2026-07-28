//! CFDL validation pass (Milestone 5).

use cfdl_parser::{Cadence, ScheduleKind, Span, Stmt};
use cfdl_resolver::{ResolveOutput, SymbolTables};

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
                    ScheduleKind::OnDate | ScheduleKind::Every => {}
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
                let end = end_of_timeline(start, time.cadence, time.periods);
                return Some((source_stmt.file.clone(), Timeline { start, end }));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Timeline {
    start: Date,
    end: Date,
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
        Stmt::Contract(s) => s.span,
        Stmt::Stream(s) => s.span,
        Stmt::Event(s) => s.span,
        Stmt::Option(s) => s.span,
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
