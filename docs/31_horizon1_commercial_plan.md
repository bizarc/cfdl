# Horizon 1 implementation plan — services and the first product

**Status:** plan, 2026-08-27.
**Scope:** the pre-launch commercial horizon from the EVS strategy survey
(evs-platform `docs/15`, §4): Academy extensions (§2.9), Excel migration and
model audit as *services* (§2.5, §2.6), and the fund waterfall & carry
calculator as the first standalone product (§2.4).
**Boundary:** everything here consumes the language, the CLI, the server and
the benchmark discipline as they exist. Where a language feature is required
it is one already recorded in the backlog (§7.25, §7.72) — this plan sequences
that work; it does not invent new constructs.

---

## What Horizon 1 is for

Three things, in rising order of ambition:

1. **Revenue that needs no new code** — migration and audit engagements run
   on `excel_compat`, the benchmark runner, and the verification gates that
   already exist.
2. **A practitioner base** — the Academy already teaches; certification makes
   the teaching legible to employers.
3. **One product with a small surface** — the waterfall calculator, whose
   only missing ingredient is the participant-level return metric (§7.72),
   now unblocked because the account shipped (`docs/28` §5.1, walk phases
   3–4).

Each workstream below is independently shippable. None blocks the v1
milestones; the only language work (W4 phases 1–2) *is* roadmap work that M4
wants anyway (§7.25).

---

## W1 — Academy certification (§2.9)

**What exists:** learn.cfdl.dev is live — 24 chapters, 22 exercise sets
(`learn/content/exercises.json`), a course-kit builder
(`learn/scripts/build-course-kit.mjs`), and in-browser evaluation via the
wasm bundle.

**What to build:** assessment on top of the existing exercises, not new
content.

| phase | deliverable | notes |
|---|---|---|
| 1 | **Graded assessment set** — a held-out exercise pool per chapter band (foundations / streams & contracts / waterfalls & state / capstone), each graded by running the learner's model against fixed expected outputs | The grader is the same wasm evaluate-against-expected loop the exercises already use; the pool is held out of the public site. |
| 2 | **Certification exam flow** — timed session, N exercises drawn from the pool, pass threshold, retake policy | Session state and draw logic live in the learn app; no engine work. |
| 3 | **Certificate issuance** — a signed, verifiable certificate page (public URL with a verification hash), named tiers (e.g. *CFDL Practitioner*, *CFDL Modeler*) | Needs a small persistence layer for issued certificates — the first server-side state the learn app has; decide hosted store before building. |
| 4 | **Corporate cohort mode** — a cohort code that groups learners, an instructor view of completion | Only after 1–3 prove demand. |

**Gates:** the learn workflow's existing checks; new exercises pass
`check-training-examples.py`. **Explicitly not:** new chapters, video,
proctoring.

---

## W2 — Excel migration & reconciliation service (§2.5)

**What exists:** `excel_compat` deterministic arithmetic (run-config), the
benchmark case discipline (CASE.md, `expected.csv`, `expected_metrics.json`,
`benchmark-runner.py`), and eight CRE cases demonstrating the rebuild
pattern — `one_lincoln_street` native/contract twins are the worked example
of a migration done both ways.

**What to build:** the service kit — tooling that makes engagement N cheaper
than engagement N−1.

| phase | deliverable | notes |
|---|---|---|
| 1 | **Reconciliation harness** — `tools/excel-recon.py`: read a workbook's output range (openpyxl), align to a run's series by a declared mapping, emit a cell-level diff report (count, largest absolute and relative difference, first divergent period) | The benchmark runner already diffs against `expected.csv`; this adds the Excel-side extractor and the alignment map. Keep it a tool, not a crate. |
| 2 | **Parity report** — a rendered engagement artifact: model inventory, mapping table, diff summary under `excel_compat`, then the same diff under decimal arithmetic as the "what Excel got wrong" appendix | Render from the diff JSON; the `VALIDATION_REVIEW.html` deck is the visual precedent. |
| 3 | **Engagement template** — an intake checklist (workbook census, external links, circularity inventory, macro inventory), a scoping rubric (which sheets become streams, contracts, waterfalls), and a fixed engagement CASE.md outline mirroring the benchmark outline | Prose, in `docs/` or a private engagements repo — decide placement at phase 3; client work product must not land in the public repo. |
| 4 | **Assisted rebuild patterns** — a documented catalog: lease grid → `cre.lease_unit` instances, debt sculpt → `energy.debt_service`, circular interest → the walk's settled-cash reads | Grows one pattern per engagement; seeds W-agent (doc 32) authoring evals. |

