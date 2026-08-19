# Claims — design

Status: **proposal.** Nothing here is implemented.

A **claim** is a balance-bearing obligation held by a party and reduced by
being paid. A note class, a tranche, a term loan, a preferred return, a
deferred fee, a handback reserve: each is one number that exists because of
prior distributions and constrains later ones. The language allocates cash to
claims already; it does not carry them.

This document argues that claims belong in the language rather than in a pack,
records what the object model already commits to, audits what the word "claim"
currently means in published material, and lists what would have to change.

---

## 1. The asymmetry that produced this

The credit pack has contracts for assets and none for liabilities.

`credit.pool_level_pay` is a contract type. Given a balance, a rate, a term, an
age and a speed, the pack lowers it into six streams — interest, scheduled
principal, prepayment, recoveries, servicing, penalty — and maintains one
field for pool survival. The pool has a balance because the contract computes
one, and that balance is a closed form of the terms: original x survival(t) x
amortization(t). Nothing outside the contract can change it, and nothing needs
to.

`Credit.Asset.Tranche` is not a contract. It is an ontology entity type whose
whole definition is a description — "A claim of stated seniority on a pool's
cash" — and two optional fields, `seniority` and `original_balance`. No
lowering rule names it. It has no balance over time; `original_balance` is a
static term, and `seniority` is documentation. It exists so that a waterfall
step has something typed to pay.

So the two do not both have balances in any sense that matters. One is
computed and one is written down.

The consequence is visible in the benchmark suite, where three cases have each
built a liability stack by hand, three different ways:

| case | how the note balances are carried |
|---|---|
| `credit/fnma_remic_2019_2_g3` | an entity field per class, restating the pool's amortization because a pass-through class's balance *is* the pool's |
| `credit/auto_abs_tranches` | no balances at all — each class's pay-down is a closed form of cumulative pool principal, clamped between two constants |
| `credit/americredit_2017_1` | seven entity fields that recompute the entire distribution one period lagged, because the split depends on cash rather than on collateral |

The third is the one that shows the problem clearly, and it is worth being
precise about what went wrong in it rather than about what might. That case
computes its distribution twice: once inside the waterfall that pays the cash,
and once inside the recurrence that carries the balances, because the
recurrence cannot see what the waterfall paid. The two copies disagreed. The
waterfall paid the pack's own servicing series, which charges a January-cutoff
pool for two months in the first collection period; the recurrence carried its
own copy of the fee and charged one. Eleven published cells and two published
weighted average lives were wrong, the grid was wrong in a way no assertion in
the case could catch, and the model's own cash was right the whole time.

The defect was not in the cash flow. The collateral is produced by contracts
from inputs alone and no allocation feeds back into it, which is the layering
a model should have. The defect was in the distribution layer having its state
kept somewhere else.

---

## 2. What already works, and why

Liabilities are not unmodelable today, and it would be wrong to write this
document as though they were. The CRE pack amortizes loans, the cases that use
it reconcile against published sources, and none of it needs anything proposed
here.

**`cre.permanent_debt` carries no balance at all.** Its one stream is debt
service, computed in closed form: interest-only while `elapsed < io_months`,
then `-pmt(rate, amort_months, principal)`, plus an optional balloon from
`-fv(...)` at maturity. The loan amortizes — correctly, to the cent, on any
calendar — and the outstanding balance is never a value in the model. It does
not need to be, because every quantity anyone asks of a mortgage is a function
of the terms and the period.

**`cre.construction_loan` carries exactly one field**, and its own comment says
why: "Cumulative funding required is the only quantity that cannot be
recomputed from the period alone, so it is the only recurrence. The equity/debt
split, the opening balance and the interest all fall out of it with `min` and
`max`." The balance of that facility *rises* as it draws, and that is expressed
today. The driver is a declared curve — data, indexed by date.

Both are liabilities reduced or increased by streams, and both work. The reason
they work is worth stating exactly, because it is the whole boundary:

> Their balances are driven by **time**. A mortgage amortizes because periods
> pass. A construction facility draws because the draw curve says so on that
> date. Neither balance depends on what any distribution paid.

The pack's convention here — closed form wherever possible, one recurrence only
where a quantity genuinely cannot be recomputed from the period — is good
engineering and produces small, cadence-neutral contracts. It is also precisely
what makes the balance unavailable to an allocation: there is no balance object
for a waterfall to pay down, because the arithmetic never needed one.

