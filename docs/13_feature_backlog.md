# CFDL — Feature Backlog

Status: informative. Things worth building that are **not** defects.

Bugs do not belong here; they get fixed or they get a failing test. What
belongs here is capability the language or a pack does not yet have, where the
gap was found by trying to model something real and hitting a wall. Each entry
therefore says what could not be expressed, and what forced the discovery —
a backlog item with no provenance is a guess.

Ordered within each section by how much it unblocks, not by effort.

---

**Closed items are removed, not archived.** A capability that ships is
described in the language documentation; reasoning that turned out to be wrong,
and shapes the language already supports, are recorded in
`docs/26_lessons_learned.md`. This file holds work to do.

## 1. CRE pack

## 2. Credit pack

## 3. OpCo pack

## 4. Energy pack

## 5. Language and engine

## 6. Cross-pack

## Where these came from

Almost every item in sections 1 to 6 was found by reconciling a benchmark
against an external reference, and the sections are named for the pack the
reconciliation exercised. Individual item numbers are not cited here: closed
items are removed from this file, so a citation would dangle. The sources are
what matter, and they are recorded in each benchmark's `NOTES.md`.

The CRE items came from `benchmarks/cre/hud_home_multifamily` against HUD's own
populated underwriting Sample — the only source in the programme that may be
redistributed, and so the only one whose reference workbook is committed beside
the model — and from `benchmarks/cre/mit_rentleg_plaza` against MIT
OpenCourseWare 11.431J Problem Set 1, the first CFDL benchmark checked against a
published third-party figure rather than an in-house reference.

The credit items came the same way, from `benchmarks/credit/mbs_pool_conventions`
against the published industry reference for MBS cash flows — which also found
three outright defects, in the prepayment base, the recovery basis and the
payment-striking divisor, all fixed rather than listed here.

The opco items came from `benchmarks/opco/banker_dcf_conventions` against a
disclosed valuation in a public merger filing — the opco pack's first external
check. All nine cells of the banker's answer grid reproduce within $1.2mm on
$19bn. It also found two outright engine defects, fixed rather than listed
here: mid-period discounting had no spelling at all, and `on day <n>` divided
by a literal 30 on every calendar.

Section 4 came from `benchmarks/energy/utility_pv_singleowner` against a
national laboratory's open-source project-finance model — the energy pack's
first external check of any kind. Five rules reproduced it to within 1e-6
dollars on the first attempt; the two items above are what the reconciliation
found *around* the agreement.

**Section 7 is below this one, not above it.** New items are appended to the end
of the file rather than filed into the section they belong to, because the
numbers are positional and inserting one renumbers everything below it. Each
appended item names its home section.

That is the argument for building more of them: an external number finds gaps
that two of your own implementations agreeing never will. See
`research/CFDL_pack_roadmap_and_model_sourcing.md` for the catalogue.

---

## 7. Appended after the section numbering settled

New items go here rather than into the section they belong to. Backlog numbers
are positional, so inserting one renumbers everything below it and silently
breaks every `NOTES.md` reference and commit-message citation pointing past the
insertion — which has already happened once. Appending never renumbers. Each
item says which section it belongs with.

### 7.3 Pack contract coverage across the benchmark suite

*Belongs with no single pack — it is about the validation programme.*

**Re-measured 2026-09-06** across all 42 registered cases (2 bespoke, 10 cre,
13 credit, 8 energy, 9 opco — the five Fannie Mae 2019-2 speed cases are
now scenarios of one, §7.23), counting a pack contract type as *exercised*
when at least one case declares it with `contract <pack>.<type>`. When first
measured (six cases, headline "the external cases route around the packs they
should be validating") the counts were energy 9/10, credit 1/4, cre 1/12,
opco 0/10 — for cre and opco the benchmarks bypassed the pack entirely, so
they validated the engine, not the domain logic. That circularity is broken:

| pack | exercised | not exercised |
|---|---|---|
| energy | **10 / 10** (see caveat) | — |
| credit | 3 / 4 | `participation` |
| cre | 11 / 14 | `lease`, `percentage_rent_expected`, `construction_stub` |
| opco | **12 / 12** | — |

The rosters have moved since the previous measure (2026-08-30, 44 cases):
credit's three pool types collapsed into one `loan` and gained `note` and
`participation` (`docs/40`, `docs/42`); cre added `lease_unit`'s companions.
`participation` is the pass-through security; the Ginnie or Fannie
pass-through case `docs/41` owes is what exercises it.

**Elections are a second roster.** Four option types refine
`Contract.Option` and lower nothing, so the contract scan does not see them.
Measured by `option … type <T>`: `CRE.Contract.RenewalOption`
(`office_renewal_option`), `Credit.Contract.CleanUpCall`
(`americredit_2017_1`) and `OpCo.Contract.EquityOption`
(`lbo_option_pool_exit`) are exercised; `CRE.Contract.PurchaseOption` is not
— the purchase option on a finance lease in `docs/41` is its case.

What closed the earlier gaps: `office_two_tenant` exercises the acquisition
spine through the pack (`lease_unit`, `rollover`, `vacancy_loss`,
`opex_line`, `permanent_debt`, `exit_forward`), `retail_strip` adds
`percentage_rent` and `exit`, `one_lincoln_street_contract` proves
`construction_loan` against the native twin, `float_bridge_pool` and
`io_bullet_loan` (now `credit.loan` with the master's `amortization` term)
close the loan, the auto ABS pilot's classes are `note`s, and `lbo_buyout`
plus `damodaran_fcff` (now on `opco.reinvestment`, the derived line) and
`dcf_exit_multiple_nwc` take opco to twelve — the
driver-disclosing sources the first measure asked for.

**Exercised is not the same as validated.** One caveat stands:
`storage_arbitrage` is declared by `solar_ppa_microgrid`, but that case
reconciles the reduced-form arbitrage margin against convention, not against
a dispatch model. The dispatch reference now exists —
`benchmarks/energy/merchant_storage_arbitrage`, a provably optimal linear
program, core-spelled with the state of charge as walked state — and its
notes say plainly that it does not validate the pack rule, whose shape takes
the cycled energy as an input the reference exists to compute. Energy's
*validated* count stays **9 / 10** until `energy.storage_dispatch`
(`docs/27` §9 stage 4; `docs/41` §5) replaces that rule and is checked
against the shipped optimum. Read strictly,
cases whose references are independently recreated conventions
(`office_two_tenant`, `retail_strip`, `solar_ppa_microgrid`) sit a step
below a published third-party model; each CASE.md states which kind it is.

**Two axes, not one.** A concept can be expressible in the core language, in
a pack contract, or both — and a case on native streams is a choice, not a
coverage failure. `one_lincoln_street` exists in both spellings, and the pair
is the assertion. `tax_equity_flip` uses no streams and no contracts at all —
declared state and an event, with the model's own comments arguing why core
is the right spelling. The penzance developments, `banker_dcf_conventions`
and `saas_sbc_convention_fork` model natively for the same reason. A
core-spelled case proves the LANGUAGE expresses the deal with no domain
vocabulary — the stronger claim; the pack contract is the ergonomics layer,
and this entry measures whether that layer is exercised, not whether it is
mandatory.

What remains, and it is narrow:

- **cre:** three types unexercised. `lease` (non-unit grain),
  `percentage_rent_expected` and `construction_stub` may want a new case
  each; `PurchaseOption` waits on the finance-lease demonstration.
- **credit:** `participation`, closed by the pass-through case.
- **energy:** `energy.storage_dispatch`, the pack rule the shipped optimum
  can validate (`docs/27` §9 stage 4).

Recorded because coverage claims must cite this table, and the table must be
re-measured — by scanning `contract <pack>.<type>` and `option … type`
declarations, not `<pack>.` prefixes, which also match namespaced stream
names — whenever cases or rosters change.

### 7.26 Time-weighted metrics measure from the model start, on period fractions; a published WAL measures from settlement to stated payment days

Belongs with section 5 (language and engine).

`docs/12_payment_timing.md` names both limits itself: the axis origin is the
model start, not a settlement date, and precision is period fractions, not
calendar days. A prospectus weighted average life is defined on neither. The
FNMA REMIC prospectus (1 November 2018) computes it as principal reductions
weighted by "the number of years from the settlement date ... to the second
such distribution date", and the 2019-2 supplement's Pricing Assumptions fix
both anchors for its tables: settlement 30 January 2019, "each Distribution
Date occurs on the 25th day of a month".

The distinction is falsifiable, and was falsified. Recomputing the deal's
seven published WALs under four axes, only settlement-to-the-25th (actual/365)
reproduces all seven to the printed tenth. The discriminating column is 400%
PSA: printed 2.9; settlement-to-25th 2.9474; the engine's month-end axis
2.9608, and both month-end-from-settlement variants 2.956-2.962 — every
month-end reading rounds to 3.0. Amounts are unaffected throughout: P&I per
period is scheduled activity and carries no day. Only the time-weighting
moves.

Today the seven REMIC cases carry the gap as tolerance: ±0.07, decomposed in
each `case.toml` as 0.05 print floor plus ~0.015 axis. That is stated rather
than hidden, but it is the wrong long-term shape, because the axis differs by
program — Ginnie I pays the 15th, Ginnie II the 20th, FNMA REMICs the 25th —
while a single widened band is deaf to which one a deal used. A band derived
from one deal's axis can hide a same-sized convention error in the next
deal's, and nothing in the suite would notice: 2019-2 happened to publish a
column (400%) that discriminates, and the next deal may not.

*Narrowed 6 September 2026.* The payment-day half shipped: a waterfall's
schedule placement is every step's, so `schedule every month on day 25`
puts each class's principal on the 25th and the seven FNMA cases assert
their published lives at the print floor (±0.05). No run-configuration
date was added, and none should be: a settlement date is not a fact the
run asks, it is the day the holder's claim came into being — the term
start of the contract the step pays — and the day count is that contract's
own term, since two instruments in one model may differ.

What remains is the origin. `wal` over a step bound to a contract
(`pay … for contract credit.note.ab line principal`) should measure from
that contract's term start in actual days over 365, the market definition
of a published life, rather than from the model start on period fractions;
the FNMA classes are entity fields today, so the REMIC tranches as notes
(`docs/41` §5, `docs/42` S4) is what lets that land. The residual on FNMA
2019-2 is the two days between its 30 January settlement and the model's
1 February start.

Found asserting the seven published WALs of FNMA 2019-2, where the 400% PSA
column refused the naive floor and the refusal was the convention speaking.

### 7.76 The account adoption pass: every pack has a reserve it could not model

*Roadmap: M2 (`docs/37`).*

**What forced the discovery:** the account shipped (`docs/28` §5.1) and the
domain survey (`docs/30`) found the same absence recorded independently in
every domain's references. `crest_solar_cost_based/NOTES.md`: the reference
EBITDA "includes interest earned on funded reserve accounts (~$4,606 in year
one), which CFDL does not model." `utility_pv_singleowner/NOTES.md` lists
reserves among what the reference zeroed out to be comparable. `docs/41` §5 carries
`cre.replacement_reserve` from two sources. The roadmap's hospitality entry
is one accumulating FF&E reserve. Servicer advancing (`docs/38` Item 3) is a
recoverable-advances balance.

**The ask, in three parts** — the first and third are done, and the second
is open. First, the migrations the shipped fleet already
owes: the flip case's hand-carried pot (`docs/25` — the one case where
revenue is computed a second time inside the distribution) and Highlands'
cumulative window, both named gate shapes in `docs/29` phase 4. **Highlands is
done** (2026-08-29): `series_sum("cre.*", 0, time.t)` became
`account deal_cash`, and the identity held to the byte — `model.total`,
`model.irr`, `model.moic` and both payee totals unmoved on the first run. The
conversion also found what a cumulative pot costs: the pot was net of
contributions, so it could not return capital, and a party account carrying the
contribution had no offsetting leg —
`moic(party.baupost)` published 0.96 on a deal returning 2.05x. Grossing the pot
up and adding two return-of-capital tiers moved both MoICs by exactly +1.0 and
no split at all. **The flip case is done as a twin** (2026-08-30,
`benchmarks/energy/tax_equity_flip_account`): the original states what it was
waiting for, so the rebuild is carried alongside it rather than replacing it,
and both reconcile against the same external anchors at the same one-cent
tolerance — against the original's own output the twin is within tolerance on
50 of 50 cells, largest difference 0.0047 dollars on figures of about four
million. The residual is reassociation, not modeling: the original reconciles
to 1.0e-6 against the reference and the twin to 4.7e-3, the same quantities
summed through the ledger rather than inside one field expression. Whether the
original retires is left open deliberately, since it is the suite's tightest
external reconciliation. Second, a
reserve contract shape per pack where a document demands one — the DSRA
funded to target with `dscr_periodic` gating the release, the replacement
reserve of `docs/41` §5, the FF&E reserve — each as the `pay <step> to account`
pattern rather than a bespoke contract. **The credit pack's is done**
(2026-08-31, `benchmarks/credit/americredit_2017_1`): clause 19's reserve, 2.0%
of the initial pool funded at closing, was a literal written out twenty-eight
times and a step `pay reserve_topup to party.certificate = 0.0` — the right
amount to the wrong payee. It is now `account reserve` funded by its own `from`
inflow at closing, with clause 19 as the top-up
`max(0.0, inputs.reserve_required - prev.reserve)`, and the
overcollateralization target reading the balance the prospectus states it
against. All 177 series unmoved at every period, zero difference.

