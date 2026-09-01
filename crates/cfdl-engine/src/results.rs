// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

/// Aggregate per-period series values by calendar year.
///
/// Each distinct calendar year present in `timeline` becomes one entry in the
/// output `Series`.  Values for all periods that fall within a given year
/// are summed.  The resulting index uses `calendar = "annual"` and `start =
/// "{first_year}-01-01"`.
/// A bucketing of the model grid into report periods.
///
/// "Grain" rather than a coined term: it is what analytics tooling already
/// calls this — Superset and Looker both offer a Time Grain of
/// day/week/month/quarter/year, meaning exactly "what one row represents".
/// Not to be confused with `pack.cadences`, which is a different thing wearing
/// a similar word: the list of model calendars a pack's rules lower correctly
/// on, rather than a frequency to aggregate at.
///
/// The model grain and the annual rollup were always two bucketings of one
/// mechanism; only one of them was written down. Making it a type means a
/// quarterly statement, an annual rollup and a valuation at a different
/// convention are the same operation with a different partition, rather than
/// three pieces of code that must be kept agreeing.
///
/// `buckets[i]` holds the model-period indices that fall in report period `i`.
/// The identity bucketing — one model period per bucket — is what everything
/// defaults to, which is why nothing moves until something opts in.
#[derive(Debug, Clone)]
pub struct Grain {
    pub calendar: String,
    pub start: String,
    pub buckets: Vec<Vec<usize>>,
    /// One label per bucket, built HERE because this is the last place the
    /// dates exist. A statement is a post-pass with only a `SeriesIndex`, and a
    /// coarse grain's buckets are opaque indices — nothing downstream can say
    /// which year bucket 3 is without rebuilding the timeline again.
    pub labels: Vec<String>,
}

/// Format one bucket's opening date for the calendar it is bucketed at.
pub(crate) fn bucket_label(date: &Date, calendar: &str) -> String {
    match calendar {
        "annual" => format!("{:04}", date.year),
        "quarterly" => format!("{:04}-Q{}", date.year, (date.month - 1) / 3 + 1),
        "daily" => format!("{:04}-{:02}-{:02}", date.year, date.month, date.day),
        // monthly, and anything unrecognized: a year-month is never wrong,
        // only less precise than it could be.
        _ => format!("{:04}-{:02}", date.year, date.month),
    }
}

impl Grain {
    /// One bucket per model period: the grid reporting on itself.
    pub fn identity(timeline: &[Date], calendar: &str, start: &str) -> Self {
        Self {
            calendar: calendar.to_string(),
            start: start.to_string(),
            buckets: (0..timeline.len()).map(|i| vec![i]).collect(),
            labels: timeline.iter().map(|d| bucket_label(d, calendar)).collect(),
        }
    }

    /// One bucket per distinct CALENDAR year — not per model year. A mid-year
    /// start therefore produces a short first bucket, which is what the annual
    /// rollup has always done and what a fiscal reader expects.
    pub fn calendar_year(timeline: &[Date]) -> Self {
        let mut years: Vec<i32> = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for d in timeline {
            if seen.insert(d.year) {
                years.push(d.year);
            }
        }
        let buckets = years
            .iter()
            .map(|&yr| {
                timeline
                    .iter()
                    .enumerate()
                    .filter_map(|(i, d)| (d.year == yr).then_some(i))
                    .collect()
            })
            .collect();
        Self {
            calendar: "annual".to_string(),
            start: years
                .first()
                .map(|y| format!("{y:04}-01-01"))
                .unwrap_or_default(),
            buckets,
            labels: years.iter().map(|y| format!("{y:04}")).collect(),
        }
    }

    /// Build a grain from what a results document already carries.
    ///
    /// A statement is a post-pass and has no timeline — only a `SeriesIndex`.
    /// Reconstructing the dates from `(calendar, start, periods)` is exact,
    /// because that triple is what generated them in the first place.
    ///
    /// `name` is the grain a declaration asked for. `None` or `"period"` gives
    /// the identity bucketing, which is what everything defaults to.
    pub fn from_index(index: &SeriesIndex, name: Option<&str>) -> Self {
        let timeline = timeline_dates(&index.start, &index.calendar, index.periods as usize)
            .unwrap_or_default();
        match name {
            Some("annual") if !timeline.is_empty() => Grain::calendar_year(&timeline),
            _ => Grain::identity(&timeline, &index.calendar, &index.start),
        }
    }

    pub fn is_identity(&self) -> bool {
        self.buckets.iter().all(|b| b.len() <= 1)
    }

