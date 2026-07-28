//! Source-string (in-memory) compile API tests: parity with the filesystem
//! path and preserved import diagnostics (E1202/E1203).

use std::collections::BTreeMap;
use std::path::PathBuf;

use cfdl_compile::{compile_sources_to_json, compile_to_json_with_options, CompileOptions};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn canon(json: &str) -> String {
    let value: serde_json::Value = serde_json::from_str(json).expect("valid json");
    serde_json::to_string(&value).expect("reserialize")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).expect("read fixture")
}

/// In-memory compile of a fixture equals the filesystem compile of the same
/// fixture (which the golden suite pins to gold/ir).
#[test]
fn memory_matches_filesystem_for_minimal_model() {
    let root = repo_root();
    let fs_ir = compile_to_json_with_options(
        &root.join("fixtures/valid/minimal_model"),
        &CompileOptions::default(),
    )
    .expect("fs compile");

    let mut files = BTreeMap::new();
    files.insert(
        "model.cfdl".to_string(),
        read("fixtures/valid/minimal_model/model.cfdl"),
    );
    let mem_ir =
        compile_sources_to_json(&files, "model.cfdl", &CompileOptions::default()).expect("mem");

    assert_eq!(canon(&mem_ir), canon(&fs_ir));
}

/// A pack model compiles from memory when given a packs directory, matching
/// the gold IR.
#[test]
fn memory_with_pack_matches_gold() {
    let mut files = BTreeMap::new();
    files.insert(
        "model.cfdl".to_string(),
        read("fixtures/valid/credit_pool_smoke/model.cfdl"),
    );
    let options = CompileOptions {
        packs_dir: Some(repo_root().join("packs")),
    };
    let mem_ir = compile_sources_to_json(&files, "model.cfdl", &options).expect("mem compile");
    let gold = read("gold/ir/credit_pool_smoke.json");
    assert_eq!(canon(&mem_ir), canon(&gold));
}

/// Multi-file imports resolve from the in-memory map.
#[test]
fn memory_resolves_imports() {
    let mut files = BTreeMap::new();
    files.insert(
        "model.cfdl".to_string(),
        "version 0.1\nmodel \"m\"\nimport \"sub/time.cfdl\"\nentity legal borrower\n".to_string(),
    );
    files.insert(
        "sub/time.cfdl".to_string(),
        "time calendar monthly from 2026-01 for 3\n".to_string(),
    );
    let ir = compile_sources_to_json(&files, "model.cfdl", &CompileOptions::default())
        .expect("compile with import");
    assert!(ir.contains("\"periods\": 3"));
}

/// An import escaping the model root is E1203, in memory as on disk.
#[test]
fn memory_import_escape_is_e1203() {
    let mut files = BTreeMap::new();
    files.insert(
        "model.cfdl".to_string(),
        "version 0.1\nmodel \"m\"\nimport \"../outside.cfdl\"\nentity legal borrower\n".to_string(),
    );
    let err = compile_sources_to_json(&files, "model.cfdl", &CompileOptions::default())
        .expect_err("should fail");
    assert!(err
        .iter()
        .any(|d| d.code == "E1203_IMPORT_OUTSIDE_MODEL_ROOT"));
}

/// With the `embedded-packs` feature and no packs directory, a `use pack`
/// model resolves against the bundled registry (the WASM/server path).
#[cfg(feature = "embedded-packs")]
#[test]
fn memory_uses_embedded_packs_without_dir() {
    let mut files = BTreeMap::new();
    files.insert(
        "model.cfdl".to_string(),
        read("fixtures/valid/credit_pool_smoke/model.cfdl"),
    );
    // No packs_dir: must fall back to the embedded registry.
    let ir = compile_sources_to_json(&files, "model.cfdl", &CompileOptions::default())
        .expect("embedded pack compile");
    let gold = read("gold/ir/credit_pool_smoke.json");
    assert_eq!(canon(&ir), canon(&gold));
}

/// A missing imported module is E1202.
#[test]
fn memory_missing_import_is_e1202() {
    let mut files = BTreeMap::new();
    files.insert(
        "model.cfdl".to_string(),
        "version 0.1\nmodel \"m\"\nimport \"missing.cfdl\"\nentity legal borrower\n".to_string(),
    );
    let err = compile_sources_to_json(&files, "model.cfdl", &CompileOptions::default())
        .expect_err("should fail");
    assert!(err.iter().any(|d| d.code == "E1202_IMPORT_NOT_FOUND"));
}
