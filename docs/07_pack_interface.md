# Pack Interface

**CFDL Domain Pack Interface v0.1**

Domain packs provide *additions and overrides* on top of a single core language: each pack adds contract types, lowering rules, metrics, and validations without forking the language itself.

Core principle: **Packs may extend validation and provide defaults/templates, but MUST NOT change core language semantics.**

---

## 1) Overview

A **pack** is a versioned module that can:

- Provide **type registries** (domain entity/contract/option types)
- Provide **aliases** (domain names → canonical core concepts)
- Provide **contract term schemas** and **lowering rules** (contracts → streams/events/options)
- Provide **validations** (domain constraints with stable diagnostic codes)
- Provide **defaults** (required observables, output specification, reporting conventions)

### Non-goals
- Packs do not change core syntax.
- Packs do not add nondeterministic behavior.
- Packs do not embed external network calls.

### Why packs (and why not bake domains into core)

CFDL core must remain simple, strongly typed, and stable. Domain logic — contract forms, regulatory constraints, industry assumptions — changes frequently. Packs isolate that volatility.

### Determinism rules
Packs must be deterministic:
- same inputs ⇒ same lowered outputs
- stable ordering in any emitted lists
- no random/time/network access

---

## 2) Pack selection in CFDL

CFDL models MAY select a pack:

```cfdl
use pack "cre" version "0.1.0"
```

Compiler rules:
- `use pack` MAY appear **only** in `model.cfdl`.
- At most one pack MAY be active in v0.1.

If no pack is selected:
- Compilation still works using core rules.
- Unknown type IDs are permitted (with warnings) except where the compiler is configured to require a pack.

---

## 3) Pack identity and versioning

### 3.1 Pack ID
A pack MUST have a stable ID string:
- The ID is the bare pack name matching `name` in `pack.toml`
  (e.g., `cre`, `energy`, `credit`, `opco`); `use pack "<id>"` resolves
  against it. Namespaced ids (`publisher/name`) are reserved for a future
  multi-publisher registry.

### 3.2 Pack version
A pack MUST have a semver-like version string:
- `MAJOR.MINOR[.PATCH]`

### 3.3 Compatibility
- Compiler version and pack version are **independently versioned**.
- A pack declares which model *calendars* it supports via `cadences` (§5.1).
  It does NOT declare supported compiler IR versions: earlier revisions of this
  page said it MUST, but no such field has ever been read or shipped. See the
  note under §5.1 on fields this page once described that do not exist.

---

## 4) Pack distribution formats

A pack MAY be distributed as:
- a local directory (dev mode)
- a signed bundle file (zip/tar)
- a registry artifact (future)

v0.1 minimum: **local directory packs**.

---

## 5) Pack structure on disk

Packs are loaded from the filesystem (v0.2 default). Example structure:

```
packs/
  cre/
    pack.toml
    aliases.toml
    templates.toml
    lowering/
      rules.toml
    validations.toml
    defaults.toml
    outputs.toml
    README.md
  opco/
    pack.toml
    ...
```

### 5.1 Required manifest (`pack.toml`)

A pack directory MUST include `pack.toml`:

```toml
name = "cre"
version = "0.1.0"
description = "Commercial Real Estate domain pack"
cadences = ["monthly"]   # optional; empty or absent means every calendar

[entrypoints]
aliases = "aliases.toml"
lowering = "lowering/rules.toml"
metrics = "metrics.toml"
validations = "validations.toml"
```

Rules:
- `name` and `version` are REQUIRED. `description` is optional.
- `cadences` is optional and lists the model calendars the pack's rules lower
  correctly on (`daily`, `monthly`, `quarterly`, `annual`). Omit it, or leave
  it empty, and the pack is unconstrained — so a third-party pack that says
  nothing is unaffected. Declare it when the expressions assume a period
  length: a rule that divides an annual figure by a literal 12 is only correct
  on a monthly grid, and on any other one the schedule adapts while the amount
  does not. A model on an unlisted calendar is `E5013_PACK_CADENCE_UNSUPPORTED`.
  A single rule may narrow this further with its own `cadences`
  (`E5014_RULE_CADENCE_UNSUPPORTED`), which is what lets a pack carry neutral
  and month-locked rules side by side mid-migration.
