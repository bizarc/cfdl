//! The engine, run over the blessed corpus, from Rust.
//!
//! WHY THIS EXISTS. The engine's real guard is the golden suite — 106 valid
//! fixtures — and until now it was reachable only through `tools/golden-runner`,
//! a shell script driving the CLI. `cargo test -p cfdl-engine` ran 27 unit
//! tests over a 2,200-line engine in 0.01 seconds. Anything that measures the
//! engine against `cargo test` was therefore measuring almost nothing, which
//! matters for two things named in `docs/29`: the mutation baseline of phase
//! 0.2, and the collapse property of phase 2, whose whole content is "every
//! blessed number is unchanged".
//!
//! WHAT IT ISOLATES. The corpus already holds compiled IR (`gold/ir`) beside
//! the results it must produce (`gold/results`), so this test needs no
//! compiler: IR in, results out, byte-compared against what is blessed. A
//! failure here is the engine's, not the parser's or the lowering's — which is
//! exactly the isolation a mutation run wants, since a mutant is injected into
//! the engine alone.
//!
//! WHAT IT DOES NOT COVER, deliberately. `domain_metrics` and `statements` are
//! assembled by the CLI from the pack registry after the engine returns, so
//! they are dropped before comparison rather than half-checked here; the
//! shell runner still covers them end to end. Fixtures whose IR or results are
//! not both blessed are skipped, and the count of what ran is asserted so a
//! corpus that silently stops being discovered fails instead of passing.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The repository root, from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<name> has two ancestors")
        .to_path_buf()
}

/// Sections the engine does not produce, so this test cannot compare them.
const NOT_THE_ENGINE: [&str; 2] = ["domain_metrics", "statements"];

fn strip_non_engine(value: &mut serde_json::Value) {
    if let Some(map) = value.as_object_mut() {
        for key in NOT_THE_ENGINE {
            map.remove(key);
        }
    }
}

/// Every fixture with both a blessed IR and a blessed results document.
fn corpus(root: &Path) -> BTreeMap<String, (PathBuf, PathBuf, Option<PathBuf>)> {
    let mut found = BTreeMap::new();
    let ir_dir = root.join("gold/ir");
    let entries = std::fs::read_dir(&ir_dir)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", ir_dir.display()));
    for entry in entries {
        let path = entry.expect("readable dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("utf-8 file stem")
            .to_string();
        let results = root.join(format!("gold/results/{name}.results.json"));
        if !results.is_file() {
            continue;
        }
        // The run config the shell runner would pass, when the fixture has one.
        let run_config = root.join(format!("fixtures/valid/{name}/run.json"));
        let run_config = run_config.is_file().then_some(run_config);
        found.insert(name, (path, results, run_config));
    }
    found
}

#[test]
fn engine_reproduces_the_blessed_corpus() {
    let root = repo_root();
    let corpus = corpus(&root);

    // The corpus is discovered from the filesystem, so its size is asserted:
    // a glob that stops matching would otherwise turn this test green by
    // running nothing at all.
    assert!(
        corpus.len() >= 100,
        "expected at least 100 blessed fixtures, found {} — has the corpus moved?",
        corpus.len()
    );

    let mut failures: Vec<String> = Vec::new();

    for (name, (ir_path, results_path, run_config)) in &corpus {
        // The shell runner's two branches, exactly: with a run config it
        // passes `--config` and no `--rate`, so the CLI's own default of 0
        // applies; without one it passes `--rate 0.10`. Getting this wrong
        // makes fixtures differ here for a reason that is not the engine's.
        let config = match run_config {
            Some(path) => match cfdl_engine::run_config_from_json_file(path, 0.0, None) {
                Ok(config) => config,
                Err(err) => {
                    failures.push(format!("{name}: run config did not load: {err}"));
                    continue;
                }
            },
            None => cfdl_engine::RunConfig {
                discount_rate: 0.10,
                ..Default::default()
            },
        };

        let produced = match cfdl_engine::run_from_file(ir_path, config) {
            Ok(results) => results,
            Err(err) => {
                failures.push(format!("{name}: run failed: {err}"));
                continue;
            }
        };

        // BOTH SIDES THROUGH THE SAME PARSER, and the reason is not cosmetic.
        // `serde_json`'s float parser is accurate to within one ULP unless its
        // `float_roundtrip` feature is on — which is what that feature exists
        // for. Comparing a freshly computed f64 against a blessed value parsed
        // from text therefore reports a difference in the last bit that the
        // engine did not produce: measured on seven `pow`-based series, where
        // the engine and the CLI both write ...377 and the parse returns
        // ...376. The shell runner never sees this because it canonicalizes
        // through Python's correctly-rounded parser on both sides. Serializing
        // and re-parsing puts the same treatment on both, so the parse error
        // cancels instead of masquerading as an engine change.
        let mut produced = match serde_json::to_string(&produced)
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text))
        {
            Ok(value) => value,
            Err(err) => {
                failures.push(format!("{name}: results did not round-trip: {err}"));
                continue;
            }
        };
        let blessed = std::fs::read_to_string(results_path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", results_path.display()));
        let mut blessed: serde_json::Value = serde_json::from_str(&blessed)
            .unwrap_or_else(|err| panic!("{} is not valid JSON: {err}", results_path.display()));

        strip_non_engine(&mut produced);
        strip_non_engine(&mut blessed);

        if produced != blessed {
            failures.push(format!("{name}: {}", first_difference(&blessed, &produced)));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} blessed fixtures did not reproduce:\n  {}",
        failures.len(),
        corpus.len(),
        failures.join("\n  ")
    );
}

