# CFDL LSP + VSCode Authoring Plan

## Purpose
Deliver an end-to-end authoring experience for CFDL in VSCode:
- syntax highlighting
- diagnostics (as-you-type)
- go-to-definition / hover
- quick fixes (where feasible)
- pack-aware completions and snippets

This plan is scoped to the **CFDL repo** and complements the CLI + Python SDK.

---

## Goals
1) **Fast feedback:** show compiler diagnostics in-editor.
2) **Deterministic + offline:** no cloud dependency required for authoring.
3) **Pack-aware authoring:** show available aliases/templates and validate `use pack`.
4) **Minimal surface area:** reuse Rust crates already built (lexer/parser/resolver/validate/compile).
5) **Extensible to notebooks:** same backend can be used by notebook tooling.

---

## Architecture

### Components
1) **Language Server (Rust)**
   - implements LSP over stdio
   - uses CFDL crates directly
   - provides diagnostics, symbol index, completion items

2) **VSCode Extension (TypeScript)**
   - starts/stops the server
   - forwards file changes to the server
   - renders diagnostics, hovers, go-to-definition

### Recommended crate layout
- `crates/cfdl-lsp/` (new)
  - depends on: lexer, parser, resolver, validate, pack, compile
  - uses `tower-lsp` or similar LSP framework
- `editors/vscode/` (new)
  - extension source

### Current implementation approach (post-Milestone C hardening)
- The LSP maintains a **per-model-root analysis context** in memory, built from lexer/parser/resolver output.
- Context currently includes:
  - merged resolver output
  - symbol tables
  - per-file tokens
  - definition bindings (for go-to-definition)
- Refresh behavior is intentionally conservative and deterministic:
  - debounce edit-triggered refreshes (target: ~300ms)
  - skip expensive refresh paths when source is not parseable
  - clear stale diagnostics/index state when refresh cannot produce valid analysis
- This context is private to `cfdl-lsp` (no public API surface added to `cfdl-compile`).

---

## Capabilities (v0)

### 1) Syntax highlighting
- Implement a TextMate grammar first (fast path)
- Later: semantic tokens from LSP

Deliverables:
- `editors/vscode/syntaxes/cfdl.tmLanguage.json`
- `editors/vscode/package.json` contributions

### 2) Diagnostics
On file change:
- compile/validate the **active file’s model root**
- return diagnostics with file+span

Rules:
- debounce edits (e.g., 250–500ms)
- avoid compiling on every keystroke if no parseable state

### 3) Workspace model root detection
CFDL models are directory-based.
Server should detect model root by:
- nearest parent containing a `model.cfdl` (or configured entrypoint)
- include imported modules

### 4) Go-to-definition
- entity symbols
- stream symbols
- contract names
- `use pack` target to pack manifest

Current coverage:
- declaration identifiers for entity/stream/contract/phase
- `stream ... on entity <entity-ref>` to entity declaration
- schedule phase references (`phase_enter`, `phase_start`, `phase_end`) to phase declaration

Deferred to Milestone D+:
- `use pack` target to pack manifest
- pack alias/type-id navigation
- richer action/reference navigation as parser surface expands

### 5) Hover
- show symbol kind and provenance (file + location)
- for contract kinds: show pack alias resolution if pack is active

### 6) Completion
Provide completions for:
- keywords
- known entity names
- known contract kinds / aliases from active pack
- schedule keywords

### 7) Snippets
Extension provides snippets for common blocks:
- model/time skeleton
- entity/stream skeleton
- contract skeleton

Pack-provided snippets (later):
- load from `packs/<name>/templates.toml` once templates are formalized

---

## Pack Integration

### Pack discovery
Language server reads pack registry from:
1) `--packs` setting in VSCode configuration
2) default `packs/` under workspace

### Pack-aware validation
Reuse the existing pack host and lowering-time validations.

### Pack content surfaced to editor
- list available packs + versions
- show aliases and expected contract kinds
- expose templates as snippets (future)

---

## Settings (VSCode)

Suggested settings:
- `cfdl.packsPath` (string)
- `cfdl.entryFile` (default `model.cfdl`)
- `cfdl.enableLoweringValidation` (bool, default true)
- `cfdl.trace.server` (off|messages|verbose)

---

## Implementation Milestones

### Milestone A — Baseline VSCode extension + server boot (done)
- scaffold `crates/cfdl-lsp`
- implement initialize/shutdown
- file open/change notifications

### Milestone B — Diagnostics MVP (done)
- run compile/validate on model root
- publish diagnostics

