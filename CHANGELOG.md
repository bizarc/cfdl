# Changelog

All notable changes to this project will be documented in this file.

This project follows Semantic Versioning: https://semver.org/

---

## [0.3.0] - Unreleased

First public release. CFDL is pre-1.0: the language and IR spec is v0.1, and
interfaces may change until 1.0 freezes the IR and Results schemas.

### Language and engine

- Deterministic compilation: the same sources, pack version and compiler
  version emit byte-identical IR, enforced by a golden suite.
- Native `cfdl-calc` expression engine with decimal-exact money arithmetic and
  an Excel-compatible function library (annuities, day counts, business-day
  calendars, MACRS, prepayment conversions).
- Deterministic DCF, scenarios, and seeded Monte Carlo, emitting
  schema-governed Results JSON.

### Domain packs

- `energy`, `cre`, `credit` and `opco`, each supplying contract types,
  template-driven lowering rules, domain metrics, and declarative validations.
- Every pack is gated by a parity suite: each model is diffed period-by-period
  against an independent reference implementation.

### Surfaces

- CLI (`cfdl compile`, `cfdl run`, `cfdl validate`).
- Python SDK (`cfdl_sdk`) with pandas result accessors.
- WebAssembly build powering the in-browser playground at cfdl.dev.
- HTTP API server, and a VS Code extension with LSP diagnostics.

### Licensing

- Business Source License 1.1 (source available, not open source). Each
  released version converts to Apache-2.0 four years after its release.
