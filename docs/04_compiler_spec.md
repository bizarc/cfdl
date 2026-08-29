# compiler_spec_v0_1.md

**CFDL Compiler Specification v0.1**

**Status:** Draft

This document specifies the implementation behavior for the **CFDL v0.1 compiler toolchain**, independent of any specific engine. It is written to enable agentic development of:
- Rust parser + validator + compiler to canonical IR
- CLI tooling
- Bindings for TypeScript and Python

**Scope:**
- `*.cfdl` source → AST → validated model → canonical IR JSON (conforms to `CFDL_v0_1_IR.schema.json`)

**Out of scope:**
- Deterministic/MC engine computations

## Normative keywords

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in BCP 14 ([RFC 2119](https://www.rfc-editor.org/rfc/rfc2119), [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174)) when, and only when, they appear in all capitals.

This specification exists so that a second implementation can be written from it. That is the reason the distinction is stated rather than assumed: a reader has to be able to tell a requirement from advice without inferring it from the surrounding sentence.
- Domain pack content (CRE/Operating), except pack interface touchpoints
- Correlation (explicitly excluded)

---

## 1) Toolchain responsibilities

### 1.1 Compiler stages (normative)
The compiler MUST implement the following stages in order:

1) **Load & Normalize**
- Discover `model.cfdl` in the model root.
- Read source files as UTF-8.
- Normalize line endings to `\n`.

2) **Lex**
- Tokenize with support for comments and string escapes.

3) **Parse**
- Parse module-level statements into an AST.
- Record **source spans** for all AST nodes.

4) **Import Resolution**
- Resolve `import` statements to a deterministic, acyclic module graph.
- Merge ASTs into a single **CompilationUnit** with deterministic ordering.

5) **Pack Resolution**
- Resolve optional `use pack` in `model.cfdl`.
- Load pack registry metadata and type registry.

6) **Name Resolution (Symbol Table)**
- Build symbol tables for entities, contracts, streams, phases, assumptions, options, events.

7) **Validation**
- Perform structural validation (required fields, duplicates).
- Perform type checking (strong typing).
- Validate schedules against master timeline.

8) **Lowering**
- Normalize literals (`YYYY-MM` → `YYYY-MM-01`, money literals, percent literals).
- Default and expand missing fields.
- Derive canonical IDs.
- **Expand each `contract` into the streams its pack's rules emit.** Every
  other stage is TRANSCRIPTION — one statement in, one object out, nothing
  newly named. This one is GENERATIVE: it creates stream names that no
  statement wrote, and it runs after stage 6, so those names cannot be in the
  symbol table.
- Construct canonical IR objects.
- Preserve provenance.

A check whose subject is a lowered stream therefore MUST run after this stage,
not at stage 6. Event stream targets are resolved here for that reason
(`E1302`, `docs/08`): at stage 6 the check cannot tell a name a contract has
not yet produced from a misspelling, and reporting both alike removes a
capability §13.2 grants.

9) **Emit**
- Serialize canonical IR JSON.
- Ensure emitted JSON validates against `CFDL_v0_1_IR.schema.json`.

### 1.2 Determinism requirements
Given identical:
- source files
- pack version
- compiler version

…the compiler MUST produce byte-for-byte identical IR JSON (after canonical JSON serialization), including deterministic IDs.

---

## 2) Inputs and directory layout

### 2.1 Model root
A valid model root directory MUST contain `model.cfdl`.

### 2.2 Relative imports
- Import paths are resolved relative to the importing file.
- The compiler MUST forbid directory traversal outside the model root (`..` escape).

### 2.3 Deterministic import ordering
To ensure determinism, the compiler MUST compute a **topological order** of imported modules. When multiple valid topological orders exist, the compiler MUST choose the lexicographically smallest by resolved normalized path.

---

## 3) AST specification

### 3.1 Source span model
Every AST node MUST carry a `Span`:
- `source_file: String`
- `start_line: u32`, `start_col: u32`
- `end_line: u32`, `end_col: u32`

**Line/col are 1-based.**

### 3.2 Core AST node set (minimum)

#### 3.2.1 Module
- `Module { statements: Vec<Stmt>, span: Span }`