    /// Sum a per-period series into this grain's buckets.
    ///
    /// Money buckets by summation. A RATIO does not, and must never be routed
    /// through here: the mean of twelve monthly coverage ratios is not the
    /// annual coverage ratio. A ratio is recomputed from its re-bucketed
    /// numerator and denominator — see `rebucket_subtotals`, whose signature
    /// takes the SPECS rather than the values so that computing the wrong
    /// thing is not the path of least resistance.
    pub fn sum(&self, values: &[f64]) -> Vec<f64> {
        self.buckets
            .iter()
            .map(|b| b.iter().filter_map(|&i| values.get(i)).sum())
            .collect()
    }
}

pub(crate) fn build_annual_rollup(
    timeline: &[Date],
    stream_series: &BTreeMap<String, Vec<f64>>,
    model_series: &[f64],
    currency: &str,
    subtotal_money: &BTreeMap<String, Vec<f64>>,
    subtotal_specs: &[IrSubtotal],
) -> AnnualRollupSection {
    // One caller of Grain rather than its own bucketing. The rollup and a
    // coarser statement ask the same question of the same partition; keeping
    // two implementations of that meant keeping them agreeing.
    //
    // The grain is constructed HERE rather than passed, and the function stays
    // `build_annual_rollup` rather than becoming a general `build_rollup`. It
    // returns `AnnualRollupSection`, and the published schema pins
    // `deterministic.annual_rollup` to `calendar: "annual"` — so a version that
    // accepted any grain could emit quarterly data under a field called
    // `annual_rollup`, which is a worse trade than one saved constructor call.
    //
    // Generality belongs where the grain genuinely varies per output: the
    // statement and valuation paths. If the rollup ever becomes "at whatever
    // grain you asked for", the field name and the schema move with it, and
    // that is a deliberate contract change rather than a rename.
    let grain = Grain::calendar_year(timeline);
    let n_years = grain.buckets.len() as u32;
    let start = grain.start.clone();
    let aggregate = |values: &[f64]| -> Vec<f64> { grain.sum(values) };

    let mut rollup = BTreeMap::new();

    rollup.insert(
        "model.net_cash_flow".to_string(),
        Series::from_values(
            "annual",
            &start,
            n_years,
            currency,
            None,
            &aggregate(model_series),
        ),
    );

    for (name, values) in stream_series {
        rollup.insert(
            format!("stream.{name}"),
            Series::from_values(
                "annual",
                &start,
                n_years,
                currency,
                None,
                &aggregate(values),
            ),
        );
    }

    // Subtotals roll up BY KIND, and that distinction is the whole reason this
    // takes the specs rather than the two value maps.
    //
    // Money folds. A ratio does not: the mean of twelve monthly coverage ratios
    // is not the annual coverage ratio, and the annual ratio is not recoverable
    // from the monthly column at all. So it is recomputed from its numerator and
    // denominator AFTER those have been rolled up — which is only possible
    // because the declaration says what they are.
    //
    // Deliberately keyed off `subtotal_money` for the inputs rather than off the
    // published ratio series, for the same reason cfdl-statement takes specs:
    // given a column of ratios and a grain, averaging them is the obvious thing
    // to write, and it is wrong.
    for (id, values) in subtotal_money {
        rollup.insert(
            id.clone(),
            Series::from_values(
                "annual",
                &start,
                n_years,
                currency,
                None,
                &aggregate(values),
            ),
        );
    }
    for spec in subtotal_specs {
        if spec.op != "ratio" {
            continue;
        }
        let (Some(num_id), Some(den_id)) = (&spec.numerator, &spec.denominator) else {
            continue;
        };
        let (Some(num), Some(den)) = (subtotal_money.get(num_id), subtotal_money.get(den_id))
        else {
            continue;
        };
        let (num, den) = (aggregate(num), aggregate(den));
        let values: Vec<Option<f64>> = num
            .iter()
            .zip(den.iter())
            .map(|(n, d)| (d.abs() > f64::EPSILON).then(|| round_amount(n / d)))
            .collect();
        rollup.insert(
            spec.id.clone(),
            Series::from_optional("annual", &start, n_years, &values),
        );
    }

    AnnualRollupSection { series: rollup }
}

/// Present value of streams that each carry their own position in period.
///
/// `v / (1+r)^(t + offset)` factorizes to `[v / (1+r)^offset] / (1+r)^t`, so a
/// stream's offset is a constant scale on its whole series.
pub(crate) fn npv_with_offsets(streams: &[(Vec<f64>, f64)], rate: f64) -> f64 {
    let mut total = 0.0_f64;
    for (values, offset) in streams {
        let scale = (1.0 + rate).powf(-offset);
        for (i, value) in values.iter().enumerate() {
            total += value * scale / (1.0 + rate).powi(i as i32);
        }
    }
    total
}

