//! The docs/32 Phase-1 gate: the MCP tool loop rebuilds an existing benchmark
//! case and matches its expectations. This drives the same functions the wire
//! serves — compile -> run -> diff -> explain — against
//! `benchmarks/cre/office_two_tenant`, the scripted "agent" that separates
//! toolkit bugs from model failures.

use std::path::PathBuf;

use cfdl_mcp::tools::{compile, diff, explain, lookup, run, skeleton, Defaults};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn defaults() -> Defaults {
    Defaults::from_root(&repo_root())
}

const CASE: &str = "benchmarks/cre/office_two_tenant";

#[test]
fn the_loop_rebuilds_a_benchmark_case() {
    let root = repo_root();
    let case_dir = root.join(CASE);

    // compile: the case's model, against the repo packs.
    let compiled = compile::compile(
        &compile::CompileParams {
            model_dir: Some(case_dir.to_string_lossy().into_owned()),
            ..Default::default()
        },
        &defaults(),
    )
    .expect("compile call");
    assert!(compiled.ok, "diagnostics: {:?}", compiled.diagnostics);

    // run: the case's run.json and pack, results kept inline.
    let ran = run::run(
        &run::RunParams {
            model_dir: Some(case_dir.to_string_lossy().into_owned()),
            config_path: Some(case_dir.join("run.json").to_string_lossy().into_owned()),
            pack: Some("cre".to_string()),
            ..Default::default()
        },
        &defaults(),
    )
    .expect("run call");
    assert!(ran.ok, "run failed: {:?} {:?}", ran.error, ran.diagnostics);
    assert!(
        ran.warnings.is_empty(),
        "engine warnings: {:?}",
        ran.warnings
    );
    assert!(ran.series.contains(&"domain.cre.noi".to_string()));
    let results = ran.results.clone().expect("inline results");

    // diff: against the case's own expectations — the byte-derived ground truth.
    let diffed = diff::diff(
        &diff::DiffParams {
            results: Some(results.clone()),
            case_dir: Some(case_dir.to_string_lossy().into_owned()),
            ..Default::default()
        },
        &defaults(),
    )
    .expect("diff call");
    assert!(
        diffed.pass,
        "diff failures: {:#?} (first: {:#?})",
        diffed.failures, diffed.first_divergence
    );
    assert!(diffed.checked.rows > 0 && diffed.checked.metrics > 0);

    // explain: a mid-timeline NOI number matches expected.csv's asserted value.
    let explained = explain::explain(
        &explain::ExplainParams {
            results: Some(results),
            results_path: None,
            series: "domain.cre.noi".to_string(),
            period: 6,
        },
        &defaults(),
    )
    .expect("explain call");
    let value = explained.value.expect("noi at period 6");
    assert!((value - 46500.0).abs() < 0.01, "noi period 6: {value}");
}

#[test]
fn diff_reports_a_seeded_divergence() {
    // The repair signal: perturb one number and the diff must localize it.
    let root = repo_root();
    let case_dir = root.join(CASE);
    let ran = run::run(
        &run::RunParams {
            model_dir: Some(case_dir.to_string_lossy().into_owned()),
            config_path: Some(case_dir.join("run.json").to_string_lossy().into_owned()),
            pack: Some("cre".to_string()),
            ..Default::default()
        },
        &defaults(),
    )
    .expect("run call");
    let mut results = ran.results.expect("inline results");
    results["deterministic"]["series"]["domain.cre.noi"]["values"][6] = serde_json::json!(999999.0);

    let diffed = diff::diff(
        &diff::DiffParams {
            results: Some(results),
            case_dir: Some(case_dir.to_string_lossy().into_owned()),
            ..Default::default()
        },
        &defaults(),
    )
    .expect("diff call");
    assert!(!diffed.pass);
    let first = diffed.first_divergence.expect("first divergence");
    assert_eq!(first.label, "domain.cre.noi");
    assert_eq!(first.period, Some(6));
}

