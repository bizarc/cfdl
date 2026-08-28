# The agent substrate — implementation plan

**Status:** plan, 2026-08-27. Phase 1 implemented 2026-08-28: `crates/cfdl-mcp`
(six tools over MCP stdio), with post-run enrichment extracted into the shared
`cfdl-run` facade and the self-test gate at `crates/cfdl-mcp/tests/self_test.rs`.
Phase 2 implemented 2026-08-28: `tools/gen-machine-docs.py` generates
`docs/machine/` (llms.txt, the machine docs bundle, llms-full.txt with the
course chapters, the diagnostics → repair catalog, and the valid-examples corpus), staged to the site by
`sync-content.mjs` and gated by `make machine-docs-check` in `ci-gates`.
Phase 3 harness implemented 2026-08-28: `tools/agent-eval/runner.py` — three
tiers (repair from the 70 fixture/fix pairs, transcribe from the 42 cases,
extend by declared assertions), agents as `replay`/`cmd:`/HTTP, grading
imported from the benchmark runner, `make agent-eval-selftest` in `ci-gates`
and `make agent-eval-replay` as the full 100% gate. The private split and
real-agent runs remain open.
**Scope:** §3.4 of the EVS strategy survey (evs-platform `docs/15`): making
CFDL the modeling target an AI agent can write, verify, and explain —
"describe the deal, get a verified model." This plan covers the toolkit, the
documentation surface, and the evaluation harness. It adds **no language
features**.
**Thesis:** agents are unreliable in spreadsheets because a spreadsheet has
no verifier. CFDL already has the three properties an agent loop needs — a
compact declarative surface (`docs/22`), a deterministic
compile → run → diagnose cycle with structured errors (`docs/08`), and
ground truth to grade against (the benchmark suite, backlog §7.3). The work
is packaging those properties so a loop can consume them, then measuring how
well models actually get written.

---

## What already exists (verified against the repo)

- **Structured diagnostics** (`docs/08`, `cfdl-validate`): stable codes,
  spans, machine-readable output — the repair signal.
- **Deterministic results** (`docs/06`, `run.json`, `expected.csv` /
  `expected_metrics.json`): a graded run is a byte comparison.
- **The journal**: per-period execution trace with action outcomes — the
  explanation substrate.
- **42 registered benchmark cases** with fixed CASE.md outlines — each one
  is a latent eval: a prose specification with an asserted numeric answer.
- **Surfaces:** `cfdl-cli`, `cfdl-server` (axum, OpenAPI docs at `/docs`),
  `cfdl-wasm`, `cfdl-py`, `cfdl-lsp`, `terminology.toml`, the glossary
  generator.

Nothing below invents infrastructure; each phase wraps one of these for
machine consumption.

---

## Phase 1 — the agent toolkit (MCP server)

A thin MCP server over the existing compile/run pipeline, exposing the loop
an authoring agent needs. Implementation: a new `tools/`-adjacent binary or
a feature of `cfdl-server` — decide by whether the MCP transport wants to
share the axum process; default to a separate small crate (`cfdl-mcp`)
calling the same library entry points the CLI uses, so the CLI, server and
MCP cannot drift.

| tool | in → out | wraps |
|---|---|---|
| `compile` | source → IR or structured diagnostics | `cfdl-compile` + `cfdl-validate` |
| `run` | IR (or source) + run-config → results per `docs/06` | `cfdl-engine` |
| `diff` | results × expected (csv/metrics) → first divergence, per-period deltas | the benchmark runner's comparison |
| `explain` | a series + period → the journal slice that produced the number | journal reader |
| `lookup` | a term → glossary/terminology entry; a pack → its contract roster and each rule's §7.3 status | `terminology.toml`, pack ontology |
| `skeleton` | a domain + shape → a minimal valid model to grow (the CASE.md "what it exercises" grid in reverse) | templates from existing cases |

**Contract:** every tool's output is already a published schema or becomes a
fixture-tested one; no free-text tool results. **Gate:** an end-to-end
self-test — the MCP loop rebuilds one existing benchmark case from its
CASE.md prose and matches `expected.csv`.

## Phase 2 — the documentation surface for machines

Agents read docs differently: retrieval-sized, deduplicated, versioned.

- **`llms.txt` + a machine docs bundle** on the site: the language spec,
  grammar, expression environment, diagnostics catalog and pack rosters in
  one versioned, plain-text artifact — generated from the same sources the
  site renders, by a `tools/` script under CI so it cannot go stale.
- **A diagnostics → repair catalog**: for each diagnostic code, one minimal
  failing example and its minimal fix. The failing examples are
  `fixtures/invalid/` with their goldens in `gold/diag/` (the validate crate
  itself carries no fixture tests — the original premise here was wrong); the
  fixes did not exist anywhere and are authored into `fixtures/repairs/`,
  compile-verified by the generator.
