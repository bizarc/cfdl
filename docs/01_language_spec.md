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
fires on entering one with `when time.phase == "operations"` — once per entry,
on the rising edge (§13.1). A phase entered once yields one occurrence because
the phase occurs once, not because the event is latched.

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
  state stabilized
}

entity asset suite_a : CRE.Asset.Unit {
  rentable_area = 10000
  part of asset.tower
  state leased
}

entity party acme : CRE.Party.Tenant { name = "Acme Corp" }
```

- The **family** is the first identifier: `asset` for something that produces or
  consumes cash, `party` for someone who contracts, owns, lends or invests,
  `container` for a grouping that scopes cash — a fund, a
  portfolio, an SPV, a transaction. Containment reuses `part of`: a container
  parent's aggregate is folded from its members by the relation, exactly as an
  asset parent's is, and counts as cash nowhere. Cash MAY attach directly to a
  container — a land purchase, a development budget, a fund-level fee is
  deal-level cash, and it is real cash, aggregated with the members'.
- The **type** is checked against the active ontology. With no pack active a
  model still has the language's own vocabulary — `Asset.Real`,
  `Asset.Financial`, `Asset.Intangible`, `Party`, `Container.Fund`,
  `Container.Portfolio`, `Container.SPV`, `Container.Transaction` — because an
  ontology is a language capability that packs supply defaults for, not one
  they own. A pack's types are added to those and cannot remove them. A pack
  type states the master it specializes (`refines`, Pack Interface §6.1), so
  "is a" is a recorded fact rather than a naming convention.
- Attribute values are **literals**, checked against the type's declared fields.
- The literal field `id` is a **stable identity**: an opaque string a layer
  above the model assigns to the real-world thing this entity refers to. The
  engine never interprets it — it is validated for uniqueness within the
  model (`E1360`) and republished in the results graph, so a consumer can
  join a package's numbers to canonical things instead of to symbol names.
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
  set entity asset.senior.status = "refinanced"
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

#### 7.3.1 Arrival actions

A transition may carry behavior. Arriving somewhere is an occurrence, and
per-arrival bookkeeping — reset a counter, record a shortfall, strike the
prevailing rent — belongs on the arrival rather than in a separate event
that has to restate the condition.

```cfdl
lifecycle unit {
  initial vacant
  state vacant, leased, downtime

  on enter leased {
    set months_in_state = 0
  }

  vacant   -> leased    when time.t >= 1
  holdover -> leased    when inputs.renews {
    set in_place_rent = prev.in_place_rent * (1 + inputs.bump)
  }
  downtime -> leased    when months_in_state >= inputs.downtime_months {
    set in_place_rent = curve_value("cre.market_rent", time.date)
  }
}
```

There are two grains because both are real:

- **Entry actions** (`on enter <state> { … }`) carry what is true of the
  STATE, however it was reached. Resetting `months_in_state` on entering
  `leased` holds for every edge that arrives there, including one added
  later. This is the primary spelling, and a pack's `types.toml` machine
  declares it once for every model using the type.
- **Edge actions** (a block after the edge) carry what is true of the PATH
  taken. A renewal and a re-let both land in `leased`, but the rent is
  struck differently because of how you arrived. An entry action cannot
  express that: it does not know which edge fired.

Rules:
- **Field names are entity-relative.** `set months_in_state`, not
  `set asset.unit_a.months_in_state` — one lifecycle is bound by many
  entities, and the behavior belongs to the entity that transitioned. The
  field MUST be declared on that entity; an unknown name is a compile
  error.
- **`set` writes FIELDS only, never `status`.** An action writing status
  would fire a second transition inside the same period, breaking
  one-transition-per-entity-per-period and inviting same-period cascades. A
  transition that should cause another transition is topology: an edge out
  of the target state, taken next period. Status writes remain the named
  event's privilege (§13.1), validated against the declared edge relation.
- **`set` is the whole vocabulary.** Stream gating is already declarative
  (`active in state`, §9.3) and schedule anchoring already follows entry
  (`state_enter`, §11.7); an imperative `activate`/`deactivate` on an edge
  would duplicate a declared pattern. `exercise option` stays with named
  events.
- **Actions run on every traversal, whatever took the edge** — its own
  guard, or a named event's `status` write moving the entity across a
  permission edge. A status write that moves an entity runs the target
  state's entry actions exactly as the machine's own path would.
- **Order on arrival is entry actions, then the taken edge's actions**, each
  block in declaration order — the specific refines the general. A
  same-field write journals the earlier value `overridden`.
- **Actions evaluate in the guard's environment**: state as the period
  opened, series strictly backward, `inputs`, curves, `time.*`. This is the
  environment the walk already proves acyclic, so actions add no cycle risk.
  A write settles the field for the period; the recurrence resumes from it
  next period, and streams later in the same period read the settled value.

#### 7.3.2 Augmenting a pack's machine

A model MAY attach entry and edge actions to a machine its pack declared,
**additively only**. It names the machine by its qualified name — a
`lifecycle` block naming a machine that already exists augments it rather
than redeclaring it:

```cfdl
use pack "opco"