/// Present value at a stated GRAIN: sum the cash into the grain's buckets
/// first, then discount each bucket once.
///
/// This is the order practitioners use — sum NOI by year, then discount the
/// year — and the order matters. `npv_with_offsets` above discounts each
/// stream-period individually and accumulates, which is the same answer only
/// when the grain IS the model grid.
///
/// Grouping is by `(bucket, offset)`, not by bucket alone. A discount factor
/// depends only on position and offset, so summing within a `(bucket, offset)`
/// group and discounting once is MATHEMATICALLY equal to the per-stream
/// accumulation at model grain, including for models whose streams settle at
/// different points in a period. Collapsing the offset dimension would change
/// every mixed-offset model, which is why it is not collapsed.
///
/// Mathematically equal is not bit-equal: float addition is not associative,
/// and regrouping the sum moves the last bit — measured at 1 ULP on a mixed-
/// offset probe. So the identity grain does NOT route through here. The default
/// path stays `npv_with_offsets` exactly as it was, and this function serves
/// callers that ask for a different grain. That keeps the promise that nothing
/// moves until something opts in, rather than re-blessing every NPV in the
/// golden suite for a change of summation order.
///
/// At a coarser grain the sub-bucket offsets do collapse, which is exactly what
/// an annual convention asserts: MIT OCW 11.431J's own footnote says "assumes
/// first cash flow occurs 1 year from present".
pub(crate) fn npv_at_grain(
    streams: &[(Vec<f64>, f64)],
    rate_per_bucket: f64,
    grain: &Grain,
) -> f64 {
    // (bucket index, quantised offset) -> summed cash. The quantisation mirrors
    // `by_offset` used for WAL and payback, so one convention describes both.
    let mut grouped: BTreeMap<(usize, i64), f64> = BTreeMap::new();
    for (values, offset) in streams {
        let key_offset = (offset * 1e9).round() as i64;
        for (bucket_idx, members) in grain.buckets.iter().enumerate() {
            let mut sum = 0.0_f64;
            for &i in members {
                if let Some(v) = values.get(i) {
                    sum += *v;
                }
            }
            if sum != 0.0 {
                *grouped.entry((bucket_idx, key_offset)).or_insert(0.0) += sum;
            }
        }
    }
    let mut total = 0.0_f64;
    for ((bucket_idx, key_offset), sum) in grouped {
        let offset = key_offset as f64 / 1e9;
        total += sum / (1.0 + rate_per_bucket).powf(bucket_idx as f64 + offset);
    }
    total
}

