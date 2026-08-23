//! Generic evaluator for pack-declared domain validations.
//!
//! Packs own *what* to check — which term, which bound, which stable code —
//! declared in `packs/<pack>/validations.toml`. The compiler owns the parts a
//! pack cannot see or safely control: source spans, the model timeline, and
//! diagnostic emission. This module is the seam between the two.
//!
//! The check kinds are a closed set with no expressions, recursion, or
//! message interpolation, so evaluating a pack's validations is bounded work
//! that cannot crash or hang the compiler.

use cfdl_pack::{
    CompareOp, NumberKind, OnInvalid, PackValidation, ValidationCheck, ValidationSeverity,
    WhenPresence,
};

use crate::Diagnostic;

/// Outcome of reading a numeric term.
enum TermValue {
    Absent,
    Unparseable,
    /// The term defers to a declared input, so its value is not known until
    /// the run supplies one. Bounds cannot be checked here — the value may
    /// differ per scenario and per Monte Carlo trial.
    Deferred,
    /// The term is an expression, evaluated per period at run time. The same
    /// tier as `Deferred`: present, well-formed (E5025 checks that it
    /// compiles), value unknowable here.
    Expression,
    Number(f64),
}

fn read_number(contract: &cfdl_parser::ContractStmt, term: &str, kind: NumberKind) -> TermValue {
    let Some(entry) = contract.terms.get(term) else {
        return TermValue::Absent;
    };
    if entry.is_input_ref() {
        return TermValue::Deferred;
    }
    if entry.kind == cfdl_parser::TermValueKind::Expr {
        return TermValue::Expression;
    }
    match kind {
        // Integer terms parse as i32 so `18.5` is rejected rather than
        // silently truncated.
        NumberKind::Integer => match entry.value.parse::<i32>() {
            Ok(value) => TermValue::Number(f64::from(value)),
            Err(_) => TermValue::Unparseable,
        },
        NumberKind::Decimal => match entry.value.parse::<f64>() {
            Ok(value) => TermValue::Number(value),
            Err(_) => TermValue::Unparseable,
        },
    }
}

fn bounds_violated(validation: &PackValidation, value: f64) -> bool {
    validation.min.is_some_and(|min| value < min)
        || validation.max.is_some_and(|max| value > max)
        || validation.exclusive_min.is_some_and(|min| value <= min)
        || validation.exclusive_max.is_some_and(|max| value >= max)
}

/// Whether a `term_number` validation should fire.
fn term_number_fires(validation: &PackValidation, contract: &cfdl_parser::ContractStmt) -> bool {
    let Some(term) = validation.term.as_deref() else {
        return false;
    };
    match read_number(contract, term, validation.number) {
        // An absent term fails only when the check runs unconditionally —
        // `when = "present"` means "validate it if it's there".
        TermValue::Absent => validation.when == WhenPresence::Always,
        // A present-but-unparseable value belongs either to this check or to
        // a sibling that owns the parse failure (the `if / else if` pairs).
        TermValue::Unparseable => {
            validation.when != WhenPresence::Present || validation.on_invalid == OnInvalid::Report
        }
        // An input-referenced or expression term is present and
        // well-formed; its value is simply not knowable yet.
        TermValue::Deferred | TermValue::Expression => false,
        TermValue::Number(value) => bounds_violated(validation, value),
    }
}

fn term_enum_fires(validation: &PackValidation, contract: &cfdl_parser::ContractStmt) -> bool {
    let Some(term) = validation.term.as_deref() else {
        return false;
    };
    let Some(entry) = contract.terms.get(term) else {
        return validation.when == WhenPresence::Always;
    };
    !validation
        .values
        .iter()
        .any(|allowed| allowed.matches(&entry.value))
}

fn term_compare_fires(validation: &PackValidation, contract: &cfdl_parser::ContractStmt) -> bool {
    let (Some(left), Some(right), Some(op)) = (
        validation.left.as_deref(),
        validation.right.as_deref(),
        validation.op,
    ) else {
        return false;
    };
    let (TermValue::Number(lhs), TermValue::Number(rhs)) = (
        read_number(contract, left, validation.number),
        read_number(contract, right, validation.number),
    ) else {
        // Comparison needs both operands; missing ones are another check's job.
        return false;
    };
    let satisfied = match op {
        CompareOp::Le => lhs <= rhs,
        CompareOp::Lt => lhs < rhs,
        CompareOp::Ge => lhs >= rhs,
        CompareOp::Gt => lhs > rhs,
    };
    !satisfied
}

/// Evaluates a pack's validations against one contract.
///
/// `term_range_ok` is supplied by the compiler because only it can resolve the
/// model timeline.
pub(crate) fn evaluate(
    validations: &[PackValidation],
    contract: &cfdl_parser::ContractStmt,
    term_range_ok: bool,
    mut emit: impl FnMut(&str, &str, ValidationSeverity, cfdl_parser::Span) -> Diagnostic,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for validation in validations {
        if !validation.applies_to(contract.name.as_str()) {
            continue;
        }

        let fires = match validation.check {
            ValidationCheck::TermPresent => validation
                .term
                .as_deref()
                .is_some_and(|term| !contract.terms.contains_key(term)),
            ValidationCheck::AnyTermPresent => !validation
                .terms
                .iter()
                .any(|term| contract.terms.contains_key(term.as_str())),
            ValidationCheck::TermsMutuallyExclusive => {
                validation
                    .terms
                    .iter()
                    .filter(|term| contract.terms.contains_key(term.as_str()))
                    .count()
                    > 1
            }
            ValidationCheck::TermNumber => term_number_fires(validation, contract),
            ValidationCheck::TermRangeWithinTimeline => !term_range_ok,
            ValidationCheck::TermEnum => term_enum_fires(validation, contract),
            ValidationCheck::TermCompare => term_compare_fires(validation, contract),
        };

        if fires {
            // Point at the offending term when there is one and it is present;
            // a missing term has no span of its own, so those fall back to the
            // contract.
            let span = validation
                .term
                .as_deref()
                .or(validation.left.as_deref())
                .and_then(|term| contract.terms.get(term))
                .map(|entry| entry.span)
                .unwrap_or(contract.span);

            diagnostics.push(emit(
                &validation.code,
                &validation.message,
                validation.severity,
                span,
            ));
        }
    }

    diagnostics
}