That convention is optimal for the scheduled case and awkward for the allocated
one, and the awkwardness is not confined to credit. The CRE JV promote fixture
carries a preferred return this way:

```cfdl
msgw_preference init inputs.msgw_capital next prev * (1.0 + inputs.pref_rate)
...
pay msgw_preference to party.msgw = min(asset.jv.msgw_preference, remaining)
```

The preference accretes, and the step pays what the pot allows — but the field
never learns what was paid. If the pot is short, the preference compounds next
period on the full balance as though nothing had been distributed. In a fixture
about expressiveness that is harmless. In a deal it is a preferred return that
cannot be paid down, and it is the same missing edge as a note class, in a
different pack, with no collateral anywhere near it.

## 3. Precisely what is missing

The pack maintains a pool's balance without difficulty, and a mortgage's
without needing one. A liability's balance is not harder arithmetic; the
question is what *drives* it.

> A balance driven by **time** — a schedule, a rate, a speed, a declared
> curve — is expressible today, and three packs do it well. A balance driven by
> an **allocation** is expressible nowhere, because an allocation is not a
> function of the period.

Everything else follows from that one distinction. The workarounds in the
table above are three ways of pretending an allocation is a schedule: restate the
schedule the allocation happens to follow (Fannie Mae), find the closed form
that happens to exist (Ally), or recompute the allocation in a place that can
carry state (AmeriCredit).

Three probes fix the boundary of what exists today. Each was run against
`target/debug/cfdl` and each result is a fact about the engine rather than
about the specification:

| reader | reads | result |
|---|---|---|
| a stream | another stream's series, `0..t-1` | works |
| a stream | a waterfall step's series | zero, and silently |
| a field's `next` | any series at all | zero, with a warning |
| a stream that reads a series | another stream that also reads a series | refused at load: *"A cross-stream read can only see streams that read none"* |
| a waterfall step | `paid.<step>`, `owed.<step>`, `remaining` of an **earlier step, same period** | works |
| a waterfall step | `paid.<step>` of a **different waterfall** | refused: `E1341_WATERFALL_FORWARD_REF` |

Read together: allocation within a period is a solved problem, and the engine
enforces its ordering properly. What is absent is any path from an allocation
to the next period. That is the whole gap.

---

## 4. The object model, confirmed

The proposal does not need a new object model. It needs the existing one
applied to liabilities.

**Entities are the cast.** The learn material states the test directly: an
entity is a thing that "could plausibly have their own statement of cash
flows," and its examples are "the building, the operating company, **the
loan**, the counterparty." A liability is already an entity in the object
model's own terms. The credit ontology agrees in structure — it has
`Credit.Asset.Tranche` — while disagreeing in family, which is worth noting:
the type is filed under `family = "asset"`, so a note is modeled as the
holder's asset and never as the trust's liability. Both views are legitimate
and the ontology currently supports only one of them.

**Streams are dated cash.** A stream is not a total; it is a set of dated
amounts owned by an entity. That does not change.

**Waterfalls allocate a pot.** A waterfall names the entity whose cash it
allocates, states a `from` expression, and pays ordered steps. That does not
change either.

**Cash is what moves between entities; a claim is the balance the movement
changes.** This is the piece with no representation today. An asset's balance
falls because its contract says time passed. A liability's balance falls
because cash reached it. The first is expressible and the second is not, and
the difference is not a matter of degree.

So the answer to "are we modeling entities, including assets and liabilities
and the cash that impacts them both" is: entities and cash, yes; liabilities
as first-class balance-bearing things, no — they are modeled as untyped
entities carrying hand-maintained fields, or as arithmetic inside a waterfall
step.

---

## 5. What the word "claim" means today

The word is already load-bearing in published material, in **three different
senses**, none of which is the object this document proposes. Any decision to
use it as a construct name has to reckon with that first, and the terminology
register — which exists precisely to enforce one word, one meaning — does not
currently list it.

1. **A claim is a stream.** Chapter 2: "The rent number is not an entity; it is
   a claim *about* an entity." And: "the model stores the dated claims." And:
   "classify every stream as one of the three kinds of claim it can be —
   scheduled, recurring, contingent."
