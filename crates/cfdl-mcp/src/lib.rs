//! The CFDL agent toolkit (docs/32 Phase 1): an MCP server exposing the
//! compile -> run -> diff -> explain loop, plus lookup and skeleton, over the
//! same library entry points the CLI and HTTP server use.
//!
//! Tool logic lives in [`tools`] as plain functions; [`service`] is the MCP
//! wire glue. Every tool result is a typed, schema-carrying structure — no
//! free-text tool output.

pub mod service;
pub mod tools;

pub use service::CfdlMcp;
pub use tools::Defaults;
