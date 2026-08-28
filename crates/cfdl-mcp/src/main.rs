//! `cfdl-mcp`: the CFDL agent toolkit over MCP.
//!
//! Usage:
//!   `cfdl-mcp [--repo <dir>] [--packs <dir>] [--benchmarks <dir>]`          stdio (default)
//!   `cfdl-mcp --http <addr> [...]`                                          Streamable HTTP at /mcp
//!
//! With no directory flags, `packs/` and `benchmarks/` are detected under
//! the working directory; without either, packs come from the embedded
//! registry.
//!
//! HTTP mode is for remote agents. When the `CFDL_MCP_TOKEN` environment
//! variable is set, every request must carry `Authorization: Bearer <token>`
//! — set it before exposing the port beyond localhost. The token rides in
//! the environment rather than argv so `ps` does not print it.

use std::path::PathBuf;
use std::sync::Arc;

use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::ServiceExt;

use cfdl_mcp::{CfdlMcp, Defaults};

struct Options {
    defaults: Defaults,
    http: Option<String>,
    allowed_hosts: Vec<String>,
}

fn parse_args() -> Result<Options, String> {
    let mut options = Options {
        defaults: Defaults::from_root(&PathBuf::from(".")),
        http: None,
        allowed_hosts: Vec::new(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = |flag: &str| {
            args.next()
                .ok_or_else(|| format!("{flag} requires an argument"))
        };
        match flag.as_str() {
            "--repo" => options.defaults = Defaults::from_root(&PathBuf::from(value("--repo")?)),
            "--packs" => options.defaults.packs_dir = Some(PathBuf::from(value("--packs")?)),
            "--benchmarks" => {
                options.defaults.benchmarks_dir = Some(PathBuf::from(value("--benchmarks")?))
            }
            "--http" => options.http = Some(value("--http")?),
            "--allowed-host" => options.allowed_hosts.push(value("--allowed-host")?),
            "--help" | "-h" => {
                eprintln!(
                    "cfdl-mcp [--repo <dir>] [--packs <dir>] [--benchmarks <dir>] [--http <addr>]\n\
                     \x20        [--allowed-host <host> ...]\n\
                     MCP server exposing compile, run, diff, explain, lookup, skeleton.\n\
                     Default transport is stdio; --http serves Streamable HTTP at /mcp\n\
                     (set CFDL_MCP_TOKEN to require `Authorization: Bearer <token>`).\n\
                     Host validation allows loopback only by default; a public deployment\n\
                     names its own hostname(s) with --allowed-host."
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown flag '{other}'")),
        }
    }
    Ok(options)
}

/// Reject requests without the configured bearer token. Constant shape: the
/// comparison is on full strings; a missing or malformed header is the same
/// 401 as a wrong token.
async fn require_bearer(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let expected = std::env::var("CFDL_MCP_TOKEN").ok();
    let Some(expected) = expected else {
        return next.run(request).await;
    };
    let authorized = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| token == expected);
    if authorized {
        next.run(request).await
    } else {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            "missing or invalid bearer token",
        )
            .into_response()
    }
}

use axum::response::IntoResponse;

async fn serve_http(
    addr: &str,
    defaults: Defaults,
    allowed_hosts: Vec<String>,
) -> anyhow::Result<()> {
    // Loopback-only by default (DNS-rebinding protection); a public
    // deployment names its own hostnames with --allowed-host.
    let mut config = StreamableHttpServerConfig::default();
    config.allowed_hosts = allowed_hosts;
    let service = StreamableHttpService::new(
        move || Ok(CfdlMcp::new(defaults.clone())),
        Arc::new(LocalSessionManager::default()),
        config,
    );
    let app = axum::Router::new()
        .nest_service("/mcp", service)
        .layer(axum::middleware::from_fn(require_bearer));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let secured = std::env::var("CFDL_MCP_TOKEN").is_ok();
    eprintln!(
        "cfdl-mcp: Streamable HTTP at http://{addr}/mcp ({})",
        if secured {
            "bearer token required"
        } else {
            "NO AUTH — do not expose beyond localhost"
        }
    );
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = parse_args().map_err(|err| anyhow::anyhow!(err))?;
    match options.http {
        Some(addr) => serve_http(&addr, options.defaults, options.allowed_hosts).await,
        None => {
            let service = CfdlMcp::new(options.defaults)
                .serve(rmcp::transport::stdio())
                .await?;
            service.waiting().await?;
            Ok(())
        }
    }
}
