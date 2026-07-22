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

## 3. Current state (as of 2026-07-22)

**Workstreams A, B, C complete; Workstream D has ALL FOUR packs shipped**
(all merged to `main`, three-OS CI green; 59 golden fixtures; 8 benchmark
cases green). Practitioner review of benchmark references and the
stochastic roadmap remain open; Workstream H (waterfalls) is designed and
unstarted.

- **A (repo hygiene)**: BSL 1.1 relicense, standalone README/policy files,
  linux/mac/windows CI, release pipeline builds cfdl CLI + LSP binaries + VSIX.
- **B (cfdl-calc)**: CEL fully removed. Bare native expressions, decimal-first
  numerics with `excel_compat` mode, snake_case builtins
  (pmt/pv/fv/rate/ipmt/ppmt/macrs_rate/year_frac/cpr_to_smm/...),
  expression-level diagnostics, LSP hover/completion.
- **C (engine completeness)**: day-count/business-day calendars (US/TARGET/UK,
  ISDA rolls); in-language `assume` distributions with per-assumption seeded
  Monte Carlo; event/option execution; parameterized pack lowering
  (`{{contract.*}}` templates, prefix rule matching, per-instance stream
  names, E5006/E5007); declarative pack metrics + engine universals
  (MOIC/payback/WAL); embedded pack registry; `run` statements; `cfdl parse`;
  fuzz + validate tests.
