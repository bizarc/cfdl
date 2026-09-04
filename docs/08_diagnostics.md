# diagnostics_spec.md

**CFDL Diagnostics Specification v0.1**

**Status:** Draft

This document defines the canonical diagnostics format, conventions, and error-code taxonomy for CFDL tooling:
- Rust compiler/CLI
- TypeScript editor integration
- Python notebook tooling

Diagnostics must be stable, machine-readable, and suitable for:
- inline editor annotations
- CI test assertions (golden diagnostics)
- user-friendly CLI output

---

## 1) Goals

1. **Actionable**: diagnostics should tell the user what happened, where, and what to do.
2. **Stable codes**: error codes must be stable across versions (with deprecation policy).
3. **Precise locations**: diagnostics should include file + span (line/col).
4. **Non-duplicative**: avoid flooding the user with cascading errors; prefer root-cause.
5. **Composable**: same schema for parser, validator, pack validation, and lowering.

---

## 2) Diagnostic object (canonical)

### 2.1 JSON schema (informative)
Tooling SHOULD represent diagnostics in this JSON shape:

```json
{
  "code": "E2103_SCHEDULE_OUT_OF_BOUNDS",
  "severity": "error",
  "message": "Schedule occurrence 2032-01-31 is outside the model timeline (ends 2031-12-31).",
  "file": "behavior.cfdl",
  "span": { "start_line": 18, "start_col": 7, "end_line": 18, "end_col": 64 },
  "path": "contracts[L1].effects.streams[rent].schedule",
  "hint": "Update the schedule 'to' date or extend the model time horizon.",
  "notes": ["Model timeline: monthly from 2026-01-01 for 72 periods."],
  "related": [
    {
      "message": "Timeline defined here.",
      "file": "time.cfdl",
      "span": { "start_line": 1, "start_col": 1, "end_line": 1, "end_col": 44 }
    }
  ]
}
```

### 2.2 Required fields
A diagnostic MUST include:
- `code` (string)
- `severity` (enum)
- `message` (string)

A diagnostic SHOULD include:
- `file` + `span` for any error tied to source location

### 2.3 Field definitions
- `code`: stable code (see §6)
- `severity`: one of `error`, `warning`, `info`
- `message`: concise, user-facing description
- `file`: relative path within model root when applicable
- `span`: source span (1-based line/col)
- `path`: optional machine path to IR/AST node for tooling
- `hint`: optional “how to fix it” guidance
- `notes`: optional list of additional context lines
- `related`: optional list of secondary locations

### 2.4 Span definition
`span` MUST be:
- `start_line`, `start_col`, `end_line`, `end_col` (all integers ≥ 1)
- inclusive start; inclusive end

---

## 3) Severity semantics

### 3.1 error
- Indicates the model cannot be compiled to IR.
- The compiler MUST fail compilation if any `error` diagnostics exist.

### 3.2 warning
- Indicates a potential issue, ambiguity, or best-practice violation.
- Compilation MAY proceed.

### 3.3 info
- Non-problem informational messages, e.g., pack hints.

---

## 4) Reporting conventions

### 4.1 Prefer root-cause errors
- When a failure would cascade, report the earliest/root issue and suppress downstream diagnostics.

Example: If `time` statement is missing, do not additionally report “phase out of bounds”.

### 4.2 Avoid duplicates
- Same logical issue should produce at most one diagnostic.

### 4.3 Provide hints for common fixes
- For errors in core DSL structure (missing `term`, missing `schedule`), a `hint` SHOULD be provided.

### 4.4 Provide related locations when helpful
- Import cycle errors SHOULD include related module locations.
- Out-of-bounds schedule errors SHOULD include timeline definition location.

---

## 5) Parser and recovery guidance

### 5.1 Parser behavior
- Parser SHOULD attempt recovery to continue parsing and emit multiple diagnostics.

### 5.2 Recovery strategies
Recommended recovery points:
- End of statement (newline/keyword boundary)
- Block boundary (`}`)

### 5.3 Parser diagnostic codes
Parser errors MUST use `E0xxx_...` codes.

---

## 6) Code taxonomy

### 6.1 Prefixes
- `E0xxx_*` Parse errors
- `E1xxx_*` Module/import/symbol errors
- `E2xxx_*` Validation errors (required fields, schedule bounds)
- `E3xxx_*` Type-check / expression contract errors
- `E4xxx_*` Pack-related validation errors
- `E5xxx_*` Lowering/IR emission errors
- `E6xxx_*` Pack lowering-time domain errors
- `W3xxx_*` Warnings (expression inference, extraction failures)
- `I6xxx_*` Informational

### 6.2 Naming conventions
`<Prefix><Number>_<CATEGORY>_<DETAIL>`
- All caps, underscores.
- Numbers are stable and monotonic within a category.

---

## 7) Canonical error codes (v0.1 minimum)

### 7.1 Parse errors (E0xxx)
- `E0001_UNEXPECTED_TOKEN` — the parser met a token it cannot use here.
- `E0002_UNTERMINATED_STRING` — a string literal opens and never closes.
- `E0003_UNTERMINATED_BLOCK_COMMENT` — a `/*` block comment opens and never closes.
- `E0004_EXPECTED_TOKEN` — something specific was required at this position and is missing. The message names what.
- `E0005_INVALID_DATE_LITERAL` — a date is not a real calendar date, or not in `YYYY-MM` or `YYYY-MM-DD` form.
### 7.2 Module/import (E12xx)
- `E1201_IMPORT_CYCLE` — two files import each other, directly or through a chain.
- `E1202_IMPORT_NOT_FOUND` — an imported file does not exist at that path.
- `E1203_IMPORT_OUTSIDE_MODEL_ROOT` — an import reaches outside the model's directory. A model is self-contained, so it can be moved or shared without carrying hidden dependencies.
### 7.3 Global structure (E11xx)
- `E1101_MISSING_VERSION` — no `version` declaration. It states which language version the model is written against.
- `E1102_MISSING_MODEL` — no `model` declaration, so the model has no name.
- `E1103_MISSING_TIME` — no `time` declaration. Without a timeline there is no grid to evaluate amounts on.
- `E1104_MULTIPLE_VERSION` — `version` is declared more than once.
- `E1105_MULTIPLE_MODEL` — `model` is declared more than once.
- `E1106_MULTIPLE_TIME` — `time` is declared more than once. A model has one timeline.
- `E1107_MULTIPLE_USE_PACK` — more than one `use pack`. A model draws contracts from a single pack.
- `E1108_USE_PACK_NOT_IN_MODEL_FILE` — `use pack` appears in an imported file rather than the model's own. The pack applies to the whole model, so it is declared where the model is.
- `E1109_MISSING_ENTITY` — no entity is declared. Every stream belongs to one.
Fields that move:

