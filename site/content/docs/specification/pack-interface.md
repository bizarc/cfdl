---
id: pack-interface
title: "Pack interface (v0.1)"
slug: "/docs/specification/pack-interface"
description: "The contract a domain pack is written against: type registries, aliases, lowering rules, metrics, and validations."
source: docs/07_pack_interface.md
generated: full
layer: specification
---

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
    metrics.toml
    statements.toml
    validations.toml
    ontology/
      types.toml
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
templates = "templates.toml"
lowering = "lowering/rules.toml"
metrics = "metrics.toml"
statements = "statements.toml"
validations = "validations.toml"
ontology = "ontology/types.toml"
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
  recognized keys are `aliases`, `templates`, `lowering`, `metrics`,
  `statements`, `validations` and `ontology`, each a path relative to the
  pack directory. An unrecognized
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

Minimum shape — `ontology/types.toml`, the file the loader reads (an
earlier revision showed a JSON registry no loader ever parsed):

```toml
[pack]
ontology_id = "cre"
version = "0.1.0"

[[entities]]
type_id = "CRE.Asset.RealProperty"
family = "asset"
class = "real"
refines = "Asset.Real"

[[entities.fields]]
name = "rentable_area"
field_type = "decimal"
required = false
unit = "sf"

[[relations]]
relation_id = "occupies"
from_family = "party"
to_family = "asset"
cardinality = "many_to_many"
inverse = "occupied_by"
```

Type registry semantics:
- Packs MUST NOT remove core types.
- Packs MAY extend fields.
- Packs MAY provide documentation strings and examples.

**Refinement (`refines`).** A pack type MAY declare the master type it
specializes — the language base's, or another type in the same pack:

```toml
[[entities]]
type_id = "CRE.Asset.RealProperty"
family = "asset"
class = "real"
refines = "Asset.Real"

[[entities]]
type_id = "CRE.Asset.Unit"
family = "asset"
class = "real"
refines = "CRE.Asset.RealProperty"   # chains are fine; they end at a master
```

Recorded rather than conventional, so "is a" is a fact the system can read:
selection by a base type reaches every refinement transitively
(`PackOntology::is_a`, called on the base-merged view), and a metric or
validation written against `Asset.Real` survives a new pack unchanged.

Rules, checked at pack load:
- The target MUST exist — in this pack or the language base.
- A refinement stays in its **family**, and an asset refinement keeps its
  master's **class**: what a thing is does not change by specializing it.
- Single parent, no cycles. A chain ends at a type that refines nothing —
  a master type.
- **Fields inherit down the chain.** A refinement carries every master
  field without restating it, and the compiler checks a model against the
  EFFECTIVE roster — required fields, near-miss detection and the declared
  list all include the masters'. A redeclared name is the same fact
  restated: a refinement MAY strengthen it (an optional master field
  becomes required — the move `CRE.Asset.Unit` makes on `rentable_area`)
  and MUST NOT retype, re-unit, or weaken it. A reader who learned a field
  from the master must not be lied to by the refinement.

**Families.** An `entity` declaration takes one of three families — `asset`,
`party`, `container` — and the graph holds five node families: those three
plus `contract` and `reference`, which are nodes (relation endpoints,
identity-bearing) though they are declared with `contract` and
`curve`/`quantile` rather than `entity`. A container groups and scopes — a
fund, a portfolio, an SPV, a transaction. It holds cash-producers; cash
attached directly to it is deal-level cash, real and aggregated with its
members'. Only assets carry a `class`. The language base ships
`Container.Fund`, `Container.Portfolio`, `Container.SPV` and
`Container.Transaction` for packs to refine.

**Relations.** An endpoint names one node family or a list
(`from_family = ["asset", "container"]`); a listed pair is a cross product.
The base vocabulary: `part_of`/`contains` (hierarchy AND containment — one
concept, widened endpoints), `owns`/`owned_by`,
`secured_by`/`secures` (contract→asset, collateral),
`guarantees`/`guaranteed_by` and `is_counterparty_to`/`has_counterparty`
(party→contract). Base relations are declarative today: validated,
published, no engine semantics.

Contract types take the same field, and the language base ships the abstract
masters they refine (`docs/40`): `Contract.Debt`, `Contract.Lease`,
`Contract.Purchase`, `Contract.Sale`, `Contract.Offtake`, `Contract.Service`,
`Contract.Tax`, `Contract.Option`, `Contract.Construction`,
`Contract.Derivative`, `Contract.Insurance`, and for the statement-line
generators `Contract.Revenue`, `Contract.Expense` and
`Contract.CapitalExpenditure`. A master is `abstract = true`: it exists to
be refined, binds no lowering rule (refused at load if it does), and cannot
be instantiated — a model that names one on an `option` is refused. The
roster is indicator-based and extensible; absence of a refinement in
today's packs is not evidence a master is unneeded.

