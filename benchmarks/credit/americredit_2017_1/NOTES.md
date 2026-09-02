# Notes — americredit_2017_1

What the reference implementation had to recover, and what it has not.

Status: the reference reproduces **192 of 195 informative cells** inside the
grid's own whole-percent rounding floor, and **all 48 published weighted
average lives** exactly. The CFDL model agrees with it to **4.4 cents** on a
$305m class, across seven classes and all 63 periods.

## The error distribution

The published grid is whole percentages, so a model that is exactly right has
errors uniform on [0, 0.5]: mean near 0.25 and maximum over *n* informative
cells near `0.5n/(n+1)`. Measured over the 192 cells inside the floor: **mean
0.2470 against 0.25 predicted, maximum 0.4990 against 0.4974 predicted.** That
is the shape of a model whose remaining error is the issuer's rounding and
nothing else.

The three cells outside the floor are not distributed like that. All three are
Class A-1 in its second or third distribution date — 05/18/17 at 0.50% and
1.00% ABS, and 03/18/17 at 2.00% — and all three are the same sign, the model
retiring A-1 by 0.60 to 0.68 of a point less than the prospectus does. That is
about $1.1m of principal in a single month, on a $182m class.

**This is an open item.** It is smaller and better localised than what it
replaced, and one candidate has been tested and rejected rather than fitted: a
stub first interest period, 25 days on a 30/360 basis or 23 actual days from
the 23 February closing to the 18 March payment. Both are worse, which is the
arithmetic consequence of assumption (iii), that every month has 30 days.

**What it replaced, and how it was wrong.** The first version of this file
recorded eleven cells outside the floor and described them as the model
retiring A-1 *slower* than the prospectus. The sign was backwards: the model
was retiring A-1 **faster**, by up to a point, which meant the model had about
$1.5m more cash in the first month than the issuer did. Read that way the
answer was immediate — the servicing fee, which is $1.9m a month on this pool.
Eight of the eleven cells closed, and two published lives that had been off by
0.01 became exact. A wrong sign in a note is worse than no note: it points the
next reader away from the answer.

## Conventions recovered

**The servicing fee accrues monthly, so the January pools pay it twice in the
first collection period.** The fee is 2.25% per annum on the pool balance, and
the first collection period spans two months for a January-cutoff pool — the
same fact that gives it two scheduled payments. Charging one month on the
opening pool balance instead leaves eleven cells outside the floor and misses
two of the published lives; accruing it per payment month leaves three and
misses none. Note where it sits in the table below: every other rejected
reading is off by a hundred cells or more, and this one by eleven. A wrong
convention this small does not announce itself.

Worth recording for a second reason: the model already had this right on the
cash side and wrong on the balance side. Clause 1 pays the pack's own servicing
series, which accrues per contract per period and therefore charged the
January pools twice without being told to; the balance recurrence carried its
own copy of the arithmetic and charged one month. That is backlog 7.37 in
miniature — the two halves of the same deal, drifting because nothing holds
them together.

**A January cutoff makes two payments before the first distribution date.** Six
of the twelve assumed pools have an assumed cutoff of 1 January 2017 and six of
1 February. First due date is the last day of the cutoff month, distributions
begin 18 March, so the January pools contribute their 31 January *and* 28
February payments to the first collection period and the February pools
contribute one. This is the largest single convention in the case: one payment
for every pool misses all 195 informative cells, three for the January pools
misses 195 as well, and the correct reading misses 3.

**ABS runs from origination, and it can exhaust a seasoned pool outright.**
Prepaying contracts each month are a constant percentage of the pool's
*original* contract count, so a pool seasoned 53 months has already lost
53 x ABS of its contracts. At 2.00% ABS that is more than all of them: four
pools, $59.1m between them, prepay in full in the first collection period. That
is not a degenerate case to be guarded against — it is what produces the
published 32% for Class A-1 at 2.00% ABS after one month, which no smoother
reading reaches. Running ABS from the cutoff date instead misses 166 cells.

**The step-down, not the turbo, is what shapes the middle of the deal.** The
first draft paid the full principal collections to the notes every month and
missed 80 cells by up to 100 points. The Principal Distributable Amount is
principal collected *less* the Step-Down Amount, and the step-down is whatever
would take the notes below `0.8525 x Pool + Reserve`. Once the target is met the
notes track that line exactly, month after month, and the retained principal
goes to the certificateholder. Paying the full collections misses 174 of the 195
informative cells, by up to 98 points.