- Every entrypoint is optional; a pack supplies only what it defines. The
  recognised keys are `aliases`, `templates`, `lowering`, `metrics` and
  `validations`, each a path relative to the pack directory. An unrecognised
  key is accepted and ignored, so check spelling: `packs/cre/pack.toml` once
  declared `defaults = "defaults.toml"`, which the loader has no field for, and
  the file sat unread.
- `version` is matched against the model's `use pack "<name>" version "<v>"`
  by exact string equality — there is no semver range logic. A pack present at
  a different version reports `E4004_MISSING_PACK` naming both versions, not
  a bare "not found".

Earlier revisions of this page described `pack_id`, `ir_versions`,
`entrypoints.types`, `contract_schemas`, `outputs` and `docs`. The loader has
never read any of them and no shipped pack declares them; a manifest written
to that description would load with no entrypoints at all.

### 5.2 Pack formats
- All pack artifacts are TOML-based.
- Keep pack files deterministic and avoid mixing YAML/JSON variants in the same pack.

---

## 6) Pack capabilities (what a pack can provide)

### 6.1 Type registry (ontology types)
A pack MAY define types used by:
- `entity ... : <TypeId>`
- `contract <TypeId> ...`
- `option ... type <TypeId>`

**Required:**
- A pack MUST provide a type registry file for at least the types it claims.

Minimum shape:
```json
{
  "types": [
    {
      "type_id": "CRE.Asset",
      "kind": "entity",
      "fields": {
        "city": {"type": "String", "required": false},
        "units": {"type": "Int", "required": false}
      }
    }
  ]
}
```

Type registry semantics:
- Packs MUST NOT remove core types.
- Packs MAY extend fields.
- Packs MAY provide documentation strings and examples.

### 6.2 Alias registry
Aliases map domain-friendly names to canonical TypeIds or contract templates.

Example:
```json
{
  "aliases": [
    {"alias": "Lease", "resolves_to": "Contract.Lease"},
    {"alias": "SeniorLoan", "resolves_to": "Contract.Loan.Senior"}
  ]
}
```

Compiler usage:
- Aliases are used by editors/CLI for suggestions.
- Aliases MAY be expanded during lowering if present in source.

Rules:
- Alias resolution must be deterministic.
- Packs must not create ambiguous alias collisions within a single loaded environment.

### 6.3 Contract term schemas
A pack MAY provide schemas for contract terms per TypeId.

Example:
```json
{
  "contracts": [
    {
      "type_id": "Contract.Lease",
      "terms": {
        "base_rent": {"type": "Money", "required": true},
        "rent_growth": {"type": "Rate", "required": false},
        "start_date": {"type": "Date", "required": false}
      }
    }
  ]
}
```

Compiler usage:
- If schema exists, the compiler MUST validate required terms and types.
- Schema validation errors are `E4003_INVALID_CONTRACT_TERMS`.

### 6.4 Lowering rules (contract → effects)
A pack MAY provide lowering rules that generate `effects` from `terms`.

**Core guarantee:**
- If a pack declares a lowering rule for a contract type, the compiler MAY allow `effects` to be omitted in source.

Lowering rule semantics:
- Inputs: contract instance (type, terms, subject, term date range)
- Output: one or more streams and/or derived term expansions
- Naming convention: generated stream names SHOULD be qualified names (dot-separated hierarchy), e.g. `cre.lease.base_rent`.
- Ownership convention: `owner` SHOULD resolve to a qualified entity symbol.

Rule interface options:
- **Declarative lowering** (recommended for v0.1)
- **Plugin function** (future)

v0.1 recommended declarative structure:
```json
{
  "lowering": [
    {
      "type_id": "Contract.Lease",
      "generates": [
        {
          "stream_name": "rent",
          "owner": "${subject}",
          "direction": "inflow",
          "currency": "",
          "schedule": {"kind": "Every", "every": "monthly", "on_rule": {"kind": "EndOfMonth"}},
          "amount_expr": {"lang": "cfdl", "src": "{{contract.base_rent}}"}
        }
      ]
    }
  ]
}
```

Template rules:
- `${subject}` resolves to the contract subject entity symbol. It is the only
  `${...}` substitution, and it applies to `owner_entity` alone.

