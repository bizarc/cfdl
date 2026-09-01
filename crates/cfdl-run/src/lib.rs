//! The shared run facade.
//!
//! Post-engine enrichment — domain metrics and statements — used to live as a
//! copy-pasted block in `cfdl-cli` and `cfdl-server`. Every surface that runs
//! a model (CLI, HTTP server, MCP server) now calls [`enrich_results`], so a
//! change to what a "run with a pack" produces happens in exactly one place.

use cfdl_engine::{EngineError, Results, RunConfig};
use cfdl_pack::PackRegistry;

/// Attach domain metrics and statements to an engine result.
///
/// `ir_json` is the IR the run came from. Statements read a stream's CATEGORY,
/// which lives on the IR rather than in results — passing `None` (or an
/// unparseable IR) skips statements but still computes metrics, which is the
/// behavior both prior copies had.
pub fn enrich_results(
    results: &mut Results,
    ir_json: Option<&str>,
    pack_name: &str,
    registry: Option<&PackRegistry>,
) {
    let specs = registry
        .map(|reg| reg.metric_specs(pack_name))
        .unwrap_or_default();
    results.domain_metrics = cfdl_metrics::compute(pack_name, &specs, results);

    let subtotal_specs = registry
        .map(|reg| reg.subtotal_specs(pack_name))
        .unwrap_or_default();
    let statement_specs = registry
        .map(|reg| reg.statement_specs(pack_name))
        .unwrap_or_default();
    // Parsed once: the statement needs both what each stream is and which
    // series are waterfall steps rather than cash.
    let ir_value = ir_json.and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok());
    let categories = ir_value
        .as_ref()
        .map(cfdl_statement::stream_categories)
        .unwrap_or_default();
    let waterfall_series = ir_value
        .as_ref()
        .map(cfdl_statement::waterfall_series)
        .unwrap_or_default();
    // The pack's RECOMMENDED vocabulary, for `W5023`. It is advice about
    // spelling, and its consequence is a presentation one — an unrecommended
    // category is the category no statement row claims — so it is reported
    // where the statement's own completeness diagnostics are, beside `W3500`.
    // `results.warnings` belongs to the engine, which has no pack.
    let recommended = registry
        .and_then(|reg| reg.pack(pack_name))
        .map(|pack| pack.manifest.categories.clone())
        .unwrap_or_default();
    results.statements = cfdl_statement::compute(
        pack_name,
        &statement_specs,
        &subtotal_specs,
        &categories,
        &waterfall_series,
        &recommended,
        results,
    );
    // A MODEL'S OWN STATEMENTS, beside the pack's (`docs/13` §7.55). One
    // evaluator, two producers: a pack enumerates its rows, a model names a
    // structure and a depth and the rows follow from the tree. The pack path
    // above is untouched, so a packed model renders exactly what it rendered.
    let model_statements = ir_value
        .as_ref()
        .map(cfdl_statement::model_statements)
        .unwrap_or_default();
    // Called unconditionally: with no declared statements this is where the
    // DEFAULT presentation is assembled (`docs/13` §7.43), and guarding on an
    // empty list here would make that unreachable.
    cfdl_statement::attach_model_statements(
        &model_statements,
        &categories,
        &waterfall_series,
        results,
    );
}

/// Run an in-memory IR and enrich the results when a pack is named.
pub fn run_enriched(
    ir_json: &str,
    config: RunConfig,
    pack_name: Option<&str>,
    registry: Option<&PackRegistry>,
) -> Result<Results, EngineError> {
    let mut results = cfdl_engine::run_from_json_str(ir_json, config)?;
    if let Some(pack_name) = pack_name {
        enrich_results(&mut results, Some(ir_json), pack_name, registry);
    }
    Ok(results)
}
