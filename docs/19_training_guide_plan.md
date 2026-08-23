# CFDL Authoring Guide — Training Program Plan

## Context

The goal is a deep, course-grade authoring guide for CFDL — usable as the basis of an MBA class or professional training program. It is a **separate property from cfdl.dev** but shares the logo and design system. The existing site's "Learn the language" track is a *tour* (15–25 lines per construct); this guide is the *course*: modeling judgment, construct interaction, one deal carried end to end, finance semantics, and failure-mode pedagogy.

Locked decisions:
- **Audience:** dual-tracked — a core track for finance professionals new to code, plus optional deep-dive chapters for technical readers.
- **Scope:** general/packless language modeling in depth, plus a deep multi-chapter **CRE capstone**.

Key ground truths (verified against the current implementation):
- Language surface: 16 statement kinds — `version, model, use pack, import, time, phase, entity, assume, curve, state, event, option, waterfall, run, contract, stream`. Custom decimal-first expression engine (`crates/cfdl-calc`), 37 builtins + `if`/`series_sum`/`series_avg`/`curve_value` special forms; bare expressions (no `cel "…"`). MC distributions: Normal, LogNormal, Uniform, Triangular (language `assume ~` and run-config levels).
- CRE pack has 13 contract types incl. `percentage_rent`, `vacancy_loss`, `rollover`, `permanent_debt`, three exit forms.
- Site design system: `site/app/tokens.css` (two-layer `--p-*`/`--cfdl-*`), Inter + JetBrains Mono, `next-themes` dark/light, inline-SVG logo `site/components/Logo.tsx`, ds components `site/components/ds/`, shiki with the real TextMate grammar, WASM playground (in-browser engine).
- Publish gates that will govern the guide: `check-tokens.sh` (no raw hex), `check-site-voice.py` (no repo paths / GitHub links / design-history narration on published pages), `check-doc-examples.py` (code in docs must compile), link/sync checks.

## Recommended format: "CFDL Academy" — a course microsite + downloadable course kit

A book-style **training microsite** (working name: `learn.cfdl.dev`) inside the cfdl repo, sharing the design system at source level, plus a **course kit** (exercise/solution model files, instructor slides later) generated from the same source. Rationale:
- The site already embeds the real engine as WASM — in-browser, zero-install exercises are the single biggest pedagogical asset available; a PDF/document can't deliver that.
- Same-repo hosting keeps every code sample compile-gated against the engine that ships, and design tokens shared by import rather than by copy — this answers "quality + independent-but-aligned."
- Separate app/deploy keeps it a distinct property with its own nav, pacing, and release cadence.

## Curriculum (the content plan)

Structure: **5 parts, ~24 chapters + 2 appendices.** Each chapter = concept → worked example (runnable in embedded playground) → exercises (with hidden solutions) → "what can go wrong" (taught from real diagnostics). Deep-dive chapters marked ⚙ are the technical track; core track skips them cleanly.

### Part I — Thinking in cash flows (the mental model)
1. **Why a language?** Spreadsheets vs declarative models; determinism, reproducibility, review. The model/IR/results pipeline at reader level.
2. **The object model.** Time as the spine (calendar grain, phases, project tail); entities as the cast; streams as the atoms of cash. First runnable model.
3. **Reading results.** Periods, series, annual rollups, NPV/IRR, statements. How to check a model against a source's *method*, not just its answer (`round_to`, decimal vs excel-compat ⚙).