lifecycle opco.enterprise {
  on enter acquired {
    set months_held = inputs.prior_hold_months
  }
}
```

A block naming a machine that does not exist declares a new one, and is then
required to carry `initial` and `state` as usual.

An augmenting block cannot add or remove states or edges, alter the topology,
or remove the pack's actions; the pack's machine stays the checkable
contract. The model's actions run after the pack's for the same arrival, so a
same-field conflict resolves the model's way and journals the pack's value
`overridden`.

Every action records **who wrote it**, and the journal names that author. A
secondary buyout that carries the sponsor's prior holding period, against a
pack whose machine resets the clock on `acquired`, reads as two journal rows
against the same field — one `overridden`, one `applied` — each naming its
author. Attribution is not optional metadata: an action that cannot say who
wrote it is refused, because an unattributable `overridden` line is the one
thing the record exists to prevent.

This is what makes arrival actions an extensibility surface rather than a
pack-author convenience: the common position is modeling ON a pack whose
machine is right but whose actions stop short.

---

## 8. Contracts (Behavior container)

For when to use contracts vs standalone streams, see the **Language guide** ("When to use streams vs contracts").

### 8.1 Contract declaration (canonical form)
Syntax (normative):

```cfdl
contract cre.lease on entity asset.sunset {
  term 2026-02..2028-01
  terms {
    rent = 42000
    escalation = 0.03
  }
}
```

Rules:
- `contract <TypeId> [<instance>] on entity <EntityRef> { term <Date>..<Date>  terms { ... } }`
- `<TypeId>` is the pack contract type (e.g. `cre.lease`). An instance name
  makes an independent instance, and it is its own token:
  `contract cre.lease_unit tenant_a`. The fused spelling
  `cre.lease_unit.tenant_a` is the same declaration; both produce the
  qualified name `cre.lease_unit.tenant_a`, which is what lowered streams
  and references use. The type is checked where it is written: a type the
  pack does not declare is `E1373`, a master is `E1374`.
- `term` is REQUIRED.
- `terms` is OPTIONAL; entries use `<name> = <literal-or-expression>`.
- Monetary amounts default to the model currency; streams declare currency
  explicitly.
- Effects come from the active pack's lowering rules, which expand the
  contract into streams in the IR. A `parties { <role> = <party> }` block
  binds parties to the roles the contract's TYPE declares, checked against
  the effective roles of its master chain (`docs/40` §5). Explicit
  `effects`/`tags` blocks are **tolerated by the parser but not
  represented in IR** in v0.1 (reserved; see `10_implementation_status.md`).
- A contract's type is the master chain its pack type refines
  (`docs/40`): its terms are checked against that chain's effective
  fields — an unknown term is `E1371`, a missing required term or an empty
  group of alternatives is `E1372` — and a master itself cannot be
  declared: a model reaches a type through a pack's concrete refinement.
  The IR records the resolved type, its master, the instance and the
  parties with their master roles.

### 8.2 Terms block
- `terms { ... }` is a set of named values.
- Term keys MAY be qualified names (e.g., `lease_up.months`).

#### 8.2.1 A term records what was agreed (normative)

A term's value is one of:

- a **literal** — a number, string, date, or `true`/`false`;
- a **reference to one declared input**, written `inputs.<name>`;
- a **reference to a declared contract or account**, by name, where the
  type's field is of type `contract` or `account` (a guarantee's `covered`,
  a note's `principal_account`; `docs/40` §4.13, §4.17) — a name nothing
  declares is refused (`E1376`); or
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
    rent  = 42000                                  // literal fact
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

contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2050-12
  terms {
    price = 3000              // contractual fact
    quantity  = inputs.annual_yield  // driver, supplied per run
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
- Streams MUST declare direction: `inflow` or `outflow` for cash, or
  `accrual` or `writeoff` for a movement of a balance with no money moving
  (`docs/42` §3.2). An accrual raises a claim (capitalized construction
  interest, a PIK coupon, an interest shortfall); a write-off extinguishes
  one (a realized loss, a bond write-down). Both publish as series, are
  excluded from every cash total, category fold and valuation, carry no
  category (`E1379`), and MUST name the account they move (`E1378`).
- A stream MAY say `moves <account>`: the account its amount moves — a
  claim declared on its owning entity by bare name, or any declared account
  by qualified name (`E1380`). Most streams move nothing. Whether a cash
  stream raises or lowers the balance follows from the account's side
  (§10.6), never from a word on the stream.
- A stream MAY read an account's OPENING balance as `prev.<account>` — the
  prior close, or the `init` in the first period. It never reads a
  same-period close (`E1382`).
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
  category investing.disposal.reversion
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

**A step may say which agreement and line it pays.** `pay <step> to <payee>
for contract <name> line <role> = <expr>` names a declared contract and one
of the lines its type declares ALLOCATED (`docs/40` §6) — a structured
note's `principal`, an equity interest's `distribution`. The step's series
then carries the contract and the line, the results graph lists the step
under the contract, and a slice or statement row by `type` and `line`
reaches it. A contract nothing declares is refused (`E1376`); a line the
type does not declare allocated is refused (`E1377`), because a line a rule
lowers is paid by the rule and a step paying it would count the cash twice.

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
- **An entity may own an account** (`docs/42` §3.6): a claim declared in
  its block, `account balance owed init 1000000`, named
  `<entity>.<account>` — `asset.loan.balance`. Its side, `owed` (a
  liability of the owner) or `due` (a receivable), decides which way a cash
  stream that `moves` it changes it; a structure account may declare a side
  the same way, after its name. A pack contract's account takes its side
  from the master, so no modeller using a pack writes it.
- `init <expr>` is the balance at the timeline's first period. It defaults
  to zero: a balance outstanding when the model opens states it; one created
  during the run is raised from zero by the cash that creates it, and
  `prev.<account>` is never absent.
- **Streams move it** (§9.1): `moves <account>` on an inflow or outflow, an
  accrual or a write-off. Each movement is a journal line naming the stream
  that caused it.
- `from` is the per-period inflow, reading cash that has settled this
  period. It MAY be negative, and **the balance has no floor**: an account
  fed a deal's whole net cash IS the deal's cumulative position, negative
  through the J-curve and positive after.
- There is no currency clause: an account is denominated by the model.

The balance law, applied at each period:

```
balance(t) = balance(t-1) + inflow(t) + moved(t) + allocated_in(t) - allocated_out(t)
balance(-1) = init
```

where `moved(t)` is what the streams naming this account moved: on an
`owed` account an inflow raises and an outflow lowers, on a `due` account
the reverse; an accrual raises and a write-off lowers on either side.

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
- **Logic, a step and a stream read a balance** — `prev.<account>` is
  settled state, strictly backward: the balance at the previous period,
  every allocation and movement through it included. In the first period it
  is the `init`. A stream reads it too — interest on the opening balance is
  the ordinary case — and nothing reads a same-period close.

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
  set entity asset.senior.status = "refinanced"
  deactivate stream loan.debt_service
}
```