**Gates:** the harness gets a self-test fixture (a small workbook checked in
under `fixtures/`); reports are reproducible from run artifacts alone.
**Explicitly not:** an automatic Excel→CFDL transpiler. The service is a
human rebuild with mechanical verification; the transpiler is a later bet.

---

## W3 — Model audit & validation service (§2.6)

**What exists:** the internal verification discipline — `make verify`, the
CI gates, `audit-measure.py`, analytic invariants, the journal as a causal
execution trace, and the re-measured coverage table (backlog §7.3).

**What to build:** turn the internal discipline into a client-facing
artifact.

| phase | deliverable | notes |
|---|---|---|
| 1 | **Validation package renderer** — from a model + snapshot + run: compile diagnostics, gate results, invariant checks, journal excerpts for the N largest flows, and the coverage statement (which constructs the model uses, which pack rules, each rule's §7.3 validation status) | Reads `run.json` and the journal; no engine changes. |
| 2 | **Audit engagement outline** — fixed review sequence: re-run and hash-compare (determinism attest), assumption census, sensitivity sweep, invariant screen, findings memo | Mirrors the benchmark CASE.md discipline; blanks = not asserted. |
| 3 | **Third-party model review mode** — the harder direction: the *client's Excel* is the subject, a CFDL rebuild is the audit instrument. W2 phases 1–2 are the dependency; the deliverable is the parity report plus a findings memo | This is W2 and W3 converging on the same tooling — build once. |

**Gates:** a validation package for one existing benchmark case is the
golden — regenerate and diff. **Explicitly not:** an opinion practice.
The package states what was checked and what diverged; sign-off language is
a business decision outside this plan.

---

## W4 — Fund waterfall & carry calculator (§2.4) — the first product

**What exists:** ordered waterfalls with schedule sovereignty over declared
pots (`docs/17`, `docs/25`), the account as a persistent, optionally
party-owned cash location (`docs/28` §5.1; engine phases 3–4, PRs #191–#193),
and the journal recording each step's action outcomes.

**What is missing:** exactly §7.72 — the return a party actually earned,
computed in the language. Its stated dependency (the account) has shipped;
its stated vehicle is the declared metric (§7.25).

| phase | deliverable | notes |
|---|---|---|
| 1 | **Declared metrics (§7.25)** — compile-time-resolved, named outputs; the construct §7.72 rides on | Language surface: spec, grammar, IR schema, resolver. Sequence with M4's output-resolution work — this phase *is* that roadmap item, pulled forward. |
| 2 | **Participant-level return (§7.72)** — `metric <name> = irr(party.<p>)` / `moic(...)` over a party-owned account's journaled contributions and receipts, computed in the valuation plane, published with series lineage | Per §7.72: over accounts, never payee streams (the §7.43 attribution trap). |
| 3 | **Benchmark case** — a published GP/LP waterfall worked example (European and American carry, preferred, catch-up, clawback test) reconciled tier-by-tier and party-by-party; register with the site | The pack question — whether tiers warrant contracts or stay core waterfall spellings — is answered by this case, not before it. `penzance_highlands` already hand-assembles Baupost's return; converting it is the second case and the §7.3 coverage proof. |
| 4 | **Calculator surface** — a focused page: waterfall definition in, per-party cash vectors and IRR/MoIC/carry out, every number traceable to a journal line. WASM for the free tier (the playground pattern), `cfdl-server` for saved runs | The product UI lives outside this repo; what this repo owes it is phases 1–3 plus a stable results contract (`docs/06`). |

**Gates:** phases 1–2 are language work — full cadence (`make ci` before
commit, `make verify` before push, goldens for the new constructs, schema
gates on the IR and results additions). Phase 3 follows `docs/20`'s case
authoring standard.

---

## Sequencing and effort shape

W1–W3 are independent and can start now; W4 phases 1–2 are the only
critical path (language work, sequenced with M4), and W4 phase 3 can be
drafted against the hand-assembled workaround §7.72 describes before the
metric lands, then converted.

Deliberately excluded from Horizon 1: hosted multi-tenant anything (the
server stays self-hostable single-tenant), the Excel transpiler, new packs,
and any EVS-side feature — per the non-preclusion rule, the seam stays
additive.