**A master is defined from what the agreement IS, and a pack conforms to
it.** A master declares its roles, its fields (the terms — there is no
separate term schema), the LINES of cash its refinements produce, and
which SIDE the subject is on. A refinement inherits all of it, may
strengthen a field or add one, may specialize a role (`landlord` refines
`lessor`; a domain word never appears on a master), may add lines, and
may not retype, re-unit, weaken or drop anything the master declared.
Pack load checks that a concrete type's rules emit every effective line
(each rule names its `line`) and that its template renders every required
effective field. Categories stay the pack's (§6.10): the master says a
debt produces interest, the pack says where a borrower's interest sits in
the statement. The roster, the argument for each core, and the staged
delivery are in `docs/40_master_contracts.md`.

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

**Arrival actions.** A machine MAY carry what happens on arrival — the same
two grains a model declares (`docs/34` D2, D3):

```toml
[[lifecycles.entry_actions]]
state = "leased"
description = "True of the STATE however it was reached."
actions = [{ set = "months_in_state", value = "0" }]

[[lifecycles.transitions]]
from = "holdover"
to = "leased"
actions = [{ set = "in_place_rent", value = "prev.in_place_rent * (1 + inputs.bump)" }]
```

- **Entry actions** carry what is true of the STATE however it was reached,
  and are the primary domain spelling: a pack declares them once and every
  entity of the type inherits them, including for an edge added later.
- **Transition actions** carry what is true of the PATH taken. A renewal and
  a re-let both land in `leased` and strike rent differently; an entry action
  cannot say that, because it does not know which edge fired.
- Both run on EVERY traversal, including one a model's event causes by
  writing `status` across a permission edge. Entry actions run first, then
  the taken edge's — the specific refines the general — and a same-field
  write journals the earlier value `overridden`, naming its author.
- `set` writes a FIELD and never `status`, refused at pack load: a status
  write would fire a second transition inside the same period. A transition
  that should cause another transition is an edge out of the target state,
  taken next period.
- The field name is **entity-relative**, refused at pack load if qualified:
  one lifecycle is bound by many entities, and the behavior belongs to
  whichever one transitioned.
- **The field itself comes from the pack's lowering rules**, like every other
  pack-populated field (`field_name` / `field_init` / `field_next` in
  §7). Those run per contract instance, so whether a given entity has the
  field is a fact about the model, not about the pack — an action naming a
  field the entity does not have is skipped with a warning at run.
- A model MAY add its own actions to a pack's machine, additively
  (`docs/34` D2a): a `lifecycle <pack machine>` block contributes actions and
  may not state `initial`, `state` or an edge (`E1357`). The model's actions
  run after the pack's, so the model wins a same-field conflict and the
  pack's value is what journals `overridden`.

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

### 6.3 Contract terms are fields

There is no separate term schema. A contract type's terms are its FIELDS,
declared on the master and inherited down the refinement chain exactly as
an entity's are (§6.1), in `types.toml`:

```toml
[[contracts]]
type_id = "CRE.Contract.PermanentDebt"
refines = "Contract.Debt"
contract_name = "cre.permanent_debt"

[[contracts.fields]]        # strengthens an inherited master field
name = "amortization_months"
field_type = "integer"
required = true
unit = "months"

[[contracts.roles]]         # specializes a master role (docs/40 §5)
name = "landlord"
refines = "lessor"

[[contracts.roles]]         # a master role this form of the agreement leaves unbound
name = "buyer"
unbound = true
```

A field may carry `one_of = "<group>"`: fields sharing a group are
alternatives and a contract must state at least one of them (a debt's
amount is `principal`, `commitment` or `draw_curve`; its rate is
`interest_rate` or `index_curve` with `margin`). A refinement may put a
field of its own into a master's group — a rollover's `renewal_rent_year`
and `market_rent_year` join the lease's `rent` group, a percentage-rent
clause's `overage_pct` does too, a capex line's `pct_of_revenue` joins
`amount`. The master's obligation stands (a lease states its rent); the
refinement states how this form of the agreement spells it. A refinement's
roles are its master's roles, specialized — it may not add a party the
agreement does not have.

Every term a pack's rules read (`{{contract.<key>}}`, and `{{periods.<key>}}`
through the months-to-periods conversion), every term its validations bound
and every term its templates render is a field of the type — declared on
the type or inherited from its master — and the loader refuses a pack where
one is not. The shipped packs declare every term they use.