### Milestone C — Symbol index + definition (done)
- build symbol tables from resolver output
- go-to-definition for entities/streams/contracts

### Milestone C.5 — Authoring hardening before D/E/F (done)
- extract reusable per-root analysis context in `cfdl-lsp`
- wire settings intake skeleton:
  - `cfdl.packsPath`
  - `cfdl.entryFile`
  - `cfdl.enableLoweringValidation`
  - `cfdl.trace.server`
- add debounce + parseable-state refresh guardrails
- extend definition coverage for phase declarations/references
- add lifecycle-focused tests for analysis rebuild and stable lookup behavior

### Milestone D readiness checklist
Use this checklist as a start gate before implementing Milestone D:

- [x] analysis context remains the single source for symbol + token + reference data (no duplicate ad hoc pipelines)
- [x] settings are consumed from LSP config with safe defaults:
  - `cfdl.packsPath`
  - `cfdl.entryFile`
  - `cfdl.enableLoweringValidation`
  - `cfdl.trace.server`
- [x] refresh lifecycle remains stable under edits:
  - debounce active
  - parseability guard active
  - stale diagnostics and stale analysis clear deterministically on failure
- [x] current definition behavior remains green:
  - declaration lookups (entity/stream/contract/phase)
  - stream entity references
  - schedule phase references
- [x] test baseline passes before adding pack behaviors:
  - `cargo test -p cfdl-lsp`
  - `make fmt && make lint && make test`

### Milestone D — Pack awareness
- read packs
- validate `use pack`
- show aliases in completion

### Milestone E readiness checklist
Use this checklist as a start gate before implementing Milestone E:

- [ ] Milestone D outputs are available and stable:
  - active pack detection is deterministic
  - pack aliases surfaced by LSP completion
  - `use pack` validation diagnostics include file/span
- [ ] snippet source boundaries are explicit:
  - generic snippets live in VSCode extension contributions
  - pack templates remain LSP-driven and opt-in until template contract is stable
- [ ] template/snippet expansion uses existing analysis context:
  - model root detection honors `cfdl.entryFile`
  - symbol context (entities/streams/contracts/phases) reused from analysis cache
  - no duplicate parse/resolve pipelines introduced for snippet generation
- [ ] completion/snippet UX guardrails are defined:
  - deterministic ordering
  - no blocking compile/validate in completion hot path
  - graceful fallback when pack/template metadata is unavailable
- [ ] regression baseline passes before template integration:
  - `cargo test -p cfdl-lsp`
  - `make fmt && make lint && make test`

### Milestone E — Snippets + templates
- ship generic snippets
- integrate pack templates once stable

### Milestone F readiness checklist
Use this checklist as a start gate before implementing Milestone F:

- [ ] token classification inputs are available from the shared analysis context:
  - per-file token maps
  - statement/symbol metadata sufficient for semantic categories
  - stable source ranges for declarations and supported references
- [ ] semantic token taxonomy is defined and versioned:
  - token types (keywords, types, entities, streams, contracts, phases, references)
  - token modifiers (declaration/reference, readonly, etc.) where applicable
  - explicit fallback behavior for unknown categories
- [ ] rendering strategy is deterministic and incremental-safe:
  - full-document token response works first
  - incremental/delta token support is deferred until correctness is proven
  - ordering and range encoding are stable across identical inputs
- [ ] interaction contract with existing TextMate grammar is documented:
  - semantic tokens augment rather than conflict with baseline highlighting
  - feature can be toggled safely during rollout
- [ ] performance guardrails are in place:
  - semantic token generation reuses cached analysis
  - no redundant compile/resolve path per token request
  - cancellation-safe behavior for rapid edits
- [ ] regression baseline passes before enabling by default:
  - `cargo test -p cfdl-lsp`
  - `make fmt && make lint && make test`

### Milestone F — Semantic tokens (optional)
- richer highlighting

---

## Testing

### Unit tests
- server utilities (root detection, path mapping)
- analysis context determinism + rebuild behavior
- definition mapping coverage (declaration + supported references)
- settings parsing/defaults

### Integration tests
- golden LSP session tests (optional, recommended before expanding D/E/F)
- VSCode manual test workspace using fixtures/examples

---

## Deliverables Checklist

In CFDL repo:
- `crates/cfdl-lsp/`
- `editors/vscode/`
- `docs/LSP_VSCODE_AUTHORING_PLAN.md` (this doc)

Initial acceptance:
- open `examples/cre_developer/model.cfdl`
- diagnostics render in VSCode
- go-to-definition works for `entity` and `stream` references
- completions show pack aliases when `use pack` is present