/// The first path at which two documents disagree, so a failure names a key
/// rather than printing two results documents.
fn first_difference(blessed: &serde_json::Value, produced: &serde_json::Value) -> String {
    fn walk(
        path: &str,
        blessed: &serde_json::Value,
        produced: &serde_json::Value,
    ) -> Option<String> {
        match (blessed, produced) {
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
                for (key, a_value) in a {
                    let child = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}/{key}")
                    };
                    match b.get(key) {
                        None => return Some(format!("{child} is missing")),
                        Some(b_value) => {
                            if let Some(found) = walk(&child, a_value, b_value) {
                                return Some(found);
                            }
                        }
                    }
                }
                for key in b.keys() {
                    if !a.contains_key(key) {
                        return Some(format!("{path}/{key} is unexpected"));
                    }
                }
                None
            }
            (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                if a.len() != b.len() {
                    return Some(format!(
                        "{path} has {} entries, expected {}",
                        b.len(),
                        a.len()
                    ));
                }
                for (idx, (a_value, b_value)) in a.iter().zip(b).enumerate() {
                    if let Some(found) = walk(&format!("{path}[{idx}]"), a_value, b_value) {
                        return Some(found);
                    }
                }
                None
            }
            _ => {
                if blessed == produced {
                    None
                } else {
                    Some(format!("{path}: expected {blessed}, produced {produced}"))
                }
            }
        }
    }
    walk("", blessed, produced).unwrap_or_else(|| "documents differ".to_string())
}