An event may also state WHEN it occurs in the schedule language, with or
without a `when` condition:

```cfdl
event covenant_test schedule every quarter
  when series_sum("plant.noi", time.t - 4, time.t - 1)
     < 1.20 * series_sum("plant.debt_service", time.t - 4, time.t - 1) {
  set entity asset.plant.status = "trapped"
}
```

A guard reads the walk's own past — series strictly backward, fields,
curves. It cannot read `domain.*`: a subtotal is a fold over the settled
ledger, and inside the walk the ledger has not settled. State the covenant
on the flows themselves, as above.

```cfdl
```

Rules:
- **An event is something that happens**, and nothing restricts it to
  happening once. A unit that defaults, cures and defaults again has had
  three events, and a model that can only record the first is wrong.
- **An event fires on each occurrence — rising edge.** It fires each time
  its conditions become true having been false, and re-arms when they fall.
  A DSCR below trigger for twelve months is ONE breach event and twelve
  breach-months; "the conditions hold" is a state, and `active in state`
  (§9.3) and edge guards (§7.3) already express states.
- **A schedule supplies the occurrences; `when` filters them.** An event may
  carry a schedule clause, a `when` clause, or both. With both present it
  fires at each scheduled occurrence where the condition holds — `schedule
  every quarter when dscr < 1.20` is a quarterly covenant test, and four
  consecutive failing tests are four breach events, because the model
  declared quarterly testing. Rising-edge applies only where no schedule is
  present, occurrences then coming from the condition's own dynamics. A
  model wanting "only on entry into breach" has the honest spelling already:
  an edge on a machine, which is what states are for.