**The energy pack's is not, and this entry was wrong about why.** It reads
above as though CREST is the near-done one — "the case that reconciles against
CREST's own ~$4,606 still wants the reference". What CREST wants is not a
reference for the interest; it is the reserve SCHEDULE the interest is earned
on, and that is the one thing not in the repo: the port is unlicensed, was run
once outside it, and only its output numbers were carried across. The ~$4,606
is a single rounded year-one aggregate against three unknowns — balance,
funding rule, rate — and the conventional structures do not fit it (6mo debt
service + 6mo opex implies 2.1368%, 6mo debt service alone 2.9417%, 12mo
1.4709%). Fitting one is numerology, and CREST is the suite's tightest external
reconciliation. There is a second, independent blocker: CREST funds reserves at
close, and the case deliberately has no close period — `funded_at_close = 0`,
and `model.cfdl` records that a period 0 would shift every escalation exponent
by one. Energy's reserve wants the port re-run, which is a sourcing step; the
choice to take it is open.

**What doing it in the credit pack found**, both of them prerequisites rather
than by-products. `account` was missing from the parser's `is_statement_start`,
so an account declared after an `assume` was silently swallowed — the same bug
that list's own comment already records once, a `metric` declared after a
contract vanishing the same way; `lifecycle` was missing too, making three
instances of one omission. And `window_bound_is_backward` did not recognise `if`, so the
AmeriCredit pot's `if(time.t == 1.0, 0.0, time.t)` lower bound — the first
distribution draws two collection periods — was read as forward, keeping the
model on the column order, where account balances are not computed at all. `if`
now joins `max` and `min`. That change also produced the first evidence that
the corpus's most intricate waterfall agrees period-for-period under the walk
and under the column order. Third, interest ON a reserve balance — **done 2026-08-30**,
`fixtures/valid/reserve_interest_on_balance`. The entry was wrong about the
spelling, and the first attempt was withdrawn for a reason that turned out to
be wrong too; both are recorded because the second one is the interesting one.

**The spelling.** The entry called this "a stream whose amount reads
`prev.<account>`, legal under §4's backward rule". That is refused —
`E1123_PREV_OUTSIDE_NEXT`, because `prev` outside a `next` means nothing — and
`docs/03` is precise that a balance is readable in rules, guards and step
expressions. A field's `next` is a rule, so the field carries the balance
forward and the stream reads the field.

**The withdrawal, and what it actually found.** The fixture failed
`walk_matches_the_column_order`: column 0 against walk 5. That reads as the
mechanism being unsound, and it is not. The test already excludes models whose
logic reads settled cash, on the stated ground that "the column order settles
all state before any stream has a value, so the read binds nothing there and
the model means something different — which is exactly the expressiveness the
walk adds". An account balance is that same category, and §5.1 above says so in
terms: `prev.<account>` is settled state read "the same way a delinquency edge
tests realised rent".

The predicate simply did not know about accounts. It detects `series_` and
predates `docs/28` §5.1, and no blessed model read a balance in logic until
this one, so the gap had never been exposed. Extending it to account reads is
completing an existing principle, not waiving a failure — and the property
still holds where it applies: 124 models compare with walk == column, four are
walk-only.

**The pin:** a reserve funds toward 3,000 out of 1,000/month, and interest
accrues at 0.5% on the PRIOR balance — 5.00 on the first 1,000, 10.03 on 2,005,
then 15.00 a month once the target holds. Reading strictly backward is what
keeps the reserve and the interest it earns from being mutually circular. The
CREST reconciliation line is closed as a mechanism; the case that reconciles
against CREST's own ~$4,606 still wants the reference.

Related: `docs/41` §5, §7.41, §7.72 (shipped), `docs/38`, `docs/25`, `docs/28` §5.1, `docs/30` §1.

### 7.77 A covenant that is published but powerless: the DSCR cash trap

*Roadmap: M2 (`docs/37`). **The mechanism shipped 2026-08-30**; what remains is
the benchmark against an external reference, which is `docs/20` §5.1's ask.*

**What could not be expressed:** consequences. The energy pack publishes
`dscr_periodic` per period (`packs/energy/statements.toml`, with its own
argument that "a project finance covenant is tested EVERY PERIOD"), and
`ppiaf_toll_highway` sizes a subsidy to hold 1.30x — but no model could say
what a real credit agreement says: below the trigger, distributions stop and
cash traps in an account; at or above it **for the cure period**, the trap
releases.

