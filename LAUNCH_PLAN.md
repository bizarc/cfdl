# CFDL.dev Launch Plan — Multi-Agent Coordination Guide

**Audience:** implementation agents (and humans) working in parallel on the CFDL.dev launch.
Read this whole file before touching anything. The authoritative product decisions live in
the "Locked decisions" section; do not re-litigate them.

---

## 1. What this project is

CFDL (Cash Flow Domain Language) is being launched at **CFDL.dev** as a standalone,
source-available cash-flow modeling language for **all classes of cash-flowing asset**,
usable via Notebook (Python/Jupyter), API, files/CLI, and VS Code. Target quality bar:
world-class underwriting practice (Argus-grade CRE DCF, project-finance energy models,
Kahr Real Estate / Daedalus Services-style standards).

- **This repo** (`~/Documents/cfdl`, remote `github.com/bizarc/cfdl`, currently **private**)
  is the product repo. It was synced from `evs-platform/cfdl-core` on 2026-07-21 and is
  now the single source of truth for CFDL. **Do not develop CFDL inside evs-platform.**
- `evs-platform` consumes CFDL only via the compiled `cfdl` binary (`CFDL_BIN` env var in
  `evs-web/src/lib/cli.ts`). It will be re-pointed to released binaries later (Workstream G).

## 2. Locked decisions (do not reopen)

| Topic | Decision |
|---|---|
| License | **BSL 1.1** (SPDX `BUSL-1.1`), Change License Apache-2.0, Change Date 2030-07-21. Says "source available", never "open source". Legal review pending before repo flips public. |
| Governance | Small internal team only. No external PRs. Public issues for bug reports only. No DCO/CLA. |
| Repo | Reuse `bizarc/cfdl` (CFDL-only history). Private during build; public at launch. |
| Expressions | **Replace CEL** with a purpose-built engine (`cfdl-calc`): Excel-like syntax, `rust_decimal` money math, first-class dates. CEL (`cfdl-expr` wrapping cel-interpreter) stays functional until migration completes. |
| Launch scope | Full product at 1.0, no public beta. Four packs at launch: **energy/microgrids (flagship)**, CRE at Argus parity, credit/lending, opco. Fallback (decide only at Phase-3 midpoint): launch energy+CRE, fast-follow credit/opco. |
| Testing bar | Every pack gated by a `benchmarks/` parity suite vs Excel/Argus-grade reference models. "Match Excel to the penny" is the headline claim. |
| Surfaces | PyPI SDK (pandas accessors), WASM playground + Astro Starlight docs at cfdl.dev (Cloudflare Pages), CLI binaries (cargo-dist + Homebrew), VS Code Marketplace, self-hostable axum API server (`ghcr.io/bizarc/cfdl-server`). Hosted `api.cfdl.dev` post-launch. |

Full original plan: `~/.claude/plans/i-want-to-be-logical-teapot.md` (evs-platform session).

## 3. Current state (as of 2026-07-21)

**Workstreams A, B, and C are complete** (all merged to `main`, three-OS CI green;
golden suite at 49 fixtures). Highlights:

- **A (repo hygiene)**: BSL 1.1 relicense, standalone README/policy files,
  linux/mac/windows CI, release pipeline builds cfdl CLI + LSP binaries + VSIX.
- **B (cfdl-calc)**: CEL fully removed. Bare native expressions
  (`amount = base_rent * (1 + esc)^t`), decimal-first numerics with
  `excel_compat` mode, snake_case builtins (pmt/pv/fv/rate/year_frac/...),
  expression-level diagnostics, LSP hover/completion.
- **C (engine completeness)**: day-count/business-day calendar engine
  (US/TARGET/UK, ISDA rolls) wired into schedules; in-language `assume`
  distributions with per-assumption seeded Monte Carlo; event/option execution
  (latch semantics, entity state, option payoffs); parameterized pack lowering
  (`{{contract.*}}` templates + defaults, E5006); declarative pack metrics
  (metrics.toml) + engine universals (MOIC/payback/WAL); embedded pack registry
  (`embedded-packs` feature) for WASM/server hosts; `run` statements; `cfdl
  parse` AST dump; parser fuzz tests; cfdl-validate unit tests.

Remaining grammar gaps are tracked in **docs/10_implementation_status.md** —
the implement-or-remove worklist for the 1.0 gate. Legacy hardcoded
`apply_cre/opco_contract_terms` in cfdl-compile are removed when Workstream D
rebuilds those packs on lowering templates.

