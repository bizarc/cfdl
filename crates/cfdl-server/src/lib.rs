//! CFDL HTTP API server.
//!
//! A small, filesystem-free axum service exposing the compiler and engine over
//! embedded packs: `POST /v1/compile|validate|run`, `GET /healthz`, plus an
//! OpenAPI document at `/openapi.json` and Swagger UI at `/docs`.
//!
//! Requests carry an in-memory `files` map (compiled via
//! `cfdl_compile::compile_sources_to_json`), never a filesystem path, and are
//! bounded by [`limits`]: a 1 MiB body, a 10 s timeout, and a Monte Carlo
//! trial cap. Packs come only from the embedded registry.

pub mod limits;

use std::collections::BTreeMap;

use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

/// A compile/validate request: an in-memory file map plus the entry module.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SourcesRequest {
    /// Root-relative path -> source text. Must contain `root_file`.
    pub files: BTreeMap<String, String>,
    /// Entry module path (default `"model.cfdl"`).
    #[serde(default = "default_root_file")]
    pub root_file: String,
}

fn default_root_file() -> String {
    "model.cfdl".to_string()
}

/// A run request: either compiled `ir`, or `files` to compile first.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RunRequest {
    /// Pre-compiled IR JSON. When absent, `files` is compiled first.
    #[serde(default)]
    pub ir: Option<String>,
    #[serde(default)]
    pub files: Option<BTreeMap<String, String>>,
    #[serde(default = "default_root_file")]
    pub root_file: String,
    /// Run configuration (same shape as a `run.json`).
    #[serde(default)]
    pub config: Option<serde_json::Value>,
    /// Fallback annual discount rate when the config omits one.
    #[serde(default)]
    pub rate: f64,
    /// Domain pack for post-engine metrics (e.g. `"cre"`).
    #[serde(default)]
    pub pack: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DiagnosticsResponse {
    pub ok: bool,
    pub diagnostics: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub ok: bool,
    pub error: String,
}

fn diagnostics_response(
    status: StatusCode,
    diagnostics: Vec<cfdl_compile::Diagnostic>,
) -> Response {
    let payload = DiagnosticsResponse {
        ok: false,
        diagnostics: diagnostics
            .iter()
            .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null))
            .collect(),
    };
    (status, Json(payload)).into_response()
}

fn error_response(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorResponse {
            ok: false,
            error: message.into(),
        }),
    )
        .into_response()
}

#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "OK")))]
async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[utoipa::path(
    post, path = "/v1/compile",
    request_body = SourcesRequest,
    responses(
        (status = 200, description = "Compiled IR JSON"),
        (status = 422, description = "Compile diagnostics", body = DiagnosticsResponse),
    )
)]
async fn compile(Json(req): Json<SourcesRequest>) -> Response {
    match cfdl_compile::compile_sources_to_json(
        &req.files,
        &req.root_file,
        &cfdl_compile::CompileOptions::default(),
    ) {
        Ok(ir) => match serde_json::from_str::<serde_json::Value>(&ir) {
            Ok(value) => Json(value).into_response(),
            Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        },
        Err(diags) => diagnostics_response(StatusCode::UNPROCESSABLE_ENTITY, diags),
    }
}

#[utoipa::path(
    post, path = "/v1/validate",
    request_body = SourcesRequest,
    responses(
        (status = 200, description = "Valid"),
        (status = 422, description = "Validation diagnostics", body = DiagnosticsResponse),
    )
)]
async fn validate(Json(req): Json<SourcesRequest>) -> Response {
    match cfdl_compile::validate_sources(&req.files, &req.root_file) {
        Ok(()) => Json(serde_json::json!({"ok": true})).into_response(),
        Err(diags) => diagnostics_response(StatusCode::UNPROCESSABLE_ENTITY, diags),
    }
}

