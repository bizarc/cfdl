// The runs: the loops around one deterministic evaluation (`docs/29` §2.2).
//
// The base run is one pass — inputs, evaluate, fold. A scenario is a full
// deterministic run with the deal's drivers and the rate it is valued at
// overridden; a Monte Carlo trial is one with its sampled inputs; neither
// rebuilds the grid, the expressions or the schedules, which `prepare_model`
// built once. This module owns the loops, the summaries they produce and the
// assembly of the results document — hashes, provenance, the sections a
// consumer reads — and nothing about how a period is computed.
use super::*;

/// The document without its VIEWS — what `model_hash` identifies.
///
/// A slice filters and a statement organizes. Neither produces cash, so two
/// users who look at identical results differently are running the same model,
/// and the hash has to say so. A metric is NOT dropped: it is a figure the
/// model claims, asserted by every benchmark's `expected_metrics.json`.
///
/// One key, dropped at the ONE site that hashes. `is_ledger` below is the
/// cautionary case — that rule was written onto one field, missed a second,
/// and moved `ledger_hash` on fifteen goldens whose cash was bit-identical.
/// A single site cannot repeat that, and anything added under `views` is
/// outside the model's identity without this function changing.
pub(crate) fn model_only(document: &Value) -> Value {
    let mut model = document.clone();
    if let Some(object) = model.as_object_mut() {
        object.remove("views");
    }
    model
}