- **Time conditions belong in the schedule language.** The schedule
  sub-language is already the language of when things occur — dates,
  intervals, anchors including `state_enter`, roll conventions, calendars.
  Write `schedule on 2027-03`, not `when time.t == 15`.
- **There is no `once` keyword, and nothing latches.** Once-ness is a
  property of the world the model declared, and it has two spellings: a
  schedule whose occurrence is singular (`schedule on 2027-03` fires once
  because the date occurs once), or a topology with no way back (a refinance
  fires once not because the event is latched but because afterwards the
  loan IS refinanced — `current -> refinanced` with no returning edge). If a
  model declares a return edge, it has declared that re-firing is possible.
- **Named and anonymous events work identically.** To describe an event
  informally, use guard conditions on the machine (§7.3); to canonize one,
  use `event`. Same firing semantics, same evaluation environment, same
  journaling. The anonymous form suits entity-local conditions; the named
  form suits occurrences worth canonizing — referenced from elsewhere, or
  spanning entities.
- `when` MUST be a boolean expression.
- **Conditions are evaluated once per period**, in declaration order, in the
  walk's state stage. An event fires at most once per period, and at most
  one transition per entity per period is taken. Rising-edge detection
  compares this period's evaluation against the last; nothing re-evaluates
  within a period.
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

`activate`/`deactivate contract` are not actions. A contract is a collection of
streams — `cre.lease` lowers into base rent, recoveries and abatement — and one
switch gives a single answer where forbearance (principal stops, interest
accrues) and an early termination (rent stops, a fee flows, recoveries continue)
need a per-stream one. Gate the streams themselves, by name below or with
`active in state` (§9.3), which is checked and can end as well as begin.