- `E1123_PREV_OUTSIDE_NEXT` — `prev` names a recurrence's own previous value and
  means nothing outside a `next`. A field's previous value is readable elsewhere
  as `prev.<entity>.<field>`.
- `E1125_NO_STATE_NAMESPACE` — an expression reads `state.<name>`. There is no
  such namespace: a value that changes over time is a field of the entity it
  describes, declared as `<name> init <expr> next <expr>` inside that entity's
  block and read as `<family>.<entity>.<name>`. Without this the reference
  reaches the engine, which warns and substitutes zero — an entire series
  evaluating to nothing while the run still reports `status: ok`.
- `E1127_FIELD_RULE_READS_FIELD` — a field's rule names another field by its family path. A field means this period's value at close, which does not exist yet inside a rule; `prev.<entity>.<field>` says the previous period. Unrejected it would resolve through the open-world entity root, return null and evaluate to zero.
- `E1128_FIELD_DECLARED_TWICE` — a field is declared both with `=` and with a rule. Both bind the same path, so one would silently win.
- `E1129_PREV_IN_FIRST_PERIOD` — a stream reads a field's previous period but runs from the model's first period, where there is none. Unrejected the read resolves to nothing and the stream evaluates to zero. Checked on hand-written and pack-lowered streams alike; the lowered form names the contract whose term set the schedule, since that is the term a model author can move.
- `E1131_UNKNOWN_FIELD_READ` — an expression reads a field the entity does not declare. Field paths resolve through the open-world `entity` root, so unrejected a misspelling reads as null and becomes zero in arithmetic. Lifecycle `status` keeps the open world; declared fields do not.
- `E1133_UNKNOWN_TIME_READ` — an expression reads a `time.` binding that does
  not exist. The vocabulary is closed — `t`, `date`, `days_in_period`, `phase`,
  `ppy` — so a miss is a typo, and unrejected it evaluates to zero every period
  with the run still reporting ok. There is deliberately no `E1132` for
  `inputs.`: an input may be supplied entirely by the run configuration, which
  the compiler never sees, so an unresolved input is the engine's to refuse.
- `E1134_SERIES_READ_IN_LOGIC` — an event's guard or action value, a field's
  rule, or an option's election or payoff calls a series reduction
  (`series_sum` and its five siblings). All
  of these are evaluated before any stream has a value, so the read binds
  nothing: the engine substitutes `false` in a guard and `0` in a rule, warns
  once per period, and publishes a full set of numbers under `status: ok` — an
  event that never fires, or a recurrence whose collapse `prev` carries for the
  rest of the run. A stream, a waterfall and the results layer do see stream
  values; drive logic from a field, a curve, `time.*` or `inputs.*` instead.
  `docs/28` §4 is where this becomes an ordering rule: under the period walk a
  guard may read a stream's settled history, at or before the previous period,
  and the same-period and forward forms stay refused.

### 7.4 Symbols and references (E13xx)
- `E1001_DUPLICATE_ENTITY` — two entities share a name.
- `E1002_DUPLICATE_CONTRACT` — two contracts share a name. Give one a suffix to keep them separable.
- `E1003_DUPLICATE_STREAM` — two streams share a name.
- `E1004_DUPLICATE_PHASE` — two phases share a name.
- `E1005_DUPLICATE_ASSUME` — two assumptions share a name.
- `E1006_DUPLICATE_OPTION` — two options share a name.
- `E1007_DUPLICATE_EVENT` — two events share a name.
- `E1008_DUPLICATE_METRIC` — two metrics share a name. Both would publish under `metric.<name>` and one would win silently.
- `E1301_UNRESOLVED_ENTITY_REF` — a stream, contract or event action names an entity that is not declared.
- `E1340_WATERFALL_NO_SOURCE` — a waterfall declares no `from`, so there is no
  pot to allocate.
- `E1341_WATERFALL_FORWARD_REF` — a step's `paid.<step>` names a step declared
  later in the same waterfall. Steps pay in declaration order, so a later step
  has not paid anything when an earlier one is evaluated.
- `E1342_WATERFALL_SERIES_NOT_VISIBLE` — a series reduction names a step
  of this waterfall or of a later one. Steps publish when their waterfall
  finishes, so the read would aggregate to zero and say nothing. An EARLIER
  waterfall is the documented composition and still compiles.
- `E1349_UNRESOLVED_LIFECYCLE_REF` — an entity binds `lifecycle <name>` and no
  lifecycle block declares it.
- `E1356_PARTICIPANT_RETURN_NOT_A_PARTY` — `irr`/`moic` names something that is not a party, or a party that owns no account, or is written as text rather than a reference. A participant's return is folded over the party's OWN ACCOUNT — contributions are negative inflows, receipts are allocations in — so a party without one has nothing to fold.
- `E1355_PARTICIPANT_RETURN_OUTSIDE_METRIC` — `irr` or `moic` appears outside a `metric` declaration. Both fold the finished projection, so reading one in a stream amount, an activation, an event guard, a waterfall step or an account inflow asks for a return on cash that expression has not produced yet. Left to run time it is a substituted zero and a warning nobody prints.
- `E1354_METRIC_FORWARD_REF` — a metric reads a metric declared below it, or reads itself. Metrics compose in DECLARATION ORDER, the same rule waterfalls follow, which makes the dependency an order rather than a graph. Reading itself is a different mistake: a metric is a fold over the finished projection, not a recurrence — carry a running quantity as a field the walk advances.
- `E1350_LIFECYCLE_CONFLICT` — an entity binds a model-declared lifecycle, but
  its ontology type already declares one. One machine per entity.
- `E1351_LIFECYCLE_NO_INITIAL` — a lifecycle block declares no `initial`.
  Every machine opens somewhere.
- `E1352_DUPLICATE_LIFECYCLE` — two lifecycle blocks share a name. One
  machine, one declaration.
- `E1353_UNREACHABLE_STATE_WRITE` — an event sets `status` to a state no
  declared edge enters. The write can never be legal, whatever state the
  entity is in at run; declare the edge or drop the write. An edge-less
  machine stays unconstrained.
- `E1347_UNRESOLVED_ACCOUNT_REF` — a step allocates `to account <name>` and no
  such account is declared. An account is not an entity and resolves in its own
  namespace, which is what the `account` keyword in the step says.