**The step-down has a floor of 0.50% of the initial pool, and it binds.**
Without it the notes follow the required balance all the way down,
overcollateralization drains to 0.7%, and 27 cells miss by up to 6 points in a
contiguous block where Class C is retiring. The floor is stated twice in the
prospectus, and sweeping it as a free parameter puts the minimum at 0.50%
exactly:

| Floor, percent of the initial pool | Misses | Worst |
|---|---:|---:|
| 0.00% | 27 | 6.05pp |
| 0.25% | 27 | 3.31pp |
| **0.50%, as stated** | **3** | **0.68pp** |
| 0.75% | 31 | 3.31pp |
| 1.00% | 37 | 6.13pp |

The document and the arithmetic agree on the same number, which is the point of
running the sweep at all.

**Weighted average life runs 30E/360 from the closing date to the 18th, with a
25-day stub.** Measuring from period zero at a flat 30/360 overstates every
life by 0.014 years, which is invisible on a long class and fatal on a short
one: 20 of the 48 published figures miss. With the stub, all 48 are exact.
This is the same finding `docs/20` §4 records for Ginnie Mae and Fannie Mae —
three issuers now, three different day counts for this purpose, none of them
stated in the document.

**Re-amortizing a pool over its remaining term and carrying it from an original
schedule are the same thing.** Assumption (ix) says each contract's scheduled
payment repays its current balance over its remaining term, which reads like a
different model from carrying a level-pay schedule forward from origination.
The balance paths are identical — `bf(a, T) / bf(s, T) = bf(a - s, T - s)` for
a level-pay contract — and the implementations agree to the last digit. Worth
recording so nobody spends the afternoon twice.

## Rejected readings

Each of these was tested against all four speeds and is worse. Kept because a
plausible wrong reading costs the same to rediscover as it did to reject.

Counts are cells outside the rounding floor, out of 195 informative.

| Reading | Misses |
|---|---:|
| As implemented (**used**) | 3 |
| One scheduled payment in the first collection period, every pool | 195 |
| Three for the January-cutoff pools | 195 |
| No reserve credit — a flat 14.75% target | 176 |
| Reserve credit at 2.0% of the *current* pool | 176 |
| Principal paid in full, no step-down | 174 |
| ABS measured from the cutoff rather than from origination | 166 |
| Servicing charged one month on the opening pool balance | 11 |

## What this case will assert, and what it cannot

The grid is a to-call grid, so the 195 informative cells and the 24 to-call
lives come from one run. The 24 to-maturity lives need a second run with the
redemption option suppressed, and they differ from the to-call figures only on
Class D. Twelve of the 48 published lives are therefore the only evidence the
no-call scenario produces, and `docs/20` §3.1 still applies: a published
weighted average life cannot be asserted, only reconciled in prose.

Mutation testing has not been done yet. `docs/20` §3.3 asks for it, and this
case has an obvious hole to check: a residual assertion would be one-sided
here, because the certificateholder's step-down release absorbs anything the
notes are not paid.


## The model, and the arithmetic it has to state twice

A class balance is a field. A field cannot read what the waterfall paid it —
see `docs/13` §7.37 — so `model.cfdl` states the distribution twice: once
lagged, inside the seven balance fields, and once at the current period, inside
the twenty-two clauses that pay the cash. Nothing enforces that they agree.
That is the case's structural weakness and it is worth knowing before reading
the file.

Three fields carry the collateral as closed forms of the twelve assumed pools —
`pool_bal`, `pool_prior` and `pool_int` — for the same reason: a recurrence
cannot read the pack's own series either. The pack contracts produce the cash
the waterfall allocates, so the pool is stated twice as well, independently,
and both are pinned to the published grid through `expected.csv`.

## The distribution, in four lines instead of forty

The first model wrote the deal the way the prospectus does — a Step-Down Amount
subtracted from the Principal Distributable Amount, then an Accelerated
Principal Amount capped by both available cash and the distance to target — and
the expression ran past forty lines per class, most of it the same
subexpressions repeated at different depths.

The same arithmetic collapses to a statement about where the notes END the
period:

```
ending = min(pool - floor, max(required, notes - principal - max(excess, 0)))
```

The notes finish at the required balance; cash may stop them getting there; and
overcollateralization may not fall below the floor. Clause 18 is then
`min(total, principal)` and clause 20 is `max(total - principal, 0)`, which is
what those clauses mean — collections first, excess cash after.

Checked rather than assumed: the two formulations agree to **$0.0000** at all
four published speeds, on every class and every period. The prospectus's form
is the one that explains *why*; this one is the one that fits on a page.

## What the cash assertions caught