- **The controlled-English register** (`docs/22`) promoted to an authoring
  contract: the subset an agent should emit. The mechanical subset is
  enforced by `tools/check-site-voice.py` (not the keyword-register tool,
  which checks the lexer's reserved words against spec §18 — this plan
  originally named the wrong gate).

**Gate:** bundle generation is a CI step; a checksum golden catches drift.

## Phase 3 — the eval harness

The benchmark suite becomes the grader. For each registered case, the eval
gives an agent the CASE.md specification (and the reference data the case is
allowed to see — never `expected.csv`), lets it drive the Phase-1 loop, and
scores the result.

- **Task tiers:** (a) *repair* — a seeded-defect model plus diagnostics or a
  wrong number, fix it; (b) *transcribe* — CASE.md → model, the full
  authoring task; (c) *extend* — an existing model plus a change request
  ("add a refinance at year 5"), graded by targeted assertions.
- **Scoring:** compiles / runs / matches — with *matches* the benchmark
  runner's own tolerance discipline; partial credit by asserted series, not
  by prose similarity. Determinism makes every score reproducible.
- **Held-out split:** tier (b) on public cases is contaminated for any model
  trained on this repo; keep a private case set (W2 engagements from
  `docs/31` feed it) for the honest headline number.
- **Harness mechanics:** a `tools/agent-eval` runner, provider-agnostic
  (the agent under test is an HTTP endpoint speaking the MCP loop), results
  as a scored JSON per case + a summary table. Publish the public-split
  results on the site once stable.

**Gate:** the harness runs a trivial scripted "agent" (replay of the known
model) at 100% — the self-test that separates harness bugs from model
failures.

## Phase 4 — the authoring assistant (productize the loop)

Only after Phases 1–3 measure well enough to demo honestly:

- **Playground copilot:** the wasm playground gains an assisted mode —
  prose in, model out, with the compile/diff/explain loop visible, every
  number linked to its journal line. The visible loop *is* the product
  argument: this is what "verified" looks like.
- **Review mode:** point the loop at an existing model — explain each
  output, flag unasserted values, propose the assertions a benchmark case
  would carry. This is `docs/31` W3's validation package, generated
  conversationally.
- **Prompt/pattern library:** the rebuild patterns catalog (`docs/31` W2
  phase 4) shipped as system-prompt material alongside the MCP server.

## The domain-agent layer (sketch)

An "Argus agent" in CRE, an "Intex agent" in structured credit, an "energy
agent" — none of these is a new toolkit. Each is the Phase-1 loop plus four
domain layers, and each layer already has a home in a phase:

| layer | what it is | where it lives |
|---|---|---|
| **vocabulary** | the pack: contract roster, templates, metrics, statements — `lookup` and `skeleton` are already pack-parameterized | shipped today; grows by backlog |
| **evaluation** | the domain's slice of the three task tiers, and its share of the private split (W2 engagements arrive one domain at a time) | Phase 3 |
| **conventions** | what the incumbent computes and how to reconcile against it: `excel_compat` arithmetic, forward-NOI exits, ordered-waterfall trigger semantics, curve and shape handling — the pattern library, as system-prompt material per domain | Phase 4 |
| **ingestion** | the domain's canonical artifact into model inputs: a rent roll, a collateral tape, a price/shape curve. The only genuinely new tool surface, and the last one to build — it is worthless until transcription scores well | Phase 4 |

**The domain agent is the pack's checkpoint.** Standing one up is the test
of whether the shipped pack covers the Pareto set of its domain's use cases
and modeling goals: the transcribe tier fails loudly and specifically where
vocabulary is missing, because a specification that names a structure the
pack cannot say produces a diagnostic, not a plausible workaround. That is
the same evidence discipline as everywhere else — an eval failure that
implicates the pack becomes a backlog entry with the failing case attached
— but run per domain and against real engagements, it becomes the
measurement `docs/13` §7.3 approximates by counting benchmark declarations
(`lookup` now derives that count mechanically). `docs/33` is the CRE
instance of this survey done by hand; the eval harness is how the other
domains get theirs without the hand.

## What this plan does not do

- **No language changes.** If an eval failure implicates the language, it
  becomes a backlog entry with the failing case as evidence — the same
  discipline as every benchmark-discovered gap.
- **No hosted inference.** The harness tests agents; it does not ship one.
  The Phase-4 copilot chooses a provider then, behind the user's key.
- **No EVS coupling.** The toolkit speaks source, IR, snapshots and results
  — the same seam as every other consumer.

## Sequencing

Phase 1 first and alone — it is the enabling artifact and is useful to
human tooling immediately. Phase 2 in parallel once Phase 1's shapes settle.
Phase 3 before any public claim about agent authoring; its numbers decide
whether Phase 4 is a launch feature or a quiet beta. Ordering with
`docs/31`: independent of W1–W3; W4's benchmark case (waterfall carry) joins
the eval set like every other case.