#### 3.2.2 Statements (`Stmt` enum)
- `Version { value: String }`
- `Model { name: String, currency: Option<Currency> }`
- `UsePack { name: String, version: String }`
- `Import { path: String, alias: Option<String> }`
- `Time { calendar: Frequency, start: DateLit, periods: u32 }`
- `Phase { name: Ident, start: DateLit, end: DateLit }`
- `Entity { ns: Ident, name: Ident, type_id: Qname, attrs: Vec<Kv> }`
- `AssumeConst { name: Ident, expr: Expr }`
- `AssumeDist { name: Ident, dist: DistExpr }`
- `Contract { type_id: Qname, name: Ident, subject: EntityRef, term: DateRangeLit, body: ContractBody }`
- `StreamStandalone { name: Ident, owner: EntityRef, direction: Direction, currency: Currency, body: StreamBody }`
- `Event { name: Ident, when: Expr, actions: Vec<Action> }`
- `Option { name: Ident, type_id: Qname, exercisable_in_phase: Option<Ident>, body: OptionBody }`
- `RunDeterministic`
- `RunMonteCarlo { trials: u32, seed: u64 }`
- `Metric { name: Ident, expr: Expr }`

#### 3.2.3 Common value nodes
- `Ident(String)`
- `Qname(String)`
- `EntityRef { symbol: String }` // `ns.name`
- `DateLit { raw: String }` // `YYYY-MM` or `YYYY-MM-DD`
- `DateRangeLit { start: DateLit, end: DateLit }`

#### 3.2.4 Expressions
- `ExprSlot { lang: String, src: String, span: Span, expr_span: Span }` —
  `lang` is always `"cfdl"`; `src` is the raw expression source, and
  `expr_span` covers exactly that slice so expression-internal byte offsets
  map back to the file for diagnostics.

#### 3.2.5 Contract body
- `ContractBody { currency: Option<Currency>, parties: Option<Map>, tags: Option<Map>, terms: Option<Map>, effects: Option<EffectsBlock> }`

#### 3.2.6 Effects
- `EffectsBlock { streams: Vec<StreamEffect> }`
- `StreamEffect { name: Ident, owner: EntityRef, direction: Direction, currency: Currency, body: StreamBody }`

#### 3.2.7 Stream body
- `StreamBody { schedule: Option<Schedule>, amount: Option<Expr>, active_when: Option<Expr> }`

#### 3.2.8 Schedule
Schedules are normalized into a structured AST:
- `Schedule::OnDate { date: DateLit }`
- `Schedule::Every { freq: Frequency, on_rule: Option<OnRule>, from: DateLit, to: DateLit, opts: ScheduleOpts }`
- `Schedule::PhaseEnter { phase: String }`
- `Schedule::EveryPhase { freq: Frequency, phase: String, opts: ScheduleOpts }`

Where:
- `OnRule::DayOfMonth(u8)`
- `OnRule::EndOfMonth`
- `OnRule::Weekdays(Vec<Weekday>)`

Schedule options:
- `ScheduleOpts { convention: Option<Convention>, calendar: Option<String>, stub: Option<StubPolicy>, except: Vec<DateLit>, also: Vec<DateLit> }`

#### 3.2.9 Dist expressions
- `DistExpr { kind: DistKind, params: Vec<DistParam>, clip: Option<(f64,f64)> }`
- `DistKind ∈ { Normal, LogNormal, Uniform, Triangular }`

#### 3.2.10 Actions
- `Action::SetEntityField { entity: EntityRef, field: Ident, value: ValueOrExpr }`
- `Action::ActivateStream { stream: Ident }`
- `Action::DeactivateStream { stream: Ident }`
- `Action::ExerciseOption { option: Ident }`

#### 3.2.11 Values
`ValueOrExpr` allows:
- primitive literals
- money literals
- date literals
- inline lists/maps
- or an `ExprSlot` (a bare `cfdl` expression)

The compiler MUST preserve raw literal text and normalized typed representation in lowering.

---

## 4) Symbol tables and resolution

### 4.1 Namespaces and uniqueness
The compiler MUST enforce uniqueness constraints:
- Entities unique by full symbol `ns.name`
- Contracts unique by `name`
- Streams unique by `name` across the entire model (including streams inside contract effects)
- Phases unique by `name`
- Assumptions unique by `name`
- Options unique by `name`
- Events unique by `name`
- Metrics unique by `name`

### 4.2 Reference resolution
The compiler MUST resolve:
- `EntityRef` → existing entity symbol
- `activate/deactivate stream X` → existing stream name
- `exercise option X` → existing option name

Resolution occurs before lowering; unresolved refs are hard errors.

---

## 5) Validation rules

All validation emits **diagnostics** (see §10).