`expected.csv` asserts every clause of the waterfall as well as every balance.
It was not always so, and adding the cash columns found two defects in a model
whose grid had matched all along — which is the argument for asserting them.

**Clause 16 fired every month for the last third of the deal.** The parity
clauses reduce a class's balance "to the Pool Balance", and the model compared
the balance each class carried into the distribution against the pool balance
*after* that period's collections. Those are two different dates. Once monthly
principal exceeded overcollateralization — around $13m against $5m, from period
37 — the difference read as undercollateralization and clause 16 paid it,
taking $4m a period from the certificateholder. The balances never moved,
because the recurrence carries its own copy of the arithmetic and its copy was
right. Both sides are now measured on the same date, which is what makes the
clause mean what the prospectus says it is for: "principal payments made to
cure this undercollateralization, if any then exists".

**The reference lost the distribution that ends the deal.** Its period loop
broke out as soon as every class was retired, and the break came before the
line item was recorded — so the clean-up call's own distribution, the largest
in the deal, was missing from the record and `expected.csv` asserted zero for
it. The model was right and the expectation was wrong, which is the failure
mode worth fearing in a case: the reference is only a check while it is
checking.

**And a disagreement that was a real question.** At the call the reference
booked the whole payoff as accelerated principal; the model booked that month's
collections under clause 18 and the redemption price under clause 20. The
model's reading is right — the receivables still pay that month — so the
reference now does the same.

## The reserve account, and the two bugs it found

Clause 19 is a reserve of 2.0% of the initial pool, funded at closing. It is
never drawn here — no losses are assumed — but it is not inert: the
overcollateralization target is stated *against* it. The Required Pro Forma
Note Balance is 14.75% of the pool "less the amount required on deposit in the
reserve account", so the reserve sets how far the turbo runs, and through that
it sets every class's retirement date.

The model carried it as a literal — 20,239,398.59, written out **twenty-eight
times**, once in each of the seven balance recurrences and twenty-one times
across the waterfall's steps — and clause 19 itself was
`pay reserve_topup to party.certificate = 0.0`: the right amount, to the wrong
payee, for no stated reason. It is now `account reserve`, funded at closing by
the account's own `from` inflow, and clause 19 is the top-up
`max(0.0, inputs.reserve_required - prev.reserve)`. It still pays nothing, but
it pays nothing *because the balance is at target*, which is what the clause
says. Were the reserve ever drawn, the step would restore it.

**Funded at closing is not a distribution.** The reserve comes out of note
proceeds before the first collection period, so it cannot be funded from the
waterfall — the waterfall allocates collections, and taking the reserve out of
them would spend cash the deal never spent. The account's `from` clause is the
inflow that is not an allocation, and it fires once, at period 0.

The reserve is also now *auditable*, which a literal never was. The journal
carries the closing inflow as its own act —
`period 0, actor account:reserve, action inflow, 0.0 -> 20239398.5856` — and
then clause 19 applying $0.00 at each of the 62 distributions, with the balance
before and after. Twenty-eight copies of a number produce no such record.

**The conversion moved nothing.** Measured with the literal's own value held —
the rounding is a separate change, below — all 177 series agree with the
pre-conversion run at every period with zero difference: not within tolerance,
identical. That is the assertion. The reserve was already load-bearing, so
restating it as the balance it is has to leave the deal alone, and anything
else would have meant the two spellings were not the same reserve.

### Two bugs, both found by trying to declare it

**The account was silently swallowed.** `assume <name> = <expr>` has no
terminator, so the parser scans to the next top-level statement, and
`is_statement_start` is the list that says where that is. `account` was missing
from it. An account declared after an assumption did not exist — no diagnostic
where it was swallowed, only `E1347_UNRESOLVED_ACCOUNT_REF` at whatever
referred to it later, and nothing at all if nothing did. `lifecycle` was
missing from the same list for the same reason and is contextual rather than a
keyword, so it failed loudly and misleadingly instead, reporting an unexpected
`{` three lines from the cause. Both are fixed; both are pinned
(`fixtures/valid/account_after_assume`, `fixtures/valid/lifecycle_after_assume`).
The list's own comment already records a `metric` declared after a contract
vanishing this way, so `account` and `lifecycle` are the second and third
instances of one omission — which is the argument for the invariant now stated
at the function: every arm of the statement dispatch appears in the list.

