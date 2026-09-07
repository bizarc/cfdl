// The prepare stage: everything about a model that does not vary from one
// run to the next, built once and replayed per scenario and per trial.
//
// The grid (the timeline), the drivers as the run resolves them, each
// stream's dependency graph and evaluation wave, the priced closure (what a
// valuation reads forward), whether the model can be walked at all, the
// compiled plans, and the accounts' compiled openings. `prepare_model` is the
// entry; the public `priced_streams` and `walk_eligibility` in `lib.rs` read
// the same graph for tooling.
use super::*;

/// A series read where no stream value exists — the engine's backstop for the
/// compiler's `E1134_SERIES_READ_IN_LOGIC`.
///
/// The compiler refuses this on every model it sees, which is every model
/// written in CFDL. The engine also accepts IR directly — the WASM, server and
/// Python paths all do — and there the compiler's check has not run. Without
/// this the engine warns once per period and substitutes `false` or `0`,
/// publishing a full set of numbers under `status: ok`: a guard that never
/// fires, or a recurrence whose collapse `prev` carries for the rest of the
/// run (`docs/13` §7.71).
///
/// `docs/28` §4 is where this becomes an ordering rule rather than a
/// prohibition: under the period walk a guard may read a stream's settled
/// history, at or before the previous period. Same-period and forward reads
/// stay refused, so this narrows rather than disappears.
/// AN ACTION KIND THE ENGINE DOES NOT EXECUTE IS REFUSED, not journaled as
/// `ignored` under a run that reports ok (`docs/13` §7.83). Only hand-written
/// IR can carry one — every kind a model can write is known to the compiler —
/// so the IR asked for something this engine cannot do, and the answer is no.
pub(crate) fn refuse_unknown_action_kinds(ir: &Ir) -> Result<(), EngineError> {
    const KNOWN: [&str; 4] = [
        "SetEntityField",
        "ActivateStream",
        "DeactivateStream",
        "ExerciseOption",
    ];
    let mut offences: Vec<String> = Vec::new();
    let hosts = ir
        .events
        .iter()
        .map(|e| (format!("event '{}'", e.name), &e.actions))
        .chain(
            ir.options
                .iter()
                .map(|o| (format!("option '{}'", o.name), &o.actions)),
        );
    for (host, actions) in hosts {
        for action in actions {
            if !KNOWN.contains(&action.kind.as_str()) {
                offences.push(format!("{host} carries action kind '{}'", action.kind));
            }
        }
    }
    if offences.is_empty() {
        return Ok(());
    }
    Err(EngineError::UnknownActionKind(format!(
        "{}; this engine executes {}. The IR asks for something it cannot do, so the run is \
         refused rather than reported as ok with the action ignored.",
        offences.join("; "),
        KNOWN.join(", ")
    )))
}