**What shipped.** `fixtures/valid/dscr_cash_trap_cure_period` runs the whole
covenant: NOI of 12,000 against 15,000 of debt service puts DSCR at 0.80
against a 1.20 trigger, the machine reads settled cash strictly backward and
traps at t=5, cash accumulates once NOI recovers (5,000 at t=7, 10,000 at
t=8), and two consecutive good periods at t=9 release the trap in full.

**The cure period was the part that waited on the arrival actions of `docs/34`**, and it is worth being
precise about why. `trapped_cash_cure` has existed since the walk, and it
cures on the *next* good period — which no credit agreement says. A cure
period is a duration measured from the last breach, and a field recurrence
counts consecutive good periods without any way to start over at each new
one. `on enter trapped { set good_periods = 0 }` is the whole difference,
and it is the same shape as the EBA probation the credit pack's machine
carries (`docs/36` §2.1).

**What remains: the external reference.** A fixture asserted against its own
engine is the suite marking its own homework (`docs/20` §5.1). The mechanism
is pinned; the covenant case wants a published credit agreement with a
cash-trap schedule and figures to reconcile against, and none is vendored.
That is a case-authoring ask with a sourcing problem, not a language gap.

Related: `docs/38`, `docs/28` §5.1 and §6, `docs/30` §2,
`docs/20` §5.1.

### 7.80 121 registered diagnostic codes have no minimal failing example

The machine docs work (docs/32 Phase 2) measured the register: docs/08 §7
names 197 codes, and only 71 appear in `fixtures/invalid/` + `gold/diag/`.
The repair catalog (`docs/machine/diagnostics-repairs.md`) lists the
uncovered codes by name, so this entry is a work queue, not a survey: each
item is one minimal failing fixture, its blessed golden, and a
compile-verified repair in `fixtures/repairs/`. Retired codes (§8) are
exempt. The catalog's coverage line is the progress meter.

### 7.82 CFDL-CE tiers are prose; nothing asserts the estate maps to them

docs/22 §2 assigns every published surface to a tier (A–D) with path
globs written in a markdown table. No tool parses that table, so a new
published file lands in no tier and no rule applies to it — the estate's
coverage is whatever `check-site-voice.py` happens to glob. Promoting the
tier table to a machine-readable form (or parsing it as written) and
asserting every published path matches exactly one tier would close the
loop the authoring contract needs. (`ste-allow:` rule ids are now
validated against §3's rule tables; the tier mapping is the remaining
unenforced half.)

### 7.84 `model.moic` does not compute what its own comment says

*Belongs with the language and engine (section 5).*

The comment above it describes a ratio of cash in to cash out over the life.
The code sums the model's net-POSITIVE periods over its net-NEGATIVE ones.
Those are the same quantity only while no period holds both.

`benchmarks/cre/basic_acquisition_exit_cap` is the minimal case. Its purchase
settles at the open of period 0 and its first year of operations at the close
of the same period, so they net inside it:

```
-1,417,958.33 + 83,077.50 = -1,334,880.83
2,542,954.53 / 1,334,880.83 = 1.905005    model.moic
2,626,032.03 / 1,417,958.33 = 1.851981    published, and what the case asserts
```

Two consequences. The figure moves with the CALENDAR: on a monthly grain the
purchase would sit alone in month 0 and the same deal would read differently,
which is not a property a return should have. And it is not a multiple on
invested capital in any published sense — MOIC partitions by KIND, capital
contributed in the denominator and value returned in the numerator, which is
how A.CRE states it for real estate and how GIPS 2020 defines the fund-level
TVPI it resembles. GIPS is emphatic in the other direction: a distribution that
is recalled *increases* paid-in capital, so the same dollar out and back raises
both sides of the ratio where this fold reduces the denominator.

**This is not a gap in the language, and the remedy is already in it.** A
multiple belongs on the valuation plane, where the model says what it counts as
invested capital rather than the engine guessing from a sign. `metric` folds
once at the horizon and reads series and `model.*`, so the case declares the
multiple in three lines and asserts the published figure exactly. `moic(party.X)`
already does the same job per party in `penzance_highlands`. Nothing needed
adding.

So the question this raises is what `model.moic` should BE, not how to fix it:
whether a whole-model multiple has a defensible meaning at all — for a levered
deal it puts debt proceeds in the numerator and repayment in the denominator —
or whether it should be narrowed, renamed to what it actually computes, or
withdrawn in favour of the declared form.

**`moic(party.X)` folds the same way and is not a counter-example.**
`penzance_highlands` reproduces exactly (baupost 1.959618, penzance 2.906607)
only because its contributions land in periods 0-91 and its single distribution
at period 153, so nothing ever shares a period — verified from its journal, the
overlap is empty. A fund distributing while it is still calling capital would
trip it.

**One existing assertion depends on the current behaviour.**
`penzance_highlands` asserts `model.moic` = 2.04664, and it is the only entry in
that case's `expected_metrics.json` with no `source` line — the party metrics
beside it both cite contributed and distributed. 43 of its 160 periods hold both
a positive and a negative stream flow. Whatever is decided above, that figure
has to be re-derived rather than carried over.

Provenance: found building `basic_acquisition_exit_cap`, 30 August 2026,
against a published equity multiple. Two earlier drafts of this entry
overreached — the first called the party metric structurally different, the
second proposed a taxonomy node and a `cre.acquisition` contract as though the
language could not express the multiple. It can, and does.

---

### 7.88 A model-level entity namespace is not validated against the families

*Belongs with the language and engine (section 5). What remains of the
container entry: the construct shipped 30–31 August 2026 — `container` is the
third entity family, `NODE_FAMILIES` adds contract and reference for
relations to range over, and a container MAY carry deal-level cash
(`docs/01` §7.1, `docs/07` §6.1, `CHANGELOG`).*

`entity carpark x` is legal and silently untyped. The model namespace was
never family-gated — which is why `entity container fund` compiled before
the family existed — so a declaration whose family the language does not
know is accepted as though it were one. Decide whether a model-level
`entity` declaration should be validated against `ENTITY_FAMILIES` at all,
and if so what the refusal says: the namespace is either a typo for a family
the roster has or a family it lacks, and the diagnostic should let the author
tell which. Found when the container family landed; it was a finding of that
work, not a change it made.

Related: `docs/01` §7.1, `docs/07` §6.1.

---

### 7.94 A reduction reads a series, never a transformed one — and cannot say WHERE

*Belongs with the language and engine (section 5). Split from the
series-reductions entry when its four reductions shipped (`docs/03` §4) and
these two did not.*

`series_max` answers "what was the peak". Two neighbouring questions it does
not answer, and neither is a reduction:

**1. A reduction over a TRANSFORMED series.** "How many periods was DSCR below
1.20", "what was the maximum drawdown", "the sum of the absolute movements" —
each folds a series that does not exist. `series_count(name, from, to)` counts
periods whose aggregate is non-zero, which is a real question and not this one;
the covenant question needs a per-period predicate, and there is nowhere to put
it. Every reduction takes a series NAME — a text selector — so no expression
can sit between the series and the fold.

Three shapes, in rising order of language cost:

- **A predicate argument**: `series_count_if(name, from, to, "<", 1.20)`,
  passing a comparison operator as a STRING. Cheap and ugly, and unlike
  anything else in the language.
- **A predicate expression**, which means lambdas or first-class expressions.
  Out of scope; the language has no construct that takes one.
- **A DECLARED per-period line**, which is the model-declared statement (`docs/01` §16) — a model cannot declare a
  subtotal, and a field's `next` reads no stream series (`docs/14` §3.1), so
  there is no legal place to compute an indicator. If the model-declared statement (`docs/01` §16) shipped, an
  indicator line declared once and `series_count` over it answers the covenant
  question WITH NO NEW SYNTAX AT ALL.

That last is the reason this entry exists rather than a `series_count_if`: the
missing thing is not a reduction, it is the line to reduce. **Do not build the
predicate argument before the model-declared statement (`docs/01` §16) is decided.**

**2. WHERE, not what.** `series_argmax`, and "the first period DSCR crossed
1.20". Three decisions, which is why it did not ride along with the reductions:

- **The return type.** A period index composes with the windows every
  reduction already takes; a DATE is what a covenant clause names. And the
  registry has no `period -> date` function, so an index cannot be turned into
  the date the clause wants — whichever is returned, the other needs a second
  function built beside it.
- **Ties.** First or last occurrence of the maximum. First is conventional and
  should be stated rather than emergent.
- **Nothing to point at.** An empty selection has no position, the same
  argument that makes `series_max` refuse it — so these inherit that refusal.

A first-crossing form additionally needs the predicate of part 1, so the two
halves of this entry are not independent: settle the model-declared statement (`docs/01` §16), and both get simpler.

