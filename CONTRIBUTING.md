# Contributing

This repo is the reference implementation workspace for **CFDL v0.1** (Cash Flow Domain Language), including the Rust compiler toolchain, CLI, tooling bindings, and domain pack interface.

The authoritative specifications live under `docs/`. Implementation changes must conform to those documents.

## Ground rules

* **Determinism is required**: same inputs + same pack + same compiler version must produce identical IR output.
* **Diagnostics codes are stable**: never reuse or rename existing diagnostic codes.
* **Golden-first**: any feature or bug fix must add/update a fixture and a gold output.
* **No correlation**: do not add correlation fields/slots in the language or IR.

## Repo conventions

* Rust workspace crates live under `crates/`.
* Fixtures live under `fixtures/`:

  * `fixtures/valid/<fixture_name>/...`
  * `fixtures/invalid/<fixture_name>/...`
* Expected outputs live under `gold/`:

  * `gold/ir/<fixture_name>.json`
  * `gold/diag/<fixture_name>.diag.json`
* Specs live under `docs/` and are treated as the source of truth.

## Setup

### Prereqs

* Rust (pinned by `rust-toolchain.toml`)
* Recommended: `make` (once the Makefile is added)

### Common commands

```bash
# Format
cargo fmt

# Lint
cargo clippy --all-targets --all-features

# Test
cargo test

# (When added) Run golden suite
make gold
```

## Development workflow

1. Pick the smallest next task from `AGENTS.md`.
2. Implement the minimal change required.
3. Add/update fixtures in `fixtures/`.
4. Add/update expected outputs in `gold/`.
5. Run:

   * `cargo fmt`
   * `cargo clippy --all-targets --all-features`
   * `cargo test`
   * golden runner (when present)
6. Open a PR with:

   * what changed
   * which fixtures/gold files were added or updated
   * any spec references (e.g., `docs/compiler_spec_v0_1.md` section)

## Pull request checklist

* [ ] Change aligns with `docs/` specs
* [ ] Deterministic output preserved
* [ ] Diagnostics codes unchanged (or new codes added)
* [ ] Fixture(s) added/updated
* [ ] Gold output(s) added/updated
* [ ] `cargo fmt` / `cargo clippy` / `cargo test` pass

## Reporting issues

For bugs, include:

* fixture or minimal repro model
* expected vs actual diagnostics/IR
* compiler version + pack version (if used)
