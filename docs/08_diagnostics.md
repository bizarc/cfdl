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
- `E0001_UNEXPECTED_TOKEN`
- `E0002_UNTERMINATED_STRING`
- `E0003_UNTERMINATED_BLOCK_COMMENT`
- `E0004_EXPECTED_TOKEN`
- `E0005_INVALID_DATE_LITERAL`

### 7.2 Module/import (E12xx)
- `E1201_IMPORT_CYCLE`
- `E1202_IMPORT_NOT_FOUND`
- `E1203_IMPORT_OUTSIDE_MODEL_ROOT`

### 7.3 Global structure (E11xx)
- `E1101_MISSING_VERSION`
- `E1102_MISSING_MODEL`
- `E1103_MISSING_TIME`
- `E1104_MULTIPLE_VERSION`
- `E1105_MULTIPLE_MODEL`
- `E1106_MULTIPLE_TIME`
- `E1107_MULTIPLE_USE_PACK`
- `E1108_USE_PACK_NOT_IN_MODEL_FILE`
- `E1109_MISSING_ENTITY`


State declarations (`docs/14_state_and_recurrence.md`):

- `E1120_STATE_MISSING_INIT` — a `state` has no `init`. The value at period 0 is
  required, not defaulted: a recurrence with an unstated base case would
  evaluate to zero for every period, and an out-of-range read returns zero
  silently, so the error would never surface.
- `E1121_STATE_MISSING_NEXT` — a `state` has no `next`. It would hold its
  initial value forever, which an `assume` expresses more clearly.
- `E1122_STATE_DUPLICATE_NAME` — two states share a name.
- `E1123_STATE_PREV_OUTSIDE_NEXT` — `prev` appears in `init`. There is no period
  before the first, so the initial value cannot depend on a previous one.
- `E1124_STATE_SAME_PERIOD_READ` — `state.<name>` appears inside a `next`.
  That path reads the **current** period, which is the same-period edge the
  design exists to prevent; `prev.<name>` reads another state's previous value.
- `E1125_STATE_UNKNOWN_REFERENCE` — a `state.<name>` or `prev.<name>` names a
  state that is not declared. Without this the reference reaches the engine,
  which warns and substitutes zero — so an entire series evaluates to nothing
  while the run still reports `status: ok`.
### 7.4 Symbols and references (E13xx)
- `E1001_DUPLICATE_ENTITY`
- `E1002_DUPLICATE_CONTRACT`
- `E1003_DUPLICATE_STREAM`
- `E1004_DUPLICATE_PHASE`
- `E1005_DUPLICATE_ASSUME`
- `E1006_DUPLICATE_OPTION`
- `E1007_DUPLICATE_EVENT`

- `E1301_UNRESOLVED_ENTITY_REF`
- `E1302_UNRESOLVED_STREAM_REF`
- `E1303_UNRESOLVED_CONTRACT_REF`
- `E1304_UNRESOLVED_OPTION_REF`
- `E1305_UNRESOLVED_PHASE_REF`
- `E1306_INVALID_ENTITY_REF_FORMAT` — entity ref, stream name, or contract name is not a qualified name with at least two segments (dotted hierarchy).

### 7.5 Contracts and streams (E20xx/E21xx)
- `E2001_CONTRACT_MISSING_TERM`
- `E2002_CONTRACT_MISSING_EFFECTS`
- `E2003_CONTRACT_CURRENCY_REQUIRED`

- `E2101_STREAM_MISSING_SCHEDULE`
- `E2102_STREAM_MISSING_AMOUNT`
- `E2103_SCHEDULE_OUT_OF_BOUNDS` — a schedule reaches outside the model
  timeline. The bound is the cash horizon **plus** any `project <n>` tail,
  since the engine evaluates streams over both; a schedule may reach into the
  tail deliberately to feed a `series_sum` valuation. Applied to hand-written
  streams during validation and mirrored onto pack-lowered ones during
  lowering, so a pack cannot express what a model may not.
- `E2104_SCHEDULE_INVALID_RANGE`
- `E2105_SCHEDULE_INVALID_DAY_OF_MONTH`
- `E2106_SCHEDULE_PHASE_NOT_FOUND`
- `E2107_STREAM_CURRENCY_MISMATCH` — a stream's currency differs from the
  model's reporting currency. Cash flows are summed period by period, so the
  two would be added as if they were the same unit. Convert explicitly in the
  amount expression, or declare the model in that currency.
- `E2108_SCHEDULE_FINER_THAN_CALENDAR` — the schedule's interval is finer than
  the model's calendar cadence, so several occurrences would fall in one period
  and collapse into a single payment. A weekly schedule on a monthly grid paid
  twelve times a year rather than fifty-two. Use a coarser interval, or declare
  a finer calendar.