**Why the position-returning forms are not urgent.** Results publish the full
per-period series in `deterministic.series`, so an analyst holding results has
everything argmax and first-crossing need and can take them in pandas. That is
a reason to sequence them late, not a reason they are unnecessary: a figure
computed outside the model is not asserted by the model, carries no lineage,
and cannot appear in a scenario column or a Monte Carlo distribution. Recorded
so a later reader knows this was decided rather than overlooked (31 August
2026).

**Also unbuilt, and related: a cumulative scan.** A peak balance is a fold over
a series that CARRIES the balance, and `fixtures/valid/series_reductions` shows that working
because the model declares the balance as a field. A model that has only flows
cannot synthesise the running total to fold — that is a scan (a series in, a
series out), not a reduction, and it is the same missing capability as part 1
seen from another side.

Provenance: split out of the series-reductions entry on 31 August 2026, when its four reductions
shipped and these did not. The `period -> date` gap and the the model-declared statement (`docs/01` §16) dependency
were both found while scoping that work, not before it.

---

### 7.95 Undefined is not zero, and a series cannot say so

*Belongs with the language and engine (section 5). The design is SETTLED below
and not built; the metric-environment work deferred it and the series
reductions sharpened it (`docs/01` §15.3, `docs/03` §4).*

A ratio subtotal publishes `null` for the periods where it is genuinely
undefined — a coverage ratio in a period with no debt service — and no
reduction can fold it. `E1365` refuses the name with a hint saying why, which
is honest and not an answer: `series_max("domain.dscr", 0, 11)` is the covenant
question, and the covenant question is the reason ratios exist.

**The cause is the representation.** A metric's visible series are
`BTreeMap<String, Vec<f64>>`, in which "undefined" has no spelling. Binding a
ratio there would have to write SOMETHING in the undefined periods, and every
candidate is a lie: 0 is a value the ratio never had, and it is the exact
failure the series reductions exist to end (`docs/03` §4).

**Two things look like "missing" and are not the same thing.** Conflating them
is the trap this entry exists to avoid, and the reductions already paid once for
conflating a neighbouring pair:

- **Past the end of the data** — the window runs into the projection tail or
  past a short series. The CELL DOES NOT EXIST. `series_avg` pads here: the
  numerator sums the cells that exist and the divisor is the REQUESTED window,
  so a window past the data averages over the full window. Shipped,
  deliberate, documented, and staying.
- **Genuinely undefined** — the period exists and the quantity does not. This
  is what a ratio's `null` says, and nothing handles it.

**The settled design:**

1. **Bind ratio series in the METRIC environment only**, as an optional-valued
   series. The causal plane reads `env.series` too — streams, guards, field
   rules — and has no ratio subtotals to read, so widening it would take the
   blast radius for no gain. Narrow first; widen if a document forces it.
2. **Every fold SKIPS the undefined periods.** They are not observations.
   `series_max`/`series_min` over the defined ones, `series_sum` adds them,
   `series_prod` multiplies them, `series_count` counts the defined non-zero.
3. **`series_avg`'s divisor counts the periods it actually folded.** This
   sounds like a change to the shipped rule and is not one: a CASH series has
   no undefined periods, so its divisor is the requested window exactly as
   today. The rule follows the SERIES, not the function, and that sentence has
   to be in the spec or it will be rediscovered as a bug.
4. **An all-undefined window** gives null for max, min and avg, and 0 for sum
   and count. A mean of nothing is not zero.

Note what this inherits: `series_max` already publishes null for an
empty selection, so the value shape and the `Scalar::Null` publication exist
and the results schema already permits them. What remains is the
representation and the skip rule.

Related: `docs/03` §4 (the reductions), `docs/01` §15.3 (which binds everything
else a metric can read), §7.94 (the transformed-series reductions, which need this decided
first — a breach indicator over a ratio is exactly a series with undefined
periods).

Provenance: deferred out of the metric-environment work on 31 August 2026,
sharpened while building the four reductions, and settled the same day rather than left as an open
question — the decision is cheap to record now and expensive to re-derive.

### 7.96 A party owns at most one account

*Belongs with the language and engine (section 5). Found converting
`benchmarks/credit/auto_abs_tranches` onto accounts, 2 September 2026.*

**What could not be expressed:** a noteholder's two positions. A class of
notes has a principal position — what has been repaid, which is what its
remaining claim is stated against — and an interest position, what it has
earned. Both are cash allocated to the same party, and `docs/01` §10.6 says a
party owns at most one account, so `pay a2_interest to party.a2_holders` and
`pay a2_principal to party.a2_holders` land in one balance and the class's
claim, `face − principal repaid`, cannot be read from it.

**What forced the discovery:** the case's principal steps read
`prev.<class>_principal` as the class's cumulative repayment. The interest
steps had to go somewhere else, and the only spelling the rule allows is a
STRUCTURE-owned account per class (`account a2_interest { from 0.0 }`, paid
by `to account`), which records the cash correctly and attributes it to
nobody: the holder's `entity.party.*.net_cash_flow` carries principal only,
and the interest a class earned is visible in an account that no party owns.
Seven such accounts in one model, each a workaround for one sentence.

**Why the rule exists, and why it is the wrong rule.** §10.6 keeps "their
account" resolvable: `pay <step> to <party>` lands in the party's account
without naming it, and with two the destination is ambiguous. That is a
reason to require the explicit form when a party owns more than one, not to
forbid the second account. A party with several positions is the ordinary
case in every structured deal — principal and interest on a note, capital and
preferred return on a partnership interest, a lender's advances and its
recoveries — and each is a claim the waterfall pays separately.

**The shape.** Lift the limit: a party MAY own several accounts. `pay <step>
to <party>` keeps its meaning while the party owns exactly one; when it owns
more, the bare form is refused at compile with the accounts named, and the
step says `to account <name>`. Party-level returns (`irr(party.x)`,
`moic(party.x)`, `entity.party.x.net_cash_flow`) fold across every account
the party owns, which is what they mean today with one. Nothing else moves:
the balance law, `prev.<account>`, and the journal are per account already.

Related: §7.76 (the account adoption pass, whose reserve was the first
account), `docs/28` §5.1 (where the one-account rule is stated as a
resolution convenience), `docs/17` §13.

### 7.99 A `reference` names an external series and cannot reach one

*Belongs with the language and engine (section 5), and with the ontology.*

The `reference` family is the fourth entity family and every pack declares
members of it — `energy.power_price` (kind `price_curve`, USD/MWh),
`credit.base_rate`, `energy.inflation`. A model may cite one from a `quantile`
through its `ref` clause, and `required_refs` records the citation. That is the
whole of what a reference does today: it is an identifier, and citing it buys
provenance.

**The sister repo specifies more, and the missing half is the half that
matters.** `evs-platform/docs/03_registries_specification.md` splits the concept
in two:

- an **Observable Registry**, which "defines the external data series a domain
  model may reference" — `Rates.SOFR.1M`, `Index.CPI`, `FX.USD.EUR`, each a time
  series;
- a **Binding Registry**, which "maps observable IDs to their data sources or
  snapshot columns", declaring for each one a "target: snapshot column, run
  config key, or connector endpoint" and a "fallback behavior when the
  observable is unavailable".

CFDL shipped the first and none of the second. A reference declares that a
series EXISTS and can be cited; nothing binds it to a source, so nothing
resolves it to values.

**Three consequences, each visible in a shipped artefact.**

`obs` is a scalar. `fixtures/valid/obs_smoke` reads `obs.rate` and the run
configuration supplies `"obs.rate": 1000.0` — one number. EVS resolved its v0.2
syntax as `obs.rate("SOFR")`, a FUNCTION returning the series, and records the
runtime question as open and High: "Observable binding hydration at runtime (who
provides values: run config? snapshot? connector?)" (`09_inconsistencies_and_gaps.md`,
item 26). A scalar cannot carry a rate curve, let alone a price year.

`curve` cannot cite a reference at all. Compare the two grammar productions:

```
curve_stmt    = "curve" IDENT [ curve_interp ] "{" curve_point … "}" ;
quantile_stmt = "quantile" IDENT [ quantile_interp ] [ quantile_order ]
                [ "ref" qname ] "{" quantile_point … "}" ;
```

The two constructs are deliberate counterparts — level by date, dispersion by
probability (§7.1) — and only one can name where its numbers came from. A
`curve` is exactly the construct an external series would arrive in, and it is
the one with no provenance.

So a model carries market data as literals. `benchmarks/energy/merchant_storage_arbitrage`
declares 730 curve points across two curves, generated by its own
`reference_gen.py` and imported from `prices.cfdl` to keep the model legible.
The import is the right shape for what exists; it is not a binding, and nothing
in the results says which observable those numbers are.

**What this is not.** Not a request for a connector, and not for CFDL to fetch
anything. The narrow version is that a declared reference should be resolvable
to a series through a stated binding — at minimum a file or a run-configuration
key — and that `curve` should be able to name the reference it carries, the way
`quantile` already does.

