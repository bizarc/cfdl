# Entity-owned properties — plan

Status: **proposal**, except stage 0, which has shipped.

A balance belongs to something. A term loan's balance is a property of the term
loan; a pool factor is a property of the pool; available funds are a property of
the trust's collection account. Today the language says none of that: those
quantities are model-level `state` declarations, floating free of the things
they describe.

This plan moves them onto the entities that own them, and retires the
model-level form.

## 0. Attribute binding — shipped

A declared attribute read as **zero** in every expression that touched it.

```cfdl
entity asset tower : CRE.Asset.RealProperty { rentable_area = 30000 }
amount = entity.state.rentable_area * 2.50      // was 0.00
```

It parsed. The ontology validated the field — misspell it and `E1313` rejects
the model. The IR carried it. And the engine's IR type did not deserialise
`attrs` at all, so the value never reached the expression environment. The
ontology checked the name; nothing checked the value arrived.

Fixed: attributes seed the entity's property map from period 0, parsed to a
number where they look like one. `fixtures/valid/entity_attributes_read` covers
it. Every existing golden and benchmark is unchanged, because nothing could read
these values before.

This is the prerequisite for everything below — an entity could not own a
property it was unable to expose.

## 1. What is actually in use

**42 `state` declarations across 19 models** as of the tax-equity flip case, up
from 29 across 15 when this was written, plus 21 pack lowering rules that emit
one. The count has grown because the waterfall fixtures and the flip case all
reach for `state`, which is the argument for this work rather than against it.

They fall into three groups, and the groups want different answers.

**A. A balance that belongs to a thing** — 11 declarations.

| state | belongs to |
|---|---|
| `tlb_balance`, `sub_balance` | the debt tranche |
| `cum_required` | the development |
| `reserve_line` | the property's reserve account |
| `exit_equity`, `value_per_share` | the enterprise |
| `book_value`, `cum_capex` | the asset |

Each has an obvious owner, and several already have an ontology type waiting for
them — `Credit.Asset.Tranche` declares `original_balance` and `seniority`.

**B. A prior value of a group-A quantity** — 4 declarations.

`tlb_balance_open`, `sub_balance_open` exist only so a stream can see both ends
of a period. They are not quantities; they are a workaround for a missing
accessor. If an entity property supports `prev`, all four disappear.

**C. An index, a rate path or a factor** — 14 declarations.

`revenue_index`, `lagged_index`, `monthly_idx`, `quarterly_idx`, and the four
escalating `opex_*` lines in the HUD case. These are **references** — the fourth
ontology family — and §5 works through which of them are curves and which are
reference properties.

The 21 pack-emitted states are survival factors: a property of the pool, so
group A.

## 2. The surface

A property with a recurrence, declared on the entity that owns it:

```cfdl
entity asset tlb : Credit.Asset.Tranche {
  seniority = 1
  balance init 275.0
          next max(0.0, prev - cfg.sweep)
}
```

`init` and `next` keep the meaning they have today, and the same no-`=` spelling
the grammar already uses. What changes is where they hang.

Reading, from anywhere:

| accessor | meaning |
|---|---|
| `asset.tlb.balance` | this period's value, at period close |
| `entity.asset.tlb.balance` | the same read, spelled the long way |
| `prev asset.tlb.balance` | the prior period's — retiring every `_open` state |
| `entity.state.balance` | the owning entity's, inside its own stream |

**The bare form resolves** — shipped ahead of the rest of stage 3. An entity's
properties are bound under its family, so `asset.tlb.balance` and
`entity.asset.tlb.balance` name the same read and only a declared family is
aliased. This repo's own documentation taught the bare form in every waterfall
example before it worked, which is a reasonable signal about which spelling is
the natural one.

## 3. Why this is better than what it replaces

**It says what the quantity is.** `state tlb_balance` names a variable;
`asset.tlb.balance` names a fact about a loan.

**Hierarchy and rollups come free.** A tranche is `part of` a deal, so a deal's
balance is its tranches' balances by the same relation that already aggregates
cash — proved by `benchmarks/credit/mbs_pool_by_loan`.

**The ontology can check it.** A pack declares which properties a type has, so a
misspelling fails at compile time. A model-level `state` name is checked against
nothing.

**It is what the waterfall needs.** `from asset.trust.available_funds` says where
the pot comes from. `from state.available_funds` says a variable exists.

## 4. Sequence

1. ~~Attribute binding~~ — **shipped**.
2. ~~The bare accessor~~ — **shipped**. `asset.tlb.balance` resolves as an alias
   for `entity.asset.tlb.balance`.
3. **Recurrence on an entity property** — `init`/`next` in an entity block.
   Parser, IR, engine. The engine already evaluates recurrences; this changes
   where it reads them from, not how it solves them.
4. **`prev` on an entity property.** Removes group B outright: four states, no
   model rewrites beyond deleting them.

   **This cannot precede the recurrence**, which the first version of this plan
   had backwards. `prev asset.tlb.balance` requires the property to be a
   recurrence, so the cheap stage depends on the expensive one and there is no
   small opener to this work.
5. **Migrate group A** — 11 declarations across 6 models. Each is a rename plus
   an owner. Benchmarks are the proof: identical numbers or the migration is
   wrong, the same bar the `legal` retirement met.
