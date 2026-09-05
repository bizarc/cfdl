// The account plane: balances rolled by the engine from the streams that move
// them (`docs/42`).
//
// An account's closing is its opening plus its declared inflow, plus what the
// period's streams moved (signed by the account's side: an inflow raises a
// liability its owner owes and lowers a receivable its owner is due; an
// accrual raises and a write-off lowers either), plus what a machine set it
// to, plus the waterfall's allocations. Its opening is the prior close — the
// `init` in the first period — and a container's account is the sum of its
// members' through `part of`. This module holds the openings, the movements,
// the fold, and the declared inflow; the stage applies them period by period.
use super::*;

/// Which declared accounts each fold sums, by the fold's full name.
pub(crate) type FoldMembers = BTreeMap<String, Vec<String>>;

/// Which declared accounts each fold sums (`docs/42` §3.4): every account
/// of the same short name on a descendant of the fold's owner, through
/// `part of`. Folds of folds are not summed — the leaves are, once.
pub(crate) fn fold_members(ir: &Ir) -> FoldMembers {
    let parent_of: BTreeMap<&str, &str> = ir
        .entities
        .iter()
        .filter_map(|e| e.parent.as_deref().map(|p| (e.symbol.as_str(), p)))
        .collect();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for fold in ir.accounts.iter().filter(|a| a.fold) {
        let Some((owner, name)) = fold.name.rsplit_once('.') else {
            continue;
        };
        for account in ir.accounts.iter().filter(|a| !a.fold) {
            let Some((member, member_name)) = account.name.rsplit_once('.') else {
                continue;
            };
            if member_name != name {
                continue;
            }
            let mut cursor = parent_of.get(member).copied();
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            while let Some(ancestor) = cursor {
                if !seen.insert(ancestor) {
                    break;
                }
                if ancestor == owner {
                    out.entry(fold.name.clone())
                        .or_default()
                        .push(account.name.clone());
                    break;
                }
                cursor = parent_of.get(ancestor).copied();
            }
        }
    }
    out
}

/// Settle every fold at `t` as the sum of its members' balances at `t`.
pub(crate) fn settle_folds(
    members: &BTreeMap<String, Vec<String>>,
    balances: &mut BTreeMap<String, Vec<f64>>,
    t: usize,
) {
    for (fold, of) in members {
        let sum: f64 = of
            .iter()
            .filter_map(|m| balances.get(m).and_then(|c| c.get(t)).copied())
            .sum();
        if let Some(column) = balances.get_mut(fold) {
            column[t] = sum;
        }
    }
}

/// What period `t`'s streams moved, per account: (stream, delta), signed
/// for the account's side (`docs/42` §3.2). A cash stream's signed amount
/// raises a liability its owner owes and lowers a receivable its owner is
/// due; an accrual raises and a write-off lowers, whichever side.
pub(crate) fn account_moves_at(
    ir: &Ir,
    columns: &BTreeMap<String, Vec<f64>>,
    account_side: &BTreeMap<&str, &str>,
    t: usize,
) -> BTreeMap<String, Vec<(String, f64)>> {
    let mut moves: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();
    for stream in &ir.streams {
        let Some(account) = stream.moves.as_deref() else {
            continue;
        };
        let Some(value) = columns.get(&stream.name).and_then(|c| c.get(t)).copied() else {
            continue;
        };
        let delta = if streams::is_cash(stream) {
            match account_side.get(account).copied() {
                Some("due") => -value,
                _ => value,
            }
        } else {
            value
        };
        moves
            .entry(account.to_string())
            .or_default()
            .push((format!("stream:{}", stream.name), delta));
    }
    moves
}