2. **A claim is a waterfall step.** Chapter 11: "a pot of cash, a sequence of
   claims, each claim taking what it is owed until the pot runs out." Also "a
   junior fee is deferred when funds run short; the deferral claim ranks..."
3. **A claim is an assertion.** Chapter 18: "every convention is a claim, so
   state it where a reviewer will look." This is the ordinary English sense and
   it is used deliberately.

Sense 2 is the one a reader is most likely to mistake for this proposal, and
it is the one to watch. "Each claim taking what it is owed" is true of a step,
but what a step is owed is an expression the author writes on the spot. There
is no object that knows what it is owed, and no balance that remembers what it
was paid. A reader who has met chapter 11 and then reads that the language
"supports claims" will reasonably conclude something stronger than the truth.

The published documentation does **not** overclaim. The site's own page for
`credit/auto_abs_wal` says outright that "this pack models the collateral
rather than the liability stack," and that a sequential-pay liability waterfall
is something "this pack does not model at all." That is accurate and it is the
right posture. The exposure is not a false statement anywhere; it is a word
doing three jobs, with the most prominent of the three sitting one inch from
the thing it is not.

Two options follow, and this is a decision rather than a technicality:

- **Register `claim`** in `docs/terminology.toml` with the balance-bearing
  meaning, and reword senses 1 and 2 in the learn chapters — "dated claims"
  becomes "dated amounts," "a sequence of claims" becomes "a sequence of
  steps." Sense 3 is ordinary English and can stay, since the register governs
  technical vocabulary rather than every use of a word.
- **Pick another name** — `obligation`, `tranche`, `liability` — and leave the
  learn prose alone. Each has its own collision: `tranche` is credit-specific
  and this construct is not, `liability` presumes a balance-sheet direction
  that a preferred return does not obviously have, and `obligation` is a
  mouthful in every step it would appear in.

The recommendation is the first. The word is right, and the register exists to
make a word mean one thing.

---

## 6. The design

### 6.1 What a claim is

A claim is declared by a contract and held by a party. It carries:

- **terms** — original balance, rate, accrual basis, seniority, a final
  scheduled date, whatever the instrument states;
- **state** — the outstanding balance, and any unpaid amount carried forward;
- **behavior** — what happens when cash reaches it.

The last is the reason this is a construct rather than a convention: paying a
claim has to *mean* something. An interest allocation clears accrued interest
and carries any shortfall; a principal allocation reduces the balance. Today a
`pay` step means "move this much cash to this entity" and nothing else, which
is why the balance has to be maintained by hand somewhere else.

### 6.2 Sketch

Illustrative only. The syntax is the least interesting part of this document
and the least settled.

```cfdl
contract credit.note_stack.notes on entity asset.trust {
  term 2017-02..2023-01

  claim a1 to party.a1_holders {
    original_balance = 182000000
    rate             = 0.0095
    accrual          = "30/360"
    seniority        = 1
    final_scheduled  = 2018-02
  }
  claim a2 to party.a2_holders { ... seniority = 2 }
}

waterfall notes.distribution on entity asset.trust {
  schedule every month from 2017-02 to 2022-11
  from series_sum("credit.pool.sched_principal.*", time.t, time.t)
       + series_sum("credit.pool.prepay.*", time.t, time.t)
       + series_sum("credit.pool.interest.*", time.t, time.t)

  pay servicing  to party.servicer = -(series_sum("credit.pool.servicing.*", time.t, time.t))
  pay a1_interest  interest of claim.a1
  pay a1_principal principal of claim.a1 = min(remaining, claim.a1.balance)
  ...
  pay residual to party.certificate = remaining
}
```

`interest of claim.a1` needs no amount: the claim states its rate and basis, so
the amount owed is derivable, and stating it again is how conventions drift.
`principal of claim.a1` does take an expression, because *how much* principal a
class receives is the deal's structure and cannot be inferred from the claim.
What the claim contributes is the cap and the effect: a principal allocation
cannot exceed `claim.a1.balance`, and it reduces it.

### 6.3 Evaluation, and why no cycle appears

This is the part to check before liking the design.

A claim's balance is two values per period: an opening and a closing.

```
opening(t) = closing(t-1)
closing(t) = opening(t) - (principal allocated at t)
```

