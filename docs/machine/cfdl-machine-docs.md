<!-- GENERATED machine docs bundle by tools/gen-machine-docs.py — do not edit by hand. Regenerate: make machine-docs -->

# CFDL machine documentation bundle — v0.7.0

> CFDL (Cash Flow Domain Language) is a declarative language for financial
> cash-flow models with a deterministic compile -> run cycle, structured
> diagnostics, and an externally-benchmarked engine. Contracts lower to
> streams: a `contract` declaration is vocabulary that a pack's lowering
> rule turns into streams; streams are the cash. Streams evaluate first on
> their schedules, then logic and events act on those amounts, then
> financing and distributions run over the aggregated flows.

One document, generated from the normative sources the site renders.
Deduplicated: the course at learn.cfdl.dev restates this material
pedagogically and is deliberately not here (see llms-full.txt).

---

# Language specification

# CFDL v0.1 Core Language Specification

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

### 7.3 The lifecycle machine

A lifecycle is a finite state machine, and the machine is a **core-language
construct**: a model declares one with no pack at all, and a pack declares
the same machine for its domain types in `types.toml` — the core has the
full functionality, and packs tailor it to domains.

```cfdl
lifecycle unit {
  initial vacant
  state vacant, leased, downtime

  vacant   -> leased    when time.t >= 1
  leased   -> leased    when time.t == 6
  leased   -> downtime  when series_sum("core.rent", time.t - 1, time.t - 1) < 50
  downtime -> leased
}
```

An entity binds a model-declared machine by name — `lifecycle unit` in its
block — and an entity whose ontology type declares a lifecycle needs no
binding (declaring both is an error: one machine per entity). `lifecycle`
and `initial` are contextual identifiers, not reserved words.

Rules:
- **States are enumerated ahead of time; edges are not.** The finite set is
  what makes the machine checkable — a misspelled state in an edge is a
  compile error naming the declared set, not a phantom state. Declaring an
  edge is what brings it into existence; an undeclared edge does not exist,
  and absence is the prohibition. A partial machine is a complete machine.
- **Guards live on edges** and are evaluated each period the entity is in
  the edge's from-state — and only then. A guard reads state as the period
  opened and series strictly backward (at or before the previous period). A
  time condition is the same construct with a `time` guard.
- **There is no latch.** Edge availability is the memory: taking an edge
  moves the machine and disarms it, and re-entry re-arms it. A covenant
  that breaches and cures is the topology walked twice.
- **A well-formed machine resolves.** A self-edge (`leased -> leased`) is a
  real transition — it journals and re-anchors a state-anchored schedule
  (§11.7). No guard true means the entity holds. When two guards hold in
  one period, declaration order picks, and at most one transition per
  entity per period is taken.
- **A guard-less edge is a permission only**: an event's write may take it,
  and the machine never fires it on its own. An event's `set … status` is
  validated against the declared relation — refused with the edge named
  where no edge permits the move — and a machine that declares no edges at
  all stays unconstrained, which is what every shipped pack machine was.
- **Every transition is journaled** with the edge taken and the values its
  guard read, and published in `deterministic.transitions` as
  `lifecycle:<id>`.

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

### 9.5 Priced amounts (the valuation exception)

An `amount` whose series window reaches the current period or beyond is a
**priced amount**: a valuation setting a causal amount. The forward-income
exit is the canonical case — the sale is a causal event, the receipt is
causal cash, and only the amount is a valuation, forward income against a
cap rate:

```cfdl
stream cre.exit on entity asset.project inflow currency USD {
  schedule on 2037-06 end
  category investing.reversion
  amount = series_sum("cre.noi", time.t + 1, time.t + 12) / inputs.cap_rate
}
```

- A priced amount is evaluated after every causal cell settles; streams that
  read it are priced with it, and a waterfall distributing at the sale
  period allocates proceeds that exist.
- **The graph must stay acyclic**: logic reading what a priced amount sets —
  sale proceeds feeding state that feeds what is being capitalized — is
  refused with the path named, like any other cycle.
- A forward window in a GUARD is not priced. Whether a stream is active is a
  causal fact, and a fact cannot be read from the future.

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

The schedule stays sovereign: a waterfall distributes only at the periods its
schedule names. On every other period its cash accumulates — with the entity,
or in a declared account (§10.6) — and a quarterly or at-exit waterfall
distributes the accumulated position when its date arrives.

### 10.5 Output and composition
Each step publishes as a series named `stream.<waterfall>.<step>`, and its cash
counts toward the payee's total. A waterfall is not a separate kind of output:
statements, metrics and the results document read it as they read any stream.

Because steps publish as series, a waterfall MAY draw on the output of a
waterfall **declared before it** — a fund's carry becoming a management
company's pot, and that company's own share becoming a third waterfall's.
Composition follows declaration order, the same rule steps follow within a
waterfall.

### 10.6 The account

Carried cash gets the industry's own object: a declared cash location whose
balance accumulates across periods — a collection account, a reserve, a
participant's distribution account.

```cfdl
account reserve {
  owner party.sponsor
  from series_sum("ops.net", time.t, time.t)
}
```

- `owner` is optional: a general account belongs to the structure, and a
  party-owned account holds what has been **allocated** to that party. A
  party owns at most one account. A party-owned balance is not an
  obligation — what is still owed under the rules is not in any account, it
  is simply not yet allocated.
- `from` is the per-period inflow, reading cash that has settled this
  period. It MAY be negative, and **the balance has no floor**: an account
  fed a deal's whole net cash IS the deal's cumulative position, negative
  through the J-curve and positive after.
- There is no currency clause: an account is denominated by the model.

The balance law, applied at each period:

```
balance(t) = balance(t-1) + inflow(t) + allocated_in(t) - allocated_out(t)
```

Three uses:
- **A waterfall draws from an account** — `from <account>` in place of a
  hand-written cumulative window. The pot is the accumulated balance,
  floored at zero (cash that is not there cannot be allocated); what its
  steps take leaves the balance, and residue stays for the next scheduled
  date.
- **A step pays to an account** — `pay <step> to account <name> = <expr>`,
  the reserve pattern's mechanism (fund to target, top up when short,
  release when over). `pay <step> to <party>` lands in that party's account
  when they own one, and behaves exactly as before when they do not.
- **Logic reads a balance** — `prev.<account>` is settled state, strictly
  backward: the balance at the previous period, every allocation through it
  included. At period 0 there is no binding: before the model began is not
  zero, it is unavailable.

A step's series is the FLOW and the account's balance is the POSITION. The
balance publishes as a non-cash series under `account.<name>`, never enters
cash totals, and every movement — inflow, allocation in, allocation out —
is journaled with the balance before and after.

`available` is unchanged and still means this period's netted cash; an
account is the ACCUMULATED cash. The two answer different questions, and a
monthly-distributing waterfall keeps using `available` untouched.

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

### 11.4 State-anchored windows

The third anchor, beside dates and phases: a state entry.

```cfdl
schedule every month from state_enter(asset.site, building) for 18 periods
```

Each ENTRY of the entity into the state opens its own window of `n` grid
periods, resolved during the walk — the entry period is settled state by
the time any stream reads it. This is what "18 months of construction from
whenever construction starts" needs: the machine enters the state whenever
its edge fires, the schedule hangs off the entry, and the activity window
carves itself out of the grid.

- **A re-entered state re-anchors**: each entry starts its own window — a
  second delinquency's cure period, a renewal restarting its term. A
  self-edge is an entry.
- Within a window the schedule behaves exactly as
  `every <interval> from <entry> to <window end>`: intervals, placement,
  day rules and payment terms mean what they mean everywhere else.
- The anchor's entity must have a machine and the state must be declared —
  the same finite-set discipline every other state reference has.
  `state_enter` and `periods` are contextual identifiers.
- Truly linear time keeps its constructs: calendar-fixed eras are phases,
  condition-driven regimes are machine states, and this anchor belongs to
  the second.

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
`docs/09_user_guide.md`. An `assume` of that name is an ordinary assumption and
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
- **The machine does not latch — and needs no policy saying so.** A
  condition that holds and clears and holds again is a lifecycle edge
  (§7.3), where edge availability is the memory. The free-standing event
  keeps its latched meaning; a regime that returns is the machine's job.
- **A status write is validated against the machine.** `set … status` on an
  entity whose lifecycle declares edges is refused — with the edge named —
  where no declared edge permits the move; the refusal is journaled as
  `declined`, and an edge-less machine stays unconstrained.
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

A `<StreamName>` is any stream the model runs: one the model declared, or one a
contract lowered (`cre.lease.base_rent` is §9.1's own example, and `docs/07`
§6.4 gives the identical string as an example of a generated name). Reaching
for a contract does not cost the ability to stop its cash. A name matching
neither is `E1302`.
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

### 18.1 In use (85)

Read by a production of the grammar:

`account`, `activate`, `active`, `also`, `annual`, `as`, `assume`, `calendar`, `clip`, `contract`, `convention`, `currency`,
`curve`, `daily`, `day`, `days`, `deactivate`, `deterministic`, `effects`, `end`, `entity`, `eom`, `event`,
`every`, `except`, `exercisable`, `exercise`, `false`, `following`, `for`, `from`, `import`, `in`, `inflow`,
`LogNormal`, `mid`, `model`, `modified_following`, `modified_preceding`, `monte_carlo`, `month`, `monthly`, `months`, `net`, `none`,
`Normal`, `on`, `option`, `owner`, `outflow`, `pack`, `parties`, `payment`, `payoff`, `phase`, `phase_end`, `phase_enter`,
`phase_start`, `preceding`, `quantile`, `quarter`, `quarterly`, `run`, `schedule`, `seed`, `set`, `state`, `start`, `stream`, `stub`,
`term`, `terms`, `time`, `to`, `trials`, `Triangular`, `true`, `type`, `Uniform`, `use`, `version`,
`waterfall`, `week`, `when`, `year`.

### 18.2 Reserved, read by no production (13)

Reserved so that adding the feature later does not break a model that had used
the word as an identifier. Writing one today is an error, and no syntax accepts
it:

`direction`, `Fri`, `long_back`, `long_front`, `Mon`, `Sat`, `short_back`, `short_front`, `Sun`, `tags`,
`Thu`, `Tue`, `Wed`.

`Mon` through `Sun` anchor a weekly schedule to a weekday. That syntax is not
implemented: `on` accepts `day <n>` or `eom`, and `weekly` is not a calendar
frequency — the frequencies are `daily`, `monthly`, `quarterly` and `annual`.
`direction` and `owner` name parts of a stream header the parser reads
positionally. `tags` and the four stub conventions belong to features that are
specified and not yet built.

`pay` is contextual — it introduces a waterfall step and is an ordinary
identifier elsewhere, and so are `lifecycle`, `initial` (§7.3),
`state_enter` and `periods` (§11.4): position disambiguates each, and no
model loses an identifier to them. `remaining`, `paid` and `owed` are
bindings the host provides inside a step expression (§10.3), not keywords,
and `available` and `prev.<account>` (§10.6) are the same kind of thing.

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


---

# Grammar (EBNF)

_The complete grammar. docs/02_grammar.md on the site is a pointer to this file._

```ebnf
(* CFDL v0.1 — Formal Grammar (EBNF) *)
(* See 02_grammar.md for lexical tokens and implementation guidance. *)

module          = { statement } ;

statement       = version_stmt
                | model_stmt
                | use_pack_stmt
                | import_stmt
                | time_stmt
                | phase_stmt
                | entity_stmt
                | assume_stmt
                | curve_stmt
                | quantile_stmt
                | contract_stmt
                | stream_stmt
                | account_stmt
                | waterfall_stmt
                | lifecycle_stmt
                | event_stmt
                | option_stmt
                | run_stmt
                ;

(* --- header / modules --- *)
version_stmt    = "version" number ;

model_stmt      = "model" string_lit model_attr* ;
model_attr      = "currency" IDENT ;   (* reporting currency; defaults to USD *)

use_pack_stmt   = "use" "pack" string_lit "version" string_lit ;

import_stmt     = "import" string_lit [ "as" IDENT ] ;

(* --- time --- *)
time_stmt       = "time" "calendar" cadence "from" date_lit "for" INT [ "project" INT ] ;

(* The grid a model is evaluated on. Adjectival, and deliberately spelled
   differently from a schedule interval. *)
cadence         = "daily" | "monthly" | "quarterly" | "annual" ;

phase_stmt      = "phase" IDENT "from" date_lit "to" date_lit ;

(* --- entities --- *)
(* Both the type annotation and the block are optional. `entity asset sunset`
   is a complete statement, and is the form §7 of the language specification
   opens with. *)
entity_stmt     = "entity" IDENT IDENT [ ":" qname ] [ entity_block ] ;
entity_block    = "{" { entity_item } "}" ;

entity_item     = entity_field
                | part_of_stmt
                | entity_state_stmt
                | entity_lifecycle
                ;

(* Where this entity sits in the hierarchy. Optional by design: the same
   building may be modeled as one asset, as unit types, or as suites. *)
part_of_stmt    = "part" "of" entity_ref ;

(* The lifecycle state the entity OPENS in. Unrelated to a field. *)
entity_state_stmt = "state" IDENT ;

(* Bind a model-declared machine to this entity. `lifecycle` is a
   contextual identifier, not a reserved word; the following IDENT is what
   disambiguates it from a field of that name. An entity whose ontology
   type declares a lifecycle needs none. *)
entity_lifecycle = "lifecycle" IDENT ;

(* --- assumptions --- *)
assume_stmt     = "assume" IDENT ( "=" expr | "~" dist_expr ) ;

(* --- fields: a value the entity has in every period ---
   `use = "office"` states a fact and holds. `balance init <expr> next <expr>`
   gives it a rule, with `prev` bound to the field's own previous value; an
   absent `next` means it holds. Read as `asset.<name>.<field>`, and
   `prev.asset.<name>.<field>` for the close before this one.

   `state <name>` inside an entity block is unrelated: it names the lifecycle
   state the entity opens in. --- *)
entity_field    = IDENT ( "=" expr | "init" expr [ "next" expr ] ) ;
(* --- curves: named date-indexed values, looked up via curve_value() --- *)
curve_stmt      = "curve" IDENT [ curve_interp ] "{" curve_point { [ "," ] curve_point } "}" ;
curve_interp    = "step" | "linear" ;
curve_point     = DATE ":" [ "-" ] NUMBER ;

(* --- quantiles: values indexed by CUMULATIVE SHARE rather than by date, read
   with quantile_at() / quantile_mean() / quantile_of().

   `step`, `linear`, `by` and `ref` are contextual identifiers, not keywords,
   exactly as a curve's interpolation mode is — the construct reserves one new
   word and no more.

   `by exceedance` writes the points worst-first, the way a duration curve
   reads. It is surface only: the compiler reverses them into the single
   canonical ascending form, so the IR carries no orientation. --- *)
quantile_stmt   = "quantile" IDENT [ quantile_interp ] [ quantile_order ]
                  [ "ref" qname ]
                  "{" quantile_point { [ "," ] quantile_point } "}" ;
quantile_interp = "step" | "linear" ;
quantile_order  = "by" ( "quantile" | "exceedance" ) ;
quantile_point  = NUMBER ":" [ "-" ] NUMBER ;

dist_expr       = dist_name "(" [ dist_arg { "," dist_arg } ] ")" ;
dist_name       = "Normal" | "LogNormal" | "Uniform" | "Triangular" ;
dist_arg        = IDENT "=" literal
                | "clip" "=" list_number
                ;

(* --- contracts --- *)
(* TWO qnames by design: the contract TYPE, which a pack declares, and the
   INSTANCE NAME. The implementation accepts only one, with the instance fused
   onto the type as a dotted suffix (`cre.opex_line.property_tax`), and that
   restriction is the root of a family of defects — the type must then be
   recovered by string surgery, which is what `dot_suffix`, the deleted
   `ContractMatch` mode, and `tools/check-pack-series.py` all exist to cope
   with. See backlog 7.58 and 7.63; the design is right and the implementation
   should meet it.

   `on entity` is optional in practice — 134 of this repository's 520 contracts
   omit it and fall back to the model's sole asset.

   `term` is a contract ITEM, inside the block. That is §8.1 of the language
   specification, which is normative and which this grammar contradicted. *)
contract_stmt   = "contract" qname [ qname ] [ "on" "entity" entity_ref ]
                  contract_block ;

contract_block  = "{" { contract_item } "}" ;

(* Payment terms: how long after a flow is earned its cash moves. Applies to
   every stream the contract lowers; a schedule may state its own. Bare counts
   are days, the commercial default — "net 45" is 45 days. Months exist because
   some lags are genuinely month-based and diverge from any day count once
   billing is not at a month end. *)
payment_stmt    = "payment" "net" INT [ "days" | "months" ] ;

contract_item   = term_stmt
                | payment_stmt
                | currency_stmt
                | terms_block
                | effects_block
                | parties_block
                ;

term_stmt       = "term" date_lit ".." date_lit ;

currency_stmt   = "currency" IDENT ;

terms_block     = "terms" map_block ;
parties_block   = "parties" map_block ;
(* NOT IMPLEMENTED. No production reads `tags`; §18.2 lists it as reserved. *)
tags_block      = "tags" map_block ;

map_block       = "{" { map_entry } "}" ;
(* A term may be an expression. Restricting it to a literal was a restriction
   of the IMPLEMENTATION, not of the design: what a contract records is often
   itself an expression — a rent escalating at "CPI plus 50 basis points", a fee
   of "3% of effective gross income", a coupon of "SOFR + 225bp". Forcing those
   through a model-level `assume` moves the agreement out of the contract and
   leaves a reference behind. See backlog 7.64. *)
map_entry       = qname literal_or_expr ;

(* --- effects (streams in v0.1 core) --- *)
effects_block   = "effects" "{" { effect_stmt } "}" ;
effect_stmt     = stream_effect_stmt ;

(* NOT IMPLEMENTED. An `effects` block parses and must currently be empty; no
   production reads `owner` or `direction`, which is why §18.2 of the language
   specification lists both as reserved for a feature not yet built. A stream
   inside a contract is written today as a lowering rule in a pack. *)
stream_effect_stmt = "stream" qname
                     "owner" "entity" entity_ref
                     "direction" direction
                     "currency" IDENT
                     stream_block ;

direction       = "inflow" | "outflow" ;

(* --- standalone streams --- *)
stream_stmt     = "stream" qname
                  "on" "entity" entity_ref
                  direction
                  "currency" IDENT
                  stream_block ;

stream_block    = "{" { stream_item } "}" ;
stream_item     = schedule_stmt
                | amount_stmt
                | active_stmt
                ;

active_stmt     = "active" "when" expr ;
amount_stmt     = "amount" expr ;

(* --- waterfalls: an ordered priority of payments over a pot ---
   `from` is the pot; each `pay` step takes min(max(0, expr), remaining) and
   what survives passes down. Steps are paid in DECLARATION ORDER, and a step
   may read the steps above it as `paid.<step>` and `owed.<step>` — what an
   earlier step actually paid, and what it would have paid unbounded. Their
   difference is that step's shortfall.

   At least one step MUST read `remaining`, so the residual has a named payee
   rather than vanishing.

   Steps publish as series `stream.<waterfall>.<step>`, so a waterfall declared
   later may draw on an earlier one's output as its own pot. Composition
   follows declaration order, the same rule steps follow inside a waterfall.

   A waterfall runs after the period's fields and streams, so it allocates cash
   that already exists and never feeds a stream in the same period.
   See docs/17_ordered_waterfall.md. --- *)
waterfall_stmt  = "waterfall" qname
                  "on" "entity" entity_ref
                  waterfall_block ;

waterfall_block = "{" schedule_stmt "from" expr { waterfall_step } "}" ;

(* --- accounts ---
   A declared cash location whose balance carries ACROSS periods. `available`
   is unchanged and still means this period's netted cash; an account is the
   accumulated cash, and is what a waterfall draws its pot from when its
   schedule names a period.

   An account is denominated by the MODEL, so there is no currency clause: the
   only thing it could express is a restatement of what `model_stmt` already
   declares, or a second unit that must be refused.

   `owner` is optional. A general account belongs to the structure — a
   collection account, a reserve — and a party-owned one holds what has been
   ALLOCATED to that party. It is not an obligation: it holds cash that
   exists, and what is still owed is simply not yet allocated.
   See docs/28_period_walk.md §5.1. --- *)
(* The declared finite state machine (docs/28 §6.1). `lifecycle` and
   `initial` are contextual identifiers, not reserved words. The STATES are
   enumerated ahead of time — the finite set is what makes a misspelled
   state in an edge a compile error rather than a phantom state. The EDGES
   are declared only as used: declaring one brings it into existence, an
   undeclared edge does not exist, and absence is the prohibition. A
   self-edge (from = to) is a real transition: it journals and re-anchors.
   A guard-less edge is a permission only — an event's write may take it,
   but the machine never fires it on its own. A guarded edge is evaluated
   each period the entity is in its from-state; there is no latch, because
   edge availability is the memory. *)
lifecycle_stmt  = "lifecycle" IDENT "{" { lifecycle_item } "}" ;
lifecycle_item  = "initial" IDENT
                | "state" IDENT { "," IDENT }
                | lifecycle_edge
                ;
lifecycle_edge  = IDENT "->" IDENT [ "when" expr ] ;

account_stmt    = "account" IDENT account_block ;

account_block   = "{" { account_item } "}" ;
account_item    = account_owner | account_from ;
account_owner   = "owner" entity_ref ;
account_from    = "from" expr ;
waterfall_step  = "pay" IDENT "to" entity_ref "=" expr ;

(* --- schedules --- *)
schedule_stmt   = "schedule" schedule_expr ;

(* Where in its period a flow sits: one axis, three positions, at most one. *)
placement       = "start" | "mid" | "end" ;

schedule_expr   = schedule_on
                | schedule_every
                | schedule_phase_enter
                | schedule_every_phase
                | schedule_state_enter
                ;

schedule_on     = "on" date_lit [ placement ] ;

schedule_phase_enter = "on" "phase_enter" "(" string_lit ")" ;

schedule_every_phase = "every" interval [ placement ]
                       "from" "phase_start" "(" string_lit ")"
                       "to" "phase_end" "(" string_lit ")"
                       [ schedule_opts ] ;

(* The third anchor (docs/28 §6.2): each ENTRY of the entity into the state
   opens its own window of n grid periods, resolved during the walk; a
   re-entered state re-anchors, and a self-edge is an entry. `state_enter`
   and `periods` are contextual identifiers, not reserved words. *)
schedule_state_enter = "every" interval [ placement ] [ "net" INT [ "days" | "months" ] ]
                  "from" "state_enter" "(" entity_ref "," IDENT ")"
                  "for" INT "periods" ;

schedule_every  = "every" interval [ placement ] [ "net" INT [ "days" | "months" ] ]
                  [ schedule_on_day ]
                  "from" date_lit "to" date_lit
                  [ schedule_opts ] ;

(* An interval is how far apart a stream's payments fall. It is distinct from
   the calendar cadence, which is the grid the model is evaluated on: a stream
   may pay quarterly on a monthly grid. Only intervals have a weekly member —
   a weekly grid is not representable, a weekly schedule on a daily grid is. *)
interval        = "day" | "week" | "month" | "quarter" | "year" ;

(* `due` marks an annuity due: payment at the start of each interval, as for
   rent. Without it the schedule is an ordinary annuity, paid at the interval's
   end, which is right for coupons and debt service. See 12_payment_timing.md. *)

schedule_on_day = "on" ( "day" INT | "eom" ) ;

schedule_opts   = { schedule_opt } ;

schedule_opt    = "convention" convention
                | "calendar" string_lit
                | "except" list_date
                | "also" list_date
                ;

convention      = "none" | "following" | "modified_following" | "preceding" | "modified_preceding" ;

list_date       = "[" date_lit { "," date_lit } "]" ;

(* --- events --- *)
event_stmt      = "event" qname "when" expr event_block ;
event_block     = "{" { action_stmt } "}" ;

action_stmt     = set_entity_stmt
                | activate_stream_stmt
                | deactivate_stream_stmt
                | activate_contract_stmt
                | deactivate_contract_stmt
                | exercise_option_stmt
                ;

set_entity_stmt = "set" "entity" entity_ref "." IDENT "=" literal_or_expr ;

activate_stream_stmt   = "activate" "stream" qname ;
deactivate_stream_stmt = "deactivate" "stream" qname ;

activate_contract_stmt   = "activate" "contract" qname ;
deactivate_contract_stmt = "deactivate" "contract" qname ;

exercise_option_stmt   = "exercise" "option" qname ;

(* --- options --- *)
(* The parser ALSO accepts `on entity <ref>` after the name, and shipped models
   use it. Neither §14.1 of the language specification nor this grammar
   documents it — an extension that grew in the implementation. Resolve it in
   one direction or the other before v1; see backlog 7.62. *)
option_stmt     = "option" qname
                  "type" qname
                  [ "exercisable" "in" qname ]
                  option_block ;

option_block    = "{" option_item* "}" ;
option_item     = "exercise" "when" expr
                | "payoff" expr
                ;

(* --- runs --- *)
run_stmt        = "run" ( "deterministic" | mc_run ) ;
mc_run          = "monte_carlo" "trials" INT "seed" INT ;

(* --- expressions & literals --- *)
expr            = or_expr ;
or_expr         = and_expr { "or" and_expr } ;
and_expr        = not_expr { "and" not_expr } ;
not_expr        = { "not" } cmp_expr ;
cmp_expr        = add_expr [ ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) add_expr ] ;
add_expr        = mul_expr { ( "+" | "-" ) mul_expr } ;
mul_expr        = unary_expr { ( "*" | "/" | "%" ) unary_expr } ;
unary_expr      = [ "-" ] pow_expr ;
pow_expr        = primary [ "^" unary_expr ] ;  (* right-associative *)
primary         = number | bool_lit | string_lit | qname
                | IDENT "(" [ expr { "," expr } ] ")"
                | "(" expr ")" ;

literal_or_expr = literal | expr ;

literal         = string_lit | number | bool_lit | date_lit | money_lit | list | map_inline ;

money_lit       = number IDENT ;  (* e.g., 42000 USD *)

list            = "[" [ literal { "," literal } ] "]" ;

map_inline      = "{" [ qname literal_or_expr { qname literal_or_expr } ] "}" ;

list_number     = "[" number { "," number } "]" ;

entity_ref      = IDENT "." IDENT { "." IDENT } ;
qname           = IDENT { "." IDENT } ;

string_lit      = STRING ;
number          = DECIMAL | INT ;
bool_lit        = "true" | "false" ;
date_lit        = DATE ;
```