/// A CURVE READ PAST ITS LAST POINT HOLDS ITS LAST VALUE when the curve
/// declares no end — right for a rate deck, wrong for a schedule declared as
/// a curve, and silent for both (`docs/13` §7.100). Warned once per (reader,
/// curve) when a stream's or a field's periods run past the curve's last
/// point, naming that date and the last date read. A curve that declares
/// `to <date>` has answered the question and is not warned about: inside its
/// dates the hold is stated, outside them the run refuses.
pub(crate) fn warn_curve_reads_past_end(
    ir: &Ir,
    plans: &[StreamPlan<'_>],
    timeline: &[Date],
    warnings: &mut Vec<String>,
) {
    let Some(horizon) = timeline.last() else {
        return;
    };
    let last_point: BTreeMap<&str, Date> = ir
        .curves
        .iter()
        .filter(|c| c.effective_to.is_none())
        .filter_map(|c| {
            let last = c.points.last()?;
            Date::parse(&last.date).ok().map(|d| (c.name.as_str(), d))
        })
        .collect();
    if last_point.is_empty() {
        return;
    }
    let mut readers: Vec<(String, Date, Vec<String>)> = Vec::new();
    for plan in plans {
        let last_read = plan
            .last_settled_period()
            .and_then(|t| timeline.get(t).cloned())
            .unwrap_or_else(|| horizon.clone());
        let mut names = cfdl_expr::curve_references(plan.amount_src());
        if let Some(src) = plan.active_when_src() {
            names.extend(cfdl_expr::curve_references(src));
        }
        readers.push((format!("stream '{}'", plan.stream_name()), last_read, names));
    }
    for entity in &ir.entities {
        for (field, rule) in &entity.rules {
            let mut names = cfdl_expr::curve_references(&rule.init.src);
            names.extend(cfdl_expr::curve_references(&rule.next.src));
            readers.push((
                format!("field '{}.{}'", entity.symbol, field),
                horizon.clone(),
                names,
            ));
        }
    }
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for (reader, last_read, names) in readers {
        for name in names {
            let Some(end) = last_point.get(name.as_str()) else {
                continue;
            };
            if last_read <= *end || !seen.insert((reader.clone(), name.clone())) {
                continue;
            }
            warnings.push(format!(
                "W5024_CURVE_READ_PAST_END: {reader} reads curve '{name}' through {last_read}, \
                 past its last point at {end}; the curve holds its last value there. Right for \
                 a rate deck, wrong for a schedule — declare the curve's effective dates \
                 (`curve {name} to <date>`) to state where it stops."
            ));
        }
    }
}

/// AN OVERRIDE THAT MATCHES NOTHING IS REFUSED (`docs/13` §7.51, §7.116).
/// A run configuration addresses what it moves by a prefixed key: `inputs.`
/// an assumption, `stream.<name>:amount` one stream's figure, `cfg.` and
/// `obs.` their namespaces. A key that resolves to nothing used to be applied
/// to nothing, and the run reported ok — a scenario setting `cpr` for
/// `inputs.cpr` was silently the base run again, and a Monte Carlo keyed the
/// same way a distribution with zero variance. This is the unresolved-name
/// rule one layer out: the run knows every source, so it refuses, naming the
/// block the key sits in and the nearest name it could have meant.
/// A VALUE THE RUN SUPPLIED IS CHECKED WHERE IT ARRIVES (`docs/13` §7.56).
///
/// The compiler checks what the text carries: a literal assumption against
/// its type's domain and its `within`, a literal term against the pack's
/// validations. What the text does not carry — an override, a scenario
/// value, a Monte Carlo draw, a `cfg.` path — it cannot check, and until this
/// the pack's bound meant less than it appeared to: a scenario could set
/// `psa_speed` to 50 against a stated 0 to 10 and the run reported ok.
///
/// So the same three bounds are applied here to the resolved values:
/// - each assumption's TYPE domain (`Fraction` is 0 to 1, `Duration` is
///   whole and non-negative), and its `within [lo, hi]`;
/// - each deferred contract term's pack bound, carried in the IR as
///   `term_bounds` with the validation's own code.
///
/// Refused, never adjusted. A Monte Carlo trial that draws outside a bound
/// refuses the way any run failure does; a distribution's `clip` is the
/// tool for keeping draws inside one, and the compiler already refuses a
/// clip that reaches past the bound.
pub(crate) fn refuse_out_of_bounds_inputs(
    ir: &Ir,
    config: &RunConfig,
    base_inputs: &BTreeMap<String, f64>,
    warnings: &mut Vec<String>,
) -> Result<(), EngineError> {
    // The resolved input map: the model's own values, then the run's.
    let mut inputs: BTreeMap<&str, f64> =
        base_inputs.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    for (key, value) in &config.parameter_overrides {
        if let Some(name) = key.strip_prefix("inputs.") {
            inputs.insert(name, *value);
        }
    }
    let mut failures: Vec<String> = Vec::new();

    let mut check_assumption = |name: &str, type_id: Option<&str>, within: Option<[f64; 2]>| {
        let Some(value) = inputs.get(name).copied() else {
            return;
        };
        let type_id = type_id.unwrap_or("Decimal");
        if let Some(reason) = type_violation(value, type_id, within) {
            let supplied = if config
                .parameter_overrides
                .contains_key(&format!("inputs.{name}"))
            {
                "the run supplied"
            } else {
                "the model evaluated"
            };
            failures.push(format!(
                "`inputs.{name}` is {value} ({supplied} it), {reason}"
            ));
        }
    };
    for (name, constant) in &ir.assumptions.constants {
        check_assumption(name, constant.type_id.as_deref(), constant.within);
    }
    for (name, random) in &ir.assumptions.random {
        check_assumption(name, random.type_id.as_deref(), random.within);
    }

    for contract in &ir.contracts {
        for (term, bound) in &contract.term_bounds {
            // The value the term reads, from whichever channel it names.
            let value = if let Some(name) = bound.reads.strip_prefix("inputs.") {
                inputs.get(name).copied()
            } else if bound.reads.starts_with("cfg.") {
                config.parameter_overrides.get(&bound.reads).copied()
            } else {
                None
            };
            let Some(value) = value else {
                continue;
            };
            let below = bound.min.is_some_and(|lo| value < lo)
                || bound.exclusive_min.is_some_and(|lo| value <= lo);
            let above = bound.max.is_some_and(|hi| value > hi)
                || bound.exclusive_max.is_some_and(|hi| value >= hi);
            if below || above {
                let lo = bound
                    .min
                    .map(|v| format!("{v} or more"))
                    .or_else(|| bound.exclusive_min.map(|v| format!("more than {v}")));
                let hi = bound
                    .max
                    .map(|v| format!("{v} or less"))
                    .or_else(|| bound.exclusive_max.map(|v| format!("less than {v}")));
                let range = match (lo, hi) {
                    (Some(lo), Some(hi)) => format!("{lo} and {hi}"),
                    (Some(lo), None) => lo,
                    (None, Some(hi)) => hi,
                    (None, None) => "in range".to_string(),
                };
                // A CONVENTION QUESTIONS, A DEFINITION REFUSES: a
                // warning-severity bound is the pack saying "almost certainly
                // not meant", and the model that means it proceeds under the
                // warning, which it can allowlist.
                if bound.severity.as_deref() == Some("warning") {
                    warnings.push(format!(
                        "{}: contract `{}` term `{term}` reads `{}` = {value}, outside the {range} the pack expects — state it if the value is meant",
                        bound.code, contract.name, bound.reads
                    ));
                } else {
                    failures.push(format!(
                        "contract `{}` term `{term}` reads `{}` = {value}, and the pack requires {range} ({})",
                        contract.name, bound.reads, bound.code
                    ));
                }
            }
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(EngineError::InputOutOfBounds(format!(
            "{} — a value outside its bound is refused, never adjusted; state one inside it, or widen the bound where the deal genuinely differs",
            failures.join("; ")
        )))
    }
}

/// Why `value` cannot be an input of this type under this bound, or `None`.
/// The same rule the compiler applies to a literal (`bound_violation` there);
/// stated twice because the crates share no code, and kept identical.
fn type_violation(value: f64, type_id: &str, within: Option<[f64; 2]>) -> Option<String> {
    if matches!(type_id, "Int" | "Duration") && value.fract() != 0.0 {
        return Some(format!(
            "not a whole number as a {} must be",
            type_id.to_lowercase()
        ));
    }
    let domain = match type_id {
        "Fraction" => Some((0.0, 1.0)),
        "Duration" => Some((0.0, f64::INFINITY)),
        _ => None,
    };
    if let Some((lo, hi)) = domain {
        if value < lo || value > hi {
            let range = if hi.is_infinite() {
                format!("{lo} or more")
            } else {
                format!("{lo} to {hi}")
            };
            return Some(format!(
                "outside the domain of a {} ({range})",
                type_id.to_lowercase()
            ));
        }
    }
    if let Some([lo, hi]) = within {
        if value < lo || value > hi {
            return Some(format!("outside the within [{lo}, {hi}] the model states"));
        }
    }
    None
}

pub(crate) fn refuse_unresolved_overrides(ir: &Ir, config: &RunConfig) -> Result<(), EngineError> {
    // What resolves. An assumption is declared by `assume`, or read as
    // `inputs.<name>` somewhere in the model — an input may be supplied
    // entirely by the run configuration. `cfg.` and `obs.` paths resolve where
    // some expression reads them. A stream resolves by its name.
    let mut inputs: BTreeSet<String> = BTreeSet::new();
    inputs.extend(ir.assumptions.constants.keys().cloned());
    inputs.extend(ir.assumptions.random.keys().cloned());
    let mut cfg_paths: BTreeSet<String> = BTreeSet::new();
    let mut obs_paths: BTreeSet<String> = BTreeSet::new();
    for src in expression_sources(ir) {
        for (prefix, set) in [
            ("inputs.", &mut inputs),
            ("cfg.", &mut cfg_paths),
            ("obs.", &mut obs_paths),
        ] {
            let mut rest = src.as_str();
            while let Some(idx) = rest.find(prefix) {
                let after = &rest[idx + prefix.len()..];
                let end = after
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
                    .unwrap_or(after.len());
                let path = after[..end].trim_end_matches('.');
                if !path.is_empty() {
                    set.insert(path.to_string());
                }
                rest = &after[end.min(after.len())..];
            }
        }
    }
    let streams: BTreeSet<&str> = ir.streams.iter().map(|s| s.name.as_str()).collect();

    let resolves = |key: &str| -> bool {
        if let Some(name) = key.strip_prefix("inputs.") {
            inputs.contains(name)
        } else if let Some(path) = key.strip_prefix("cfg.") {
            cfg_paths
                .iter()
                .any(|p| p == path || p.starts_with(&format!("{path}.")))
        } else if let Some(path) = key.strip_prefix("obs.") {
            obs_paths
                .iter()
                .any(|p| p == path || p.starts_with(&format!("{path}.")))
        } else if let Some(name) = key
            .strip_prefix("stream.")
            .and_then(|k| k.strip_suffix(":amount"))
        {
            streams.contains(name)
        } else {
            false
        }
    };
    let hint = |key: &str| -> String {
        if !key.contains('.') && inputs.contains(key) {
            return format!("did you mean `inputs.{key}`?");
        }
        // The stream form with the wrong punctuation: a dotted `.amount` or a
        // bracketed name. Name the colon form the engine reads.
        if key.starts_with("stream") && key.ends_with("amount") {
            let inner = key
                .trim_start_matches("stream")
                .trim_end_matches("amount")
                .trim_matches(|c: char| c == '.' || c == ':' || c == '[' || c == ']' || c == '"');
            if let Some(near) = nearest(inner, streams.iter().copied()) {
                return format!("did you mean `stream.{near}:amount`?");
            }
        }
        if let Some(name) = key.strip_prefix("inputs.") {
            if let Some(near) = nearest(name, inputs.iter().map(String::as_str)) {
                return format!("did you mean `inputs.{near}`?");
            }
        }
        if let Some(near) = nearest(key, inputs.iter().map(String::as_str)) {
            return format!("did you mean `inputs.{near}`?");
        }
        "a key is `inputs.<name>`, `stream.<name>:amount`, `cfg.<path>` or `obs.<path>`".to_string()
    };

    let mut offences: Vec<String> = Vec::new();
    let mut check = |block: &str, keys: &mut dyn Iterator<Item = &String>| {
        for key in keys {
            if !resolves(key) {
                offences.push(format!(
                    "{block} sets `{key}`, which nothing in this model declares or reads — {}",
                    hint(key)
                ));
            }
        }
    };
    check(
        "the deterministic block",
        &mut config.parameter_overrides.keys(),
    );
    for (name, scenario) in &config.scenarios {
        check(
            &format!("scenario '{name}'"),
            &mut scenario.parameter_overrides.keys(),
        );
    }
    if let Some(mc) = &config.monte_carlo {
        check("monte_carlo.distributions", &mut mc.distributions.keys());
    }
    if offences.is_empty() {
        return Ok(());
    }
    Err(EngineError::InvalidRunConfig(format!(
        "{} — an override that matches nothing would leave the run unchanged and reported as ok",
        offences.join("; ")
    )))
}

/// Every expression source in the document, for the scans that need one.
fn expression_sources(ir: &Ir) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for stream in &ir.streams {
        out.push(stream.amount.src.clone());
        if let Some(e) = &stream.active_when {
            out.push(e.src.clone());
        }
    }
    for entity in &ir.entities {
        for rule in entity.rules.values() {
            out.push(rule.init.src.clone());
            out.push(rule.next.src.clone());
        }
    }
    for account in &ir.accounts {
        if let Some(e) = &account.inflow {
            out.push(e.src.clone());
        }
        if let Some(e) = &account.init {
            out.push(e.src.clone());
        }
    }
    for waterfall in &ir.waterfalls {
        out.push(waterfall.source.src.clone());
        for step in &waterfall.steps {
            out.push(step.amount.src.clone());
        }
    }
    for metric in &ir.metrics {
        out.push(metric.expr.src.clone());
    }
    for constant in ir.assumptions.constants.values() {
        out.push(constant.expr.src.clone());
    }
    for option in &ir.options {
        out.push(option.exercise_when.src.clone());
        out.push(option.payoff.src.clone());
    }
    for event in &ir.events {
        if let Some(e) = &event.when {
            out.push(e.src.clone());
        }
        for action in &event.actions {
            if let Some(e) = &action.value {
                out.push(e.src.clone());
            }
        }
    }
    out
}

/// The declared name nearest to `word` by edit distance, when one is close.
fn nearest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let mut best: Option<(usize, &str)> = None;
    for candidate in candidates {
        let d = edit_distance(word, candidate);
        if d <= 2.max(word.len() / 3) && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, candidate));
        }
    }
    best.map(|(_, c)| c)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

