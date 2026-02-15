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

### Milestone A — Baseline VSCode extension + server boot
- scaffold `crates/cfdl-lsp`
- implement initialize/shutdown
- file open/change notifications

### Milestone B — Diagnostics MVP
- run compile/validate on model root
- publish diagnostics

### Milestone C — Symbol index + definition
- build symbol tables from resolver output
- go-to-definition for entities/streams/contracts

### Milestone D — Pack awareness
- read packs
- validate `use pack`
- show aliases in completion

### Milestone E — Snippets + templates
- ship generic snippets
- integrate pack templates once stable

### Milestone F — Semantic tokens (optional)
- richer highlighting

---

## Testing

### Unit tests
- server utilities (root detection, path mapping)

### Integration tests
- golden LSP session tests (optional)
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