---

# Expression environment

# CFDL v0.1 — Expression Environment

Status: Normative for the CFDL expression language (implemented by `cfdl-calc`,
exposed through `cfdl-expr`).

CFDL expressions are bare, Excel-familiar formulas written directly in model
source:

```
amount = base_rent * (1 + escalation) ^ (time.t / 12)
active when time.t >= 6
```

They are deterministic and terminating by construction: no loops, no recursion,
no I/O, no user-defined functions. Every expression, on the same inputs, always
produces the same value.

## 1. Numeric semantics

Two evaluation modes exist; models always run in **decimal mode**.

- **Decimal mode (default).** All arithmetic is exact 128-bit decimal
  (`rust_decimal`, 28 significant digits). `0.1 + 0.2 == 0.3` is `true`.
  Float64 is used ONLY as a documented escape for transcendental operations:
  fractional exponents (`x ^ 0.5`), and iterative solvers (`rate`,
  `cpr_to_smm`). Integer exponents are decimal-exact.
- **excel_compat mode.** All arithmetic runs in IEEE-754 float64, reproducing
  Excel's representation artifacts (`0.1 + 0.2 - 0.3` yields ~5.55e-17, exactly
  as Excel does), for proving parity against Excel reference models and
  explaining decimal-vs-float differences.

  A run selects it with the `arithmetic` key in its run configuration —
  `"decimal"` (the default) or `"excel_compat"` — so a model CAN be run in it
  without touching Rust. The engine carries the choice into every expression
  environment, and `eval` is `eval_with_mode(compiled, env, env.mode)`, so the
  mode is honoured on every evaluation rather than being an unused escape.

  Whether that matters is measured rather than assumed:
  `excel_compat_stability` in `crates/cfdl-calc/src/lib.rs` runs the credit
  pack's arithmetic both ways and pins the divergence below 1e-12 — about ten
  orders of magnitude inside the tolerance of the benchmark it feeds. Decimal
  mode already routes fractional exponents through the f64 escape, so the two
  modes differ only where a model accumulates long sums or compares for
  equality.

Rounding: `round()` follows Excel semantics (half away from zero), not
banker's rounding. `round_down`/`round_up` truncate toward/away from zero.

## 2. Syntax

- Operators, by precedence (loosest to tightest):
  `or` < `and` < `not` < comparisons (`==`, `!=`, `<`, `<=`, `>`, `>=`) <
  `+ -` < `* / %` < unary `-` < `^` (right-associative).
- `=` and `<>` are accepted as Excel-style aliases for `==` and `!=` inside
  expressions.
- Literals: decimal numbers (`1200`, `0.05`, `1_000_000`), `true`/`false`,
  double-quoted strings.
- Variables are dotted paths resolved from the host environment (see §3).
- Function calls are lowercase snake_case: `pmt(0.005, 360, 100000)`.

## 3. Namespaces

The host (compiler or engine) provides values under these roots:

| Root | Contents |
|---|---|
| `model` | `model.id`, `model.base_currency` |
| `time` | `time.t` (0-based period index), `time.date`, `time.phase`, `time.ppy` (periods per year for the model's calendar), `time.days_in_period` |
| `entity` | fields of the stream's owning entity, and every entity's fields under its family — `entity.asset.tlb.balance` |
| `asset`, `party`, `contract`, `reference` | an entity's fields, spelled bare: `asset.tlb.balance` is the same read as `entity.asset.tlb.balance` |
| `cfg` | run-config values (scenario knobs) |
| `obs` | observations (rates, curves) supplied at run time |
| `inputs` | assumption values (`assume` statements), including ones derived from other assumptions (§2.1) |
| `prev` | a field's own previous value, bare — present inside that field's `next` only |
| `prev.<entity>.<field>` | a field one period back — `prev.asset.tlb.balance`, inside a rule |
| `prev.<account>` | an account's balance at the previous period, every allocation through it included — rules, guards, and step expressions; absent (not zero) at period 0 |
| `available` | this period's netted stream cash of the waterfall's entity — waterfall `from` and step expressions |
| `remaining` | what is left in the pot — present in waterfall step expressions only (§3.2) |
| `paid` | `paid.<step>`, what an earlier waterfall step actually paid — steps only |
| `owed` | `owed.<step>`, what an earlier step would have paid, unbounded — steps only |

Unknown variables are hard errors (`EXPR_EVAL`), not nulls.

### 2.1 Derived assumptions

An `assume` may read another through `inputs.<name>`:

```
assume gross_sf   = 10000.0
assume efficiency = 0.85
assume net_sf     = inputs.gross_sf * inputs.efficiency
```

Assumptions resolve in dependency order, so each one is evaluated after
everything it reads — declaration order and name order are both irrelevant.
Random assumptions (`assume ... ~ <dist>`) resolve first: a distribution's
central value reads nothing, so a derived assumption may be built on one.

A circular derivation is refused with the cycle named, the same way a circular
series read is (§3.1): no order satisfies it, and the engine does not iterate
toward a fixed point. A name that is not an assumption is not a dependency —
it comes from the run configuration, or from nowhere, and an unresolved name
is a hard error by the rule above.

`time.ppy` is how many periods of the model's calendar make a year — 365, 12,
4 or 1 — so a model can spread an annual figure without hardcoding a divisor
and without being rewritten when the calendar changes:

```
amount = inputs.rent_year / time.ppy
```

Domain packs do **not** use it. A lowering rule resolves its own
periods-per-year at compile time (`{{model.periods_per_year}}`, see
`docs/07_pack_interface.md`), because a rule may pay on its own interval: a
monthly-paying loan carried on a daily book divides by 12, not 365, and only
the compiler can see that. `time.ppy` reads the calendar and would say 365.

`time.days_in_period` is the actual calendar days the current period spans —
31 in January, 28 in a non-leap February, 1 on a daily grid. It is what makes
an Actual/360 or Actual/365 accrual expressible: `rate * time.days_in_period /
360`. Packs reach it through `{{model.accrual_divisor}}` rather than directly.

### 3.1 Fields that move: `<family>.<entity>.<field>` and `prev`

A field with a rule is a named number per period defined by a recurrence — the
one shape `pow(1 + r, t)` cannot express, since that applies a single period's
rate as though it had held from the start. It belongs to the entity it describes:

```cfdl
entity asset firm : Asset.Financial {
  revenue_index init 1.0
                next prev * (1 + curve_value("growth", time.date))
}

stream firm.revenue on entity asset.firm inflow currency USD {
  schedule every year from 2026-01 to 2035-01
  amount = 21765.4 * asset.firm.revenue_index
}
```

`init` is the value at period 0 and is **mandatory** — an unstated base case
would otherwise evaluate as a silent zero for every period, since an unmatched
lookup returns 0. `next` is the value at every later period.

Inside `next`, bare `prev` is this field's own previous value and
`prev.<family>.<entity>.<field>` is another field's. A rule may not read any
field at the current period, which is what keeps a cycle unexpressible:

| form | resolves to | present in |
|---|---|---|
| `<family>.<entity>.<field>` | that field at the **current** period | streams, waterfall steps, event guards |
| `prev` | this field at the **previous** period | `next` expressions |
| `prev.<family>.<entity>.<field>` | another field at the **previous** period | `next` expressions, streams |

`prev` accepts the entity-root spelling too — `prev.entity.asset.tlb.balance`
is the same read as `prev.asset.tlb.balance`, exactly as the two current-period
spellings are one read. It is the form a pack lowering rule produces, since
`field.<name>` resolves through the entity root (`docs/07`).

A stream environment carries no bare `prev`, so a stream cannot ask for "the
previous value" of something it does not own — the entry is not there to be
found. The same mechanism as `series` being empty when a wave-0 stream
evaluates.

Because everything a rule can read is already finished, no reference can close a
cycle. Fields may therefore reference each other freely, including mutually, and
**declaration order carries no meaning**:

```cfdl
entity asset pair : Asset.Financial {
  a init 1.0 next prev + prev.asset.pair.b
  b init 1.0 next prev + prev.asset.pair.a
}
```

### A field steps on the clock of whatever brought it

A field has no `schedule` clause of its own. An entity is not a temporal thing:
it does not start, stop or recur, so there is no cadence for it to carry. A
field declared directly on an entity therefore steps every model period.

A field a CONTRACT brings inherits that contract's schedule, because the
contract is the thing with a term and a payment frequency. The recurrence
**steps** on that cadence and **holds** between ticks, which is what lets a pool
carried on a daily calendar but paying monthly compound its hazard twelve times
a year rather than three hundred and sixty-five.

Two details that are off-by-one traps:

- The recurrence steps on **accrual** periods, not settlement periods. A
  quarterly schedule accrues at periods 0, 3, 6 and settles at 2, 5, 8; a
  stream's amount is evaluated at the accrual, so that is where the field must
  align.
- `init` is the value **at the first tick**, not at model period 0. Otherwise
  the first payment would read the second value of the recurrence.

Three further properties, each the opposite of a defensible alternative:

- **Holding is not being inactive.** Between ticks a field keeps its value. An
  inactive *stream* yields 0; a field does not, which is why `active when` has
  no meaning here — a cadence says *when the recurrence advances*, not *whether
  the quantity exists*.
- **A field is not cash.** It has no direction or currency. It is published in
  results under its own path as bare numbers, and never enters `model.total`,
  `model.npv`, the annual rollup or any domain metric.
- **`next` has no series access** in v0.1. It sees `prev`,
  `prev.<family>.<entity>.<field>`, `time.*`, `inputs.*`, `cfg`, `obs` and
  curves. Reading a *stream's* history from a recurrence is not expressible;
  `series_sum` remains the route to a stream's window, from a stream.

See `docs/14_state_and_recurrence.md` for the design and the prior art it
follows.

### 3.2 Waterfall steps: `remaining`, `paid` and `owed`

A waterfall step is an expression like any other, with four extra names.

`available` is the cash the waterfall's entity produced this period: its
streams' signed values, netted, with its children rolled up by `part of`.
Streams only — no distribution feeds it, so a waterfall can never read its own
output through it. The engine supplies it before the waterfall runs; no model
declares a field for it. `from available` is therefore the ordinary spelling of
a pot, and the `from` expression remains free for the deals that draw on
something narrower.

`remaining` is what survives the steps above it. A step pays
`min(max(0, expr), remaining)`, so `= remaining` means exactly what it says,
a step asking for more than is left takes what is left, and a negative
expression pays nothing rather than clawing cash back.

`paid.<step>` and `owed.<step>` read a step declared **earlier in the same
waterfall** — what it actually paid, and what it would have paid had the pot
been deep enough. They differ exactly when a step could not be paid in full,
so their difference is that step's shortfall:

```
amount = owed.trustee_fee - paid.trustee_fee
```

That is how a capped fee gets its overflow paid at a later priority, and how a
step measures a balance "after giving effect to" the payments above it. Reading
a step declared later is a compile error.

A step also sees everything a stream sees, including entity fields at the
current period — a waterfall runs after fields are evaluated, so the balances
it tests are period-close values.

Steps publish as series `<waterfall>.<step>`, so `series_sum` reaches an earlier
waterfall's output from a later one's `from` expression. That is how one
waterfall's payment becomes another's pot:

```
from series_sum("senior.residual", time.t, time.t)
```

Results publish the same series one namespace down, as
`stream.<waterfall>.<step>`, because in results every stream carries that
prefix. **The results name is not the name an expression reads.** This section
gave the results name until August 2026, and a model that followed it got an
empty pot rather than a diagnostic — a name nothing matches is not an error,
because a selector that matches nothing must be able to sum to zero. That
asymmetry is now the difference between a `.*` selector and a literal name; see
`E5022_UNKNOWN_SERIES_REFERENCE`.

A step's series is visible to a later waterfall's `from` and to nothing else.
Neither a stream nor a field's `next` can read it.

## 4. Builtin functions

Conditionals & aggregates: `if(cond, a, b)` (lazy — only the taken branch is
evaluated), `min`, `max`, `sum`, `avg`, `abs`.

Rounding: `round(x, [digits])`, `round_down(x, [digits])`,
`round_up(x, [digits])`.

Math: `pow(base, exp)` (function form of `^`), `clamp(x, lo, hi)`.

Time value of money (Excel sign conventions, decimal-exact for whole-period
terms). **Excel sign conventions mean `pmt` returns a negative number for a
positive `pv`** — a loan payment is money leaving. On a stream already declared
`outflow`, negate it (`amount = -pmt(...)`), or the two negatives cancel and the
payment registers as income: `pmt(rate, nper, pv, [fv], [due])`, `pv(rate, nper, pmt, [fv], [due])`,
`fv(rate, nper, pmt, [pv], [due])`, `nper(rate, pmt, pv, [fv], [due])`,
`rate(nper, pmt, pv, [fv], [due], [guess])` (Newton solver, f64, tolerance
1e-12), `ipmt(rate, per, nper, pv, [fv])` / `ppmt(rate, per, nper, pv, [fv])`
(interest/principal split of payment `per`, 1-based; ordinary annuities).

Depreciation: `macrs_rate(year, life)` — IRS Pub 946 GDS half-year convention
percentages for 5/7/15/20-year property (`year` is 0-based; 0 beyond the
recovery period).

Credit: `cpr_to_smm(cpr)`, `cpr_to_periodic(cpr, ppy)`.

`cpr_to_smm(x)` is `1 - (1-x)^(1/12)` and always means *monthly*.
`cpr_to_periodic(x, ppy)` is the same conversion on a grid of `ppy` periods per
year, and `cpr_to_periodic(x, 12) == cpr_to_smm(x)` exactly. Note this is a
**root**, not a division: CPR and CDR are effective annual rates, so they
convert by taking a root, while note rates are nominal and convert by dividing.
Using one convention for the other is a silent factor-level error.

Curves: `curve_value(name, date)` looks up a model-declared `curve`
statement at a date. `step` curves (the default) are flat-forward: the last
point at or before the query date (the first value before the first point).
`linear` curves interpolate linearly in calendar days between bracketing
points and clamp flat outside the declared range. Referencing an undeclared
curve is an evaluation error.

Quantiles: a `quantile` declaration is indexed by cumulative share, not by
date, and three functions read one. `quantile_at(name, share)` is the value at
a share — the direct analogue of `curve_value`. `quantile_mean(name, from, to)`
is the mean over a share slice: a PARTIAL EXPECTATION, computed as the exact
integral of the interpolated function (rectangles under `step`, trapezoids
under `linear`) rather than by sampling it. `quantile_of(name, value)` is the
inverse of `quantile_at` — the share at or below which a value sits — and is
what turns a stated threshold, a lease breakpoint or a tranche attachment
point, into a share a slice can be taken against. Referencing an undeclared
quantile is an evaluation error, and so is passing a curve to these or a
quantile to `curve_value`: the two are on different axes and neither resolves
against the other.

Cross-stream series: `series_sum(name, from_t, to_t)` /
`series_avg(name, from_t, to_t)` aggregate another stream's signed per-period
amounts over an inclusive period window (`prefix.*` wildcards supported).
Streams evaluate in dependency order — waves. A stream that reads no series is
wave 0; a reader evaluates one wave past the deepest stream it reads, against
a store in which everything it names is already finished, to any depth. A
circular read is the one thing no order can satisfy, and the engine refuses it
with the named cycle rather than iterating toward a fixed point (`docs/14`
§5). A read whose series name is computed at runtime evaluates after every
literally-named stream and cannot itself be read. Windows may extend into the
projection tail (`time ... project <n>`), which is computed for valuation
lookups but excluded from cash results and NPV.

Logs: `ln(x)` (natural logarithm, `x > 0`) and `exp(x)`. These exist to turn a
cumulative **product** into a cumulative **sum**: a survival factor or growth
path under a *varying* rate is `PROD(1 + r_i)`, which has no closed form and is
not `pow(1 + r, t)` — that applies one period's rate as though it had held
throughout. Since `series_sum` aggregates a stream over a window,
`exp(series_sum(helper, 0, t))` recovers the product from a stream carrying
`ln(1 + r_t)`. Both escape to float64, as `pow` already does for fractional
exponents, so they are **not decimal-exact**; prefer a closed form where one
exists.

Dates: `date(y, m, d)`, `parse_date(text)` (ISO `YYYY-MM-DD` or `YYYY-MM`),
`edate(d, months)`, `eomonth(d, months)`, `months_between(d1, d2)`,
`days_between(d1, d2)`, `year_frac(d1, d2, basis)`. Date arithmetic: `d2 - d1` yields days;
`d + n` / `d - n` shift by days.

Day-count bases for `year_frac`: `"30/360"` (aliases `"30/360 us"`, `"bond"`),
`"30e/360"` (alias `"eurobond"`), `"act/360"`, `"act/365"`, `"act/act"` (ISDA;
aliases `"actual/actual"`, `"act/act isda"`), per the standard market conventions
definitions.

`act/act` splits the span at calendar-year boundaries and measures each part
against its own year's length, so a period crossing a leap year is not charged
365 days for a 366-day year: 2024-07-01 to 2025-07-01 is 184/366 + 181/365,
not 365/365.

Business days: `is_business_day(d, calendar)`, `roll(d, convention, calendar)`,
`add_business_days(d, n, calendar)`.

- Calendars: `"weekend"` / `"none"` (weekends only), `"us"` / `"us_federal"` /
  `"sifma"`, `"target"` / `"target2"` / `"eur"`, `"uk"` / `"uk_bank"` /
  `"london"`.
- Roll conventions: `"none"`, `"following"`, `"modified_following"`,
  `"preceding"`, `"modified_preceding"`.

```
roll(parse_date("2027-01-01"), "following", "us")   -- next US business day
add_business_days(time.date, 2, "london")           -- T+2 on the UK calendar
```

## 5. Errors and diagnostics

Every parse and evaluation error carries a byte-offset span into the
expression source. The compiler surfaces them as diagnostics with code
`E3001_EXPR_PARSE_ERROR`; runtime failures surface as `EXPR_EVAL` warnings in Results
(the engine substitutes 0 / false and records the warning).

## 6. IR representation

Expressions are stored in IR as their raw source text with
`"lang": "cfdl"`:

```json
{ "lang": "cfdl", "src": "50000 * pow(1.15, time.t / 12.0)" }
```


---

# IR schema

_Generated from docs/schemas/ir.schema.json; the JSON Schema itself is served at /schemas/CFDL_v0_1_IR.schema.json._

<!-- GENERATED from docs/schemas/ir.schema.json — do not edit by hand.
     tools/check-ir-schema.py fails the build if this drifts. This page
     was an independently maintained copy, which is how the results
     schema drifted four releases before anyone noticed. -->

# IR schema

The shape of a `cfdl compile` IR document. This is the published contract,
also served at `cfdl.dev/schemas`; every committed IR golden is validated
against it by `make ir-schema`.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cfdl.dev/schemas/CFDL_v0_1_IR.schema.json",
  "title": "CFDL v0.1 Canonical IR",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "ir_version",
    "model",
    "time",
    "phases",
    "entities",
    "assumptions",
    "contracts",
    "streams",
    "events",
    "options",
    "runs",
    "required_observables",
    "required_refs",
    "provenance"
  ],
  "properties": {
    "ir_version": {
      "type": "string",
      "const": "0.1"
    },
    "model": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "currency"
      ],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        }
      }
    },
    "time": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "calendar",
        "start",
        "periods"
      ],
      "properties": {
        "calendar": {
          "$ref": "#/$defs/Frequency"
        },
        "start": {
          "$ref": "#/$defs/Date"
        },
        "periods": {
          "type": "integer",
          "minimum": 1
        },
        "projection": {
          "type": "integer",
          "minimum": 0
        }
      }
    },
    "phases": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Phase"
      }
    },
    "entities": {
      "type": "array",
      "minItems": 1,
      "items": {
        "$ref": "#/$defs/Entity"
      }
    },
    "assumptions": {
      "$ref": "#/$defs/Assumptions"
    },
    "curves": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Curve"
      }
    },
    "quantiles": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Quantile"
      },
      "description": "Values indexed by cumulative share. Always ascending by share: a declaration written `by exceedance` is reversed at compile time, so this document carries one orientation and no reader has to know which way the source was written."
    },
    "quantile_inputs": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/QuantileCall"
      },
      "description": "Every quantile call site in the document, resolved and deduplicated. The audit record for a nonlinear input: publishing the declaration alone would say a distribution existed, not which slice of it struck a number."
    },
    "accounts": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Account"
      },
      "description": "Declared cash locations. Omitted when a model declares none, so existing IR stays byte-identical."
    },
    "lifecycles": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Lifecycle"
      },
      "description": "Every machine an entity binds — pack-declared and model-declared resolved to the same shape. Absent when no entity has one."
    },
    "waterfalls": {
      "type": "array",
      "minItems": 0,
      "description": "Ordered allocations of a pot — a priority of payments. Steps run in declaration order after the period's fields and streams are known; each takes min(max(0, its amount), what remains). Omitted when a model declares none.",
      "items": {
        "$ref": "#/$defs/Waterfall"
      }
    },
    "subtotals": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Subtotal"
      },
      "description": "Per-period subtotals declared by the active pack, in dependency order. Omitted when the pack declares none."
    },
    "contracts": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Contract"
      }
    },
    "streams": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Stream"
      }
    },
    "stream_inputs": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/StreamInputs"
      },
      "description": "Per-stream record of what each pack rule consumed. Omitted when nothing was lowered from a pack."
    },
    "events": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Event"
      }
    },
    "options": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Option"
      }
    },
    "runs": {
      "type": "array",
      "minItems": 1,
      "items": {
        "$ref": "#/$defs/Run"
      }
    },
    "metrics": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Metric"
      },
      "description": "Reserved. Metrics are computed at run time by the engine and by the active pack, so a compile output does not carry them; no compiler emits this field."
    },
    "required_observables": {
      "type": "array",
      "description": "Ontology observable IDs referenced via obs('...')",
      "items": {
        "type": "string",
        "minLength": 1
      },
      "uniqueItems": true
    },
    "required_refs": {
      "type": "array",
      "description": "Ontology reference IDs referenced via ref('...')",
      "items": {
        "type": "string",
        "minLength": 1
      },
      "uniqueItems": true
    },
    "provenance": {
      "$ref": "#/$defs/Provenance"
    }
  },
  "$defs": {
    "Id": {
      "type": "string",
      "minLength": 1,
      "maxLength": 256
    },
    "Qname": {
      "type": "string",
      "pattern": "^[A-Za-z_][A-Za-z0-9_]*(\\.[A-Za-z_][A-Za-z0-9_]*)*$"
    },
    "Date": {
      "type": "string",
      "description": "ISO date YYYY-MM-DD",
      "pattern": "^\\d{4}-\\d{2}-\\d{2}$"
    },
    "DateRange": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "start",
        "end"
      ],
      "properties": {
        "start": {
          "$ref": "#/$defs/Date"
        },
        "end": {
          "$ref": "#/$defs/Date"
        }
      }
    },
    "Frequency": {
      "type": "string",
      "enum": [
        "daily",
        "weekly",
        "monthly",
        "quarterly",
        "annual"
      ]
    },
    "Currency": {
      "type": "string",
      "description": "ISO 4217",
      "pattern": "^[A-Z]{3}$"
    },
    "Decimal": {
      "type": "number"
    },
    "Money": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "amount",
        "currency"
      ],
      "properties": {
        "amount": {
          "$ref": "#/$defs/Decimal"
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        }
      }
    },
    "Rate": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "value"
      ],
      "properties": {
        "value": {
          "$ref": "#/$defs/Decimal"
        },
        "basis": {
          "type": "string",
          "description": "Optional semantic basis label (e.g., 'annual')",
          "minLength": 1
        }
      }
    },
    "Expr": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "lang",
        "src"
      ],
      "properties": {
        "lang": {
          "type": "string",
          "enum": [
            "cfdl"
          ]
        },
        "src": {
          "type": "string",
          "minLength": 1
        }
      }
    },
    "Phase": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "range",
        "name"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "range": {
          "$ref": "#/$defs/DateRange"
        },
        "name": {
          "$ref": "#/$defs/Id"
        }
      }
    },
    "EntityRef": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "symbol"
      ],
      "properties": {
        "symbol": {
          "type": "string",
          "pattern": "^[A-Za-z_][A-Za-z0-9_]*\\.[A-Za-z_][A-Za-z0-9_]*$",
          "description": "Entity symbol like 'asset.Sunset'"
        }
      }
    },
    "Entity": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "symbol",
        "type",
        "fields"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "symbol": {
          "type": "string",
          "pattern": "^[A-Za-z_][A-Za-z0-9_]*\\.[A-Za-z_][A-Za-z0-9_]*$"
        },
        "type": {
          "$ref": "#/$defs/Qname"
        },
        "fields": {
          "type": "object",
          "description": "Field values declared in the entity's block, checked against the fields its ontology type declares. Literals here: a field stated with '=' is a fact about the thing. A field that moves carries an 'init'/'next' rule instead.",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "state": {
          "type": "object",
          "description": "Mutable runtime state fields (may be empty at compile time)",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "parent": {
          "type": "string",
          "description": "The entity this one is part of. ALWAYS OPTIONAL, and absent for most entities: hierarchy is available at every grain and required at none. A pool models collective behavior with no loans under it; a building needs no units. The modeller chooses the grain, and the language does not prefer one."
        },
        "initial_state": {
          "type": "string",
          "description": "The lifecycle state this entity starts in, overriding its type's declared initial. Absent when the type declares no lifecycle. An entity WITH a lifecycle is always in exactly one of its states — there is no null state and no undeclared state, which is what makes a misspelled status a compile error rather than a wrong answer."
        },
        "rules": {
          "type": "object",
          "description": "Fields that MOVE, as recurrences owned by this entity. A field stated with '=' is a fact and lives in `fields`; a field with an 'init'/'next' rule lives here. A rule with no 'next' in source is written out as `next prev`, because a field with no rule holds.",
          "additionalProperties": {
            "type": "object",
            "required": [
              "init",
              "next"
            ],
            "additionalProperties": false,
            "properties": {
              "init": {
                "$ref": "#/$defs/Expr"
              },
              "next": {
                "$ref": "#/$defs/Expr"
              },
              "schedule": {
                "$ref": "#/$defs/Schedule"
              }
            },
            "description": "A field's recurrence. `schedule` is present only on a field a PACK emitted: it inherits the contract's payment rhythm, so a monthly-paying pool on a daily book compounds twelve times a year rather than 365. A field a modeller wrote has none and steps every period."
          }
        },
        "lifecycle": {
          "type": "string",
          "description": "The machine this entity is governed by — an id into `lifecycles`. Absent for the many entities that have none."
        }
      }
    },
    "TypedValue": {
      "description": "Strongly-typed value union used for fields/terms/state. anyOf, not oneOf: the members overlap structurally — an Expr {lang, src} is also a valid Map of strings — so requiring exactly one match can never hold for an untagged union.",
      "anyOf": [
        {
          "type": "string"
        },
        {
          "type": "boolean"
        },
        {
          "type": "integer"
        },
        {
          "type": "number"
        },
        {
          "$ref": "#/$defs/Date"
        },
        {
          "$ref": "#/$defs/Money"
        },
        {
          "$ref": "#/$defs/Rate"
        },
        {
          "$ref": "#/$defs/Expr"
        },
        {
          "type": "array",
          "items": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        }
      ]
    },
    "Curve": {
      "type": "object",
      "required": [
        "name",
        "points"
      ],
      "additionalProperties": false,
      "properties": {
        "name": {
          "type": "string"
        },
        "interpolation": {
          "type": "string",
          "enum": [
            "step",
            "linear"
          ]
        },
        "points": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "object",
            "required": [
              "date",
              "value"
            ],
            "additionalProperties": false,
            "properties": {
              "date": {
                "type": "string"
              },
              "value": {
                "type": "number"
              }
            }
          }
        }
      }
    },
    "Quantile": {
      "type": "object",
      "required": [
        "name",
        "points"
      ],
      "additionalProperties": false,
      "properties": {
        "name": {
          "type": "string"
        },
        "interpolation": {
          "type": "string",
          "enum": [
            "step",
            "linear"
          ],
          "description": "The same two words a curve uses. It also fixes quadrature: `quantile_mean` is the exact integral of the function these describe, so the declaration carries no separate quadrature choice."
        },
        "reference": {
          "type": "string",
          "description": "The pack `[[references]]` id this realises, when the declaration named one. Present only when stated."
        },
        "points": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "object",
            "required": [
              "share",
              "value"
            ],
            "additionalProperties": false,
            "properties": {
              "share": {
                "type": "number",
                "minimum": 0,
                "maximum": 1,
                "description": "Cumulative share of the measure. The measure itself — hours, balance, square feet — belongs to the contract that reads this, which is what lets one declaration serve assets of different size."
              },
              "value": {
                "type": "number"
              }
            }
          }
        }
      }
    },
    "QuantileCall": {
      "type": "object",
      "required": [
        "quantile",
        "function",
        "args"
      ],
      "additionalProperties": false,
      "properties": {
        "quantile": {
          "type": "string",
          "description": "The quantile named at the call site."
        },
        "function": {
          "type": "string",
          "enum": [
            "quantile_at",
            "quantile_mean",
            "quantile_of"
          ]
        },
        "args": {
          "type": "array",
          "items": {
            "type": "number"
          },
          "description": "The literal arguments after the name, in source order. Empty when they were not literals."
        },
        "value": {
          "type": "number",
          "description": "What the call resolves to, rounded to the engine's published-number policy so it agrees exactly with the ledger figure it explains. ABSENT when an argument is not a literal — the call is still listed, because a silently omitted call site would read as a model that never made one."
        }
      }
    },
    "Assumptions": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "constants",
        "random"
      ],
      "properties": {
        "constants": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/AssumeConstant"
          }
        },
        "random": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/AssumeRandom"
          }
        }
      }
    },
    "AssumeConstant": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "expr",
        "type"
      ],
      "properties": {
        "name": {
          "$ref": "#/$defs/Id"
        },
        "expr": {
          "$ref": "#/$defs/Expr"
        },
        "type": {
          "$ref": "#/$defs/ValueTypeId"
        }
      }
    },
    "AssumeRandom": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "dist",
        "type"
      ],
      "properties": {
        "name": {
          "$ref": "#/$defs/Id"
        },
        "dist": {
          "$ref": "#/$defs/Distribution"
        },
        "type": {
          "$ref": "#/$defs/ValueTypeId"
        }
      }
    },
    "ValueTypeId": {
      "type": "string",
      "enum": [
        "String",
        "Bool",
        "Int",
        "Decimal",
        "Date",
        "Money",
        "Rate"
      ]
    },
    "Distribution": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind",
        "params"
      ],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "Normal",
            "LogNormal",
            "Uniform",
            "Triangular"
          ]
        },
        "params": {
          "type": "object",
          "additionalProperties": {
            "type": [
              "number",
              "string",
              "boolean"
            ]
          }
        },
        "clip": {
          "type": "array",
          "items": {
            "type": "number"
          },
          "minItems": 2,
          "maxItems": 2
        }
      }
    },
    "Contract": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "type",
        "subject",
        "term",
        "currency",
        "terms",
        "effects",
        "provenance"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "type": {
          "$ref": "#/$defs/Qname"
        },
        "subject": {
          "$ref": "#/$defs/EntityRef"
        },
        "term": {
          "$ref": "#/$defs/DateRange"
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        },
        "parties": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "tags": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "terms": {
          "type": "object",
          "description": "Contract terms; pack may validate and type-check",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "effects": {
          "$ref": "#/$defs/Effects"
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        }
      }
    },
    "Effects": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "streams"
      ],
      "properties": {
        "streams": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/Stream"
          }
        }
      }
    },
    "Stream": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "owner",
        "direction",
        "currency",
        "schedule",
        "amount",
        "active_when",
        "provenance"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "owner": {
          "$ref": "#/$defs/EntityRef"
        },
        "direction": {
          "$ref": "#/$defs/Direction"
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        },
        "category": {
          "type": "string",
          "description": "What this stream is economically (revenue, opex, debt_service, ...). Aggregation reads this rather than pattern-matching the name, so the meaning is declared once at the point of emission instead of being re-derived by every consumer. Must name a category the active pack declares (E5022). Absent when the stream is unclassified, which is legal and leaves it out of every category fold."
        },
        "schedule": {
          "$ref": "#/$defs/Schedule"
        },
        "amount": {
          "$ref": "#/$defs/Expr"
        },
        "active_when": {
          "description": "If omitted in source, compiler should emit true",
          "$ref": "#/$defs/Expr"
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        }
      }
    },
    "Direction": {
      "type": "string",
      "enum": [
        "inflow",
        "outflow"
      ]
    },
    "Schedule": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind"
      ],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "OnDate",
            "Every",
            "PhaseEnter",
            "EveryPhase",
            "StateEnter"
          ]
        },
        "on": {
          "$ref": "#/$defs/Date"
        },
        "every": {
          "$ref": "#/$defs/Frequency"
        },
        "from": {
          "$ref": "#/$defs/Date"
        },
        "to": {
          "$ref": "#/$defs/Date"
        },
        "on_rule": {
          "description": "Optional rule for day-of-month or weekday sets",
          "$ref": "#/$defs/OnRule"
        },
        "convention": {
          "type": "string",
          "enum": [
            "none",
            "following",
            "modified_following",
            "preceding",
            "modified_preceding"
          ]
        },
        "calendar": {
          "type": "string"
        },
        "phase": {
          "type": "string",
          "description": "Phase name for PhaseEnter/EveryPhase"
        },
        "except_dates": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "also_dates": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "net_days": {
          "description": "Days between a flow being earned and its cash moving. Omitted when cash lands in the period that earned it.",
          "type": "integer",
          "minimum": 0
        },
        "net_months": {
          "description": "Months between a flow being earned and its cash moving, stepped by the calendar rather than as 30-day units.",
          "type": "integer",
          "minimum": 0
        },
        "placement": {
          "type": "string",
          "enum": [
            "start",
            "mid",
            "end"
          ],
          "description": "Where in its period the flow sits. One axis with three positions, so two placements cannot both be stated. Omitted for the form's default, which differs by form: a one-shot (`OnDate`) opens its period, a recurrence closes it (an ordinary annuity — the interval elapses, then payment falls). `start` is an annuity due and what expense-like streams want; `mid` is the project-finance convention, half a period on every calendar, a convention rather than a date; `end` is what a disposal needs, since a reversion is taken at the close of the holding period. Mutually exclusive with a day rule and with payment terms (E2109). See 12_payment_timing.md."
        },
        "anchor_entity": {
          "type": "string",
          "description": "state_enter anchor (docs/28 §6.2): the entity whose entries open the windows. Present only for kind StateEnter."
        },
        "anchor_state": {
          "type": "string",
          "description": "The state whose entry anchors the window; a re-entered state re-anchors, and a self-edge is an entry."
        },
        "anchor_periods": {
          "type": "integer",
          "minimum": 1,
          "description": "Window length in grid periods from each entry."
        }
      },
      "allOf": [
        {
          "if": {
            "properties": {
              "kind": {
                "const": "OnDate"
              }
            }
          },
          "then": {
            "required": [
              "on"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "Every"
              }
            }
          },
          "then": {
            "required": [
              "every",
              "from",
              "to"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "PhaseEnter"
              }
            }
          },
          "then": {
            "required": [
              "phase"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "EveryPhase"
              }
            }
          },
          "then": {
            "required": [
              "every",
              "phase"
            ]
          }
        }
      ]
    },
    "OnRule": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind"
      ],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "DayOfMonth",
            "EndOfMonth"
          ]
        },
        "day": {
          "type": "integer",
          "minimum": 1,
          "maximum": 31
        }
      },
      "allOf": [
        {
          "if": {
            "properties": {
              "kind": {
                "const": "DayOfMonth"
              }
            }
          },
          "then": {
            "required": [
              "day"
            ]
          }
        }
      ]
    },
    "Weekday": {
      "type": "string",
      "enum": [
        "Mon",
        "Tue",
        "Wed",
        "Thu",
        "Fri",
        "Sat",
        "Sun"
      ]
    },
    "BusinessDayConvention": {
      "type": "string",
      "enum": [
        "none",
        "following",
        "modified_following",
        "preceding",
        "modified_preceding"
      ]
    },
    "Event": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "when",
        "actions",
        "provenance"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "when": {
          "$ref": "#/$defs/Expr"
        },
        "actions": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/Action"
          }
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        }
      }
    },
    "Action": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind"
      ],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "SetEntityField",
            "ActivateStream",
            "DeactivateStream",
            "ActivateContract",
            "DeactivateContract",
            "ExerciseOption"
          ]
        },
        "entity": {
          "$ref": "#/$defs/EntityRef"
        },
        "field": {
          "$ref": "#/$defs/Id"
        },
        "value": {
          "$ref": "#/$defs/TypedValue"
        },
        "stream": {
          "$ref": "#/$defs/Id"
        },
        "contract": {
          "$ref": "#/$defs/Id"
        },
        "option": {
          "$ref": "#/$defs/Id"
        }
      },
      "allOf": [
        {
          "if": {
            "properties": {
              "kind": {
                "const": "SetEntityField"
              }
            }
          },
          "then": {
            "required": [
              "entity",
              "field",
              "value"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "ActivateStream"
              }
            }
          },
          "then": {
            "required": [
              "stream"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "DeactivateStream"
              }
            }
          },
          "then": {
            "required": [
              "stream"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "ActivateContract"
              }
            }
          },
          "then": {
            "required": [
              "contract"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "DeactivateContract"
              }
            }
          },
          "then": {
            "required": [
              "contract"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "ExerciseOption"
              }
            }
          },
          "then": {
            "required": [
              "option"
            ]
          }
        }
      ]
    },
    "Option": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "type",
        "exercise_when",
        "payoff",
        "provenance"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "type": {
          "$ref": "#/$defs/Qname"
        },
        "exercisable_in_phase": {
          "$ref": "#/$defs/Id"
        },
        "exercise_when": {
          "$ref": "#/$defs/Expr"
        },
        "payoff": {
          "$ref": "#/$defs/Expr"
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        },
        "owner": {
          "$ref": "#/$defs/EntityRef",
          "description": "The asset this option is written on. AN OPTION IS A CONTRACT WITH AN ELECTION, so it attaches to something the way every other contract does. Absent on an option written before options had owners; without one its payoff belongs to no entity and falls out of every per-entity total."
        },
        "parties": {
          "type": "array",
          "description": "Who the option is between, by role. The role is named by the contract TYPE rather than by the party, because the same party is lessor in one agreement and lender in another — the role belongs to the agreement.",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [
              "role",
              "entity"
            ],
            "properties": {
              "role": {
                "type": "string"
              },
              "entity": {
                "$ref": "#/$defs/EntityRef"
              }
            }
          }
        }
      }
    },
    "Run": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind"
      ],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "deterministic",
            "monte_carlo"
          ]
        },
        "trials": {
          "type": "integer",
          "minimum": 1
        },
        "seed": {
          "type": "integer",
          "minimum": 0
        }
      },
      "allOf": [
        {
          "if": {
            "properties": {
              "kind": {
                "const": "monte_carlo"
              }
            }
          },
          "then": {
            "required": [
              "trials",
              "seed"
            ]
          }
        }
      ]
    },
    "Metric": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "expr",
        "provenance"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "expr": {
          "$ref": "#/$defs/Expr"
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        }
      }
    },
    "Provenance": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "sources",
        "compiler"
      ],
      "properties": {
        "sources": {
          "type": "array",
          "items": {
            "type": "string",
            "minLength": 1
          },
          "minItems": 1
        },
        "compiler": {
          "type": "object",
          "additionalProperties": false,
          "required": [
            "name",
            "version",
            "hash"
          ],
          "properties": {
            "name": {
              "type": "string",
              "minLength": 1
            },
            "version": {
              "type": "string",
              "minLength": 1
            },
            "hash": {
              "type": "string",
              "minLength": 8
            },
            "notes": {
              "type": "array",
              "items": {
                "type": "string",
                "minLength": 1
              }
            }
          }
        }
      }
    },
    "NodeProvenance": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "source_file",
        "source_span"
      ],
      "properties": {
        "source_file": {
          "type": "string",
          "minLength": 1
        },
        "source_span": {
          "type": "object",
          "additionalProperties": false,
          "required": [
            "start_line",
            "start_col",
            "end_line",
            "end_col"
          ],
          "properties": {
            "start_line": {
              "type": "integer",
              "minimum": 1
            },
            "start_col": {
              "type": "integer",
              "minimum": 1
            },
            "end_line": {
              "type": "integer",
              "minimum": 1
            },
            "end_col": {
              "type": "integer",
              "minimum": 1
            }
          }
        },
        "notes": {
          "type": "string"
        },
        "generated_by": {
          "$ref": "#/$defs/GeneratedBy"
        }
      }
    },
    "GeneratedBy": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "pack",
        "rule_id"
      ],
      "properties": {
        "pack": {
          "$ref": "#/$defs/PackRef"
        },
        "rule_id": {
          "type": "string",
          "minLength": 1
        }
      }
    },
    "PackRef": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "version"
      ],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1
        },
        "version": {
          "type": "string",
          "minLength": 1
        }
      }
    },
    "StreamInputs": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "stream",
        "contract",
        "terms"
      ],
      "description": "What a pack lowering rule CONSUMED to strike one stream: the placeholders its templates actually substituted, plus the rule defaults that filled a gap. Not the contract's whole term map — a contract lowers to several streams and each reads a different subset, so 'the contract's terms' is not an answer to 'what struck this line'. Pack, rule id and source span are already on the stream's own provenance and are not repeated here. Absent for hand-written streams, which no rule struck.",
      "properties": {
        "stream": {
          "$ref": "#/$defs/Id"
        },
        "contract": {
          "type": "string",
          "description": "The contract instance the rule matched, including any suffix."
        },
        "terms": {
          "type": "object",
          "additionalProperties": {
            "type": "string"
          },
          "description": "Resolved placeholder values, as the strings the templates substituted. Not coerced: a term's payload is text plus a span, which is the contract packs already work against."
        },
        "defaults_applied": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Keys the contract did not supply, filled from the rule's own defaults. Separated because 'the model said 0' and 'the pack assumed 0' are different facts, and a reader tracing a number needs to tell them apart."
        }
      }
    },
    "Subtotal": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "kind",
        "op"
      ],
      "description": "A per-period subtotal: a named fold over the ledger, lowered from the active pack. Where a metric reduces to one lifetime scalar, this produces a value per period — the middle rows of a statement. Folds CATEGORIES by preference rather than stream names, so net operating income is everything under `operating.*` and nothing enumerates which streams those are. Array order is DEPENDENCY order: an entry may reference only ones before it, which makes a cycle unexpressible rather than merely rejected. A subtotal is a fold OF the cash and never counts as cash: it is excluded from model.total, model.npv, model.net_cash_flow and the per-stream annual rollup, by the same construction that keeps a field out of the cash.",
      "properties": {
        "id": {
          "type": "string",
          "description": "Output series key; must start with `domain.`."
        },
        "kind": {
          "enum": [
            "money",
            "number"
          ]
        },
        "op": {
          "enum": [
            "sum",
            "negated_sum",
            "cumulative",
            "negated_cumulative",
            "ratio"
          ],
          "description": "How the subtotal folds. `sum` and `negated_sum` total one period; `cumulative` and `negated_cumulative` carry a running total, which is how a stock is derived from a flow — principal paid to date, capital called to date. `ratio` divides two money subtotals."
        },
        "categories": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Category path selectors, e.g. `operating.revenue.*`."
        },
        "streams": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Stream-name selectors, for what a category cannot express."
        },
        "subtotals": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Ids of subtotals declared earlier."
        },
        "numerator": {
          "type": "string"
        },
        "denominator": {
          "type": "string"
        },
        "formula": {
          "type": "string",
          "description": "Human-readable lineage, emitted verbatim."
        }
      }
    },
    "Waterfall": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "entity",
        "source",
        "steps"
      ],
      "properties": {
        "name": {
          "type": "string"
        },
        "entity": {
          "type": "string",
          "description": "The entity whose cash this allocates."
        },
        "schedule": {
          "$ref": "#/$defs/Schedule"
        },
        "source": {
          "$ref": "#/$defs/Expr"
        },
        "steps": {
          "type": "array",
          "minItems": 0,
          "items": {
            "$ref": "#/$defs/WaterfallStep"
          }
        }
      }
    },
    "WaterfallStep": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "payee",
        "amount"
      ],
      "properties": {
        "name": {
          "type": "string"
        },
        "payee": {
          "type": "string",
          "description": "The entity this step pays."
        },
        "amount": {
          "$ref": "#/$defs/Expr",
          "description": "What the step is owed. `remaining`, `paid.<step>` and `owed.<step>` are bound on top of the ordinary expression environment."
        },
        "payee_is_account": {
          "type": "boolean",
          "description": "The payee is an ACCOUNT rather than a party. Allocating to a party allocates into that party's account when it has one; naming an account directly is the explicit form, and is how a reserve is funded. Omitted when false, so existing IR stays byte-identical."
        }
      }
    },
    "Account": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name"
      ],
      "description": "A declared cash location whose balance carries across periods. `available` is unchanged and still means this period's netted cash; an account is the ACCUMULATED cash, and is what a waterfall draws its pot from when its schedule names a period. There is no currency: an account is denominated by the model, so the only thing a clause could express is a restatement of what the model already declares, or a second unit that must be refused.",
      "properties": {
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string",
          "description": "The party this account belongs to, when it belongs to one. A general account has none — a collection account, a reserve — and a party-owned one holds what has been ALLOCATED to that party. It is not an obligation: it holds cash that exists, and what is still owed is simply not yet allocated."
        },
        "inflow": {
          "$ref": "#/$defs/Expr",
          "description": "What flows in each period. May be negative: an account fed a deal's whole net cash IS the deal's cumulative position, negative through the J-curve and positive after."
        }
      }
    },
    "Lifecycle": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "initial",
        "states"
      ],
      "description": "A declared finite state machine (docs/28 §6.1). The states are enumerated ahead of time — the finite set is what makes a misspelled state a compile error rather than a phantom state. The edges are declared only as used: declaring one brings it into existence, an undeclared edge does not exist, and absence is the prohibition. Only machines an entity binds are published.",
      "properties": {
        "id": {
          "type": "string"
        },
        "initial": {
          "type": "string",
          "description": "The state the machine opens in; an entity's initial_state overrides it."
        },
        "states": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "minItems": 1
        },
        "edges": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/LifecycleEdge"
          },
          "description": "Empty or absent means the machine is unconstrained — permits()'s empty-means-open rule."
        }
      }
    },
    "LifecycleEdge": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "from",
        "to"
      ],
      "description": "One edge, from -> to. A self-edge (from = to) is a real transition: it journals and re-anchors. A guard-less edge is a permission only — an event's write may take it, but the machine never fires it on its own.",
      "properties": {
        "from": {
          "type": "string"
        },
        "to": {
          "type": "string"
        },
        "guard": {
          "$ref": "#/$defs/Expr",
          "description": "Evaluated each period the entity is in `from`, reading state as the period opened and series strictly backward (docs/28 §4). There is no latch: edge availability is the memory."
        }
      }
    }
  }
}
```


---

# Results schema

_Generated from docs/schemas/results.schema.json; the JSON Schema itself is served at /schemas/CFDL_v0_1_Results.schema.json._

<!-- GENERATED from docs/schemas/results.schema.json — do not edit by hand.
     tools/check-results-schema.py fails the build if this drifts.
     This page existed as an independently maintained copy and was four
     releases behind: it declared results_version 0.1 while the engine
     emitted 0.2, and omitted two whole sections. -->

# Results schema

The shape of a `cfdl run` results document. This is the published contract,
also served at `cfdl.dev/schemas`; every committed results golden is validated
against it by `make results-schema`.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cfdl.dev/schemas/CFDL_v0_1_Results.schema.json",
  "title": "CFDL v0.1 Results",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "results_version",
    "model_hash",
    "engine",
    "warnings",
    "deterministic",
    "scenarios",
    "monte_carlo",
    "ledger_hash"
  ],
  "properties": {
    "results_version": {
      "type": "string",
      "const": "0.5",
      "description": "Schema version of this document. 0.5 added the machine's `transition` journal action. 0.4 added the account journal actions `inflow`, `allocate_in` and `allocate_out`. 0.3 added `ledger_hash` and the optional `inputs` section, and `category` on IR streams upstream of it."
    },
    "model_hash": {
      "type": "string",
      "description": "Hash of canonical IR for traceability",
      "minLength": 8
    },
    "ledger_hash": {
      "type": "string",
      "description": "SHA-256 over the canonical form of the deterministic ledger — `deterministic.series` and `deterministic.annual_rollup`. Together with `model_hash` and `engine` this closes the chain: identical inputs on an identical engine must reproduce an identical ledger_hash. It covers the LEDGER, not the metrics: NPV and IRR are derived FROM the ledger, so including them would make the hash move for a reason the ledger did not. It is therefore invariant to the discount rate, which is correct — the ledger is cash before discounting."
    },
    "engine": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "version"
      ],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1
        },
        "version": {
          "type": "string",
          "minLength": 1
        },
        "build": {
          "type": "string"
        }
      }
    },
    "warnings": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "inputs": {
      "$ref": "#/$defs/InputsSection"
    },
    "deterministic": {
      "$ref": "#/$defs/DeterministicSection"
    },
    "scenarios": {
      "$ref": "#/$defs/ScenariosSection"
    },
    "monte_carlo": {
      "$ref": "#/$defs/MonteCarloSection"
    },
    "domain_metrics": {
      "$ref": "#/$defs/DomainMetrics"
    },
    "statements": {
      "$ref": "#/$defs/StatementsSection"
    }
  },
  "$defs": {
    "Currency": {
      "type": "string",
      "pattern": "^[A-Z]{3}$"
    },
    "Decimal": {
      "type": "number"
    },
    "Money": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "amount",
        "currency"
      ],
      "properties": {
        "amount": {
          "$ref": "#/$defs/Decimal"
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        }
      }
    },
    "Date": {
      "type": "string",
      "pattern": "^\\d{4}-\\d{2}-\\d{2}$"
    },
    "SeriesIndex": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "calendar",
        "start",
        "periods"
      ],
      "properties": {
        "calendar": {
          "type": "string",
          "enum": [
            "daily",
            "monthly",
            "quarterly",
            "annual"
          ]
        },
        "start": {
          "$ref": "#/$defs/Date"
        },
        "periods": {
          "type": "integer",
          "minimum": 1
        }
      }
    },
    "Scalar": {
      "description": "Scalar metric output",
      "oneOf": [
        {
          "type": "number"
        },
        {
          "$ref": "#/$defs/Money"
        },
        {
          "type": "string"
        },
        {
          "type": "boolean"
        },
        {
          "type": "null"
        }
      ]
    },
    "Series": {
      "description": "Time series aligned to the model timeline",
      "type": "object",
      "additionalProperties": false,
      "required": [
        "index",
        "values"
      ],
      "properties": {
        "index": {
          "$ref": "#/$defs/SeriesIndex"
        },
        "values": {
          "type": "array",
          "minItems": 1,
          "items": {
            "oneOf": [
              {
                "$ref": "#/$defs/Decimal"
              },
              {
                "$ref": "#/$defs/Money"
              },
              {
                "type": "null"
              }
            ]
          },
          "description": "One entry per period. Money for a cash series; a bare number for a dimensionless one such as an entity field, which has no denomination."
        },
        "offset": {
          "description": "Where in each period this series' cash falls: 0.0 at the period's open (an annuity due, or a one-shot on its date), 1.0 at its close (an ordinary annuity, the default), 0.5 for the mid-period convention. The same offset used to discount the series, and the axis `model.wal_years` and `model.payback_years` are measured on — so an ordinary annuity's first monthly collection is at 1/12 of a year, not 0. Absent on aggregates (`model.net_cash_flow`, the annual rollup), which sum streams whose placements differ. See 12_payment_timing.md. Absent on field series, which are not paid and so sit nowhere in their period.",
          "type": "number"
        }
      }
    },
    "MetricMap": {
      "type": "object",
      "description": "Named metric scalars",
      "additionalProperties": {
        "$ref": "#/$defs/Scalar"
      }
    },
    "SeriesMap": {
      "type": "object",
      "description": "Named time series outputs. Keys are prefixed by what they are: `stream.<name>` and `option.<name>` are cash and carry a currency; `model.net_cash_flow` is their aggregate; `<family>.<entity>.<field>` is an entity field and is NOT cash — it is a bare number with no currency and no offset, published so a recurrence can be inspected, and it never enters model.total, model.npv, the annual rollup or any domain metric. `entity.<symbol>.net_cash_flow` is an entity's cash AGGREGATED BY RELATION — its own streams plus every descendant's, following `part_of` rather than a name prefix, so a building's cash is its units' cash because they ARE its units. An entity with no children carries its own streams only, which is the pool that models collective behavior directly; the grain is the modeller's choice. Like a subtotal it is a fold OF the cash and never counts AS cash — excluded from model.total, model.npv, model.net_cash_flow and the annual rollup, because counting a parent and its children would double what it touches. `domain.<pack>.<name>` is a per-period SUBTOTAL — a declared aggregation of the classified streams. Money for a sum, a bare number or `null` for a ratio whose denominator vanishes. Like a field, it never enters model.total, model.npv, model.net_cash_flow or the per-stream annual rollup: it is an aggregation OF the cash, so counting it as cash would double what it touches. It carries no `offset`, because a subtotal spans streams that may settle at different points in a period and so has no single placement to claim.",
      "additionalProperties": {
        "$ref": "#/$defs/Series"
      }
    },
    "DeterministicSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "status",
        "metrics",
        "series"
      ],
      "properties": {
        "status": {
          "type": "string",
          "enum": [
            "not_run",
            "ok",
            "error"
          ]
        },
        "metrics": {
          "$ref": "#/$defs/MetricMap"
        },
        "series": {
          "$ref": "#/$defs/SeriesMap"
        },
        "errors": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/RuntimeError"
          }
        },
        "annual_rollup": {
          "$ref": "#/$defs/AnnualRollupSection"
        },
        "transitions": {
          "type": "array",
          "description": "Every state change an event made, in the order it happened — the audit trail for whether and when something occurred. Entity state is otherwise unobservable: nothing else distinguishes an event that fired against a misspelled target from an event that never fired, and without this a case cannot assert a transition. Recorded even when the value does not change, because the question the log answers is whether the event fired. Omitted when a model has no events. Visibility is two rules, not one: an event or option guard reads the state as the period OPENED, so declaration order cannot change an answer; a stream reads it as the period CLOSED, so a transition takes effect in the period it fires.",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [
              "period",
              "date",
              "entity",
              "field",
              "to",
              "event"
            ],
            "properties": {
              "period": {
                "type": "integer",
                "minimum": 0
              },
              "date": {
                "type": "string"
              },
              "entity": {
                "type": "string"
              },
              "field": {
                "type": "string"
              },
              "from": {
                "type": "string",
                "description": "The value before. Absent when the field had none — which, for a typed entity with a lifecycle, should not happen, because it opens in its declared initial state."
              },
              "to": {
                "type": "string"
              },
              "event": {
                "type": "string",
                "description": "The event that fired. A transition always has a cause."
              }
            }
          }
        },
        "journal": {
          "type": "array",
          "description": "Every causal act the run performed, with what became of it, in the order the engine performed them. `transitions` records field CHANGES; the journal answers the question a reviewer asks — what did the model DO, and did each thing it was asked to do happen. An action that was declined, ignored or overridden changes nothing and so appears nowhere else: an `activate stream` that lost to the stream's own `active when` used to leave no trace at all. Flat on purpose — one row per act — so a golden asserts on lines, a reviewer greps for a stream name, and this schema checks one row type. Omitted when a model has no events, options or waterfalls, so such a model publishes exactly what it published before.",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [
              "period",
              "date",
              "actor",
              "action",
              "target",
              "outcome"
            ],
            "properties": {
              "period": {
                "type": "integer",
                "minimum": 0
              },
              "date": {
                "type": "string"
              },
              "actor": {
                "type": "string",
                "description": "Who acted, qualified by kind: `event:<name>`, `waterfall:<name>`, `option:<name>`, `stream:<name>`. Qualified because a waterfall and an event may share a name and the log must not conflate them."
              },
              "action": {
                "type": "string",
                "enum": [
                  "set",
                  "activate_stream",
                  "deactivate_stream",
                  "activate_contract",
                  "deactivate_contract",
                  "exercise_option",
                  "pay",
                  "inflow",
                  "allocate_in",
                  "allocate_out",
                  "transition"
                ]
              },
              "target": {
                "type": "string",
                "description": "What was acted on — a field path, a stream name, or a step and its payee."
              },
              "outcome": {
                "type": "string",
                "enum": [
                  "applied",
                  "declined",
                  "overridden",
                  "ignored",
                  "failed"
                ],
                "description": "`applied` is the only one that changed anything. `declined` was refused for a stated reason. `overridden` was done and then lost to a stronger declaration — a stream activation against a false `active when`, or a waterfall step against a short pot. `ignored` is an action the engine does not execute yet. `failed` means the action's own expression did not evaluate."
              },
              "from": {
                "type": "string"
              },
              "to": {
                "type": "string"
              },
              "amount": {
                "type": "number",
                "description": "What the step allocated. Allocated, not transferred: a waterfall is an ordered allocation over a pot, deciding what each step is entitled to out of what remains. Whether that cash physically settles is a question the language does not model."
              },
              "pot_before": {
                "type": "number",
                "description": "The pot before the step drew on it, so a short pot is visible as the reason a step was allocated less than it was owed."
              },
              "pot_after": {
                "type": "number"
              },
              "note": {
                "type": "string",
                "description": "Why, when the outcome is not `applied`."
              }
            }
          }
        }
      }
    },
    "ScenariosSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "status",
        "summaries"
      ],
      "properties": {
        "status": {
          "type": "string",
          "enum": [
            "not_run",
            "ok",
            "error"
          ]
        },
        "summaries": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/ScenarioSummary"
          }
        },
        "errors": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/RuntimeError"
          }
        }
      }
    },
    "ScenarioSummary": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "metrics"
      ],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1
        },
        "metrics": {
          "$ref": "#/$defs/MetricMap"
        }
      }
    },
    "MonteCarloSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "status",
        "trials",
        "seed",
        "metrics",
        "trial_summaries"
      ],
      "properties": {
        "status": {
          "type": "string",
          "enum": [
            "not_run",
            "ok",
            "error"
          ]
        },
        "trials": {
          "type": "integer",
          "minimum": 1
        },
        "seed": {
          "type": "integer",
          "minimum": 0
        },
        "metrics": {
          "type": "object",
          "description": "Named Monte Carlo metric summaries",
          "additionalProperties": {
            "$ref": "#/$defs/MetricSummary"
          }
        },
        "trial_summaries": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/TrialSummary"
          }
        },
        "aggregates": {
          "$ref": "#/$defs/MonteCarloAggregates"
        },
        "errors": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/RuntimeError"
          }
        },
        "journal": {
          "type": "array",
          "description": "When each act happened across the trials, and how often — the question a stochastic run asks of the journal. A per-trial log is the wrong shape: trials x acts of output, and nobody reads ten thousand copies of the same sequence. So each distinct act gets one row, bounded by the model rather than the trial count, carrying the share of trials in which it occurred and the distribution over the period it FIRST did. Omitted when no trial recorded any act.",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [
              "actor",
              "action",
              "target",
              "outcome",
              "trials_occurred",
              "share",
              "first_period"
            ],
            "properties": {
              "actor": {
                "type": "string",
                "description": "The act's identity, matching the deterministic journal's own fields so a summary lines up against a single run's trail."
              },
              "action": {
                "type": "string"
              },
              "target": {
                "type": "string"
              },
              "outcome": {
                "type": "string"
              },
              "trials_occurred": {
                "type": "integer",
                "minimum": 0,
                "description": "Trials in which this act occurred at least once."
              },
              "share": {
                "type": "number",
                "minimum": 0,
                "maximum": 1
              },
              "first_period": {
                "type": "object",
                "additionalProperties": false,
                "required": [
                  "min",
                  "p10",
                  "median",
                  "p90",
                  "max",
                  "mean"
                ],
                "description": "Over the trials where the act occurred, the period it first did. Quantiles are nearest-rank order statistics rather than interpolated, because a quantile of periods should be a period: \"the covenant first broke around month 9\", not month 9.5. The mean stays fractional, being explicitly an average rather than an observation.",
                "properties": {
                  "min": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "p10": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "median": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "p90": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "max": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "mean": {
                    "type": "number",
                    "minimum": 0
                  }
                }
              }
            }
          }
        }
      }
    },
    "TrialSummary": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "trial",
        "metrics"
      ],
      "properties": {
        "trial": {
          "type": "integer",
          "minimum": 0
        },
        "metrics": {
          "$ref": "#/$defs/MetricMap"
        }
      }
    },
    "MonteCarloAggregates": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "npv"
      ],
      "properties": {
        "npv": {
          "$ref": "#/$defs/NpvAggregate"
        }
      }
    },
    "NpvAggregate": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "mean",
        "median",
        "stddev",
        "p_negative"
      ],
      "properties": {
        "mean": {
          "$ref": "#/$defs/Decimal"
        },
        "median": {
          "$ref": "#/$defs/Decimal"
        },
        "stddev": {
          "$ref": "#/$defs/Decimal"
        },
        "p_negative": {
          "$ref": "#/$defs/Decimal"
        }
      }
    },
    "MetricSummary": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "type",
        "mean",
        "p50"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "number",
            "money"
          ]
        },
        "mean": {
          "$ref": "#/$defs/Scalar"
        },
        "stdev": {
          "$ref": "#/$defs/Scalar"
        },
        "min": {
          "$ref": "#/$defs/Scalar"
        },
        "max": {
          "$ref": "#/$defs/Scalar"
        },
        "p01": {
          "$ref": "#/$defs/Scalar"
        },
        "p05": {
          "$ref": "#/$defs/Scalar"
        },
        "p10": {
          "$ref": "#/$defs/Scalar"
        },
        "p25": {
          "$ref": "#/$defs/Scalar"
        },
        "p50": {
          "$ref": "#/$defs/Scalar"
        },
        "p75": {
          "$ref": "#/$defs/Scalar"
        },
        "p90": {
          "$ref": "#/$defs/Scalar"
        },
        "p95": {
          "$ref": "#/$defs/Scalar"
        },
        "p99": {
          "$ref": "#/$defs/Scalar"
        }
      }
    },
    "RuntimeError": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "code",
        "message"
      ],
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1
        },
        "message": {
          "type": "string",
          "minLength": 1
        },
        "path": {
          "type": "string"
        },
        "hint": {
          "type": "string"
        }
      }
    },
    "MetricLineage": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "numerator_streams",
        "denominator_streams",
        "formula"
      ],
      "description": "Where a domain metric's value came from: the stream selectors it summed and the human-readable formula the pack declared. Emitted so a metric can be audited without reading the pack.",
      "properties": {
        "numerator_streams": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "denominator_streams": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "formula": {
          "type": "string"
        }
      }
    },
    "DomainMetrics": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "pack",
        "metrics",
        "lineage"
      ],
      "description": "Pack-defined metrics, present only when the run named a pack (`--pack <name>`). Engine-universal metrics live in `deterministic.metrics`; these are the domain's own, declared in the pack's metrics.toml.",
      "properties": {
        "pack": {
          "type": "string"
        },
        "metrics": {
          "$ref": "#/$defs/MetricMap"
        },
        "lineage": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/MetricLineage"
          }
        }
      }
    },
    "AnnualRollupSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "series"
      ],
      "description": "The deterministic series aggregated to annual buckets, for reporting a sub-annual model on a yearly grid. Present whenever the deterministic run succeeded. Carries no `offset`: an annual bucket sums periods whose placements differ.",
      "properties": {
        "series": {
          "$ref": "#/$defs/SeriesMap"
        }
      }
    },
    "InputsSection": {
      "type": "object",
      "additionalProperties": false,
      "description": "What went in, above the line items — the top of the audit chain. Absent when the model declares neither assumptions nor pack-lowered streams.",
      "properties": {
        "resolved": {
          "type": "object",
          "additionalProperties": {
            "type": "number"
          },
          "description": "Evaluated `assume` values, as `inputs.<name>` resolves them. In a deterministic run a random assumption resolves to its clipped CENTRAL value rather than to a draw; publishing it here is what stops that being invisible."
        },
        "streams": {
          "type": "array",
          "items": {
            "type": "object"
          },
          "description": "Per-stream record of the contract terms a pack rule consumed to strike it, passed through from the IR's `stream_inputs` verbatim. See the IR schema's StreamInputs. Hand-written streams have no entry, because no rule struck them."
        },
        "quantiles": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/QuantileCall"
          },
          "description": "Which slice of a declared quantile each expression asked for, and what it resolved to. Passed through from the IR's quantile_inputs verbatim. A nonlinear input whose evaluation is not published is a number no reviewer can check: the top 2% of hours averaging 340.00 is the fact that explains the revenue, and the declaration alone does not state it."
        }
      }
    },
    "StatementsSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "pack",
        "statements"
      ],
      "description": "Statements the active pack declares, rendered against this run. Rows carry order, labels, depth and a display sign; they compute nothing the engine has not already aggregated. Absent when the pack declares no statement.",
      "properties": {
        "pack": {
          "type": "string"
        },
        "statements": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/Statement"
          }
        }
      }
    },
    "Statement": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "label",
        "default",
        "grain",
        "rows",
        "reconciliation"
      ],
      "properties": {
        "id": {
          "type": "string"
        },
        "label": {
          "type": "string"
        },
        "default": {
          "type": "boolean"
        },
        "grain": {
          "$ref": "#/$defs/StatementGrain"
        },
        "rows": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/StatementRow"
          }
        },
        "reconciliation": {
          "$ref": "#/$defs/StatementReconciliation"
        },
        "diagnostics": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/StatementDiagnostic"
          },
          "description": "Completeness findings. Empty is the healthy case."
        }
      }
    },
    "StatementGrain": {
      "type": "object",
      "additionalProperties": false,
      "description": "The grain this statement reports at, and one ready-to-render label per column. Published because a consumer cannot derive it: an annual statement over a monthly model has ten values where the model has 120, and nothing else in the document says which ten periods those are.",
      "required": [
        "calendar",
        "start",
        "labels"
      ],
      "properties": {
        "calendar": {
          "type": "string",
          "description": "monthly | quarterly | annual | daily — the bucketing, not the model grid."
        },
        "start": {
          "type": "string"
        },
        "labels": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "One per column, aligned with every row's `values`."
        }
      }
    },
    "StatementRow": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind",
        "depth",
        "display_sign"
      ],
      "description": "One row. `residual` is emitted for cash no row claimed and cannot be authored; a `spacer` carries no values.",
      "properties": {
        "kind": {
          "enum": [
            "line",
            "subtotal",
            "ratio",
            "spacer",
            "residual"
          ]
        },
        "label": {
          "type": "string"
        },
        "depth": {
          "type": "integer",
          "minimum": 0
        },
        "display_sign": {
          "type": "number",
          "enum": [
            1,
            -1
          ],
          "description": "How to RENDER the sign. `values` is always the signed arithmetic quantity, so a consumer that ignores this still adds up correctly. -1 is how a deduction prints as a positive number in a 'less:' row while still being counted negatively — a line can be shown AND counted."
        },
        "values": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/SeriesValue"
          }
        },
        "total": {
          "type": "number",
          "description": "Lifetime total. Absent for a ratio, where summing a column of ratios answers nothing, and for a spacer."
        },
        "streams": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "The streams this row drew from — what makes a published figure traceable without a flow ledger."
        }
      }
    },
    "StatementReconciliation": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "bottom_line",
        "model_total",
        "residual"
      ],
      "description": "Does the statement account for the model's cash? Published always and asserted rather than corrected: a bottom line that quietly differs from model.total is the failure this exists to make visible.",
      "properties": {
        "bottom_line": {
          "type": "number"
        },
        "model_total": {
          "type": "number"
        },
        "residual": {
          "type": "number"
        }
      }
    },
    "StatementDiagnostic": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "code",
        "message"
      ],
      "properties": {
        "code": {
          "type": "string"
        },
        "message": {
          "type": "string"
        }
      }
    },
    "SeriesValue": {
      "description": "A series point: money, a bare number, or null where undefined."
    },
    "QuantileCall": {
      "type": "object",
      "required": [
        "quantile",
        "function",
        "args"
      ],
      "additionalProperties": false,
      "properties": {
        "quantile": {
          "type": "string",
          "description": "The quantile named at the call site."
        },
        "function": {
          "type": "string",
          "enum": [
            "quantile_at",
            "quantile_mean",
            "quantile_of"
          ]
        },
        "args": {
          "type": "array",
          "items": {
            "type": "number"
          },
          "description": "The literal arguments after the name, in source order. Empty when they were not literals."
        },
        "value": {
          "type": "number",
          "description": "What the call resolves to, rounded to the engine's published-number policy so it agrees exactly with the ledger figure it explains. ABSENT when an argument is not a literal — the call is still listed, because a silently omitted call site would read as a model that never made one."
        }
      }
    }
  }
}
```