pub(crate) fn compute_results(
    ir: &Ir,
    model_hash: String,
    config: RunConfig,
) -> Result<Results, EngineError> {
    refuse_series_reads_in_logic(ir)?;

    // A model may declare its own run modes. Honour a declared Monte Carlo run
    // when the run config does not ask for one, so `run monte_carlo trials N
    // seed S` in source does what it says without a separate config file.
    // An explicit run config still wins.
    let mut config = config;
    if config.monte_carlo.is_none() {
        if let Some(declared) = ir
            .runs
            .iter()
            .find(|run| run.kind == "monte_carlo" && run.trials.is_some_and(|n| n > 0))
        {
            config.monte_carlo = Some(MonteCarloRunConfig {
                trial_count: declared.trials.unwrap_or(1),
                seed: declared.seed.unwrap_or(0),
                distributions: BTreeMap::new(),
            });
        }
    }
    let config = config;

    // ONCE PER MODEL, not once per run: the base run, every scenario and
    // every Monte Carlo trial share one grid, one set of compiled expressions
    // and one set of schedules.
    let mut prep_warnings: Vec<String> = Vec::new();
    let prep = prepare_model(ir, &mut prep_warnings)?;
    let base_run = run_deterministic(ir, &config, &prep)?;
    // Preparation happens once, so its warnings are emitted once — they belong
    // to the run that publishes them rather than being repeated per trial.
    let mut warnings = prep_warnings.clone();
    warnings.extend(base_run.warnings.clone());

    let deterministic = DeterministicSection {
        status: "ok".to_string(),
        metrics: base_run.metrics.clone(),
        series: base_run.series,
        transitions: base_run.transitions.clone(),
        journal: base_run.journal.clone(),
        annual_rollup: base_run.annual_rollup,
        errors: None,
    };

    let mut scenario_summaries = Vec::new();
    for (name, scenario) in &config.scenarios {
        let mut merged_overrides = config.parameter_overrides.clone();
        for (key, value) in &scenario.parameter_overrides {
            merged_overrides.insert(key.clone(), *value);
        }
        let scenario_run = run_deterministic(
            ir,
            &RunConfig {
                // Run-wide: a scenario varies the deal's drivers and the rate it
                // is valued at, not the arithmetic every scenario shares.
                arithmetic: config.arithmetic,
                discount_rate: scenario.discount_rate.unwrap_or(config.discount_rate),
                as_of: scenario.as_of.clone().or_else(|| config.as_of.clone()),
                parameter_overrides: merged_overrides,
                scenarios: BTreeMap::new(),
                monte_carlo: None,
                valuation_grain: None,
            },
            &prep,
        )?;
        warnings.extend(scenario_run.warnings);
        // A scenario is a FULL deterministic run — `run_deterministic` above
        // computed every metric the base run computes. Publishing only NPV
        // threw the rest away: a stress case could not report its IRR, its
        // MoIC, or any per-stream total, and a model whose whole subject is
        // how returns move with leverage had nothing to show for the scenario
        // that varied it.
        //
        // The base run's own metrics are the same map, so scenarios and the
        // deterministic block cannot report different metric sets.
        let mut scenario_metrics = scenario_run.metrics;
        scenario_metrics.insert(
            "model.npv".to_string(),
            Scalar::Money(Money {
                amount: round_amount(scenario_run.npv),
                currency: ir.model.currency.clone(),
            }),
        );
        scenario_summaries.push(ScenarioSummary {
            name: name.clone(),
            metrics: scenario_metrics,
        });
    }

    let scenarios = if scenario_summaries.is_empty() {
        ScenarioSection {
            status: "not_run".to_string(),
            summaries: vec![],
            errors: None,
        }
    } else {
        ScenarioSection {
            status: "ok".to_string(),
            summaries: scenario_summaries,
            errors: None,
        }
    };

    let monte_carlo = if let Some(monte_carlo_config) = &config.monte_carlo {
        let mut trial_summaries = Vec::with_capacity(monte_carlo_config.trial_count as usize);
        // ACT IDENTITY -> the period it first occurred, one entry per trial that
        // saw it. Keyed rather than accumulated per trial, because §7.18's
        // objection to a per-trial log is its size: this map is bounded by the
        // model's acts, not by the trial count.
        let mut journal_firsts: BTreeMap<(String, String, String, String), Vec<usize>> =
            BTreeMap::new();
        let mut npv_values = Vec::with_capacity(monte_carlo_config.trial_count as usize);
        // METRIC NAME -> its samples across the trials. Bounded by the model's
        // metric names rather than by the trial count, which is the same size
        // argument §7.18 made about the journal — and the reason a scalar map
        // is carried per trial where a series map is not.
        let mut metric_samples: BTreeMap<String, MetricSamples> = BTreeMap::new();
        for trial in 0..monte_carlo_config.trial_count {
            let mut trial_overrides = config.parameter_overrides.clone();
            let mut rng_state = splitmix64(
                monte_carlo_config
                    .seed
                    .wrapping_add((trial as u64).wrapping_mul(0x9e3779b97f4a7c15)),
            );
            for (name, distribution) in &monte_carlo_config.distributions {
                let sampled = apply_clip(
                    sample_distribution(&distribution.spec, &mut rng_state),
                    distribution.clip,
                );
                trial_overrides.insert(name.clone(), sampled);
            }
            // In-language assumptions: independent, per-assumption seed
            // streams so adding one assumption never reshuffles another's
            // draws. Run-config overrides above still win on key collision.
            for (name, random) in &ir.assumptions.random {
                let key = format!("inputs.{name}");
                if trial_overrides.contains_key(&key) {
                    continue;
                }
                let spec = ir_distribution_spec(&random.dist)?;
                let mut assumption_rng = splitmix64(
                    monte_carlo_config
                        .seed
                        .wrapping_add(fnv1a(name))
                        .wrapping_add((trial as u64).wrapping_mul(0x9e3779b97f4a7c15)),
                );
                let sampled = apply_clip(
                    sample_distribution(&spec, &mut assumption_rng),
                    random.dist.clip,
                );
                trial_overrides.insert(key, sampled);
            }
            let trial_run = run_deterministic(
                ir,
                &RunConfig {
                    arithmetic: config.arithmetic,
                    discount_rate: config.discount_rate,
                    as_of: config.as_of.clone(),
                    parameter_overrides: trial_overrides,
                    scenarios: BTreeMap::new(),
                    monte_carlo: None,
                    valuation_grain: None,
                },
                &prep,
            )?;
            warnings.extend(trial_run.warnings);
            npv_values.push(trial_run.npv);

            // FIRST occurrence per act in THIS trial, so a repeating act
            // contributes one period rather than one per period.
            let mut seen_this_trial: BTreeMap<(String, String, String, String), usize> =
                BTreeMap::new();
            for entry in &trial_run.journal {
                let key = (
                    entry.actor.clone(),
                    entry.action.clone(),
                    entry.target.clone(),
                    entry.outcome.clone(),
                );
                seen_this_trial.entry(key).or_insert(entry.period);
            }
            for (key, period) in seen_this_trial {
                journal_firsts.entry(key).or_default().push(period);
            }

            // EVERY metric the trial computed, not NPV alone (`docs/13`
            // §7.87). A trial IS a complete deterministic run, so this map
            // already holds `model.npv` beside `model.irr`, `model.moic`,
            // every `stream.*.total` and `entity.*.total`, each `domain.*` KPI
            // and every metric the model declared. The scenario path one
            // function up has carried the whole map since it was written; the
            // trial loop had `trial_run.metrics` in scope and built a fresh
            // one-entry map instead, so the figure a stochastic case exists to
            // assert existed in no trial.
            //
            // `entity.*.total` is the join the published entity graph was
            // built for: a trial row keys `entity.asset.tower.total`, and
            // `graph.entities[].symbol` is `asset.tower`, so a per-entity
            // distribution is now readable from results alone — the ownership
            // axis of §7.43, worn stochastically.
            let trial_metrics = trial_run.metrics;
            for (name, value) in &trial_metrics {
                metric_samples
                    .entry(name.clone())
                    .or_default()
                    .observe(value);
            }
            trial_summaries.push(MonteCarloTrialSummary {
                trial,
                metrics: trial_metrics,
            });
        }

        let aggregates = if npv_values.is_empty() {
            None
        } else {
            Some(MonteCarloAggregates {
                npv: NpvAggregate {
                    mean: round_amount(stats_mean(&npv_values)),
                    median: round_amount(stats_median(&npv_values)),
                    stddev: round_amount(stats_stddev_population(&npv_values)),
                    p_negative: round_amount(probability_negative(&npv_values)),
                },
            })
        };
        // ONE SUMMARY PER METRIC THE TRIALS PUBLISHED (`docs/13` §7.87).
        //
        // This used to be a single hand-built entry for `model.npv`, which is
        // why a declared metric, a MoIC, an IRR or any `domain.*` KPI had no
        // distribution at all. `model.npv` is not special-cased any more — it
        // arrives through the same accumulator as every other name, and comes
        // out with the mean, stdev, min, max and p50 it always had, now with
        // the tails beside them.
        //
        // `MonteCarloAggregates` below still carries the NPV's own four
        // figures, `p_negative` among them: that is the loss probability, a
        // question about a threshold rather than a quantile, and it stays
        // where consumers already read it.
        let metrics: BTreeMap<String, MetricSummary> = metric_samples
            .iter()
            .filter_map(|(name, samples)| Some((name.clone(), samples.summarise()?)))
            .collect();

        let trials_run = monte_carlo_config.trial_count.max(1) as f64;
        let journal_summary: Vec<JournalTrialSummary> = journal_firsts
            .into_iter()
            .map(|((actor, action, target, outcome), mut periods)| {
                periods.sort_unstable();
                let occurred = periods.len();
                JournalTrialSummary {
                    actor,
                    action,
                    target,
                    outcome,
                    trials_occurred: occurred as u32,
                    share: round_share(occurred as f64 / trials_run),
                    first_period: period_distribution(&periods),
                }
            })
            .collect();

        MonteCarloSection {
            status: "ok".to_string(),
            trials: monte_carlo_config.trial_count,
            seed: monte_carlo_config.seed,
            metrics,
            trial_summaries,
            journal: journal_summary,
            aggregates,
            errors: None,
        }
    } else {
        MonteCarloSection {
            status: "not_run".to_string(),
            journal: Vec::new(),
            trials: 1,
            seed: 0,
            metrics: BTreeMap::new(),
            trial_summaries: vec![],
            aggregates: None,
            errors: None,
        }
    };

    // Hashed over the ledger only — the per-stream, per-period series. Not the
    // metrics: NPV and IRR are folds OF the ledger, so including them would
    // make the hash change for a reason the ledger did not.
    // `domain.*` is excluded on the same argument that excludes the metrics:
    // a subtotal is a fold OF the ledger, so a pack changing how it chooses to
    // subtotal must not make the hash claim the cash moved. What is hashed is
    // the cash and the states that produced it.
    //
    // THE FILTER APPLIES TO THE ROLLUP TOO. It did not, and the rollup gaining
    // kind-aware subtotals moved `ledger_hash` on fifteen goldens whose cash was
    // bit-identical — the hash asserting the ledger changed when only a fold
    // over it had. The exclusion belongs to the argument, not to the field it
    // was first written on, so it is expressed once and applied to both.
    let is_ledger = |key: &str| !key.starts_with("domain.");
    let ledger_only: BTreeMap<&String, &Series> = deterministic
        .series
        .iter()
        .filter(|(key, _)| is_ledger(key))
        .collect();
    let rollup_only: Option<BTreeMap<&String, &Series>> = deterministic
        .annual_rollup
        .as_ref()
        .map(|r| r.series.iter().filter(|(key, _)| is_ledger(key)).collect());
    // A LEDGER HAS JOURNAL ENTRIES IN IT. That is what a ledger is, and the
    // trace was in neither hash: two runs with identical series and different
    // journals hashed identically, so the record of what the model DID — the
    // thing `explain` walks — was outside the guarantee. `transitions` joins
    // it on the same argument: which fields changed, and when.
    //
    // THE RUN CONFIGURATION IS DELIBERATELY NOT HERE. Folding it in would give
    // three prepayment speeds over one model three different hashes, and no
    // way to tell whether the cash moved or only a setting. Comparing results
    // across runs is the point; the configuration is an input, and this is the
    // output hash.
    let ledger_hash = canonical_hash(&serde_json::json!({
        "series": ledger_only,
        "annual_rollup": rollup_only.map(|series| serde_json::json!({ "series": series })),
        "journal": deterministic.journal,
        "transitions": deterministic.transitions,
    }));

    let inputs = {
        let section = InputsSection {
            resolved: base_run.resolved_inputs.clone(),
            streams: ir.stream_inputs.clone(),
            quantiles: ir.quantile_inputs.clone(),
        };
        (!section.resolved.is_empty()
            || !section.streams.is_empty()
            || !section.quantiles.is_empty())
        .then_some(section)
    };

    // THE MODEL'S GRAPH, republished from the IR so results stand alone
    // (docs/13 §7.43, §7.91): symbol, family, type, the stable id a layer
    // above assigned, and the part_of parent. What a consumer needed the IR
    // for — attributing a stream to a thing — it now has beside the values.
    // THE CONTRACTS BESIDE THE ENTITIES (docs/40 stage 4): each resolved to
    // its type and master by the compiler, with the streams its pack rules
    // lowered — read off each stream's provenance, which names the contract.
    let graph = (!ir.entities.is_empty() || !ir.contracts.is_empty()).then(|| ResultsGraph {
        entities: ir
            .entities
            .iter()
            .map(|e| GraphEntity {
                symbol: e.symbol.clone(),
                family: e.symbol.split('.').next().unwrap_or_default().to_string(),
                type_id: e.type_id.clone(),
                id: e.fields.get("id").cloned(),
                parent: e.parent.clone(),
            })
            .collect(),
        contracts: ir
            .contracts
            .iter()
            .map(|c| GraphContract {
                name: c.name.clone(),
                type_id: c.type_id.clone(),
                master: c.master.clone(),
                contract_name: c.contract_name.clone(),
                instance: c.instance.clone(),
                subject: c.subject.symbol.clone(),
                parties: c
                    .parties
                    .iter()
                    .map(|p| GraphParty {
                        role: p.role.clone(),
                        master_role: p.master_role.clone(),
                        entity: p.entity.symbol.clone(),
                    })
                    .collect(),
                streams: ir
                    .streams
                    .iter()
                    .filter(|s| lowering_contract(s) == Some(c.name.as_str()))
                    .map(|s| s.name.clone())
                    .chain(ir.waterfalls.iter().flat_map(|w| {
                        w.steps
                            .iter()
                            .filter(|step| step.contract.as_deref() == Some(c.name.as_str()))
                            .map(move |step| format!("{}.{}", w.name, step.name))
                    }))
                    .collect(),
            })
            .collect(),
    });

    Ok(Results {
        results_version: "0.13".to_string(),
        model_hash,
        ledger_hash,
        engine: EngineInfo {
            name: "cfdl-engine".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build: None,
        },
        warnings,
        inputs,
        deterministic,
        scenarios,
        monte_carlo,
        domain_metrics: None,
        statements: None,
        graph,
        slices: (!base_run.slices.is_empty()).then(|| base_run.slices.clone()),
    })
}