- `E1343_WATERFALL_DUPLICATE_STEP` — two steps in one waterfall share a name,
  which would make `paid.<step>` ambiguous.
- `E1344_WATERFALL_NO_REMAINDER` — a waterfall never says where the remainder
  goes, so cash could be left unallocated with nothing to say so.
- `E1345_WATERFALL_STEP_NO_AMOUNT` — a step says nothing about what it pays.
- `E1348_WATERFALL_NO_SCHEDULE` — a waterfall does not say when it distributes.
  The schedule is half of what a distribution says: between its scheduled
  periods the pot accumulates, so "every quarter" and "once at exit" are
  different deals rather than two spellings of one. The omission used to lower
  to a one-shot in the first period, distributing whatever that period happened
  to produce; there is no default right often enough to be silent.
- `E1346_STREAM_READS_WATERFALL_STEP` — a STREAM's series reduction
  names a waterfall step. Every waterfall runs after every stream and a step's
  series is visible to a later waterfall's `from` and to nothing else, so the
  read could only ever aggregate to zero. Model the quantity the step pays as a
  stream or a field if a stream must read it.
- `E1302_UNRESOLVED_STREAM_REF` — an event activates or deactivates a stream the model does not run. Event action targets were never resolved, so a misspelling matched nothing and the action was silently inert: the stream it was meant to stop kept paying, with no diagnostic and no warning. Checked after lowering rather than in the resolver, so a name a CONTRACT produced resolves as readily as one the model declared — the symbol table is built before the pack is chosen, and a check running there reported an unlowered name and a typo alike. The hint lists every stream in the model, both kinds.
- `E1357_LIFECYCLE_AUGMENT_TOPOLOGY` — a `lifecycle` block names a machine the PACK declared and also states `initial`, `state`, or an edge. A model may add arrival actions to a pack's machine and nothing else (`docs/34` D2a): the pack's machine is the checkable contract, and a model needing different topology declares a separate machine under its own name. The states and edges are refused rather than ignored — silently dropping them would leave the model saying one thing and the machine doing another.
- `E1358_ARRIVAL_ACTION_SETS_STATUS` — an `on enter` or edge action writes `status`. An arrival action sets FIELDS on the entity that transitioned; a status write would fire a second transition inside the same period, breaking one-transition-per-entity-per-period. A transition that should cause another transition is topology — an edge out of the target state, taken next period — and status writes remain the named event's privilege (`docs/34` D4).
- `E1359_ARRIVAL_ACTION_UNKNOWN_FIELD` — an `on enter` or edge action sets a field the entity bound to that machine does not have. The name is entity-relative, so it resolves against every entity bound to the machine and all of them need the field; the set is the union of what the model's entity block declares and what its ontology type contributes. Refused because a misspelled field is a write that lands nowhere — the silent-substitution shape `docs/13` §7.38 records for a misspelled series.
- `E1360_DUPLICATE_ENTITY_ID` — two entities declare the same literal field `id`. The id is a stable identity for the layer above the model — engine-opaque, published in the results graph (`docs/06`) — and a consumer joining on it would merge two things into one. Uniqueness within the model is the one thing the language can check about a value it must not interpret (`docs/13` §7.91).
- `E1361_DUPLICATE_SLICE` — two slices share a name. Same rule as a metric: one name, one selection.
- `E1362_SLICE_UNKNOWN_ENTITY` — a slice's `entity` (or `except entity`) names an entity the model does not declare. A slice selects by reference, and a reference is what the compiler can check — refused rather than silently matching nothing.
- `E1363_SLICE_UNKNOWN_TYPE` — a slice's or a statement row's `type` names an ontology type the active ontology does not define. The hint lists the known contract types; a master type (`Contract.Debt`) matches transitively through `refines`.
- `E1375_UNKNOWN_LINE_ROLE` — a slice's or a statement row's `line` names a line no contract type in the active ontology produces. A line is a role a master names (`docs/40` §6) — `interest`, `rent`, `proceeds` — and each pack rule names the one it emits, so the hint offers the near miss or lists the lines the vocabulary can produce.
- `E1364_SLICE_CATEGORY_ROOT` — a slice's category selector is not rooted in operating, investing or financing. A selector that could never match anything is a typo, not a choice.
- `E1371_UNKNOWN_CONTRACT_TERM` — a contract states a term its type does not declare. The roster is the pack type's own terms plus its masters' (`docs/40` §3); a term outside it is read by no rule, so before this check a misspelled `escalation` was a lease that never escalated. The hint names the near miss, or lists the type's terms.
- `E1372_MISSING_CONTRACT_TERM` — a contract omits a term its type requires, or states none of a group of alternatives (`one_of`: a lease's rent is `rent` or `rent_year`). Checked against the effective roster before any rule is expanded; `E5006` remains the rule-consumption backstop for a term a rule reads with no default.
- `E1373_UNKNOWN_CONTRACT_TYPE` — a type named on a declaration resolves to nothing the model may declare there: an `option ... type` the active ontology does not define, a two-token `contract <type> <instance>` whose type the pack does not declare, a fused contract name no rule lowers, an election written as a `contract`, or a lowered type written as an `option`. The hint names the near miss or lists what may be declared. Supersedes `E2002` for a contract under a pack that declares contract types.
- `E1374_ABSTRACT_TYPE_INSTANTIATED` — a declaration names a master (`Contract.Debt`, `Contract.Option`). A master is refined, never declared (`docs/40` §2); the hint lists its concrete refinements.
- `E1366_DUPLICATE_STATEMENT` — two statements share a name. Same rule as a metric and a slice: one name, one presentation.
- `E1367_STATEMENT_UNKNOWN_STRUCTURE` — a statement presents a hierarchy the engine does not build, or asks for a category hierarchy in a model whose streams declare no category. Either would render as one residual row and nothing else — technically complete and useless — so it is refused rather than shipped empty. Known structures: `entity` (the `part of` tree the results graph publishes) and `category` (the dotted path). `docs/13` §7.55.
- `E1369_STATEMENT_AUTHORED_AND_GENERATED` — a statement states both a `structure` and its own rows, or neither. A generated statement partitions the cash by construction, because a hierarchy covers its own tree; an authored one partitions it by the author's care. Mixed, neither guarantee holds — an authored row claims streams the generated rows already claimed, so the bottom line double-counts and the reconciliation that makes a statement trustworthy becomes noise. A statement stating neither would render nothing. `docs/13` §7.55.
- `E1368_STATEMENT_UNKNOWN_REFERENCE` — a statement filters by a slice, or shows a metric, that the model does not declare. A presentation that silently shows nothing is the failure §7.55 exists to end.
- `E1370_STATEMENT_SERIES_ROW_CLAIMS` — an authored row draws a published `series` beside a claim clause (`category`, `stream`, `slice`, `entity`, or a ratio's `of`/`to`). A series row presents a fold of the ledger, claims no streams and stays out of the bottom line; a claim clause beside it could only be resolved by a precedence the reader cannot see, which is a silently ignored clause. Refused instead. `docs/13` §7.55.
- `E1365_METRIC_UNKNOWN_SERIES` — a metric folds a series name this model does not publish. `series_sum`/`series_avg` (and each sibling reduction, to its own identity) return 0.0 for a selector that matches nothing, which is right for a `.*` selector and wrong for a name spelled out in full; in a metric it is worse than wrong, because a fold publishes ONE number under a name the author chose, with no series beside it to show the zero (`docs/13` §7.85). A metric may fold any series the valuation plane publishes: a stream by its own name or as `stream.<name>`, a waterfall step, `entity.<symbol>.net_cash_flow`, `account.<name>`, an entity field, a money subtotal, a declared slice's net as `slice.<name>`, or `model.net_cash_flow`. A RATIO subtotal is refused with its own hint — its undefined periods publish as null rather than zero, and what a fold should do with null has not been decided.
- `E1304_UNRESOLVED_OPTION_REF` — an event exercises an option that is not declared. Checked in the compiler rather than the resolver, because options are not in the symbol tables.
- `E1310_ENTITY_BLOCK_WITHOUT_TYPE` — an entity uses a block but declares no type, so there is nothing to check the block against.
- `E1311_UNKNOWN_ENTITY_TYPE` — an entity declares a type the active ontology does not define. The known types are listed.
- `E1312_MISSING_REQUIRED_FIELD` — an entity omits a field its type requires.
- `E1313_UNKNOWN_ENTITY_FIELD` — an entity sets a field its type does not declare. The declared fields are listed.
- `E1314_UNKNOWN_PARENT_ENTITY` — `part of` names an entity that is not declared. Hierarchy is optional; a declared parent is not.
- `E1315_ENTITY_PART_OF_ITSELF` — an entity is its own parent.
- `E1330_CONFLICTING_ACTIVE_CLAUSES` — a stream declares both `active when` and `active in state`. Use one: `active in state` for a lifecycle state, `active when` for anything else.
- `E1331_OWNER_HAS_NO_LIFECYCLE` — a stream is active in a lifecycle state but its owner's type declares no lifecycle.
- `E1332_UNKNOWN_ACTIVE_STATE` — a stream is active in a state its owner's lifecycle does not declare. A state name is checked against the lifecycle; a string comparison such as `entity.status == "leasd"` is not, and stays false for every period.
- `E1318_ENTITY_HIERARCHY_CYCLE` — `part of` forms a cycle. Reported once, from the cycle's lexicographically first entity, rather than once per member. An entity aggregates its children, so a cycle has no bottom to sum from.
- `E1316_UNKNOWN_LIFECYCLE_STATE` — an entity starts in a state its lifecycle does not declare. This is the misspelled status made impossible rather than merely unlikely.
- `E1317_TYPE_HAS_NO_LIFECYCLE` — an entity declares a starting state but its type has no lifecycle.
- `E1320_UNKNOWN_PARTY_ENTITY` — a contract or option binds a role to an entity that is not declared.
- `E1321_NOT_A_PARTY` — a role is bound to an asset. A contract is between parties.
- `E1322_UNKNOWN_PARTY_ROLE` — a role is bound that the contract type does not declare, or one the type leaves UNBOUND (a purchased pool's borrowers are many and unnamed). Roles are the type's effective roles, resolved through its master chain (`docs/40` §5): a CRE lease binds `landlord`, which is the master's `lessor`, and the hint lists each role a model may bind with the master's word beside it. A role belongs to the agreement, not to the entity.
- `E1302_UNRESOLVED_STREAM_REF` — something names a stream that is not declared — often an event deactivating one.
- `E1304_UNRESOLVED_OPTION_REF` — an event exercises an option that is not declared.
- `E1306_INVALID_ENTITY_REF_FORMAT` — entity ref, stream name, or contract name is not a qualified name with at least two segments (dotted hierarchy).

### 7.5 Contracts and streams (E20xx/E21xx)
- `E2001_CONTRACT_MISSING_TERM` — a contract omits a term its pack requires. The message names it; see the pack's contract table.
- `E2002_CONTRACT_MISSING_EFFECTS` — a contract produces no streams, so it has no effect on the model. Under a pack that declares contract types, a contract no rule lowers is a type the pack does not declare and is reported as `E1373` instead.
- `E2101_STREAM_MISSING_SCHEDULE` — a stream has no `schedule`, so there is no period for its cash to land in.
- `E2102_STREAM_MISSING_AMOUNT` — a stream has no `amount`.
- `E2103_SCHEDULE_OUT_OF_BOUNDS` — a schedule reaches outside the model
  timeline. The bound is the cash horizon **plus** any `project <n>` tail,
  since the engine evaluates streams over both; a schedule may reach into the
  tail deliberately to feed a `series_sum` valuation. Applied to hand-written
  streams during validation and mirrored onto pack-lowered ones during
  lowering, so a pack cannot express what a model may not.
- `E2104_SCHEDULE_INVALID_RANGE` — a schedule's `to` is before its `from`.
- `E2105_SCHEDULE_INVALID_DAY_OF_MONTH` — a day rule names a day outside 1–31.
- `E2106_SCHEDULE_PHASE_NOT_FOUND` — a schedule is anchored to a phase that is not declared.
- `E2107_STREAM_CURRENCY_MISMATCH` — a stream's currency differs from the
  model's reporting currency. Cash flows are summed period by period, so the
  two would be added as if they were the same unit. Convert explicitly in the
  amount expression, or declare the model in that currency.
- `E2108_SCHEDULE_FINER_THAN_CALENDAR` — the schedule's interval is finer than
  the model's calendar cadence. The occurrences are not lost: a period holds
  many accruals and their amounts **sum**, which is the same machinery a
  settlement lag uses. What cannot be done is telling them apart — an accrual is
  stored as a model period index, so occurrences inside one period share an
  environment, and an amount that varies over time is computed once and
  multiplied rather than summed across the occurrences. A constant amount would
  be exact; anything else is silently wrong, so both are rejected. Use a coarser
  interval, or declare a finer calendar.
- `E2109_SCHEDULE_CONFLICTING_PLACEMENT` — a schedule combines `mid` with
  a day rule or `net` payment terms. Each states where in its period
  the cash sits; two placements is a contradiction, not a refinement.

### 7.6 Events and actions (E22xx)

- `E2201_EVENT_WHEN_NOT_BOOL` — an event's `when` is not a true/false expression.
- `E2202_STREAM_ACTIVE_NOT_BOOL` — a stream's `active when` is not a true/false expression.
- An event may set a field the entity does not declare: the `entity` root is deliberately open-world, which is how lifecycle `status` works. A DECLARED field refuses a value it cannot hold — the engine warns `set … to non-numeric …; store unchanged` and the stored value is untouched, so nothing downstream reads the bad write.
### 7.7 Expressions / typing (E30xx/W30xx)
- `E3001_EXPR_PARSE_ERROR` — an expression is not valid CFDL.
- `E3003_EXPR_TYPE_ERROR` — an expression combines types that cannot combine, such as a date and a number.
- `E3004_EXPR_ILLEGAL_OP` — an operator is not defined for these operands.
Warnings:
- `W3001_EXPR_TYPE_UNKNOWN` — an expression's type could not be determined ahead of evaluation. It still runs; the warning notes the check was skipped.
- `W3002_OBS_REF_EXTRACTION_FAILED` — an observation reference could not be read out of an expression, so the run may not know it needs that input.
### 7.8 Pack errors (E4xxx)
- `E4004_MISSING_PACK` — the named pack could not be loaded — not found, or found and rejected.
### 7.9 Lowering/emission (E5xxx)
- `E5002_IR_SCHEMA_VALIDATION_FAILED` — the IR the compiler produced does not satisfy the published IR schema, or the IR being read does not.
- `E5003_IR_EMIT_FAILED` — the IR could not be written.
- `E5004_INVALID_LOWERING_RULE` — a pack's lowering rule is malformed.
- `E5005_PHASE_NOT_FOUND` — a lowering rule anchors to a phase the model does not declare.
- `E5006_MISSING_CONTRACT_TERM` — a lowering rule reads a contract term the contract does not supply.
- `E5007_DUPLICATE_LOWERED_STREAM` — two contracts lower to the same stream name. Give one a suffix.
- `E5008_INVALID_CURVE` — duplicate curve name, duplicate point date, or
  malformed point in a `curve` statement
- `E5028_INVALID_QUANTILE` — duplicate quantile name, a malformed point, a
  share outside `0..1`, shares out of order or repeated, or values that fall as
  share rises. The last is the one worth stating plainly: a quantile function
  that decreases leaves `quantile_of` with no single answer, so a threshold
  lookup would silently pick one of several. Rejecting it is what makes the
  inverse well-defined rather than merely usually right.
- `E5009_LOWERED_EXPR_INVALID` — a pack lowering rule expanded to an amount
  expression the parser rejects. Without this the engine evaluates the failed
  expression as zero and continues with only a warning.
- `E5020_LOWERED_FIELD_INVALID` — a pack lowering rule expanded to a field
  `init` or `next` the parser rejects. Same reasoning as `E5009`: the engine's
  fallback for a failed rule is zero, which would flatten every stream reading
  the field rather than fail loudly.
- `E5021_DUPLICATE_LOWERED_FIELD` — two contracts lower to one field name with
  *different* recurrences, so one would silently win. Give the rule's
  `field_name` a per-contract discriminator (`{{contract.suffix_ident}}`).
  Identical definitions collapse instead, which is what several contracts
  sharing one curve should do.
Statement completeness. These are warnings rather
than errors: the statement still renders, and the point is that the reader can
see what is wrong with it.
- `E5029_STREAM_MISSING_CATEGORY` — a stream declares no `category` while a pack
  is active. Its cash still reaches `model.total` and the entity roll-up and
  folds into no subtotal at all, so every domain metric is computed as though
  the stream were not there. An error rather than a warning because with a pack
  loaded there is always a right answer available — a flow that does not belong
  in net operating income takes a different root — and a coverage ratio that
  quietly excluded a stream is wrong and says so nowhere. Without a pack a
  category stays optional, because nothing folds.
- `E5030_AMBIGUOUS_CONTRACT_CATEGORY` — a contract states one `category` and
  lowers more than one stream. A contract lowers one or more streams and its
  pack states a category for each, so a single clause cannot say which it
  reclassifies: it would set all of them to the same value, and a coverage ratio
  computed off a principal repayment reclassified as interest is wrong with
  nothing to show for it. Name the stream — `category <stream> = <path>` — once
  per stream. The bare form stays legal where the contract lowers exactly one,
  because there is then nothing to disambiguate.

- `W5022_UNKNOWN_SERIES_REFERENCE` — a series reduction (`series_sum`,
  `series_avg`, `series_min`, `series_max`, `series_prod`, `series_count`)
  names a series no stream, contract or waterfall step produces, so it reduces
  over nothing and whatever reads it is reading nothing. A warning rather than an error because a
  literal name matching nothing is also a pack idiom: `cre.exit` sums NOI
  components by name whether or not the property declared each one. Selectors
  ending in `.*` are exempt, and are how a model states that matching nothing is
  intended.
- `W3500_STATEMENT_UNCLASSIFIED_STREAM` — cash that no row of the statement
  claims, usually a hand-written stream carrying no `category`. It is collected
  into a visible `residual` row rather than dropped, so the bottom line still
  reconciles and the omission is on the page instead of in the difference.
  The pack loader checks the same property for declared CATEGORIES statically;
  this is the half that needs a run, because a stream with no category at all
  is invisible until one happens.
- `W3501_STATEMENT_STREAM_DOUBLE_COUNTED` — a stream claimed by more than one
  row. Worse than an omission: the bottom line is then wrong in a direction
  that looks entirely plausible.
- `W3502_STATEMENT_BOTTOM_LINE_RESIDUAL` — the statement's rows do not sum to
  what the statement is accountable for, within half a cent. That is
  `model.total` for an unfiltered statement and the SLICE's total for one
  scoped to a slice: reconciling a filtered statement against the model would
  report the filter as a shortfall, and a warning that fires on a correct model
  is noise. Asserted, never corrected.

- `W3503_STATEMENT_UNKNOWN_STRUCTURE` — a model-declared statement asks for a
  hierarchy the evaluator does not build. A compiled model cannot reach this:
  `E1367` refuses the same condition earlier, with a span. It survives for
  hand-written IR, which the compiler never saw — the same reason the `ignored`
  journal outcome survives.

- `W5023_UNRECOGNISED_PACK_CATEGORY` — a stream's category is well-rooted and
  valid, and is not one the active pack recommends. The three roots are the only
  gate: a pack's `categories` list is the domain's conventional spelling, not
  permission, because a pack cannot enumerate every leaf a deal needs. Reported
  in the statement's diagnostics rather than in `results.warnings`, beside
  `W3500`, for two reasons: the consequence of an unrecommended category is a
  presentation one — no row of a pack statement claims it, so it lands in the
  residual — and `results.warnings` belongs to the engine, which has no pack.
  Names a near match when one is a single edit away, the bar the compiler
  already uses for a misspelled term. Reported once per distinct category:
  thirteen expense lines sharing one misspelling are one mistake.

- `E5022_UNKNOWN_STREAM_CATEGORY` — a stream declares `category <path>` that
  the active pack does not list in its manifest `categories`. A category is a
  dotted path into the cash flow statement (`operating.deduction.abatement`)
  and is what a fold aggregates on, so an unlisted one would leave the stream
  reported as a line and counted in no subtotal — visible and wrong, rather
  than absent and obvious. Use one the pack declares, or add it to the pack's
  vocabulary. With no pack in use there is no vocabulary, so any category is
  unknown. A pack whose own vocabulary is not rooted in `operating`,
  `investing` or `financing` fails to load rather than reaching this check.
- `E5010_TERM_UNKNOWN_INPUT` — a contract term references `inputs.<name>` for
  an input that is not declared. Declare it with `assume <name> = …` or
  `assume <name> ~ <Dist>(…)`.
- `E5012_RULE_INVALID_INTERVAL` — a lowering rule's `schedule_every` is not
  one of `day`, `week`, `month`, `quarter`, `year`.
- `E5011_TERM_CLIP_OUT_OF_BOUNDS` — a term defers to an input whose `clip`
  can produce values outside the range the pack allows for that term. The
  value itself cannot be checked until the run, but the clip states the range
  the driver can reach, so it can be.
- `E5013_PACK_CADENCE_UNSUPPORTED` — the model's calendar is not one the pack
  declares in `cadences`. A pack whose expressions divide annual figures by a
  literal 12 assumes one period is one month; on any other grid the *schedule*
  adapts correctly and only the *amount* does not, so the model produces
  plausible figures out by a factor of twelve. Refusing to lower is the only
  safe option. Use a calendar the pack supports, or a pack that supports the
  calendar.
- `E5014_RULE_CADENCE_UNSUPPORTED` — as above, but declared by one lowering
  rule rather than the whole pack. This exists so a pack can carry neutral and
  month-locked rules side by side while it is being migrated, instead of being
  gated wholesale.
- `E5018_TERM_START_OFF_GRID` — a pack contract's `term_start` does not fall on
  one of the model's period boundaries. Periods step from the model's start by
  whole calendar units, and elapsed-period counting measures whole steps from
  the term, so a term beginning mid-period counts short for the contract's
  whole life. Always satisfied on a monthly calendar, where every `YYYY-MM`
  term is a boundary.

- `E5015_TERM_MONTHS_NOT_DIVISIBLE` — a `_months` term used as a count of
  payment periods does not divide into whole periods on this grid. A 30-month
  loan is not two and a half annual payments, and no closed form can express
  one, so this is an error rather than a rounding. Thresholds such as
  `free_rent_months` pro-rate instead and never reach here.
- `E5016_RESERVED_TERM_PREFIX` — a contract term begins `model.`, `time.`,
  `periods.` or `whole_periods.`. Lowering rules resolve those prefixes before
  contract terms, so the term would be shadowed and never read. Term keys may
  legitimately be dotted, so this is reachable by accident.
- `E5017_PERIOD_TERM_NOT_LITERAL` — a `_months` term that a rule converts into
  periods is not a literal number: it defers to `inputs.<name>`, holds an
  expression, or does not parse as a number at all. The conversion happens at
  compile time and a non-literal is not known until the run.
- `E5019_UNKNOWN_DAY_COUNT` — a contract's `day_count` or
  `amortization_day_count` is not one of `30/360`, `30e/360`, `act/360`,
  `act/365`. Not defaulted silently: the gap between act/360 and act/365 is
  roughly 1.4% of interest.
- `E5027_ACTUAL_AMORTIZATION_BASIS` — a contract's `amortization_day_count`
  is `act/360` or `act/365`. That term chooses what the CONSTANT payment is
  struck on, and an Actual basis expands to a period-local divisor
  (`360 / time.days_in_period`) which the annuity then applies to every
  remaining period — so the payment moves with month length. Measured on a
  single 1.2m loan at 6%: a 460.68 swing over twelve months, with no pool, no
  prepayment and no defaults involved. Strike the payment on `30/360` and
  accrue interest on the Actual basis with `day_count`, which is what an
  Actual/360 loan document says; `day_count` itself is unaffected, because a
  per-period divisor is exactly right for a per-period accrual.
- `E2301_ASSUME_UNKNOWN_DIST` — a random assumption names a distribution that
  does not exist. The supported set is `Normal`, `LogNormal`, `Uniform`,
  `Triangular`.
- `E2302_ASSUME_INVALID_PARAM` — a distribution parameter is not a number, or
  is outside what the distribution admits.
- `E2303_ASSUME_MISSING_PARAM` — a distribution is missing a parameter it
  requires.
- `E2304_ASSUME_INVALID_CLIP` — a `clip=[lo, hi]` is malformed or inverted.
- `E2401_OPTION_MISSING_EXERCISE` — an option declares no `exercise when`, so
  nothing can ever trigger it.
- `E2402_OPTION_MISSING_PAYOFF` — an option declares no `payoff`, so exercising
  it would move no cash.
- `E5023_SUBTOTAL_UNKNOWN_CATEGORY` — a pack subtotal folds a category no rule
  emits, so the row would always be zero.
- `E5024_TERM_UNIT_MISMATCH` — a term is supplied in units the rule does not
  declare for it.
- `E5025_TERM_EXPR_INVALID` — a term holds an expression that does not
  compile. Checked at the term's own span, before substitution: after the
  splice the error would point at a rule the modeller did not write.
- `E5026_TERM_EXPR_IN_LITERAL_SLOT` — a term holding an expression is used by
  a rule where only a literal can go: a stream name, a schedule date, a
  frequency, or a net-days count. Those slots are never parsed as
  expressions, so an expression there is not evaluated late — it is wrong.
  Expression terms are valid where the rule uses the term in an expression,
  which is `amount_expr` and a field's `init`/`next`.

Both `cadences` gates are a migration scaffold rather than a permanent
statement about a pack: the entries are removed rule by rule as the
expressions become cadence-neutral.

### 7.10 Pack domain validations (E6xxx–E9xxx)

Two term spellings that mean the same figure in different units — a per-period
`amount` and an annual `amount_year` — are checked in both directions: at
least one must be given (`any_term_present`), and at most one may be
(`terms_mutually_exclusive`). The second matters because a lowering rule sums
the pair with zero defaults, templates having no conditional, so stating both
would silently add them. `E6030`, `E7010` and `E7011` are those checks.

These diagnostics come from a pack's own `validations.toml`, evaluated by the
compiler against each contract. They are pack-origin diagnostics and must
include file/span (contract span when a term-level span is unavailable).

Each first-party pack owns a reserved code range; the pack loader rejects a
validations file whose codes fall outside its declared `code_prefix`.

| Pack | Range | File |
|---|---|---|
| CRE | `E6xxx` | `packs/cre/validations.toml` |
| OpCo | `E7xxx` | `packs/opco/validations.toml` |
| Energy | `E8xxx` | `packs/energy/validations.toml` |
| Credit | `E9xxx` | `packs/credit/validations.toml` |

Presence of terms required by a lowering template is *not* listed here: that
is handled generically for every pack by `E5006_MISSING_CONTRACT_TERM`.

CRE pack codes:

- `E6001_CRE_LEASE_MISSING_BASE_RENT`
- `E6002_CRE_LEASE_INVALID_TERM_RANGE`
- `E6003_CRE_LEASE_UP_MISSING_MONTHS`
- `E6010_CRE_EXIT_MISSING_EXIT_CAP`
- `E6011_CRE_EXIT_INVALID_EXIT_CAP`
- `E6012_CRE_EXIT_MISSING_NOI_VALUE`
- `E6020_CRE_OPS_MISSING_AMOUNT`
- `E6030_CRE_LEASE_AMBIGUOUS_RENT` — a CRE lease states both `base_rent` (per period) and `base_rent_year` (annual). They would be summed; give one.
- `E6031_CRE_UNIT_INVALID_FREE_RENT` — `free_rent_months` is a whole number of
  months, 0 or more
- `E6033_CRE_UNIT_INVALID_ESCALATION` — a lease unit's `escalation` is below -1, which would make rent negative on the first step.
- `E6032_CRE_UNIT_INVALID_PRO_RATA` — `pro_rata_share` is a fraction between 0
  and 1
- `E6040_CRE_ROLLOVER_INVALID_PROBABILITY` — `renewal_probability` is a
  probability between 0 and 1
- `E6041_CRE_ROLLOVER_INVALID_DOWNTIME` — `downtime_months` is a whole number
  of months, 0 or more
- `E6050_CRE_DEBT_MISSING_PRINCIPAL` / `E6051_CRE_DEBT_INVALID_PRINCIPAL` — a
  pair: the first owns absent-or-unparseable, the second parsed-but-not-positive
- `E6052_CRE_DEBT_MISSING_RATE` / `E6053_CRE_DEBT_INVALID_RATE` — the same pair
  for the nominal annual rate
- `E6054_CRE_DEBT_INVALID_AMORT` — `amortization_months` strikes the payment and is
  normally longer than the loan's term
- `E6055_CRE_DEBT_INVALID_IO_MONTHS` — whole months, 0 or more
- `E6056_CRE_DEBT_INVALID_BALLOON_FLAG` — `balloon_at_maturity` is 0 or 1
- `E6057_CRE_CONSTRUCTION_INVALID_EQUITY_COMMITMENT` — zero or greater; zero is
  an all-debt build and legal, so the bound is not exclusive
- `E6058_CRE_CONSTRUCTION_INVALID_RATE` — a nominal annual rate in [0, 1], which
  catches 8 entered where 0.08 was meant
- `E6059_CRE_CONSTRUCTION_INVALID_DRAW_ACCRUAL_FRACTION` — where in the period a
  draw lands, in [0, 1]; 0.5 is funding drawn ratably through it
- `E6060_CRE_CONSTRUCTION_INVALID_TERM_RANGE` — the build must sit inside the
  model timeline, or the schedule silently loses draws
- `E6061_CRE_OPEX_LINE_MISSING_AMOUNT` — an operating expense line states
  `amount` or `amount_year`; both default to zero, so stating neither is a line
  that silently costs nothing
- `E6062_CRE_OPEX_LINE_PCT_FIXED_RANGE` — the fixed SHARE, in [0, 1]; catches 81
  entered where 0.81 was meant, which would otherwise report a wrong expense
  rather than fail
- `E6063_CRE_OPEX_LINE_OCCUPANCY_RANGE` — a ratio of occupied space, in [0, 1];
  zero is a fully dark building and is legitimate
- `E6065_CRE_CONSTRUCTION_INVALID_CAPITALIZE_INTEREST` — a construction loan's
  `capitalize_interest` is neither 0 nor 1. It is an election, not a rate: 1
  rolls each period's accrued interest into the balance, 0 pays it as it
  accrues. 0 is the default, so a model that says nothing is unaffected.

- `E6066_CRE_PCT_RENT_MISSING_SALES_QUANTILE` — `cre.percentage_rent_expected`
  states no `sales_quantile`. There is then no distribution to take an
  expectation over, and the natural fallback — treat the point estimate as
  certain — is not a smaller version of this contract, it IS
  `cre.percentage_rent`. The message names that contract rather than letting
  the two collapse silently.
- `E6067_CRE_PCT_RENT_INVALID_OVERAGE_PCT` — a fraction between 0 and 1.

- `E6064_CRE_REVENUE_LINE_MISSING_AMOUNT` — a revenue line states `amount` or
  `amount_year`; both default to zero, so stating neither is a line that
  silently earns nothing

OpCo pack codes:

- `E7001_OPCO_LINE_MISSING_AMOUNT`
- `E7002_OPCO_LINE_INVALID_SCHEDULE`
- `E7003_OPCO_LINE_INVALID_GROWTH`
- `E7010_OPCO_LINE_AMBIGUOUS_AMOUNT` — a line states both `amount` (per
  period) and `amount_year` (annual); they would be summed, so stating both is
  refused
- `E7025_OPCO_PERPETUITY_RATE_NOT_ABOVE_GROWTH` — a growing perpetuity needs
  `discount_rate` strictly above `growth_rate`. At or below it the denominator
  reaches zero and then goes negative, so the contract would return a huge
  value and then a negative one with nothing to say the model had stopped
  meaning anything.
- `E7026_OPCO_PERPETUITY_MISSING_BASE_VALUE` — the terminal-period flow the
  perpetuity is struck on.
- `E7027_OPCO_PERPETUITY_MISSING_DISCOUNT_RATE` — the terminal capitalization
  rate, stated on the contract rather than taken from the run's discount rate.
- `E7028_OPCO_PERPETUITY_MISSING_GROWTH` — state 0 for a flat perpetuity.
- `E7029_OPCO_PERPETUITY_INVALID_SELLING_COSTS` — a fraction between 0 and 1.
- `E7011_OPCO_TAXES_AMBIGUOUS_DA` — OpCo cash taxes state both `da_monthly` (per period) and `da_year` (annual). They would be summed; give one.
- `E7012_OPCO_TAXES_MISSING_RATE` — a cash-taxes contract states neither
  `tax_rate` nor `tax_rate_curve`. `tax_rate` carries a default of 0 so a curve
  may stand alone; without this check, stating neither would silently model a
  business that pays no tax.
- `E7013_OPCO_WC_MISSING_AMOUNT_OR_RULE`
- `E7014_OPCO_WC_INVALID_SCHEDULE`
- `E7020_OPCO_EXIT_MISSING_MULTIPLE`
- `E7021_OPCO_EXIT_INVALID_MULTIPLE`
- `E7022_OPCO_EXIT_MISSING_BASE_VALUE`
- `E7023_OPCO_EXIT_INVALID_SCHEDULE`
- `E7024_OPCO_EXIT_EBITDA_INVALID_MULTIPLE`
- `E7030_OPCO_DEBT_INVALID_AMORT`
- `E7031_OPCO_DEBT_INVALID_RATE`

Energy pack codes:

- `E8001_ENERGY_INVALID_DEGRADATION`
- `E8002_ENERGY_INVALID_AVAILABILITY`
- `E8003_ENERGY_INVALID_ESCALATION`
- `E8004_ENERGY_INVALID_PRICE_ESCALATION`
- `E8010_ENERGY_INVALID_MACRS_LIFE`
- `E8011_ENERGY_INVALID_TAX_RATE`
- `E8020_ENERGY_DEBT_INVALID_RATE`
- `E8021_ENERGY_DEBT_INVALID_TERM_MONTHS`
- `E8022_ENERGY_DEBT_INVALID_PRINCIPAL`

Credit pack codes:

- `E9001_CREDIT_INVALID_BALANCE`
- `E9002_CREDIT_INVALID_RATE`
- `E9003_CREDIT_INVALID_TERM_MONTHS`
- `E9010_CREDIT_INVALID_CPR`
- `E9011_CREDIT_INVALID_CDR`
- `E9012_CREDIT_INVALID_SEVERITY`
- `E9013_CREDIT_INVALID_RECOVERY_LAG`
- `E9014_CREDIT_INVALID_SERVICING_FEE`
- `E9015_CREDIT_INVALID_PREPAY_PENALTY`
- `E9016_CREDIT_INVALID_PSA_SPEED` — `psa_speed` is a MULTIPLE of the standard
  prepayment curve, so 1.5 means 150% PSA. Must be 0..10; 0 selects the flat
  `cpr` path.
- `E9017_CREDIT_INVALID_SDA_SPEED` — `sda_speed` is a multiple of the standard
  default assumption. Must be 0..10; 0 selects the flat `cdr` path.
- `E9018_CREDIT_INVALID_ABS_SPEED` — `abs_speed` is the Absolute Prepayment
  Model speed: the fraction of ORIGINAL balance prepaying each month. Already
  monthly, so unlike `cpr`/`cdr` it is not converted. Must be 0..1.
- `E9019_CREDIT_INVALID_AGE_MONTHS` — `age_months` is the pool's weighted
  average age at closing. PSA, SDA and the ABS model are all indexed from
  ORIGINATION, so a seasoned pool starts part-way up the ramp; leaving it at
  the default 0 on a seasoned pool understates prepayment. Non-negative
  integer.
- `E9020_CREDIT_RATE_FLOOR_ABOVE_CAP`

---

## 8) Deprecation and evolution policy

**Before 1.0, a retired code is DELETED.** There is no installed base, so
there is nobody holding a saved diagnostic whose meaning a reuse could
corrupt, and a register carrying entries for conditions that can no longer
arise costs every reader — human and machine — the work of telling live codes
from dead ones. Remove the entry, remove the check, and let the number return
to the pool. The rules below take effect at 1.0, when saved artifacts start to
outlive the release that produced them:

1. **Do not reuse codes**: once assigned, a code is never reused.
2. **Soft deprecation**: a deprecated code may remain emitted for one minor version with a note.
3. **Hard deprecation**: removal only in a major language version.

**A documented code must be an emitted code, both ways.** `make
check-diagnostic-parity` compares this page against every code the crates and
the pack validations emit, and against the numbers the pack READMEs cite. A
promised diagnostic that never fires is worse than an undocumented one: the
repair catalog teaches an agent to expect a code that will not come.

---

## 9) CLI rendering (informative)

CLI tools SHOULD render diagnostics as:

- Single-line summary:
  - `error[E2103_SCHEDULE_OUT_OF_BOUNDS] behavior.cfdl:18:7 Schedule occurrence ...`
- Then snippet with caret underline (optional)
- Then hint and notes

---

## 10) Golden diagnostics files

For invalid fixtures, store expected diagnostics as:
- `gold/diag/<fixture>.diag.json`

Rules:
- Assert `code`, `severity`, `file`, and `span`.
- Messages are asserted in FULL. The golden runner compares canonical JSON
  and diffs it, so rewording a message changes a golden and must be re-blessed
  with `CFDL_GOLD_UPDATE=1`.

  An earlier revision of this page said messages "may be asserted via substring
  match to allow minor wording changes". No runner has ever done that. The
  exact comparison is the better behavior and is kept deliberately: a
  diagnostic's wording is part of its contract with the reader, and a silent
  drift in what the compiler says is exactly as bad as a silent drift in what
  it computes. Making a reword show up in a diff is the point, not friction.

