//! MCP wire glue: six tools over [`crate::tools`], served on stdio.

use std::sync::Arc;

use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    model::{ErrorData, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};

use crate::tools::{self, Defaults};

const INSTRUCTIONS: &str =
    "CFDL is a declarative cash-flow modeling language with a deterministic \
compile -> run cycle and structured diagnostics. The authoring loop: `skeleton` (or `lookup`) to \
start from a valid model, `compile` to get diagnostics, `run` to get results, `explain` to trace \
any number to the journal entries that produced it — and, when expectation files are available to \
you, `diff` to compare results against them (some deployments withhold `diff`; the loop is \
complete without it). \
Diagnostics and diff failures are data for repair, not errors. The evaluation model: a CONTRACT \
declaration (`contract <pack>.<type>.<instance> on entity ...`) LOWERS to streams through its \
pack's lowering rule — contracts are vocabulary, streams are the cash; streams evaluate first on \
their schedules, then logic/events act on those amounts, then financing and distributions run \
over the aggregated flows. Use `lookup` with a pack name for the contract types and the rule each \
one lowers through.";

#[derive(Clone)]
pub struct CfdlMcp {
    defaults: Arc<Defaults>,
}

fn invalid(err: String) -> ErrorData {
    ErrorData::invalid_params(err, None)
}

#[tool_router]
impl CfdlMcp {
    pub fn new(defaults: Defaults) -> Self {
        Self {
            defaults: Arc::new(defaults),
        }
    }

    #[tool(
        description = "Compile a CFDL model to IR. Returns ok plus structured diagnostics (docs/08) on failure — the repair signal. Source is a model_dir on disk or an in-memory files map."
    )]
    fn compile(
        &self,
        Parameters(params): Parameters<tools::compile::CompileParams>,
    ) -> Result<Json<tools::compile::CompileResult>, ErrorData> {
        tools::compile::compile(&params, &self.defaults)
            .map(Json)
            .map_err(invalid)
    }

    #[tool(
        description = "Compile (if needed) and run a CFDL model: source or IR plus a run configuration (run.json shape) -> results per docs/06, enriched with domain metrics and statements when a pack is named. Set `out` to write results to a file and get a summary back."
    )]
    async fn run(
        &self,
        Parameters(params): Parameters<tools::run::RunParams>,
    ) -> Result<Json<tools::run::RunResult>, ErrorData> {
        let defaults = self.defaults.clone();
        // The engine is synchronous and CPU-bound; run it off the async runtime.
        tokio::task::spawn_blocking(move || tools::run::run(&params, &defaults))
            .await
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?
            .map(Json)
            .map_err(invalid)
    }

    #[tool(
        description = "Compare results against expectations: a benchmark case_dir (expected.csv, expected_metrics.json, case.toml tolerances) or standalone expectation files. Reports the first divergence and per-period/metric deltas, with the benchmark harness's tolerance discipline."
    )]
    fn diff(
        &self,
        Parameters(params): Parameters<tools::diff::DiffParams>,
    ) -> Result<Json<tools::diff::DiffResult>, ErrorData> {
        tools::diff::diff(&params, &self.defaults)
            .map(Json)
            .map_err(invalid)
    }

    #[tool(
        description = "Explain one number: a series key + period (0-based) -> its value, neighbors, and the journal entries (actor, action, target, amounts, pot levels) that produced it. A plain scheduled stream has no journal rows; waterfalls, events, and accounts do."
    )]
    fn explain(
        &self,
        Parameters(params): Parameters<tools::explain::ExplainParams>,
    ) -> Result<Json<tools::explain::ExplainResult>, ErrorData> {
        tools::explain::explain(&params, &self.defaults)
            .map(Json)
            .map_err(invalid)
    }

    #[tool(
        description = "Look up a term in the CFDL terminology register (the glossary's source), or a pack for its contract roster (with benchmark coverage), templates, and metrics. With no arguments, lists the available packs."
    )]
    fn lookup(
        &self,
        Parameters(params): Parameters<tools::lookup::LookupParams>,
    ) -> Result<Json<tools::lookup::LookupResult>, ErrorData> {
        tools::lookup::lookup(&params, &self.defaults)
            .map(Json)
            .map_err(invalid)
    }

    #[tool(
        description = "Generate a minimal valid CFDL model for a pack from its own templates — a starting point to grow. The skeleton is compiled AND run before it is returned: `ok` is the compile, `run_ok` says the run produced no engine warnings, and `warnings`/`notes` say what to fill in (stubbed curves included)."
    )]
    fn skeleton(
        &self,
        Parameters(params): Parameters<tools::skeleton::SkeletonParams>,
    ) -> Result<Json<tools::skeleton::SkeletonResult>, ErrorData> {
        tools::skeleton::skeleton(&params, &self.defaults)
            .map(Json)
            .map_err(invalid)
    }
}

#[tool_handler]
impl ServerHandler for CfdlMcp {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.instructions = Some(INSTRUCTIONS.to_string());
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info.name = "cfdl-mcp".to_string();
        info.server_info.version = env!("CARGO_PKG_VERSION").to_string();
        info
    }
}