### 7.6 Events and actions (E22xx)
- `E2109_SCHEDULE_CONFLICTING_PLACEMENT` — a schedule combines `mid` with
  `due`, a day rule, or `net` payment terms. Each states where in its period
  the cash sits; two placements is a contradiction, not a refinement. See
  `docs/12_payment_timing.md`.

- `E2201_EVENT_WHEN_NOT_BOOL`
- `E2202_STREAM_ACTIVE_NOT_BOOL`
- `E2203_ACTION_SET_FIELD_INVALID`

### 7.7 Expressions / typing (E30xx/W30xx)
- `E3001_EXPR_PARSE_ERROR`
- `E3002_EXPR_UNKNOWN_IDENT`
- `E3003_EXPR_TYPE_ERROR`
- `E3004_EXPR_ILLEGAL_OP`

Warnings:
- `W3001_EXPR_TYPE_UNKNOWN`
- `W3002_OBS_REF_EXTRACTION_FAILED`

### 7.8 Pack errors (E4xxx)
- `E4001_UNKNOWN_TYPE_ID`
- `E4002_INVALID_ENTITY_ATTR`
- `E4003_INVALID_CONTRACT_TERMS`
- `E4004_MISSING_PACK`

### 7.9 Lowering/emission (E5xxx)
- `E5001_ID_GENERATION_FAILED`
- `E5002_IR_SCHEMA_VALIDATION_FAILED`
- `E5003_IR_EMIT_FAILED`
- `E5004_INVALID_LOWERING_RULE`
- `E5005_PHASE_NOT_FOUND`
- `E5006_MISSING_CONTRACT_TERM`
- `E5007_DUPLICATE_LOWERED_STREAM`
- `E5008_INVALID_CURVE` — duplicate curve name, duplicate point date, or
  malformed point in a `curve` statement
- `E5009_LOWERED_EXPR_INVALID` — a pack lowering rule expanded to an amount
  expression the parser rejects. Without this the engine evaluates the failed
  expression as zero and continues with only a warning.
- `E5020_LOWERED_STATE_INVALID` — a pack lowering rule expanded to a `state`
  `init` or `next` the parser rejects. Same reasoning as `E5009`: the engine's
  fallback for a failed state is zero, which would flatten every stream reading
  it rather than fail loudly.
- `E5021_DUPLICATE_LOWERED_STATE` — two contracts lower to one state name with
  *different* recurrences, so one would silently win. Give the rule's
  `state_name` a per-contract discriminator (`{{contract.suffix_ident}}`).
  Identical definitions collapse instead, which is what several contracts
  sharing one curve should do.
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
  honest option. Use a calendar the pack supports, or a pack that supports the
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
  periods defers to `inputs.<name>`. The conversion happens at compile time and
  an input is not known until the run.
- `E5019_UNKNOWN_DAY_COUNT` — a contract's `day_count` or
  `amortization_day_count` is not one of `30/360`, `30e/360`, `act/360`,
  `act/365`. Not defaulted silently: the gap between act/360 and act/365 is
  roughly 1.4% of interest.

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
- `E6021_CRE_OPS_INVALID_SCHEDULE`

`E6004_CRE_LEASE_UP_INVALID_OCCUPANCY` is **retired**: it validated lease-up
occupancy terms that no longer exist. Per §8 the code is never reused.

OpCo pack codes:

- `E7001_OPCO_LINE_MISSING_AMOUNT`
- `E7002_OPCO_LINE_INVALID_SCHEDULE`
- `E7003_OPCO_LINE_INVALID_GROWTH`
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

1. **Do not reuse codes**: once assigned, a code is never reused.
2. **Soft deprecation**: a deprecated code may remain emitted for one minor version with a note.
3. **Hard deprecation**: removal only in a major language version.

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
- `gold/<fixture>.diag.json`

Rules:
- Assert `code`, `severity`, `file`, and `span`.
- Messages are asserted in FULL. `tools/golden-runner` compares canonical JSON
  and diffs it, so rewording a message changes a golden and must be re-blessed
  with `CFDL_GOLD_UPDATE=1`.

  An earlier revision of this page said messages "may be asserted via substring
  match to allow minor wording changes". No runner has ever done that. The
  exact comparison is the better behaviour and is kept deliberately: a
  diagnostic's wording is part of its contract with the reader, and a silent
  drift in what the compiler says is exactly as bad as a silent drift in what
  it computes. Making a reword show up in a diff is the point, not friction.