#[test]
fn lookup_terms_and_pack_roster() {
    let found = lookup::lookup(
        &lookup::LookupParams {
            term: Some("stream".to_string()),
            ..Default::default()
        },
        &defaults(),
    )
    .expect("lookup call");
    assert!(
        found.terms.iter().any(|t| t.term == "stream"),
        "terms: {:?}",
        found.terms.iter().map(|t| &t.term).collect::<Vec<_>>()
    );

    let found = lookup::lookup(
        &lookup::LookupParams {
            pack: Some("cre".to_string()),
            ..Default::default()
        },
        &defaults(),
    )
    .expect("pack lookup");
    let pack = found.pack.expect("pack info");
    assert!(!pack.contracts.is_empty());
    assert!(!pack.templates.is_empty());
    // Coverage: cre.lease_unit is exercised by office_two_tenant among others.
    let lease = pack
        .contracts
        .iter()
        .find(|c| c.contract_name.as_deref() == Some("cre.lease_unit"))
        .expect("cre.lease_unit in roster");
    let exercised = lease.exercised_by.as_ref().expect("coverage scanned");
    assert!(
        exercised.iter().any(|case| case == "cre/office_two_tenant"),
        "exercised_by: {exercised:?}"
    );
    // The type is read against its master chain (docs/40): the master, the
    // effective roster with the master's fields inherited, and each role
    // beside the master's word for it.
    assert_eq!(lease.master.as_deref(), Some("Contract.Lease"));
    assert_eq!(lease.refines.as_deref(), Some("CRE.Contract.Lease"));
    let rent = lease
        .fields
        .iter()
        .find(|f| f.name == "rent_year")
        .expect("rent_year in the roster");
    assert!(rent.required);
    assert!(
        lease.fields.iter().any(|f| f.name == "free_rent_months"),
        "the master's field is inherited"
    );
    let landlord = lease
        .roles
        .iter()
        .find(|r| r.name == "landlord")
        .expect("landlord role");
    assert_eq!(landlord.master, "lessor");
    assert!(lease.lines.iter().any(|l| l == "rent"));
    let debt = pack
        .masters
        .iter()
        .find(|m| m.type_id == "Contract.Debt")
        .expect("the pack refines Contract.Debt");
    assert!(debt.is_abstract);
    assert!(debt.fields.iter().any(|f| f.name == "interest_rate"));
}

#[test]
fn lookup_reads_a_master_by_name() {
    let found = lookup::lookup(
        &lookup::LookupParams {
            master: Some("Contract.Security".to_string()),
            ..Default::default()
        },
        &defaults(),
    )
    .expect("master lookup");
    let master = found.master.expect("master info");
    assert!(master.is_abstract);
    assert!(master.fields.iter().any(|f| f.name == "face" && f.required));
    assert!(
        master.lines.iter().any(|l| l == "principal (allocated)"),
        "{:?}",
        master.lines
    );
    assert!(found.pack.is_none() && found.packs.is_empty());
}

#[test]
fn skeleton_names_the_refinements_of_a_master() {
    let err = skeleton::skeleton(
        &skeleton::SkeletonParams {
            pack: "cre".to_string(),
            contract_types: Some(vec!["Contract.Debt".to_string()]),
            calendar: None,
            periods: None,
            start: None,
            template_params: None,
            packs_dir: None,
        },
        &defaults(),
    )
    .expect_err("a master is never declared");
    assert!(
        err.contains("cre.permanent_debt") && err.contains("master"),
        "{err}"
    );
    // An ontology type id resolves to the pack's template for it.
    let built = skeleton::skeleton(
        &skeleton::SkeletonParams {
            pack: "cre".to_string(),
            contract_types: Some(vec!["CRE.Contract.PermanentDebt".to_string()]),
            calendar: None,
            periods: None,
            start: None,
            template_params: None,
            packs_dir: None,
        },
        &defaults(),
    )
    .expect("type id resolves");
    assert!(
        built
            .templates_used
            .iter()
            .any(|t| t == "cre.permanent_debt"),
        "{:?}",
        built.templates_used
    );
}

#[test]
fn skeleton_compiles_for_each_pack_with_templates() {
    let registry_packs = ["cre", "credit", "energy", "opco"];
    for pack in registry_packs {
        let built = skeleton::skeleton(
            &skeleton::SkeletonParams {
                pack: pack.to_string(),
                contract_types: None,
                calendar: None,
                periods: None,
                start: None,
                template_params: None,
                packs_dir: None,
            },
            &defaults(),
        );
        let Ok(built) = built else {
            // A pack that ships no templates is allowed to say so.
            continue;
        };
        assert!(
            built.ok,
            "skeleton for '{pack}' does not compile: {:?}\n{}",
            built.diagnostics, built.model
        );
    }
}