A `<StreamName>` is any stream the model runs: one the model declared, or one a
contract lowered (`cre.lease.base_rent` is §9.1's own example, and `docs/07`
§6.4 gives the identical string as an example of a generated name). Reaching
for a contract does not cost the ability to stop its cash. A name matching
neither is `E1302`.
- `exercise option <OptionName>`

### 13.3 Event timing and the grid (normative)

An event fires at **each occurrence**, evaluated once per period against the
state as that period opened. Where the event carries a schedule, the
occurrences are the scheduled ones the `when` clause admits; where it does not,
an occurrence is the period its condition becomes true having been false. It
cannot fire between periods.

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

A trial IS a complete deterministic run, and every metric it computed is
published: each trial summary carries the same metric map the deterministic
block carries — `model.*`, `domain.*` and `metric.*` alike — and
`monte_carlo.metrics` summarises each name across the trials with a mean, a
standard deviation, a minimum, a maximum and the full set of percentiles. A
summary also states `trials`, the number that published that name, because not
every trial publishes every one: `model.irr` exists only where the flows solve
for a rate. A name a distribution cannot be taken over — a metric published as
a string, or one whose kind changed between trials — is carried per trial and
left out of the summary.

Per-trial SERIES are not retained: a stochastic run's output is bounded by the
model's metric names and its trial count, not by its horizon as well.

### 15.2 Engine-computed outputs
Output metrics (NPV, IRR, DSCR, NOI, etc.) are computed by the engine based on the domain pack's output specification.

### 15.3 Model-declared metrics (normative)

A model MAY declare a metric — a figure it solved for that neither the engine
nor a pack mints:

```cfdl
metric class_a_wal   = series_sum("credit.class_a.principal", 0, 59) / 12.0
metric crossover     = metric.class_a_wal - inputs.expected_wal
```

Rules:
- A metric name MUST be unique within the model (`E1008`).
- A metric is evaluated ONCE, at the horizon, over the finished projection —
  the valuation plane of `docs/28` §2. It is a fold over a completed
  projection, not a recurrence: nothing it computes can feed back into the
  walk.
- Its expression MAY read series (including the projection tail, which is what
  a forward-looking figure needs), entity fields, `inputs`, `cfg`, the
  engine's `model.*` metrics, and `metric.<name>` for any metric DECLARED
  ABOVE IT.
- **The series it may fold are the ones this model PUBLISHES**, in either
  spelling: a stream by its own name (`ops.rev`) or by its published key
  (`stream.ops.rev`), a waterfall step, `entity.<symbol>.net_cash_flow`,
  `account.<name>`, an entity field's own series, a money subtotal, a
  declared slice's net as `slice.<name>`, and `model.net_cash_flow`. The
  two spellings of one stream name the same cash. A slice is how a metric
  folds cash selected by TYPE and LINE — every debt's interest — without
  naming a stream or a pack's category.
  A ratio subtotal is NOT foldable: its undefined periods publish as `null`
  rather than zero, and what a fold should do with `null` is not yet decided.
- A metric that folds a name this model does not publish is REFUSED
  (`E1365`), not read as zero. A selector ending in `.*` may still match
  nothing, because matching nothing is what a selector states at its call
  site.
- Metrics compose in declaration order — the same rule waterfalls follow
  (§10.5) — so the dependency is an order rather than a graph. A forward or
  circular reference is refused (`E1354`).
- Every metric is published as `metric.<name>` in `deterministic.metrics`, in
  every scenario summary, and in every Monte Carlo trial summary, so a
  scenario grid can assert a derived figure per column and a stochastic run
  gives that figure a distribution — not only the engine's built-ins.

**A participant's realized return.** Two folds are available in a metric and
nowhere else:

```cfdl
metric lp_irr  = irr(party.lp)
metric lp_moic = moic(party.lp)
```

**The party is a REFERENCE, not text.** A party is an entity, named the way the
language names entities everywhere else — `pay … to party.lp`, `owner
party.lp`, `on entity asset.x` — and the reference is what lets the compiler
resolve it: an undeclared name is `E1301`, an entity that is not a party or
that owns no account is `E1356`. Text would defer all three to the run.