/// IRR over offset-carrying streams: the rate at which their present value is
/// zero. Bisection, because the basis is rebuilt for each candidate rate.
pub(crate) fn irr_with_offsets(streams: &[(Vec<f64>, f64)]) -> Option<f64> {
    // AN IRR NEEDS SOMETHING TO BE A RETURN ON. Two shapes qualify:
    //
    //   an INVESTMENT — the first cash moved is out. Interim sign changes are
    //   ordinary in a real asset (lease-up, capex, TI and commissions), so a
    //   strict Descartes test would throw away figures practitioners quote: a
    //   two-tenant office DCF here changes sign seven times and its IRR is
    //   exactly the number the case exists to report.
    //
    //   a FINANCING — cash in, then out, and nothing further. Its rate is the
    //   cost of funds: a 5.5% nominal mortgage compounding monthly returns
    //   0.0564, which is (1 + 0.055/12)^12 - 1 and worth having.
    //
    // What is left over is a series that neither invests nor borrows — a rent
    // line alternating with a sweep, which is a schedule demonstration and not a
    // deal. A root still exists near the bracket floor and the bisection will
    // find it; on a daily calendar it annualizes to exactly -1.0, and a number
    // that precise is read as a finding rather than as an artifact.
    //
    // Counted on the flows as VALUED, bucketed by position in time, because that
    // is the series the solve below is run against. Streams net only within an
    // offset (docs/12), so netting the periods first would describe a different
    // series: two goldens have an all-zero `model.net_cash_flow` and a non-empty
    // valued series.
    //
    // Without this an alternating series still converges — the bisection finds a
    // root near the bracket floor and reports it, which on a daily calendar
    // annualizes to exactly -1.0. A number that precise is read as a finding.
    let mut buckets: BTreeMap<i64, f64> = BTreeMap::new();
    for (values, offset) in streams {
        for (i, value) in values.iter().enumerate() {
            let position = ((i as f64 + offset) * 1e9).round() as i64;
            *buckets.entry(position).or_insert(0.0) += *value;
        }
    }
    let mut sign_changes = 0usize;
    let mut previous = 0.0_f64;
    let mut first = 0.0_f64;
    for amount in buckets.values() {
        if amount.abs() <= 1e-9 {
            continue;
        }
        if first == 0.0 {
            first = *amount;
        }
        if previous != 0.0 && previous * amount < 0.0 {
            sign_changes += 1;
        }
        previous = *amount;
    }
    let invests = first < 0.0;
    let finances = first > 0.0 && sign_changes == 1;
    if !invests && !finances {
        return None;
    }

    let f = |r: f64| npv_with_offsets(streams, r);
    let mut hi = 10.0_f64;
    let f_hi = f(hi);
    // The lower bracket cannot simply be -0.9999. `npv_with_offsets` divides by
    // (1 + r)^i, and at r = -0.9999 that is 1e-4^i, which underflows to zero
    // once i passes ~81. The inflows then evaluate to +inf and the outflows to
    // -inf, their sum is NaN, and the sign test below rejects a perfectly
    // ordinary cash flow. Every model longer than about 82 periods — which is
    // most development deals on a monthly grid — lost its IRR that way.
    //
    // So walk inward until the present value is finite. The first candidate is
    // the original bound, so nothing moves for series short enough to evaluate
    // there; only the models that used to return None are affected.
    let mut lo = f64::NAN;
    let mut f_lo = f64::NAN;
    for candidate in [
        -0.9999_f64,
        -0.999,
        -0.99,
        -0.95,
        -0.9,
        -0.8,
        -0.6,
        -0.4,
        -0.2,
    ] {
        let value = f(candidate);
        if value.is_finite() {
            lo = candidate;
            f_lo = value;
            break;
        }
    }
    if f_lo.is_nan() || f_hi.is_nan() || f_lo * f_hi > 0.0 {
        return None;
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let f_mid = f(mid);
        if f_mid.abs() < 1e-10 {
            return Some(mid);
        }
        if f_lo * f_mid < 0.0 {
            hi = mid;
        } else {
            lo = mid;
            f_lo = f_mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Advance a date by one interval.
pub(crate) fn step_once(d: &Date, interval: &str) -> Date {
    match interval {
        "daily" => d.add_days(1),
        "weekly" => d.add_days(7),
        "quarterly" => d.add_months(3),
        "annual" => d.add_months(12),
        _ => d.add_months(1),
    }
}

pub(crate) fn round_amount(value: f64) -> f64 {
    // Single global rounding policy for deterministic numeric outputs.
    (value * 1_000_000.0).round() / 1_000_000.0
}

pub(crate) fn canonical_hash(value: &Value) -> String {
    let canonical = canonical_json(value);
    let digest = Sha256::digest(canonical.as_bytes());
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

pub(crate) fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(v) => v.to_string(),
        Value::String(v) => serde_json::to_string(v).unwrap_or_else(|_| "\"\"".to_string()),
        Value::Array(values) => {
            let inner = values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{inner}]")
        }
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut parts = Vec::with_capacity(keys.len());
            for key in keys {
                let key_json = serde_json::to_string(&key).unwrap_or_else(|_| "\"\"".to_string());
                let value_json = canonical_json(&map[&key]);
                parts.push(format!("{key_json}:{value_json}"));
            }
            format!("{{{}}}", parts.join(","))
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainMetrics {
    pub pack: String,
    pub metrics: BTreeMap<String, Scalar>,
    pub lineage: BTreeMap<String, MetricLineage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricLineage {
    pub numerator_streams: Vec<String>,
    pub denominator_streams: Vec<String>,
    pub formula: String,
}

#[derive(Debug, Serialize)]
pub struct Results {
    pub results_version: String,
    pub model_hash: String,
    /// Content hash of the deterministic ledger — the per-stream, per-period
    /// series this run produced.
    ///
    /// Together with `model_hash`, `engine` and the run config in
    /// `deterministic.metrics`, this closes the chain: identical inputs on an
    /// identical engine must reproduce an identical `ledger_hash`. If they do
    /// not, something is nondeterministic, and the golden suite would otherwise
    /// report that as a flapping test rather than as the defect it is.
    ///
    /// It hashes the LEDGER, not the inputs, deliberately. "Did the inputs
    /// change" is already answerable from `model_hash`; what nothing answered
    /// before is "did the output change", which is the question a reviewer
    /// staring at a re-blessed golden actually has.
    pub ledger_hash: String,
    pub engine: EngineInfo,
    pub warnings: Vec<String>,
    /// Resolved assumptions and the contract terms each lowered stream
    /// consumed. Absent when the model declares neither.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<InputsSection>,
    pub deterministic: DeterministicSection,
    pub scenarios: ScenarioSection,
    pub monte_carlo: MonteCarloSection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_metrics: Option<DomainMetrics>,
    /// Rendered statements. Present only when the active pack declares one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statements: Option<StatementsSection>,
    /// The model's entity graph (docs/13 §7.43, §7.91). Absent only for a
    /// model declaring no entities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph: Option<ResultsGraph>,
    /// Declared slices and what each came to (docs/13 §7.90). Absent when
    /// the model declares none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slices: Option<Vec<SliceResult>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementsSection {
    pub pack: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Statement {
    pub id: String,
    pub label: String,
    pub default: bool,
    /// The grain this statement reports at, and the period labels that go with
    /// it. Published because a consumer CANNOT derive it: an annual statement
    /// over a monthly model has ten values where the model has 120, and nothing
    /// else in the document says which ten periods those are. The playground
    /// needs it to label a column; so does anyone rendering the JSON.
    pub grain: StatementGrain,
    pub rows: Vec<StatementRow>,
    pub reconciliation: StatementReconciliation,
    /// Completeness findings. Empty is the healthy case.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<StatementDiagnostic>,
}

/// How a statement's columns are bucketed, and what to call them.
#[derive(Debug, Clone, Serialize)]
pub struct StatementGrain {
    /// `monthly` | `quarterly` | `annual` | whatever the model grid is.
    pub calendar: String,
    /// First bucket's start date.
    pub start: String,
    /// One label per column, ready to render.
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementRow {
    /// `line` | `subtotal` | `ratio` | `spacer` | `residual`.
    pub kind: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub label: String,
    pub depth: u32,
    /// How to RENDER the sign: +1 shows the value as stored, -1 flips it for
    /// display only. `values` is always the signed arithmetic quantity, so a
    /// consumer that ignores this still adds up correctly.
    pub display_sign: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<SeriesValue>,
    /// Lifetime total of the row. Absent for a ratio, where summing means
    /// nothing, and for a spacer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    /// The streams this row drew from. Present on `line` and `residual` rows;
    /// it is what makes a published figure traceable without the ledger.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub streams: Vec<String>,
}

/// Does the statement add up to the model's cash?
///
/// Published always and asserted rather than corrected. A statement whose
/// bottom line quietly differs from `model.total` is the failure this exists to
/// make visible.
#[derive(Debug, Clone, Serialize)]
pub struct StatementReconciliation {
    pub bottom_line: f64,
    pub model_total: f64,
    pub residual: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatementDiagnostic {
    pub code: String,
    pub message: String,
}

/// The top of the audit chain: what went in, above the line items.
#[derive(Debug, Clone, Serialize)]
pub struct InputsSection {
    /// Evaluated `assume` values, as `inputs.<name>` resolves them.
    ///
    /// In a deterministic run a random assumption resolves to its clipped
    /// central value, not to a draw — publishing it here is what stops that
    /// being invisible.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub resolved: BTreeMap<String, f64>,
    /// Per-stream record of the contract terms a pack rule consumed. Passed
    /// through from the IR verbatim, so `IrStream` and the per-period
    /// evaluation path are untouched by it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub streams: Vec<serde_json::Value>,
    /// Which slice of a declared quantile each expression asked for, and what
    /// it resolved to.
    ///
    /// A nonlinear input whose evaluation is not published is a number no
    /// reviewer can check: the declaration alone says a price stack existed,
    /// not that the top 2% of hours averaged 340.00 and that this is what
    /// struck the revenue.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quantiles: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct EngineInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeterministicSection {
    pub status: String,
    pub metrics: BTreeMap<String, Scalar>,
    pub series: BTreeMap<String, Series>,
    /// Every state change an event made, in the order it happened. Omitted when
    /// the model has none, so a model without events is unchanged.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<TransitionRecord>,
    /// Every causal act the run performed, with what happened to it.
    ///
    /// `transitions` answers "did this field change". The journal answers the
    /// question a reviewer actually asks — WHAT DID THE MODEL DO, and did each
    /// thing it was asked to do happen. An action that was declined or
    /// overridden appears here; before this it appeared nowhere, which is how
    /// an `activate stream` that lost to the stream's own `active when` could
    /// leave no trace at all.
    ///
    /// Omitted when empty, so a model with no events, options or waterfalls
    /// publishes exactly what it published before. `docs/28` §8.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub journal: Vec<JournalEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annual_rollup: Option<AnnualRollupSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RuntimeError>>,
}

/// One causal act, in the order the engine performed it.
///
/// FLAT ON PURPOSE. A nested shape (event → its actions → their outcomes)
/// reads better as a tree and worse as evidence: a golden asserts on lines, a
/// reviewer greps for a stream name, and the schema gate checks one row type.
/// So each row is one act, carrying who did it and what became of it.
#[derive(Debug, Clone, Serialize)]
pub struct JournalEntry {
    pub period: usize,
    pub date: String,
    /// Who acted, qualified by kind: `event:covenant_breach`,
    /// `waterfall:jv.distribution`, `option:renewal`. Qualified because a
    /// waterfall and an event may share a name and the log must not conflate
    /// them.
    pub actor: String,
    /// What was attempted: `set`, `activate_stream`, `deactivate_stream`,
    /// `exercise_option`, `pay`.
    pub action: String,
    /// What it acted on — a field path, a stream name, a step and its payee.
    pub target: String,
    /// What became of it. `applied` is the only one that changed anything:
    ///
    /// * `applied` — done, and the model reflects it.
    /// * `declined` — refused for a stated reason, e.g. an option outside its
    ///   `exercisable in` window, which is not an option anyone holds.
    /// * `overridden` — done, then lost to a stronger declaration. A stream
    ///   activation is `overridden` when the stream's own `active when` is
    ///   false for that period: both gates must pass, so the event cannot turn
    ///   on what the model says is off.
    /// * `ignored` — an action kind the engine does not know, which only
    ///   hand-written IR can carry
    ///   (`activate contract`, until the contract runtime of `docs/29` M2).
    /// * `failed` — the action's own expression did not evaluate.
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// What the step ALLOCATED, for a waterfall step.
    ///
    /// Allocated, not transferred. A waterfall is an ordered allocation over a
    /// pot (`docs/17`): it decides what each step is entitled to out of what
    /// remains. Whether that cash then physically settles is a different
    /// question the language does not model, and calling this "paid" would
    /// claim it does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    /// The pot before and after the step took from it, so a short pot is
    /// visible as the reason a payee got less than it was owed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_before: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pot_after: Option<f64>,
    /// Why, when the outcome is not `applied`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// What this occurrence DID, when the occurrence and its effects are two
    /// different things (`docs/34` D7).
    ///
    /// A transition is the event; its arrival actions are what it did. They
    /// are children rather than siblings because the tie between them is real:
    /// sharing a period and an entity only implies it, and a reader
    /// reconstructing which `set` belonged to which arrival would be guessing
    /// where several entities move in one period.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<JournalEntry>,
}

impl JournalEntry {
    pub(crate) fn new(
        period: usize,
        date: &str,
        actor: String,
        action: &str,
        target: String,
        outcome: &str,
    ) -> Self {
        Self {
            period,
            date: date.to_string(),
            actor,
            action: action.to_string(),
            target,
            outcome: outcome.to_string(),
            from: None,
            to: None,
            amount: None,
            pot_before: None,
            pot_after: None,
            note: None,
            children: Vec::new(),
        }
    }

    pub(crate) fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub(crate) fn with_change(mut self, from: Option<String>, to: String) -> Self {
        self.from = from;
        self.to = Some(to);
        self
    }
}

/// What a stochastic run asks of the journal: not the trail, the DISTRIBUTION.
///
/// `docs/13` §7.18 settles the shape and rules out the obvious one. A per-trial
/// log is trials x acts of output and nobody reads ten thousand copies of the
/// same sequence. The question a stochastic run actually asks is WHEN something
/// happens and HOW OFTEN — so each distinct act gets one row, and the row's
/// size is bounded by the model rather than by the trial count.
#[derive(Debug, Clone, Serialize)]
pub struct JournalTrialSummary {
    /// The act's identity, matching the deterministic journal's own fields, so
    /// a reader can line a summary up against a single run's trail.
    pub actor: String,
    pub action: String,
    pub target: String,
    pub outcome: String,
    /// Trials in which this act occurred at least once.
    pub trials_occurred: u32,
    /// That count over the trials run — the share `docs/13` §7.18 asks for.
    pub share: f64,
    /// Over the trials where it occurred, the period it FIRST did.
    ///
    /// A latched event fires once, so this is its timing distribution. For an
    /// act that recurs — a waterfall step on a schedule — it is the first
    /// occurrence, which is the same question asked of a repeating thing.
    pub first_period: PeriodDistribution,
}

/// A distribution over periods, reported the way the NPV aggregate is.
#[derive(Debug, Clone, Serialize)]
pub struct PeriodDistribution {
    pub min: usize,
    pub p10: usize,
    pub median: usize,
    pub p90: usize,
    pub max: usize,
    pub mean: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloSection {
    pub status: String,
    pub trials: u32,
    pub seed: u64,
    pub metrics: BTreeMap<String, MetricSummary>,
    pub trial_summaries: Vec<MonteCarloTrialSummary>,
    /// One row per distinct act, summarising when and how often it occurred
    /// across the trials. Omitted when no trial recorded any act, so a model
    /// with no events, options or waterfalls is unchanged. `docs/13` §7.18.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub journal: Vec<JournalTrialSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregates: Option<MonteCarloAggregates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RuntimeError>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioSection {
    pub status: String,
    pub summaries: Vec<ScenarioSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<RuntimeError>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioSummary {
    pub name: String,
    pub metrics: BTreeMap<String, Scalar>,
}

/// Calendar-year aggregates of all per-period series.
/// Omitted when the model frequency is already "annual".
#[derive(Debug, Clone, Serialize)]
pub struct AnnualRollupSection {
    pub series: BTreeMap<String, Series>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloTrialSummary {
    pub trial: u32,
    pub metrics: BTreeMap<String, Scalar>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonteCarloAggregates {
    pub npv: NpvAggregate,
}

#[derive(Debug, Clone, Serialize)]
pub struct NpvAggregate {
    pub mean: f64,
    pub median: f64,
    pub stddev: f64,
    pub p_negative: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Scalar {
    Number(f64),
    Money(Money),
    String(String),
    /// NO ANSWER, published as JSON `null` — which the results schema has
    /// always permitted for a scalar. A metric that folded a selection with
    /// nothing in it has no maximum, and stringifying that as `"null"` (which
    /// is what the catch-all arm used to do) makes an absence look like a
    /// value of type text. `docs/13` §7.86.
    Null,
}

/// One period's value on a published series.
///
/// Cash carries a currency; a declared `state` does not — it is an index, a
/// factor, a count. Publishing a state as `Money` would assert a denomination
/// it does not have, and would make it look summable alongside cash. The
/// results schema has always permitted a bare number here.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SeriesValue {
    Money(Money),
    Number(f64),
    /// A period where the value is genuinely undefined — a coverage ratio in a
    /// period with no debt service. Published as JSON `null`, which the results
    /// schema has always permitted.
    ///
    /// Not zero: a coverage ratio of "no debt" is not a coverage ratio of zero,
    /// and a consumer that averaged the series would be badly misled. Not an
    /// omission either, because a shortened series breaks index alignment.
    Null,
}

impl SeriesValue {
    /// The cash amount, or `None` for a series that is not money. Callers that
    /// weight or sum cash use this, so a state cannot silently contribute.
    pub fn money_amount(&self) -> Option<f64> {
        match self {
            SeriesValue::Money(m) => Some(m.amount),
            SeriesValue::Number(_) | SeriesValue::Null => None,
        }
    }
}

/// A slice's published result (docs/13 §7.90): a named, deliberately
/// partial selection and what it came to. NO reconciliation block, by
/// design — the absence is what the declaration means. A partial number
/// must not dress as a complete one, so a slice never publishes a residual
/// and never claims the model's total.
#[derive(Debug, Clone, Serialize)]
pub struct SliceResult {
    pub id: String,
    /// The selection as declared — the lineage. Kinds intersect, values
    /// within a kind union, excepts subtract.
    pub selection: SliceSelection,
    /// Every stream the selection matched, by name. Empty is published, not
    /// omitted: a slice that matched nothing should be seen matching
    /// nothing.
    pub streams: Vec<String>,
    /// The selection's net cash per period — a fold OF the matched streams,
    /// never counted as cash anywhere else.
    pub net: Series,
    /// total / npv / irr over the matched streams, on the same axis the
    /// model's own figures use. `irr` is absent when the flows never change
    /// sign.
    pub metrics: BTreeMap<String, Scalar>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceSelection {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub streams: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub except_streams: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub except_categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub except_entities: Vec<String>,
}

/// The model's entity graph, published so a consumer holding results alone
/// can build the hierarchy view — who is part of what, what each thing is,
/// and the stable identity a governance layer assigned (docs/13 §7.43,
/// §7.91). Values, not vocabulary: the pack's type ROSTER lives in the pack;
/// this is the graph THIS model declared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsGraph {
    pub entities: Vec<GraphEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEntity {
    /// The reference the model uses everywhere — `asset.tower`.
    pub symbol: String,
    /// The symbol's first segment: asset, party, or container.
    pub family: String,
    /// The ontology type, when the declaration states one.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_id: Option<String>,
    /// The stable identity the model carries for a layer above it — the
    /// literal field `id`, engine-opaque, unique within the model (E1360).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The `part of` parent, when the model groups this entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Series {
    pub index: SeriesIndex,
    /// Where in each period this series' cash falls, per
    /// docs/12_payment_timing.md — the same offset used to discount it, and
    /// the axis `model.wal_years` is measured on. Absent on aggregates
    /// (`model.net_cash_flow`, the annual rollup), which sum streams whose
    /// placements differ and so have no single position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<f64>,
    /// The entity this stream is attached to (`asset.tower`,
    /// `container.fund`). Present on stream series only: a subtotal spans
    /// owners and an aggregate has none. This is what lets a consumer holding
    /// results alone attribute cash to a thing — the gap docs/13 §7.43
    /// records: name inspection is not a substitute, because a pack-lowered
    /// stream's name does not contain the symbol of the entity that owns it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    /// The stream's declared category (`operating.revenue.base_rent`).
    /// Present on categorized stream series only. Beside `entity`, the other
    /// axis a selection needs: ownership says whose cash, category says what
    /// kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub values: Vec<SeriesValue>,
}

impl Series {
    pub(crate) fn from_values(
        calendar: &str,
        start: &str,
        periods: u32,
        currency: &str,
        offset: Option<f64>,
        values: &[f64],
    ) -> Self {
        Self {
            index: SeriesIndex {
                calendar: calendar.to_string(),
                start: start.to_string(),
                periods,
            },
            offset,
            entity: None,
            category: None,
            values: values
                .iter()
                .map(|amount| {
                    SeriesValue::Money(Money {
                        amount: round_amount(*amount),
                        currency: currency.to_string(),
                    })
                })
                .collect(),
        }
    }

    /// A dimensionless series: a declared `state`, published so a recurrence
    /// can be inspected rather than only its effect on cash. No currency and
    /// no offset — a state is not paid, so it does not sit anywhere in its
    /// period.
    /// A plain-number series where some periods are genuinely undefined.
    /// `None` publishes as JSON `null`, which the results schema permits.
    ///
    /// Rounded like every other published number. That is not cosmetic: a
    /// ratio's numerator is a fold of signed cash, so a period whose flows
    /// cancel leaves a residue rather than an exact zero — around 2e-12 in
    /// practice — and dividing that by a real denominator publishes something
    /// like 2.655e-17. Whose last bits differ by platform: this shipped, and
    /// the Windows runner disagreed with Linux and macOS on one golden while
    /// both of those agreed with each other.
    ///
    /// `round_amount` is described at its definition as the single global
    /// rounding policy for deterministic numeric outputs. Skipping it here was
    /// the defect; nothing else published bypasses it.
    pub(crate) fn from_optional(
        calendar: &str,
        start: &str,
        periods: u32,
        values: &[Option<f64>],
    ) -> Self {
        Self {
            index: SeriesIndex {
                calendar: calendar.to_string(),
                start: start.to_string(),
                periods,
            },
            offset: None,
            entity: None,
            category: None,
            values: values
                .iter()
                .map(|v| match v {
                    Some(x) => SeriesValue::Number(round_amount(*x)),
                    None => SeriesValue::Null,
                })
                .collect(),
        }
    }

    pub(crate) fn from_plain(calendar: &str, start: &str, periods: u32, values: &[f64]) -> Self {
        Self {
            index: SeriesIndex {
                calendar: calendar.to_string(),
                start: start.to_string(),
                periods,
            },
            offset: None,
            entity: None,
            category: None,
            values: values.iter().map(|v| SeriesValue::Number(*v)).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SeriesIndex {
    pub calendar: String,
    pub start: String,
    pub periods: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricSummary {
    pub r#type: String,
    /// How many trials published this metric. Not every trial publishes every
    /// name: `model.irr` exists only where the flows solve for a rate, so a
    /// mean over three trials and a mean over five hundred would otherwise
    /// read identically. `docs/13` §7.87.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trials: Option<u32>,
    pub mean: Scalar,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdev: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p01: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p05: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p10: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p25: Option<Scalar>,
    pub p50: Scalar,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p75: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p90: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p95: Option<Scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p99: Option<Scalar>,
}