pub(crate) fn refuse_series_reads_in_logic(ir: &Ir) -> Result<(), EngineError> {
    let mut offences: Vec<String> = Vec::new();

    let mut check = |src: &str, site: String| {
        // Narrowed in phase 3: settled history is readable, this period and
        // the future are not. `docs/28` §4.
        if let Some(w) = cfdl_expr::series_windows(src)
            .into_iter()
            .find(|w| !cfdl_expr::window_bound_is_strictly_backward(&w.to_src))
        {
            offences.push(format!("{site} reads `{}` to `{}`", w.name, w.to_src));
        }
    };

    for entity in &ir.entities {
        for (field, rule) in &entity.rules {
            check(
                &rule.init.src,
                format!("field '{}.{field}' in 'init'", entity.symbol),
            );
            check(
                &rule.next.src,
                format!("field '{}.{field}' in 'next'", entity.symbol),
            );
        }
    }
    for event in &ir.events {
        if let Some(when) = &event.when {
            check(&when.src, format!("event '{}' guard", event.name));
        }
        for action in &event.actions {
            if let Some(value) = &action.value {
                check(&value.src, format!("event '{}' action value", event.name));
            }
        }
    }
    for option in &ir.options {
        check(
            &option.exercise_when.src,
            format!("option '{}' election", option.name),
        );
        check(
            &option.payoff.src,
            format!("option '{}' payoff", option.name),
        );
        for action in &option.actions {
            if let Some(value) = &action.value {
                check(&value.src, format!("option '{}' action value", option.name));
            }
        }
    }

    if offences.is_empty() {
        return Ok(());
    }
    Err(EngineError::SeriesReadInLogic(format!(
        "logic cannot read this period or later: {}. An event's guard and action values, a \
         field's rule, and an option's election and payoff all settle before this period's \
         cash exists, so only settled history is readable — end the window at `time.t - 1` \
         or earlier.",
        offences.join("; ")
    )))
}