#[utoipa::path(
    post, path = "/v1/run",
    request_body = RunRequest,
    responses(
        (status = 200, description = "Results JSON"),
        (status = 422, description = "Compile diagnostics", body = DiagnosticsResponse),
        (status = 400, description = "Bad request / limit exceeded", body = ErrorResponse),
    )
)]
async fn run(Json(req): Json<RunRequest>) -> Response {
    // Obtain IR: use provided IR, else compile the files.
    let ir_json = match req.ir {
        Some(ir) => ir,
        None => {
            let Some(files) = req.files.as_ref() else {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "run requires either `ir` or `files`",
                );
            };
            match cfdl_compile::compile_sources_to_json(
                files,
                &req.root_file,
                &cfdl_compile::CompileOptions::default(),
            ) {
                Ok(ir) => ir,
                Err(diags) => return diagnostics_response(StatusCode::UNPROCESSABLE_ENTITY, diags),
            }
        }
    };

    // Parse run config (fallback rate applied), then enforce the MC trial cap.
    let config_result = match req.config {
        Some(value) => {
            let raw = value.to_string();
            cfdl_engine::run_config_from_json_str(&raw, req.rate, None)
        }
        None => Ok(cfdl_engine::RunConfig {
            discount_rate: req.rate,
            ..Default::default()
        }),
    };
    let config = match config_result {
        Ok(config) => config,
        Err(err) => return error_response(StatusCode::BAD_REQUEST, err.to_string()),
    };
    if let Some(mc) = config.monte_carlo.as_ref() {
        if mc.trial_count > limits::MAX_MC_TRIALS {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "monte_carlo trial_count {} exceeds the limit of {}",
                    mc.trial_count,
                    limits::MAX_MC_TRIALS
                ),
            );
        }
    }

    // The engine is synchronous and CPU-bound; run it off the async runtime.
    let pack = req.pack.clone();
    let join = tokio::task::spawn_blocking(move || {
        let mut results = cfdl_engine::run_from_json_str(&ir_json, config)?;
        if let Some(pack_name) = pack {
            let registry = cfdl_pack::PackRegistry::load_embedded().ok();
            let specs = registry
                .as_ref()
                .map(|reg| reg.metric_specs(&pack_name))
                .unwrap_or_default();
            results.domain_metrics = cfdl_metrics::compute(&pack_name, &specs, &results);
            // Statements read a stream's CATEGORY, which lives on the IR rather
            // than in results. Parsing it back here keeps the results document from
            // carrying a field only this consumer wants.
            let statement_specs = registry
                .as_ref()
                .map(|reg| reg.statement_specs(&pack_name))
                .unwrap_or_default();
            let categories = serde_json::from_str::<serde_json::Value>(&ir_json)
                .ok()
                .map(|ir| cfdl_statement::stream_categories(&ir))
                .unwrap_or_default();
            results.statements =
                cfdl_statement::compute(&pack_name, &statement_specs, &categories, &results);
        }
        Ok::<_, cfdl_engine::EngineError>(results)
    })
    .await;

    match join {
        Ok(Ok(results)) => match serde_json::to_value(&results) {
            Ok(value) => Json(value).into_response(),
            Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        },
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err.to_string()),
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(healthz, compile, validate, run),
    components(schemas(SourcesRequest, RunRequest, DiagnosticsResponse, ErrorResponse)),
    info(
        title = "CFDL API",
        description = "Compile, validate, and run CFDL models over embedded packs. Limits: 1 MiB body, 10 s timeout, Monte Carlo trials capped.",
    )
)]
pub struct ApiDoc;

/// Build the application router (used by `main` and integration tests).
pub fn app() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/compile", post(compile))
        .route("/v1/validate", post(validate))
        .route("/v1/run", post(run))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            limits::REQUEST_TIMEOUT,
        ))
        // Two layers: axum's own extractor limit + a tower-http byte cap.
        .layer(DefaultBodyLimit::max(limits::MAX_BODY_BYTES))
        .layer(RequestBodyLimitLayer::new(limits::MAX_BODY_BYTES))
}