Provenance: found writing `merchant_storage_arbitrage`, whose market input is
730 literal points that no results document can attribute. The desired shape was
then read from `evs-platform/docs/03_registries_specification.md` rather than
inferred. Related: `docs/27` §4.4 (what `ref` buys), and EVS question 26.

### 7.102 A field cannot fold a stream "since my last step"

Belongs with §5, language and engine. Found 4 September 2026 building the
level-pay pool's balance.

The natural row for a balance is `closing = opening − what the streams paid`:
a field whose `next` reads the prior period's principal streams, which a
field may do (`fixtures/valid/recurrence_reads_settled_cash`). On a monthly
book it works. On a daily book with monthly payments it does not: the field
steps on payment dates, the streams strike on the same dates, and
`series_sum(…, time.t − 1, time.t − 1)` reads the prior DAY, which is zero.
A window in model periods cannot be written in the rule, because the rule
knows its payment frequency and not the model's calendar, and a month is
28 to 31 days. So the balance is rolled forward from the RATES instead —
opening × (1 − principal fraction) × (1 − mdr − smm), the same row written
in terms of the schedule rather than the cells — which reproduces the
closed form exactly and restates the hazard fragments the fragment gate
already polices.

*Narrowed 5 September 2026: a BALANCE no longer needs this. The account
rolls every model period and a stream reads the prior close as
`prev.<account>` on any grid (`docs/42` §7). What remains is the field.*

**The ask:** a fold bounded by the reader's own cadence — the stream's
values since the field last stepped — so a stream-driven field is
expressible at every cadence. It is what a loan-level pool will want, where
the reductions are actual payments and there is no rate to roll forward
from. Related: §7.98, `docs/28` §4.

### 7.104 The pool's amortization schedule and its accrued interest can disagree

Belongs with §2, credit pack. Found 4 September 2026 reading the level-pay
rules against `pack_amortization_day_count`.

The balance amortizes on the annuity factor at the AMORTIZATION divisor
(`{{model.amortization_divisor}}`, 30/360 by default) while the scheduled
principal stream is the level payment less interest at the ACCRUAL divisor.
When the two differ (an `act/360` accrual on a 30/360 amortization) the
principal the pool pays is not the principal the balance loses, and over a
term the two drift. A real level-pay loan fixes the PAYMENT and lets
principal be the remainder after actual interest; the balance then falls by
that remainder. Not changed here — the balance reproduces the former closed
form exactly, which is what the rollout needs — but it should be decided:
either the balance rolls forward from the stream (which §7.102 enables), or
the day-count term is documented as an interest convention that leaves the
amortization schedule untouched.

### 7.106 MOIC of a zero cash flow is a ratio of sign noise

Belongs with §5, metrics. Found 4 September 2026 when the decimal-to-float
conversion became deterministic.

`fixtures/valid/credit_participation` passes every dollar of the pool
through to the holder, so `model.net_cash_flow` is zero in every period.
`model.moic` published 0.818182 before the conversion change and 0.9 after
it: both are counts of which periods' zeros carried a negative sign at the
28th digit. A multiple on money that never moved should be null, as the
pool factor is before the pool is bought, not a number that changes when a
float flips its last bit. The same applies to `model.irr` on such a series.

A SECOND INSTANCE, and this one is shipped and published. `benchmarks/cre/
penzance_highlands` opens with 79 periods before any cash, and 23 of its
per-period net cash flows are `-0.0` — negative zeros. Every one counts as a
sign change, so the solver finds a root where the deal has none:
`model.irr` publishes 809072402.742629, which is 8.1e10 percent. It is the
same defect as the pool factor above, reached by a different route: there the
zeros were the whole series, here they are a stretch of it, and in both cases
a signed zero is being read as information. Whatever null-ing rule closes this
entry must cover a series that is PARTLY zero, not only one that is entirely
zero — and `retail_strip`, which publishes 327% honestly on a model that
books an exit without booking the acquisition, must be left alone by it: that
is a modelling choice in the case, not sign noise.

### 7.107 The construction loan compounds interest within the period

Belongs with §1, CRE pack. Found 5 September 2026 moving the construction
loan onto the balance account.

