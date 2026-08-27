//! Declarative domain metrics.
//!
//! Packs declare their metric sets in `metrics.toml` (see
//! `cfdl_pack::MetricSpec` and docs/07_pack_interface.md); this crate
//! evaluates those specs against engine results. Nothing here is
//! pack-specific — adding a domain means adding a metrics.toml, not Rust.
//!
//! Engine-universal metrics (NPV, IRR, MOIC, payback, WAL) live in
//! `cfdl-engine`, not here.

use cfdl_engine::{DomainMetrics, MetricLineage, Money, Results, Scalar, Series, SeriesValue};
use cfdl_pack::MetricSpec;
use std::collections::BTreeMap;

/// Evaluate a pack's declared metric specs against engine results.
/// Returns `None` when the pack declares no metrics.
pub fn compute(pack: &str, specs: &[MetricSpec], results: &Results) -> Option<DomainMetrics> {
    if specs.is_empty() {
        return None;
    }
    let stream_metrics = &results.deterministic.metrics;
    let currency = currency_from_results(results);

    let mut metrics: BTreeMap<String, Scalar> = BTreeMap::new();
    let mut lineage: BTreeMap<String, MetricLineage> = BTreeMap::new();

    for spec in specs {
        let (value, lineage_num, lineage_den) = match spec.op.as_str() {
            "sum" => {
                let value = sum_stream_totals(stream_metrics, &spec.numerator_streams)
                    + sum_stream_totals(stream_metrics, &spec.denominator_streams);
                (
                    Some(value),
                    spec.numerator_streams.clone(),
                    spec.denominator_streams.clone(),
                )
            }
            "negated_sum" => {
                let value = -sum_stream_totals(stream_metrics, &spec.numerator_streams);
                (
                    Some(value),
                    spec.numerator_streams.clone(),
                    spec.denominator_streams.clone(),
                )
            }
            "ratio" => {
                let (Some(num_id), Some(den_id)) =
                    (&spec.numerator_metric, &spec.denominator_metric)
                else {
                    // Load-time validation prevents this; skip defensively.
                    continue;
                };
                let num = metrics.get(num_id).map(scalar_value);
                let den = metrics.get(den_id).map(scalar_value);
                let value = match (num, den) {
                    // Omit when either input is absent or the denominator is ~0.
                    (Some(num), Some(den)) if den.abs() > f64::EPSILON => Some(num / den),
                    _ => None,
                };
                (value, vec![num_id.clone()], vec![den_id.clone()])
            }
            "subtotal_total" => {
                // Reduce a published per-period subtotal to its lifetime total.
                //
                // The point is that the metric stops DEFINING anything. Before
                // this, `domain.cre.noi` was nine hand-listed stream selectors
                // here and a category fold in statements.toml — two independent
                // statements of one quantity, which is a drift waiting to
                // happen and which needed an analytic identity to police.
                // Now the fold is the definition and this is a reduction of it.
                let Some(id) = &spec.subtotal else { continue };
                let Some(series) = results.deterministic.series.get(id) else {
                    // Absent because the pack declares no such subtotal, or
                    // because no stream carried its categories. Omitting is
                    // right: publishing 0 would assert a total nobody computed.
                    continue;
                };
                let total: f64 = series
                    .values
                    .iter()
                    .filter_map(|v| match v {
                        SeriesValue::Money(m) => Some(m.amount),
                        SeriesValue::Number(n) => Some(*n),
                        SeriesValue::Null => None,
                    })
                    .sum();
                (Some(total), vec![id.clone()], vec![])
            }
            "wal_years" => {
                let value = wal_years(results, &spec.numerator_streams);
                (value, spec.numerator_streams.clone(), vec![])
            }
            _ => continue,
        };

        let Some(value) = value else { continue };
        if spec.require_positive && value <= 0.0 {
            continue;
        }

        let scalar = match spec.kind.as_str() {
            "money" => Scalar::Money(Money {
                amount: round6(value),
                currency: currency.clone(),
            }),
            _ => Scalar::Number(round6(value)),
        };
        metrics.insert(spec.id.clone(), scalar);
        lineage.insert(
            spec.id.clone(),
            MetricLineage {
                numerator_streams: lineage_num,
                denominator_streams: lineage_den,
                formula: spec.formula.clone(),
            },
        );
    }

    Some(DomainMetrics {
        pack: pack.to_string(),
        metrics,
        lineage,
    })
}