### Part II — The core language (packless modeling)
4. **Streams and schedules.** Direction/sign discipline, currency; the full schedule sub-language: cadences, `on day`/`eom`, `due` (annuity due vs ordinary — the finance meaning), phase-driven schedules, business-day conventions and calendars, `except`/`also`, `net N days` payment terms vs accrual.
5. **Expressions.** The environment (`model/time/entity/cfg/obs/inputs/state/prev`), decimal-first arithmetic, the TVM builtins (`pv/fv/pmt/ipmt/ppmt/nper/rate`) mapped to their Excel equivalents, date functions, `if`.
6. **Assumptions and inputs.** `assume x = …`, cfg/obs, what belongs in the model vs the run configuration; scenario thinking.
7. **Curves.** Rate curves, price decks, escalation paths; step vs linear interpolation; `curve_value`; clamping behavior; when a curve vs an assumption.
8. **State and recurrence.** `init`/`next`, `prev`, why same-period cycles are impossible by construction; balances, rounded-escalation, waterfall-adjacent accumulators. The canonical "match a published escalating figure" case.
9. **Growth, ramps, and guards.** `pow` compounding, `clamp` ramps, `active when`, phase gating — the idiom kit for real models.
10. **Events and options.** Latch semantics, `set`/`activate`/`deactivate`/`exercise`; rule-based exercise (and why optimal exercise is deliberately out); lifecycle state gating.
11. **Waterfalls.** `from pot pay step to payee`, `remaining`/`paid`/`owed`, chaining waterfalls; ordinary distribution structures expressed declaratively.
12. **Uncertainty and Monte Carlo.** `assume ~ Dist`, the four distributions and when each fits, `run monte_carlo`, per-assumption draw streams and reproducibility, reading p01–p99, per-trial branching idiom for bimodal outcomes; run-config override precedence.
13. **Multi-file models and style.** Import structure, naming conventions, a style guide for reviewable models.
14. ⚙ **Under the hood.** Lexer→parser→resolver→validate→lower→IR→engine; deterministic IDs, provenance, canonical hashing; dependency-ordered `series_sum` waves and the cycle refusal.
15. ⚙ **Diagnostics as a debugging discipline.** The code taxonomy, reading spans/hints, a tour of the invalid-fixture curriculum; "nothing is silently discarded."

### Part III — Modeling judgment (the MBA heart)
16. **Choosing grain.** Timeline grain (monthly vs quarterly vs annual and what each hides), entity grain (building vs unit, pool vs loan), and the cost of getting it wrong.
17. **Stream, contract, state, event, option, or waterfall?** A decision framework with paired examples — the chapter nothing on the site provides.
18. **Finance semantics the language encodes.** Day counts and `year_frac`, roll conventions, annuity due, CPR/SMM (`cpr_to_smm`), MACRS, cap-rate vs forward-NOI exits, why coverage ratios are recomputed not averaged.
19. **Packs: when and why.** What a pack adds (ontology, lowering, validations, metrics, statements); packless-first then migrate; survey of the four packs.

### Part IV — The CRE capstone (one deal, end to end)
A single development deal built across chapters, each ending with a working model:
20. **The deal and the skeleton.** An office/mixed-use acquisition+development case; phases (construction → lease-up → ops → exit), entities, timeline.
21. **Revenue.** `cre.lease`/`cre.lease_unit`, `percentage_rent`, `vacancy_loss`, `rollover`; lease-up ramps; when to drop to bespoke streams.
22. **Costs and financing.** `construction_stub` draws, `property_opex`/`ops_expense`, `permanent_debt`; construction-IO vs perm debt; DSCR.
23. **Exit and returns.** The three exit forms (`exit`, `exit_cap`, `exit_forward`); NOI reference semantics; levered/unlevered IRR, equity multiple.
24. **Risk.** Scenarios (base/downside), Monte Carlo on rents/exit cap/costs; an equity waterfall (pref → catch-up → promote) over the deal; presenting the risk story.

### Part V — Appendices
- A. **Language quick reference** (statement forms + schedule grammar on two pages).
- B. **CRE pack term tables** (all 13 contract types, required/defaulted terms).
- ⚙ C. **Instructor notes** (per-chapter teaching objectives, discussion prompts, problem-set variants) — later milestone.

Every chapter's models live as real `model.cfdl` + `run.json` + expected-results files, compile-and-run gated in CI.

## Delivery architecture

### Topology: sibling app `learn/` in the cfdl repo, shared design system via gated sync

- New standalone Next.js app at `learn/` (own `package.json`, lockfile, `tsconfig` with the same `@/*` alias) — a sibling of `site/`. No npm-workspace refactor: the repo root is a Cargo workspace, `site/` is self-contained, and the shareable surface is only ~15 files.
- **Sharing mechanism** (answers "independent but aligned"): `learn/scripts/sync-shared.mjs` copies a manifest of files from `site/` (`app/tokens.css`, `components/Logo.tsx`, `components/ds/*`, `lib/cn.ts`, `lib/shiki.ts`, playground libs) into `learn/`, with `--check` diff mode gated in **both** CI workflows. `site/` stays the single source of truth; drift fails CI in either direction. This mirrors the repo's existing sync-with-check idiom (`sync-content.mjs`, `sync-playground-examples.mjs`). If a third surface ever appears, the manifest becomes the file list of a `packages/ui`.
- Internal imports all use `@/*`, so mirrored files compile unmodified; `site/lib/shiki.ts` resolves the TextMate grammar via `cwd/../editors/vscode/...`, which a root-level sibling app satisfies with zero changes.
- Rejected: separate repo (breaks engine-gated verification of examples and guarantees token/wasm drift — exactly the failure mode to avoid); npm workspaces now (CI cache paths, makefile, and Vercel root-dir churn for a 15-file problem).