Within a period, the waterfall reads openings and writes closings. Nothing
reads its own output. Across periods, every read is of a completed column —
the same backward-only discipline `docs/14` §3.1 imposes on a recurrence, and
for the same reason. The acyclicity guarantee in
`docs/03_expression_environment.md` survives, and survives structurally rather
than by analysis: a claim's closing balance is simply not in the environment
that computes it.

This is also why the fix belongs here rather than in the recurrence
environment. Putting completed series into a field's `next` — which is what
`docs/13` §7.37 proposed, and what `docs/14` §3.1 already promises — would make
the AmeriCredit workaround *safe* without making it *unnecessary*. The model
would still state its distribution twice; the second copy would merely be able
to see the first. That is a smaller improvement than it looks, and it leaves
the balance living somewhere other than on the thing that has it.

### 6.4 The split: language and pack

| language | pack |
|---|---|
| a claim has a balance | what an accrual basis is called and how it computes |
| an allocation to a claim reduces it | whether unpaid interest compounds, and at what rate |
| a claim cannot be overpaid | a PAC's target schedule, a TAC's targeted balance |
| ordering, seniority, `remaining`, `paid.`, `owed.` | which class types exist and what they mean |
| unpaid amounts carry forward | whether a shortfall is a default or a deferral |
| a claim's balance is a series in results | validations — colliding seniorities, a stack larger than its pool |

The test for the left column is whether the shape recurs outside credit, and
every row does. A JV promote pays down a preferred return, which is a claim. A
construction facility draws and repays, which is a claim whose balance can
rise. A P3 handback reserve fills and empties against a target. A deferred
servicing fee accrues when funds run short and is paid later, which is
`docs/17`'s catch-up under a different name. If claims were a credit-pack
construct, each of those domains would build its own, which is the situation
today one level up.

---

## 7. Required changes

Ordered by dependency, not by effort.

### 7.1 Grammar and parser

- A `claim` declaration inside a contract block: a name, a holder, and terms.
- Two new step forms — `interest of claim.<name>` and
  `principal of claim.<name> = <expr>` — or one general form with a role. A
  step that names a claim replaces `pay <name> to <entity>` rather than
  extending it, so the existing form stays exactly as it is.
- `claim.<name>.balance` as an expression path, readable in streams and in
  waterfall steps.

### 7.2 IR

`docs/05_ir_schema.md` gains a `Claim` definition and `WaterfallStep` gains an
optional target and role. The current step is `{name, payee, amount}` with
`payee` documented as "The entity this step pays"; a claim-targeting step needs
`{name, claim, role, amount?}`. Both forms must validate, because every
existing model uses the first, and the schema gate (`make ir-schema`) checks 95
goldens against the published schema.

### 7.3 Engine

- Claim state as an opening/closing pair per period, computed in the same pass
  as the waterfall that allocates to it.
- Allocation semantics: cap a principal allocation at the opening balance;
  compute an interest allocation from the claim's own terms; carry the
  shortfall.
- Claim balances published as result series, so that a case can assert them
  the way `benchmarks/credit/americredit_2017_1` asserts
  `asset.trust.bal_a1` today. This is what lets a published decrement table be
  asserted directly rather than through a hand-built field.
- Ordering: claims are evaluated where their waterfall is evaluated, so a model
  with no claims is unaffected and the existing phase structure does not move.

### 7.4 Packs

- `docs/07_pack_interface.md` §6 gains a capability: a pack may declare claim
  *kinds* with their conventions, the way §6.3 declares contract term schemas.
- The credit pack gains a liability contract — a note stack, whose claims carry
  an accrual basis, a seniority and a final scheduled date.
- `Credit.Asset.Tranche` is either re-familied or joined by a liability-family
  type. A note is an asset to its holder and a liability of the trust; the
  ontology should be able to say which view a model is taking.