fn scalar_value(scalar: &Scalar) -> f64 {
    match scalar {
        Scalar::Money(m) => m.amount,
        Scalar::Number(v) => *v,
        _ => 0.0,
    }
}

/// Sum the signed stream totals (`stream.{name}.total` Money scalars) for the
/// given stream names; absent streams contribute 0. An entry ending in `.*` is
/// a prefix wildcard: `cre.unit.base_rent.*` sums every per-instance stream
/// lowered from a suffixed contract, AND the bare `cre.unit.base_rent` that an
/// unsuffixed contract lowers to.
///
/// Matching delegates to `cfdl_expr::selector_matches` so there is one dialect.
/// This previously matched `stream.{prefix}.` against the whole key, which
/// reached the bare instance only because the key's own `.total` suffix
/// happened to supply the separating dot — a coincidence of key format rather
/// than a decision, and one `wal_years` does not share because its keys carry
/// no `.total`. Recovering the NAME and matching that removes the coincidence.
fn sum_stream_totals(metrics: &BTreeMap<String, Scalar>, streams: &[String]) -> f64 {
    let mut total = 0.0;
    for name in streams {
        if name.ends_with(".*") {
            for (key, scalar) in metrics {
                let Some(stream_name) = key
                    .strip_prefix("stream.")
                    .and_then(|rest| rest.strip_suffix(".total"))
                else {
                    continue;
                };
                if cfdl_expr::selector_matches(name, stream_name) {
                    if let Scalar::Money(m) = scalar {
                        total += m.amount;
                    }
                }
            }
        } else {
            let key = format!("stream.{name}.total");
            if let Some(Scalar::Money(m)) = metrics.get(&key) {
                total += m.amount;
            }
        }
    }
    total
}

/// Weighted average life in years of the matched streams' positive
/// per-period amounts: `sum(((t + offset)/ppy) * v) / sum(v)`.
///
/// `offset` is the stream's placement in its period, the same axis discounting
/// uses (docs/12_payment_timing.md). An ordinary annuity's first monthly
/// collection is therefore at 1/12 of a year, not 0 — which is what a
/// prospectus means by "the number of years from the closing date to the
/// related distribution date".
///
/// Periods-per-year comes from the engine's `run.periods_per_year` metric
/// (default 12). Omitted when the matched streams have no positive amounts.
fn wal_years(results: &Results, streams: &[String]) -> Option<f64> {
    let series = &results.deterministic.series;
    let ppy = match results.deterministic.metrics.get("run.periods_per_year") {
        Some(Scalar::Number(v)) if *v > 0.0 => *v,
        _ => 12.0,
    };
    let mut weighted = 0.0_f64;
    let mut total = 0.0_f64;
    for name in streams {
        // Series keys are `stream.<name>` with no `.total`, so the boundary dot
        // that rescued `sum_stream_totals` is absent here: matching
        // `stream.<prefix>.` against the key dropped the BARE instance an
        // unsuffixed contract lowers to, and `domain.credit.wal_years` selects
        // sched_principal, prepay, bullet and recoveries exactly that way.
        // Delegating to the shared selector is the fix.
        let matched: Vec<&Series> = if name.ends_with(".*") {
            series
                .iter()
                .filter_map(|(key, s)| {
                    let stream_name = key.strip_prefix("stream.")?;
                    cfdl_expr::selector_matches(name, stream_name).then_some(s)
                })
                .collect()
        } else {
            series.get(&format!("stream.{name}")).into_iter().collect()
        };
        for s in matched {
            // Default 1.0, not 0.0: a series that somehow arrived without its
            // placement is far likelier to be an ordinary annuity than an
            // annuity due, and 0.0 would silently restore the off-by-one this
            // offset exists to remove. 1.0 is `discount_offset`'s own default.
            let offset = s.offset.unwrap_or(1.0);
            for (t, value) in s.values.iter().enumerate() {
                // `None` for a non-money series. Only `stream.` keys are
                // reached here, so this cannot currently see a state — the
                // filter is the guarantee rather than a hope.
                let Some(amount) = value.money_amount() else {
                    continue;
                };
                if amount > 0.0 {
                    weighted += ((t as f64 + offset) / ppy) * amount;
                    total += amount;
                }
            }
        }
    }
    (total > 0.0).then(|| weighted / total)
}