### 5.1 Required global statements
- `version` MUST exist exactly once.
- `model` MUST exist exactly once.
- `time` MUST exist exactly once.
- `use pack` MAY exist at most once (and only in `model.cfdl`).

### 5.2 Contract rules
- `term` is REQUIRED.
- `currency` is REQUIRED if contract emits any monetary stream.
- `effects` is REQUIRED unless a pack provides a lowering rule for this contract type.
- If a matching pack lowering rule exists, core validation MUST NOT emit
  `E2002_CONTRACT_MISSING_EFFECTS` for that contract.
- For contracts lowered by a pack, required term/field enforcement is the
  pack's responsibility (pack schema/rule validation), while core validation
  continues to enforce universal structural rules (for example, `term`).

### 5.3 Stream rules
- Streams MUST have: owner, direction, currency.
- Streams MUST include `schedule` and `amount`.
- If `active_when` omitted, it defaults to the expression `true`.

### 5.4 Schedule validation
- Schedule `from`/`to` MUST be within the master timeline.
- For `every` schedules, `from <= to`.
- `DayOfMonth` must be 1..31.
- `except` and `also` dates must be within the schedule bounds.
- `PhaseEnter` / `EveryPhase` phase MUST exist and be within timeline.

### 5.5 Type checking (v0.1 core)
The compiler MUST type-check:
- `Money` literals → Money
- Stream `amount` expression type: MUST be Money (or Decimal convertible to Money by stream currency)
- `active_when` and `event.when` expressions: MUST be Bool

Note: Expression type inference may be shallow in v0.1; if the compiler cannot prove type correctness statically, it MUST emit a warning and allow compile, unless the slot requires a hard type (Bool slots MUST be hard-checked).

### 5.6 Pack-based type validation
If a pack is active:
- Entity `type_id` must exist in pack type registry.
- Contract `type_id` must exist if the pack claims authority over it.
- Term schemas may be validated by the pack.

If no pack:
- The compiler still enforces core structural rules, but does not reject unknown type IDs.

---

## 6) Lowering rules (AST → IR)

### 6.1 Date normalization
- If `DateLit` is `YYYY-MM`, normalize to `YYYY-MM-01`.
- All IR dates MUST be `YYYY-MM-DD`.

### 6.2 Literal normalization
- `10%` MUST normalize to Rate `{ value: 0.10 }`.
- `42000 USD` MUST normalize to Money `{ amount: 42000, currency: "USD" }`.

### 6.3 Defaulting
- Stream `active_when`: default to `{lang:"cfdl", src:"true"}`.
- Optional schedule options default to:
  - convention: `none`
  - stub: `none`
  - except/also: empty

### 6.4 Deterministic ID generation
IDs are required for IR nodes. Use deterministic generation:

**Algorithm (normative):**
- `id = sha256( kind + ":" + stable_key + ":" + model_hash_seed )` then base32 or hex truncated.

Where:
- `kind` ∈ {Entity, Contract, Stream, Event, Option, Metric, Phase}
- `stable_key`:
  - Entity: `symbol`
  - Contract: `name`
  - Stream: `name`
  - Event: `name`
  - Option: `name`
  - Metric: `name`
  - Phase: `name`
- `model_hash_seed` is a stable string derived from compiler name+version and pack name+version (or empty).

**Truncation:** 16 bytes (32 hex chars) minimum.

### 6.5 Provenance
Each IR node MUST include `NodeProvenance`:
- `source_file`
- `source_span` from AST
- optional `notes`

The top-level `provenance` MUST include:
- `sources`: list of all included source files (deterministically sorted)
- `compiler`: name/version/hash

### 6.6 Required observables/refs inference
The compiler MUST populate:
- `required_observables`: all unique IDs referenced in `obs.rate(...)`, `obs.index(...)`, `obs.fx(...)` calls inside expressions
- `required_refs`: all unique IDs referenced in `ref.*` accessors inside expressions

**v0.1 requirement:**
- Implement a **lexical extractor** (regex/finite scan) that finds `obs.rate('X')` / `obs.index('X')` / `obs.fx('X','Y')` and `ref.<name>` patterns.
- Extraction MUST ignore escaped quotes inside strings.
- If extraction fails, emit a warning and continue.

### 6.7 Contract effects lowering
- Streams declared inside `contract.effects` are emitted both:
  - within the contract’s `effects.streams[]` in IR, and
  - also appended to the top-level IR `streams[]` array (so engines can treat all streams uniformly).