Both read the party's own ACCOUNT — a contribution is a negative inflow, a
receipt is an allocation in, so the sign change an IRR needs is recorded rather
than inferred. They are folds over the party's account and never over a payee's
streams: a step's payee says who was paid, but attributing through stream names
is a different question. A party owns at most one account.

Both are refused outside a metric (`E1355`): reading a return in a stream
amount asks for a return on cash that stream has not produced yet. What cannot
be known until the run — flows that never change sign — refuses the run naming
the party, because a metric the author declared must not silently go missing.

The three namespaces stay distinct, and the prefix says who minted the number:
`model.*` is the engine's, `domain.*` is the active pack's, `metric.*` is this
model's.

### 15.4 Slices (normative)

A model MAY declare a slice — a named, deliberately partial selection of its
own streams, with figures computed over the selection:

```cfdl
slice artist_a_royalties {
  entity asset.artist_a
  category "operating.revenue.royalty"
}

slice label_ex_merch {
  entity container.label
  except category "operating.revenue.merchandise"
}

slice debt {
  type Contract.Debt
}

slice debt_interest {
  type Contract.Debt
  line interest
}

slice west_2027 {
  entity asset.west_tower
  window from 2027-01 to 2028-12
}
```

Rules:
- A slice name MUST be unique within the model (`E1361`).
- Clause KINDS intersect; values within a kind union; `except` subtracts
  last. A kind that is absent does not constrain, so a slice of nothing but
  excepts reads "everything minus these".
- `entity` and `except entity` take REFERENCES — an undeclared entity is
  refused (`E1362`) — and select the entity together with its `part of`
  descendants, so a container's slice is its members'.
- `type` names an ontology type, matched transitively through `refines`:
  `type Contract.Debt` selects every stream lowered from a contract whose
  type is_a `Contract.Debt`, and streams owned by entities of a conforming
  type. An unknown type is refused with the known types named (`E1363`).
- `line` names a LINE BY ROLE — what a master says the agreement produces
  (`docs/40` §6): `line interest` selects every stream a pack rule emits as
  its type's interest line, whichever pack and whatever category it spells.
  Beside `type` the two intersect: `type Contract.Debt line interest` is
  the interest of every debt. A line nothing produces is refused with the
  near miss (`E1375`).
- `category` and `stream` take QUOTED selectors — the dialect `series_sum`
  reads, exact or one trailing `.*` — and a category selector must be rooted
  in a statement section (`E1364`).
- `window` bounds the PERIODS, where every other clause bounds the streams. A
  period outside it contributes nothing, so `total`, `npv` and `irr` are folds
  over the window. At most one window per slice.

  **A window is not a phase.** A phase is a lifecycle anchor — `phase_start()`
  and `phase_end()` drive schedules, and a period is named by the phase it sits
  in. A window is a reporting bound applied to a finished projection. Giving one
  construct both jobs would mean neither could change without the other.

  Dates, not period indices: an index is a fact about one grid, and a window
  that survives a change of calendar has to be stated in dates. A month-only
  bound means the first of that month, as a phase's does.
- Results publish each slice's selection (the lineage), the streams it
  matched, its net per-period series, and `total`/`npv`/`irr` over the
  matched streams on the model's own axis. A slice carries **no
  reconciliation block**: it is partial by design, and must be seen to be —
  a slice never publishes a residual and never claims the model's total.

See the Pack Interface specification for details on how packs define output categories, aggregations, and metrics.

---

- **A slice is a VIEW, and changes no identity.** It filters a completed
  result; it produces no cash. The compiler files it under the document's
  `views`, which `model_hash` is taken over WITHOUT — so two users who look at
  identical results differently share a model hash, and a slice moves neither
  hash. A declared METRIC is not a view: it is a figure the model claims, so it
  belongs to the model and does move `model_hash`.