`currency` is **not** templated, and earlier revisions of this page were wrong
to describe a `${contract.currency}`: a contract has no currency to resolve —
`ContractStmt` carries no such field, so nothing could ever have supplied one.
The real mechanism is simpler. Leave a rule's `currency` empty and the stream
inherits the model's declared currency, which is what keeps a pack usable
outside the United States. Set it only when the instrument is genuinely fixed
to one currency, and the model must then agree (`E2107`).

### Schedule interval

Omit `schedule_every` and a recurring rule pays at the model's calendar
cadence, which is what every shipped rule does. Set it when the instrument
pays on its own rhythm regardless of the grid the model happens to use — a
quarterly coupon or an annual true-up on a monthly model:

```toml
schedule_kind = "every"
schedule_every = "quarter"
```

Values are the schedule intervals: `day`, `week`, `month`, `quarter`, `year`.
An unrecognised value is `E5012_RULE_INVALID_INTERVAL`. An interval finer than
the model's calendar is `E2108_SCHEDULE_FINER_THAN_CALENDAR` — occurrences
inside one period share that period's environment and cannot be told apart, so
a pack cannot express what a model may not.

### Currency

Omit `currency` from a lowering rule. An empty value inherits the model's
declared currency, which is what keeps a pack usable outside the United
States — a PPA in Rajasthan is not a USD contract, and a lease in Frankfurt is
not either. The model is where a currency belongs, and it defaults to `USD`
when a model declares none, so nothing is lost by leaving it out here.

Set it only when the instrument is genuinely denominated in a fixed currency
regardless of the model around it — a USD-denominated eurobond, say. A rule
that pins a currency the model does not declare is rejected with
`E2107_STREAM_CURRENCY_MISMATCH`, because cash flows are summed period by
period and the two would otherwise be added as though the units matched.

Compiler behavior:
- Lowering runs after core validation.
- Generated streams MUST carry provenance notes referencing the contract.
- Generated streams SHOULD use deterministic dotted naming to preserve ontology/data-source mapping stability.
- If lowering fails, emit `E500x` and fail compilation.

### 6.5 Term payloads (current host behavior)
Contract `terms { ... }` values are captured as a lightweight map and exposed to pack lowering logic.

Current contract for packs:
- Terms are key/value pairs with string payloads plus source span.
- Packs are responsible for explicit parsing/coercion (e.g. Int/Decimal/Date).
- Packs should not rely on implicit casts; invalid values must emit diagnostics.
- If term-level spans are unavailable for a rule, use contract span consistently.

### 6.6 Ontology observable and reference IDs
Packs define canonical IDs for:
- `obs.rate(<name>)`, `obs.index(<name>)`, `obs.fx(<from>, <to>)`
- `ref.<name>`

Packs MAY provide registries:
- `observables.json`
- `refs.json`

Compiler behavior:
- If pack provides registries, the compiler MAY validate that referenced IDs exist.
- Missing observable IDs SHOULD be warnings in v0.1 (allow offline modeling).

### 6.7 Expression functions

The expression function vocabulary is fixed and built into the engine
(`cfdl-calc`) — `pmt`, `year_frac`, `cpr_to_smm`, `curve_value`, and so on
(see the [Expression Environment](/specification/expression-environment)).
Packs do **not** define their own expression functions in v0.1.

### 6.8 Pack validations

Packs can add domain-specific validations, e.g.:
- Lease must have start/end
- Construction loan must have draw period
- Exit cap must be within bounds

Validation must:
- Produce diagnostics with stable codes
- Include file/span when possible
- Never crash

Validations are declared as data in `packs/<pack>/validations.toml` and
evaluated by the compiler; they are not implemented in the engine. Each rule
names the contract it applies to, the check, a stable diagnostic code, and a
message. Available checks: `term_present`, `any_term_present`, `terms_mutually_exclusive`, `term_number`
(integer or decimal, with `min`/`max`/`exclusive_min`/`exclusive_max`, and
`when`/`on_invalid` to control absent and unparseable values),
`term_range_within_timeline`, `term_enum`, and `term_compare`. The set is
closed — no expressions, recursion, or message interpolation — so evaluating
a pack's validations is bounded work that cannot crash or hang the compiler.