fn currency_from_results(results: &Results) -> String {
    for scalar in results.deterministic.metrics.values() {
        if let Scalar::Money(m) = scalar {
            return m.currency.clone();
        }
    }
    "USD".to_string()
}

fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use cfdl_engine::{
        DeterministicSection, EngineInfo, MonteCarloSection, Results, ScenarioSection,
    };

    #[allow(clippy::too_many_arguments)]
    fn spec(
        id: &str,
        kind: &str,
        op: &str,
        num_streams: &[&str],
        den_streams: &[&str],
        num_metric: Option<&str>,
        den_metric: Option<&str>,
        formula: &str,
        require_positive: bool,
    ) -> MetricSpec {
        MetricSpec {
            id: id.to_string(),
            kind: kind.to_string(),
            op: op.to_string(),
            numerator_streams: num_streams.iter().map(|s| s.to_string()).collect(),
            denominator_streams: den_streams.iter().map(|s| s.to_string()).collect(),
            numerator_metric: num_metric.map(|s| s.to_string()),
            denominator_metric: den_metric.map(|s| s.to_string()),
            formula: formula.to_string(),
            require_positive,
            subtotal: None,
        }
    }

    fn cre_specs() -> Vec<MetricSpec> {
        vec![
            spec(
                "domain.cre.noi",
                "money",
                "sum",
                &["cre.lease.base_rent", "cre.ops.revenue"],
                &["cre.opex.line"],
                None,
                None,
                "sum(numerator_streams) + sum(denominator_streams)",
                false,
            ),
            spec(
                "domain.cre.debt_service",
                "money",
                "negated_sum",
                &["loan.construction_interest", "loan.permanent_debt_service"],
                &[],
                None,
                None,
                "-sum(numerator_streams)",
                true,
            ),
            spec(
                "domain.cre.dscr",
                "number",
                "ratio",
                &[],
                &[],
                Some("domain.cre.noi"),
                Some("domain.cre.debt_service"),
                "domain.cre.noi / domain.cre.debt_service",
                false,
            ),
        ]
    }

    fn make_results_with_metrics(stream_metrics: Vec<(&str, f64)>) -> Results {
        let mut metrics = BTreeMap::new();
        for (key, amount) in stream_metrics {
            metrics.insert(
                key.to_string(),
                Scalar::Money(Money {
                    amount,
                    currency: "USD".to_string(),
                }),
            );
        }
        Results {
            results_version: "0.3".to_string(),
            model_hash: "test".to_string(),
            ledger_hash: "test".to_string(),
            engine: EngineInfo {
                name: "cfdl-engine".to_string(),
                version: "0.1.0".to_string(),
                build: None,
            },
            warnings: vec![],
            inputs: None,
            deterministic: DeterministicSection {
                status: "ok".to_string(),
                metrics,
                series: BTreeMap::new(),
                transitions: Vec::new(),
                journal: Vec::new(),
                annual_rollup: None,
                errors: None,
            },
            scenarios: ScenarioSection {
                status: "not_run".to_string(),
                summaries: vec![],
                errors: None,
            },
            monte_carlo: MonteCarloSection {
                status: "not_run".to_string(),
                trials: 1,
                seed: 0,
                metrics: BTreeMap::new(),
                trial_summaries: vec![],
                journal: Vec::new(),
                aggregates: None,
                errors: None,
            },
            domain_metrics: None,
            statements: None,
        }
    }

    #[test]
    fn spec_driven_cre_dscr() {
        let results = make_results_with_metrics(vec![
            ("stream.cre.ops.revenue.total", 720_000.0),
            ("stream.cre.opex.line.total", -240_000.0),
            ("stream.loan.permanent_debt_service.total", -360_000.0),
        ]);
        let dm = compute("cre", &cre_specs(), &results).expect("metrics");
        assert_eq!(dm.pack, "cre");
        let noi = match dm.metrics.get("domain.cre.noi").expect("noi") {
            Scalar::Money(m) => m.amount,
            other => panic!("expected money, got {other:?}"),
        };
        assert!((noi - 480_000.0).abs() < 1e-3);
        let dscr = match dm.metrics.get("domain.cre.dscr").expect("dscr") {
            Scalar::Number(v) => *v,
            other => panic!("expected number, got {other:?}"),
        };
        assert!((dscr - (480_000.0 / 360_000.0)).abs() < 1e-5);
    }

    #[test]
    fn sum_glob_includes_the_unsuffixed_instance() {
        // `sum` gets this right, but only by accident, and the accident is worth
        // pinning. It matches `stream.<prefix>.` against keys that carry a
        // `.total` suffix, so a BARE stream's key — `stream.<prefix>.total` —
        // supplies the boundary dot itself. Change the key format and the bare
        // instance silently drops out, which is what happens in `wal_years`
        // below, where the keys have no `.total`.
        let specs = vec![spec(
            "domain.credit.interest",
            "money",
            "sum",
            &["credit.pool.interest.*"],
            &[],
            None,
            None,
            "sum(numerator_streams)",
            false,
        )];
        let results = make_results_with_metrics(vec![
            ("stream.credit.pool.interest.total", 1_000.0),
            ("stream.credit.pool.interest.p.total", 250.0),
        ]);
        let dm = compute("credit", &specs, &results).expect("metrics");
        let interest = match dm.metrics.get("domain.credit.interest").expect("interest") {
            Scalar::Money(m) => m.amount,
            other => panic!("expected money, got {other:?}"),
        };
        assert!(
            (interest - 1_250.0).abs() < 1e-6,
            "expected the bare and the suffixed stream, got {interest}"
        );
    }

    #[test]
    fn wal_years_glob_includes_the_unsuffixed_instance() {
        // The real defect. `wal_years` matches against SERIES keys, which are
        // `stream.<name>` with no `.total`, so the accident that rescues `sum`
        // does not apply: `stream.credit.pool.prepay` does not start with
        // `stream.credit.pool.prepay.`, and the bare stream was dropped.
        //
        // `packs/credit/metrics.toml`'s `domain.credit.wal_years` selects
        // sched_principal, prepay, bullet and recoveries exactly this way, and
        // goldens ship all four bare — so an unsuffixed pool reported a WAL over
        // a subset of its own principal, with no diagnostic. Nothing caught it:
        // none of the affected fixtures runs with `--pack`, so `domain_metrics`
        // is absent from every golden that would have shown it.
        use cfdl_engine::{Series, SeriesIndex, SeriesValue};
        let mut results = make_results_with_metrics(vec![]);
        results
            .deterministic
            .metrics
            .insert("run.periods_per_year".to_string(), Scalar::Number(12.0));
        let series = |values: &[f64]| Series {
            index: SeriesIndex {
                calendar: "monthly".to_string(),
                start: "2026-01-01".to_string(),
                periods: values.len() as u32,
            },
            offset: Some(1.0),
            values: values
                .iter()
                .map(|v| {
                    SeriesValue::Money(Money {
                        amount: *v,
                        currency: "USD".to_string(),
                    })
                })
                .collect(),
        };
        // Bare stream pays 100 in period 0 (instant 1/12); suffixed pays 100 in
        // period 2 (instant 3/12). Including both gives a mean of 2/12; dropping
        // the bare one gives 3/12.
        results.deterministic.series.insert(
            "stream.credit.pool.prepay".to_string(),
            series(&[100.0, 0.0, 0.0]),
        );
        results.deterministic.series.insert(
            "stream.credit.pool.prepay.p".to_string(),
            series(&[0.0, 0.0, 100.0]),
        );
        let specs = vec![spec(
            "domain.credit.wal_years",
            "number",
            "wal_years",
            &["credit.pool.prepay.*"],
            &[],
            None,
            None,
            "wal_years(numerator_streams)",
            false,
        )];
        let dm = compute("credit", &specs, &results).expect("metrics");
        let wal = match dm.metrics.get("domain.credit.wal_years").expect("wal") {
            Scalar::Number(v) => *v,
            other => panic!("expected number, got {other:?}"),
        };
        // Tolerance is 1e-6, not tighter: `compute` publishes through `round6`,
        // so 2/12 arrives as 0.166667 and an exact comparison would fail on the
        // rounding rather than on the selector.
        assert!(
            (wal - 2.0 / 12.0).abs() < 1e-6,
            "expected the bare stream to be weighted too; got {wal} \
             ({} means it was dropped)",
            3.0 / 12.0
        );
    }

    #[test]
    fn glob_selector_does_not_reach_a_sibling_sharing_a_text_prefix() {
        // The boundary the fix must not widen: `.*` adds a path segment, so a
        // differently-named sibling stays out. Matching on the raw string
        // prefix would sweep it in.
        let specs = vec![spec(
            "domain.credit.interest",
            "money",
            "sum",
            &["credit.pool.interest.*"],
            &[],
            None,
            None,
            "sum(numerator_streams)",
            false,
        )];
        let results = make_results_with_metrics(vec![
            ("stream.credit.pool.interest.total", 1_000.0),
            ("stream.credit.pool.interest_accrued.total", 999.0),
        ]);
        let dm = compute("credit", &specs, &results).expect("metrics");
        let interest = match dm.metrics.get("domain.credit.interest").expect("interest") {
            Scalar::Money(m) => m.amount,
            other => panic!("expected money, got {other:?}"),
        };
        assert!(
            (interest - 1_000.0).abs() < 1e-6,
            "interest_accrued is a different stream, got {interest}"
        );
    }

    #[test]
    fn require_positive_gates_dependent_ratio() {
        let results = make_results_with_metrics(vec![
            ("stream.cre.ops.revenue.total", 100_000.0),
            ("stream.cre.opex.line.total", -40_000.0),
        ]);
        let dm = compute("cre", &cre_specs(), &results).expect("metrics");
        assert!(!dm.metrics.contains_key("domain.cre.debt_service"));
        assert!(!dm.metrics.contains_key("domain.cre.dscr"));
        assert!(dm.metrics.contains_key("domain.cre.noi"));
    }

    #[test]
    fn ratio_omitted_on_zero_denominator() {
        let specs = vec![
            spec(
                "m.revenue",
                "money",
                "sum",
                &["a.rev"],
                &[],
                None,
                None,
                "sum(numerator_streams)",
                false,
            ),
            spec(
                "m.margin",
                "number",
                "ratio",
                &[],
                &[],
                Some("m.revenue"),
                Some("m.revenue"),
                "m.revenue / m.revenue",
                false,
            ),
        ];
        let results = make_results_with_metrics(vec![("stream.a.rev.total", 0.0)]);
        let dm = compute("x", &specs, &results).expect("metrics");
        assert!(dm.metrics.contains_key("m.revenue"));
        assert!(!dm.metrics.contains_key("m.margin"));
    }

    #[test]
    fn empty_specs_return_none() {
        let results = make_results_with_metrics(vec![]);
        assert!(compute("anything", &[], &results).is_none());
    }

    #[test]
    fn wal_years_weights_positive_amounts_by_period() {
        use cfdl_engine::{Series, SeriesIndex, SeriesValue};
        let mut results = make_results_with_metrics(vec![]);
        results
            .deterministic
            .metrics
            .insert("run.periods_per_year".to_string(), Scalar::Number(12.0));
        let series = |offset: Option<f64>, values: &[f64]| Series {
            index: SeriesIndex {
                calendar: "monthly".to_string(),
                start: "2026-01-01".to_string(),
                periods: values.len() as u32,
            },
            offset,
            values: values
                .iter()
                .map(|v| {
                    SeriesValue::Money(Money {
                        amount: *v,
                        currency: "USD".to_string(),
                    })
                })
                .collect(),
        };
        // Both are ordinary annuities (offset 1.0), so 100 in period 0 falls at
        // 1/12 of a year and 100 in period 24 at 25/12. WAL is the mean of the
        // two instants: 13/12 years. Under the old index-only convention this
        // was 1.0, which put the first collection at time zero — the
        // off-by-one the offset exists to remove.
        results.deterministic.series.insert(
            "stream.credit.pool.prepay.a".to_string(),
            series(Some(1.0), &[100.0, 0.0, 0.0, 0.0]),
        );
        let mut tail = vec![0.0; 25];
        tail[24] = 100.0;
        results.deterministic.series.insert(
            "stream.credit.pool.bullet.a".to_string(),
            series(Some(1.0), &tail),
        );
        // Negative amounts are excluded from the weighting.
        results.deterministic.series.insert(
            "stream.credit.pool.other".to_string(),
            series(Some(1.0), &[-50.0]),
        );
        let specs = vec![spec(
            "domain.credit.wal_years",
            "number",
            "wal_years",
            &[
                "credit.pool.prepay.*",
                "credit.pool.bullet.*",
                "credit.pool.other",
            ],
            &[],
            None,
            None,
            "wal_years(numerator_streams)",
            false,
        )];
        let dm = compute("credit", &specs, &results).expect("metrics");
        let wal = match dm.metrics.get("domain.credit.wal_years").expect("wal") {
            Scalar::Number(v) => *v,
            other => panic!("expected number, got {other:?}"),
        };
        // 1e-6, not tighter: metrics are published rounded to six decimals and
        // 13/12 does not terminate. The error this pins is a whole period.
        assert!((wal - 13.0 / 12.0).abs() < 1e-6, "wal = {wal}");

        // Two series matched by one spec, at different placements: each must be
        // weighted with its own offset, not with a single shared one. 100 due
        // (offset 0.0) in period 0 sits at time 0; 100 ordinary (offset 1.0) in
        // period 0 sits at 1/12. Their mean is 1/24.
        let mut mixed = make_results_with_metrics(vec![]);
        mixed
            .deterministic
            .metrics
            .insert("run.periods_per_year".to_string(), Scalar::Number(12.0));
        mixed.deterministic.series.insert(
            "stream.credit.pool.prepay.due".to_string(),
            series(Some(0.0), &[100.0]),
        );
        mixed.deterministic.series.insert(
            "stream.credit.pool.prepay.ordinary".to_string(),
            series(Some(1.0), &[100.0]),
        );
        let mixed_specs = vec![spec(
            "domain.credit.wal_years",
            "number",
            "wal_years",
            &["credit.pool.prepay.*"],
            &[],
            None,
            None,
            "wal_years(numerator_streams)",
            false,
        )];
        let dm = compute("credit", &mixed_specs, &mixed).expect("metrics");
        let wal = match dm.metrics.get("domain.credit.wal_years").expect("wal") {
            Scalar::Number(v) => *v,
            other => panic!("expected number, got {other:?}"),
        };
        assert!((wal - 1.0 / 24.0).abs() < 1e-6, "wal = {wal}");

        // No positive amounts anywhere -> metric omitted.
        let mut empty = make_results_with_metrics(vec![]);
        empty.deterministic.series.insert(
            "stream.credit.pool.other".to_string(),
            series(Some(1.0), &[-1.0]),
        );
        let dm = compute(
            "credit",
            &[spec(
                "domain.credit.wal_years",
                "number",
                "wal_years",
                &["credit.pool.other"],
                &[],
                None,
                None,
                "wal_years(numerator_streams)",
                false,
            )],
            &empty,
        )
        .expect("metrics");
        assert!(!dm.metrics.contains_key("domain.credit.wal_years"));
    }
}