### 15.5 Statements (normative)

A model MAY declare a statement — how its results are organized. A statement
enumerates NO rows:

```cfdl
statement portfolio {
  label     "Portfolio by property"
  structure entity
  depth     2
}

statement operating {
  label     "Operating statement"
  structure category
  depth     3
  slice     west_2027
  metrics   noi_yield, lp_irr
}
```

Rules:
- A statement name MUST be unique within the model (`E1366`).
- `structure` names an existing hierarchy: `entity`, the `part of` tree the
  results graph publishes, or `category`, the dotted category path. A structure
  the engine does not build is refused (`E1367`), as is a category statement in
  a model whose streams declare no category — either would render one residual
  row and nothing else.
- Generated rows are ordered so the result reads as a hierarchy: DEPTH FIRST
  for an entity structure, so a parent is followed by its own subtree rather
  than by every other node at its level. Category ROOTS follow the canonical
  order — operating, investing, financing, the order a cash flow statement is
  read in — and siblings below a root sort alphabetically, which is arbitrary
  but stable.
- A generated row's LABEL is derived from the name it was generated from: the
  last path segment, underscores opened out, first letter capitalized, so
  `operating.revenue.base_rent` reads "Base rent". An authored row states its
  own.
- `depth` sets the LEVEL OF AGGREGATION, and the rows follow from the tree.
  **A subtotal is not declared.** A node whose children are shown is a
  `subtotal`; a node whose children are cut off by `depth` is a `line`,
  carrying all of its descendants' cash. That single rule is what keeps the
  bottom line reconciling at every depth: the lines always partition the cash,
  whichever level the tree is cut at.
- `slice` filters, orthogonally to the structure — any structure may be shown
  for any filter. A statement so filtered reconciles against the SLICE's total
  rather than the model's, because reporting the filter as a shortfall would
  make a warning fire on a correct model.
- `metrics` names declared metrics to publish beside the statement. A metric is
  one number at the horizon and every row is a series, so the figures sit in
  their own map rather than as a row kind. An undeclared slice or metric is
  refused (`E1368`).
**A statement may state its own rows instead.** A generated statement is right
when the tree IS the presentation; a pro forma is not that, because its rows
carry curated labels, its expenses are shown positive under "Less:", and it
ends in a coverage ratio that is a node of no hierarchy.

```cfdl
statement operating {
  label    "Operating statement"
  line     "Base rental revenue"   { category "operating.revenue.base_rent" }
  line     "Less: operating costs" { category "operating.expense.*" display positive }
  line     "Less: interest"        { type Contract.Debt line interest display positive }
  subtotal "Net operating income"  { category "operating.*" }
  spacer
  ratio    "DSCR"                  { of noi to debt_service display positive }
}
```

- A statement is AUTHORED OR GENERATED, never both, and never neither
  (`E1369`). A generated statement partitions the cash by construction; an
  authored one partitions it by the author's care. Mixed, neither holds.
- A row draws from `category`, `stream`, `type`, `line`, `slice` or
  `entity`. `type` and `line` mean what they mean on a slice — every
  debt's interest, whichever pack lowered it — and the compiler expands
  them to the exact streams the row claims, so the bottom line reconciles
  as it does for a category row (`E1363`, `E1375` as on a slice). A
  `subtotal` row folds rows stated elsewhere and CLAIMS nothing, so it never
  doubles the bottom line.
- A row may instead draw a published `series` (`series "domain.cre.noi"`) — a
  fold OF the ledger rather than cash in it, so the row claims no streams and
  its figure stays out of the bottom line. A claim clause beside a `series` is
  refused (`E1370`), and a key this run does not publish renders no values and
  no total rather than a column of fabricated zeros.
- A `ratio` divides two declared SLICES — a slice is already a named selection
  with a per-period net, so a ratio needs no row identifiers. A zero
  denominator publishes `null`, not zero. A ratio carries no total, because
  summing one means nothing.