### Content pipeline

- **Chapters:** MDX authored directly in `learn/content/chapters/NN-slug.mdx`; rendered with the site's stack (`next-mdx-remote` + shiki + the real CFDL TextMate grammar). No repo-root sync hop — chapters have no second consumer.
- **Exercises:** real model dirs at `training/exercises/<chapter>/<exercise>/` — `model.cfdl` (starter), `solution.cfdl`, `run.json`, `expected.json` (asserted metrics), `README.md` (prompt). `learn/scripts/sync-exercises.mjs` (modeled on `sync-playground-examples.mjs`) bundles them into `learn/content/exercises.json`, `--check` gated.
- **Runnable in-browser exercises:** the site's `Playground.tsx` is not reusable as-is (hardcoded default example, share-hash/draft chrome), but its parts are: build `learn/components/exercise/ExerciseRunner.tsx` (~150 lines) composing `EditorPane` + `useEngine` + `ResultsPanel`, embedded from MDX as `<Exercise id="…"/>`, loaded `ssr:false`.
- **WASM:** the worker fetches same-origin `/wasm/…`, so `learn/public/wasm/` gets its own copy — either parameterize `OUT_DIR` in `site/scripts/build-wasm.sh` or have learn's prebuild copy `site/public/wasm/` + stamp; replicate `wasmBuildId()` cache-busting in `learn/next.config.ts`.
- **Course kit:** downloadable zips of exercise/solution files (and later instructor materials) generated at build from `training/`, freshness-gated like the notebooks.

### Quality gates

- New `tools/check-training-examples.py` (pattern: `tools/check-doc-examples.py`): every `model.cfdl` compiles; every `solution.cfdl` compiles, runs, and matches `expected.json` within tolerance; fenced `cfdl` blocks in chapters compile. Runs in the Rust `ci` chain via a makefile target.
- Reused learn-local gates: tokens check (no raw hex outside tokens.css), links, no-native-dialogs, wasm version; `sync-shared.mjs --check` + `sync-exercises.mjs --check`.
- Extend `tools/check-site-voice.py` scope to `learn/content/` + `training/` — chapter prose must not cite repo paths, GitHub links, or design history (write for the published page from day one).
- Makefile stays the single gate list: add `verify-learn-nofresh`.

### CI/deploy

- New Vercel project (`cfdl-learn` → `learn.cfdl.dev`), new `.github/workflows/learn.yml` cloned from `site.yml` (GH Actions builds because Vercel's image lacks Rust; team-scoped token already serves both projects), path-filtered on `learn/**`, `training/**`, shared-manifest files, `crates/**`, `examples/**`, grammar.

## Milestones (each shippable)

1. **Scaffold + design parity** — `learn/` app, sync-shared + gate in both workflows, header/footer/logo/theme toggle, landing page, deploy to learn.cfdl.dev. Proves topology before content.
2. **Chapter pipeline, prose-only** — MDX + CFDL highlighting, chapter nav/TOC, Part I written (ch. 1–3), fenced-code compile gate live. A publishable book with verified code.
3. **Runnable exercises** — `training/exercises/` layout, ExerciseRunner + `<Exercise/>` embed, wasm serving, solution/expected verification. The differentiating milestone.
4. **Core language + judgment parts** — Parts II–III (ch. 4–19) with exercises per chapter.
5. **CRE capstone** — Part IV (ch. 20–24), one deal carried end to end with its model files gate-verified at every chapter checkpoint.
6. **Course completeness** — appendices, solution-reveal UX, "open in full playground" deep links (lz-string hash into cfdl.dev/playground works cross-origin), downloadable course kit; instructor notes as a later add-on.

## Verification

- Per milestone: `make verify-learn-nofresh` locally, then the `learn.yml` build job green in Actions (check once per turn, never watch loops).
- `tools/check-training-examples.py` is the correctness backbone — every exercise solution must reproduce `expected.json` from the real engine.
- Visual parity: open learn.cfdl.dev (or `npm run dev` in `learn/`) side-by-side with cfdl.dev in both themes; tokens gate enforces the rest mechanically.
- Pedagogy check: each chapter's exercise must be solvable using only material introduced so far (review pass per part).
