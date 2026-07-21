# Security Policy

## Reporting a vulnerability

Please **do not open a public GitHub issue** for security vulnerabilities.

Report privately via GitHub's **"Report a vulnerability"** (Security → Advisories →
Report a vulnerability) on this repository. Include a minimal reproduction (`.cfdl`
model or API call) where possible.

We aim to acknowledge reports within 5 business days.

## Scope

In scope:

- The CFDL compiler, engine, CLI, LSP, and Python bindings (`crates/`, `python/`)
- Sandbox escapes: CFDL models or expressions that read/write the filesystem or
  network, execute arbitrary code, or fail to terminate
- Panics/crashes reachable from untrusted `.cfdl` source, IR JSON, or run configs

Out of scope:

- Resource exhaustion from deliberately huge but well-formed models run locally
- Issues in third-party dependencies without a demonstrated CFDL-reachable impact

## Supported versions

Pre-1.0: only the latest release receives security fixes.