/// An account's balance, period by period.
///
/// `balance(t) = balance(t-1) + inflow(t)`, and draws are subtracted as a
/// waterfall takes them — which is why this is computed INSIDE the walk rather
/// than as a post-pass: the balance at `t` is what a distribution at `t` may
/// draw on, and what logic at `t + 1` may read.
///
/// A NEGATIVE INFLOW LOWERS THE BALANCE, with no floor. The language models
/// returns, and an account fed a deal's whole net cash IS the deal's
/// cumulative position — negative through the J-curve and positive after. What
/// is floored is the DRAW: cash that is not there cannot be allocated.
#[allow(clippy::too_many_arguments)] // stage inputs, as elsewhere in this file
pub(crate) fn account_inflow_at(
    ir: &Ir,
    config: &RunConfig,
    account: &IrAccount,
    compiled: Option<&cfdl_expr::CompiledExpr>,
    t: usize,
    date: &Date,
    base_inputs: &BTreeMap<String, f64>,
    series: Option<&Arc<BTreeMap<String, Vec<f64>>>>,
    warnings: &mut Vec<String>,
) -> f64 {
    let Some(compiled) = compiled else {
        return 0.0;
    };
    let mut env = build_expr_env(ir, None, config, t, date, base_inputs);
    if let Some(series) = series {
        env.series = Arc::clone(series);
    }
    // An account's inflow reads cash that has settled this period, the way a
    // waterfall's pot does — it is the period's cash arriving, not logic
    // deciding on it.
    env.series_available_to = Some(t);
    match cfdl_expr::eval(compiled, &env) {
        Ok(ExprValue::Decimal(v)) => v,
        Ok(ExprValue::Int(v)) => v as f64,
        Ok(other) => {
            warnings.push(format!(
                "Account '{}' inflow evaluated to {other:?}, which is not a number; using 0.",
                account.name
            ));
            0.0
        }
        Err(err) => {
            warnings.push(format!(
                "Account '{}' inflow failed [{}]: {}; using 0.",
                account.name, err.code, err.message
            ));
            0.0
        }
    }
}

/// Each account's opening in the first period — its `init`, evaluated once
/// against the run's inputs (`docs/42` §7); absent means zero — with every
/// fold's opening as its members' openings summed. Returns the openings and
/// the fold membership the walk settles each period.
pub(crate) fn initial_balances(
    ir: &Ir,
    config: &RunConfig,
    prep: &ModelPrep<'_>,
    timeline: &[Date],
    base_inputs: &BTreeMap<String, f64>,
    warnings: &mut Vec<String>,
) -> (Arc<BTreeMap<String, f64>>, FoldMembers) {
    // EACH ACCOUNT'S `init`: the balance at the timeline's first period,
    // evaluated once against the run's inputs (`docs/42` §7). Absent means
    // zero — a balance created during the run is raised by the cash that
    // creates it.
    let openings: Arc<BTreeMap<String, f64>> = Arc::new(
        ir.accounts
            .iter()
            .enumerate()
            .map(|(idx, account)| {
                let value = prep
                    .account_inits
                    .get(idx)
                    .and_then(|c| c.as_ref())
                    .map(|compiled| {
                        let env = build_expr_env(ir, None, config, 0, &timeline[0], base_inputs);
                        match cfdl_expr::eval(compiled, &env) {
                            Ok(ExprValue::Decimal(v)) => v,
                            Ok(ExprValue::Int(v)) => v as f64,
                            Ok(other) => {
                                warnings.push(format!(
                                    "Account '{}' init evaluated to {other:?}, which is not a number; opens at zero.",
                                    account.name
                                ));
                                0.0
                            }
                            Err(err) => {
                                warnings.push(format!(
                                    "Account '{}' init failed to evaluate [{}]: {}; opens at zero.",
                                    account.name, err.code, err.message
                                ));
                                0.0
                            }
                        }
                    })
                    .unwrap_or(0.0);
                (account.name.clone(), value)
            })
            .collect(),
    );
    // A fold's opening is its members' openings summed, in the first period
    // as after it.
    let members = fold_members(ir);
    let openings: Arc<BTreeMap<String, f64>> = {
        let mut inits = (*openings).clone();
        for (fold, of) in &members {
            let sum: f64 = of.iter().filter_map(|m| inits.get(m).copied()).sum();
            inits.insert(fold.clone(), sum);
        }
        Arc::new(inits)
    };
    (openings, members)
}

/// Each account's side, where it declares one: `owed` or `due` from its
/// owner's view (`docs/42` §3.6), which decides which way a cash movement
/// changes it.
pub(crate) fn account_sides(ir: &Ir) -> BTreeMap<&str, &str> {
    ir.accounts
        .iter()
        .filter_map(|a| a.side.as_deref().map(|side| (a.name.as_str(), side)))
        .collect()
}