Each lowered stream MUST preserve provenance of the stream declaration span and include a reference note to its parent contract.

---

## 7) IR assembly rules

### 7.1 Required sections
The compiler MUST emit all required top-level sections per `CFDL_v0_1_IR.schema.json`.

### 7.2 Ordering rules (determinism)
Arrays MUST be emitted in deterministic order:
- phases: by phase name
- entities: by symbol
- assumptions.constants: by name
- assumptions.random: by name
- contracts: by name
- streams: by name
- events: by name
- options: by name
- runs: deterministic first, then monte_carlo
- required_observables/required_refs: lexical sort

### 7.3 Active pack recording
The IR does not embed pack schemas; however, the compiler MAY include pack identifiers in `provenance.compiler.hash` seed derivation (see §6.4) and in `provenance.compiler.notes`.

---

## 8) Rust crate decomposition (recommended)

A suggested split for agentic development:

- `cfdl-lexer`:
  - token types
  - comment stripping
  - string literal unescaping

- `cfdl-parser`:
  - recursive-descent or Pratt parser
  - AST + spans

- `cfdl-resolver`:
  - import graph
  - symbol tables
  - reference resolution

- `cfdl-validate`:
  - structural validation
  - schedule checks
  - minimal expression slot typing

- `cfdl-compile`:
  - lowering + normalization
  - deterministic IDs
  - required_observables/refs extraction
  - IR emission

- `cfdl-cli`:
  - `cfdl parse`, `cfdl validate`, `cfdl compile`

---

## 9) Acceptance tests (golden suite)

### 9.1 Fixture layout
- `fixtures/valid/*.cfdl` with expected IR in `gold/*.json`
- `fixtures/invalid/*.cfdl` with expected diagnostics in `gold/*.diag.json`

### 9.2 Minimal required fixtures
1. `minimal_model` (time + one entity + one stream)
2. `contract_with_effect_stream`
3. `event_sets_entity_state`
4. `phase_enter_schedule`
5. `bad_duplicate_stream`
6. `bad_missing_term`
7. `bad_schedule_out_of_bounds`
8. `obs_ref_extraction`

### 9.3 Diagnostics conformance
Invalid fixtures MUST match:
- diagnostic codes
- file and span presence

---

## 10) Diagnostics contract (compiler side)

The compiler MUST produce diagnostics with:
- `code` (stable string)
- `severity` ∈ {error, warning, info}
- `message` (human readable)
- `file` + `span` (required for parse/validation errors)
- optional `hint`

**Behavior:**
- Parsing errors MAY attempt recovery to continue producing more diagnostics.
- If any `error` diagnostics exist, compilation MUST fail and MUST NOT emit IR.

---

## 11) Error code guide (minimum set)

- `E1001_DUPLICATE_ENTITY`
- `E1002_DUPLICATE_CONTRACT`
- `E1003_DUPLICATE_STREAM`
- `E1004_DUPLICATE_PHASE`
- `E1101_MISSING_VERSION`
- `E1102_MISSING_MODEL`
- `E1103_MISSING_TIME`
- `E1201_IMPORT_CYCLE`
- `E1202_IMPORT_NOT_FOUND`
- `E1301_UNRESOLVED_ENTITY_REF`
- `E1302_UNRESOLVED_STREAM_REF`
- `E1303_UNRESOLVED_CONTRACT_REF`
- `E1304_UNRESOLVED_OPTION_REF`
- `E2001_CONTRACT_MISSING_TERM`
- `E2002_CONTRACT_MISSING_EFFECTS`
- `E2101_STREAM_MISSING_SCHEDULE`
- `E2102_STREAM_MISSING_AMOUNT`
- `E2103_SCHEDULE_OUT_OF_BOUNDS`
- `E2104_SCHEDULE_INVALID_RANGE`
- `E2201_EVENT_WHEN_NOT_BOOL`
- `E2202_STREAM_ACTIVE_NOT_BOOL`
- `W3001_EXPR_TYPE_UNKNOWN`
- `W3002_OBS_REF_EXTRACTION_FAILED`

---

## 12) Canonical JSON emission

The compiler MUST emit canonical JSON:
- stable key order (recommended)
- stable float formatting (recommended)
- deterministic array ordering (required)

If canonical serialization is not available, the compiler MUST at least ensure semantic determinism (same parsed content → same data structures) and provide a `cfdl fmt-json` tool to canonicalize.