The file declares its pack's reserved code prefix, which the loader enforces:

- `E6xxx_*` for CRE
- `E7xxx_*` for OpCo
- `E8xxx_*` for Energy
- `E9xxx_*` for Credit

Terms that a lowering template requires are checked generically by
`E5006_MISSING_CONTRACT_TERM`; validations express the domain constraints on
top of that (bounds, enumerations, and relationships between terms).

### 6.9 Documentation metadata
Packs SHOULD provide docs for:
- types and fields
- contract templates
- output definitions
- examples

Editors may use this for hover hints and snippet insertion.

### 6.10 Reporting: categories, subtotals and statements

A pack describes how its domain reports cash — what each line item *is*, what
subtotals and ratios matter, and how a statement is laid out. Declared in
`statements.toml`, registered in `[entrypoints]`:

```toml
[entrypoints]
statements = "statements.toml"
```

#### What you get

- **Per-period line items**, grouped the way the domain groups them.
- **Subtotals and ratios** computed every period, not just over the life of the
  deal — a lender tests coverage each year, and a lifetime ratio of 1.4 can
  contain a year at 0.9.
- **Several statements from one model.** A pack can publish a monthly pro forma
  and an annual summary of the same cash, and more than one *layout* of it: a
  remittance report and a statement of operations read the same pool
  differently.
- **A reconciliation** on every statement, so the bottom line is checked against
  the model's own cash rather than assumed to match.
#### Step 1 — declare the vocabulary

A **category** says what a stream is, economically. It is a dotted path whose
first segment is one of `operating`, `investing`, `financing` — the sections of
a statement of cash flows (ASC 230-10-45 / IAS 7.10). The pack declares the
leaves it uses in `pack.toml`:

```toml
categories = [
  "operating.revenue.base_rent",
  "operating.expense.opex",
  "financing.debt_service",
]
```

Keeping the set closed is what stops two models in the same pack spelling the
same idea two ways.

#### Step 2 — classify at the point of emission

The lowering rule that creates a stream says what it is:

```toml
[[rules]]
id = "cre_lease_base_rent"
category = "operating.revenue.base_rent"
```

A hand-written stream can declare one directly:

```
stream cre.unit.base_rent.a on entity asset.tower inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  category operating.revenue.base_rent
  amount = 10000
}
```

Because the rule that emits a stream is the thing that classifies it, a stream
is reported as a line **and** counted in its subtotal — never one without the
other.

#### Step 3 — declare subtotals

Each is a per-period series published as `domain.<pack>.<name>`.

```toml
[[subtotals]]
id = "domain.cre.noi"
kind = "money"
op = "sum"
categories = ["operating.*"]

[[subtotals]]
id = "domain.cre.debt_service"
kind = "money"
op = "negated_sum"
categories = ["financing.debt_service", "financing.mortgage_insurance"]

[[subtotals]]
id = "domain.cre.dscr"
kind = "number"
op = "ratio"
numerator = "domain.cre.noi"
denominator = "domain.cre.debt_service"
```

| field | meaning |
|---|---|
| `id` | output key; must start with `domain.` |
| `kind` | `money` or `number` |
| `op` | `sum`, `negated_sum`, `ratio` |
| `categories` | selectors to aggregate; `operating.*` matches the prefix and its children |
| `streams` | stream-name selectors, for what a category cannot express |
| `subtotals` | other subtotals to add, declared above this one |
| `numerator` / `denominator` | for `op = "ratio"` |
| `formula` | a human-readable note; not evaluated |

Declare in dependency order — a subtotal may reference only ones above it.

`negated_sum` exists because cash is stored signed. An expense is negative, and
a line a reader expects positive — debt service, a servicing fee — flips once
here rather than at every consumer.

A **ratio** is recomputed from its inputs at whatever grain it is reported at.
An annual coverage ratio is annual NOI over annual debt service; it is never the
average of twelve monthly ratios, which would be a different and wrong number.
Where the denominator is zero the value is `null`, not zero — a period with no
debt service has no coverage ratio.

#### Step 4 — lay out the statement

