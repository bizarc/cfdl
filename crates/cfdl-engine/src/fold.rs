// The fold: from what the evaluation settled to what the results carry
// (`docs/28` §3 names it last; `docs/06` is its shape).
//
// Streams the walk did not run — a model on the column order — evaluate here
// a column at a time in dependency waves. Then the subtotals a pack declares,
// the account and entity roll-ups through `part of`, every published series,
// the metrics (NPV, IRR, MOIC, WAL, totals, participants' returns), the
// slices, the declared metrics against the valuation plane, the annual
// roll-up and the inputs the statements read. A fold OF the cash is never
// counted AS cash: a parent and its children would otherwise double what they
// touch. An unresolved name anywhere in the run is fatal here, not a zero.
use super::*;

#[derive(Debug, Clone)]
pub(crate) struct DeterministicRunOutput {
    pub(crate) slices: Vec<SliceResult>,
    pub(crate) warnings: Vec<String>,
    /// Evaluated `assume` values, carried out so `compute_results` can publish
    /// them without re-evaluating (which would duplicate every warning).
    pub(crate) resolved_inputs: BTreeMap<String, f64>,
    pub(crate) metrics: BTreeMap<String, Scalar>,
    pub(crate) series: BTreeMap<String, Series>,
    pub(crate) npv: f64,
    pub(crate) annual_rollup: Option<AnnualRollupSection>,
    pub(crate) transitions: Vec<TransitionRecord>,
    pub(crate) journal: Vec<JournalEntry>,
}

/// The distinct unresolved names a run's warnings report.
///
/// Every evaluation site formats the error's code into its warning, and
/// `ExprError`'s Display writes `[CODE] message`, so one marker finds them all
/// however the site chose to phrase the rest. Deduplicated because a name that
/// fails once fails every period.
/// WHAT THE RUN DISCOUNTS WITH (`docs/13` §7.4): one annual rate, or a curve
/// the model declares read at each period's date. The flat case keeps the
/// scalar formula byte for byte; the curve case walks the path.
pub(crate) enum Discount {
    Flat {
        annual: f64,
        per_period: f64,
    },
    Path {
        annual: Vec<f64>,
        per_period: Vec<f64>,
    },
}

impl Discount {
    pub(crate) fn from_config(
        ir: &Ir,
        config: &RunConfig,
        timeline: &[Date],
        ppy: f64,
    ) -> Result<Self, EngineError> {
        let Some(name) = &config.discount_curve else {
            return Ok(Discount::Flat {
                annual: config.discount_rate,
                per_period: (1.0 + config.discount_rate).powf(1.0 / ppy) - 1.0,
            });
        };
        let curves = env::ir_curve_defs(ir);
        let Some(curve) = curves.get(name.as_str()) else {
            let known: Vec<&str> = curves.keys().map(String::as_str).collect();
            return Err(EngineError::InvalidRunConfig(format!(
                "annual_discount_curve '{name}' names no curve this model declares{}",
                if known.is_empty() {
                    String::new()
                } else {
                    format!("; declared: {}", known.join(", "))
                }
            )));
        };
        let mut annual = Vec::with_capacity(timeline.len());
        for date in timeline {
            let Some(calc) = cfdl_calc::CalcDate::new(date.year, date.month, date.day) else {
                return Err(EngineError::InvalidDate(date.to_string()));
            };
            match cfdl_expr::curve_value_at(curve, calc) {
                cfdl_calc::CurveLookup::Value(v) => {
                    annual.push(cfdl_calc::decimal_to_f64(v));
                }
                cfdl_calc::CurveLookup::OutsideRange { from, to } => {
                    let bounds = match (from, to) {
                        (Some(f), Some(t)) => format!("from {f} to {t}"),
                        (Some(f), None) => format!("from {f}"),
                        (None, Some(t)) => format!("to {t}"),
                        (None, None) => String::new(),
                    };
                    return Err(EngineError::CurveReadOutsideRange(format!(
                        "the run's discount curve '{name}' has no value at {date}: its effective \
                         dates run {bounds} — outside them a curve has no value. Extend the \
                         curve's dates to the valuation horizon, or end the model where the \
                         curve does."
                    )));
                }
                cfdl_calc::CurveLookup::Unknown => {
                    return Err(EngineError::InvalidRunConfig(format!(
                        "annual_discount_curve '{name}' has no value at {date}"
                    )));
                }
            }
        }
        let per_period = annual
            .iter()
            .map(|r| (1.0 + r).powf(1.0 / ppy) - 1.0)
            .collect();
        Ok(Discount::Path { annual, per_period })
    }

    pub(crate) fn npv(&self, streams: &[(Vec<f64>, f64)]) -> f64 {
        match self {
            Discount::Flat { per_period, .. } => npv_with_offsets(streams, *per_period),
            Discount::Path { per_period, .. } => npv_along_path(streams, per_period),
        }
    }

    /// At an annual grain each bucket takes the annual rate at its first
    /// period's date.
    pub(crate) fn npv_at_grain(
        &self,
        streams: &[(Vec<f64>, f64)],
        grain: &Grain,
        timeline: &[Date],
    ) -> f64 {
        match self {
            Discount::Flat { annual, .. } => npv_at_grain(streams, *annual, grain),
            Discount::Path { annual, .. } => {
                let _ = timeline;
                let per_bucket: Vec<f64> = grain
                    .buckets
                    .iter()
                    .map(|members| {
                        members
                            .first()
                            .and_then(|&i| annual.get(i).copied())
                            .unwrap_or(0.0)
                    })
                    .collect();
                npv_at_grain_along_path(streams, &per_bucket, grain)
            }
        }
    }
}

pub(crate) fn unresolved_names(warnings: &[String], declared: &BTreeSet<String>) -> Vec<String> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for w in warnings {
        if !w.contains(cfdl_expr::EXPR_UNKNOWN_NAME) {
            continue;
        }
        // The message reads `... unknown variable `inputs.x`; using 0.`
        let Some(start) = w.find("unknown variable `") else {
            seen.insert(w.clone());
            continue;
        };
        let rest = &w[start + "unknown variable `".len()..];
        let Some(end) = rest.find('`') else {
            seen.insert(w.clone());
            continue;
        };
        let name = &rest[..end];
        // DECLARED SOMEWHERE IS NOT THE SAME AS BOUND HERE. An input may be
        // declared only as a Monte Carlo distribution, which leaves it unbound
        // in the deterministic pass — `run_dists_full` is exactly that model,
        // and its deterministic run is incidental to the trials it exists to
        // exercise. That is a different condition from a name nothing
        // declares, and only the second is fatal.
        if declared.contains(name) {
            continue;
        }
        seen.insert(format!("`{name}` is not declared"));
    }
    seen.into_iter().collect()
}