**A conditional window bound was read as forward, and that cost the account its
balance.** The first distribution draws two collection periods — six of the
twelve assumed pools have a January cutoff and six a February one — so the pot
is bounded below by `if(time.t == 1.0, 0.0, time.t)`. `window_bound_is_backward`
reads bound source text and recognised literals, `time.t` plus signed literals,
and `max`/`min` of those; it did not recognise `if`. So a bound that is
backward down both branches was called forward, the model kept the **column
order**, and the column order has no periods to carry a balance through. The
account compiled, published no balance, and every `prev.reserve` read as zero —
which collapsed the notes to zero at period 2.

The engine said so rather than publishing zeros quietly ("this model declares
accounts, but a forward-reaching read keeps it on the column order"), and the
benchmark runner fails on warnings, so it could not have shipped. `if` now
joins `max` and `min` as "the value is one of these operands, so it is backward
when all of them are" — the condition is not examined and need not be.

This case is therefore also the first evidence that the corpus's most intricate
waterfall gives the same answer under the period walk as under the column
order: 177 series, 63 periods, zero difference.

### What the rounding was worth

The literal was 20,239,398.59. The deal is 2.0% of 1,011,969,929.28, which is
20,239,398.5856, and the reference computes the product
(`reference_gen.py`, `RESERVE = 0.02 * POOL0`). Stating the rule rather than the
rounded number moves **260 cells**, by at most **0.0044 dollars**, on figures of
about twelve million — the model moving onto the reference's own arithmetic.
The published assertions are unaffected at the case's tolerance.

The same shape remained in two other places. 101,196,992.93 — 10% of the
initial pool, the clean-up call threshold — was written out thirty times and is
now `assume call_threshold`. 5,059,849.65, the step-down floor at 0.50% of the
initial pool, is still a literal in twenty-eight places and is one edit away.

## The clean-up call, and what it ended

The call was in this model from the start, as arithmetic: a pool-balance
comparison in every principal step, and in the pot a hand-written rising edge —
`pool_bal <= X and pool_prior > X` — to catch the one distribution that pays the
redemption price. It reproduced the published tables exactly. What it did not do
was end the deal.

**The deal ran twenty-three periods past its own end.** The twelve pool
contracts kept amortizing to the end of the book, the certificateholder took
every dollar of it through clause 22, and `model.total` asserted the result:
$100,885,317.21 of collections on receivables the servicer had already bought.
The case said so in its own "does not assert" list and cited backlog 7.39 for
it — *a contract cannot be bought out*.

**The machine was already declared, and unused.** `Credit.Asset.LoanPool` binds
`credit.pool`, whose own description in `packs/credit/ontology/types.toml` says
a clean-up call "is NOT a state: it is an occurrence, and it drives
`amortizing -> retired` as an event with a no-return topology" — written when
`docs/36` §2.2 retired `called` as a state. All thirteen entities in this model
carry that machine and none of them ever left `amortizing`;
`deterministic.transitions` was empty.

The case also carried a stale citation, "backlog 7.39 — a contract cannot be
bought out", for a gap that had since closed. That is the failure mode a
deleted-on-close backlog invites: the entry disappears and the work waiting on
it does not notice.

### Why the transition is a period after the redemption

The state is evaluated as a period opens. A pool is carried into the period,
collects, and the period ends; so the guard reads `pool_prior` — the pool the
trust carried in — and fires at period 48. The redemption price is paid at 47,
on the last distribution made while the trust still owns the receivables, and
the pot says exactly that, as the period the pool crosses the threshold:

```cfdl
if(container.trust.pool_prior > inputs.call_threshold
   and container.trust.pool_bal <= inputs.call_threshold, container.trust.pool_bal, 0.0)
```

**That two-sided test was in the model from the start and it was never the
defect.** What was wrong with the original was the threshold written out as a
literal thirty times, not the comparison, and it is worth recording that the
first two attempts to "improve" it made it worse: reading the trust's own status
put a hand-asserted state where a derived one belongs, and folding
`series_sum("credit.pool.interest.*", …)` replaced two checked field reads with
an unchecked pattern. A misspelled field is refused; a selector that matches
nothing folds to zero in silence, and here that silence would have stopped the
largest distribution in the deal.

The pot needs a SAME-PERIOD answer, which is the other reason it reads the trust
rather than its state: a container's derived state lags its parts by one period
by construction, so `wound_up` arrives too late to gate the redeeming
distribution and would be the wrong question anyway.

### The pool is extinguished, not silenced

`credit.pool_level_pay` carries exactly one piece of state. Scheduled
amortization is a closed form in elapsed periods, but attrition is a recurrence
— `credit_level_pay_survival<suffix>`, `field_init 1`,
`field_next prev * ((1 - default rate) - prepay rate)` — and every one of the six
streams the contract lowers is balance x amortization factor x that fraction.

So a purchased pool is expressed by writing zero into it. The recurrence resumes
from zero, `prev * anything` stays zero, and all six streams are zero for good.
No stream is gated and no contract is switched off: the pool has no surviving
balance, which is what `retired` means.

The first version did gate, with `deactivate stream` on all seventy-two lowered
streams. The cash was identical to the byte — the entire difference between the
two runs was twenty survival fields, which under the gated version went on
declaring that 48.78% of pool p01 was still performing while its streams were
silent:

```
asset.p01 survival, periods 45-51
  deactivate: [0.487805, 0.487805, 0.487805, 0.487805, 0.487805, 0.487805, 0.487805]
  survival  : [0.487805, 0.487805, 0.487805, 0.0,      0.0,      0.0,      0.0]
```

A declared state and a behavior that disagree, with only the behavior enforced.
The current spelling cannot drift that way because one fact drives both.

**The pack should do this and cannot.** It declares the `retired` state and it
declares the survival recurrence and connects them nowhere, so `retired` has no
consequence for any model but this one. The entry action that would fix it —
`on enter retired { set <survival> = 0 }` — cannot be written, because the field
name is templated per contract instance while a lifecycle is per type. Recorded
at `docs/13` §7.96.

### The trust winds up because it is empty

The trust is a `Container.SPV`, not a loan pool. It holds the twelve pools; the
pools hold the receivables; the servicer buys the receivables. Typing it as a
`Credit.Asset.LoanPool` is what made the first version retire the trust and
leave its twelve pools amortizing.

Its wind-up is derived rather than asserted. The container carries what it still
owns — its parts' surviving fractions, summed through `prev` — and the edge
reads it:

```cfdl
amortizing -> wound_up when container.trust.surviving == 0.0
```

It lands at period 49, one period after the pools retire at 48, and that is
correct rather than a lag to remove: state is evaluated as a period opens, so
the trust can only see a settled pool. Winding it up any earlier would end the
trust before its pools' last period of activity had been counted — and period 47
is the redeeming distribution, the largest in the deal.

Which is also why the redemption clause in the pot asks the POOLS whether the
trust still owns them, rather than asking the trust its own status: the pot
needs a same-period answer, and a container's derived state is a period behind
by construction.

**What this cost.** Summing twelve named fields is an enumeration of something
the run already knows. Containment is materialized and published — `graph.entities`
carries one row per entity naming its parent, and twelve of those rows say
`container.trust` holds twelve pools — so the trust reconstructs by hand a fact
its own results state. Nothing lets a model ask the relation how many parts it
has, or how many are still amortizing. Recorded at `docs/13` §7.98.

A wrong turn worth recording, because it looks reasonable: the first attempt
guarded on the container's aggregated cash. It is the wrong question twice over —
a container's cash cannot say whether it still holds anything, since a container
with twelve live pools nets zero in any idle period — and the guard also failed
silently, folding an unresolvable selector to zero and firing in the first
period. That silence is `docs/13` §7.97.

### What moved, and what did not

Nothing at or before the call. All 179 series the run publishes are
bit-identical through period 47 — 8,592 cells, none of them differing — so the
published grid, the 48 weighted average lives and the five-cent reconciliation
are untouched; every changed cell is at period 48 or later. What moved is what should: `model.total` falls by the phantom
$100,885,317.21, the certificateholder's total by $100,874,067.21, and
`model.wal_years` from 1.89 to 1.65.

One consequence is worth reading rather than fixing. The pack's
`domain.credit.principal_paid_to_date` now stops at $916,770,831.19 instead of
running to the initial pool, and the difference is $95,199,098.09 — the
redemption price exactly. That is correct: the trust collected the rest, and the
servicer bought what was left.

## What the model does not carry

- **The final scheduled distribution dates are written but inert.** With no
  losses assumed every class retires years early, so clauses 5, 8, 11, 14 and
  17 pay nothing at every period and every speed. They are in the model because
  the deal has them.
- **The parity clauses are inert for the same reason.** The pool always exceeds
  the notes, so clauses 4, 7, 10, 13 and 16 pay nothing.
- **The pool's closed form runs past the call.** `pool_bal` keeps amortizing to
  the end of the book after the trust is retired, and that is the receivables'
  balance rather than the trust's — the servicer owns them and they carry on
  paying somebody. Nothing reads it after period 48 except the call guard,
  which cannot fire twice.
- **One speed.** The case runs at 1.50% ABS. The other three published speeds
  are this model with `abs_speed` changed, and `docs/20` §2.3 is the reason
  they are not four directories.