`cre.construction_loan`'s interest row accrues on `funded − draw − equity`,
and `funded` at that point already includes the period's own capitalized
interest (the funded field's `next` adds it before the streams read the
field). So interest is charged on interest accrued in the same period — a
compounding no loan agreement writes. The account reproduces it exactly
(the `capitalized_interest` accrual row copies the interest row's base) so
that no number moves; the right base is the opening balance plus the
period's draw at its accrual fraction, which is what `prev.balance` would
give. Decide with the pack, and re-bless `one_lincoln_street_contract`
when it changes. Related: §7.104.

### 7.108 An exercise cannot bring a contract into being

Belongs with §5, language and engine. Found 5 September 2026 building
`benchmarks/cre/office_renewal_option`.

A renewal option, exercised, produces a lease: five more years at the stated
rent, with the pack's own lowering — escalation, recoveries, the leasing cost
as a dated one-shot. The language cannot say that. A contract takes no
activation guard (`docs/01` §13.4), `activate`/`deactivate contract` was
removed (§7.73), and an option's actions reach entity fields and streams,
never a contract. So the case declares BOTH outcomes as leases and the
election switches one off: the option's actions deactivate the market
lease's four lowered streams, and a scheduled event with the opposite test
deactivates the renewal lease's — two declarations of one decision, and
eight `deactivate` lines for what is one sentence in the lease.

The shape to decide: an option's action that activates a declared contract
(`activate contract cre.lease_unit.tenant_a_renewal`, the contract's streams
inactive until then), or a contract whose `term` starts at an occurrence
(`term from exercise(renewal) for 60 months`). The second reads as the
agreement it is — the renewal term begins when the right is exercised — and
keeps the contract free of guards. Either way the renewal case's lapse
event and its `deactivate` lines go away. Related: §7.73, `docs/40` §10.

### 7.109 An option's payoff carries no category

Belongs with §5, language and engine. Found the same day, in the same case.

A stream states a category and the pack's subtotals fold it (`docs/35`). An
option's payoff publishes as `option.<name>` with an owner and no category,
so a renewal's leasing cost, paid as the exercise's payoff, would be in
`model.total` and absent from `domain.cre.leasing_costs`. The option should
take the stream's `category` clause, or the election type should fix one
the way a lowering rule does for its line — `Contract.Option` declares the
line `payoff`, and a pack refinement knows what its payoff IS. Until then a
case that needs the subtotal writes the exercise cash as a gated stream and
leaves the payoff at zero, which puts the exercise's cash outside the option
— what `benchmarks/cre/office_renewal_option` does, with the renewal
lease's own leasing cost as the exercise's cash.

### 7.110 A stream whose expression fails pays zero, under a warning

Belongs with §5, language and engine. Found 5 September 2026 routing the
curve-outside-its-dates refusal through the evaluation sites.

§7.103 made a FIELD whose rule fails fatal: a value that was never computed
is not a number, and the run refuses naming the field and the period. The
same failure everywhere else is still softened. A stream's amount that fails
to evaluate — a division by zero, a function on an argument out of range, a
name that resolves to nothing — is paid as 0; a guard that fails is read as
false; an account inflow that fails contributes 0; an option payoff that
fails pays 0. Each leaves a warning in `results.warnings` (`… evaluation
failed [EXPR_EVAL]: …; using 0.`), and the run reports ok with a total that
is the true total minus whatever the failed expression would have paid. A
compile failure of a stream's or field's expression is softened the same
way (`… expression compile failed …; using 0.`).

The benchmark harness fails a case on any warning, so no shipped case hides
one; a modeller's run does not have that gate. The rule §7.103 settled
applies without change: the causal plane refuses, naming the reader, the
clause and the period, and the fold does it once over the walk's markers
(the shape `E5032` and `E5040` already use). Decide whether the guard's
`false` is the one exception — a guard that cannot be evaluated is closer
to "the condition did not hold" than an amount is to "nothing was paid" —
before building. Related: §7.103 (the field half, closed), §7.95 (undefined
is not zero).

### 7.111 A security named by one waterfall and not another is a valid model

Belongs with §5, language and engine. Found 6 September 2026 building the
securitization front door in the UI prototypes.

A note class in a structured deal is five declarations: its holders, the
account that IS its position, an account for its interest, the note itself,
and a step in EACH of two waterfalls — interest at its coupon, principal by
seniority. `benchmarks/credit/auto_abs_tranches` writes seven of those, which
is thirty-five declarations and fourteen steps kept in agreement by hand.

Omit one step and nothing says so. A class present in `notes.interest` and
absent from `notes.principal` compiles, runs clean, and is never repaid
principal; the trust simply keeps the money, the ledger balances, and every
metric is a real number. The failure surfaces only as a class whose holder
account ends at zero, which is indistinguishable from a class that was
genuinely never reached.

BOTH MODELS ARE VALID, which is why this is a warning rather than an error. A
security paid interest and no principal is a legitimate thing to model — an
interest-only strip is exactly that — so the check cannot refuse. What it can
do is say that the deal declared a line the priority of payments never
mentions.

The ontology already knows enough. A `credit.note` is a `Contract.Security`
whose master declares its lines; a waterfall step binds `for contract <c> line
<l>`. So for each contract on a subject that some waterfall on that subject
pays, the checker can ask which of its declared lines no step names, and warn
once per unnamed line, naming the contract and the line. It should stay quiet
when no waterfall pays that subject at all — a security in a deal with no
priority of payments is a different and deliberate shape, not an omission.

The same evidence supports the mirror check, which is cheaper and stricter: a
step naming `line principal` on a contract whose master declares no such line
is already an error, and this is the other half of that pair.

Related: `docs/40` (Contract.Security and its lines), §7.96–7.98 (an account
per party, from the same benchmark), `benchmarks/credit/auto_abs_tranches`.

### 7.112 A statement row cannot repeat over the instances of a type

Belongs with §5, language and engine. Found 6 September 2026 building the
securitization front door in the UI prototypes.

`packs/credit/statements.toml` ships the artifact this domain publishes — a
servicer remittance report, folding categories rather than stream names, so it
stays correct as the pack grows. It applies to a deal that declares no
statement of its own, which is the right default and works.

It reports `Paid to holders: interest` and `Paid to holders: principal` as ONE
line each, across every class. An investor report is per class: what Class A
was paid, then B, then C. The pack cannot write those rows, because the class
names belong to a deal it has never seen.

A model can declare its own `statement` with a row per class naming that
class's stream — `streams = [...]` exists on a row for what a category cannot
express. That is hand-written and has to be kept in agreement with the classes,
which is §7.111's failure mode wearing different clothes: add a class, forget
the row, and the statement quietly reports less than the deal paid.

The shape to decide: a row that REPEATS over the instances of a contract type —
one row per `credit.note`, its label built from the instance, its figure the
line that instance was paid. Order is the declaration order of those contracts,
which is safe here in a way it would not be in a waterfall: a statement is a
VIEW, no cash depends on the order, and there is no seniority to get wrong.
Related: §7.111 (the waterfall half), §7.113, `docs/40` (a master's lines).

### 7.113 A pack template cannot extend a declaration it did not create

Belongs with §6, cross-pack. Found the same day, in the same work.

`packs/credit/templates.toml` already answers most of "add a note class". Its
`credit.note` body emits FOUR declarations — the trust, the holders, the
account that is the class's position, and the contract — which is more than a
template is usually credited with doing.

It cannot emit the other three quarters of the answer. A class is also a step
in the interest waterfall and a step in the principal waterfall, and a template
creates declarations; it cannot insert into one that already exists. So the
template covers four of the seven parts, and the three it cannot reach are
exactly the three whose omission nothing detects (§7.111).

The shape to decide: a template clause that appends a step to a NAMED
waterfall, with the position stated rather than implied — after a named step,
or at the end. Position must be explicit precisely because order is the
meaning: a template that silently appended would make the newest class the
most junior, which is right about half the time and wrong silently the other
half.

What this buys is that "add a class" becomes a pack capability rather than an
application's. Someone writing CFDL in an editor gets the same seven parts the
prototype's tranche table writes, from the pack that knows what a class is.
Related: §7.111, §7.112, `packs/credit/templates.toml`.

### 7.115 Conventions checks: the model is legal and almost certainly not meant

Belongs with §5, language and engine. Found the same day; §7.111 is the first
member and the reason to name the family.

Some models compile, run clean, and are wrong in a way the language cannot
call an error, because the same shape is legitimate elsewhere. A note paid
interest and never principal is an interest-only strip or a forgotten
waterfall step, and nothing in the text distinguishes them.

A diagnostic is the right home for these, and not a document, because it is
the ONE channel every consumer already reads: the CLI, the MCP `compile`, the
wasm engine, and any application built on them, each getting the span with it.
A convention recorded in prose reaches whoever read the prose.

The family, as warnings — never fatal, allowlistable, and each naming what it
saw and what it expected:

- a security whose declared line no waterfall step on its subject names (§7.111)
- an `assume` nothing reads
- a note whose `principal_account` is not the account its holder owns
- a waterfall step naming a contract that is not written on that waterfall's
  subject
- a party that owns no account in a deal whose waterfalls pay parties

To decide before building: these are new W-codes, so they land against the
W-code parity gate, and a warned run is already a suspect run to the benchmark
harness — which means each member has to be quiet on every shipped case before
it can ship. That is a feature: a member that fires on a benchmark is either a
finding or a badly drawn rule, and both are worth knowing before release.

Related: §7.111, W-code parity gate, `docs/22` (how a diagnostic should read).

### 7.117 `lookup` cannot reach the entity types, the published fields, or the categories

Belongs with §5, language and engine (the tooling half). Found the same day,
and over the preceding week building against the MCP tools.

`lookup` resolves a pack's contract types through their master chains and
returns effective roles, fields, lines, side and templates — the form-builder
payload, and it is good. `PackInfo` carries `contracts`, `masters`,
`templates`, `metrics`, `validations`. Three things a model needs are not
reachable through it:

ENTITY TYPES. A pack's entities are absent, so what a `contract ... on entity
...` may be written on is unknown. Guessing produced `Energy.Asset.Project`
and `OpCo.Asset.Business`, neither of which exists; the real names are
`Energy.Asset.GenerationFacility` and `OpCo.Asset.Enterprise`. The fix is a
field beside `contracts`:

    entities: [ { type_id, family, class, refines, lifecycle,
                  fields: [ { name, field_type, required, unit, description } ] } ]

WHAT A LOWERING PUBLISHES. A `credit.note` publishes
`credit_note_claim_<instance>` and `credit_note_interest_due_<instance>` on its
subject, and a waterfall step is written in terms of them. Those names appear
in no tool output; they are learned from a benchmark. Beside `lines` on
`ContractInfo`:

    publishes: [ { name: "credit_note_claim_<instance>", of: "money",
                   description: "face less what the holder's account received" } ]

THE CATEGORIES A RULE EMITS. An account, a statement row and a subtotal all
fold categories, and which categories a type's lowering produces is not
returned. Beside `lines`:

    categories: [ "operating.collection.interest", ... ]

Two smaller ones: a role whose name collides with a reserved word cannot be
bound and the failure reads as a parse error (`owner`), so `RoleInfo` should
say so; and the metric key shapes a run publishes (`entity.party.<x>.total`,
`model.npv`) are discoverable only by running, which is a poor way to find out
what a scenario grid can have columns of.

The honest counterweight, recorded because it shapes the priority: `RoleInfo`
ALREADY returns `unbound`, and a role that may not be bound was bound anyway,
because nothing consumed the field. Exposure is necessary and not sufficient.
This entry is worth doing and it is worth less than §7.115.

MEASURED, 2026-09-07. A transcribe run of `gpt-5.6-sol` over 18 cases put a
number on the cost: of 113 asserted groups missed, 89 were name-dependent and
24 were economic. `penzance_highlands` is the clearest instance — 7 of its 8
name-independent assertions matched, the deal was right down to debt service,
and it scored 0.280 because 17 of its 25 groups are labels no specification
conveys. The economics were not the difficulty; naming was.

AND THE ACCOUNTS. A waterfall step is written in terms of accounts as well as
published series: which accounts exist on the subject, which party owns each,
and which side it sits on. None of that is reachable through `lookup` either,
so the second half of a step is as unwritable as the first. Beside
`publishes`:

    accounts: [ { name, owner, side, opens_at, description } ]

That is the difference between naming the cash a step moves and naming where
it moves from and to, and a step needs both.

Related: §7.115, §7.116, §7.118, `crates/cfdl-mcp/src/tools/lookup.rs`.

### 7.118 `skeleton` starts a valid model, not a valid structure

Belongs with §5, language and engine (the tooling half). Found the same day.

`skeleton` sets the right bar: it compiles AND runs its output before
returning, so a starter that warns is not a starter. What it returns is a valid
MODEL — the smallest thing that runs.

Every mistake worth guarding here is STRUCTURAL. A securitization is a trust, a
pool, two collection accounts, and per class a holder, two accounts, a note and
a step in each of two waterfalls, in an order that IS the seniority. Assembling
that from a menu is where a class ends up in one waterfall and not the other
(§7.111); starting from a shape that already has three classes wired
consistently is where that cannot happen.

The shape to decide: named shapes per pack — `securitization`, `acquisition` —
answered from a few values and returned already compiled and run, which is the
bar `skeleton` already holds itself to. The shapes belong in the pack rather
than in the tool, because what a deal of a kind consists of is domain knowledge
and the pack is where domain knowledge lives; §7.113's template extension is
most of the mechanism.

MEASURED, 2026-09-07, and it reframes the shape. The same transcribe run
shows an author that fills terms well and does not invent structure:
`penzance_highlands` reproduced the whole income statement — potential gross
rent, effective gross income, opex, NOI, leasing costs, debt service and the
lifetime total — and omitted the partnership above it entirely, all eight
`jv.distribution.*` lines. `auto_abs_tranches` matched none of its economics
and missed 13 name-dependent groups. Ten of the 42 cases carry a waterfall;
`skeleton` emits none, in any pack, in any shape.

So the deficit is not that a starter is small. It is that nothing tells an
author what a model may CONTAIN. Named shapes answer that for the deals
someone thought to name, and a catalogue answers it for the rest:

- the inventory — every construct a model of this domain may hold, contracts,
  accounts, waterfalls, statements, slices, options, and what each is for
- the CLUES for discerning when each applies — the tells in a specification
  that say a promote is present, that cash is split between parties at all,
  that a reserve exists, that a class is subordinated. A component nobody can
  tell they need is not discoverable by having been listed

The clues are the harder half and the more valuable one. A catalogue without
them is a glossary; with them it is the thing that would have made an author
notice that Highlands has a partnership in it.

ORDERING. This entry depends on §7.117 and must not ship before it. A
skeleton that emits a waterfall whose series and accounts an author cannot
then inspect produces a structurally correct model referring to names that do
not resolve — a new failure wearing the shape of progress.

Related: §7.111, §7.113, §7.117.

### 7.119 A non-cash stream cannot be written in a model with a pack

Belongs with §5, language and engine. Found 2026-09-07, authoring the repair
fixtures the catalog is missing.

Two checks disagree, and between them they refuse every `accrual` and
`writeoff` stream in any model with a pack active:

- with a category, `E1379_NONCASH_STREAM_CATEGORY` — a non-cash stream that
  carries one is refused, on `!cash && stream.category.is_some()`
- without one, `E5029_STREAM_MISSING_CATEGORY` — a stream that declares no
  category is refused while a pack is active, on `None if pack_active`, which
  makes no exemption for the non-cash kinds

There is no third option, so the stream cannot be written. A minimal model
proves it: take a `writeoff` moving a balance under `use pack "cre"`, and each
variant produces its own error, neither of which the other admits.

The tell is in E5029's own message, which says the uncategorized stream's cash
"would reach model.total and fold into no subtotal". That is true of a cash
stream and false of a write-off, which moves a balance and reaches no total at
all. The rationale for demanding a category does not describe a non-cash
stream, which suggests E5029 should exempt them rather than E1379 relaxing —
but that is the decision, not a foregone conclusion.

The evidence was found where it should have been: `fixtures/invalid/`
`noncash_stream_with_category` declares `use pack "cre"`, so the fixture that
demonstrates E1379 has no writable repair. A diagnostic whose minimal fix
cannot be stated is the cheapest possible signal that two rules disagree, and
it argues for the repair catalog covering every code rather than most.

Related: §7.115, `crates/cfdl-compile/src/lib.rs` (E1379, E5029),
`fixtures/invalid/noncash_stream_with_category`.

### 7.120 A party metric undefined in any scenario fails the whole run

Belongs with §5, language and engine. Found 12 September 2026 giving
`benchmarks/cre/penzance_one_rosslyn` the Highlands split.

`irr(party.<p>)` and `moic(party.<p>)` are evaluated over the party's own
account (§7.72). When the party contributed and received nothing back, the
fold marks the return undefined (`party 'penzance' never received anything
from account 'penzance_capital'`), and a declared `metric` reading it is
refused with `E5031`. That refusal is right for the deterministic run: a
declared metric with no value is a silent zero waiting to happen (§7.95).
But the same refusal fires when the metric is undefined in ANY named
scenario, and it fails the entire run — the deterministic answer, every
other scenario, and the Monte Carlo — even though the deterministic run
evaluates it.

The scenario where a partner is wiped out is exactly the scenario a
downside exists to show. On One Rosslyn, paid investor-first, the
2026-discount run returned the investor 191,197,430.68 of its capital and
the sponsor nothing, and the whole case refused to run. The case avoided the
question by returning capital pro rata, which is the better convention; the
engine's behaviour is unchanged and the next case that meets it will meet it
the same way.

Two things to decide, in order. First, whether a wiped-out partner's
`moic` is undefined at all: 0.0 is the defined answer (received over
contributed, with received 0), and only `irr` has no rate to solve for —
the fold sets both undefined today because it tests `received <= 0` before
computing either. Second, how a scenario summary carries a metric the
scenario cannot evaluate: `model.irr` is already omitted from a summary
when undefined (the deterministic One Rosslyn summary carries no
`model.irr`, since the contributions leave its cash without a sign change),
so the precedent is to omit the key and say why, not to fail the run. The
deterministic run's refusal should stand.

Related: §7.72 (participant-level return), §7.95 (undefined is not zero),
`crates/cfdl-engine/src/fold.rs` (`party_returns`, and the `E5031` site for
declared metrics), `benchmarks/cre/penzance_one_rosslyn/NOTES.md`.

### 7.121 The cre pack has no multifamily absorption: a lease-up in units from a delivery date is written by hand

Belongs with §1, CRE pack. Found 14 September 2026 authoring
`benchmarks/cre/penzance_one_rosslyn` in full; corrected the same day after
a check against the pack's own rules.

Two rental towers deliver six months apart and each leases up over eighteen
months, net of a vacancy and collection allowance, and the sale reads their
combined income. What the pack offers, exactly:

- `cre.lease` — one tenant's rent with a ramp fixed in the rule,
  `clamp((elapsed + 1) / lease_up_months, 0, 1)` from the term start,
  `lease_up_months` an integer (`E6003`). The ramp's shape is not a term, it
  cannot start at a delivery date inside the term, and it carries no unit
  count and no vacancy.
- `cre.vacancy_loss` — `rate * potential_gross_year / ppy`, where `rate` MAY
  be an expression (the shipped template steps it at a date) and
  `potential_gross_year` may read the rent roll. A lease-up CAN be stated
  this way: a `cre.revenue_line` per tower at full potential rent, and a
  vacancy loss whose rate is one minus the clamped ramp plus the allowance.
- `cre.opex_line` — takes `occupancy = <expr>`, so an expense can follow the
  same ramp. `cre.revenue_line` has no occupancy term.

So the pack is not silent on occupancy, and the first version of this entry
overstated the gap. What it lacks is the construct a multifamily
underwriting states once — units, delivery, pace, allowance, rent and
expense per unit — and emits the occupancy-scaled lines from. Written on the
vacancy contract, the ramp appears twice per tower (the vacancy rate and the
expense occupancy) and the potential-gross series carries rent the building
cannot yet collect; written by hand, as both Penzance cases do, it appears
three times per tower. Either is a restatement. The decision is between a
`cre.absorption` contract and an `occupancy` term on `cre.lease` and
`cre.revenue_line`; the Penzance streams are the fixture either must
reproduce, and the One Rosslyn case is the candidate to restate on
`cre.revenue_line` + `cre.vacancy_loss` first, so the pack is exercised
rather than bypassed (§7.3).

Related: §7.3 (`lease` is one of three cre contracts no case exercises),
`packs/cre/lowering/rules.toml` (`cre.lease`, `cre.vacancy_loss`,
`cre.revenue_line`, `cre.opex_line`), `packs/cre/templates.toml`
(`cre.vacancy_loss.tracking`), `packs/cre/validations.toml` (E6003, E6004
retired), `benchmarks/cre/penzance_one_rosslyn/model.cfdl`,
`benchmarks/cre/penzance_highlands/model.cfdl`.

### 7.122 `cre.construction_loan` funds from a curve's name, so a draw stated as a formula cannot use it

Belongs with §1, CRE pack. Found 14 September 2026, same work.

The contract's `draw_curve` term is the NAME of a declared `curve`, by design
(`packs/cre/README.md`: "the draw schedule is a curve, not a term"). The
argument there is right for a published sixteen-quarter schedule or a
contractor's requisitions, which are data. It is wrong for the other common
case: a draw profile stated as a formula — One Rosslyn's is a parabola over
the construction window, `(t - (start - 1)) * ((start + months) - t)` as a
share of the weights' sum — which a modeller can write as an expression and
cannot write as a curve without tabulating it. Tabulating it means computing
the table outside the model, which is what the case set out to stop. Both
Penzance cases therefore build the facility from fields by hand: equity
funded to date, interest, draw, repayment, balance.

The contract should take the draw as an expression as well as a curve name —
every other cre term already may hold an expression — or accept a field
reference, so a model can state the profile once and hand it over. The
hand-built facility in `penzance_one_rosslyn` is the fixture: same
commitments, same rate, same capitalized interest to the cent.

Related: §7.3, `packs/cre/README.md` (`cre.construction_loan`),
`packs/cre/lowering/rules.toml`, `benchmarks/cre/one_lincoln_street` (the
contract's shipped case).

### 7.123 A field may not read `prev.<account>` in a model with a priced amount, whatever the account holds

Belongs with §5, language and engine. Found 14 September 2026, same work.

`priced_refusal` (`crates/cfdl-engine/src/prepare.rs`) refuses, as
`E5035_SERIES_CYCLE`, any field rule that reads `prev.<account>` when the
model carries a forward-priced amount — the message says so: "a balance may
carry priced cash logic cannot yet see". The refusal is right when the account
is fed by the priced stream or anything downstream of it: the priced pass
runs after the causal walk, so the balance is not settled when the field
reads it. It fires just the same when the account's members are all causal.
On One Rosslyn the tail-valued sale is the priced amount; an `equity_funded`
account rolled from the four contribution streams has nothing to do with it,
and the preference recurrence that wanted to compound on `prev.equity_funded`
was refused. The case restates the equity funded to date in closed form
instead (a cubic in the months elapsed), which is the kind of restatement
docs/42 exists to remove.

The check has what it needs to be exact: `priced_closure` names the priced
streams, and `initial_balances` resolves each account's members. Refuse the
read when a member is in the closure, and allow it otherwise. The message
already distinguishes the two cases in words; the code should.

Related: docs/28 §7 (the priced exception), docs/42 §3.3 (a field reads the
prior close), §7.124 (the same read from a stream),
`crates/cfdl-engine/src/prepare.rs` (`priced_refusal`, `priced_closure`),
`benchmarks/cre/penzance_one_rosslyn/NOTES.md`.

### 7.124 A stream's `prev.<account>` read is a silent zero in a model with a priced amount

Belongs with §5, language and engine. Found 14 September 2026, same work.
The most serious of the four filed that day: a wrong number with no
diagnostic.

The language reference says a stream reads an account's OPENING balance:
docs/01 §9.1, "A stream MAY read an account's OPENING balance as
`prev.<account>` — the prior close, or the `init` in the first period. It
never reads a same-period close (`E1382`)", and docs/03 §3's table lists
`prev.<account>` as readable in stream amounts. In a model without a
forward-priced amount the engine does what the reference says: a stream whose amount is
`prev.pot + 1.0`, rolling into `account pot`, produces 1, 2, 4, 8, 16 from
the first period. Add one stream that prices a sale from the projection tail
and the same model produces 0, 1, 1, 1, 1: every `prev.pot` read returns
zero, the run finishes, and `warnings` is empty. On One Rosslyn a facility
built the docs/42 way — `account facility_balance` rolled from the loan
streams, interest as `prev.facility_balance * rate / 12` — compiled clean,
ran clean, and accrued no interest at all; the loan repaid the wrong amount
and the deal's cash was overstated by 764 million. Nothing said so.

Two things to fix, in order. The read must be loud: whatever the walk does
with accounts in a priced model, a stream reading a balance it cannot see
must be refused the way `E5035` refuses the field, not answered with zero
(§7.95: undefined is not zero). Then it should work: the priced pass changes
how the walk fills accounts, and a causal account has a settled opening in
every period whether or not a priced stream exists elsewhere in the model.
The probe above is the fixture; the One Rosslyn facility is the case that
wants it.

Related: §7.95, §7.123, docs/42 §3.3, `crates/cfdl-engine/src/walk.rs`
(`walk_periods`, the priced pass and `account_balances`),
`benchmarks/cre/penzance_one_rosslyn/NOTES.md`.

### 7.125 A number in a rendered grid cannot be traced back to what made it

Belongs with §5, language and engine (the tooling half). Found 14 September
2026, reviewing what stands between the current surfaces and an Excel user.

`explain` already does the hard half: it traces any number to the journal
entries that produced it, which is the capability a spreadsheet cannot offer at
all. Three surfaces render numbers — the site playground's results panel, the
Python SDK's `results.cashflows()` DataFrame, and a statement — and none of them
can reach it. A reader looking at a cell has no way to ask why it is that
number.

**What it needs is addressing, not new tracing.** A rendered cell is a pair:
the series key it came from and the period index within that series. Both are
already in the results document — keys are stable and `index` carries
`{calendar, start, periods}`. So the ask is that `explain` accept
`(series, period)` and return the acts, and that each surface offer it: a click
in the playground, a method on the SDK results object, a cell reference in a
statement.

**The one known hazard** is `docs/26`'s "explain matches by name, not identity":
the dot-versus-colon spelling means a stream's own act never matches on the full
key. A cell-level lookup walks into that immediately, and it must be fixed
first or every cell in a stream's own row will fail to explain.

Ordering: independent of §7.117 and §7.118, and cheaper than either. It is the
single largest gap between "a results document a reviewer trusts" and "a grid a
modeller interrogates".

Related: `docs/26` (explain matches by name), §7.117, `crates/cfdl-mcp`.

### 7.126 The ontology can generate an entry form and does not expose enough to

Belongs with §5, language and engine (the tooling half). Found 14 September
2026, asking what stands between the ontology and a form a modeller could use
instead of a text editor.

A form over a typed ontology should be GENERATED, not configured. Salesforce
and its kind are configured per object — someone builds a layout, picks fields,
writes validation — and the configuration is a second artefact that drifts from
the schema. `lookup` already returns most of what a generator needs per contract
type: `fields` with `field_type`, `required`, `unit`, `description` and
`one_of`; `roles` with the master's word beside the pack's and an `unbound`
flag; the `refines`/`master` chain; `lines` and `side`; and a `template`.

Four affordances fall out of that with no configuration at all. `one_of` is a
radio group — `rent_year` against `rent_psf` is a mutually exclusive choice the
ONTOLOGY states. `unit` is the suffix and the parse rule. `required` is the
asterisk, and the loader already refuses a template that omits one. And the
master chain gives a label vocabulary, so a form can say "landlord" to a CRE
modeller and store `lessor`.

**Validation is where this beats a configured form, and it is already built.**
Typed assumptions, `within [lo, hi]`, the `fraction` domain (`E5041`), pack
bounds with warn-versus-refuse severity, and `E1372` for a missing required
term are the SAME checks the compiler runs. A generated form therefore cannot
accept what the model would reject, and cannot drift, because there is one
implementation rather than two.

**What blocks it is §7.117 exactly**, and the list is short: entity types (so
"what may this contract be written on" is a dropdown rather than a guess),
what a lowering publishes (so a step's amount can offer the series that exist),
the categories a rule emits (so a statement row builder has a list), and the
accounts on a subject (so "pay to" is a picker). Without those a form renders a
contract's terms and nothing around them, which is why a prototype built on
today's surface feels thin: it is working from half the ontology.

**Order:** §7.117 first — alone it probably reaches a better-than-configured
single-contract form. Then §7.113 (template extension) so a form starts from a
pack template rather than empty, then §7.118 (named shapes) so a whole deal can
be scaffolded rather than one contract at a time.

**The honest limit.** A form is good for the structured 80% of a deal and bad
for the bespoke 20%. An expression editor on any term field has to be
first-class, or a modeller meets the wall and leaves. Terms already accept
per-period expressions, so the language side of that escape hatch exists.

Related: §7.117, §7.113, §7.118, `crates/cfdl-mcp/src/tools/lookup.rs`.

### 7.127 A scenario diff can name the source line that moved the number, and nothing renders it

Belongs with §5, language and engine (the tooling half). Found 14 September
2026, beside §7.126.

Scenarios publish as their own block carrying the same metric map and series
shape as the deterministic run, so three panes are computable from ONE results
document with no engine change: which `inputs.*` differ, the per-period series
delta, and the metric delta. That much is table stakes and any BI tool does it.

**Three things follow from the model being text plus a journal, and no tool on
a binary format can do them.**

CAUSE, NOT JUST MAGNITUDE. A workbook diff says "H47 changed". We can put the
results delta beside `git diff` over the model and say *NPV moved 1.2m;
`model.cfdl:214` changed `renewal_probability` 0.65 to 0.75*. The join is
available today and unrendered: every IR node already carries `provenance` with
`source_file` and `source_span`.

BEHAVIOUR, NOT ARITHMETIC. The journal records each step's payment, each clamp,
each transition, so a diff can say WHICH STEP behaved differently — "the OC test
passed in month 14 under B, so 600 was not diverted". That is a causal
explanation of a delta rather than a subtraction, and it composes with §7.125:
explain-in-place on a delta cell rather than a value cell.

SHAPE, NOT ONLY VALUES. Two scenarios may differ structurally — a contract in
one and absent in the other. `results_version` 0.13 publishes `graph.contracts`,
so the deal graph itself is diffable: "B has no mezzanine tranche". A
spreadsheet has no deal graph to diff.

**Sequencing.** The first two panes are cheap and unremarkable. The source
attribution is the differentiator, needs nothing new, and should therefore be
built first rather than last — which is the opposite of how a diff view is
usually approached.

Related: §7.125, §7.126, `docs/06` (`graph.contracts`, scenarios).