```toml
[[statements]]
id = "operating"
label = "Operating statement"
default = true

[[statements.rows]]
kind = "line"
label = "Base rental revenue"
depth = 1
categories = ["operating.revenue.base_rent"]

[[statements.rows]]
kind = "line"
label = "Less: vacancy"
depth = 1
categories = ["operating.deduction.vacancy"]
display = "positive"

[[statements.rows]]
kind = "subtotal"
label = "Net operating income"
depth = 0
subtotal = "domain.cre.noi"

[[statements.rows]]
kind = "ratio"
label = "Debt service coverage"
depth = 0
subtotal = "domain.cre.dscr"
```

Row kinds are `line`, `subtotal`, `ratio`, `spacer` and `residual`. `depth`
indents. `display = "positive"` shows a stored negative as a positive number in
a "less:" row — the published value stays signed, so anything consuming the
data still adds up correctly while a rendered statement reads the way a
practitioner expects.

#### How to report the same cash at another grain

Add a statement with a `grain`. Both are published from one run:

```toml
[[statements]]
id = "operating_annual"
label = "Operating statement (annual)"
grain = "annual"
```

Grain belongs to the output, not the run. Each statement publishes its own
column labels, since an annual view of a monthly model has ten columns where the
model has 120.

#### How to offer more than one view of a domain

Declare several statements over the same categories. The same pool can present
as a remittance report — principal split scheduled and unscheduled, because that
is what a prepayment speed acts on — and as a statement of operations reporting
total and net investment income. See `packs/credit/statements.toml`.

#### Every category must appear exactly once

In each statement, every category the pack declares must appear in exactly one
`line` row. This is checked when the pack loads, before any model runs.

A category in no row is cash the statement never shows, so the bottom line is
short. A category in two rows is cash counted twice — and a statement that is
wrong by double counting looks entirely plausible. The check is what lets a pack
offer several layouts safely: each is provably complete.

At run time a `residual` row catches any stream carrying no category at all, and
`reconciliation.residual` — the statement's bottom line against the model's cash
— is published on every statement whether it is zero or not.

## 7) Compiler ↔ Pack API (programmatic)

### 7.1 Pack loader interface
The compiler should expose a minimal interface:

- `load_pack(name, version) -> Pack`
- `Pack.type_registry() -> TypeRegistry`
- `Pack.aliases() -> AliasRegistry`
- `Pack.contract_schema(type_id) -> Option<ContractSchema>`
- `Pack.lowering_rule(type_id) -> Option<LoweringRule>`
- `Pack.observable_registry() -> Option<ObservableRegistry>`
- `Pack.ref_registry() -> Option<RefRegistry>`
- `Pack.output_spec() -> Option<OutputSpec>`

### 7.2 Error behavior
- Pack not found: `E4004_MISSING_PACK`
- Pack manifest invalid: `E4004_MISSING_PACK` with details
- Unsupported IR version: `E4004_MISSING_PACK` with details

### 7.3 CLI
Recommended CLI behaviors:

- `cfdl pack list --path packs/`
- `cfdl pack validate --path packs/`
- `cfdl compile <model> --packs packs/`
- `cfdl run <ir> --packs packs/ --config run.json`

---

## 8) Determinism and provenance with packs

### 8.1 Determinism
Pack identity MUST participate in determinism:
- The id generation seed includes `<name>@<version>` when a pack is active,
  because a different pack lowers a contract differently and so produces a
  genuinely different object.
- The seed MUST NOT include the compiler version. An id identifies a thing in
  a model, not the build that emitted it; including the version rewrote every
  id on every release, churning goldens and making a downstream store treat
  the same entity as new after an upgrade. The compiler version belongs in
  provenance, and stays there.

### 8.2 Provenance
The compiler SHOULD record pack info in top-level provenance notes.

---

## 9) Testing packs

Packs must be tested via golden fixtures.

Recommended test types:
- Pack load tests
- Alias resolution tests
- Template expansion tests
- Lowering tests (contract → streams)
- End-to-end example fixtures (compile + run results)

For each pack, include:
- `examples/<pack>/...` models
- `fixtures/valid/...` that use the pack
- Gold IR and results

---

## 10) Future-proofing (non-normative)

v0.2+ may add:
- multi-pack layering (base + overlays)
- signed pack artifacts
- executable lowering plugins (WASM)
- richer ontology reasoning
- sensitivity analysis definitions in output spec
- custom engine computation plugins