/// WHICH MODELS CAN BE WALKED, asserted rather than tabulated.
///
/// `docs/29` §2.0 measured the corpus by hand before the period walk was
/// designed: a period walk cannot serve a read that reaches forward, because
/// at period 3 there is no period 24. This pins that measurement in code, so
/// the table in the plan cannot quietly go stale and a new forward-reading
/// model cannot arrive unnoticed.
///
/// The expected set is exactly the two constructs `docs/28` §7 migrates to the
/// valuation plane — the forward-income exit and the expense stop's base year
/// — plus the two fixtures carrying an absolute window. Everything else in the
/// corpus reads cumulatively BACKWARD (`[0..time.t]`), which a walk serves
/// exactly; that distinction is what makes the collapse property reachable.
#[test]
fn only_the_known_models_read_forward() {
    let root = repo_root();
    let corpus = corpus(&root);
    assert!(corpus.len() >= 100, "corpus not found: {}", corpus.len());

    // Since phase 6, a forward window in an AMOUNT is a PRICED amount — a
    // valuation setting a causal amount (`docs/28` §7) — and the walk serves
    // it in a priced pass, so nothing here is walk-ineligible any more. The
    // inventory stays deliberate: a new priced amount is added below by
    // hand, not by blessing a diff, and anything ineligible for another
    // reason (a forward GUARD, a priced coupling) shows up in `ineligible`.
    let mut ineligible: Vec<String> = Vec::new();
    let mut priced: Vec<String> = Vec::new();
    for (name, (ir_path, _, _)) in &corpus {
        let raw = std::fs::read_to_string(ir_path).expect("blessed IR is readable");
        match cfdl_engine::walk_eligibility(&raw) {
            Ok(None) => {}
            Ok(Some(_)) => ineligible.push(name.clone()),
            Err(err) => panic!("{name}: eligibility could not be computed: {err}"),
        }
        if !cfdl_engine::priced_streams(&raw)
            .expect("priced inventory computes")
            .is_empty()
        {
            priced.push(name.clone());
        }
    }

    // Two causes, both migrated to the priced exception:
    //
    //   cre_derived_lines      the expense stop's absolute base year,
    //                          `cre.opex.line[24..24]` — declared in the
    //                          valuation plane, which is what ties the MIT
    //                          Rentleg reference
    //   cre_office_two_tenant  \ the CRE pack's `cre.exit_forward` lowering,
    //   pack_cadence_cre_*     /  which reads `[time.t + 1 .. time.t + 12]`
    let expected_priced = [
        "cre_derived_lines",
        "cre_office_two_tenant",
        "pack_cadence_cre_annual",
        "pack_cadence_cre_monthly",
        "pack_cadence_cre_quarterly",
    ];
    assert_eq!(
        priced, expected_priced,
        "the set of priced fixtures changed. If a new amount prices a forward \
         window, `docs/28` §7 describes what it may and may not touch — add it \
         here deliberately, not by blessing a diff."
    );
    assert_eq!(
        ineligible,
        Vec::<String>::new(),
        "a fixture became walk-ineligible; the reason names the coupling"
    );
}