**Next up: Workstream D** (industry packs + Excel-parity benchmarks, energy
first) — its dependencies (C's pack templates, metrics, events) are all done.
E (Python SDK) and F (surfaces) can also start; F's playground depends only on
B (done) and C6 (done).

## 4. Build & test (all agents)

```bash
cd ~/Documents/cfdl
make ci            # fmt + clippy (-D warnings) + cargo test --workspace + golden suite
make gold          # golden fixtures only (42 must pass)
```
- Rust workspace: 12 crates under `crates/`. Pipeline: lexer → parser → resolver →
  validate → compile (→ IR JSON) → engine (→ Results JSON) → metrics.
- Specs: `docs/01..09_*.md`; machine contracts: `docs/schemas/ir.schema.json`,
  `results.schema.json`, `CFDL_v0_1_Grammar.ebnf`.
- **Never hand-edit `gold/`**; only re-bless via the runner, and only for intentional
  output changes, explained in the commit message.

## 5. Coordination rules

1. **One workstream = one agent = one branch.** Branch from `main`, name
  `ws/<letter>-<slug>` (e.g. `ws/b-cfdl-calc`). Merge to `main` only with `make ci` green.
2. **File ownership is the conflict-avoidance mechanism.** Each workstream lists the files
   it owns. Do not edit files owned by another active workstream; if you must, coordinate
   via the shared task list / PR review instead of editing directly.
3. Cross-cutting contracts (`docs/schemas/*`, the grammar EBNF, IR shape) are **owned by
   Workstream B** while it is active. Everyone else treats them as read-only inputs.
4. Golden re-blessing is serialized: only one workstream re-blesses per merge, and the
   merge that re-blesses must be rebased on latest `main` first.
5. Conventional Commits (`feat(engine): ...`, `chore(docs): ...`). Update `CHANGELOG.md`
   at milestones.
6. Anything user-facing says "source available", not "open source". Never publish
   (push public, crates.io, PyPI, Marketplace, Pages) without explicit human approval —
   the repo stays private until the human flips it at launch.

## 6. Workstreams

Dependency graph:

```
A (repo hygiene/CI)──────────────────────────┐
B (cfdl-calc expression engine)──► D (packs + benchmarks)──► launch gate
C (engine completeness)──────────► D                         │
E (Python SDK) ──────────────► needs B only for final polish │
F (docs site / playground / server / VSIX) ── needs B for playground; docs can start now
G (evs-platform re-point) ── needs first tagged release
```

### Workstream A — Repo hygiene, CI/CD, policy (S–M, no dependencies, start immediately)
**Owns:** `README.md`, `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `SECURITY.md`,
`.github/**`, `distribution/**`, `CHANGELOG.md`, name reservations.
- Rewrite `README.md` as the standalone CFDL product README (language pitch, quickstart,
  surfaces, source-available licensing section, "not accepting external PRs" note).
  Fix stale doc links (point at `docs/0X_*.md`).
- Refresh or replace stale `AGENTS.md`/`CLAUDE.md` (point them at this file).
- Rewrite `CONTRIBUTING.md` for closed-contribution model; add `SECURITY.md` and a
  bug-report-only issue template.
- Modernize `.github/workflows/ci.yml` (pre-existing from old repo — audit): fmt, clippy
  `-D warnings`, test, golden job; linux/mac/windows matrix. Set up **cargo-dist** for
  tag-triggered releases (mac arm/x86, linux x86/arm, windows) + Homebrew tap formula.
  Dry-run only; no public publishing.
- Reserve names (needs human approval per rule 5.6): `cfdl` + crate names on crates.io,
  `cfdl` on PyPI, `cfdl` publisher on VS Code Marketplace.
- Audit `docs/archive/`, `distribution/` for anything not fit for public eyes; propose
  deletions.

### Workstream B — `cfdl-calc` expression engine (L, the critical path)
**Owns:** new `crates/cfdl-calc`, `crates/cfdl-expr`, `crates/cfdl-lexer`,
`crates/cfdl-parser` (expression grammar), `docs/03_expression_environment.md`,
`docs/schemas/CFDL_v0_1_Grammar.ebnf`, `docs/02_grammar.md`.

**Locked design decisions (2026-07-21, user-approved):**
- **Bare native expressions** in `.cfdl`: `amount = base_rent * (1 + escalation)^years`
  — no keyword, no quoted string; expressions are first-class grammar with spans.
- **snake_case function vocabulary**: `pmt()`, `eomonth()`, `year_frac()` — no Excel
  uppercase aliases.
- **Dual-mode numerics ("best, plus match-or-explain Excel")**:
  - Default **decimal mode**: `rust_decimal` for money; ISDA/SIFMA day counts;
    `round()` = Excel half-away-from-zero; float64 escape ONLY for transcendental ops
    (fractional `^`, IRR solving) at documented conversion points.
  - **`excel_compat` mode**: identical expressions evaluated in IEEE-754 float64 to
    reproduce Excel artifacts. Benchmark harness runs BOTH modes per reference model:
    compat proves Excel parity; decimal-vs-compat delta explains any difference.
  - Rationale: day counts/accrual are standardized (ISDA/ICMA/SIFMA); decimal money is
    the accounting convention (ISO 4217 minor units); Excel float64 is a de-facto quirk,
    not a standard; IRR solvers have no standard (document solver + tolerance).
- Comparisons: `==` / `!=` canonical; `<>` accepted as alias. `and/or/not` keywords.
- New crate `cfdl-calc`: Excel-familiar syntax (`base_rent * (1 + escalation)^t`,
  `if(dscr < 1.20, a, b)`), `rust_decimal` arithmetic with documented rounding rules,
  first-class date/period types (`year_frac(d1,d2,"30/360")`, `eomonth`, `edate`),
  aggregations (`sum/min/max/avg/npv/irr`), financial builtins
  (`pmt/ipmt/ppmt/rate/nper/pv/fv`, `amortize()`, `cpr_to_smm()`, curve interpolation).
  No loops/recursion; wasm-clean (no fs, no threads, no tokio).
- Every builtin unit-tested against Excel-computed values (commit the expected values
  with a comment naming the Excel formula used).
- Swap behind `cfdl-expr`'s existing interface; grammar migrates `cel "..."` strings to
  native expressions with real spans (better diagnostics + LSP hover later). Keep CEL
  compiling until all fixtures are migrated, then remove cel-interpreter deps.
- Migrate `fixtures/**`, `packs/*/lowering/rules.toml`, re-bless goldens (rule 5.4).

### Workstream C — Engine completeness (M–L, parallel with B; coordinate IR changes via B)
**Owns:** `crates/cfdl-engine`, `crates/cfdl-compile`, `crates/cfdl-resolver`,
`crates/cfdl-validate`, `crates/cfdl-pack`, `crates/cfdl-metrics`,
`docs/04..08_*.md`.
- Day-count & business-day calendar engine (ACT/360, ACT/365F, 30/360 US, 30E/360;
  following/modified_following/EOM the grammar already declares; weekend + US/TARGET/UK
  holiday calendars as data; pack-extensible). Wire into schedule generation.
- In-language `assume`/distributions → IR → Monte Carlo (add LogNormal/Triangular;
  per-assumption deterministic seeding; run-config JSON becomes an override layer).
- Event/option execution: condition → effects (activate/terminate streams, set state);
  rule-based option exercise. Prerequisite for credit prepayment + energy contracts.
- Parameterized pack lowering (`crates/cfdl-pack`): rule templates reference contract
  terms (`{{contract.term_months}}`) with pack defaults + validations — replaces
  hardcoded rule amounts. **Must land before Workstream D writes new packs.**
- Metrics refactor: packs declare metric sets; engine-universal NPV/IRR/MOIC/payback/WAL.
- Embedded pack registry (`include_str!` standard packs, `embedded-packs` feature).
- Grammar audit: every declared construct implemented or removed — no silent no-ops.
  Implement `cfdl parse` (AST dump); add `cargo-fuzz` targets for lexer/parser.
- Add unit tests to `cfdl-validate` (currently zero — derive from invalid fixtures).

### Workstream D — Industry packs + benchmark suites (XL, after B & C's pack interface)
**Owns:** `packs/**`, new `benchmarks/**`, `fixtures/**` (new fixtures), pack cookbook docs.
- Benchmark methodology (applies to all packs): `benchmarks/<pack>/<case>/` =
  `.cfdl` model + reference CSV (period-level cash flows + summary metrics, built/verified
  in Excel by someone who knows the domain) + tolerance spec (decimal-exact for schedule
  math; bps tolerance for IRR-class iteratives). CI diffs `cfdl run` output against it.
- `cfdl/energy` (flagship): PPA revenue (fixed/escalating/indexed), merchant curves,
  degradation, availability/curtailment, storage arbitrage + capacity payments, O&M
  escalation, ITC/PTC, MACRS depreciation, DSCR-sculpted debt. Partnership flips: scope
  carefully, defer HLBV explicitly if needed.
- `cfdl/cre` at Argus parity: lease-by-lease, rollover/renewal, market leasing
  assumptions, recoveries (stops, base years, gross-ups), TI/LC, free rent, exit cap on
  forward NOI.
- `cfdl/credit`: level-pay/IO/bullet, CPR/SMM, CDR, severity/recovery lag, floaters off
  rate curves (needs `curve` input concept — coordinate with C).
- `cfdl/opco` deepened: working capital (DSO/DPO/DIO), capex/depreciation, debt schedules
  with sweeps, LBO returns.
- Flag any reference model that lacks expert verification — parity claims against wrong
  references are worse than none.

### Stochastic modeling — the differentiator (cuts across D/E/F)

Excel and Argus underwrite on point estimates and expected-value blends; CFDL
models are **natively stochastic**, deterministic-seeded, and reproducible.
This is a headline launch claim — the marketing story is "the same model
file gives you the Argus number AND the distribution around it."

**Shipped today** (do not re-plan; demonstrate):
- In-language `assume <x> ~ Normal|LogNormal|Uniform|Triangular(..., clip)`;
  per-assumption FNV-seeded draw streams (adding an assumption never
  reshuffles another's draws); byte-reproducible runs.
- Scenario-consistent branching via expressions: `if(inputs.draw < p, ...)`
  gives BINARY outcomes per trial (see fixture cre_stochastic_rollover:
  renewal vs re-lease with coherent rent/downtime/TI per trial — the
  bimodal distribution expected-value blends conceal).
- Run-config distributions as an override layer; MC aggregates for NPV
  (mean/median/stddev/p_negative).

**Pre-1.0 roadmap (in priority order):**
1. **Distribution outputs for every metric**, not just NPV: percentiles
   (P5/P25/P50/P75/P95) for IRR, DSCR, domain metrics; per-period cash
   flow fan charts in Results (feeds the SDK/playground visual story).
2. **Stochastic market-leasing pack rules**: first-class stochastic variants
   of cre.rollover (draw-based renewal, stochastic downtime months) so the
   pattern in the fixture is a one-line contract choice.
3. **Correlated assumptions**: rank correlation (Iman–Conover) or a simple
   factor model across assume draws — rent growth and exit caps do not move
   independently.
4. **Rate paths** for the credit pack's floaters (mean-reverting short-rate
   path per trial, seeded like assumptions).

Post-1.0: scenario trees / optimal exercise, full HLBV under uncertainty.

### Workstream E — Python/Jupyter SDK (M, start anytime; final polish after B)
**Owns:** `crates/cfdl-py`, `python/**`, `examples/notebooks/**` (new).
- Pandas accessors in the Python layer (`results.cashflows()/.metrics()/.scenarios()`
  parsing Results JSON — keep the native module thin); plotting behind `[viz]` extra.
- pytest suite against golden fixtures (currently zero tests).
- maturin CI wheel matrix (abi3 via `pyo3/abi3-py310`; manylinux x86_64/aarch64, macOS
  universal2, Windows). Build only — publishing to PyPI needs human approval.
- Example notebooks per industry (solar microgrid, CRE acquisition, loan pool, LBO);
  they double as docs content for F.

### Workstream F — Surfaces: docs site, playground, VSIX, API server (M–L)
**Owns:** `docs-site/**`, new `crates/cfdl-wasm`, new `crates/cfdl-server`,
`editors/vscode/**`, `crates/cfdl-lsp`.
- Docs site: this repo already has a **Docusaurus** `docs-site/` (predates the plan's
  Starlight recommendation). Evaluate: keep Docusaurus (content exists: getting-started,
  language-guide, reference) vs migrate to Starlight — decide once, document why, don't
  churn. Content refresh from `docs/0X_*.md`; add per-industry cookbook + benchmark
  methodology page; serve schemas at their `$id` URLs. Deploy target Cloudflare Pages
  (deploy needs human approval).
- Playground: `crates/cfdl-wasm` (wasm-bindgen) over source-string entry points
  (add to `cfdl-compile`; engine already has `run_from_json_str`) + embedded packs
  (from C). Monaco editor reusing the VS Code TextMate grammar. Requires B (cfdl-calc is
  wasm-clean; cel-interpreter's wasm support is unverified — do not build the playground
  on CEL).
- VS Code: platform-specific VSIX bundling LSP binaries from the release pipeline
  (`vsce --target`); OpenVSX too. Publishing needs human approval.
- API server: `crates/cfdl-server` (axum): `POST /v1/compile|validate|run`, OpenAPI via
  utoipa, limits (1 MB request, ~10 s timeout, MC trial cap, embedded packs only,
  no filesystem). Dockerfile → `ghcr.io/bizarc/cfdl-server` (publish needs approval).

### Workstream G — Re-point evs-platform (S, after first tagged release)
**Owns (in evs-platform repo):** `evs-web/src/lib/cli.ts`, `evs-web/src/app/api/**`,
`evs-core/crates/evs-cli/src/main.rs` integration, `cfdl-core/` removal.
- Pin a released `cfdl` binary version; verify evs-web build + S1–S8 flows; keep one
  release of overlap; then delete `cfdl-core/` from evs-platform and update its
  `CLAUDE.md`/docs.

## 7. Launch gate (1.0)

All must be true:
1. `benchmarks/` green within declared tolerances for energy, CRE, credit, opco.
2. Grammar audit clean: no declared-but-unimplemented constructs.
3. IR/Results schemas frozen as v1 with additive-only policy documented.
4. `make ci` green on linux/mac/windows; fuzzers run clean for CI budget.
5. Fresh-machine installs work: brew, cargo, pip, VSIX, docker (rehearsed privately).
6. BSL legal review done. Human flips repo public and approves all publishing.