This v0.1 interface is designed to evolve without breaking core models.

---

## Parameterized lowering rules (templates)

Lowering-rule fields `amount_expr`, `schedule_from`, `schedule_to`,
`stream_name`, `schedule_net_days`, `schedule_net_months` and `schedule_every`
may contain `{{...}}` placeholders, resolved at compile time:

1. `{{contract.term_start}}` / `{{contract.term_end}}` — the contract's
   `term A..B` range (normalized dates).
2. `{{contract.<key>}}` — the contract's `terms { <key> = <value> }` entry
   (dotted keys like `lease_up.months` are supported).
3. Rule defaults — a per-rule `[rules.defaults]` table supplies fallback
   values when the contract does not declare the term.

A placeholder with no contract value and no default is a compile error
(`E5006_MISSING_CONTRACT_TERM`), one diagnostic per missing key.

### Where a one-shot flow sits in its period

`schedule_kind = "on_date"` settles on the stated date, which discounts from
the period's open. Set `schedule_at_period_end = true` for a **disposal**: a
reversion is taken at the end of the holding period and discounts the full n
periods. The distinction is by kind, not by being one-shot — acquisitions,
draws, dated leasing costs and tax credits all want the default.

### Mid-period discounting

`schedule_mid = true` puts a rule's cash halfway through the period that earned
it rather than at its end — the mid-period (mid-year) convention that project
finance and banker DCFs use, on the reasoning that a period's cash arrives
throughout it rather than on the last day.

It is a convention rather than a date, so it is half a period on every
calendar. It works on `every` and on `on_date` rules alike, and it is mutually
exclusive with `schedule_due` and with a day rule — all three name a position,
and stating two is `E2109_SCHEDULE_CONFLICTING_PLACEMENT`.

Use it for operating flows. Do not use it for a **price**: a disposal, a
terminal value or an acquisition is struck at a point in time and discounts
whole, which is what `schedule_at_period_end` is for.

### A rule may declare a state

A rule that compounds a rate which MOVES cannot use `pow(1 + g, t)` — that
applies one period's rate as though it had held from the start, which is exact
while the rate is flat and wrong the moment it varies. Three optional fields let
the rule declare a recurrence instead:

```toml
amount_expr = "{{contract.amount}} * state.opco_revenue_growth{{contract.suffix_ident}}"
field_name  = "opco_revenue_growth{{contract.suffix_ident}}"
field_init  = "1"
field_next  = "prev * pow(1 + curve_value(\"{{contract.growth_curve}}\", time.date), 1 / {{model.periods_per_year}})"
```

A state may also carry its own clock:

```toml
field_every = "{{contract.payment_frequency}}"
field_from  = "{{contract.term_start}}"
field_to    = "{{contract.term_end}}"
```

`field_every` is the recurrence's cadence — `day`, `week`, `month`, `quarter`,
`year`. Empty means every model period. **Set it whenever the recurrence belongs
to the instrument's rhythm rather than the book's.** A pool carried on a daily
calendar but paying monthly must compound twelve times a year, not three hundred
and sixty-five, and `{{time.elapsed_periods}}` counts the rule's *payment*
periods — so a rule whose `schedule_every` is templated almost certainly wants
`field_every` templated the same way.

The state steps on its **accrual** periods and **holds** between ticks and
outside `field_from`/`field_to`. It does not fall to zero: that is what
separates a schedule from `active when`, which a state deliberately does not
have. An interval finer than the model calendar is
`E2108_SCHEDULE_FINER_THAN_CALENDAR`, the same rule a stream obeys.

All templated. `field_name` must expand to a **single identifier** —
`state.<name>` resolves one segment, so `{{contract.dot_suffix}}` would produce
an unreachable path; use `{{contract.suffix_ident}}`.

States are deduplicated by name across contracts. Identical definitions collapse,
which is what several contracts sharing one curve should do; differing ones are
`E5021_DUPLICATE_LOWERED_STATE` rather than one silently winning. A state the
*model* declares under the same name wins over the rule's — a pack should never
invisibly override what a modeller wrote.

The construct itself is language-level and needs no pack: see
`docs/14_state_and_recurrence.md` and §3.1 of `docs/03_expression_environment.md`.