/// THE COLLAPSE PROPERTY, on every model that can be walked.
///
/// `docs/29` phase 2 rests on one claim: evaluating a period at a time computes
/// what evaluating a column at a time computes. That claim is checkable, and
/// this checks it — both orders, every blessed model, every stream, every
/// period, compared exactly.
///
/// It is worth more than the goldens for this purpose. A golden says the engine
/// still produces the blessed numbers; this says the two ORDERS agree, which is
/// the property the reorder actually needs and the one a golden cannot see
/// while only one order runs in production.
///
/// Models that read forward are skipped rather than failed: at period 3 there
/// is no period 24, so the walk is inapplicable there, not wrong. Which models
/// those are is pinned by `only_the_known_models_read_forward`.
#[test]
fn walk_matches_the_column_order() {
    let root = repo_root();
    let corpus = corpus(&root);
    assert!(corpus.len() >= 100, "corpus not found: {}", corpus.len());

    let mut compared = 0usize;
    let mut skipped = 0usize;
    let mut walk_only = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for (name, (ir_path, _, run_config)) in &corpus {
        let raw = std::fs::read_to_string(ir_path).expect("blessed IR is readable");
        let config = match run_config {
            Some(path) => cfdl_engine::run_config_from_json_file(path, 0.0, None)
                .unwrap_or_else(|err| panic!("{name}: run config: {err}")),
            None => cfdl_engine::RunConfig {
                discount_rate: 0.10,
                ..Default::default()
            },
        };
        // A MODEL WHOSE LOGIC READS CASH IS NOT EXPRESSIBLE IN BOTH ORDERS.
        // The column order settles all state before any stream has a value, so
        // the read binds nothing there and the model means something different
        // — which is exactly the expressiveness the walk adds (`docs/28` §4).
        // Comparing the two would assert that a new capability changes nothing,
        // which is the opposite of what it is for.
        let ir_value: serde_json::Value = serde_json::from_str(&raw).expect("blessed IR parses");
        // An ACCOUNT BALANCE is the same category as a series read, and
        // `docs/28` §5.1 says so: "`prev.<account>` is settled state, strictly
        // backward under §4 — an OC/IC-style trigger tests a reserve balance
        // the same way a delinquency edge tests realised rent." The column
        // order settles all state before any waterfall has moved cash, so the
        // balance reads zero there and the model means something different,
        // exactly as it does for a series read.
        //
        // This clause was missing rather than decided against: the predicate
        // predates accounts (`docs/28` §5.1, results_version 0.4), and no
        // blessed model read a balance in logic until one did.
        let accounts: Vec<String> = ir_value["accounts"]
            .as_array()
            .map(|xs| {
                xs.iter()
                    .filter_map(|a| a["name"].as_str().map(|n| format!("prev.{n}")))
                    .collect()
            })
            .unwrap_or_default();
        let reads_settled_state = |text: String| {
            text.contains("series_") || accounts.iter().any(|a| text.contains(a.as_str()))
        };
        let logic_reads_cash = ["events", "options"].iter().any(|key| {
            ir_value[key]
                .as_array()
                .is_some_and(|xs| xs.iter().any(|x| reads_settled_state(x.to_string())))
        }) || ir_value["entities"].as_array().is_some_and(|xs| {
            xs.iter()
                .any(|e| reads_settled_state(e["rules"].to_string()))
        })
        // A stream that moves an account, or reads one's opening, needs the
        // balance carried period by period (`docs/42` §3); the column order
        // has no period to carry it through and the engine refuses it.
        || ir_value["streams"].as_array().is_some_and(|xs| {
            xs.iter().any(|s| {
                s.get("moves").is_some()
                    || reads_settled_state(s["amount"].to_string())
                    || reads_settled_state(s["active_when"].to_string())
            })
        });
        if logic_reads_cash {
            walk_only += 1;
            continue;
        }

        match cfdl_engine::compare_evaluation_orders(&raw, config) {
            Err(err) => failures.push(format!("{name}: {err}")),
            Ok(None) => skipped += 1,
            Ok(Some((column, walked))) => {
                compared += 1;
                if column.len() != walked.len() {
                    failures.push(format!(
                        "{name}: {} streams by column, {} by walk",
                        column.len(),
                        walked.len()
                    ));
                    continue;
                }
                for (stream, col_values) in &column {
                    let Some(walk_values) = walked.get(stream) else {
                        failures.push(format!("{name}: the walk produced no '{stream}'"));
                        continue;
                    };
                    for (t, (c, w)) in col_values.iter().zip(walk_values).enumerate() {
                        // Exactly equal, not approximately: the two orders step
                        // the same `StreamPlan`, so any difference is a defect
                        // in the ordering rather than in the arithmetic.
                        if c != w {
                            failures.push(format!(
                                "{name}: '{stream}' period {t}: column {c}, walk {w}"
                            ));
                            break;
                        }
                    }
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} disagreement(s) between the two evaluation orders ({compared} models compared, \
         {skipped} skipped as forward-reading):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
    assert!(
        compared >= 95,
        "only {compared} models were comparable; the walk should apply to nearly all of them"
    );
    println!(
        "walk == column on {compared} models; {skipped} skipped as forward-reading, \
         {walk_only} as walk-only (their logic reads cash)"
    );
}

/// The cycle the priced exception refuses, refused (`docs/28` §7).
///
/// Sale proceeds feeding state that feeds what is being capitalized: the
/// exit prices forward rent, the machine's edge reads realised exit
/// proceeds, and the rent is gated on the state the proceeds moved. No
/// evaluation order serves it, so the engine refuses with the path named —
/// at prepare, like every other cycle, because evaluation order is an
/// engine concept. The IR is a committed asset: the model compiles (the
/// cycle crosses the machine, which series-level wave analysis cannot see),
/// so it cannot live in `fixtures/invalid`.
#[test]
fn the_priced_cycle_is_refused_with_the_path_named() {
    let raw = include_str!("data/priced_amount_cycle.ir.json");
    let err = cfdl_engine::run_from_json_str(raw, Default::default())
        .expect_err("the priced cycle must be refused");
    let message = err.to_string();
    assert!(
        message.contains("held -> sold") && message.contains("core.exit"),
        "the refusal names the path: {message}"
    );
}