6. **Pack lowering emits entity properties** rather than model states. 21 rules,
   and it changes the pack interface, so the pack interface spec and its
   conformance check move with it.
7. **Migrate group C onto references** (§5) — curves where the values are
   known, a reference property with a recurrence where they compound.
8. **Retire `state`** — reject it, migrate the remaining fixtures, and update
   the language guide, the object-model page, the specification, the grammar and
   `docs/14`.

Stages 3–5 are independently landable and each keeps the suite green. Stage 6 is
the one that touches the pack contract, and stage 8 is the breaking change.

## 4a. What a property read means inside another property's `next` — settled

The question stage 3 cannot avoid: inside `asset.tlb.balance`'s `next`, what
does `asset.sub.balance` mean — this period's value or last period's?

**Settled: it means neither, because it is rejected.** Inside a `next`, a bare
property read is a compile error naming the fix, and the previous period is
spelled explicitly:

```cfdl
entity asset tlb : Credit.Asset.Tranche {
  balance init 275.0
          next prev - min(prev, cfg.sweep + prev asset.sub.balance * 0.0)
}
```

Three reasons, in order of weight.

**One spelling, one meaning.** Everywhere else in the language
`asset.tlb.balance` means *this period's value, at period close*. If the same
text meant *last period's* inside a `next`, the meaning of a read would depend
on where it sits — which is the class of trap this whole document exists to
remove. A construct that cannot mean the usual thing should say so rather than
quietly mean a different one.

**It keeps the order an order.** Allowing current-period reads between
properties makes the answer depend on a topological sort of the recurrences,
and a cycle then needs a solver. The waterfall settled the house rule already —
an order, not a graph — and model-level `state` follows it today.

**It is the rule `E1126` already established.** A state's `init` may not read
another state, because at period 0 there is nothing there to read; the answer
was a diagnostic rather than an evaluation order. This is the same shape one
period along, and it should have the same answer and a neighbouring code.

### The hazard this closes

Stage 2 aliased bare family paths onto the `entity` root, which is
**open-world** — a missing key resolves to null rather than failing, because an
event-written status field does not exist until an event writes it.

Left alone, that would make a bare read inside a `next` return null *silently*
and evaluate to zero: precisely the failure mode of `init = 100`, of the
unbound entity attributes, and of the bare waterfall path — all three of which
this project has already had to find the hard way. The compile-time rejection
closes it before stage 3 can open it.

It also argues for a narrower rule later: a property **declared by the
ontology** is knowable at compile time, so a typo in one should be an error
rather than a null, and open-world behaviour should be reserved for lifecycle
status fields. That is a separate change and is noted here so it is not lost.

### Spelling

`prev asset.tlb.balance` — the prefix form, as §2 proposes. `prev.<name>` stays
for model-level states until stage 8 retires them, so the two coexist for the
duration and neither is ambiguous: `prev` takes a path, `prev.` takes a state
name.

## 5. Group C is the reference family

Group C is not a third category. An index, a rate path, a price deck — these
are **references**, the fourth ontology family, and the packs already declare
them:

```toml
[[references]]
reference_id = "energy.inflation"
kind = "index"
unit = "ratio"
```

So a compounding `revenue_index` is not a nameless model variable; it is an
index reference, and it belongs to the model the same way a tranche balance
belongs to a tranche. That makes the rule uniform: **every quantity belongs to
something, and a reference is one of the things it can belong to.**

Two mechanisms cover the group, and which one applies is decided by whether the
values are known:

- **Known values → a `curve`.** A curve is already a declared, date-indexed
  reference. `monthly_idx` and `quarterly_idx` are calendar factors with stated
  values and want nothing more than this.
- **Compounding values → a reference entity with a recurrence.** An index that
  rolls forward off a rate is a recurrence, and it uses exactly the mechanism
  §2 gives an asset. A reference is an entity; entities own properties; a
  property may have an `init` and a `next`.

The HUD `opex_*` lines are the interesting case, and they land in neither: they
are recurrences *only* because the source escalates an already-rounded figure
each year. That is a convention of the reference being reconciled against, not a
property of the building — so they become an escalation reference the four lines
read, rather than four separate recurrences.

**This closes the question §4 stage 6 was reserved for.** With references
carrying group C, no quantity in the repository needs a model-level `state`, and
stage 7 becomes a straightforward removal rather than a decision. The staging
still holds — group C is migrated last, because the reference surface wants the
entity-property work from stages 2 and 3 underneath it.

## 6. What has to be updated when this lands

- **Models**: 15 under `benchmarks/`, `examples/`, `fixtures/`.
- **Packs**: 21 lowering rules in `credit` and `opco`, plus the pack ontologies
  that must declare the new properties.
- **Schemas**: the IR entity gains properties with recurrences; `docs/05` and
  the published IR schema move with it.
- **Site**: the language guide's State section, the object-model page, the
  expression reference's binding table, and `docs/14`, whose §5 boundary is
  about this construct.
- **Specification**: `docs/01` and the grammar.

Every one of those is generated from or checked against the repository, so the
existing gates catch a miss: `sync:check`, `check-doc-examples`, the snippet
parser, and the golden and benchmark suites.