### Cadence placeholders

A rule must not assume how long a period is. These placeholders carry that,
and are resolved *before* contract terms — declaring a term under one of the
reserved prefixes `model.`, `time.`, `periods.` or `whole_periods.` is
`E5016_RESERVED_TERM_PREFIX`, because the term would never be read.

| Placeholder | Expands to |
|---|---|
| `{{model.periods_per_year}}` | `365` / `52` / `12` / `4` / `1` |
| `{{model.calendar}}` | the rule's effective frequency name |
| `{{model.accrual_divisor}}` | what a nominal annual rate divides by |
| `{{model.amortization_divisor}}` | what a level payment is struck from |
| `{{periods.<term>}}` | `<term>` months as periods, fractional allowed |
| `{{whole_periods.<term>}}` | the same, but must be integral |
| `{{time.elapsed_periods}}` | whole periods since `term_start` |
| `{{time.elapsed_years}}` | whole years since `term_start` |
| `{{time.periods_to_term_end}}` | whole periods from now to `term_end` |
| `{{contract.dot_suffix}}` | the contract instance's suffix, `.core` |
| `{{contract.suffix_ident}}` | the same without the dot, `_core` — for state names |

**Periods-per-year comes from the rule's payment interval, not the model's
calendar.** A rule that declares `schedule_every` accrues on that rhythm, so a
monthly-paying loan carried on a daily book divides by 12, not 365. This is why
the value is resolved here rather than exposed at run time: the expression
environment sees the calendar and knows nothing of the schedule. Hand-written
models get `time.ppy` instead, which is the calendar-based equivalent.

`schedule_every` is itself templated, so a contract can declare its own rhythm
(`payment_frequency = "month"`) and one rule can serve the monthly, quarterly
and daily-book versions of an instrument.

**`_months` terms always mean calendar months**, on every calendar: they
describe the contract, not the modeller's grid. `{{periods.X}}` converts one
into a possibly-fractional period count and is for thresholds — five months
free rent is 5 periods monthly, 1.667 quarterly, 0.417 annually, and pro-rates
exactly in each case. `{{whole_periods.X}}` is for payment *counts* that go
into `pow` exponents and annuity term arguments, where a fractional value is
meaningless: a 30-month loan is not 2.5 annual payments, so that is
`E5015_TERM_MONTHS_NOT_DIVISIBLE` rather than a rounding. Both need a literal;
a term deferred to `inputs.<name>` cannot be converted at compile time
(`E5017_PERIOD_TERM_NOT_LITERAL`).

**Day count.** A nominal annual rate becomes a periodic one by dividing, but
by what depends on the convention. `{{model.accrual_divisor}}` reads the
contract's `day_count` term and expands to:

| `day_count` | expands to | meaning |
|---|---|---|
| absent, `30/360`, `30e/360` | `<ppy>` | every period is 1/ppy of a year |
| `act/360` | `(360 / time.days_in_period)` | actual days over a 360-day year |
| `act/365` | `(365 / time.days_in_period)` | actual days over a 365-day year |

Dividing by `(360 / days)` is multiplying by `days / 360`, so a 31-day January
accrues more than a 28-day February — which is the point of an Actual
convention. On a daily grid it collapses to `rate / 360`. The default expands
to exactly the same text as `{{model.periods_per_year}}`, so a rule can adopt
the placeholder without changing any existing model. An unrecognised value is
`E5019_UNKNOWN_DAY_COUNT` rather than a silent fallback: act/360 against
act/365 is about 1.4% of interest.

Use it for every **nominal** rate — note rates, servicing strips, floating
index-plus-margin. Do not use it for annual *quantities* (`rent_year`,
`om_year`), which spread by `{{model.periods_per_year}}` regardless of day
count.

**Amortisation is a second, separate basis.** An amortising loan strikes its
level payment once, from a schedule the parties agree, and then accrues interest
period by period on whatever the accrual convention says; principal is the plug.
Those are two different divisors, and collapsing them makes the payment itself
move with month length — which no amortising instrument does.
`{{model.amortization_divisor}}` reads an `amortization_day_count` term and
expands by the same table as `{{model.accrual_divisor}}`, **defaulting to
`day_count`** when absent. So:

- a rule that uses only `{{model.accrual_divisor}}` is unchanged;
- a model that sets only `day_count` is unchanged, both divisors agreeing;
- `day_count = "act/360"` with `amortization_day_count = "30/360"` is the
  common US commercial case — a fixed payment, interest varying by month length.

An amortising rule should therefore strike the annuity factor from
`{{model.amortization_divisor}}`, accrue interest from
`{{model.accrual_divisor}}`, and make scheduled principal the difference.
`amortization_day_count` is validated by the same `E5019_UNKNOWN_DAY_COUNT`.

Two conventions that must not be confused when dividing by periods-per-year:
note rates are **nominal** and divide (`rate / ppy`), while CPR and CDR are
**effective annual** and take a root (`cpr_to_periodic(x, ppy)`). Growth and
escalation are effective-annual too and step on `{{time.elapsed_years}}`.

Substitution is textual: numeric terms yield valid expression fragments;
string-valued terms must be quoted inside the template. Example:

```toml
[[rules]]
id = "lease_base_rent_v2"
contract_name = "cre.lease"
stream_name = "cre.lease.base_rent"
owner_entity = "${subject}"
direction = "inflow"
amount_expr = "{{contract.base_rent}} * clamp((time.t - {{contract.lease_up.start_period}} + 1) / {{contract.lease_up.months}}, 0, 1)"
schedule_kind = "every"
schedule_from = "{{contract.term_start}}"
schedule_to = "{{contract.term_end}}"

[rules.defaults]
"lease_up.start_period" = "6"
"lease_up.months" = "18"
```

---

## Declarative domain metrics (metrics.toml)

Packs declare their metric sets via a `metrics = "metrics.toml"` entrypoint;
the engine host evaluates them after a run (`cfdl run --pack <name>`). Adding
a domain means adding a metrics.toml, not Rust. Metrics are evaluated in file
order, so ratios may reference earlier ids.

```toml
[[metrics]]
id = "domain.cre.noi"           # output key
kind = "money"                  # money | number
op = "sum"                      # sum | negated_sum | ratio | wal_years
numerator_streams = ["cre.lease.base_rent", "cre.ops.revenue"]
denominator_streams = ["cre.ops.expense"]
formula = "sum(numerator_streams) + sum(denominator_streams)"  # lineage text

[[metrics]]
id = "domain.cre.debt_service"
kind = "money"
op = "negated_sum"
numerator_streams = ["loan.permanent_debt_service"]
formula = "-sum(numerator_streams)"
require_positive = true          # omit unless value > 0

[[metrics]]
id = "domain.cre.dscr"
kind = "number"
op = "ratio"
numerator_metric = "domain.cre.noi"
denominator_metric = "domain.cre.debt_service"
formula = "domain.cre.noi / domain.cre.debt_service"
# ratios are omitted when either input metric is absent or the denominator ~ 0

[[metrics]]
id = "domain.credit.wal_years"
kind = "number"                  # wal_years requires kind = "number"
op = "wal_years"
numerator_streams = ["credit.pool.sched_principal.*", "credit.pool.prepay.*"]
formula = "wal_years(numerator_streams)"   # sum(((t + offset)/ppy) * v) / sum(v)
# weighted average life in years of the matched streams' positive per-period
# amounts: sum(t/ppy * v) / sum(v), using the engine's run.periods_per_year;
# omitted when the matched streams have no positive amounts
```

Engine-universal metrics are computed for every model regardless of pack:
`model.npv`, `model.irr`, `model.moic`, `model.payback_periods` /
`model.payback_years` (when cumulative net cash turns non-negative, given the
model starts cash-negative), and `model.wal_years` (inflow-weighted average
life).

`npv`, `irr`, `wal_years` and `payback_years` are all measured on the **same
time axis**: a flow sits at `(period + offset)`, where `offset` is its
placement in the period (`docs/12_payment_timing.md`). `moic` is a ratio of
cash in to cash out and does not care when the cash moved, so it uses the plain
net series.

The time-weighted ones net **within an offset, not across one**: two flows in
the same period at different points in it are not the same cash at the same
moment, so a purchase settling on its date does not cancel that period's
collections. When every stream shares a placement this reduces exactly to the
net cash-flow series.
