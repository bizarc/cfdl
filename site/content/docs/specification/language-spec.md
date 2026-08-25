---
id: language-spec
title: "Language spec (v0.1)"
slug: "/docs/specification/language-spec"
description: "The normative specification for CFDL v0.1 — entities, contracts, streams, time, and the canonical IR a model compiles to."
source: docs/01_language_spec.md
generated: full
layer: specification
---

**Status:** Draft

**Purpose:** CFDL (Cash Flow Domain Language) is a proprietary, human-readable DSL for defining cash-flow models across asset classes. A CFDL model compiles deterministically to a canonical JSON IR used by valuation engines (deterministic DCF, Monte Carlo, scenarios, risk/metrics).

## Normative keywords

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in BCP 14 ([RFC 2119](https://www.rfc-editor.org/rfc/rfc2119), [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174)) when, and only when, they appear in all capitals.

This specification exists so that a second implementation can be written from it. That is the reason the distinction is stated rather than assumed: a reader has to be able to tell a requirement from advice without inferring it from the surrounding sentence.

---

## 1. Design goals

### 1.1 Non-negotiables
1. **Human-readable, file-based modeling:** models are composed of `*.cfdl` files.
2. **Deterministic compilation:** the same inputs produce the same canonical IR (subject to explicitly controlled seeds for stochastic runs).
3. **Separation of concerns:** **Time**, **Structure**, and **Behavior** are separate concepts in the language.
4. **Contracts are first-class:** all domain concepts can be represented as a **contract** with terms and effects.
5. **Streams are first-class:** cash flows are represented as **streams** (time-indexed vectors) attached to an owning entity.
6. **Events are first-class:** discrete time-step events can mutate entity state and activate/deactivate behaviors.
7. **Strong typing:** core types (Money, Date, Frequency, Rate, etc.) are defined and validated.
8. **Multi-currency:** Money values include currency; conversions are explicit in expressions.
9. **Single canonical IR:** compiler output is one JSON IR format used by all engines.
10. **No correlations in core:** correlation is not a core-language concept and is not present in the IR.

### 1.2 What v0.1 intentionally excludes
- Domain Pack specifications (defined separately)
- Correlated sampling declarations
- Optimization policies for options (beyond deterministic triggers)
- Comprehensive relationship constraints/ontology reasoning beyond type validation

---

## 2. Model lifecycle and artifacts

### 2.1 Compilation pipeline (normative)
1. **Parse**: `*.cfdl` sources → AST
2. **Resolve**: imports + symbol table (entities, contracts, streams)
3. **Validate**: type-check + schedule/time bounds + required fields
4. **Lower**: contract terms + effects → canonical IR objects (preserving provenance)
5. **Emit**: canonical JSON IR

### 2.2 Execution pipeline (informative)
- Engines consume IR and produce Results JSON.
- Deterministic runs produce exact cashflows and metrics.
- Monte Carlo runs sample distributions declared in CFDL assumptions with explicit seeds.

---

## 3. Files, modules, and imports

### 3.1 File extension
- CFDL source files MUST use `.cfdl`.

### 3.2 Entry module
- A model directory MUST contain an entry file named `model.cfdl`.

### 3.3 Imports
Syntax:

```cfdl
import "relative/path/to/file.cfdl"
import "contracts/lease.cfdl" as lease
```

Rules:
- Import paths are **relative to the importing file**.
- Import cycles are forbidden.
- The compiler MUST establish a deterministic import order (e.g., lexical by resolved absolute path) and treat the model as a single merged module.

### 3.4 Domain pack selection
Syntax:

```cfdl
use pack "cre" version "0.1.0"
```

Rules:
- `use pack` MAY appear in `model.cfdl` only.
- If multiple `use pack` statements exist, compilation MUST fail.
- A pack MAY:
  - define/extend the ontology type registry
  - define alias mappings
  - define validators for contract terms/types
  - define contract lowering rules
  - (reserved) define helper expression *sugar* that expands to core primitives
    — not in v0.1: the expression function vocabulary is engine-owned and fixed
    (see Pack Interface §6.7); packs compose the existing primitives

Core language semantics MUST NOT change based on pack selection.

---

## 4. Lexical conventions

### 4.1 Identifiers
- Identifiers match: `[A-Za-z_][A-Za-z0-9_]*`
- Qualified identifiers (Type IDs, namespaces) use dot notation: `A.B.C`
- Dot (`.`) is structural: it separates hierarchical name segments.
- Underscore (`_`) is lexical: it is allowed *within* a segment but does not create hierarchy.

### 4.2 Literals
- String: `"..."` with escapes.
- Integer: `123`, allow `_` separators (`8_500_000`).
- Decimal: `0.035`, allow `_` separators.
- Boolean: `true` / `false`.

### 4.3 Dates
- Date literals MUST be ISO-like:
  - `YYYY-MM-DD` (date)
  - `YYYY-MM` is permitted where a *period start* is unambiguous (see Time normalization).

### 4.4 Comments
- Line comment: `// ...`
- Block comment: `/* ... */`

---

## 5. Core type system (strong typing)

### 5.1 Primitive types
- `String`, `Bool`, `Int`, `Decimal`

### 5.2 Temporal types
- `Date` (calendar date)
- `DateRange` (`start..end`, inclusive of start, inclusive of end unless otherwise stated)
- `Frequency` (`daily`, `weekly`, `monthly`, `quarterly`, `annual`)
- `Duration` (e.g., `30d`, `12m`, `30y`)

### 5.3 Financial types
- `Currency` (ISO 4217 string, e.g., `USD`)
- `Money` = `{ amount: Decimal, currency: Currency }`
- `Rate` (unitless decimal with semantic meaning)
- `Percent` (syntactic sugar for Rate; `10%` == `0.10`)

### 5.4 Reference types
- `EntityRef` (reference to an entity symbol)
- `ContractRef` (reference to a contract instance name)
- `StreamRef` (reference to a stream instance name)

### 5.5 Type checking rules (normative)
- Any `amount` that represents money MUST have a currency, either explicitly or inferred from surrounding context.
- Streams MUST have a declared currency.
- Expressions MUST type-check to the required slot type.

---

## 5.6 Model currency

A model declares the currency it reports in:

```cfdl
model "solar-portfolio" currency INR
```

Rules:
- The code is an ISO 4217 identifier. When omitted, the model reports in `USD`.
- Every metric — `model.npv`, `model.total`, entity and domain metrics — is
  denominated in this currency.
- A stream MUST declare the same currency. Cash flows are summed period by
  period, so a stream in another currency would be added as though it were the
  same unit; the compiler rejects the mismatch
  (`E2107_STREAM_CURRENCY_MISMATCH`) rather than produce a meaningless total.
- Cross-currency models require an explicit conversion in the amount
  expression. The language does not apply FX rates implicitly.

---

## 6. Time model

### 6.1 Master timeline
Syntax:

```cfdl
time calendar monthly from 2026-01-01 for 120
```

Rules:
- `calendar` MUST be one of: `daily`, `monthly`, `quarterly`, `annual`.
- `from` MUST be a `Date`.
- `for` MUST be a positive integer count of periods.
- The compiler MUST derive the sequence of time points `t[0..N-1]`.

### 6.2 Date normalization
- If `YYYY-MM` is used, it MUST be normalized to the first day of the month (`YYYY-MM-01`).
- All schedules MUST ultimately resolve to occurrences that align to the master timeline’s grain or be mapped deterministically (see schedule resolution rules).

### 6.3 Phases
Syntax:

```cfdl
phase construction from 2026-01-01 to 2026-12-31
phase perm from 2027-01-01 to 2031-12-31
```

Rules:
- Phase date ranges MUST fall within the master timeline.
- Phases are named ranges used for organization, scoping, and schedule helpers.

### 6.4 Phase boundary helpers
Three helpers resolve in SCHEDULE position (see §11):
- `phase_start("name")`
- `phase_end("name")`
- `phase_enter("name")` (an instant)

They are not expression functions. An expression reads the phase it is in as
`time.phase`, the name of the phase covering the current period, so a guard
gates on a phase with `active when time.phase == "operations"` and an event
fires on entering one with `when time.phase == "operations"` — once, because an
event latches (§13.1).

---

## 7. Entities (Structure)

### 7.1 Entity declaration
Syntax (v0.1 core):

```cfdl
entity asset sunset
```

Rules:
- Declaration form: `entity <family> <name>` — two bare identifiers — optionally
  followed by `: <Type>` and a block.
- References use the qualified dotted form: `asset.sunset`
  (e.g. `on entity asset.sunset`).

```cfdl
entity asset tower : CRE.Asset.RealProperty {
  asset_class = "office"
  state operating
}

entity asset suite_a : CRE.Asset.Unit {
  rentable_area = 10000
  part of asset.tower
  state leased
}

entity party acme : CRE.Party.Tenant { name = "Acme Corp" }
```

- The **family** is the first identifier: `asset` for something that produces or
  consumes cash, `party` for someone who contracts, owns, lends or invests.
- The **type** is checked against the active ontology. With no pack active a
  model still has the language's own vocabulary — `Asset.Real`,
  `Asset.Financial`, `Asset.Intangible`, `Party` — because an ontology is a
  language capability that packs supply defaults for, not one they own. A pack's
  types are added to those and cannot remove them.
- Attribute values are **literals**, checked against the type's declared fields.
- `part of` declares hierarchy and is **always optional, at every grain**. A
  pool models collective behavior perfectly well with no loans under it; a
  building needs no units. The modeller chooses the grain and the language does
  not prefer one. Where a parent does have children, its cash is aggregated from
  them **by the relation**, not by a name prefix — published as
  `entity.<symbol>.net_cash_flow`.
- An untyped entity remains legal, so a model written before types existed still
  compiles. The type is what unlocks the checks, not a condition of being an
  entity.

### 7.2 Entity state
- An entity's type MAY declare a **lifecycle**: a closed set of states with a
  mandatory initial one. An entity with a lifecycle is ALWAYS in exactly one of
  its states, from period 0 — there is no null state and no undeclared state.
- Events move it:

```cfdl
event refinance when time.t >= 12 {
  set entity loan.senior.status = "refinanced"
}
```

Rules:
- Entity state is the primary mechanism used to trigger/activate contractual
  behavior. A stream may test it directly with `active in state <name>`, which
  checks the name against the owner's declared lifecycle — a string comparison
  cannot be checked, and `entity.status == "leasd"` is simply false
  forever.
- Every write is published in `deterministic.transitions`, so a transition is
  observable and assertable rather than inferred from the cash.

---

## 8. Contracts (Behavior container)

For when to use contracts vs standalone streams, see the **Language guide** ("When to use streams vs contracts").

### 8.1 Contract declaration (canonical form)
Syntax (normative):

```cfdl
contract cre.lease on entity asset.sunset {
  term 2026-02..2028-01
  terms {
    base_rent = 42000
    escalation = 0.03
  }
}
```

Rules:
- `contract <TypeId>[.<instance>] on entity <EntityRef> { term <Date>..<Date>  terms { ... } }`
- `<TypeId>` is the pack contract type (e.g. `cre.lease`); an optional dotted
  instance suffix creates independent instances
  (`cre.lease_unit.tenant_a`, `cre.lease_unit.tenant_b`).
- `term` is REQUIRED.
- `terms` is OPTIONAL; entries use `<name> = <literal-or-expression>`.
- Monetary amounts default to the model currency; streams declare currency
  explicitly.
- Effects come from the active pack's lowering rules, which expand the
  contract into streams in the IR. Explicit `effects`/`parties`/`tags`
  blocks are **tolerated by the parser but not represented in IR** in v0.1
  (reserved; see `10_implementation_status.md`).

### 8.2 Terms block
- `terms { ... }` is a set of named values.
- Term keys MAY be qualified names (e.g., `lease_up.months`).

#### 8.2.1 A term records what was agreed (normative)

A term's value is one of:

- a **literal** — a number, string, date, or `true`/`false`;
- a **reference to one declared input**, written `inputs.<name>`; or
- an **expression**.

A contract records what was agreed — and what was agreed is often itself an
expression. A lease escalating at "CPI plus 50 basis points" agreed exactly
that; so did a coupon of "SOFR + 225bp". The term states it directly:

```cfdl
curve cpi step {
  2026-01: 0.021
  2027-01: 0.024
}

contract cre.lease on entity asset.sunset {
  term 2026-02..2036-01
  terms {
    base_rent  = 42000                                  // literal fact
    escalation = curve_value("cpi", time.date) + 0.005  // the agreed formula
  }
}
```

An expression term is compiled at the term's own site
(`E5025_TERM_EXPR_INVALID` if it does not parse) and substituted into the
pack's lowering rule **parenthesised**, so `a + b` multiplied by another term
associates the way it reads. It is valid only where the rule uses the term in
an expression position — an amount or a field rule. A term a rule reads as a
name, a date, a frequency, or a period count must stay a literal
(`E5026_TERM_EXPR_IN_LITERAL_SLOT`, `E5017_PERIOD_TERM_NOT_LITERAL`).

A quantity that varies **per run** — a yield under study, an escalator being
stressed — is still named as an input rather than computed inline:

```cfdl
assume annual_yield ~ Normal(mean=5000, stdev=350, clip=[4000, 6000])

contract energy.ppa.plant_a on entity project.plant {
  term 2026-01..2050-12
  terms {
    ppa_price = 3000              // contractual fact
    mwh_year  = inputs.annual_yield  // driver, supplied per run
  }
}
```

The value then arrives from whichever layer is driving the run — `assume x = …`
for a fixed case, a scenario's `parameters` in `run.json`, or a Monte Carlo
draw. All three write to the same `inputs.<name>` channel, so one declaration
serves every mode and variation stays layered on top of the contract rather
than embedded inside it. An expression term may reference inputs
(`inputs.cpi + 0.005`) and keeps that property.

Bounds are checked where the value is knowable. A literal term is checked
against the pack's declared bounds at compile time. A term referencing an
input is checked against that input's `clip`
(`E5011_TERM_CLIP_OUT_OF_BOUNDS`), and referencing an undeclared input is an
error (`E5010_TERM_UNKNOWN_INPUT`). An expression term's value is not knowable
until the run, so pack bounds do not apply to it at compile time — the same
tier as an input reference.

Pack interaction:
- A pack MAY provide a schema for `<TypeId>` and validate `terms`.
- If no schema exists, terms are allowed but only minimally validated.

### 8.3 Effects block
- `effects { ... }` contains effect declarations that produce IR objects.
- In v0.1 core, supported effects are:
  - `stream` definitions (cash flow series)
  - (optional) `activate/deactivate` declarations via events, not in static effects

### 8.4 Contract names and references
- Contract instance names MUST be unique across the model.
- Contract instance names MUST be qualified names with at least two segments; uniqueness applies to the full name.
- Expression-level contract references (`contract("...")`) are reserved and not in the v0.1 dialect.

---

## 9. Streams (cash flow vectors)

For when to use standalone streams vs contracts, see the **Language guide** ("When to use streams vs contracts").

### 9.1 Stream declaration (standalone)
Syntax:

```cfdl
stream asset.taxes on entity asset.sunset outflow currency USD {
  schedule every month from 2026-01 to 2031-12
  amount = 150000 / 12
}
```

Rules:
- Streams MUST be owned by exactly one entity.
- Streams MUST declare direction: `inflow` or `outflow`.
- Streams MUST declare a currency.
- Stream names MUST be qualified names with at least two segments (e.g., `cre.lease.base_rent`, `real_estate.ops_expense`).

### 9.2 Stream declaration inside contract effects (reserved)

Explicit `effects { stream ... }` blocks inside contracts are declared in
the grammar but **not represented in IR** in v0.1 — pack lowering rules are
the mechanism that turns contracts into streams. See
`10_implementation_status.md`.

### 9.3 Activation guards
Streams MAY include an activation predicate:

```cfdl
active when entity.status != "refinanced"
```

If omitted, streams are active for all scheduled occurrences.

### 9.4 Amount
- Streams MUST define an `amount` expression.
- Amount expressions MUST evaluate to `Money` or `Decimal` depending on slot; in v0.1, `amount` MUST evaluate to `Money` (or a `Decimal` that is implicitly converted to Money using the stream currency).

---

## 10. Waterfalls (ordered allocation)

Some cash is not earned, it is allocated. A waterfall declares a priority of
payments: an ordered list of steps sharing out a pot.

### 10.1 Waterfall declaration
Syntax:

```cfdl
waterfall deal.distribution on entity asset.trust {
  schedule every month from 2026-01 to 2030-12
  from available

  pay servicing to party.servicer    = 12500.0
  pay senior    to asset.class_a     = 6250.0
  pay residual  to party.certificate = remaining
}
```

Rules:
- A waterfall MUST be owned by exactly one entity.
- A waterfall MUST declare a `schedule` — the same construct a stream takes —
  and a `from` expression, which is the pot.
- Waterfall names MUST be qualified names with at least two segments.
- A waterfall MUST declare at least one step.

### 10.2 Steps (normative)
A step is `pay <name> to <payee> = <expr>`. There is one form for every kind of
payment; a fixed fee, a cap, a balance target and a shortfall are all
arithmetic over the names in §10.3.

- Step names MUST be unique within their waterfall.
- Every step MUST declare an amount expression.
- Steps are paid in **declaration order**.
- A step pays `min(max(0, expr), remaining)`. A step asking for more than is
  left takes what is left; a negative expression pays nothing rather than
  clawing cash back.
- At least one step MUST read `remaining`, so the residual has a named payee
  instead of vanishing.

### 10.3 What a step may read
On top of the ordinary expression environment (§16):

| binding | meaning |
|---|---|
| `remaining` | what is still in the pot at this step |
| `paid.<step>` | what an earlier step actually paid |
| `owed.<step>` | what an earlier step would have paid, unbounded |

`owed` and `paid` differ exactly when a step could not be paid in full, so
their difference is that step's shortfall.

A step MUST NOT read a step declared after it. A priority of payments is an
order, not a system of equations, so a forward reference is a compile error.

### 10.4 Evaluation order (normative)
A waterfall runs **after** the period's fields and streams are evaluated, so it
allocates cash that already exists. A waterfall MUST NOT feed a stream in the
same period.

### 10.5 Output and composition
Each step publishes as a series named `stream.<waterfall>.<step>`, and its cash
counts toward the payee's total. A waterfall is not a separate kind of output:
statements, metrics and the results document read it as they read any stream.

Because steps publish as series, a waterfall MAY draw on the output of a
waterfall **declared before it** — a fund's carry becoming a management
company's pot, and that company's own share becoming a third waterfall's.
Composition follows declaration order, the same rule steps follow within a
waterfall.

---

## 11. Schedules (important)

Schedules define occurrence dates/times for streams.

### 11.1 Core schedule grammar
Schedules appear inside a stream block:

```cfdl
schedule <schedule_expr>
```

### 11.2 Schedule expressions
#### 10.2.1 One-time
```cfdl
schedule on 2026-02-15
```

#### 10.2.2 Bounded recurring (frequency)
```cfdl
schedule every month from 2026-02-01 to 2028-01-31
```

#### 10.2.3 Position in the period
```cfdl
schedule every year start from 2026-01 to 2030-01 // start of each period
schedule every year mid   from 2026-01 to 2030-01 // halfway through
schedule every year end   from 2026-01 to 2030-01 // end (the default)

schedule on 2029-06 start                         // one-shot (the default)
schedule on 2029-06 mid                           // one-shot, same axis
schedule on 2029-06 end                           // one-shot, settles at close
```

**One axis, three positions, at most one.** `start`, `mid` and `end` say where
in its period a payment sits, and so how far it is discounted. Stating two is a
parse error rather than a diagnostic — the positions are alternatives, not
flags.

What differs between the forms is only which position they DEFAULT to when the
model says nothing: a recurrence defaults to `end` (an ordinary annuity — the
interval elapses, then payment falls), a one-shot to `start` (it settles on the
date stated, not after waiting a period it never waited through). Because the
default is not a single constant, every position is nameable in both forms so a
model never has to rely on it.

`start` is an annuity due, which is what expense-like streams want. `mid` is the project-finance mid-period convention: cash
arrives through the period rather than at one end, so it is summarized at the
midpoint — half a period on **every** calendar, unlike a day rule. Stating two
positions at once is `E2109_SCHEDULE_CONFLICTING_PLACEMENT`. See
`docs/12_payment_timing.md`.

#### 10.2.4 Day rules
```cfdl
schedule every month on day 1 from 2026-02-01 to 2028-01-31
schedule every month on eom from 2026-02-01 to 2028-01-31
```

#### 10.2.5 Weekday sets (daily/weekly)
```cfdl
schedule every week on Mon,Wed,Fri from 2026-02-01 to 2026-06-30
```

#### 10.2.6 Business-day conventions
```cfdl
schedule every month on eom
  from 2026-02-01 to 2028-01-31
  convention modified_following
  calendar "NYSE"
```

Conventions:
- `none`
- `following`
- `modified_following`
- `preceding`
- `modified_preceding`

#### 10.2.7 Stub rules
```cfdl
schedule every month on day 15
  from 2026-02-01 to 2026-12-31
  stub short_front
```

Stub policies:
- `none`
- `short_front`, `short_back`
- `long_front`, `long_back`

#### 10.2.7 Include/exclude
```cfdl
schedule every month on eom from 2026-02-01 to 2028-01-31
  except [2026-12-31]
  also [2027-01-02]
```

#### 10.2.8 Phase-relative helpers
```cfdl
schedule on phase_enter("perm")
schedule every quarter from phase_start("hold") to phase_end("hold")
```

### 11.3 Schedule resolution rules (normative)
- A schedule MUST resolve to a set of dates.
- If the master timeline is not daily, dates MUST be mapped deterministically to the nearest representable period boundary according to the schedule’s convention.
- The compiler MUST either:
  - (a) define a deterministic mapping rule, or
  - (b) reject schedules that cannot be represented at the timeline grain.

A schedule MUST NOT be finer than the master timeline. An interval of `week`
on a monthly calendar, or `month` on a quarterly one, is rejected
(`E2108_SCHEDULE_FINER_THAN_CALENDAR`) rather than silently collapsed: several
occurrences in one period cannot be distinguished once they land in the same
bucket. Coarser is fine — a quarterly schedule on a monthly grid pays in every
third period, which is representable.

**The grain rule.** `E2108` is not a limitation to be worked around; it is the
enforcement of the rule the language is built on:

> **Model at the finest grain at which anything varies; report at any coarser
> grain by folding.**

No granularity is imposed. A model may be annual, monthly or daily, and the
timeline it declares is the grain at which its expressions are evaluated —
`evaluate_stream` builds its environment from `timeline[idx]`, so every
occurrence inside one period sees an identical environment. A constant amount is
therefore exact; anything varying with `time.*` is computed once and multiplied.

That is precisely why a finer schedule is rejected. Take a monthly mortgage on
an annual grid. The level payment is constant, so twelve identical evaluations
sum to the right annual figure — but decomposed into interest and principal,
interest is `balance x rate/12` against a balance that falls every month, and
all twelve evaluations see the same `time.t`. Interest is overstated and
principal understated while still summing to a total that looks correct. Wrong
at every line beneath the one line that looks right.

If interest varies monthly, the model is monthly. Reporting it annually is then
a regrouping of the same ledger and costs nothing: a statement, a rollup and a
valuation each name the grain they report at, and many coexist in one run. See
`docs/06_results_schema.md` for `StatementGrain`.

An earlier proposal took the opposite trade — retire `E2108` and add a
sub-period occurrence layer so a model could compute finer than its grid. It was
**rejected**; the reasoning is recorded in `docs/15_streams_and_the_grid.md`.

Recommended v0.1 rule (normative):
- If timeline is monthly/quarterly/annual, schedule occurrences are represented at that period’s end-date unless the schedule specifies otherwise.

---

## 12. Assumptions (deterministic and stochastic)

### 12.1 Deterministic assumption
```cfdl
assume base_rent = 4000
```

A deterministic assumption is a named value the model owns. Terms and
expressions read it as `inputs.<name>`, and a scenario overrides it by the same
name, which makes `assume` the model's single channel for variation.

Discounting is not an assumption. The valuation rate belongs to the run, so one
set of cash flows can be valued at several rates without editing the model. It
is `annual_discount_rate` in the run configuration; see
[Language guide](/docs/language-guide). An `assume` of that name is an ordinary assumption and
does not move `model.npv`.

### 12.2 Stochastic assumption (distribution)
```cfdl
assume rent_growth ~ Normal(mean=0.03, stdev=0.01, clip=[-0.02, 0.08])
```

Supported distributions (v0.1 core):
- `Normal(mean, stdev, clip?)`
- `LogNormal(mu, sigma, clip?)`
- `Uniform(min, max)`
- `Triangular(min, mode, max)`

### 12.3 Where distributions may appear
- Distributions MUST be declared via `assume <name> ~ Dist(...)`.
- Any term or expression can reference stochastic values via `inputs.<name>`.

### 12.4 Reproducibility (normative)
- Any Monte Carlo run MUST declare an explicit seed.
- Engines MUST use this seed deterministically for sampling.

### 12.5 Curves (date-indexed inputs)

Named date-indexed value series (forward rate curves, price decks):

```cfdl
curve sofr {
  2026-01: 0.048
  2026-07: 0.045
  2027-01: 0.042
}

curve power_price linear {
  2026-01: 42.0
  2036-01: 55.0
}
```

- Interpolation is `step` (default; flat-forward: the last point at or
  before the query date, the first value before the first point) or
  `linear` (linear in calendar days between bracketing points, clamped
  flat outside the range).
- Expressions look curves up with `curve_value("<name>", <date>)`, e.g.
  `curve_value("sofr", time.date) + margin`.
- Curve names MUST be unique; a curve MUST declare at least one point and
  at most one value per date.

### 12.6 Quantiles (share-indexed inputs)

A `curve` is indexed by **when**. A `quantile` is indexed by **how much** — a
named series of values against cumulative share, for a quantity whose
*dispersion* drives the answer rather than its level:

```cfdl
quantile ercot_north linear by exceedance ref energy.power_price {
  1.00: 512.0
  0.98: 340.0
  0.50:  28.0
  0.00:  11.0
}
```

- Shares MUST lie in `0..1`. The physical measure — hours in the year, pool
  balance, rentable area — belongs to whatever reads the quantile, so one
  declaration serves assets of different size.
- Values MUST be non-decreasing in share. A quantile function that fell would
  leave `quantile_of` without a single answer.
- Interpolation is `step` (the last point at or below the query share) or
  `linear`, with the same meanings they carry on a curve. It also fixes
  quadrature: `quantile_mean` is the **exact integral** of the function these
  describe, so no separate quadrature mode is declared and none is needed.
- `by exceedance` writes the points worst-first, as a duration curve reads.
  It is authoring surface only: the compiler normalises to one ascending form,
  so the IR carries no orientation. `by quantile` is the default.
- `ref <id>` names the pack `[[references]]` entry this realises, which is what
  supplies its unit.
- Names MUST be unique and a quantile MUST declare at least one point
  (`E5028_INVALID_QUANTILE`).

Expressions read one with `quantile_at`, `quantile_mean` and `quantile_of`
(§16.2).

A quantile is **univariate**. A joint declaration over two quantities would be
a correlation, which §1.1.10 and §17.4 exclude from the core language and from
the IR. It is also never sampled: it is declared data, and uncertainty *about*
one is an ordinary `assume` scaling it.

---

## 13. Events (discrete model changes)

### 13.1 Event declaration
Syntax:

```cfdl
event refi_if_rates_drop when curve_value("sofr", time.date) < 0.045 {
  set entity loan.senior.status = "refinanced"
  deactivate stream loan.debt_service
}
```

Rules:
- Events are evaluated **discretely** at each time step of the master timeline.
- `when` MUST be a boolean expression.
- **Events LATCH.** An event fires at most once per run, at the first period its
  condition holds. It does not re-fire if the condition becomes true again, and
  there is no repeating or level-triggered form. ("Evaluated discretely at each
  time step" describes when the condition is TESTED, not how often the event may
  fire — a distinction this spec previously left to be discovered.)
- **Declaration order decides which event fires first** within a period, and
  which write wins when two events set the same field.
- **A guard reads the state as the period OPENED.** Every event and option in a
  period evaluates against the same frozen pre-state; writes accumulate and are
  visible at `t+1`. A STREAM, by contrast, reads the state as the period CLOSED,
  so a transition takes effect in the period it fires. That is the synchronous
  discipline: transitions all evaluate against the current state, the state
  commits, then outputs read the committed result. It is what keeps declaration
  order from changing the value of a guard.
- A guard may read any entity field by qualified path — `asset.tower.status`,
  `asset.pool.factor` — since an event has no owner of its own. It may NOT read a
  stream.
- Every write is published in `deterministic.transitions` — period, entity,
  field, from, to, and the firing event — so a transition is assertable.

### 13.2 Actions (v0.1 core)
Supported actions:
- `set entity <EntityRef>.<field> = <value>`
- `activate stream <StreamName>`
- `deactivate stream <StreamName>`
- `activate contract <ContractName>` (optional)
- `deactivate contract <ContractName>` (optional)
- `exercise option <OptionName>`

### 13.3 Event timing and the grid (normative)

An event fires in the **first period whose condition holds**, evaluated once per
period against the state as that period opened. It cannot fire between periods.

The model calendar therefore bounds how precisely a condition can be met: a
condition that becomes true partway through a period takes effect at that
period's boundary, not when it became true. Where an event determines an
allocation, the calendar is a term of the model and not a presentation choice.

### 13.4 Entity state in an activation guard
A stream is active on its effective dates: its schedule is what brings it into
being, and §9.3 is normative on that — a guard is optional and its absence means
active for every scheduled occurrence.

Where a stream's activity depends on something the model tracks rather than on
the calendar, entity state is what a guard reads:

```cfdl
active when entity.status != "refinanced"
```

This is allowable, not required, and not a substitute for the effective dates.
A contract takes no such guard. A contract records what was agreed and its
`term` states when its obligations run; whether a right or an obligation is
exercised is a modeling decision, carried by an event, an option, or the entity
state those write — never by the record of the agreement.

---

## 14. Options (real options, minimal v0.1)

### 14.1 Option declaration
Syntax:

```cfdl
option refi_1 type Option.Refinance exercisable in construction {
  exercise when curve_value("sofr", time.date) < 0.045
  payoff cfg.refi_savings_estimate - 250000
}
```

Rules:
- Options MAY be activated/deactivated via events.
- v0.1 supports only deterministic exercise triggers.
- Optimization/search policies are out of scope for v0.1.

---

## 15. Runs

### 15.1 Run declarations
```cfdl
run deterministic
run monte_carlo trials 20000 seed 42
```

Rules:
- `monte_carlo` MUST provide `trials` and `seed`.

### 15.2 Engine-computed outputs
Output metrics (NPV, IRR, DSCR, NOI, etc.) are computed by the engine based on the domain pack's output specification. CFDL models do not declare output metrics.

See the Pack Interface specification for details on how packs define output categories, aggregations, and metrics.

---

## 16. Expressions (CFDL expression language)

### 16.1 Expression syntax
Expressions are written as:

```cfdl
<expression>
```

Expressions MUST be:
- side-effect free
- deterministic given the same inputs
- terminating

### 16.2 Namespaces and built-ins (normative)
The expression environment MUST support:

**Model/time**
- `time.t` (0-based period index), `time.date`, `time.phase`

**Inputs**
- `inputs.<name>` for assumptions (fixed or stochastic)
- `cfg.<name>` for run-config values

**Entities**
- `entity.<field>` on the owning entity, and `<family>.<entity>.<field>` from
  anywhere — entity fields and lifecycle state (an entity with a lifecycle
  opens in its declared initial state; other event-set fields are null before
  first set)

**Observables and curves**
- `obs.<name>` — externally supplied observable values (provided via
  run-config parameters with the `obs.` key prefix)
- `curve_value(<name>, <date>)` — lookup into a declared `curve`
- `quantile_at(<name>, <share>)`, `quantile_mean(<name>, <from>, <to>)`,
  `quantile_of(<name>, <value>)` — lookups into a declared `quantile`
- `ref.<name>` is reserved for ontology references (not in the v0.1 dialect)

**Cross-stream series**
- `series_sum(<pattern>, <window>)` / `series_avg(...)` — cross-stream
  references (dependency-ordered waves; cycles rejected)

**Math and finance**
- Standard arithmetic `+ - * / ^`, comparisons, `and/or/not`, `if(cond, a, b)`
- `min/max/sum/avg/abs/round/round_down/round_up/clamp/pow`
- `pmt/ipmt/ppmt/rate/nper/pv/fv` (Excel sign conventions; `npv` and `irr` are
  *metrics* computed over results, not expression functions)
- `year_frac/eomonth/edate/date/parse_date/months_between`
- `is_business_day/roll/add_business_days` with named holiday calendars
- `macrs_rate`, `cpr_to_smm`
- `curve_value`, `series_sum`, `series_avg`
- `quantile_at`, `quantile_mean`, `quantile_of` (lookups into a declared
  `quantile`; see §12.6)
- The authoritative function catalog is `03_expression_environment.md`

### 16.3 Currency literals
The language MAY support syntactic sugar:
- `42000 USD` as Money
- `10%` as Rate

These MUST compile to typed values in IR.

---

## 17. Canonical JSON IR requirements (high level)

### 17.1 Single IR
- The compiler MUST emit one canonical JSON IR.

### 17.2 Preserve provenance
- The IR MUST preserve:
  - all entities
  - all contracts (including terms)
  - all streams (explicit and derived)
  - provenance links from derived artifacts back to their contract source

### 17.3 Required external inputs
- The IR MUST include lists of required inputs inferred from expressions:
  - `required_observables`: list of ontology observable ids referenced via `obs.*()` calls
  - `required_refs`: list of ontology ref ids referenced via `ref.*`

### 17.4 No correlation
- The IR MUST NOT contain any correlation field/slot.

---

## 18. Reserved keywords (v0.1)

A reserved word cannot be used as an identifier. The list is exhaustive and is
checked against the lexer, so a word added to one appears in the other.

### 18.1 In use (83)

Read by a production of the grammar:

`activate`, `active`, `also`, `annual`, `as`, `assume`, `calendar`, `clip`, `contract`, `convention`, `currency`,
`curve`, `daily`, `day`, `days`, `deactivate`, `deterministic`, `effects`, `end`, `entity`, `eom`, `event`,
`every`, `except`, `exercisable`, `exercise`, `false`, `following`, `for`, `from`, `import`, `in`, `inflow`,
`LogNormal`, `mid`, `model`, `modified_following`, `modified_preceding`, `monte_carlo`, `month`, `monthly`, `months`, `net`, `none`,
`Normal`, `on`, `option`, `outflow`, `pack`, `parties`, `payment`, `payoff`, `phase`, `phase_end`, `phase_enter`,
`phase_start`, `preceding`, `quantile`, `quarter`, `quarterly`, `run`, `schedule`, `seed`, `set`, `state`, `start`, `stream`, `stub`,
`term`, `terms`, `time`, `to`, `trials`, `Triangular`, `true`, `type`, `Uniform`, `use`, `version`,
`waterfall`, `week`, `when`, `year`.

### 18.2 Reserved, read by no production (14)

Reserved so that adding the feature later does not break a model that had used
the word as an identifier. Writing one today is an error, and no syntax accepts
it:

`direction`, `Fri`, `long_back`, `long_front`, `Mon`, `owner`, `Sat`, `short_back`, `short_front`, `Sun`, `tags`,
`Thu`, `Tue`, `Wed`.

`Mon` through `Sun` anchor a weekly schedule to a weekday. That syntax is not
implemented: `on` accepts `day <n>` or `eom`, and `weekly` is not a calendar
frequency — the frequencies are `daily`, `monthly`, `quarterly` and `annual`.
`direction` and `owner` name parts of a stream header the parser reads
positionally. `tags` and the four stub conventions belong to features that are
specified and not yet built.

`pay` is contextual — it introduces a waterfall step and is an ordinary
identifier elsewhere. `remaining`, `paid` and `owed` are bindings the host
provides inside a step expression (§10.3), not keywords.

---

## 19. Minimal multi-file example (Core)

This example compiles and runs against the `cre` pack as written.

**model.cfdl**
```cfdl
version 0.1
model "sunset-apartments"
use pack "cre" version "0.1.0"

import "time.cfdl"
import "structure.cfdl"
import "assumptions.cfdl"
import "behavior.cfdl"
import "runs.cfdl"
```

**time.cfdl**
```cfdl
time calendar monthly from 2026-01 for 72
phase construction from 2026-01 to 2026-12
phase operations from 2027-01 to 2031-12
```

**structure.cfdl**
```cfdl
entity asset sunset
entity loan senior
```

**assumptions.cfdl**
```cfdl
assume base_rent = 42000
assume rent_growth ~ Normal(mean=0.03, stdev=0.01, clip=[-0.02, 0.08])

curve sofr linear {
  2026-01: 0.050
  2028-01: 0.038
}
```

**behavior.cfdl**
```cfdl
contract cre.lease on entity asset.sunset {
  term 2027-01..2031-12
  terms {
    base_rent = inputs.base_rent
  }
}

stream loan.debt_service on entity loan.senior outflow currency USD {
  active when entity.status != "refinanced"
  schedule every month from 2026-01 to 2031-12
  amount = -pmt(0.06 / 12, 72, 8500000)
}

event refi_if_rates_drop when curve_value("sofr", time.date) < 0.045 {
  set entity loan.senior.status = "refinanced"
  deactivate stream loan.debt_service
}
```

**runs.cfdl**
```cfdl
run deterministic
run monte_carlo trials 20000 seed 42
```

---

## 20. Conformance
An implementation conforms to CFDL v0.1 Core if it:
1. Parses valid CFDL programs per this spec.
2. Rejects invalid programs with actionable diagnostics.
3. Validates strong types and required fields.
4. Emits deterministic canonical IR that preserves contracts/streams/provenance.
5. Supports the schedule primitives and discrete event semantics.
6. Supports CFDL-native expressions and the required namespaces/functions.