- **D — energy (flagship, 3 increments)**: 10 template-driven contract types
  (PPA/merchant/storage/capacity/O&M/ITC/PTC/MACRS shield/capex/debt);
  contract-anchored vintages (degradation/escalation clocks start at each
  contract's own COD); availability factors; tax attributes metered
  separately from EBITDA. Benchmarks: solar+storage microgrid, wind
  PTC+MACRS.
- **D — CRE at Argus parity (4 increments)**: lease-by-lease
  (`cre.lease_unit.<id>`), anniversary-anchored escalations, free rent,
  recoveries with expense stops + base-year gross-up, percentage rent,
  probability-weighted rollover with Argus expected-value downtime AND
  Argus-timed turnover costs (renewal cost at expiry, re-lease cost after
  downtime), vacancy, property opex, `cre.exit_forward` valuing off
  ENGINE-DERIVED forward NOI via cross-stream `series_sum` + the
  `time ... project <n>` valuation tail. Legacy v1 hardcoded compiler path
  (apply_cre_contract_terms) DELETED — all v1 contracts are templates.
  Benchmarks: two-tenant office (full parity), retail strip (base-year
  gross-up + percentage rent). No documented deviations remain.
- **D — credit/lending (3 increments)**: `credit.pool_level_pay` and
  `credit.pool_io_bullet` — homogeneous fixed-rate pools with CPR
  prepayments, CDR defaults, severity, recovery lag, via the exact
  closed-form pool-factor decomposition (no loops needed); `credit.purchase`
  for acquisition pricing; collections/loss metrics incl. collections
  multiple. Benchmarks: level_pay_pool, io_bullet_loan — validated against
  independent month-by-month RECURSIVE references (the closed form must
  match the recursion to the penny, and does). Increment 9 added the
  `curve` model statement (step/linear interpolation, curve_value() in
  expressions) and `credit.pool_float_io_bullet` — floating-rate pools with
  margin/floor/cap off a named index curve (benchmark float_bridge_pool).
  Increment 10 added servicing/penalty streams, discount purchase, and a
  principal WAL via the new `wal_years` metrics op (per-stream weighted
  average life with `.*` wildcards). Still deferred: zero-rate level-pay
  pools; STOCHASTIC rate paths (curves are deterministic inputs today —
  stochastic roadmap item 4).
- **Language additions shipped for D**: `series_sum`/`series_avg`
  cross-stream references (two-phase, cycle-free), `time ... project <n>`
  projection tail, `parse_date`/`months_between` lease-anniversary
  anchoring, stochastic scenario branching proven
  (fixture cre_stochastic_rollover: binary renewal outcomes, 71.9% observed
  vs p=0.7, byte-reproducible).

- **D — opco at LBO grade (increment 11)**: legacy
  apply_opco_contract_terms DELETED (no hardcoded lowering paths remain
  anywhere); all v1 contracts templated with behavior-preserving defaults;
  new working_capital_policy (DSO/DPO/DIO), capex_line, term_debt
  (proceeds + ipmt/ppmt interest/principal split), cash_taxes,
  exit_ebitda (TRAILING-twelve-month EBITDA multiple via series_sum — the LBO convention, unlike CRE's forward NOI), acquisition.
  Benchmark: lbo_buyout vs independent recursive reference.

**Remaining for Workstream D:**
1. **Practitioner review** of all 8 benchmark references (risk register 3 —
   every case.toml carries a provenance note; references are independent
   implementations, not yet Excel/Argus-verified by a practitioner).
2. Stochastic roadmap items 1–4 (see the differentiator section below).
   Deterministic `curve` inputs shipped (increment 9); item 4 is now
   specifically per-trial stochastic rate paths on those curves.
3. Note: cash sweeps / revolver mechanics for opco debt are intentionally
   NOT in the pack — they are stateful/sequential and belong to
   Workstream H's allocation pass (see §6H), not to expression templates.

Grammar gaps remain tracked in **docs/10_implementation_status.md** (the
implement-or-remove worklist for the 1.0 gate).

**E (Python SDK) and F (surfaces) are COMPLETE.** E1–E4: object model + pandas
accessors, golden pytest parity, abi3 wheel matrix, industry notebooks. F1–F5:
docs refresh (kept Docusaurus), source-string compile API + `SourceProvider`
seam, WASM playground (`crates/cfdl-wasm` + Monaco page, verified in-browser),
axum API server (`crates/cfdl-server` + Dockerfile), and distribution polish
(Homebrew formula + generator, Open VSX prep; kept the hand-rolled pipeline).
All build-only — no publishing without human approval. A source-string SDK
`compile_source()` remains an available follow-up on the F2 foundation.

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
H (waterfalls / capital stack)── needs D pack conventions; blocks launch gate (see §6H)
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
  `-D warnings`, test, golden job; linux/mac/windows matrix. **F5 decision: kept
  the hand-rolled release pipeline instead of cargo-dist** (rationale in
  `distribution/README.md`); Homebrew formula template + generator shipped.
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
  rate curves — **shipped (increments 8–9)**. Remaining parity worklist
  (2026-07-22 assessment vs Intex/Bloomberg-class tools):
  - *Achievable now* — **shipped (increment 10)**: servicing-fee /
    prepayment-penalty streams, discount/premium purchase,
    principal-weighted WAL (`wal_years` metric op) + servicing/penalty
    metrics.
  - *Needs an engine primitive*: **vectored CPR/CDR** (PSA-style ramps, CDR
    curves) and **floating level-pay** — the balance path becomes a
    cumulative product with no closed form. Ship an `amortize()`-family
    builtin in cfdl-calc that runs the pool recursion natively (curves as
    the vector carrier); design it alongside H3's state machinery.
  - *Convention fidelity*: document our default-timing/prepay convention and
    match a named reference convention in benchmarks (practitioner review).
  - *With H (waterfalls)*: sequential note classes, OC/IC, reserve accounts.
  - *Post-1.0*: roll-rate/delinquency transition models, servicer advances,
    loan-level (vs rep-line) granularity, SOFR lookback/compounding
    mechanics.
- `cfdl/opco` deepened — **shipped (increment 11)**: DSO/DPO/DIO working
  capital, capex (fixed + % of revenue), scheduled term debt (IO/amort/
  balloon), cash taxes (no NOLs), trailing-EBITDA exit, acquisition;
  lbo_buyout benchmark; legacy apply_opco_contract_terms retired.
  Remaining: **cash sweeps / revolvers / PIK and NOL carryforwards need
  H3-style state**; depreciation is a declared schedule (no fixed-asset
  roll-forward yet).
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

### Workstream E — Python/Jupyter SDK (M) — **COMPLETE (increments E1–E4)**
**Owns:** `crates/cfdl-py`, `python/**`, `examples/notebooks/**`.
- ✅ Pandas accessors (`cashflows()/metrics()/metrics_frame()/scenarios()/`
  `monte_carlo()/annual()`) over a `Model`/`Results` object model; structured
  `CompileError.diagnostics`/`RunError`; plotting behind the `[viz]` extra.
  Native module kept thin — it gained `rate/as_of/pack` so the SDK applies the
  fallback rate and pack domain metrics (E1).
- ✅ pytest suite: byte-exact golden parity over all 37 valid fixtures, plus
  DataFrame/error/API tests; runs in CI on three OSes (E2).
- ✅ maturin abi3 wheel matrix (manylinux x86_64/aarch64, macOS universal2,
  Windows) + sdist + wheel-install smoke, build-only on tags/dispatch (E3).
- ✅ Four industry notebooks (solar microgrid, CRE office, loan pool, LBO) on
  the benchmark models, executed in CI (Linux); double as F docs content (E4).
- Deferred to human approval: publishing wheels to PyPI (rule 5.6).

### Workstream F — Surfaces: docs site, playground, VSIX, API server (M–L)
**Owns:** `docs-site/**`, new `crates/cfdl-wasm`, new `crates/cfdl-server`,
`editors/vscode/**`, `crates/cfdl-lsp`.
- Docs site — **F1 COMPLETE**: decision recorded to **keep Docusaurus** (not
  Starlight); rationale in `docs-site/README.md`. Content sync extended to the
  full `docs/01–10` set, per-pack **Cookbooks** generated from
  `packs/*/README.md`, a **Benchmark methodology** page, and JSON schemas
  staged at `static/schemas/` (serve at their `/schemas/...` `$id` paths;
  DNS/domain wiring deferred). Deploy stays GitHub Pages via `docs-pages.yml`
  (Cloudflare/domain move needs human approval).
- Playground — **F3 COMPLETE**: `crates/cfdl-wasm` (wasm-bindgen) over the F2
  source-string API + embedded packs; a Docusaurus `/playground` page (Monaco +
  a Monarch grammar port, diagnostics markers, live metrics). Engine is
  wasm-clean (FNV-seeded MC, no getrandom); `wasm-pack` bundle is CI-built
  during the site build (gitignored). Verified running in a browser.
- API server — **F4 COMPLETE**: `crates/cfdl-server` (axum): `POST
  /v1/compile|validate|run`, OpenAPI via utoipa, limits (1 MiB body, 10 s
  timeout, MC trial cap, embedded packs only, no filesystem). Dockerfile →
  `ghcr.io/bizarc/cfdl-server`; CI builds/saves the image (publish needs
  approval).
- VS Code: platform-specific VSIX bundling LSP binaries from the release pipeline
  (`vsce --target`); OpenVSX prep in F5. Publishing needs human approval.

### Workstream G — Re-point evs-platform (S, after first tagged release)
**Owns (in evs-platform repo):** `evs-web/src/lib/cli.ts`, `evs-web/src/app/api/**`,
`evs-core/crates/evs-cli/src/main.rs` integration, `cfdl-core/` removal.
- Pin a released `cfdl` binary version; verify evs-web build + S1–S8 flows; keep one
  release of overlap; then delete `cfdl-core/` from evs-platform and update its
  `CLAUDE.md`/docs.

### Workstream H — Waterfalls & capital stack distributions (XL, core feature)

**Added 2026-07-22 (user decision).** Distributing entity-level cash flow to
investors through a declared capital stack is a **core product feature for
every pack** — a cash-flow language that stops at the property/company line
is not adoptable by the funds, lenders, and sponsors we are targeting.
Asset-level modeling (Workstreams B–D) answers "what cash does the deal
produce"; H answers "who gets it" — and that second question is the one an
LP report, a credit committee memo, and a sponsor promote calc all hinge on.
**No shortcuts**: expected-value approximations of promote tiers, ignoring
period-by-period hurdle accrual, or netting waterfall effects into a single
blended stream are explicitly out of bounds — the industry tools we claim
parity with (Argus/Excel promote models, LBO debt schedules, Intex-style
note waterfalls) get this exactly right, and so must we.

**Owns:** new `crates/cfdl-waterfall` (or an engine module — decide at design
review), waterfall grammar surface in `crates/cfdl-parser`/`cfdl-lexer`,
engine allocation pass in `crates/cfdl-engine`, Results schema additions,
`docs/11_waterfalls.md` (new spec), waterfall sections of `packs/**`,
`benchmarks/**/waterfall_*` cases.

**Why this is engine work, not pack templating:** waterfalls are inherently
stateful and sequential — tranche balances decline with principal paid,
reserve accounts fill and release, hurdle IRRs accrue on outstanding
capital, triggers flip on tested ratios, sweeps depend on that period's
remaining cash. The expression dialect is deliberately pure/loop-free and
events are latch-once, so none of this is expressible today. H introduces a
dedicated per-period **allocation pass** that runs after stream evaluation:
ordered rules consume from a cash source and carry persistent state forward.

**Deliverables (each lands with docs + goldens; benchmarks at H5):**

1. **H1 — Design & spec (`docs/11_waterfalls.md`)**: capital stack and
   waterfall declaration model; evaluation semantics (ordering, state,
   period boundaries, day counts on accruals); Results schema for per-node
   flows. Reviewed against reference material for each domain BEFORE
   implementation: CRE/PE promote conventions (European whole-fund vs
   American deal-by-deal, preferred return compounding, catch-up, clawback
   noted for post-1.0), LBO debt schedules (revolver draws/paydowns,
   mandatory amort, cash sweep tiers, PIK toggles), securitization
   waterfalls (sequential/pro-rata principal, interest/principal separation,
   OC/IC coverage tests with cure logic, reserve accounts).
2. **H2 — Language surface**: `stack`/`tranche`/`account`/`waterfall`
   declarations (final grammar at H1); typed references to entities and
   streams; validation diagnostics (undefined tiers, circular references,
   unallocated residual). Grammar/EBNF/docs updated together; `cfdl parse`
   dumps the typed AST.
3. **H3 — Engine allocation pass**: post-stream, per-period sequential
   evaluation with persistent state (tranche balances, account balances,
   cumulative contributions/distributions, accrued-and-unpaid ledgers,
   trigger states with latch/cure); deterministic ordering; distributions
   integrate with Monte Carlo (per-trial waterfall execution, not
   expected-value blending). Results carry per-tranche/per-account series
   plus investor-level metrics (contributed, distributed, net IRR, MOIC,
   DPI/RVPI/TVPI) computed by the engine, not by packs.
4. **H4 — Pack integration** (the per-domain standard stacks, shipped as
   pack-level waterfall templates the way lowering rules are today):
   - **CRE**: senior + mezz debt service, refi/exit paydown, LP/GP equity
     with pref → return of capital → promote tiers over IRR hurdles.
   - **OpCo/LBO**: revolver + term loans with mandatory amort and excess-
     cash-flow sweep tiers, mezz/PIK, sponsor equity returns at exit.
   - **Credit**: sequential note classes with interest/principal waterfalls,
     OC/IC tests diverting to senior paydown, reserve account.
   - **Energy**: DSCR-sculpted senior debt, reserves (DSRA/MMRA), sponsor
     equity; partnership flip allocations (pre-flip/post-flip percentages)
     — full HLBV stays post-1.0 as already noted.
5. **H5 — Benchmarks**: one waterfall benchmark per pack against
   practitioner-grade Excel references (promote model, LBO debt schedule,
   sequential-pay note structure, flip allocation), same tolerance and
   provenance rules as §6D. These are launch-gate blockers.
6. **H6 — Surfaces**: Results schema additions flow through the Python SDK
   (`results.distributions()`, investor tables) and playground/docs
   examples (E and F own the rendering; H owns the data contract).

**Dependencies:** H1 can start immediately. H2–H3 follow H1 review. H4
needs each pack's asset-level conventions from D (CRE/credit done; opco in
progress; energy done). H interacts with the IR/Results freeze (launch gate
3) — schemas must not freeze before H3's additions land.

**Explicitly related backlog folded into H's design (not separate work):**
the same stateful sequential engine capability is what credit vectored
amortization (CPR/CDR ramps), floating level-pay, and opco cash sweeps
need. H3's design must not preclude reusing the state machinery for those,
but they remain scoped under D (credit/opco increments).

## 7. Launch gate (1.0)

All must be true:
1. `benchmarks/` green within declared tolerances for energy, CRE, credit, opco.
2. Grammar audit clean: no declared-but-unimplemented constructs.
3. IR/Results schemas frozen as v1 with additive-only policy documented —
   **after** Workstream H3's waterfall additions land (do not freeze early).
4. **Waterfall benchmarks green** (H5): CRE promote, LBO debt schedule +
   sweep, credit sequential-pay, energy flip allocation — each against a
   practitioner-verified reference.
5. `make ci` green on linux/mac/windows; fuzzers run clean for CI budget.
6. Fresh-machine installs work: brew, cargo, pip, VSIX, docker (rehearsed privately).
7. BSL legal review done. Human flips repo public and approves all publishing.
