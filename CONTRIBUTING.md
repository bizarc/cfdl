# Contributing

CFDL is maintained by a small internal team at EVS.

> **External pull requests are not accepted at this time.** We welcome **bug reports**
> via GitHub issues (see "Reporting issues" below). Feature requests may be filed as
> issues; we read them, but the roadmap is set internally (`LAUNCH_PLAN.md`).

Everything below documents the working conventions for the maintaining team.

## Ground rules

* **Determinism is required**: same inputs + same pack + same compiler version must produce identical IR output.
* **Diagnostics codes are stable**: never reuse or rename existing diagnostic codes.
* **Golden-first**: any feature or bug fix must add/update a fixture and a gold output.
* **No correlation**: do not add correlation fields/slots in the language or IR.
* The authoritative specifications live under `docs/`. Implementation changes must conform to those documents.

## Repo conventions

* Rust workspace crates live under `crates/`.
* Fixtures live under `fixtures/`:

  * `fixtures/valid/<fixture_name>/...`
  * `fixtures/invalid/<fixture_name>/...`
* Expected outputs live under `gold/`:

  * `gold/ir/<fixture_name>.json`
  * `gold/diag/<fixture_name>.diag.json`
  * `gold/results/<fixture_name>.results.json`
* Specs live under `docs/` and are treated as the source of truth.

## Setup

### Prereqs

* Rust (pinned by `rust-toolchain.toml`)
* `make`

### Common commands

```bash
make ci      # fmt + clippy (-D warnings) + tests + golden suite
make gold    # golden suite only
```

## Development workflow

1. Pick the next task from your workstream in `LAUNCH_PLAN.md`.
2. Branch from `main` (`ws/<letter>-<slug>` for workstream branches).
3. Implement the minimal change required.
4. Add/update fixtures in `fixtures/` and expected outputs in `gold/`
   (`CFDL_GOLD_UPDATE=1 ./tools/golden-runner run` — intentional changes only,
   explained in the commit message).
5. Run `make ci`; merge only when green.
6. PRs/commits state: what changed, which fixtures/gold files were added or updated,
   and any spec references (e.g., `docs/04_compiler_spec.md` section).

## Change checklist

* [ ] Change aligns with `docs/` specs
* [ ] Deterministic output preserved
* [ ] Diagnostics codes unchanged (or new codes added)
* [ ] Fixture(s) added/updated
* [ ] Gold output(s) added/updated
* [ ] `make ci` passes

## Reporting issues

For bugs, include:

* fixture or minimal repro model (`.cfdl` source)
* expected vs actual diagnostics/IR/results
* compiler version (`cfdl --version`) + pack version (if used)
* OS / platform

Security issues: see `SECURITY.md` — do not open a public issue.