/// One stream's series-read facts, extracted before any stream evaluates.
pub(crate) struct StreamDeps {
    /// Calls `series_sum`/`series_avg` anywhere in its amount or guard.
    pub(crate) uses: bool,
    /// At least one of those calls computes its series name at runtime.
    pub(crate) computed: bool,
    /// The literal read patterns, as written — globs included.
    pub(crate) refs: Vec<String>,
    /// Reads a window that can reach at or beyond the current period's future.
    ///
    /// A period walk can serve a read only if everything it reaches has
    /// already happened, so this is what decides whether a model can be
    /// walked. `docs/29` §2.0 measured the corpus: two benchmarks and two
    /// fixtures read forward, and they are exactly the two constructs
    /// `docs/28` §7 migrates to the valuation plane. Everything else reads
    /// cumulatively backward, which a walk serves exactly.
    pub(crate) reads_forward: bool,
    /// The forward reach is in the AMOUNT alone — the priced exception's
    /// shape (`docs/28` §7): the amount is a valuation over cells beyond the
    /// flow, and may set a causal amount where the graph stays acyclic. A
    /// forward-reaching GUARD is not priced: whether a stream is active is a
    /// causal fact, and a fact cannot be read from the future.
    pub(crate) amount_reads_forward: bool,
}

/// The cycle the priced exception refuses (`docs/28` §7), as a hard error
/// with the path named — not a routing decision, because no evaluation order
/// serves it: LOGIC settles before the priced pass in the walk, and reads
/// nothing at all under the column order. Sale proceeds feeding state that
/// feeds what is being capitalized is the canonical instance; logic reading
/// `prev.<account>` in a priced model is the same read through a balance
/// that may carry priced cash.
pub(crate) fn priced_refusal(ir: &Ir, deps: &[StreamDeps]) -> Option<String> {
    let closure = priced_closure(ir, deps);
    if !closure.iter().any(|p| *p) {
        return None;
    }
    let closure_names: Vec<&str> = ir
        .streams
        .iter()
        .zip(&closure)
        .filter(|(_, inc)| **inc)
        .map(|(stream, _)| stream.name.as_str())
        .collect();
    for (site, src) in &logic_expression_sources(ir) {
        for pattern in cfdl_expr::series_references(src) {
            if let Some(hit) = closure_names
                .iter()
                .find(|p| cfdl_expr::selector_matches(&pattern, p))
            {
                return Some(format!(
                    "{site} reads '{hit}', which a priced amount sets: a valuation feeding logic that feeds what is being capitalized is the cycle the priced exception refuses (docs/28 §7)"
                ));
            }
        }
        for account in &ir.accounts {
            let needle = format!("prev.{}", account.name);
            if src.contains(&needle) {
                return Some(format!(
                    "{site} reads {needle} in a model with a priced amount, and a balance may carry priced cash logic cannot yet see"
                ));
            }
        }
    }
    None
}

