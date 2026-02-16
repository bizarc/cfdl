# CFDL v0.1 Core Language Specification

**Status:** Draft (Greenfield v0.1)

**Purpose:** CFDL (Cash Flow Domain Language) is a proprietary, human-readable DSL for defining cash-flow models across asset classes. A CFDL model compiles deterministically to a canonical JSON IR used by valuation engines (deterministic DCF, Monte Carlo, scenarios, risk/metrics).

---

## 1. Design goals

### 1.1 Non‑negotiables
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
use pack "evs/cre" version "0.1"
```

Rules:
- `use pack` MAY appear in `model.cfdl` only.
- If multiple `use pack` statements exist, compilation MUST fail.
- A pack MAY:
  - define/extend the ontology type registry
  - define alias mappings
  - define validators for contract terms/types
  - define contract lowering rules
  - define additional CEL functions

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
The language provides schedule helpers (see §9) and event helpers (see §11):
- `phase_start("name")`
- `phase_end("name")`
- `phase_enter("name")` (an instant)

---

## 7. Entities (Structure)

### 7.1 Entity declaration
Syntax:

```cfdl
entity asset Sunset : CRE.Asset {
  city "Austin"
  units 24
}
```

Rules:
- `entity <namespace> <name> : <TypeId> { ... }`
- The full entity symbol is a qualified name with at least two segments (e.g., `asset.Sunset`, `org.asset.Sunset`).
- `<TypeId>` MUST resolve in the active type registry (core + pack).
- Attributes MUST type-check against the ontology when a pack provides schemas; otherwise attributes are permitted but only minimally typed.

### 7.2 Entity state
- Entities MAY have mutable state values through events:

```cfdl
set entity loan.Senior.status = "refinanced"
```

Rules:
- Entity state is the primary mechanism used to trigger/activate contractual behavior.

---

## 8. Contracts (Behavior container)

For when to use contracts vs standalone streams, see the **Language Guide** ("When to use streams vs contracts").

### 8.1 Contract declaration (canonical form)
Syntax (normative):

```cfdl
contract Contract.Lease L1
  on entity asset.Sunset
  term 2026-02-01 .. 2028-01-31
{
  currency USD

  terms {
    base_rent 42000 USD
    escalator cel "1 + inputs.rent_growth"
  }

  effects {
    stream rent owner entity asset.Sunset direction inflow currency USD {
      schedule every month on eom
        from 2026-02-01 to 2028-01-31
        convention modified_following
        calendar "NYSE"

      amount cel "terms.base_rent * pow(terms.escalator, t.year_index)"
    }
  }
}
```

Rules:
- `contract <TypeId> <Name> on entity <EntityRef> term <Date> .. <Date> { ... }`
- `<Name>` SHOULD be a qualified name for ontology/domain alignment (e.g., `cre.lease.primary`).
- `term` is REQUIRED.
- `currency` is REQUIRED if any monetary effects are emitted by this contract.
- `terms` is OPTIONAL.
- `effects` is REQUIRED unless the active pack guarantees a lowering rule that produces effects.

### 8.2 Terms block
- `terms { ... }` is a set of named values.
- Term names are scoped to the contract instance and accessible in expressions as `terms.<name>`.
- Term keys MAY be qualified names (e.g., `lease_up.months`).

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
- Contract instance names MAY be qualified names; uniqueness applies to the full name.
- Contracts may be referenced by name via `contract("L1")` in expressions (see §12).

---

## 9. Streams (cash flow vectors)

For when to use standalone streams vs contracts, see the **Language Guide** ("When to use streams vs contracts").

### 9.1 Stream declaration (standalone)
Syntax:

```cfdl
stream taxes on entity asset.Sunset outflow currency USD {
  schedule every year on 2026-12-31 from 2026-01-01 to 2031-12-31
  amount cel "ref('TaxRates.County').rate * entity('asset.Sunset').assessed_value"
}
```

Rules:
- Streams MUST be owned by exactly one entity.
- Streams MUST declare direction: `inflow` or `outflow`.
- Streams MUST declare a currency.
- Stream names MAY be qualified names; dotted names are recommended for domain/ontology-aligned models.

### 9.2 Stream declaration inside contract effects
Syntax:

```cfdl
effects {
  stream rent owner entity asset.Sunset direction inflow currency USD { ... }
}
```

### 9.3 Activation guards
Streams MAY include an activation predicate:

```cfdl
active when cel "entity('loan.Senior').status != 'refinanced'"
```

If omitted, streams are active for all scheduled occurrences.

### 9.4 Amount
- Streams MUST define an `amount` expression.
- Amount expressions MUST evaluate to `Money` or `Decimal` depending on slot; in v0.1, `amount` MUST evaluate to `Money` (or a `Decimal` that is implicitly converted to Money using the stream currency).

---

## 10. Schedules (important)

Schedules define occurrence dates/times for streams.

### 10.1 Core schedule grammar
Schedules appear inside a stream block:

```cfdl
schedule <schedule_expr>
```

### 10.2 Schedule expressions
#### 10.2.1 One-time
```cfdl
schedule on 2026-02-15
```

#### 10.2.2 Bounded recurring (frequency)
```cfdl
schedule every month from 2026-02-01 to 2028-01-31
```

#### 10.2.3 Day rules
```cfdl
schedule every month on day 1 from 2026-02-01 to 2028-01-31
schedule every month on eom from 2026-02-01 to 2028-01-31
```

#### 10.2.4 Weekday sets (daily/weekly)
```cfdl
schedule every week on Mon,Wed,Fri from 2026-02-01 to 2026-06-30
```

#### 10.2.5 Business-day conventions
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

#### 10.2.6 Stub rules
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

### 10.3 Schedule resolution rules (normative)
- A schedule MUST resolve to a set of dates.
- If the master timeline is not daily, dates MUST be mapped deterministically to the nearest representable period boundary according to the schedule’s convention.
- The compiler MUST either:
  - (a) define a deterministic mapping rule, or
  - (b) reject schedules that cannot be represented at the timeline grain.

Recommended v0.1 rule (normative):
- If timeline is monthly/quarterly/annual, schedule occurrences are represented at that period’s end-date unless the schedule specifies otherwise.

---

## 11. Assumptions (deterministic and stochastic)

### 11.1 Deterministic assumption
```cfdl
assume discount_rate = cel "0.10"
```

### 11.2 Stochastic assumption (distribution)
```cfdl
assume rent_growth ~ Normal(mean=0.03, stdev=0.01, clip=[-0.02, 0.08])
```

Supported distributions (v0.1 core):
- `Normal(mean, stdev, clip?)`
- `LogNormal(mu, sigma, clip?)`
- `Uniform(min, max)`
- `Triangular(min, mode, max)`

### 11.3 Where distributions may appear
- Distributions MUST be declared via `assume <name> ~ Dist(...)`.
- Any term or expression can reference stochastic values via `inputs.<name>`.

### 11.4 Reproducibility (normative)
- Any Monte Carlo run MUST declare an explicit seed.
- Engines MUST use this seed deterministically for sampling.

---

## 12. Events (discrete model changes)

### 12.1 Event declaration
Syntax:

```cfdl
event refi_if_rates_drop when cel "obs('Rates.SOFR.1M') < 0.035" {
  set entity loan.Senior.status = "refinanced"
  deactivate stream debt_service
}
```

Rules:
- Events are evaluated **discretely** at each time step of the master timeline.
- `when` MUST be a boolean expression.

### 12.2 Actions (v0.1 core)
Supported actions:
- `set entity <EntityRef>.<field> = <value>`
- `activate stream <StreamName>`
- `deactivate stream <StreamName>`
- `activate contract <ContractName>` (optional)
- `deactivate contract <ContractName>` (optional)
- `exercise option <OptionName>`

### 12.3 Entity-state-driven activation
Contracts and streams SHOULD use entity state as the primary activation mechanism:

```cfdl
active when cel "entity('loan.Senior').status != 'refinanced'"
```

---

## 13. Options (real options, minimal v0.1)

### 13.1 Option declaration
Syntax:

```cfdl
option refi_1 type Option.Refinance exercisable in construction {
  exercise when cel "obs('Rates.SOFR.1M') < 0.035"
  payoff cel "npv(stream('debt_service_old') - stream('debt_service_new'), discount=assume('discount_rate')) - 250000 USD"
}
```

Rules:
- Options MAY be activated/deactivated via events.
- v0.1 supports only deterministic exercise triggers.
- Optimization/search policies are out of scope for v0.1.

---

## 14. Runs and metrics

### 14.1 Runs
```cfdl
run deterministic
run monte_carlo trials 20000 seed 42
```

Rules:
- `monte_carlo` MUST provide `trials` and `seed`.

### 14.2 Metrics
```cfdl
metric npv_base = cel "npv(model.cashflows, discount=assume('discount_rate'))"
metric irr_equity = cel "irr(stream('equity_cashflows'))"
metric prob_irr_gt15 = cel "prob(metric('irr_equity') > 0.15)"
```

Rules:
- Metrics are named expressions.
- In deterministic runs, metrics evaluate to scalars.
- In Monte Carlo runs, metric values produce distributions and summary stats.

---

## 15. Expressions (CEL-compatible)

### 15.1 Expression syntax
Expressions are written as:

```cfdl
cel "<expression>"
```

Expressions MUST be:
- side-effect free
- deterministic given the same inputs
- terminating

### 15.2 Namespaces and built-ins (normative)
The expression environment MUST support:

**Model/time**
- `t.index`, `t.date`, `t.year_index` (minimum)

**Inputs**
- `inputs.<name>` for stochastic assumptions
- `assume('<name>')` for deterministic/stochastic values

**Entities**
- `entity('<symbol>')` returns an entity view
- `entity('<symbol>').<attr_or_state>` accesses attributes/state

**Observables and references**
- `obs('<OntologyId>')` returns an observable value at `t.date` (or a series)
- `ref('<OntologyId>')` returns a reference object (table/static)

**Streams**
- `stream('<name>')` returns a stream series
- `model.cashflows` returns aggregated model cashflows (Money series)

**Finance/math**
- `npv(series, discount=<rate>)`
- `irr(series)`
- `convert(money, '<currency>')`
- `fx('<from>','<to>', date)` (optional in v0.1; recommended)

**Probability (MC)**
- `prob(condition)`

### 15.3 Currency literals
The language MAY support syntactic sugar:
- `42000 USD` as Money
- `10%` as Rate

These MUST compile to typed values in IR.

---

## 16. Canonical JSON IR requirements (high level)

### 16.1 Single IR
- The compiler MUST emit one canonical JSON IR.

### 16.2 Preserve provenance
- The IR MUST preserve:
  - all entities
  - all contracts (including terms)
  - all streams (explicit and derived)
  - provenance links from derived artifacts back to their contract source

### 16.3 Required external inputs
- The IR MUST include lists of required inputs inferred from expressions:
  - `required_observables`: list of ontology observable ids referenced via `obs('...')`
  - `required_refs`: list of ontology ref ids referenced via `ref('...')`

### 16.4 No correlation
- The IR MUST NOT contain any correlation field/slot.

---

## 17. Reserved keywords (v0.1)

`version`, `model`, `use`, `pack`, `import`, `as`, `time`, `calendar`, `from`, `for`, `phase`, `to`,
`entity`, `contract`, `on`, `term`, `currency`, `terms`, `effects`, `stream`, `owner`, `direction`,
`inflow`, `outflow`, `schedule`, `on`, `every`, `day`, `eom`, `week`, `Mon`, `Tue`, `Wed`, `Thu`, `Fri`, `Sat`, `Sun`,
`convention`, `calendar`, `stub`, `except`, `also`,
`assume`, `run`, `deterministic`, `monte_carlo`, `trials`, `seed`,
`metric`, `cel`,
`event`, `when`, `set`, `activate`, `deactivate`,
`option`, `type`, `exercisable`, `exercise`, `payoff`.

---

## 18. Minimal multi-file example (Core)

**model.cfdl**
```cfdl
version 0.1
model "Sunset" currency USD
use pack "evs/cre" version "0.1"