/// The fold: everything the results carry, computed over what the evaluation
/// settled. Streams the walk did not run (the column order) evaluate here a
/// column at a time in dependency waves; then subtotals, the account and
/// entity roll-ups, every published series, the metrics, the slices, the
/// declared metrics, the annual roll-up and the statements' inputs.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fold_results(
    ir: &Ir,
    config: &RunConfig,
    prep: &ModelPrep<'_>,
    timeline: &[Date],
    base_inputs: BTreeMap<String, f64>,
    evaluated: Evaluated,
    mut warnings: Vec<String>,
) -> Result<DeterministicRunOutput, EngineError> {
    // Cash horizon vs full evaluation window: the projection tail
    // (`time ... project <n>`) is computed so series_sum/series_avg can read
    // past the horizon (e.g. forward NOI at exit), but contributes nothing to
    // cash results, totals, or NPV.
    let cash_periods = ir.time.periods as usize;
    let periods = cash_periods;
    let Evaluated {
        state_values,
        event_sim,
        walked_columns,
        walked_refusals,
        account_balances,
        walked_waterfalls,
        mut stage_journal,
    } = evaluated;
    let transitions = event_sim.transitions.clone();
    // The journal opens with what the state stage did and grows as each later
    // stage acts, so its order is the order the run happened in.
    let mut journal = event_sim.journal.clone();
    // THE MASK DEFAULTS TO TRUE, so "the mask is on" does not mean an event
    // turned it on — a stream nothing ever touched has an all-true mask. Only
    // a period at or after an actual `activate stream` can be an activation
    // that `active when` then refused, so the first such period per stream is
    // the threshold the streams stage measures against.
    let first_activation: BTreeMap<String, usize> = journal
        .iter()
        .filter(|entry| entry.action == "activate_stream" && entry.outcome == "applied")
        .fold(BTreeMap::new(), |mut acc, entry| {
            let slot = acc.entry(entry.target.clone()).or_insert(entry.period);
            *slot = (*slot).min(entry.period);
            acc
        });
    let mut stream_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    // Each stream's placement in its period, published on the series so a
    // consumer holding results.json can recompute the time-weighted metrics
    // the engine reported. `stream_series` is keyed by bare name; so is this.
    let mut stream_offsets: BTreeMap<String, f64> = BTreeMap::new();
    let mut stream_totals: BTreeMap<String, f64> = BTreeMap::new();
    let mut model_series = vec![0.0_f64; cash_periods];
    // Each stream's series paired with where in its period the cash falls;
    // valuation needs both, while reported cash uses model_series alone.
    let mut valued_streams: Vec<(Vec<f64>, f64)> = Vec::new();

    // --- Streams: dependency-ordered waves ---------------------------------
    //
    // Wave 0 is every stream that reads no series. A reader's wave is one past
    // the deepest stream it reads, so every wave evaluates against a store in
    // which everything it references is already finished. The graph is the one
    // the model states: `series_references` extracts each read as written and
    // `selector_matches` resolves it to the streams it names — the same edges
    // the old two-phase guard walked to REJECT any chain of depth two. Sorting
    // them instead gives exactly acyclicity: a genuine circular read is the
    // only rejection left, because a cycle has no evaluation order and the
    // engine does not iterate to convergence (docs/14 §5 — no fixed-point
    // solver). History that shaped this: the guard matched references as
    // SELECTORS, not exact names, because `cre.exit_forward` reads
    // `series_sum("cre.unit.recoveries.*", ...)` and an exact lookup of the
    // pattern found nothing — measured on `mit_rentleg_plaza`, an exit price
    // $116,440 lower with no diagnostic.
    let waves = &prep.waves;
    let max_wave = waves.iter().copied().max().unwrap_or(0);

    let mut full_series: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for wave in 0..=max_wave {
        // Wave 0 reads nothing and gets no store, and every later wave gets a
        // snapshot sealed when the wave before it finished — wrapped once per
        // wave, not per accrual, so every env in the wave shares one map.
        let snapshot = (wave > 0).then(|| Arc::new(full_series.clone()));
        for (idx, stream) in ir.streams.iter().enumerate() {
            if waves[idx] != wave {
                continue;
            }
            let mut activation_refused: Vec<usize> = Vec::new();
            let plan = &prep.plans[idx];
            let mut values = vec![0.0_f64; timeline.len()];
            if let Some(columns) = &walked_columns {
                // The walk already evaluated every period of this stream, in
                // the order that let the state stage see the cash. Recomputing
                // here would be the same arithmetic twice.
                if let Some(column) = columns.get(&stream.name) {
                    values.clone_from(column);
                }
                activation_refused = walked_refusals
                    .get(&stream.name)
                    .cloned()
                    .unwrap_or_default();
            } else {
                for (pay_idx, slot) in values.iter_mut().enumerate() {
                    *slot = plan.step(
                        ir,
                        config,
                        pay_idx,
                        timeline,
                        &base_inputs,
                        &event_sim.entity_state,
                        &event_sim.stream_active,
                        &state_values,
                        snapshot.as_ref(),
                        &mut warnings,
                        &mut activation_refused,
                        None,
                        None,
                    );
                }
            }
            // ONE ROW PER STREAM, at the first period the refusal bit. An
            // activation persists forward, so a per-period row would repeat
            // the same fact for the rest of the run.
            let activated_at = first_activation.get(&stream.name).copied();
            let activation_refused: Vec<usize> = match activated_at {
                Some(from) => activation_refused
                    .into_iter()
                    .filter(|idx| *idx >= from)
                    .collect(),
                None => Vec::new(),
            };
            if let Some(&first) = activation_refused.first() {
                let count = activation_refused.len();
                journal.push(
                    JournalEntry::new(
                        first,
                        &timeline[first].to_string(),
                        format!("stream:{}", stream.name),
                        "activate_stream",
                        stream.name.clone(),
                        "overridden",
                    )
                    .with_note(format!(
                        "an event activated this stream and its own `active when` was \
                         false for {count} scheduled period(s) from this one; both \
                         gates must pass, so the activation did not turn it on"
                    )),
                );
            }
            warn_if_cash_settles_in_tail(stream, &values, cash_periods, &mut warnings);
            let offset = discount_offset(&stream.schedule, &ir.time.calendar);
            stream_offsets.insert(stream.name.clone(), offset);
            if streams::is_cash(stream) {
                valued_streams.push((values[..cash_periods.min(values.len())].to_vec(), offset));
            }
            record_stream(
                stream,
                &values,
                cash_periods,
                &mut model_series,
                &mut stream_totals,
                &mut stream_series,
            );
            // The FULL (projection-inclusive) values feed later waves.
            full_series.insert(stream.name.clone(), values);
        }
        if wave == 0 {
            // A NAME THAT PRODUCES NOTHING READS AS ZERO, and said nothing at
            // all until now. Same reasoning as the cycle check — a read that
            // can never resolve reported a plausible number — with a softer
            // verdict, because a literal name matching nothing is a pack idiom
            // as well as a typo.
            check_series_names(ir, &mut warnings);
        }
    }

    for (name, values) in &event_sim.option_cash {
        let cash = &values[..cash_periods.min(values.len())];
        for (idx, value) in cash.iter().enumerate() {
            model_series[idx] += *value;
        }
        valued_streams.push((cash.to_vec(), 0.0));
        let total = cash.iter().sum::<f64>();
        stream_totals.insert(format!("option.{name}"), total);
        stream_series.insert(format!("option.{name}"), cash.to_vec());
        // Option exercise cash settles on its exercise date, so it sits at the
        // period's open — matching the 0.0 pushed into valued_streams above.
        stream_offsets.insert(format!("option.{name}"), 0.0);
    }

    // --- Subtotals: the fold layer ------------------------------------------
    //
    // Evaluated after every stream and every option, so the ledger is complete,
    // and in the IR's array order, which is the dependency order the pack
    // declared. A reference can only reach something already computed, which is
    // what makes a cycle unexpressible rather than merely rejected.
    //
    // These live in their OWN maps and are never merged into `stream_series`.
    // That is the same construction the `state.` prefix relies on below, and it
    // is load-bearing: `model_series` was summed from streams alone,
    // `valued_streams` drives NPV and IRR, and `build_annual_rollup` iterates
    // `stream_series`. A subtotal is a fold OF the cash, so counting it as cash
    // would double every number it touches.
    let stream_category: BTreeMap<&str, &str> = ir
        .streams
        .iter()
        .filter_map(|s| s.category.as_deref().map(|c| (s.name.as_str(), c)))
        .collect();
    // A non-cash stream is never folded into a money subtotal, by category
    // or by name (`docs/42` §3.7).
    let noncash: BTreeSet<&str> = ir
        .streams
        .iter()
        .filter(|s| !streams::is_cash(s))
        .map(|s| s.name.as_str())
        .collect();
    let mut subtotal_money: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut subtotal_ratio: BTreeMap<String, Vec<Option<f64>>> = BTreeMap::new();

    for spec in &ir.subtotals {
        match spec.op.as_str() {
            "sum" | "negated_sum" => {
                let sign = if spec.op == "negated_sum" { -1.0 } else { 1.0 };
                let mut acc = vec![0.0_f64; cash_periods];
                for (name, values) in &stream_series {
                    if noncash.contains(name.as_str()) {
                        continue;
                    }
                    // A stream is folded if its CATEGORY is selected, or if it
                    // is named outright. Category first: it is what the pack
                    // meant, and it keeps a subtotal correct when the pack
                    // grows a contract nobody thought to add here.
                    let by_category = stream_category
                        .get(name.as_str())
                        .is_some_and(|c| cfdl_expr::selector_matches_any(&spec.categories, c));
                    let by_name = cfdl_expr::selector_matches_any(&spec.streams, name);
                    if !(by_category || by_name) {
                        continue;
                    }
                    for (t, v) in values.iter().take(cash_periods).enumerate() {
                        acc[t] += sign * v;
                    }
                }
                for referenced in &spec.subtotals {
                    if let Some(src) = subtotal_money.get(referenced) {
                        for (t, v) in src.iter().enumerate() {
                            acc[t] += sign * v;
                        }
                    }
                }
                // Rounded HERE, not just on the way out, so the ratio below
                // divides the same numbers that get published. Two reasons.
                //
                // A fold of signed cash whose flows cancel leaves a residue —
                // about 2e-12 — rather than an exact zero. Dividing that by a
                // real denominator yields ~2.6e-17, whose last bits differ by
                // platform: that shipped, and Windows disagreed with Linux and
                // macOS on one golden.
                //
                // And it makes the published rows self-consistent: a reader can
                // divide the published NOI by the published debt service and
                // get the published coverage ratio, instead of a number that
                // only reconciles against intermediates nobody can see.
                for v in acc.iter_mut() {
                    *v = round_amount(*v);
                }
                subtotal_money.insert(spec.id.clone(), acc);
            }
            // A RUNNING TOTAL, which a per-period fold cannot express.
            //
            // Percent-of-pool-outstanding is cumulative principal over the
            // original balance, and that shape appears wherever a stock is
            // derived from a flow: principal paid to date, cumulative capital
            // called, drawn-to-date on a facility. Every other op answers "what
            // happened in this period"; this one answers "how much so far".
            //
            // Built from the per-period fold rather than beside it, so a
            // cumulative subtotal and the periodic one it accumulates cannot
            // disagree about what they are summing.
            "cumulative" | "negated_cumulative" => {
                let sign = if spec.op == "negated_cumulative" {
                    -1.0
                } else {
                    1.0
                };
                let mut acc = vec![0.0_f64; cash_periods];
                for (name, values) in &stream_series {
                    if noncash.contains(name.as_str()) {
                        continue;
                    }
                    let by_category = stream_category
                        .get(name.as_str())
                        .is_some_and(|c| cfdl_expr::selector_matches_any(&spec.categories, c));
                    let by_name = cfdl_expr::selector_matches_any(&spec.streams, name);
                    if !(by_category || by_name) {
                        continue;
                    }
                    for (t, v) in values.iter().take(cash_periods).enumerate() {
                        acc[t] += sign * v;
                    }
                }
                for referenced in &spec.subtotals {
                    if let Some(src) = subtotal_money.get(referenced) {
                        for (t, v) in src.iter().enumerate() {
                            acc[t] += sign * v;
                        }
                    }
                }
                let mut running = 0.0;
                for v in acc.iter_mut() {
                    running += *v;
                    *v = round_amount(running);
                }
                subtotal_money.insert(spec.id.clone(), acc);
            }
            "ratio" => {
                let (Some(num_id), Some(den_id)) = (&spec.numerator, &spec.denominator) else {
                    continue;
                };
                let (Some(num), Some(den)) =
                    (subtotal_money.get(num_id), subtotal_money.get(den_id))
                else {
                    continue;
                };
                // A zero denominator publishes `null` and says nothing else.
                // It is not a warning: a coverage ratio is genuinely undefined
                // once a loan matures, and HUD's does at year 14 of 29 — that
                // is the model being right, not a problem. A warning firing on
                // correct models is noise, and it would fail every benchmark,
                // since tools/benchmark-runner.py treats any warning as a
                // failure.
                //
                // Nothing is discarded silently either, which is the standard
                // that would otherwise argue for a warning: the null is IN the
                // series, per period, so a reader sees exactly which periods
                // are undefined and a consumer cannot mistake one for zero.
                let values: Vec<Option<f64>> = (0..cash_periods)
                    .map(|t| {
                        let d = den.get(t).copied().unwrap_or(0.0);
                        (d.abs() > f64::EPSILON).then(|| num.get(t).copied().unwrap_or(0.0) / d)
                    })
                    .collect();
                subtotal_ratio.insert(spec.id.clone(), values);
            }
            _ => {}
        }
    }

    // Waterfalls run last: a priority of payments allocates cash that this
    // period's streams and states have already produced.
    // Computed from streams alone, before any distribution: the quantity a
    // waterfall's `available` binding reads.
    let waterfall_series = match walked_waterfalls {
        // The walk already ran the stage, period by period; its journal joins
        // here, after the state stage's entries, the position the post-pass
        // wrote from.
        Some(series) => {
            journal.append(&mut stage_journal);
            series
        }
        None => {
            let available_by_entity = stream_cash_by_entity(ir, &stream_series, cash_periods);
            run_waterfalls(
                ir,
                timeline,
                &base_inputs,
                &state_values,
                &event_sim.entity_state,
                &ir_curve_defs(ir),
                &stream_series,
                &available_by_entity,
                config,
                &mut warnings,
                &mut journal,
            )
        }
    };

    let mut series_map = BTreeMap::new();
    // Ownership and category, published beside the values (docs/13 §7.43):
    // a consumer holding results alone can attribute a stream to the thing
    // that owns it and the kind of cash it is.
    let stream_attribution: BTreeMap<&str, StreamAttribution<'_>> = ir
        .streams
        .iter()
        .map(|s| {
            (
                s.name.as_str(),
                (
                    s.owner.symbol.as_str(),
                    s.category.as_deref(),
                    lowering_contract(s),
                    lowering_line(s),
                ),
            )
        })
        .collect();
    for (name, values) in &stream_series {
        let mut series = Series::from_values(
            &ir.time.calendar,
            &ir.time.start,
            periods as u32,
            &ir.model.currency,
            stream_offsets.get(name).copied(),
            values,
        );
        if let Some((owner, category, contract, line)) = stream_attribution.get(name.as_str()) {
            series.entity = Some((*owner).to_string());
            series.category = category.map(str::to_string);
            series.contract = contract.map(str::to_string);
            series.line = line.map(str::to_string);
        }
        series_map.insert(format!("stream.{name}"), series);
    }
    // AN ACCOUNT IS A LOCATION, NOT A FLOW. Its balance publishes as a bare
    // number under its own prefix — never as cash and never into a total —
    // because the cash it holds has already been counted once as the stream
    // that produced it. The step series is the flow; this is the position.
    // Cut at the cash horizon like every other series: the walk carries a
    // balance through the projection tail for lookups, and a reader lining
    // series up by period must find one length.
    for (name, values) in &account_balances {
        let cash = &values[..values.len().min(ir.time.periods as usize)];
        series_map.insert(
            format!("account.{name}"),
            Series::from_plain(&ir.time.calendar, &ir.time.start, periods as u32, cash),
        );
    }
    // Each waterfall step is a stream, so a priority of payments publishes
    // under the same prefix everything else pays under and needs no special
    // handling downstream.
    let waterfall_entity: BTreeMap<&str, &str> = ir
        .waterfalls
        .iter()
        .map(|w| (w.name.as_str(), w.entity.as_str()))
        .collect();
    // A step that pays for a contract: its series carries the contract and
    // the line, as a lowered stream's does (docs/40 §6).
    let step_attribution: BTreeMap<String, (&str, &str)> = ir
        .waterfalls
        .iter()
        .flat_map(|w| {
            w.steps.iter().filter_map(move |step| {
                Some((
                    format!("{}.{}", w.name, step.name),
                    (step.contract.as_deref()?, step.line.as_deref()?),
                ))
            })
        })
        .collect();
    for (name, values) in &waterfall_series {
        let mut series = Series::from_values(
            &ir.time.calendar,
            &ir.time.start,
            periods as u32,
            &ir.model.currency,
            None,
            values,
        );
        // A step's series belongs to the waterfall's attached entity.
        if let Some(owner) = name.split('.').next().and_then(|w| waterfall_entity.get(w)) {
            series.entity = Some((*owner).to_string());
        }
        if let Some((contract, line)) = step_attribution.get(name.as_str()) {
            series.contract = Some((*contract).to_string());
            series.line = Some((*line).to_string());
        }
        series_map.insert(format!("stream.{name}"), series);
    }
    // States and fields, published for inspection and never counted as cash.
    //
    // WHAT KEEPS THEM OUT IS THE COLLECTION THEY ARE NOT IN, not the name they
    // publish under. Every cash consumer reads a stream collection: the WAL and
    // payback weightings look up `stream.<name>` keys, the annual rollup
    // iterates `stream_series`, and `model_series` and `valued_streams` were
    // summed above from streams alone, before this map is written. A value that
    // never entered those cannot reach model.total, model.npv, the IRR or any
    // domain metric.
    //
    // This comment used to say the `state.` prefix was the guard, which stopped
    // being true when a field started publishing under its owning entity —
    // `asset.pool.balance` carries no prefix and is out anyway. Preserving the
    // prefix while breaking the collection boundary would keep the sentence
    // true and the invariant lost, so the boundary is what this names.
    for (name, values) in &state_values {
        // A FIELD PUBLISHES UNDER THE THING THAT OWNS IT. `state.` names a
        // model-level state, and a field is not one — it is `asset.pool.balance`
        // in the model and reads the same in the results.
        let key = if name.matches('.').count() == 2 {
            name.clone()
        } else {
            format!("state.{name}")
        };
        series_map.insert(
            key,
            Series::from_plain(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &values[..periods.min(values.len())],
            ),
        );
    }
    // Subtotals, under their own `domain.` prefix. Money keeps a currency and
    // no offset — a fold spans streams that may settle at different points, so
    // there is no single placement to claim. Ratios are plain numbers, and
    // `null` where the denominator vanishes.
    for (id, values) in &subtotal_money {
        series_map.insert(
            id.clone(),
            Series::from_values(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &ir.model.currency,
                None,
                &values[..periods.min(values.len())],
            ),
        );
    }
    for (id, values) in &subtotal_ratio {
        series_map.insert(
            id.clone(),
            Series::from_optional(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &values[..periods.min(values.len())],
            ),
        );
    }
    series_map.insert(
        "model.net_cash_flow".to_string(),
        Series::from_values(
            &ir.time.calendar,
            &ir.time.start,
            periods as u32,
            &ir.model.currency,
            None,
            &model_series,
        ),
    );

    // ------------------------------------------------------------------
    // Per-entity cash, AGGREGATED BY RELATION rather than by string glob.
    //
    // A cross-stream read matches series by NAME (`series_sum("cre.rent.*")`),
    // which works only when the modeller encoded the hierarchy into the names.
    // The `part_of` relation says it directly, so a building's cash is its
    // units' cash because they are its units — not because someone prefixed
    // them consistently.
    //
    // AN ENTITY WITH NO CHILDREN IS UNAFFECTED: its series is its own streams,
    // which is the pool that models collective behavior directly. The grain
    // stays the modeller's choice.
    //
    // Like a subtotal, this is a fold OF the cash and never counts AS cash: it
    // is excluded from model.net_cash_flow, model.total and NPV, because
    // counting a parent and its children would double what it touches.
    // ------------------------------------------------------------------
    let mut entity_own: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    let mut add_owned = |symbol: &str, values: &[f64]| {
        let slot = entity_own
            .entry(symbol.to_string())
            .or_insert_with(|| vec![0.0; periods]);
        for (idx, value) in values.iter().enumerate().take(periods) {
            slot[idx] += value;
        }
    };
    for stream in &ir.streams {
        if !streams::is_cash(stream) {
            continue;
        }
        if let Some(values) = stream_series.get(&stream.name) {
            add_owned(&stream.owner.symbol, values);
        }
    }
    // A waterfall step's cash belongs to whoever it pays. Without this a
    // priority of payments would move money and no entity's total would show
    // it — the payee is named in the step and would have gone unread.
    for waterfall in &ir.waterfalls {
        for step in &waterfall.steps {
            if let Some(values) = waterfall_series.get(&format!("{}.{}", waterfall.name, step.name))
            {
                add_owned(&step.payee, values);
            }
        }
    }
    // An option is a contract, so its payoff belongs to the asset it is
    // written on — which is why options gained an owner.
    for option in &ir.options {
        if let (Some(owner), Some(values)) = (
            option.owner.as_ref(),
            stream_series.get(&format!("option.{}", option.name)),
        ) {
            add_owned(&owner.symbol, values);
        }
    }

    let parent_of: BTreeMap<&str, &str> = ir
        .entities
        .iter()
        .filter_map(|e| e.parent.as_deref().map(|p| (e.symbol.as_str(), p)))
        .collect();
    let mut entity_rollup: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for entity in &ir.entities {
        entity_rollup
            .entry(entity.symbol.clone())
            .or_insert_with(|| vec![0.0; periods]);
    }
    for (symbol, own) in &entity_own {
        // Walk from the owner up to the root, adding its cash to every
        // ancestor. `visited` bounds the walk even though a cycle is rejected
        // at compile time — this reads IR that may not have come from there.
        let mut visited: BTreeSet<&str> = BTreeSet::new();
        let mut cursor: Option<&str> = Some(symbol.as_str());
        while let Some(current) = cursor {
            if !visited.insert(current) {
                break;
            }
            let slot = entity_rollup
                .entry(current.to_string())
                .or_insert_with(|| vec![0.0; periods]);
            for (idx, value) in own.iter().enumerate().take(periods) {
                slot[idx] += value;
            }
            cursor = parent_of.get(current).copied();
        }
    }
    for (symbol, values) in &entity_rollup {
        series_map.insert(
            format!("entity.{symbol}.net_cash_flow"),
            Series::from_values(
                &ir.time.calendar,
                &ir.time.start,
                periods as u32,
                &ir.model.currency,
                None,
                values,
            ),
        );
    }

    let mut metrics = BTreeMap::new();
    for (stream_name, total) in stream_totals {
        metrics.insert(
            format!("stream.{stream_name}.total"),
            Scalar::Money(Money {
                amount: round_amount(total),
                currency: ir.model.currency.clone(),
            }),
        );
    }
    // Rolled up, so the lifetime total agrees with the series above rather
    // than disagreeing with it for any entity that has children.
    for (entity_symbol, values) in &entity_rollup {
        metrics.insert(
            format!("entity.{entity_symbol}.total"),
            Scalar::Money(Money {
                amount: round_amount(values.iter().sum::<f64>()),
                currency: ir.model.currency.clone(),
            }),
        );
    }

    let model_total = model_series.iter().sum::<f64>();
    let ppy = periods_per_year(&ir.time.calendar);
    let discount = Discount::from_config(ir, config, timeline, ppy)?;
    // The identity grain keeps the original path, byte for byte. Regrouping the
    // sum changes its last bit (measured at 1 ULP), and no published NPV should
    // move because a capability was added that nobody asked for yet.
    let npv = match config.valuation_grain.as_deref() {
        Some("annual") => {
            let grain = Grain::calendar_year(&timeline[..cash_periods.min(timeline.len())]);
            // One bucket is one year, so the rate for a bucket is the ANNUAL
            // rate — not the per-period rate the grid would use.
            discount.npv_at_grain(&valued_streams, &grain, timeline)
        }
        _ => discount.npv(&valued_streams),
    };
    metrics.insert(
        "model.total".to_string(),
        Scalar::Money(Money {
            amount: round_amount(model_total),
            currency: ir.model.currency.clone(),
        }),
    );
    // NO RATE, NO NPV (`docs/13` §7.46). A discounted figure whose rate nobody
    // stated is a missing term, not a shortcut: the zero it would discount at
    // is a real arithmetic answer to a question nobody asked, and a reader
    // scanning for a present value would take it for one. The rate is
    // published only when it was stated, for the same reason.
    if config.rate_stated {
        metrics.insert(
            "model.npv".to_string(),
            Scalar::Money(Money {
                amount: round_amount(npv),
                currency: ir.model.currency.clone(),
            }),
        );
    } else {
        warnings.push(
            "No discount rate was stated, so `model.npv` and `run.annual_discount_rate` are not \
             published: a present value needs a rate. State `annual_discount_rate` or \
             `annual_discount_curve` in the run configuration, or pass `--rate`."
                .to_string(),
        );
    }
    if let Some(pp_irr) = irr_with_offsets(&valued_streams) {
        let annual_irr = (1.0 + pp_irr).powf(ppy) - 1.0;
        metrics.insert(
            "model.irr".to_string(),
            Scalar::Number(round_amount(annual_irr)),
        );
    }
    // Engine-universal return metrics: MOIC, payback
    // period, WAL. Domain metrics live in pack metrics.toml files.
    //
    // WAL and payback are measured on the SAME TIME AXIS as discounting: a
    // flow's position is (period + offset), the exponent npv_with_offsets
    // uses. See docs/12_payment_timing.md. So an ordinary annuity's first
    // monthly collection is at 1/12 of a year, not 0 — which is the market
    // definition, and what a prospectus means by "the number of years from
    // the closing date to the related distribution date".
    //
    // Streams net only WITHIN an offset. Two flows in the same period at
    // different points in it are not the same cash at the same moment, so a
    // purchase settling on its date cannot cancel that period's collections.
    // Bucketing by offset and summing inside each bucket reduces exactly to
    // the old net-series computation whenever every stream shares an offset.
    let mut by_offset: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
    for (values, offset) in &valued_streams {
        // f64 is not Ord and these are exact fractions, so quantise to key.
        let key = (offset * 1e9).round() as i64;
        let bucket = by_offset
            .entry(key)
            .or_insert_with(|| vec![0.0; cash_periods]);
        for (idx, value) in values.iter().enumerate() {
            if idx < bucket.len() {
                bucket[idx] += *value;
            }
        }
    }
    // MOIC keeps the whole-model net series: it is a ratio of cash in to cash
    // out over the life, and where inside a period the cash sits does not
    // change how much of it there is. Only the time-weighted metrics below
    // need the offset, so they compute their own totals.
    let total_inflows: f64 = model_series.iter().filter(|v| **v > 0.0).sum();
    let total_outflows: f64 = -model_series.iter().filter(|v| **v < 0.0).sum::<f64>();
    if total_outflows > 0.0 && total_inflows > 0.0 {
        metrics.insert(
            "model.moic".to_string(),
            Scalar::Number(round_amount(total_inflows / total_outflows)),
        );
    }
    // Payback: the first INSTANT at which cumulative net cash flow becomes
    // non-negative, given the model starts cash-negative. Omitted otherwise.
    //
    // Instants, not periods: cash is ordered by (period + offset), so an
    // outlay settling on its date at period 0 precedes collections that fall
    // at the end of that same period. `payback_periods` stays a whole period
    // index, because that is what it names.
    if model_series.first().copied().unwrap_or(0.0) < 0.0 {
        let mut instants: Vec<(f64, usize, f64)> = Vec::new();
        for (key, values) in &by_offset {
            let offset = *key as f64 / 1e9;
            for (idx, value) in values.iter().enumerate() {
                if *value != 0.0 {
                    instants.push((idx as f64 + offset, idx, *value));
                }
            }
        }
        instants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        let mut cumulative = 0.0_f64;
        let mut payback: Option<(f64, usize)> = None;
        for (position, idx, value) in &instants {
            cumulative += *value;
            if cumulative >= 0.0 {
                payback = Some((*position, *idx));
                break;
            }
        }
        if let Some((position, period)) = payback {
            metrics.insert(
                "model.payback_periods".to_string(),
                Scalar::Number(period as f64),
            );
            metrics.insert(
                "model.payback_years".to_string(),
                Scalar::Number(round_amount(position / ppy)),
            );
        }
    }
    // WAL: net-inflow-weighted average life in years, on the discounting axis.
    let mut wal_weighted = 0.0_f64;
    let mut wal_inflows = 0.0_f64;
    for (key, values) in &by_offset {
        let offset = *key as f64 / 1e9;
        for (idx, value) in values.iter().enumerate() {
            if *value > 0.0 {
                wal_weighted += ((idx as f64 + offset) / ppy) * *value;
                wal_inflows += *value;
            }
        }
    }
    if wal_inflows > 0.0 {
        metrics.insert(
            "model.wal_years".to_string(),
            Scalar::Number(round_amount(wal_weighted / wal_inflows)),
        );
    }
    if config.rate_stated {
        match &config.discount_curve {
            Some(name) => metrics.insert(
                "run.annual_discount_curve".to_string(),
                Scalar::String(name.clone()),
            ),
            None => metrics.insert(
                "run.annual_discount_rate".to_string(),
                Scalar::Number(round_amount(config.discount_rate)),
            ),
        };
    }
    // Published for downstream metric evaluation (e.g. cfdl-metrics
    // `wal_years`, which needs to convert period indices to years).
    metrics.insert("run.periods_per_year".to_string(), Scalar::Number(ppy));
    if let Some(as_of) = &config.as_of {
        metrics.insert("run.as_of".to_string(), Scalar::String(as_of.to_string()));
    }

    let annual_rollup = if ir.time.calendar == "annual" {
        None
    } else {
        Some(build_annual_rollup(
            &timeline[..cash_periods],
            &stream_series,
            &model_series,
            &ir.model.currency,
            &subtotal_money,
            &ir.subtotals,
        ))
    };

    // A NAME THAT RESOLVED TO NOTHING IS FATAL, per docs/03 §2. Detected here,
    // once, rather than at each of the six evaluation sites: those return a
    // number deep inside a per-period loop, and the useful message names the
    // DISTINCT unresolved names rather than repeating one of them per period.
    //
    // `inputs.` cannot be checked at compile time — an input may be supplied
    // entirely by the run configuration, as `run_dists_full` does — so this is
    // the first layer that knows every source. `time.` is closed and is caught
    // earlier, by E1133.
    let mut declared: BTreeSet<String> = BTreeSet::new();
    for name in base_inputs.keys() {
        declared.insert(format!("inputs.{name}"));
    }
    if let Some(mc) = &config.monte_carlo {
        for name in mc.distributions.keys() {
            declared.insert(if name.starts_with("inputs.") {
                name.clone()
            } else {
                format!("inputs.{name}")
            });
        }
    }
    for scenario in config.scenarios.values() {
        for name in scenario.parameter_overrides.keys() {
            declared.insert(name.clone());
        }
    }
    // `prev.<account>` is DECLARED by the account statement and UNBOUND in
    // exactly one place: period 0, where the previous period does not exist.
    // Before the model began is not zero, it is unavailable — the read warns
    // and substitutes there, the same condition as an input declared only as
    // a distribution, and only a name nothing declares is fatal.
    for account in &ir.accounts {
        declared.insert(format!("prev.{}", account.name));
    }
    // DECLARED METRICS (`docs/13` §7.25), evaluated in the valuation plane.
    //
    // Last, and deliberately: every series has settled, every engine metric is
    // computed, and each metric can therefore read `model.npv` and the ones
    // declared above it. `metric.<name>` is a third namespace beside the
    // engine's `model.*` and a pack's `domain.*`, so a results file says who
    // minted every number in it.
    //
    // The same map the deterministic block publishes is what a scenario
    // summary carries, so a declared metric reaches every scenario column
    // without a second computation.
    // A PARTICIPANT'S REALISED RETURN (`docs/13` §7.72), folded per party.
    //
    // The vector is the party's own account, which is why this is computed
    // over ACCOUNTS and never over payee streams: a step's payee names who was
    // paid, but attribution through stream names is the trap §7.43 records.
    // An account's journal already separates the two directions — a
    // contribution is a NEGATIVE inflow (the capital call), a receipt is an
    // allocation in — so the sign change an IRR needs is recorded rather than
    // inferred.
    let mut party_returns: BTreeMap<String, cfdl_expr::ParticipantReturn> = BTreeMap::new();
    if !ir.metrics.is_empty() {
        let mut account_owner: BTreeMap<String, String> = BTreeMap::new();
        for account in &ir.accounts {
            if let Some(owner) = &account.owner {
                let party = owner.strip_prefix("party.").unwrap_or(owner).to_string();
                // One account per party is the rule the waterfall stage
                // already enforces; the first declaration keeps it.
                account_owner.entry(party).or_insert(account.name.clone());
            }
        }
        for (party, account) in &account_owner {
            let mut flows = vec![0.0; periods];
            for entry in &journal {
                if entry.period >= periods {
                    continue;
                }
                let touches = entry.target == *account
                    || entry.target.ends_with(&format!("account:{account}"));
                if !touches {
                    continue;
                }
                let amount = entry.amount.unwrap_or(0.0);
                match entry.action.as_str() {
                    "inflow" => flows[entry.period] += amount,
                    "allocate_in" => flows[entry.period] += amount,
                    "allocate_out" => flows[entry.period] -= amount,
                    _ => {}
                }
            }
            let received: f64 = flows.iter().filter(|v| **v > 0.0).sum();
            let contributed: f64 = -flows.iter().filter(|v| **v < 0.0).sum::<f64>();
            let mut entry = cfdl_expr::ParticipantReturn::default();
            if contributed <= 0.0 {
                entry.undefined_because =
                    Some(format!("party '{party}' never contributed to account '{account}', so there is no investment to return on"));
            } else if received <= 0.0 {
                entry.undefined_because = Some(format!(
                    "party '{party}' never received anything from account '{account}'"
                ));
            } else {
                entry.moic = Some(received / contributed);
                entry.irr = irr_with_offsets(&[(flows.clone(), 0.0)])
                    .map(|per_period| (1.0 + per_period).powf(ppy) - 1.0);
                if entry.irr.is_none() {
                    entry.undefined_because = Some(format!(
                        "party '{party}' has flows that do not solve for a rate"
                    ));
                }
            }
            party_returns.insert(party.clone(), entry);
        }
    }

    // SLICES (docs/13 §7.90): each declared selection, resolved against the
    // settled ledger. Kinds intersect, values within a kind union, excepts
    // subtract; an empty include-kind does not constrain, so a slice of
    // nothing but excepts reads "everything minus these". Matching delegates
    // to `cfdl_expr::selector_matches` — one dialect. A slice never carries
    // a reconciliation block: partial by design, and seen to be partial.
    let slice_results: Vec<SliceResult> = if ir.views.slices.is_empty() {
        Vec::new()
    } else {
        // parent chains, for entity clauses selecting descendants.
        let parent_of: BTreeMap<&str, &str> = ir
            .entities
            .iter()
            .filter_map(|e| e.parent.as_deref().map(|p| (e.symbol.as_str(), p)))
            .collect();
        let in_scope = |owner: &str, roots: &[String]| -> bool {
            let mut current = owner;
            for _ in 0..=parent_of.len() {
                if roots.iter().any(|r| r == current) {
                    return true;
                }
                match parent_of.get(current) {
                    Some(next) => current = next,
                    None => return false,
                }
            }
            false
        };
        // Streams, and the waterfall steps that pay for a contract: an
        // allocated line is paid by a step, so a selection by type and line
        // has to be able to reach one (docs/40 §6). A step carries no
        // category and belongs to the waterfall's entity.
        let step_names: Vec<(String, &str)> = ir
            .waterfalls
            .iter()
            .flat_map(|w| {
                w.steps
                    .iter()
                    .filter(|step| step.contract.is_some())
                    .map(move |step| (format!("{}.{}", w.name, step.name), w.entity.as_str()))
            })
            .collect();
        let mut stream_meta: BTreeMap<&str, (&str, Option<&str>)> = ir
            .streams
            .iter()
            .map(|st| {
                (
                    st.name.as_str(),
                    (st.owner.symbol.as_str(), st.category.as_deref()),
                )
            })
            .collect();
        for (name, owner) in &step_names {
            stream_meta.insert(name.as_str(), (owner, None));
        }
        ir.views
            .slices
            .iter()
            .map(|slice| {
                let mut matched: Vec<&str> = Vec::new();
                for (name, (owner, category)) in &stream_meta {
                    let entity_ok = slice.entities.is_empty() || in_scope(owner, &slice.entities);
                    // `type` and `line` were expanded by the compiler into
                    // exact names; either clause present means the list binds.
                    let type_ok = (slice.types.is_empty() && slice.lines.is_empty())
                        || slice.type_streams.iter().any(|t| t == name);
                    let cat_ok = slice.categories.is_empty()
                        || category
                            .is_some_and(|c| cfdl_expr::selector_matches_any(&slice.categories, c));
                    let name_ok = slice.streams.is_empty()
                        || cfdl_expr::selector_matches_any(&slice.streams, name);
                    if !(entity_ok && type_ok && cat_ok && name_ok) {
                        continue;
                    }
                    let dropped = cfdl_expr::selector_matches_any(&slice.except_streams, name)
                        || category.is_some_and(|c| {
                            cfdl_expr::selector_matches_any(&slice.except_categories, c)
                        })
                        || (!slice.except_entities.is_empty()
                            && in_scope(owner, &slice.except_entities));
                    if !dropped {
                        matched.push(name);
                    }
                }
                matched.sort_unstable();
                // THE WINDOW SELECTS PERIODS, the clauses above select streams.
                // A period outside it contributes nothing — to the net series,
                // and so to `total`, `npv` and `irr`, which are folds of it.
                // Dates rather than indices, compared the way `phase_at` does,
                // so a window means the same thing on any calendar.
                let in_window: Vec<bool> = match &slice.window {
                    None => vec![true; cash_periods],
                    Some(range) => {
                        let start = Date::parse(&range.start);
                        let end = Date::parse(&range.end);
                        (0..cash_periods)
                            .map(|t| match timeline.get(t) {
                                None => false,
                                Some(date) => {
                                    start.as_ref().is_ok_and(|s| date >= s)
                                        && end.as_ref().is_ok_and(|e| date <= e)
                                }
                            })
                            .collect()
                    }
                };
                let mut net = vec![0.0_f64; cash_periods];
                let mut valued: Vec<(Vec<f64>, f64)> = Vec::new();
                for name in &matched {
                    if let Some(values) = stream_series
                        .get(*name)
                        .or_else(|| waterfall_series.get(*name))
                    {
                        let cash: Vec<f64> = values[..cash_periods.min(values.len())]
                            .iter()
                            .enumerate()
                            .map(|(t, v)| if in_window[t] { *v } else { 0.0 })
                            .collect();
                        for (t, v) in cash.iter().enumerate() {
                            net[t] += *v;
                        }
                        valued.push((cash, stream_offsets.get(*name).copied().unwrap_or(1.0)));
                    }
                }
                let mut slice_metrics: BTreeMap<String, Scalar> = BTreeMap::new();
                slice_metrics.insert(
                    "total".to_string(),
                    Scalar::Money(Money {
                        amount: round_amount(net.iter().sum()),
                        currency: ir.model.currency.clone(),
                    }),
                );
                slice_metrics.insert(
                    "npv".to_string(),
                    Scalar::Money(Money {
                        amount: round_amount(discount.npv(&valued)),
                        currency: ir.model.currency.clone(),
                    }),
                );
                if let Some(pp_irr) = irr_with_offsets(&valued) {
                    slice_metrics.insert(
                        "irr".to_string(),
                        Scalar::Number(round_amount((1.0 + pp_irr).powf(ppy) - 1.0)),
                    );
                }
                let mut net_series = Series::from_values(
                    &ir.time.calendar,
                    &ir.time.start,
                    cash_periods as u32,
                    &ir.model.currency,
                    None,
                    &net,
                );
                net_series.entity = None;
                SliceResult {
                    id: slice.name.clone(),
                    selection: SliceSelection {
                        entities: slice.entities.clone(),
                        types: slice.types.clone(),
                        lines: slice.lines.clone(),
                        categories: slice.categories.clone(),
                        streams: slice.streams.clone(),
                        except_streams: slice.except_streams.clone(),
                        except_categories: slice.except_categories.clone(),
                        except_entities: slice.except_entities.clone(),
                        window: slice.window.as_ref().map(|r| SliceWindow {
                            from: r.start.clone(),
                            to: r.end.clone(),
                        }),
                    },
                    streams: matched.iter().map(|n| n.to_string()).collect(),
                    net: net_series,
                    metrics: slice_metrics,
                }
            })
            .collect()
    };

    let mut declared_metrics: BTreeMap<String, ExprValue> = BTreeMap::new();
    if !ir.metrics.is_empty() {
        let horizon = periods.saturating_sub(1);
        let date = timeline
            .get(horizon)
            .cloned()
            .unwrap_or_else(|| timeline[0].clone());
        // WHAT A METRIC CAN SEE (`docs/13` §7.85).
        //
        // The expression dialect first — a stream is `ops.rev`, a waterfall
        // step is `<waterfall>.<step>` — because that is what every existing
        // metric is written against and none of it may change meaning.
        let mut visible: BTreeMap<String, Vec<f64>> = stream_series.clone();
        for (name, values) in &waterfall_series {
            visible.insert(name.clone(), values.clone());
        }
        // THEN THE KEYS THE RUN ACTUALLY PUBLISHES, under the same names the
        // results document uses. Without this a metric could read a stream's
        // cash and nothing else the valuation plane computed: an aggregate
        // over an entity, a pack's subtotal, an account's balance, a field's
        // whole series — each published, each reading zero IN SILENCE, and
        // `stream.ops.rev` reading zero while the bare `ops.rev` beside it
        // read the same cash correctly.
        //
        // `docs/03` records the August 2026 ambiguity as an argument for
        // keeping the two dialects apart, and the measurement says the
        // opposite: the ambiguity IS one spelling working while the other
        // silently returns nothing. Binding both dissolves it, and only here —
        // a pot's window and a guard's read are untouched, because a metric
        // reads the FINISHED projection and they read the walk.
        for (name, values) in &stream_series {
            visible.insert(format!("stream.{name}"), values.clone());
        }
        for (name, values) in &waterfall_series {
            visible.insert(format!("stream.{name}"), values.clone());
        }
        for (symbol, values) in &entity_rollup {
            visible.insert(format!("entity.{symbol}.net_cash_flow"), values.clone());
        }
        for (name, values) in &account_balances {
            let cash = &values[..values.len().min(ir.time.periods as usize)];
            visible.insert(format!("account.{name}"), cash.to_vec());
        }
        for (name, values) in &state_values {
            // A field publishes under the thing that owns it, a model-level
            // state under `state.` — the same rule the results map applies.
            let key = if name.matches('.').count() == 2 {
                name.clone()
            } else {
                format!("state.{name}")
            };
            visible.insert(key, values[..periods.min(values.len())].to_vec());
        }
        for (id, values) in &subtotal_money {
            visible.insert(id.clone(), values[..periods.min(values.len())].to_vec());
        }
        // RATIO SUBTOTALS ARE DELIBERATELY NOT BOUND. A ratio has periods that
        // are genuinely undefined — a coverage ratio in a period with no debt
        // service — which is why it publishes `null` rather than zero, and a
        // fold over it needs a decision (skip the undefined periods, or refuse
        // the fold) that belongs with the reductions of §7.86. Leaving it
        // unbound is not the old behaviour: the check below refuses the name
        // outright, where before it read zero and said nothing.
        visible.insert("model.net_cash_flow".to_string(), model_series.clone());
        // A SLICE'S NET, under the key its results publish it by (docs/13
        // §7.90: a named selection that functions consume). A metric folding
        // `slice.all_debt` reads cash selected by TYPE and LINE — the
        // cross-pack reading of docs/40 stage 5 — without naming a stream.
        for slice in &slice_results {
            let net: Vec<f64> = slice
                .net
                .values
                .iter()
                .map(|v| match v {
                    SeriesValue::Money(m) => m.amount,
                    SeriesValue::Number(n) => *n,
                    _ => 0.0,
                })
                .collect();
            visible.insert(format!("slice.{}", slice.id), net);
        }
        let shared = Arc::new(visible);
        for metric in &ir.metrics {
            let mut env = build_expr_env(ir, None, config, horizon, &date, &base_inputs);
            env.series = Arc::clone(&shared);
            // ENTITY FIELDS, AT THE HORIZON (`docs/13` §7.85). `docs/01` §15.3
            // has promised these in normative text since metrics entered the
            // spec, and the binding was simply absent: `bind_states` is called
            // for streams, distributions and state evaluation, and was never
            // called here, so `asset.proj.drawn` in a metric was
            // EXPR_UNKNOWN_NAME. A metric is a fold over the finished
            // projection, so the horizon is the period it reads — the field's
            // whole series is reachable beside it, under the key the results
            // document publishes it under.
            bind_states(&mut env, &state_values, horizon);
            env.party_returns = party_returns.clone();
            // A metric is a fold over the FINISHED projection, so every period
            // is readable — including the tail, which is what a forward-looking
            // figure needs.
            env.series_available_to = Some(timeline.len().saturating_sub(1));
            // The engine's own metrics, so a declared one can build on them.
            for (key, value) in &metrics {
                if let Some(rest) = key.strip_prefix("model.") {
                    let bound = match value {
                        Scalar::Number(v) => ExprValue::Decimal(*v),
                        Scalar::Money(m) => ExprValue::Money(cfdl_expr::Money {
                            amount: m.amount,
                            currency: m.currency.clone(),
                        }),
                        Scalar::String(v) => ExprValue::String(v.clone()),
                        Scalar::Null => ExprValue::Optional(None),
                    };
                    env.model.insert(rest.to_string(), bound);
                }
            }
            for (name, value) in &declared_metrics {
                env.metrics.insert(name.clone(), value.clone());
            }
            let compiled = match cfdl_expr::compile_expr(&metric.expr.src) {
                Ok(compiled) => compiled,
                Err(err) => {
                    return Err(EngineError::UnknownName(format!(
                        "Metric '{}' does not compile: {err}",
                        metric.name
                    )));
                }
            };
            // A NAME NOTHING BINDS IS REFUSED, NOT READ AS ZERO
            // (`docs/13` §7.85).
            //
            // `series_sum`/`series_avg` return 0.0 for a selector that matches
            // nothing, which is right for a `.*` selector — matching nothing is
            // a stated possibility there — and wrong for a spelled-out name. In
            // a stream that miss is a warning (`W5022`), because the series it
            // produces is there to be looked at. In a metric there is nothing
            // to look at: a fold publishes ONE number, under a name the author
            // chose, and a wrong one is indistinguishable from a right one.
            //
            // The engine's own stance is that a metric which fails to evaluate
            // is fatal, "because a missing key reads as 'not run' rather than
            // 'not defined'". A metric that evaluates against a name nothing
            // binds is the same failure, one step earlier.
            for referenced in cfdl_expr::series_references(&metric.expr.src) {
                if referenced.ends_with(".*") || shared.contains_key(&referenced) {
                    continue;
                }
                return Err(EngineError::UnknownName(format!(
                    "Metric '{}' names series '{}', which this run does not \
                     publish. It would fold to zero, and a metric is one number \
                     with no series beside it to show the zero. Check the \
                     spelling; a selector ending in `.*` states that matching \
                     nothing is intended.",
                    metric.name, referenced
                )));
            }
            match cfdl_expr::eval(&compiled, &env) {
                Ok(value) => {
                    declared_metrics.insert(metric.name.clone(), value.clone());
                    let published = match &value {
                        ExprValue::Money(m) => Scalar::Money(Money {
                            amount: round_amount(m.amount),
                            currency: m.currency.clone(),
                        }),
                        ExprValue::Decimal(v) => Scalar::Number(round_amount(*v)),
                        ExprValue::Int(v) => Scalar::Number(*v as f64),
                        ExprValue::Bool(v) => Scalar::String(v.to_string()),
                        ExprValue::Date(d) => {
                            Scalar::String(format!("{:04}-{:02}-{:02}", d.year, d.month, d.day))
                        }
                        ExprValue::String(v) => Scalar::String(v.clone()),
                        // NULL PUBLISHES AS NULL, not as the text "null". A
                        // metric that folded a selection with nothing in it has
                        // no answer, and the results schema has always
                        // permitted a null scalar; the catch-all below would
                        // have made that absence look like a value of type
                        // text (`docs/13` §7.86).
                        ExprValue::Optional(None) => Scalar::Null,
                        other => Scalar::String(describe_value(other)),
                    };
                    metrics.insert(format!("metric.{}", metric.name), published);
                }
                Err(err) => {
                    // A metric that cannot evaluate is not a metric worth
                    // publishing as zero, or omitting: the case exists to
                    // assert this number, and a missing key reads as "not run"
                    // rather than "not defined".
                    //
                    // A participant's return knows WHY it is undefined, so say
                    // it here rather than leave the author with a hook that
                    // returned nothing.
                    let mut why = party_returns
                        .iter()
                        .find(|(party, found)| {
                            found.undefined_because.is_some() && err.message.contains(*party)
                        })
                        .and_then(|(_, found)| found.undefined_because.clone());
                    // A party with no account has no entry at all, and the
                    // evaluator's "not available here" is the wrong reason —
                    // the call IS in a metric. Name the real one.
                    if why.is_none() {
                        for call in ["irr(\"", "moic(\""] {
                            let mut rest = metric.expr.src.as_str();
                            while let Some(at) = rest.find(call) {
                                rest = &rest[at + call.len()..];
                                let Some(end) = rest.find('"') else { break };
                                let named = &rest[..end];
                                let party = named.strip_prefix("party.").unwrap_or(named);
                                if !party_returns.contains_key(party) {
                                    why = Some(format!(
                                        "party '{party}' owns no account, and a participant's return is folded over the party's own account — declare one with `account <name> {{ owner party.{party} … }}` and pay the waterfall's steps into it"
                                    ));
                                    break;
                                }
                            }
                            if why.is_some() {
                                break;
                            }
                        }
                    }
                    return Err(EngineError::UnknownName(match why {
                        Some(why) => {
                            format!("Metric '{}' could not be evaluated: {why}.", metric.name)
                        }
                        None => format!("Metric '{}' could not be evaluated: {err}", metric.name),
                    }));
                }
            }
        }
    }

    // A CURVE READ OUTSIDE ITS EFFECTIVE DATES IS FATAL (`docs/13` §7.100).
    // The curve declared where it stops; a read past that has no value, and
    // the reader that made it is named rather than paid its end value.
    let curve_reads: Vec<String> = {
        // One line per (reader, curve): the walk is chronological, so the
        // first marker names the first date the read ran off the end.
        let mut seen: BTreeSet<String> = BTreeSet::new();
        warnings
            .iter()
            .filter_map(|w| w.strip_prefix(env::CURVE_OUTSIDE_RANGE_MARKER))
            .filter(|w| {
                let key = w.split(" has no value at ").next().unwrap_or(w);
                seen.insert(key.to_string())
            })
            .map(|w| w.to_string())
            .collect()
    };
    if !curve_reads.is_empty() {
        return Err(EngineError::CurveReadOutsideRange(format!(
            "{} — outside its effective dates a curve has no value. End the reader's \
             schedule where the curve ends, or extend the curve's dates.",
            curve_reads.join("; ")
        )));
    }

    // A FIELD WHOSE RULE FAILED IS FATAL (`docs/13` §7.103). The walk marks
    // each failure with the field, the clause and the period; the run refuses
    // here, once, naming every distinct failure rather than substituting zero
    // under a warning.
    let field_failures: Vec<String> = {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        warnings
            .iter()
            .filter_map(|w| w.strip_prefix("FIELD_EVALUATION_FAILED: "))
            .filter(|w| seen.insert(w.to_string()))
            .map(|w| w.to_string())
            .collect()
    };
    if !field_failures.is_empty() {
        return Err(EngineError::FieldEvaluationFailed(format!(
            "{} — a value that was never computed is not a number. Guard the expression \
             for the period it fails in, or correct it.",
            field_failures.join("; ")
        )));
    }

    let unresolved = unresolved_names(&warnings, &declared);
    if !unresolved.is_empty() {
        // DECLARED BUT UNRESOLVED IS NOT "NOT DECLARED" (`docs/13` §7.68). An
        // assumption the model states in plain sight may have failed to
        // produce a number; the warning that says why is already in the
        // array, and the refusal should point at it rather than at a
        // declaration the author will look for and find.
        let mut described: Vec<String> = Vec::new();
        for name in &unresolved {
            let assumed = name
                .strip_prefix("inputs.")
                .filter(|n| {
                    ir.assumptions.constants.contains_key(*n)
                        || ir.assumptions.random.contains_key(*n)
                })
                .map(|n| n.to_string());
            match assumed {
                Some(short) => {
                    let why = warnings
                        .iter()
                        .find(|w| w.starts_with(&format!("Assumption '{short}' ")))
                        .cloned()
                        .unwrap_or_else(|| "it produced no number".to_string());
                    described.push(format!(
                        "`{name}` is declared as `assume {short}` but did not produce a number ({why})"
                    ));
                }
                None => described.push(format!(
                    "`{name}` is not declared — declare it, supply it in the run configuration, or correct the name"
                )),
            }
        }
        return Err(EngineError::UnknownName(format!(
            "{} — each read as zero.",
            described.join("; ")
        )));
    }

    Ok(DeterministicRunOutput {
        slices: slice_results,
        journal,
        warnings,
        resolved_inputs: base_inputs,
        metrics,
        series: series_map,
        npv,
        annual_rollup,
        transitions,
    })
}