`parties = ["lender", "borrower"]` stays the shorthand for roles inherited
by the master's own word. A master also declares `lines` (`[[contracts.lines]]
name = "interest"`) and, where it serves one side only, `side = "pays"` or
`"receives"`; a refinement inherits both, may add lines, and fixes a side
the master left open. Each lowering rule names the line it emits
(`line = "interest"` on the `[[rules]]` entry), and load checks that a
type's rules cover its effective lines. Every shipped rule names its line. A
contract template must render every required effective field and one member
of each `one_of` group, also checked at load.

The master's fields are the schema; the lowering rule consumes them by
name (`{{contract.principal}}`); the template renders the required ones;
`validations.toml` bounds their values. The compiler checks a model's
`terms` against the EFFECTIVE roster: an unknown term is refused with a
near-miss hint (`E1371`), a missing required field or an empty group is
refused (`E1372`), and a unit stated on a term is checked against the
rule's (`E5024`). A rule that consumes a
term the type does not declare is a pack-load error, so the three sources
that once had to agree by care — rules, templates, validations — are
checked against one declaration. See `docs/40` §3 and §8.

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
folded. A statement itemizes by selecting streams; a subtotal folds by
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

**A refinement when the cash shape differs; a term when only the
settlement differs.** Credit's level-pay, interest-only and floating pools
are three refinements because the amortization profile and the rate basis
differ. An interest-only period, a balloon, capitalized interest and a
PIK period are TERMS on one instrument (`interest_only_months`, `balloon_at_maturity`,
`capitalize_interest`, `pik_months`) because they change when the same
instrument settles, not what it is. Under that rule the roster grows by
instrument and the terms stay what a term sheet states.

**Every rule names the line it emits.** `line = "interest"` on a
`[[rules]]` entry ties the stream to a line the contract's master declared
(`docs/40` §6). Pack load checks coverage; the category the stream carries
is still the pack's classification of that line.

**Scheduled cash is the contract's; discretionary cash is the
waterfall's.** A debt contract lowers its draws, interest and scheduled
principal and carries its own balance as a lowered field. Repayment that
depends on what cash is available — a sweep, proceeds of a sale — is a
waterfall step allocating to the lender's party account, and the
contract's balance never reads the waterfall (`docs/13` §7.97). Where a
deal repays from a named cash source, the contract takes a series or
curve reference as a term, as `cre.construction_loan` takes its draw
curve.

**A contract paying finer than the model calendar aggregates into the
period rather than being refused**, and its day count is a year fraction
the placeholder expands to, so a monthly-paying mortgage runs on an
annual model and act/act falls out of the same expansion (`docs/13`
§7.16, §7.57). Statutory or workbook rounding belongs on the rule as a
`round_step` term, never in a case's hand stream.

**Parties in roles are real.** A model's `parties { landlord = party.acme
}` is validated against the type's effective roles — the master's,
specialized by the pack — and carried into the IR under both the pack's
word and the master role, so `party.acme` is a lessor to any reader that
does not know CRE.

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

### 6.6 References (market observables)
A pack's ontology declares its references in `types.toml` —
`[[references]]` entries with a `reference_id`, a `kind` (`rate_curve`,
`index`), an optional `unit` and a description. They are vocabulary: what a
model using the pack may observe, resolved at run time from the model's own
`curve`/`quantile` declarations and the run configuration.

Earlier revisions of this section described `observables.json` and
`refs.json` registries and an `obs.rate(...)` accessor family. No loader
has ever read such files and no such accessors exist; the expression
environment's observable surface is `cfg.*`/`obs.*` bindings and
`curve_value` (see the Expression Environment).

### 6.7 Expression functions

The expression function vocabulary is fixed and built into the engine
(`cfdl-calc`) — `pmt`, `year_frac`, `cpr_to_smm`, `curve_value`, and so on
(see the [Expression Environment](/docs/specification/expression-environment)).
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
names the contract it applies to — `contract` for one, `contracts` for a
list, exactly one of the two — the check, a stable diagnostic code, a
message, and an optional `severity` (`error` by default). `term` names the
term under test; `terms` lists them for `any_term_present`; `values` lists
what `term_enum` accepts; `left`/`op`/`right` are `term_compare`'s
operands. Available checks: `term_present`, `any_term_present`, `terms_mutually_exclusive`, `term_number`
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
  "financing.debt.service",
]
```

The list is a recommendation, not a gate. The three roots are what the language
validates, with or without a pack, so a model may name a leaf the pack never
enumerated — a departmental operating expense, an acquisition basis — and it
folds exactly as a listed one does. A well-rooted category the pack does not
list raises `W5023` naming the near match, which is what keeps two models in one
pack from spelling the same idea two ways.

A pack cannot enumerate every leaf a deal needs, because the leaf is not
knowable by a pack that shipped before the deal. What the list carries is the
domain's conventional spelling.

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
categories = ["financing.debt.service", "financing.debt.mortgage_insurance"]

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
`fee_year`), which spread by `{{model.periods_per_year}}` regardless of day
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