- **The CRE pack needs nothing, and should get something anyway.**
  `cre.permanent_debt` is correct as it stands and no claim would improve it:
  a mortgage on a schedule is a closed form and should stay one. What a claim
  would add is the case the closed form cannot reach — a loan repaid out of a
  waterfall rather than on a schedule, which is a cash sweep, an excess-cash
  paydown, and the "revolver and cash sweep" the LBO work has been waiting on
  (`docs/13` §3.x, and the validation review's "part ready, rest needs
  waterfalls"). The convention to adopt is not "carry a balance everywhere" but
  "carry one when an allocation drives it."

### 7.5 Cases

Three cases and one fixture stop hand-rolling a liability stack:

- `credit/fnma_remic_2019_2_g3` and its six speed variants — the AB and IO
  balances become claims, and IO becomes what it actually is, a claim with a
  notional balance and no principal allocation.
- `credit/auto_abs_tranches` — the closed-form clamps become six claims paid in
  seniority order. This case is the cheapest test of the design, because its
  structure is pure sequential pay and its expected results already exist.
- `credit/americredit_2017_1` — the seven recurrences and the duplicated
  distribution both go. This is the case the design is for.
- `fixtures/valid/waterfall_abs_22_step` — the twenty-two clauses stop paying
  entities and start paying claims.
- `fixtures/valid/waterfall_cre_jv_promote` — the two preferences become claims
  that a distribution reduces, which is the smallest possible demonstration
  that this is not a credit construct.

Every one of them has a published external reference, so the migration is
checkable rather than a rewrite on faith. That is the argument for doing the
cases in that order: Ally proves the mechanism against a matched result before
AmeriCredit asks anything hard of it.

### 7.6 Documentation

- `docs/13` §2.4 is the entry this work closes, and §7.37 folds into it as
  evidence rather than standing as its own item. §7.37's proposed fix — series
  in the recurrence environment — should be recorded as the workaround it is.
- `docs/14` §3.1 still promises a recurrence environment the engine does not
  build. That is independently wrong and stays on the list whatever happens
  here.
- `docs/01_language_spec.md` §1 lists what is first-class: streams, events.
  Claims join it.
- `docs/17_ordered_waterfall.md` is where the allocation semantics live and is
  the natural home for what paying a claim means.
- The learn chapters need the word freed up — see §5 — and chapter 11 gains the
  construct it currently describes without having.

### 7.7 Gates

- `make ir-schema` and `make results-schema` cover the new shapes.
- The pack validation gate learns the new capability, since
  `tools/check-pack-validations.py` enumerates what a pack may declare.
- The glossary gate follows `docs/terminology.toml`, so registering `claim`
  regenerates the published glossary.

---

## 8. What this does not solve

Recorded so the scope is honest.

- **Termination.** A clean-up call redeems every claim and ends the deal, and a
  contract still runs for its declared term (`docs/13` §7.39). Claims make the
  redemption expressible and do not make the ending expressible.
- **Triggers that reorder a waterfall.** `docs/17` §5 and `docs/20` §2.4 want a
  priority that changes on an event. Claims are orthogonal to it.
- **The distribution's other state.** A reserve account balance and an
  overcollateralization target are not claims — nobody holds them — and
  `benchmarks/credit/americredit_2017_1` needs both. They are closed forms of
  the pool in that deal and will not always be.
- **Losses.** Every case in the suite assumes none, so no claim has ever had to
  absorb a writedown. The design says nothing about what happens when a claim's
  collateral fails, which is the next thing a real deal will ask.

---

## 9. Open questions

1. `claim` or another name, and if `claim`, whether the learn prose is reworded
   in the same change or after it.
2. Whether a claim's balance can *rise*. A construction facility draws; a
   revolver redraws; an accreting class capitalizes its interest. If the answer
   is yes, "allocation reduces the balance" becomes "allocation moves the
   balance," and the accrual side needs the same treatment as the payment side.
3. Whether interest is a claim of its own or a role on the principal claim.
   Accrued-and-unpaid interest is itself a balance that ranks somewhere, which
   argues for the first, and the second is simpler for the ninety per cent of
   deals that never defer.
4. Whether a pack may define claim *kinds* with behavior, or only conventions.
   The line matters: a PAC class's schedule is a convention, but its support
   class's obligation to absorb the difference is behavior.
5. Whether the closed-form-first convention should be revisited anywhere it is
   currently applied. The position taken here is no: it is the right default,
   it produces contracts that survive a change of calendar, and every place it
   looks awkward is a place where the driver is an allocation rather than the
   period. If that holds, no existing contract changes and claims are purely
   additive — which is a claim worth trying to falsify before building
   anything.

---

Provenance: written after `benchmarks/credit/americredit_2017_1`, August 2026,
which needed a liability stack, could not have one, and paid for it with a
defect the case's own assertions could not see.