/// The contract a pack-lowered stream came from, read off its provenance
/// (`generated_by.contract`). `None` for a hand-written stream.
/// What a stream series is attributed to: its owner, its category, the
/// contract it was lowered from, and its line by role.
pub(crate) type StreamAttribution<'a> =
    (&'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>);

/// The line a pack-lowered stream is, by role, read off its provenance
/// (`generated_by.line`). `None` for a hand-written stream.
pub(crate) fn lowering_line(stream: &ir::IrStream) -> Option<&str> {
    stream
        .provenance
        .as_ref()?
        .generated_by
        .as_ref()?
        .get("line")?
        .as_str()
}

pub(crate) fn lowering_contract(stream: &ir::IrStream) -> Option<&str> {
    stream
        .provenance
        .as_ref()?
        .generated_by
        .as_ref()?
        .get("contract")?
        .as_str()
}

#[cfg(test)]
mod unresolved_name_tests {
    use super::unresolved_names;
    use std::collections::BTreeSet;

    fn warn(name: &str) -> String {
        format!(
            "Stream 'x' amount evaluation failed [{}]: unknown variable `{name}`; using 0.",
            cfdl_expr::EXPR_UNKNOWN_NAME
        )
    }

    #[test]
    fn a_name_nothing_declares_is_fatal() {
        let found = unresolved_names(&[warn("inputs.typo")], &BTreeSet::new());
        assert_eq!(found, vec!["`inputs.typo` is not declared".to_string()]);
    }

    #[test]
    fn a_declared_name_merely_unbound_here_is_not() {
        // An input declared only as a Monte Carlo distribution is unbound in
        // the deterministic pass. `run_dists_full` is that model, and its
        // deterministic run is incidental to the trials it exercises.
        let declared: BTreeSet<String> = ["inputs.n".to_string()].into_iter().collect();
        assert!(unresolved_names(&[warn("inputs.n")], &declared).is_empty());
    }

    #[test]
    fn one_entry_per_distinct_name_however_many_periods() {
        let warnings = vec![warn("inputs.a"), warn("inputs.a"), warn("inputs.b")];
        assert_eq!(unresolved_names(&warnings, &BTreeSet::new()).len(), 2);
    }

    #[test]
    fn an_ordinary_evaluation_failure_is_left_alone() {
        let w = "Stream 'x' amount evaluation failed [EXPR_EVAL]: division by zero; using 0.";
        assert!(unresolved_names(&[w.to_string()], &BTreeSet::new()).is_empty());
    }
}
