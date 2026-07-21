use cfdl_engine::{DomainMetrics, MetricLineage, Money, Results, Scalar};
use std::collections::BTreeMap;

/// Compute pack-specific domain metrics from engine results.
/// Returns `None` if the pack name is not recognised.
pub fn compute(pack: &str, results: &Results) -> Option<DomainMetrics> {
    match pack {
        "cre" => Some(compute_cre(results)),
        "opco" => Some(compute_opco(results)),
        _ => None,
    }
}

/// Sum the signed stream totals for the given stream names.
/// Looks up `stream.{name}.total` Money scalars and returns the sum of their amounts.
/// Streams absent from `metrics` contribute 0.
fn sum_stream_totals(metrics: &BTreeMap<String, Scalar>, streams: &[&str]) -> f64 {
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

/// Extract the model currency from the first Money scalar found in metrics.
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

fn compute_cre(results: &Results) -> DomainMetrics {
    let m = &results.deterministic.metrics;
    let currency = currency_from_results(results);

    let revenue_streams: &[&str] = &["cre.lease.base_rent", "cre.ops.revenue"];
    let expense_streams: &[&str] = &["cre.ops.expense"];
    let loan_streams: &[&str] = &["loan.construction_interest", "loan.permanent_debt_service"];

    // NOI = sum(revenue inflows) + sum(expense outflows as negatives)
    let noi = sum_stream_totals(m, revenue_streams) + sum_stream_totals(m, expense_streams);
    // Debt service = absolute value of outflow loan totals
    let debt_service_signed = sum_stream_totals(m, loan_streams);
    let debt_service = -debt_service_signed;

    let mut metrics: BTreeMap<String, Scalar> = BTreeMap::new();
    let mut lineage: BTreeMap<String, MetricLineage> = BTreeMap::new();

    metrics.insert(
        "domain.cre.noi".to_string(),
        Scalar::Money(Money {
            amount: round6(noi),
            currency: currency.clone(),
        }),
    );
    lineage.insert(
        "domain.cre.noi".to_string(),
        MetricLineage {
            numerator_streams: revenue_streams.iter().map(|s| s.to_string()).collect(),
            denominator_streams: expense_streams.iter().map(|s| s.to_string()).collect(),
            formula: "sum(numerator_streams) + sum(denominator_streams)".to_string(),
        },
    );

    if debt_service > 0.0 {
        metrics.insert(
            "domain.cre.debt_service".to_string(),
            Scalar::Money(Money {
                amount: round6(debt_service),
                currency: currency.clone(),
            }),
        );
        lineage.insert(
            "domain.cre.debt_service".to_string(),
            MetricLineage {
                numerator_streams: loan_streams.iter().map(|s| s.to_string()).collect(),
                denominator_streams: vec![],
                formula: "-sum(numerator_streams)".to_string(),
            },
        );

        let dscr = noi / debt_service;
        metrics.insert("domain.cre.dscr".to_string(), Scalar::Number(round6(dscr)));
        lineage.insert(
            "domain.cre.dscr".to_string(),
            MetricLineage {
                numerator_streams: vec!["domain.cre.noi".to_string()],
                denominator_streams: vec!["domain.cre.debt_service".to_string()],
                formula: "domain.cre.noi / domain.cre.debt_service".to_string(),
            },
        );
    }

    DomainMetrics {
        pack: "cre".to_string(),
        metrics,
        lineage,
    }
}

fn compute_opco(results: &Results) -> DomainMetrics {
    let m = &results.deterministic.metrics;
    let currency = currency_from_results(results);

    let revenue_streams: &[&str] = &["opco.revenue.recurring"];
    let opex_streams: &[&str] = &["opco.opex.recurring"];

    let revenue = sum_stream_totals(m, revenue_streams);
    let ebitda = revenue + sum_stream_totals(m, opex_streams);

    let mut metrics: BTreeMap<String, Scalar> = BTreeMap::new();
    let mut lineage: BTreeMap<String, MetricLineage> = BTreeMap::new();

    metrics.insert(
        "domain.opco.revenue".to_string(),
        Scalar::Money(Money {
            amount: round6(revenue),
            currency: currency.clone(),
        }),
    );
    lineage.insert(
        "domain.opco.revenue".to_string(),
        MetricLineage {
            numerator_streams: revenue_streams.iter().map(|s| s.to_string()).collect(),
            denominator_streams: vec![],
            formula: "sum(numerator_streams)".to_string(),
        },
    );

    metrics.insert(
        "domain.opco.ebitda".to_string(),
        Scalar::Money(Money {
            amount: round6(ebitda),
            currency: currency.clone(),
        }),
    );
    lineage.insert(
        "domain.opco.ebitda".to_string(),
        MetricLineage {
            numerator_streams: revenue_streams.iter().map(|s| s.to_string()).collect(),
            denominator_streams: opex_streams.iter().map(|s| s.to_string()).collect(),
            formula: "sum(numerator_streams) + sum(denominator_streams)".to_string(),
        },
    );

    if revenue.abs() > f64::EPSILON {
        let ebitda_margin = ebitda / revenue;
        metrics.insert(
            "domain.opco.ebitda_margin".to_string(),
            Scalar::Number(round6(ebitda_margin)),
        );
        lineage.insert(
            "domain.opco.ebitda_margin".to_string(),
            MetricLineage {
                numerator_streams: vec!["domain.opco.ebitda".to_string()],
                denominator_streams: vec!["domain.opco.revenue".to_string()],
                formula: "domain.opco.ebitda / domain.opco.revenue".to_string(),
            },
        );
    }

    DomainMetrics {
        pack: "opco".to_string(),
        metrics,
        lineage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cfdl_engine::{
        DeterministicSection, EngineInfo, MonteCarloSection, Results, ScenarioSection,
    };

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
    fn compute_cre_dscr_smoke() {
        // cre.ops.revenue: +720_000 (inflow, 30k/mo * 24)
        // cre.ops.expense: -240_000 (outflow, 10k/mo * 24)
        // loan.permanent_debt_service: -360_000 (outflow, 15k/mo * 24)
        let results = make_results_with_metrics(vec![
            ("stream.cre.ops.revenue.total", 720_000.0),
            ("stream.cre.ops.expense.total", -240_000.0),
            ("stream.loan.permanent_debt_service.total", -360_000.0),
        ]);
        let dm = compute("cre", &results).expect("cre metrics");
        assert_eq!(dm.pack, "cre");

        let noi = match dm.metrics.get("domain.cre.noi").expect("noi") {
            Scalar::Money(m) => m.amount,
            other => panic!("expected money, got {other:?}"),
        };
        assert!((noi - 480_000.0).abs() < 1e-3, "noi={noi}");

        let ds = match dm
            .metrics
            .get("domain.cre.debt_service")
            .expect("debt_service")
        {
            Scalar::Money(m) => m.amount,
            other => panic!("expected money, got {other:?}"),
        };
        assert!((ds - 360_000.0).abs() < 1e-3, "debt_service={ds}");

        let dscr = match dm.metrics.get("domain.cre.dscr").expect("dscr") {
            Scalar::Number(v) => *v,
            other => panic!("expected number, got {other:?}"),
        };
        assert!((dscr - (480_000.0 / 360_000.0)).abs() < 1e-5, "dscr={dscr}");
    }

    #[test]
    fn compute_cre_no_debt_service_omits_dscr() {
        let results = make_results_with_metrics(vec![
            ("stream.cre.ops.revenue.total", 100_000.0),
            ("stream.cre.ops.expense.total", -40_000.0),
        ]);
        let dm = compute("cre", &results).expect("cre metrics");
        assert!(!dm.metrics.contains_key("domain.cre.dscr"));
        assert!(!dm.metrics.contains_key("domain.cre.debt_service"));
        let noi = match dm.metrics.get("domain.cre.noi").expect("noi") {
            Scalar::Money(m) => m.amount,
            _ => panic!(),
        };
        assert!((noi - 60_000.0).abs() < 1e-3);
    }

    #[test]
    fn compute_opco_ebitda_margin() {
        // revenue: +720_000 (100k/mo * 72 periods? just use simple values)
        // opex: -432_000
        let results = make_results_with_metrics(vec![
            ("stream.opco.revenue.recurring.total", 100_000.0),
            ("stream.opco.opex.recurring.total", -60_000.0),
        ]);
        let dm = compute("opco", &results).expect("opco metrics");

        let rev = match dm.metrics.get("domain.opco.revenue").expect("revenue") {
            Scalar::Money(m) => m.amount,
            _ => panic!(),
        };
        assert!((rev - 100_000.0).abs() < 1e-3);

        let ebitda = match dm.metrics.get("domain.opco.ebitda").expect("ebitda") {
            Scalar::Money(m) => m.amount,
            _ => panic!(),
        };
        assert!((ebitda - 40_000.0).abs() < 1e-3, "ebitda={ebitda}");

        let margin = match dm.metrics.get("domain.opco.ebitda_margin").expect("margin") {
            Scalar::Number(v) => *v,
            _ => panic!(),
        };
        assert!((margin - 0.4).abs() < 1e-5, "margin={margin}");
    }

    #[test]
    fn unknown_pack_returns_none() {
        let results = make_results_with_metrics(vec![]);
        assert!(compute("unknown_pack", &results).is_none());
    }
}