- `display` says how to RENDER the sign and never what is summed: `values`
  carries the signed amount, so a consumer that ignores it still adds up
  correctly. An outflow is negative cash, so a coverage ratio is arithmetically
  negative and `display positive` is how it is shown.
- A row's `depth` is an indent. The STATEMENT's `depth` is a level of
  aggregation — an authored row states where it sits, a generated one is told
  by the tree.

- Every clause word is CONTEXTUAL; only `statement` is reserved.
- **A statement is a VIEW.** It changes no value and no identity: the compiler
  files it under the document's `views`, which `model_hash` is taken over
  without, so adding a statement moves neither hash. Views may be declared
  beside the streams they present, or kept in their own file and `import`ed.

A pack declares its own statements the same way, and both render through one
evaluator.

**A model that declares no statement gets one.** When neither the model nor a
pack provides a presentation, the entity hierarchy is rendered as a default,
marked `default`, so a reader holding results sees the model's shape rather
than a flat list of series keyed by symbol. It is assembled when results are
rendered and never enters the compiled document, so it changes neither hash;
and it yields to any declared statement, because a declaration means the
presentation question is already answered.


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
- `series_sum(<pattern>, <window>)` / `series_avg` / `series_max` /
  `series_min` / `series_prod` / `series_count` — cross-stream reductions
  (dependency-ordered waves; cycles rejected). Every one folds the per-period
  aggregate of the matched streams; `series_max`/`series_min` refuse a
  selection that matches nothing, where the others return their identity

**Math and finance**
- Standard arithmetic `+ - * / ^`, comparisons, `and/or/not`, `if(cond, a, b)`
- `min/max/sum/avg/abs/round/round_down/round_up/clamp/pow`
- `pmt/ipmt/ppmt/rate/nper/pv/fv` (Excel sign conventions; `npv` and `irr` are
  *metrics* computed over results, not expression functions)
- `year_frac/eomonth/edate/date/parse_date/months_between`
- `is_business_day/roll/add_business_days` with named holiday calendars
- `macrs_rate`, `cpr_to_smm`
- `curve_value`, `series_sum`, `series_avg`, `series_max`, `series_min`,
  `series_prod`, `series_count`
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

### 18.1 In use (91)

Read by a production of the grammar:

`account`, `accrual`, `activate`, `active`, `also`, `annual`, `as`, `assume`, `calendar`, `clip`, `contract`, `convention`, `currency`,
`curve`, `daily`, `day`, `days`, `deactivate`, `deterministic`, `effects`, `end`, `entity`, `eom`, `event`,
`every`, `except`, `exercisable`, `exercise`, `false`, `following`, `for`, `from`, `import`, `in`, `inflow`,
`LogNormal`, `metric`, `mid`, `model`, `modified_following`, `modified_preceding`, `monte_carlo`, `month`, `monthly`, `months`, `moves`, `net`, `none`,
`Normal`, `on`, `option`, `owner`, `outflow`, `pack`, `parties`, `payment`, `payoff`, `phase`, `phase_end`, `phase_enter`,
`phase_start`, `preceding`, `quantile`, `quarter`, `quarterly`, `run`, `schedule`, `seed`, `set`, `slice`, `state`, `start`, `statement`, `stream`, `stub`,
`term`, `terms`, `time`, `to`, `trials`, `Triangular`, `true`, `type`, `Uniform`, `use`, `version`,
`waterfall`, `week`, `when`, `writeoff`, `year`.

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
entity asset senior : Asset.Financial
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
    rent = inputs.base_rent
  }
}

stream loan.debt_service on entity asset.senior outflow currency USD {
  active when entity.status != "refinanced"
  schedule every month from 2026-01 to 2031-12
  amount = -pmt(0.06 / 12, 72, 8500000)
}

event refi_if_rates_drop when curve_value("sofr", time.date) < 0.045 {
  set entity asset.senior.status = "refinanced"
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

