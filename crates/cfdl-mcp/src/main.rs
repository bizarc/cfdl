//! `cfdl-mcp`: the CFDL agent toolkit over MCP stdio.
//!
//! Usage: `cfdl-mcp [--repo <dir>] [--packs <dir>] [--benchmarks <dir>]`
//! With no flags, `packs/` and `benchmarks/` are detected under the working
//! directory; without either, packs come from the embedded registry.

use std::path::PathBuf;

use rmcp::ServiceExt;

use cfdl_mcp::{CfdlMcp, Defaults};

fn parse_args() -> Result<Defaults, String> {
    let mut defaults = Defaults::from_root(&PathBuf::from("."));
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = |flag: &str| {
            args.next()
                .ok_or_else(|| format!("{flag} requires a directory argument"))
        };
        match flag.as_str() {
            "--repo" => defaults = Defaults::from_root(&PathBuf::from(value("--repo")?)),
            "--packs" => defaults.packs_dir = Some(PathBuf::from(value("--packs")?)),
            "--benchmarks" => defaults.benchmarks_dir = Some(PathBuf::from(value("--benchmarks")?)),
            "--help" | "-h" => {
                eprintln!(
                    "cfdl-mcp [--repo <dir>] [--packs <dir>] [--benchmarks <dir>]\n\
                     MCP server (stdio) exposing compile, run, diff, explain, lookup, skeleton."
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown flag '{other}'")),
        }
    }
    Ok(defaults)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let defaults = parse_args().map_err(|err| anyhow::anyhow!(err))?;
    let service = CfdlMcp::new(defaults)
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
