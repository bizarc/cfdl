//! Declarative domain metrics.
//!
//! Packs declare their metric sets in `metrics.toml` (see
//! `cfdl_pack::MetricSpec` and docs/07_pack_interface.md); this crate
//! evaluates those specs against engine results. Nothing here is
//! pack-specific — adding a domain means adding a metrics.toml, not Rust.
//!
//! Engine-universal metrics (NPV, IRR, MOIC, payback, WAL) live in
//! `cfdl-engine`, not here.

use cfdl_engine::{DomainMetrics, MetricLineage, Money, Results, Scalar};
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
/// given stream names; absent streams contribute 0.
fn sum_stream_totals(metrics: &BTreeMap<String, Scalar>, streams: &[String]) -> f64 {
    streams
        .iter()
        .filter_map(|name| {
            let key = format!("stream.{name}.total");
            match metrics.get(&key)? {
                Scalar::Money(m) => Some(m.amount),
                _ => None,
            }
        })
        .sum()
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
        }
    }

    fn cre_specs() -> Vec<MetricSpec> {
        vec![
            spec(
                "domain.cre.noi",
                "money",
                "sum",
                &["cre.lease.base_rent", "cre.ops.revenue"],
                &["cre.ops.expense"],
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
            results_version: "0.2".to_string(),
            model_hash: "test".to_string(),
            engine: EngineInfo {
                name: "cfdl-engine".to_string(),
                version: "0.1.0".to_string(),
                build: None,
            },
            warnings: vec![],
            deterministic: DeterministicSection {
                status: "ok".to_string(),
                metrics,
                series: BTreeMap::new(),
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
                aggregates: None,
                errors: None,
            },
            domain_metrics: None,
        }
    }

    #[test]
    fn spec_driven_cre_dscr() {
        let results = make_results_with_metrics(vec![
            ("stream.cre.ops.revenue.total", 720_000.0),
            ("stream.cre.ops.expense.total", -240_000.0),
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
    fn require_positive_gates_dependent_ratio() {
        let results = make_results_with_metrics(vec![
            ("stream.cre.ops.revenue.total", 100_000.0),
            ("stream.cre.ops.expense.total", -40_000.0),
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
}