import "time.cfdl"
import "structure.cfdl"
import "assumptions.cfdl"
import "behavior.cfdl"
import "metrics.cfdl"
```

**time.cfdl**
```cfdl
time calendar monthly from 2026-01-01 for 120
phase construction from 2026-01-01 to 2026-12-31
phase perm from 2027-01-01 to 2031-12-31
```

**structure.cfdl**
```cfdl
entity asset Sunset : CRE.Asset { city "Austin" units 24 }
entity loan Senior : Debt.Loan { principal 8_500_000 USD index "SOFR_1M" }
```

**assumptions.cfdl**
```cfdl
assume discount_rate = cel "0.10"
assume rent_growth ~ Normal(mean=0.03, stdev=0.01, clip=[-0.02, 0.08])
```

**behavior.cfdl**
```cfdl
contract Contract.Lease L1
  on entity asset.Sunset
  term 2026-02-01 .. 2028-01-31
{
  currency USD
  terms { base_rent 42000 USD }
  effects {
    stream rent owner entity asset.Sunset direction inflow currency USD {
      schedule every month on eom from 2026-02-01 to 2028-01-31 convention modified_following calendar "NYSE"
      amount cel "terms.base_rent"
    }
  }
}

contract Contract.Loan D1
  on entity loan.Senior
  term 2026-01-01 .. 2031-12-31
{
  currency USD
  effects {
    stream debt_service owner entity loan.Senior direction outflow currency USD {
      active when cel "entity('loan.Senior').status != 'refinanced'"
      schedule every month on eom from 2026-01-01 to 2031-12-31 convention modified_following calendar "NYSE"
      amount cel "/* engine-provided pmt(...) */ 0"
    }
  }
}

event refi_if_rates_drop when cel "obs('Rates.SOFR.1M') < 0.035" {
  set entity loan.Senior.status = "refinanced"
}
```

**metrics.cfdl**
```cfdl
run deterministic
run monte_carlo trials 20000 seed 42

metric npv_base = cel "npv(model.cashflows, discount=assume('discount_rate'))"
```

---

## 19. Conformance
An implementation conforms to CFDL v0.1 Core if it:
1. Parses valid CFDL programs per this spec.
2. Rejects invalid programs with actionable diagnostics.
3. Validates strong types and required fields.
4. Emits deterministic canonical IR that preserves contracts/streams/provenance.
5. Supports the schedule primitives and discrete event semantics.
6. Supports CEL-compatible expressions and the required namespaces/functions.

