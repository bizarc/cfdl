//! Integration tests driving the router directly via `oneshot`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

use cfdl_server::app;

const MODEL: &str = "version 0.1\nmodel \"m\"\ntime calendar monthly from 2026-01 for 3\nentity legal borrower\nstream legal.rent on entity legal.borrower {\n  schedule every monthly from 2026-01 to 2026-03\n  amount = 1000\n}\n";

async fn post(path: &str, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, value)
}

#[tokio::test]
async fn healthz_ok() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn compile_happy_path() {
    let (status, body) = post("/v1/compile", json!({ "files": { "model.cfdl": MODEL } })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ir_version"], "0.1");
}

#[tokio::test]
async fn compile_invalid_returns_diagnostics() {
    let bad = "version 0.1\nmodel \"m\"\ncontract test.loan on entity legal.borrower {\n  terms { x = 1 }\n}\n";
    let (status, body) = post("/v1/compile", json!({ "files": { "model.cfdl": bad } })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["ok"], false);
    assert!(!body["diagnostics"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn run_from_files() {
    let (status, body) = post(
        "/v1/run",
        json!({ "files": { "model.cfdl": MODEL }, "rate": 0.10 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["results_version"], "0.2");
    assert!(body["deterministic"]["metrics"]["model.npv"].is_object());
}

#[tokio::test]
async fn run_with_embedded_pack_metrics() {
    // A model using the credit pack resolves domain metrics from the embedded
    // registry (no packs directory).
    let model = "version 0.1\nmodel \"p\"\nuse pack \"credit\" version \"0.1.0\"\ntime calendar monthly from 2026-01 for 6\nentity fund buyer\ncontract credit.pool_io_bullet.a on entity fund.buyer {\n  term 2026-01..2026-06\n  terms { balance = 1000000 rate = 0.07 term_months = 6 }\n}\n";
    let (status, body) = post(
        "/v1/run",
        json!({ "files": { "model.cfdl": model }, "rate": 0.08, "pack": "credit" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["domain_metrics"]["pack"], "credit");
}

#[tokio::test]
async fn run_rejects_excessive_mc_trials() {
    let (status, body) = post(
        "/v1/run",
        json!({
            "files": { "model.cfdl": MODEL },
            "config": { "monte_carlo": { "trial_count": 100000, "seed": 1 } }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("trial_count"));
}

#[tokio::test]
async fn body_limit_rejects_oversized_request() {
    // > 1 MiB body is rejected before handler logic.
    let big = "x".repeat((1 << 20) + 1024);
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/compile")
                .header("content-type", "application/json")
                .body(Body::from(big))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn openapi_document_served() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(doc["paths"]["/v1/run"].is_object());
}