/// Which streams the priced pass owns: the priced amounts and every stream
/// that transitively reads one (`docs/28` §7). A priced amount SETS a causal
/// amount, and causal cells reading it are ordinary downstream flow — the
/// management fee on collections that include a priced recovery is the
/// shipped case. They evaluate together, in wave order, after the causal
/// walk settles; what stays outside is anything the walk itself must serve.
pub(crate) fn priced_closure(ir: &Ir, deps: &[StreamDeps]) -> Vec<bool> {
    let mut in_closure: Vec<bool> = deps.iter().map(|d| d.amount_reads_forward).collect();
    loop {
        let mut changed = false;
        for (idx, dep) in deps.iter().enumerate() {
            if in_closure[idx] {
                continue;
            }
            let reads_closure = dep.refs.iter().any(|pattern| {
                ir.streams.iter().enumerate().any(|(j, other)| {
                    in_closure[j] && cfdl_expr::selector_matches(pattern, &other.name)
                })
            });
            if reads_closure {
                in_closure[idx] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    in_closure
}

/// Every LOGIC expression a model has — event guards and their set values,
/// field rules, and machine edge guards — each with a name a refusal can
/// print. Logic settles before the priced pass runs, which is why a priced
/// coupling here routes the model back to the column order.
pub(crate) fn logic_expression_sources(ir: &Ir) -> Vec<(String, String)> {
    let mut sources: Vec<(String, String)> = Vec::new();
    for event in &ir.events {
        if let Some(when) = &event.when {
            sources.push((format!("event '{}' guard", event.name), when.src.clone()));
        }
        for action in &event.actions {
            if let Some(value) = &action.value {
                sources.push((
                    format!("event '{}' set value", event.name),
                    value.src.clone(),
                ));
            }
        }
    }
    for entity in &ir.entities {
        for (name, rule) in &entity.rules {
            sources.push((
                format!("field '{}.{name}' init", entity.symbol),
                rule.init.src.clone(),
            ));
            sources.push((
                format!("field '{}.{name}' next", entity.symbol),
                rule.next.src.clone(),
            ));
        }
    }
    for lifecycle in &ir.lifecycles {
        for edge in &lifecycle.edges {
            if let Some(guard) = &edge.guard {
                sources.push((
                    format!(
                        "lifecycle '{}' edge '{} -> {}' guard",
                        lifecycle.id, edge.from, edge.to
                    ),
                    guard.src.clone(),
                ));
            }
        }
    }
    sources
}

/// Can this model be evaluated one period at a time?
///
/// The question the period walk asks before it runs. `None` means yes;
/// `Some(reason)` names the first stream that reads forward, so a refusal or a
/// routing decision can say which construct forced it.
///
/// Not yet a routing decision: until `docs/28` §4's backward reads land, no
/// model couples cash into logic, so the walk and the column order agree
/// wherever both can run. This is the predicate that will choose between them,
/// and the one the equivalence test uses to know which fixtures to compare.
/// Each stream's series dependencies, the reads it makes and whether any
/// window reaches forward. Extracted so the walk's eligibility query and the
/// wave ordering compute it the same way rather than twice.
pub(crate) fn stream_deps(ir: &Ir) -> Vec<StreamDeps> {
    let mut deps: Vec<StreamDeps> = Vec::with_capacity(ir.streams.len());
    for stream in &ir.streams {
        // A STREAM READS SERIES IF *ANY* OF ITS EXPRESSIONS DOES, not just its
        // amount. `active when series_sum(...) > 0` on a stream whose amount
        // happens not to use one was once handed an empty series map, and its
        // guard then failed — warned, evaluated FALSE, and the stream silently
        // produced nothing at all. An expression that fails to compile
        // contributes nothing here; `evaluate_stream` warns about it later.
        let probe = |src: &str| -> (bool, bool) {
            cfdl_expr::compile_expr(src)
                .map(|c| {
                    (
                        cfdl_expr::uses_series(&c),
                        cfdl_expr::has_computed_series_name(&c),
                    )
                })
                .unwrap_or((false, false))
        };
        // A window reaching forward is measured over the same expressions the
        // reads are extracted from, so a guard cannot smuggle one past.
        let forward = |src: &str| -> bool {
            cfdl_expr::series_windows(src).iter().any(|w| {
                !(cfdl_expr::window_bound_is_backward(&w.from_src)
                    && cfdl_expr::window_bound_is_backward(&w.to_src))
            })
        };
        let (mut uses, mut computed) = probe(&stream.amount.src);
        let mut refs = cfdl_expr::series_references(&stream.amount.src);
        let amount_reads_forward = forward(&stream.amount.src);
        let mut reads_forward = amount_reads_forward;
        if let Some(guard) = &stream.active_when {
            let (guard_uses, guard_computed) = probe(&guard.src);
            uses |= guard_uses;
            computed |= guard_computed;
            refs.extend(cfdl_expr::series_references(&guard.src));
            reads_forward |= forward(&guard.src);
        }
        deps.push(StreamDeps {
            uses,
            computed,
            refs,
            reads_forward,
            amount_reads_forward,
        });
    }
    deps
}

pub(crate) fn walk_ineligible_reason(ir: &Ir, deps: &[StreamDeps]) -> Option<String> {
    // THE PRICED EXCEPTION (`docs/28` §7). A forward window in an AMOUNT is a
    // valuation setting a causal amount — the forward-income exit, the
    // expense stop's base year — and the walk serves it in a priced pass
    // after the causal cells settle. A forward window in a GUARD is not
    // priced: activity is a causal fact, and the model keeps the column
    // order, named.
    let mut priced: Vec<&str> = Vec::new();
    for (stream, dep) in ir.streams.iter().zip(deps) {
        if dep.reads_forward && !dep.amount_reads_forward {
            return Some(format!(
                "stream '{}' has a guard whose series window can reach beyond the current period",
                stream.name
            ));
        }
        if dep.amount_reads_forward {
            priced.push(&stream.name);
        }
    }
    let _ = priced;
    // A WATERFALL'S STEPS READ SERIES TOO, and they are not in `ir.streams`.
    // `waterfall_nested_split` reads `[0..5]` from a step, which a
    // streams-only check reported as walkable — the fund waterfall composition
    // of `docs/17` is exactly where an absolute window is natural.
    let reaches_forward = |src: &str| -> bool {
        cfdl_expr::series_windows(src).iter().any(|w| {
            !(cfdl_expr::window_bound_is_backward(&w.from_src)
                && cfdl_expr::window_bound_is_backward(&w.to_src))
        })
    };
    for waterfall in &ir.waterfalls {
        {
            if reaches_forward(&waterfall.source.src) {
                return Some(format!(
                    "waterfall '{}' draws from a pot whose window can reach beyond the current period",
                    waterfall.name
                ));
            }
        }
        for step in &waterfall.steps {
            if reaches_forward(&step.amount.src) {
                return Some(format!(
                    "waterfall '{}' step '{}' reads a series window that can reach beyond the current period",
                    waterfall.name, step.name
                ));
            }
        }
    }
    None
}

/// Assign each stream the wave it evaluates in: 0 for streams that read no
/// series, and one past the deepest stream it reads for everything else. The
/// only rejections are the ones no order can satisfy — a circular read, and a
/// read into a stream whose series names are computed at runtime.
pub(crate) fn assign_waves(names: &[&str], deps: &[StreamDeps]) -> Result<Vec<usize>, EngineError> {
    // Resolve each literal pattern to the streams it names, reader -> producers.
    // Matched as SELECTORS, not exact names: `cre.unit.recoveries.*` as written
    // must find `cre.unit.recoveries.suite_100` as lowered.
    let mut edges: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); names.len()];
    for (reader, dep) in deps.iter().enumerate() {
        for pattern in &dep.refs {
            for (producer, name) in names.iter().enumerate() {
                if cfdl_expr::selector_matches(pattern, name) {
                    edges[reader].insert(producer);
                }
            }
        }
    }

    // A computed-name reader cannot be placed in the order (its edges are
    // unknowable), so it evaluates after every literally-named stream — and
    // nothing may read it, because such a read could never be ordered.
    for (reader, edge_set) in edges.iter().enumerate() {
        for &producer in edge_set {
            if deps[producer].computed {
                return Err(EngineError::SeriesCycle(format!(
                    "Stream '{}' reads series '{}', which computes its series names at \
                     runtime, so its place in the evaluation order cannot be determined. \
                     A stream with computed series names always evaluates last and cannot \
                     be read by another stream.",
                    names[reader], names[producer]
                )));
            }
        }
    }

    // Depth-first depth assignment. GRAY means "on the current chain", so
    // reaching a GRAY stream closes a genuine cycle — the one thing that has
    // no evaluation order. The engine refuses it rather than iterating toward
    // a fixed point.
    const WHITE: u8 = 0;
    const GRAY: u8 = 1;
    const BLACK: u8 = 2;
    fn depth_of(
        node: usize,
        names: &[&str],
        deps: &[StreamDeps],
        edges: &[BTreeSet<usize>],
        color: &mut [u8],
        depth: &mut [usize],
        chain: &mut Vec<usize>,
    ) -> Result<usize, EngineError> {
        if color[node] == BLACK {
            return Ok(depth[node]);
        }
        if color[node] == GRAY {
            let start = chain.iter().position(|&n| n == node).unwrap_or(0);
            let mut path: Vec<&str> = chain[start..].iter().map(|&n| names[n]).collect();
            path.push(names[node]);
            return Err(EngineError::SeriesCycle(format!(
                "cyclic series reads: {}. Each read needs the stream it names \
                 finished first, so no evaluation order exists. CFDL refuses a \
                 circular reference rather than iterating it; break the cycle by \
                 removing one of the reads.",
                path.iter()
                    .map(|n| format!("'{n}'"))
                    .collect::<Vec<_>>()
                    .join(" -> ")
            )));
        }
        color[node] = GRAY;
        chain.push(node);
        let mut deepest = 0usize;
        for &producer in &edges[node] {
            deepest = deepest.max(depth_of(producer, names, deps, edges, color, depth, chain)?);
        }
        chain.pop();
        color[node] = BLACK;
        // A reader is never wave 0 even when its reads resolve to nothing: it
        // still receives the sealed store, exactly as the old phase 2 did, so
        // an unresolved read keeps aggregating to zero under W5022 instead of
        // becoming a missing-context warning.
        depth[node] = if deps[node].uses { deepest + 1 } else { 0 };
        Ok(depth[node])
    }

    let mut color = vec![WHITE; names.len()];
    let mut depth = vec![0usize; names.len()];
    let mut chain: Vec<usize> = Vec::new();
    let mut max_literal = 0usize;
    for node in 0..names.len() {
        if deps[node].computed {
            continue;
        }
        let d = depth_of(
            node, names, deps, &edges, &mut color, &mut depth, &mut chain,
        )?;
        max_literal = max_literal.max(d);
    }
    for node in 0..names.len() {
        if deps[node].computed {
            depth[node] = max_literal + 1;
        }
    }
    Ok(depth)
}

/// Everything about a model that does not vary from one run to the next.
///
/// THE GRID IS BUILT ONCE. Monte Carlo runs the whole deterministic engine per
/// trial, and a trial varies only its sampled inputs — not the calendar, not
/// the expressions, not the schedules. Rebuilding those per trial meant twenty
/// thousand trials compiling one model twenty thousand times, and `docs/29`
/// §2.2 already said the schedule is "computed once and replayed per scenario
/// and per trial" while the code did the opposite.
pub(crate) struct ModelPrep<'a> {
    pub(crate) timeline: Vec<Date>,
    /// Compiled amounts, guards and schedules, in `ir.streams` order.
    pub(crate) plans: Vec<StreamPlan<'a>>,
    /// Each stream's evaluation wave, from the dependency graph.
    pub(crate) waves: Vec<usize>,
    /// Each account's compiled inflow, in `ir.accounts` order.
    pub(crate) account_inflows: Vec<Option<cfdl_expr::CompiledExpr>>,
    /// Each account's compiled `init`, likewise; `None` opens at zero.
    pub(crate) account_inits: Vec<Option<cfdl_expr::CompiledExpr>>,
    /// Why this model cannot be walked, when it cannot: a window somewhere
    /// reaches past the period being computed, and a walk has no such period
    /// yet. `None` means the walk runs it.
    pub(crate) walk_ineligible: Option<String>,
}