---

# Pack interface

# Pack Interface

**CFDL Domain Pack Interface v0.1**

Domain packs provide *additions and overrides* on top of a single core language: each pack adds contract types, lowering rules, metrics, and validations without forking the language itself.

Core principle: **Packs may extend validation and provide defaults/templates, but MUST NOT change core language semantics.**

## Normative keywords

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in BCP 14 ([RFC 2119](https://www.rfc-editor.org/rfc/rfc2119), [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174)) when, and only when, they appear in all capitals.

This document is the contract a pack author writes against. That is the reason the distinction is stated rather than assumed: a pack author has to be able to tell a requirement from advice without inferring it from the surrounding sentence.

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
  recognized keys are `aliases`, `templates`, `lowering`, `metrics` and
  `validations`, each a path relative to the pack directory. An unrecognized
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

**Lifecycles.** A type MAY declare a lifecycle — the same finite state
machine a model declares with a `lifecycle` block (`docs/28` §6.1): the core
has the full functionality, and a pack tailors it to its domain. In
`types.toml`:

```toml
[[lifecycles]]
lifecycle_id = "cre.unit"
initial = "vacant"
states = ["vacant", "leased", "downtime"]

[[lifecycles.transitions]]
from = "leased"
to = "downtime"
guard = 'series_sum("cre.rent", time.t - 1, time.t - 1) < 50'
```

- A transition without `guard` is a **permission**: an event's write may
  take it, and the machine never fires it on its own. Every edge shipped
  before guards existed is this kind, so no pack changed meaning.
- A `guard` makes the edge self-driving, evaluated each period an entity of
  the type is in `from` — reading series strictly backward, the same rule a
  model-declared edge's `when` follows, checked at compile.
- A lifecycle that declares no transitions at all is unconstrained.
- An entity whose type declares a lifecycle MUST NOT also bind a
  model-declared one (`E1350`): one machine per entity.

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

### Contract design: decomposition and terms

A contract is a helper: the modeller follows its shape and provides the terms,
and the streams emerge. These rules are what make that shape trustworthy. Each
exists because its violation shipped and cost something.

**One stream per economically distinct line.** A contract MUST lower to one
stream per line an operating or financing statement would show — interest and
principal separately, gross proceeds and selling costs separately. Netting is
presentation, and belongs to statements; lowering is data. A netted line
cannot be un-netted downstream, so every consumer of the split — a tax line, a
coverage ratio, an amortization schedule — is foreclosed at the rule. The
worst form is cash the model never sees at all: a mortgage rule that emits
debt service but not the loan proceeds funds nothing, and the model's levered
return is wrong even while the net line reconciles.

**Two shapes, chosen by the domain.** An *instrument* lowers one contract into
its components — a loan pool into interest, principal, prepayments,
recoveries, servicing; a lease into rent, abatement, recoveries, TI/LC. A
*line item* is one instanced contract per statement line — an operating
expense schedule — with the vocabulary carried by the instance name
(`cre.opex_line.property_tax`), never by an enum term restating it.

**Grain by instancing; level by entity.** The modeller chooses grain by how
many instances they declare, and level by the entity each contract hangs on —
`part_of` rolls it up. A rule MUST NOT take a `level` or `scope` term: that
restates the entity tree in a place that can disagree with it.

**Terms carry the agreement; rules carry the instrument.** A term may hold an
expression, so a pack SHOULD NOT pre-bake value shapes the modeller can state
directly — an escalator is `escalation = curve_value("cpi", time.date) + 0.005`,
not a `rate`/`curve` twin-term pair with a selector spliced into the rule.
What belongs in the rule is the instrument's own mechanics: amortization
arithmetic, the split of a payment into interest and principal, the schedule.
What belongs in the term is what the parties agreed.

**Pack streams are named `[domain].[category].[line]{.[instance]}`.** A
contract's several streams share their `[domain].[category]` — `cre.unit.`
carries `base_rent`, `abatement`, `recoveries` and `ti_lc`; `opco.debt.`
carries `proceeds`, `interest` and `principal`. A line-item contract puts the
line in the instance slot: `cre.opex.line.property_tax`. Contract TYPES may
use underscores (`cre.lease_unit`, `cre.opex_line`) — they are authoring
surface — but the streams a rule emits may not. This applies to PACK-LOWERED
streams only: a hand-written stream is the modeller's own name and no pattern
is enforced on it.

**Categories are the semantics; names are addresses.** Every stream a rule
emits MUST carry a category, and aggregation reads the category — which is why
decomposition never moves a total: the components fold where the netted line
folded. A statement itemises by selecting streams; a subtotal folds by
category; both stay correct as the grain changes.

**Every read of an instanceable family is globbed.** A rule whose stream name
carries `{{contract.dot_suffix}}` emits per instance, and any `series_sum`,
metric selector, or statement row reading that family MUST use `<base>.*` —
a bare name matches only the unsuffixed instance and silently drops every
sibling. `tools/check-pack-series.py` gates this; `# series-allow:` marks the
rare deliberate bare read.

**The conventional vocabulary ships as templates.** `templates.toml` carries
the standard set — the nine expense lines a statement usually shows, with
sensible defaults — as editor snippets. The modeller starts from the
convention and is free to name their own instance; anything unclaimed lands on
the statement's residual row rather than vanishing.

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
An unrecognized value is `E5012_RULE_INVALID_INTERVAL`. An interval finer than
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
the period's open. Set `schedule_placement = "end"` for a **disposal**: a
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
and a rule states at most one, as `schedule_placement = "start" | "mid" | "end"`.

Use it for operating flows. Do not use it for a **price**: a disposal, a
terminal value or an acquisition is struck at a point in time and discounts
whole, which is what `schedule_placement` is for.

### A rule may declare a field

A rule that compounds a rate which MOVES cannot use `pow(1 + g, t)` — that
applies one period's rate as though it had held from the start, which is exact
while the rate is flat and wrong the moment it varies. Three optional keys let
the rule declare a recurrence instead:

```toml
amount_expr = "{{contract.amount}} * field.opco_revenue_growth{{contract.suffix_ident}}"
field_name  = "opco_revenue_growth{{contract.suffix_ident}}"
field_init  = "1"
field_next  = "prev * pow(1 + curve_value(\"{{contract.growth_curve}}\", time.date), 1 / {{model.periods_per_year}})"
```

The field is attached to the contract's subject entity, because that is what the
value is a fact about. `field.<name>` is the rule's own placeholder for it: a
rule cannot know which entity it will be attached to, so it names the field it
declares and lowering rewrites that to the entity path a model would write.

A field a contract brings inherits the contract's own clock:

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

The field steps on its **accrual** periods and **holds** between ticks and
outside `field_from`/`field_to`. It does not fall to zero: that is what
separates a cadence from `active when`, which a field does not have. An interval
finer than the model calendar is `E2108_SCHEDULE_FINER_THAN_CALENDAR`, the same
rule a stream obeys.

All templated. `field_name` must expand to a **single identifier** —
`field.<name>` resolves one segment, so `{{contract.dot_suffix}}` would produce
an unreachable path; use `{{contract.suffix_ident}}`.

**A rule may read BOTH ENDS of the field it declares.** `field.<name>` is the
value at this period and `prev.field.<name>` is the value at the previous
close, which is what an average-balance accrual needs:

```toml
amount_expr = "(prev.field.loan_balance + field.loan_balance) / 2 * {{contract.rate}}"
```

Both spellings lower through the entity root, since the bare family alias
covers the four declared families only and a rule may sit on any entity. A
stream reading `prev` must not start in the model's FIRST period — there is no
close before it (`E1129`). `fixtures/valid/pack_rule_reads_prev_field` pins it.

What a rule may NOT do is read a stream from `field_next`: a recurrence sees
`prev`, other fields' previous values, `time.*`, `inputs.*`, `cfg`, `obs` and
curves, and no series at all. That absence is what makes cycles impossible by
construction, so a balance cannot be defined as "last period plus this period's
draw stream". Derive it instead — a cumulative field over the schedule, with
the period's split computed from it. `benchmarks/cre/one_lincoln_street` is the
worked case: one field, and equity draw, debt draw and opening balance all fall
out of `min`/`max` over it.

Fields are deduplicated by name across contracts. Identical definitions collapse,
which is what several contracts sharing one curve should do; differing ones are
`E5021_DUPLICATE_LOWERED_FIELD` rather than one silently winning.

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
the placeholder without changing any existing model. An unrecognized value is
`E5019_UNKNOWN_DAY_COUNT` rather than a silent fallback: act/360 against
act/365 is about 1.4% of interest.

Use it for every **nominal** rate — note rates, servicing strips, floating
index-plus-margin. Do not use it for annual *quantities* (`rent_year`,
`om_year`), which spread by `{{model.periods_per_year}}` regardless of day
count.

**Amortization is a second, separate basis.** An amortizing loan strikes its
level payment once, from a schedule the parties agree, and then accrues interest
period by period on whatever the accrual convention says; principal is the plug.
Those are two different divisors, and collapsing them makes the payment itself
move with month length — which no amortizing instrument does.
`{{model.amortization_divisor}}` reads an `amortization_day_count` term and
expands by the same table as `{{model.accrual_divisor}}`, **defaulting to
`day_count`** when absent. So:

- a rule that uses only `{{model.accrual_divisor}}` is unchanged;
- a model that sets only `day_count` is unchanged, both divisors agreeing;
- `day_count = "act/360"` with `amortization_day_count = "30/360"` is the
  common US commercial case — a fixed payment, interest varying by month length.

An amortizing rule should therefore strike the annuity factor from
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
numerator_streams = ["cre.lease.base_rent", "cre.revenue.line.*"]
denominator_streams = ["cre.opex.line"]
formula = "sum(numerator_streams) + sum(denominator_streams)"  # lineage text

[[metrics]]
id = "domain.cre.debt_service"
kind = "money"
op = "negated_sum"
numerator_streams = ["cre.debt.interest.*", "cre.debt.principal.*"]
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


---

# Diagnostics

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
  rule, or an option's election or payoff calls `series_sum`/`series_avg`. All
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
- `E1301_UNRESOLVED_ENTITY_REF` — a stream, contract or event action names an entity that is not declared.
- `E1340_WATERFALL_NO_SOURCE` — a waterfall declares no `from`, so there is no
  pot to allocate.
- `E1341_WATERFALL_FORWARD_REF` — a step's `paid.<step>` names a step declared
  later in the same waterfall. Steps pay in declaration order, so a later step
  has not paid anything when an earlier one is evaluated.
- `E1342_WATERFALL_SERIES_NOT_VISIBLE` — `series_sum`/`series_avg` names a step
  of this waterfall or of a later one. Steps publish when their waterfall
  finishes, so the read would aggregate to zero and say nothing. An EARLIER
  waterfall is the documented composition and still compiles.
- `E1349_UNRESOLVED_LIFECYCLE_REF` — an entity binds `lifecycle <name>` and no
  lifecycle block declares it.
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
- `E1346_STREAM_READS_WATERFALL_STEP` — a STREAM's `series_sum`/`series_avg`
  names a waterfall step. Every waterfall runs after every stream and a step's
  series is visible to a later waterfall's `from` and to nothing else, so the
  read could only ever aggregate to zero. Model the quantity the step pays as a
  stream or a field if a stream must read it.
- `E1302_UNRESOLVED_STREAM_REF` — an event activates or deactivates a stream the model does not run. Event action targets were never resolved, so a misspelling matched nothing and the action was silently inert: the stream it was meant to stop kept paying, with no diagnostic and no warning. Checked after lowering rather than in the resolver, so a name a CONTRACT produced resolves as readily as one the model declared — the symbol table is built before the pack is chosen, and a check running there reported an unlowered name and a typo alike. The hint lists every stream in the model, both kinds.
- `E1303_UNRESOLVED_CONTRACT_REF` — an event activates or deactivates a contract that is not declared.
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
- `E1322_UNKNOWN_PARTY_ROLE` — a role is bound that the contract type does not declare. The declared roles are listed; a role belongs to the agreement, not to the entity.
- `E1302_UNRESOLVED_STREAM_REF` — something names a stream that is not declared — often an event deactivating one.
- `E1303_UNRESOLVED_CONTRACT_REF` — something names a contract that is not declared.
- `E1304_UNRESOLVED_OPTION_REF` — an event exercises an option that is not declared.
- `E1305_UNRESOLVED_PHASE_REF` — a schedule names a phase that is not declared.
- `E1306_INVALID_ENTITY_REF_FORMAT` — entity ref, stream name, or contract name is not a qualified name with at least two segments (dotted hierarchy).

### 7.5 Contracts and streams (E20xx/E21xx)
- `E2001_CONTRACT_MISSING_TERM` — a contract omits a term its pack requires. The message names it; see the pack's contract table.
- `E2002_CONTRACT_MISSING_EFFECTS` — a contract produces no streams, so it has no effect on the model.
- `E2003_CONTRACT_CURRENCY_REQUIRED` — a contract does not state its currency and none can be inferred.
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
- `E2203_ACTION_SET_FIELD_INVALID` — an event sets an entity field that does not exist or cannot hold that value.
### 7.7 Expressions / typing (E30xx/W30xx)
- `E3001_EXPR_PARSE_ERROR` — an expression is not valid CFDL.
- `E3002_EXPR_UNKNOWN_IDENT` — an expression names something not in scope. Bindings are `time.*`, `inputs.*`, `model.*`, `entity.*`, `cfg.*`, `obs.*` and entity fields by qualified path (`<family>.<entity>.<field>`).
- `E3003_EXPR_TYPE_ERROR` — an expression combines types that cannot combine, such as a date and a number.
- `E3004_EXPR_ILLEGAL_OP` — an operator is not defined for these operands.
Warnings:
- `W3001_EXPR_TYPE_UNKNOWN` — an expression's type could not be determined ahead of evaluation. It still runs; the warning notes the check was skipped.
- `W3002_OBS_REF_EXTRACTION_FAILED` — an observation reference could not be read out of an expression, so the run may not know it needs that input.
### 7.8 Pack errors (E4xxx)
- `E4001_UNKNOWN_TYPE_ID` — a declaration names a type the active pack does not define.
- `E4002_INVALID_ENTITY_ATTR` — an entity field is not one the pack declares, or holds the wrong kind of value.
- `E4003_INVALID_CONTRACT_TERMS` — a contract's terms do not satisfy the pack's schema for that contract.
- `E4004_MISSING_PACK` — the named pack could not be loaded — not found, or found and rejected.
### 7.9 Lowering/emission (E5xxx)
- `E5001_ID_GENERATION_FAILED` — the compiler could not derive a stable identifier for a declaration.
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

- `W5022_UNKNOWN_SERIES_REFERENCE` — a `series_sum`/`series_avg` names a series
  no stream, contract or waterfall step produces, so it aggregates to zero and
  whatever reads it is reading nothing. A warning rather than an error because a
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
  `model.total` within half a cent. Asserted, never corrected.

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
- `E6021_CRE_OPS_INVALID_SCHEDULE`
- `E6031_CRE_UNIT_INVALID_FREE_RENT` — `free_rent_months` is a whole number of
  months, 0 or more
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
- `E6054_CRE_DEBT_INVALID_AMORT` — `amort_months` strikes the payment and is
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

`E6020_CRE_OPS_MISSING_AMOUNT` and `E6021_CRE_OPS_INVALID_SCHEDULE` are
**retired** with `cre.ops_revenue`: the line item takes either amount term
(E6064), and a revenue term legitimately reaches the projection tail so that
forward NOI has revenue to read. Per §8 the codes are never reused.

`E6004_CRE_LEASE_UP_INVALID_OCCUPANCY` is **retired**: it validated lease-up
occupancy terms that no longer exist. Per §8 the code is never reused.

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


---

# Controlled English — the authoring contract

_The register an authoring agent should emit. The mechanical subset is enforced by tools/check-site-voice.py; the terminology below is the word list._

# 22 — CFDL Controlled English (CFDL-CE)

The writing standard for everything published on cfdl.dev and learn.cfdl.dev.

CFDL-CE is **derived from ASD-STE100 Simplified Technical English, Issue 9**. It
adopts STE's writing rules, tiered by content type. It does **not** adopt STE's
approved-word dictionary, and therefore **does not claim ASD-STE100
conformance** — see §6. The evidence behind each rule is in
`21_documentation_standards_audit.md`; the approved forms are in
`terminology.toml`.

Status: adopted as the standard. The mechanical subset (spellings, synonyms, contractions, number formats) is enforced by `tools/check-site-voice.py` (`make site-voice`), which also validates `ste-allow:` rule ids against §3; the tier mapping in §2 is not yet machine-checked (backlog 7.82).

---

## 1. Why a derived standard and not STE itself

STE was written for aircraft maintenance procedures read under time pressure,
often by non-native speakers, where a misread instruction is a safety event. Its
rules for instructions are excellent and transfer directly. Its dictionary —
about 950 words, each with one approved meaning and one approved part of speech —
excludes nearly the whole vocabulary this documentation exists to convey.
`amortization`, `waterfall`, `covenant`, `lowering`, and `span` are not approved
words, and paraphrasing them would make normative documents less precise, not
more.

STE anticipates this and provides Technical Names and Technical Verbs as the
escape. At this vocabulary's scale, using that escape produces a
CFDL-specific controlled language rather than STE. CFDL-CE says so plainly
instead of overclaiming.

---

## 2. Tiers

Every published file belongs to exactly one tier. Tiers are path globs so they
stay mechanically checkable; there is currently no frontmatter field that
distinguishes a tutorial from a reference page.

| Tier | Content | Paths |
|---|---|---|
| **A** | Procedural — the reader is doing something now | `site/content/docs/install/**`, `site/content/docs/getting-started.md`, `site/content/docs/troubleshooting.md`, `training/exercises/*/*/README.md`, `examples/language_tutorial/*/README.md` |
| **B** | Reference and normative | `site/content/docs/reference/**`, `site/content/docs/specification/**`, `docs/0*.md`, `docs/1[2457]_*.md`, JSON schema `description` strings |
| **C** | Conceptual and instructional | `site/content/docs/{concepts,object-model,language-guide,stochastic-modeling,faq,benchmarks}.md`, `site/content/docs/guides/**`, `site/content/docs/packs/**`, `learn/content/chapters/*.mdx`, `benchmarks/*/*/CASE.md`, `site/app/private/*/content.html` |
| **D** | Marketing | `site/app/page.tsx`, `site/components/SiteFooter.tsx`, playground microcopy |

A page edited at its source inherits the tier of the page it generates. The
specification pages under `site/content/docs/specification/` are byte-copies of
`docs/0*.md`; both are Tier B, and the edit goes to `docs/`.

**Tier A is the tier that matters.** It is where STE was designed to operate,
where the audit found the worst violations, and where conformance is
non-negotiable. Tiers C and D exist mostly to record what is deliberately *not*
constrained, so that a future gate does not fire on prose that is correct.

---

The private case pages are Tier C. They explain a model to a reader outside the
project, which is instructional work, and the reader is often the counterparty
whose deal is being modeled. Marketing licence does not apply to them: an
idiom, a personified model, or a sentence about what the language cannot do all
cost more there than anywhere else on the site.

## 3. Rules

`•` applies · `—` does not apply. Where a tier relaxes a rule, the reason is
stated; an unexplained divergence is a defect in this document.

### 3.1 Sentences

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| S1 | Maximum 20 words in an instruction | • | • | • | — |
| S2 | Maximum 25 words in descriptive text | • | • | • | — |
| S3 | Maximum 6 sentences in a paragraph | • | • | • | • |
| S4 | One instruction per sentence | • | • | • | — |
| S5 | Use a vertical list for more than one action | • | • | • | — |

S1–S2, Tier C: this is a target, not a gate threshold. Long sentences in the
curriculum are often long in order to hold two ideas in tension, and splitting
one of those costs comprehension rather than buying it. Write shorter where
shorter is clearer. Do not split a sentence whose halves mean less apart.

The 2026-08-25 chapter pass (register item 12) settled where that defence stops.
It holds for a sentence in the 25-to-35-word band, which is usually one thought
with a qualifier attached. It does not hold past roughly 35 words, where the
sentence is a pileup of three clauses rather than a pair of ideas — a claim, a
parenthetical, and an em-dash aside, stacked. Treat 35 words as the point where
the burden shifts: below it, keep the sentence unless shorter is clearer; above
it, split it unless you can say what the halves lose.

One rewrite to reach for first. A long sentence that lists things — bases,
shapes, parts, options, separated by commas or semicolons — is an enumeration
wearing prose clothes, and S5's vertical list is both shorter and more
scannable. That accounted for six of the pass's rewrites.

### 3.2 Voice and verbs

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| V1 | Use the active voice | • | • | • | • |
| V2 | Passive only where the actor is genuinely unknown or irrelevant | — | • | • | • |
| V3 | Instructions are imperative | • | • | • | — |
| V4 | No `-ing` form as a modifier or verb form | • | • | • | — |
| V5 | Use simple present or simple past; one tense per passage | • | • | • | • |
| V6 | No contractions | • | • | • | • |

V2, Tier A: passive is not permitted in an instruction at all. An instruction
without an actor is an instruction the reader cannot follow.

V4: `-ing` remains correct as a technical noun (`lowering`, `underwriting`,
`netting`) — those are registered in `terminology.toml`. The banned form is the
participial modifier: write "Use the CLI to compile the model", not "Using the
CLI, compile the model".

V6: the audit found 9 contractions in 70,438 words. This rule records existing
practice rather than demanding a change. The measured corpora now hold none, and
the two `can't`s the audit named in TSX are both fixed — `site/app/page.tsx`, and
`learn/app/page.tsx`, which the audit never measured (2026-08-25).

TSX microcopy remains the estate's blind spot. `tools/check-site-voice.py` reads
Markdown and MDX sources, so prose hardcoded in a component is checked by nobody;
that is how a second `can't` survived the audit that reported the first. Anything
user-facing written in a `.tsx` file is Tier D at minimum and still bound by W1–W3
and V6.

### 3.3 Words

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| W1 | One word, one form — use the spelling in `terminology.toml` | • | • | • | • |
| W2 | One concept, one term — no synonyms for a defined thing | • | • | • | • |
| W3 | Use the approved verb for an action (`click`, not `hit` or `press`) | • | • | • | • |
| W4 | Multi-word domain terms must be registered as Technical Names | • | • | • | • |
| W5 | Noun clusters: maximum 3 words unless registered under W4 | • | • | • | — |
| W6 | No marketing ornament | • | • | • | • |
| W7 | Define a term on first use, or link to the glossary | — | • | • | — |

W1–W3 are the rules the audit found broken most consistently and are the
cheapest to enforce, because they reduce to a word list.

W6 is already enforced by `tools/check-site-voice.py`; `terminology.toml` seeds
its `[[not_approved]]` section from that same list so the two cannot drift.

### 3.4 Clarity

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| C1 | No ambiguous `it`, `this`, `that` — repeat the noun | • | • | • | — |
| C2 | Use articles; no telegraphic style | • | • | • | — |
| C3 | Do not carry meaning in italics or bold alone | • | • | — | — |
| C4 | No metaphor as the primary explanation | • | • | — | — |
| C5 | Keep related words together; no long subordinate chains | • | • | • | — |

**C3 and C4 are the deliberate Tier C relaxations, and they are the most
important entries in this document.**

The curriculum's method is to build a concept before naming it, and it does that
with metaphor and semantic stress. Chapter 1 opens "Every financial model is a
claim: *if these assumptions hold, this is the cash*" — the italics carry the
meaning and STE bans the construction. It is also the thesis sentence of the
course. Chapter 2 calls time "the spine", entities "the cast", and streams "the
atoms".

These are permitted in Tier C on two conditions:

1. The metaphor is registered in `terminology.toml` under `[[pedagogical]]`,
   with the plain term it stands for.
2. The plain term appears too. A metaphor introduces a concept; it does not
   replace its name.

Registration is the point. It converts a stylistic habit into a recorded
decision that a reviewer can audit and a future writer can find.

### 3.5 Procedures

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| P1 | A step is an imperative naming one action | • | • | • | — |
| P2 | A step is not a question or a claim | • | • | • | — |
| P3 | State the condition before the action | • | • | • | — |
| P4 | State the expected result after an action with a visible outcome | • | — | • | — |

P1–P2, Tier C: this is the audit's §4.6 finding. The training chapters write
procedures as descriptions of a discipline — "The discipline for challenging any
figure in the output…", followed by numbered steps that are questions
("**Which streams feed it?**"). A reader following along must convert each into
an action.

The rule for Tier C: **the prose around a procedure may explain; the numbered
steps must instruct.** Keep the framing sentence. Make step 1 "Identify the
streams that feed the figure", and let the explanation follow it.

---

## 4. Conventions STE does not cover

STE has no rules for a software documentation site. These are CFDL's own.

**Code in prose.** Set identifiers, file names, commands, and diagnostic codes
in backticks. A backticked identifier is one lexical item — it does not count
toward a noun cluster or a sentence-length limit. Do not inflect an identifier
to fit a sentence: write "the `stream` construct", not "the streams construct".

**Diagnostic codes.** Cite as `` `E0123` `` and link to the diagnostics
reference on first use in a page.

**Normative keywords.** Tier B only. Use MUST, MUST NOT, SHOULD, SHOULD NOT, MAY
with their RFC 2119 / BCP 14 meanings, and cite BCP 14 in the document that uses
them. The audit found 143 such keywords across three specifications and no
citation anywhere. Do not use these words in upper case in Tier A, C, or D,
where they carry no normative force and only look like they do.

**Headings.** Sentence case, matching the page's frontmatter `title`. This is
already the convention, recorded in the header comment of
`site/content/nav.ts`.

**Numbers, currency, units.**

- Write multiplication as `8.0x`, not `8.0×` (U+00D7). Four files under
  `benchmarks/*/*/` currently use the multiplication sign, and they are
  published.
- Write `$33.6m`, not `$33.6mm`. `mm` reads as millimetres and is not a standard
  abbreviation for millions outside a trading desk. 37 benchmark source files
  currently use it.
- Use a plain hyphen (U+002D). `docs/01_language_spec.md` contains one
  non-breaking hyphen (U+2011) inherited from an earlier editor. One is enough to
  make the rule worth stating: it is invisible in review and it breaks search.
- Dates in ISO 8601 (`2026-08-14`).

**Frontmatter.** Every published page carries `title` and `description`. The
`learn` chapters already do this; no `site` doc page does. The description is one
sentence and is what a search result shows.

---

## 5. Conformance and the escape hatch

Three levels, so a page can be honest about where it stands:

- **Conforming** — meets every rule for its tier.
- **Conforming with exceptions** — carries `ste-allow:` annotations, each with a
  reason.
- **Not assessed** — the default for anything not yet reviewed.

The reserved annotation form mirrors the existing `site-allow:` convention in
`tools/check-site-voice.py`:

```
ste-allow: <rule id> <reason>
```

For example: `ste-allow: S2 the split sentence loses the causal link`.

A reason is required. An annotation without one is a defect, because the value
of the escape hatch is the record it leaves, not the suppression it performs.

**The mechanical subset is enforced.** `tools/check-site-voice.py` checks every
site-facing source — the specifications included — for retired spellings (W1,
from the register's `[spelling.map]`), retired synonyms (W2), `hit` aimed at a
control (W3), the number formats of §4, and contractions (V6). Word lists load
from `terminology.toml` at run time, so the gate and the register cannot drift,
and `ste-allow: <rule id> <reason>` waives a line.

**What the gate deliberately does not check:** sentence length (S1–S2), voice
(V1–V2), imperative form (P1–P2), and everything else that requires judgment.
Those live in review against this document, not in a regex — a gate that flags
judgment gets disabled, and then it checks nothing.

---

## 6. Relationship to other standards

| Standard | Status |
|---|---|
| **ASD-STE100 Issue 9** | Writing rules adopted and tiered. Dictionary not adopted. **Conformance is not claimed.** |
| **RFC 2119 / BCP 14** | Adopted for Tier B. |
| **ISO/IEC/IEEE 26514:2022**, **IEC/IEEE 82079-1:2019** | Adopted as the frame for structure, completeness, and findability — glossary, descriptions, document types. |
| **WCAG 2.2 AA / EN 301 549** | Applies to the sites as software. Not yet assessed; no conformance claimed. |

Rule numbers from ASD-STE100 are deliberately not cited in this document. The
official Issue 9 copy has not been obtained, and citing a rule number that turns
out to be wrong is worse than describing the rule. Obtain the copy — it is free
on request from asd-ste100.org — and add the numbers then.


---

# Glossary

# Glossary

Every term CFDL uses in a specific sense, with the one meaning it carries.

Terms are listed with one definition each. Where a term is an abbreviation,
the expansion given is the form to use on first mention.

## Language

The constructs a model is written from.

**account**

A declared cash location whose balance accumulates across periods under the balance law; drawn by a waterfall, credited by a step, read by logic as settled history.

**assumption**

A declared input to the model, constant or stochastic. Declared with `assume`.

**config namespace**

The `cfg.<path>` expression binding, which reads scenario knobs from the run configuration. Distinct from the run configuration itself.

**contract**

A first-class agreement carrying terms and effects.

**curve**

A named series of values indexed by time.

**entity**

A thing the model is about. An asset produces or consumes cash; a party contracts, owns, or lends.

**event**

A construct that fires on a condition and changes model state.

**fact field**

A field on an entity that takes a literal and nothing else. A value that moves must be a rule field.

**grain**

How finely the model slices time and things. Timeline grain and entity grain are independent choices.

**lifecycle**

A declared finite state machine: enumerated states, an initial one, and guarded edges declared only as used. A core-language construct that packs tailor to domains.

**metric**

A derived summary figure over the model's cash flows.

**option**

An electable right held by a party.

**pack**

A domain vocabulary supplying types, roles, and terms to a model.

**priced amount**

A stream amount whose series window reaches forward: a valuation setting a causal amount, evaluated after the causal cells settle, refused where the graph cycles.

**quantile**

A named series of values indexed by cumulative share.

**rule field**

A field carrying a recurrence: `init` gives the first period's value, `next` is evaluated each later period with `prev` bound to the field's own previous value.

**scenario**

A named set of overrides applied to a run.

**schedule**

The set of dates on which a stream pays.

**statement**

A per-period report published from the model's results.

**stream**

A dated, directed movement of cash attached to an entity and laid out on the timeline.

**waterfall**

An ordered distribution of cash through steps by seniority.

## Compiler

What the toolchain does with a model, and what it reports.

**diagnostic**

A compiler message carrying a stable code and a source span.

**IR** — intermediate representation (IR)

The canonical JSON intermediate representation a model compiles to.

**lowering**

The compiler stage that reduces language constructs to the canonical IR. A technical noun, so exempt from the rule against -ing forms.

**span**

The region of source a diagnostic points at.

## Finance

Domain terms the packs and the documentation use in their standard sense.

**cap rate**

The capitalization rate applied to stabilized income to derive value.

**catch-up**

The waterfall tier that restores a party to its target share after a preferred return is paid.

**covenant**

A contractual test the borrower must satisfy.

**DSCR** — debt service coverage ratio (DSCR)

Debt service coverage ratio.

**EBITDA**

Earnings before interest, taxes, depreciation, and amortization.

**expense stop**

The expense level above which recoveries are billed to a tenant.

**IRR** — internal rate of return (IRR)

Internal rate of return: the discount rate at which NPV is zero. Undefined for a series that never changes sign.

**lease-up**

The period during which vacant space is let to stabilized occupancy.

**MOIC** — multiple of invested capital (MOIC)

Multiple of invested capital.

**NPV** — net present value (NPV)

Net present value.

**PPA** — power purchase agreement (PPA)

Power purchase agreement.

**promote**

The sponsor's disproportionate share of profit above a return hurdle.

**reversion**

The terminal value realized at the end of a hold.

**takeout**

Permanent financing that repays construction debt.

**term power purchase agreement**

A power offtake contract for a fixed term.

**weighted average life**

The average time to principal return, weighted by principal repaid.

**working capital**

Receivables, payables, and inventory, modeled on a days-based policy.

## Verbs

Each of these describes one action and is used for no other.

**accrue**

To recognize an amount in a period without paying it.

**compile**

To translate a model into the canonical IR.

**diff**

To compare two artefacts and report the differences.

**discount**

To apply a discount rate to a future cash flow.

**escalate**

To increase an amount on a stated schedule.

**evaluate**

To compute a value for a period.

**lower**

To reduce a language construct to its IR form.

**resolve**

To bind a name to its declaration.

**seed**

To fix the random draw so a stochastic run reproduces.


---

# Terminology register

```toml
# CFDL controlled terminology register
#
# The single source for approved word forms, Technical Names, and Technical
# Verbs. The rules that use it are in 22_cfdl_controlled_english.md; the
# evidence for each entry is in 21_documentation_standards_audit.md.
#
# WHY THIS FILE EXISTS. ASD-STE100's central rule is one word, one form, one
# meaning. The audit found the same words spelled two ways and one object
# carrying three names, because nothing recorded a decision. This file is that
# record.
#
# It has a second job. STE caps a noun cluster at three words, and much of this
# domain's vocabulary is longer than that — a "term power purchase agreement" is
# what the instrument is called. STE's own answer is to register such phrases as
# Technical Names, after which the cap does not apply. Registration is therefore
# not bookkeeping: it is what keeps a correct term from being flagged and
# rewritten into a wrong one.
#
# TOML because tools/ already parses it with tomllib (see
# tools/check-pack-validations.py and tools/benchmark-runner.py), so a future
# gate reads this with no new dependency.
#
# `occurrences` is the count in published prose — site/content/docs/**/*.md,
# learn/content/chapters/*.mdx, and training/exercises/*/*/README.md, with code
# fences and frontmatter removed — at the time of the audit. It is provenance,
# not a target, and it goes stale. Re-measure before relying on it.

schema_version = 1
derived_from = "ASD-STE100 Issue 9"
conformance_claimed = false

# ---------------------------------------------------------------------------
# Preferred forms. Each entry resolves a conflict the corpus actually contains.
# ---------------------------------------------------------------------------

# ---------------------------------------------------------------------------
# Spelling: US throughout.
#
# The corpus was not split along a clean national line — `behavior` and
# `modeling` leaned American while `amortisation` and `amortising` leaned
# British — so no convention was overturned. There was no convention. US was
# chosen because the larger share of the corpus and the primary readership
# already used it.
#
# APPLIED, IN TWO ROUNDS. 431 replacements across 41 forms in published content
# and its generating sources, then 106 more across every remaining tracked file:
# Rust and TypeScript comments, the benchmark reference generators, the tool
# scripts, the fixtures, CHANGELOG.md and the research notes. **Zero UK forms
# remain in any tracked file.**
#
# The audit measured only published prose and found 7 conflicting pairs.
# Sweeping the sources that FEED that prose found 41. Sweeping the whole
# repository found more still. Measure sources, never rendered pages.
#
# IDENTIFIERS WERE RENAMED TOO, deliberately and with their dependents:
#
#   - `packs/credit` lifecycle state `amortising` -> `amortizing`, in `initial`,
#     `states` and every transition. This is a PACK API CHANGE, not a spelling
#     fix: a model may declare the state by name, so an existing model that says
#     `amortising` will no longer resolve. Nothing in this repository referenced
#     it. Done on an explicit instruction that the American spelling applies
#     repo-wide.
#   - Fixture field and stream names (`amortisation` -> `amortization` in
#     `rule_reads_literal_field` and `event_guard_reads_field`), which changed
#     three IR goldens and their derived model hashes.
#   - The generated-region key `diagnostics-catalogue` -> `diagnostics-catalog`,
#     renamed in the page markers and in sync-content.mjs together. The first
#     attempt renamed only the markers and the site build failed hard. Note that
#     snake_case protects itself from a \b-anchored rule and a hyphen does not.
#
# ONE THING THIS RULE STILL DOES NOT TOUCH: `config`. See the `run
# configuration` entry below — it is a different word for a different thing.
# ---------------------------------------------------------------------------

[spelling]
convention = "US"
replacements_applied = 537
forms_applied = 41
scope = "every tracked file. gold/ is re-blessed, never edited; tools/audit-measure.py holds the UK forms as data and is exempt."
enforced_by = "tools/check-site-voice.py, which loads [spelling.map] at run time"

[spelling.map]
modelled = "modeled"
modelling = "modeling"
amortisation = "amortization"
amortising = "amortizing"
amortise = "amortize"
amortised = "amortized"
amortises = "amortizes"
unamortised = "unamortized"
behaviour = "behavior"
licence = "license"
catalogue = "catalog"
centre = "center"
labelled = "labeled"
favour = "favor"
organised = "organized"
summarised = "summarized"
recognised = "recognized"
recognises = "recognizes"
unrecognised = "unrecognized"
realised = "realized"
capitalised = "capitalized"
capitalising = "capitalizing"
capitalises = "capitalizes"
capitalisation = "capitalization"
stabilised = "stabilized"
stabilising = "stabilizing"
stabilisation = "stabilization"
annualised = "annualized"
securitisation = "securitization"
overcollateralisation = "overcollateralization"
unsubsidised = "unsubsidized"
practised = "practiced"
categorised = "categorized"
monetised = "monetized"
reorganised = "reorganized"
generalised = "generalized"
generalising = "generalizing"
factorises = "factorizes"
materialises = "materializes"
parameterised = "parameterized"
parameterises = "parameterizes"
parameterising = "parameterizing"

# One concept, one term.

[[preferred]]
use = "run configuration"
instead_of = ["run config", "run settings"]
occurrences = { "run configuration" = 57, "run config" = 7, "run settings" = 2 }
note = "Spell it out in prose."
# DOES NOT APPLY TO `config` ON ITS OWN. `config` names a different thing in this
# platform and normalizing the two together would destroy a real distinction:
#
#   run configuration   the object a compiled model is evaluated under —
#                       scenarios, seeds, rate, as-of date
#   cfg / config        the EXPRESSION NAMESPACE, `cfg.<path>`, which reads the
#                       scenario knobs out of that object
#
# So a run configuration CONTAINS what `cfg.*` reads; they are not synonyms.
# docs/03_expression_environment.md glosses `cfg` as "run-config values
# (scenario knobs)", and that gloss is the binding between the identifier and
# its meaning. Leave it alone: rewriting it to "run configuration values" would
# loosen the tie to `cfg` without making anything clearer.
never_rewrite = [
  "cfg",
  "cfg.<path>",
  "config namespace",
  "run-config values",           # the gloss in docs/03, line 70
  "config",                      # also a JSON key in learn/content/exercises.json
]

[[technical_name]]
term = "config namespace"
category = "language"
definition = "The `cfg.<path>` expression binding, which reads scenario knobs from the run configuration. Distinct from the run configuration itself."
note = "Registered so the one-concept-one-term rule never collapses this into `run configuration`."

[[preferred]]
use = "results document"
instead_of = ["output document", "results doc"]
occurrences = { "results document" = 9, "output document" = 2 }

[[preferred]]
use = "click"
instead_of = ["hit", "press", "tap"]
occurrences = { click = 1, hit = 7, press = 1 }
note = "`hit` is not an approved instruction verb. site/content/docs/getting-started.md:31 reads `Hit **Run**` and is the most-read procedural line on the site. `press` stays correct for a physical keyboard key."

# ---------------------------------------------------------------------------
# Technical Names — language constructs.
#
# Approved as nouns with the stated meaning. Each is exempt from the STE
# dictionary and, where multi-word, from the noun-cluster cap.
# ---------------------------------------------------------------------------

[[technical_name]]
term = "entity"
category = "language"
definition = "A thing the model is about. An asset produces or consumes cash; a party contracts, owns, or lends."
occurrences = 191

[[technical_name]]
term = "stream"
category = "language"
definition = "A dated, directed movement of cash attached to an entity and laid out on the timeline."
occurrences = 264

[[technical_name]]
term = "contract"
category = "language"
definition = "A first-class agreement carrying terms and effects."
occurrences = 272

[[technical_name]]
term = "assumption"
category = "language"
definition = "A declared input to the model, constant or stochastic. Declared with `assume`."
occurrences = 33

[[technical_name]]
term = "curve"
category = "language"
definition = "A named series of values indexed by time."
occurrences = 98

[[technical_name]]
term = "quantile"
category = "language"
definition = "A named series of values indexed by cumulative share."
occurrences = 0
note = """SHIPPED — the declaration, `quantile_at`, `quantile_mean` and
`quantile_of`. Designed in docs/27_quantiles.md, specified in docs/01 §12.6.
`occurrences` is 0 because the count predates the construct; re-measure.

The naming decision was the hard part, and three candidates were rejected on
measured evidence:

  profile       10 occurrences, every one meaning a TIME-ORDERED path — "the
                collection profile shortens", "a development's funding profile",
                "generation profile", "decline profile". A chronology-free object
                cannot take the corpus's word for a chronological one.
  distribution  279 occurrences. In CFDL this means cash paid down a waterfall.
                It names an engine stage and a lesson in docs/26.
  deck          37 occurrences. "Price deck" is already an informal synonym for a
                date-indexed curve in docs/01 and docs/09.

Domain terms were rejected on principle rather than on count. The precedent is
`curve` itself: the language took the neutral noun and left "price deck" and
"rate curve" in prose. A construct serving four domains must not adopt one
domain's word. The terms of art — price duration curve, load duration curve,
stratification table, exceedance probability curve, unit mix — stay in
documentation.

One clause separates this definition from `curve`'s, which is the right distance
for two constructs that are siblings."""

[[technical_name]]
term = "fact field"
category = "language"
definition = "A field on an entity that takes a literal and nothing else. A value that moves must be a rule field."
occurrences = 160          # count is for "field"; the two kinds are not counted separately

[[technical_name]]
term = "rule field"
category = "language"
definition = "A field carrying a recurrence: `init` gives the first period's value, `next` is evaluated each later period with `prev` bound to the field's own previous value."
occurrences = 160          # as above

[[technical_name]]
term = "event"
category = "language"
definition = "A construct that fires on a condition and changes model state."
occurrences = 76

[[technical_name]]
term = "option"
category = "language"
definition = "An electable right held by a party."
occurrences = 53

[[technical_name]]
term = "waterfall"
category = "language"
definition = "An ordered distribution of cash through steps by seniority."
occurrences = 100
note = "Also the finance term. One meaning covers both here, which is why it is registered once."

[[technical_name]]
term = "account"
category = "language"
definition = "A declared cash location whose balance accumulates across periods under the balance law; drawn by a waterfall, credited by a step, read by logic as settled history."
occurrences = 0
note = "docs/28 §5.1. The word 'pot' retired with nothing left to denote: 'available' is this period's netted cash, an account is the accumulated cash."

[[technical_name]]
term = "period walk"
category = "engine"
definition = "The engine's evaluation order: per period, state settles, streams evaluate, waterfalls the schedule names distribute — so logic reads cash that has already happened."
occurrences = 0
note = "docs/28. The column order remains for models that cannot walk, and the two compute the same numbers wherever both run."

[[technical_name]]
term = "lifecycle"
category = "language"
definition = "A declared finite state machine: enumerated states, an initial one, and guarded edges declared only as used. A core-language construct that packs tailor to domains."
occurrences = 0
note = "docs/28 §6.1. There is no latch: edge availability is the memory, and a re-entered state re-arms its edges."

[[technical_name]]
term = "priced amount"
category = "language"
definition = "A stream amount whose series window reaches forward: a valuation setting a causal amount, evaluated after the causal cells settle, refused where the graph cycles."
occurrences = 0
note = "docs/28 §7. The forward-income exit and the expense stop's base year are the two shipped cases."

[[technical_name]]
term = "pack"
category = "language"
definition = "A domain vocabulary supplying types, roles, and terms to a model."
occurrences = 580

[[technical_name]]
term = "grain"
category = "language"
definition = "How finely the model slices time and things. Timeline grain and entity grain are independent choices."
occurrences = 65

[[technical_name]]
term = "scenario"
category = "language"
definition = "A named set of overrides applied to a run."
occurrences = 38

[[technical_name]]
term = "statement"
category = "language"
definition = "A per-period report published from the model's results."
occurrences = 87

[[technical_name]]
term = "metric"
category = "language"
definition = "A derived summary figure over the model's cash flows."
occurrences = 98

[[technical_name]]
term = "schedule"
category = "language"
definition = "The set of dates on which a stream pays."
occurrences = 166

[[technical_name]]
term = "diagnostic"
category = "compiler"
definition = "A compiler message carrying a stable code and a source span."
occurrences = 20

[[technical_name]]
term = "span"
category = "compiler"
definition = "The region of source a diagnostic points at."
occurrences = 23

[[technical_name]]
term = "lowering"
category = "compiler"
definition = "The compiler stage that reduces language constructs to the canonical IR. A technical noun, so exempt from the rule against -ing forms."
occurrences = 94

[[technical_name]]
term = "IR"
category = "compiler"
definition = "The canonical JSON intermediate representation a model compiles to."
expand_on_first_use = "intermediate representation (IR)"

# ---------------------------------------------------------------------------
# Technical Names — finance.
#
# Registered so the three-word noun-cluster cap does not fire on terms that are
# correct as written. Rewriting these to fit a word count would produce
# documentation that is shorter and wrong.
# ---------------------------------------------------------------------------

[[technical_name]]
term = "term power purchase agreement"
category = "finance"
definition = "A power offtake contract for a fixed term."
occurrences = 2
exempt_from_noun_cluster = true

[[technical_name]]
term = "weighted average life"
category = "finance"
definition = "The average time to principal return, weighted by principal repaid."
occurrences = 39
exempt_from_noun_cluster = true
note = "The axis convention this is measured on is not settled. Do not state one in prose without checking."

[[technical_name]]
term = "working capital"
category = "finance"
definition = "Receivables, payables, and inventory, modeled on a days-based policy."
occurrences = 17
exempt_from_noun_cluster = true

[[technical_name]]
term = "expense stop"
category = "finance"
definition = "The expense level above which recoveries are billed to a tenant."
occurrences = 3
exempt_from_noun_cluster = true

[[technical_name]]
term = "cap rate"
category = "finance"
definition = "The capitalization rate applied to stabilized income to derive value."
occurrences = 14

[[technical_name]]
term = "lease-up"
category = "finance"
definition = "The period during which vacant space is let to stabilized occupancy."
occurrences = 18

[[technical_name]]
term = "reversion"
category = "finance"
definition = "The terminal value realized at the end of a hold."
occurrences = 14

[[technical_name]]
term = "takeout"
category = "finance"
definition = "Permanent financing that repays construction debt."
occurrences = 8

[[technical_name]]
term = "promote"
category = "finance"
definition = "The sponsor's disproportionate share of profit above a return hurdle."
occurrences = 12

[[technical_name]]
term = "catch-up"
category = "finance"
definition = "The waterfall tier that restores a party to its target share after a preferred return is paid."
occurrences = 8

[[technical_name]]
term = "covenant"
category = "finance"
definition = "A contractual test the borrower must satisfy."
occurrences = 11

[[technical_name]]
term = "DSCR"
category = "finance"
definition = "Debt service coverage ratio."
expand_on_first_use = "debt service coverage ratio (DSCR)"
occurrences = 22

[[technical_name]]
term = "MOIC"
category = "finance"
definition = "Multiple of invested capital."
expand_on_first_use = "multiple of invested capital (MOIC)"
occurrences = 19

[[technical_name]]
term = "IRR"
category = "finance"
definition = "Internal rate of return: the discount rate at which NPV is zero. Undefined for a series that never changes sign."
expand_on_first_use = "internal rate of return (IRR)"
occurrences = 39

[[technical_name]]
term = "NPV"
category = "finance"
definition = "Net present value."
expand_on_first_use = "net present value (NPV)"
occurrences = 71

[[technical_name]]
term = "PPA"
category = "finance"
definition = "Power purchase agreement."
expand_on_first_use = "power purchase agreement (PPA)"
occurrences = 24

[[technical_name]]
term = "EBITDA"
category = "finance"
definition = "Earnings before interest, taxes, depreciation, and amortization."
occurrences = 17

# ---------------------------------------------------------------------------
# Technical Verbs.
# ---------------------------------------------------------------------------

[[technical_verb]]
term = "compile"
definition = "To translate a model into the canonical IR."
occurrences = 85

[[technical_verb]]
term = "lower"
definition = "To reduce a language construct to its IR form."
occurrences = 12

[[technical_verb]]
term = "evaluate"
definition = "To compute a value for a period."
occurrences = 13

[[technical_verb]]
term = "resolve"
definition = "To bind a name to its declaration."
occurrences = 19

[[technical_verb]]
term = "discount"
definition = "To apply a discount rate to a future cash flow."
occurrences = 36

[[technical_verb]]
term = "accrue"
definition = "To recognize an amount in a period without paying it."
occurrences = 2

[[technical_verb]]
term = "escalate"
definition = "To increase an amount on a stated schedule."
occurrences = 1

[[technical_verb]]
term = "seed"
definition = "To fix the random draw so a stochastic run reproduces."
occurrences = 22

[[technical_verb]]
term = "diff"
definition = "To compare two artefacts and report the differences."
occurrences = 13

# ---------------------------------------------------------------------------
# Pedagogical metaphors — Tier C only.
#
# Permitted in the training material on two conditions: registered here, and
# used alongside the plain term rather than instead of it. A metaphor may
# introduce a concept; it may not be the only name the reader is given.
#
# Registering them is the point. It turns a stylistic habit into a decision a
# reviewer can audit.
# ---------------------------------------------------------------------------

[[pedagogical]]
metaphor = "the spine"
stands_for = "the model timeline"
occurrences = 2

[[pedagogical]]
metaphor = "the cast"
stands_for = "the entities"
occurrences = 12

[[pedagogical]]
metaphor = "the atoms"
stands_for = "the streams"
occurrences = 2

[[pedagogical]]
metaphor = "smells"
stands_for = "signs of a modeling problem"
occurrences = 4

[[pedagogical]]
metaphor = "idiom kit"
stands_for = "the set of recurring modeling patterns"
occurrences = 2

[[pedagogical]]
metaphor = "storyline"
stands_for = "one economic subject and the constructs that express it"
occurrences = 10

# ---------------------------------------------------------------------------
# Not approved.
#
# Seeded from the ornament list already encoded in tools/check-site-voice.py so
# the two cannot drift. All currently measure zero in published prose — that
# gate works, and this section exists to keep it that way rather than to report
# a problem.
# ---------------------------------------------------------------------------

[[not_approved]]
terms = [
  "blazing", "blazingly", "lightning-fast", "world-class", "cutting-edge",
  "state-of-the-art", "revolutionary", "seamless", "seamlessly", "effortless",
  "effortlessly", "game-changer", "game-changing", "best-in-class",
  "unparalleled", "robust and", "powerful and", "simply put", "crown jewel",
]
reason = "Marketing ornament. Each is a claim the reader should be left to make."
use_instead = "State what the software does."
enforced_by = "tools/check-site-voice.py"
occurrences = 0

[[not_approved]]
terms = ["mm"]
reason = "Reads as millimetres. Not a standard abbreviation for millions outside a trading desk."
use_instead = "m — write $33.6m, not $33.6mm"
context = "currency"

[[not_approved]]
terms = ["×"]
reason = "U+00D7 multiplication sign. Present in the generated benchmark pages."
use_instead = "x — write 8.0x"

[[not_approved]]
terms = ["‑"]
reason = "U+2011 non-breaking hyphen. Invisible in review and breaks search. Present in docs/01_language_spec.md."
use_instead = "- (U+002D)"
```


---

# Pack rosters

_Generated from packs/*/ (pack.toml, ontology/types.toml, metrics.toml, templates.toml). The diagnostics -> repair catalog is a sibling document: diagnostics-repairs.md._

### Pack `cre` 0.1.0

Commercial Real Estate pack v0.1 (deterministic lowering).

Contract types (a `contract <name>` declaration lowers to streams through the pack's rule for it; a type with no rule name is an election the engine resolves):

| type | contract name | parties | description |
|---|---|---|---|
| `CRE.Contract.Lease` | `cre.lease` | landlord, tenant |  |
| `CRE.Contract.UnitLease` | `cre.lease_unit` | landlord, tenant | A lease modeled at unit grain, with recoveries and expense stops. |
| `CRE.Contract.Rollover` | `cre.rollover` | landlord | The probability-weighted outcome at expiry. Names only the landlord: the renewing or replacing tenant is not known when the model is written. |
| `CRE.Contract.PercentageRent` | `cre.percentage_rent` | landlord, tenant |  |
| `CRE.Contract.PercentageRentExpected` | `cre.percentage_rent_expected` | landlord, tenant |  |
| `CRE.Contract.VacancyAllowance` | `cre.vacancy_loss` | landlord |  |
| `CRE.Contract.OperatingExpense` | `cre.opex_line` | owner | One operating expense line. Instance it per expense for an itemised schedule, or declare one unsuffixed for a single blended figure; the entity it hangs on sets the level. |
| `CRE.Contract.OperatingRevenue` | `cre.revenue_line` | owner |  |
| `CRE.Contract.PermanentDebt` | `cre.permanent_debt` | borrower, lender |  |
| `CRE.Contract.ConstructionFunding` | `cre.construction_stub` | owner, lender |  |
| `CRE.Contract.ConstructionLoan` | `cre.construction_loan` | owner, lender | A construction facility funded behind an equity commitment: equity draws first, the loan takes the balance once the commitment is exhausted, and interest accrues on the drawn balance through the build. |
| `CRE.Contract.Disposition` | `cre.exit` | seller |  |
| `CRE.Contract.DispositionAtCap` | `cre.exit_cap` | seller |  |
| `CRE.Contract.DispositionAtForwardCap` | `cre.exit_forward` | seller |  |
| `CRE.Contract.RenewalOption` | (election) | landlord, tenant | A tenant's right to extend at stated terms. |
| `CRE.Contract.PurchaseOption` | (election) | grantor, holder | A right to buy the asset at a stated price. |

Metrics: `domain.cre.noi`, `domain.cre.debt_service`, `domain.cre.dscr`, `domain.cre.leasing_costs`

Templates (starting points; the `skeleton` MCP tool assembles them into a compiling model):

- `cre.opex_line.property_tax` — Operating expense — property tax
- `cre.opex_line.insurance` — Operating expense — insurance
- `cre.opex_line.utilities` — Operating expense — utilities
- `cre.opex_line.repairs_maintenance` — Operating expense — repairs and maintenance
- `cre.opex_line.cleaning` — Operating expense — cleaning
- `cre.opex_line.security` — Operating expense — security
- `cre.opex_line.landscaping` — Operating expense — landscaping and grounds
- `cre.opex_line.management` — Operating expense — management fee (fixed)
- `cre.opex_line.management_egi` — Operating expense — management fee (percentage of EGI)
- `cre.opex_line.general_admin` — Operating expense — general and administrative
- `cre.opex_line.blended` — Operating expense — single blended line
- `cre.revenue_line.parking` — Revenue — parking
- `cre.revenue_line.storage` — Revenue — storage
- `cre.revenue_line.antenna` — Revenue — antenna and rooftop
- `cre.revenue_line.laundry_vending` — Revenue — laundry and vending
- `cre.revenue_line.blended` — Revenue — single blended line
- `cre.permanent_debt` — Permanent mortgage (proceeds, interest, principal)
- `cre.lease_unit` — Unit lease (rent, abatement, recoveries, TI/LC)
- `cre.exit_forward` — Exit at forward NOI (gross, selling costs)
- `cre.vacancy_loss` — Vacancy and credit loss
- `cre.vacancy_loss.tracking` — Vacancy and credit loss (tracks the rent roll)
- `cre.construction_loan` — Construction loan (equity, draws, interest)

### Pack `credit` 0.1.0

Credit / lending pack v0.1: fixed-rate loan pools (level-pay, IO/bullet) with CPR prepayments, CDR defaults, severity and recovery lag.

Contract types (a `contract <name>` declaration lowers to streams through the pack's rule for it; a type with no rule name is an election the engine resolves):

| type | contract name | parties | description |
|---|---|---|---|
| `Credit.Contract.LevelPayPool` | `credit.pool_level_pay` | holder | Amortizing pool with prepayment, default, severity and recovery lag. |
| `Credit.Contract.InterestOnlyPool` | `credit.pool_io_bullet` | holder |  |
| `Credit.Contract.FloatingInterestOnlyPool` | `credit.pool_float_io_bullet` | holder | Coupon resets off a declared reference rather than being fixed at origination. |
| `Credit.Contract.Purchase` | `credit.purchase` | buyer, seller |  |
| `Credit.Contract.CleanUpCall` | (election) | issuer, holder | The issuer's right to retire the pool once it falls below a stated size. |

Metrics: `domain.credit.interest`, `domain.credit.principal`, `domain.credit.recoveries`, `domain.credit.penalties`, `domain.credit.servicing`, `domain.credit.wal_years`, `domain.credit.collections`, `domain.credit.purchase`, `domain.credit.collections_multiple`

Templates (starting points; the `skeleton` MCP tool assembles them into a compiling model):

- `credit.pool_level_pay` — Level-pay pool (six collection streams)
- `credit.pool_io_bullet` — Interest-only pool, bullet at maturity
- `credit.pool_float_io_bullet` — Floating-rate IO pool, bullet at maturity
- `credit.purchase` — Pool purchase price

### Pack `energy` 0.1.0

Energy & microgrids pack: PPA/merchant revenue, storage, O&M, tax credits, project debt.

Contract types (a `contract <name>` declaration lowers to streams through the pack's rule for it; a type with no rule name is an election the engine resolves):

| type | contract name | parties | description |
|---|---|---|---|
| `Energy.Contract.PPA` | `energy.ppa` | seller, offtaker | Power purchase agreement: contracted price for delivered energy. |
| `Energy.Contract.MerchantSale` | `energy.merchant` | seller | Uncontracted sale into the market. The counterparty is the market, so only the seller is named. |
| `Energy.Contract.StorageArbitrage` | `energy.storage_arbitrage` | seller |  |
| `Energy.Contract.CapacityPayment` | `energy.capacity` | seller, offtaker |  |
| `Energy.Contract.OperationsAndMaintenance` | `energy.om` | owner, service_provider |  |
| `Energy.Contract.ProjectDebt` | `energy.debt_service` | borrower, lender |  |
| `Energy.Contract.Capex` | `energy.capex` | owner |  |
| `Energy.Contract.InvestmentTaxCredit` | `energy.itc` | claimant |  |
| `Energy.Contract.ProductionTaxCredit` | `energy.ptc` | claimant |  |
| `Energy.Contract.DepreciationShield` | `energy.macrs_shield` | claimant |  |
| `Energy.Contract.OfftakeExtension` | (election) | seller, offtaker | An offtaker's right to extend the contracted term. |

Metrics: `domain.energy.revenue`, `domain.energy.opex`, `domain.energy.ebitda`, `domain.energy.debt_service`, `domain.energy.dscr`, `domain.energy.tax_benefits`

Templates (starting points; the `skeleton` MCP tool assembles them into a compiling model):

- `energy.ppa` — PPA revenue
- `energy.merchant` — Merchant revenue
- `energy.om` — Operations and maintenance
- `energy.capex` — Capital outlay
- `energy.debt_service` — Project debt (proceeds, interest, principal)
- `energy.ptc` — Production tax credit
- `energy.macrs_shield` — MACRS depreciation shield

### Pack `opco` 0.1.0

Operating Business pack v0.1 (deterministic lowering + E7xxx checks).

Contract types (a `contract <name>` declaration lowers to streams through the pack's rule for it; a type with no rule name is an election the engine resolves):

| type | contract name | parties | description |
|---|---|---|---|
| `OpCo.Contract.RevenueLine` | `opco.revenue_line` | owner |  |
| `OpCo.Contract.OperatingExpenseLine` | `opco.opex_line` | owner |  |
| `OpCo.Contract.CapexLine` | `opco.capex_line` | owner |  |
| `OpCo.Contract.WorkingCapital` | `opco.working_capital` | owner |  |
| `OpCo.Contract.WorkingCapitalPolicy` | `opco.working_capital_policy` | owner | Days-based policy — receivables, payables, inventory — rather than a stated movement. |
| `OpCo.Contract.TermDebt` | `opco.term_debt` | borrower, lender |  |
| `OpCo.Contract.CashTaxes` | `opco.cash_taxes` | taxpayer |  |
| `OpCo.Contract.Acquisition` | `opco.acquisition` | buyer, seller |  |
| `OpCo.Contract.ExitAtMultiple` | `opco.exit_multiple` | seller |  |
| `OpCo.Contract.ExitAtEbitdaMultiple` | `opco.exit_ebitda` | seller |  |
| `OpCo.Contract.ExitAtPerpetuity` | `opco.exit_perpetuity` | seller |  |
| `OpCo.Contract.EquityOption` | (election) | grantor, holder | A management or investor option over the enterprise's equity, struck at a stated price. |

Metrics: `domain.opco.revenue`, `domain.opco.ebitda`, `domain.opco.ebitda_margin`, `domain.opco.capex`, `domain.opco.working_capital`, `domain.opco.taxes`, `domain.opco.debt_service`, `domain.opco.fcf`, `domain.opco.fcf_to_debt_service`

Templates (starting points; the `skeleton` MCP tool assembles them into a compiling model):

- `opco.revenue_line.core` — Revenue line — core
- `opco.opex_line.operating` — Opex line — operating
- `opco.capex_line.maintenance` — Capex line — maintenance
- `opco.term_debt` — Term loan (proceeds, interest, principal)
- `opco.cash_taxes` — Cash taxes on EBIT
- `opco.working_capital_policy` — Working capital — days policy
- `opco.exit_ebitda` — Exit at an EBITDA multiple (gross, selling costs)
- `opco.acquisition` — Acquisition price