/// Compile and schedule a model once, for every run that follows.
pub(crate) fn prepare_model<'a>(
    ir: &'a Ir,
    warnings: &mut Vec<String>,
) -> Result<ModelPrep<'a>, EngineError> {
    let total_periods = ir.time.periods as usize + ir.time.projection as usize;
    let timeline = timeline_dates(&ir.time.start, &ir.time.calendar, total_periods)?;
    refuse_unknown_action_kinds(ir)?;
    let deps = stream_deps(ir);
    if let Some(path) = priced_refusal(ir, &deps) {
        return Err(EngineError::SeriesCycle(path));
    }
    let stream_names: Vec<&str> = ir.streams.iter().map(|s| s.name.as_str()).collect();
    let waves = assign_waves(&stream_names, &deps)?;
    let mut plans = Vec::with_capacity(ir.streams.len());
    for stream in &ir.streams {
        plans.push(plan_stream(ir, stream, &timeline, warnings)?);
    }
    warn_curve_reads_past_end(ir, &plans, &timeline, warnings);
    let account_inflows: Vec<Option<cfdl_expr::CompiledExpr>> = ir
        .accounts
        .iter()
        .map(|account| {
            account.inflow.as_ref().and_then(|expr| {
                cfdl_expr::compile_expr(&expr.src)
                    .map_err(|err| {
                        warnings.push(format!(
                            "Account '{}' inflow failed to compile [{}]: {}; treated as zero.",
                            account.name, err.code, err.message
                        ));
                    })
                    .ok()
            })
        })
        .collect();
    let account_inits: Vec<Option<cfdl_expr::CompiledExpr>> = ir
        .accounts
        .iter()
        .map(|account| {
            account.init.as_ref().and_then(|expr| {
                cfdl_expr::compile_expr(&expr.src)
                    .map_err(|err| {
                        warnings.push(format!(
                            "Account '{}' init failed to compile [{}]: {}; opens at zero.",
                            account.name, err.code, err.message
                        ));
                    })
                    .ok()
            })
        })
        .collect();
    let walk_ineligible = walk_ineligible_reason(ir, &deps);
    Ok(ModelPrep {
        timeline,
        plans,
        waves,
        account_inflows,
        account_inits,
        walk_ineligible,
    })
}
