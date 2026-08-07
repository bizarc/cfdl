# Ordered waterfall — design

Status: **proposal.** Nothing here is implemented.

A waterfall is an author-declared priority over a pot of cash. Each step takes
what it is owed, up to what is left, and the remainder passes down. It is not a
dependency graph to be solved, which is why it needs no cycle detection and does
not relax any stream reference rule — the boundary `docs/14` §5 settled.

This is the gate on roughly two thirds of the pack roadmap: securitisation
tranches, private-fund carry, CMBS, RMBS, and the LBO exit waterfall that
`benchmarks/opco/lbo_option_pool_exit` currently expresses by hand.

## 1. The bar, and why this deal

The test is a real, fully specified waterfall from a free public document: the
22-step priority of payments in the AmeriCredit Automobile Receivables Trust
2017-1 prospectus. It is a US SEC filing, so the text is a public record.

The prospectus specifies the structure but publishes no period-by-period tranche
schedule, so it is a **specification source, not a numeric benchmark**. It
answers "can CFDL express this?", which is the roadmap's own stated bar for this
candidate. Numeric agreement is a separate step, against a runnable reference —
see §6.

## 2. The 22 steps are 7 rules

Read literally, the deal has twenty-two payment instructions. They are seven
rules applied to different payees:

| rule | steps |
|---|---|
| pay a stated amount | 1, 3, 6, 9, 12, 15 |
| pay a stated amount, capped; the excess is paid at a later step | 2, 21 |
| pay down to a target computed this period | 4, 7, 10, 13, 16, 20 |
| pay the remaining balance, on a stated date only | 5, 8, 11, 14, 17 |
| pay a stated distributable amount | 18 |
| top an account up to a specified level | 19 |
| pay everything that survives | 22 |

That is the whole surface. A construct with seven payment forms and an ordered
list expresses this deal completely, and the same seven express a CLO, a CMBS
and a fund waterfall, because the vocabulary of priority-of-payments is small
and the variety is in the ordering and the payees.

The three that are not simply "pay X" are the ones worth designing carefully.

**Pay down to a target** (4, 7, 10, 13, 16, 20). "To the extent necessary to
reduce the combined principal balance of the Class A and Class B Notes to the
pool balance." The amount is not stated; it is whatever closes a gap measured
*this period*, bounded by what is left in the pot. This is the "solve-to-target
within a period" the roadmap names, and it is not a solver: it is
`min(remaining, max(0, current - target))`.

**Capped, with a later overflow** (2, 21). Trustee fees are paid subject to an
annual limit, and step 21 pays whatever exceeded it. So a step needs a cap, and
a later step needs to name the overflow the cap created. The overflow is state,
not a recomputation.

**Everything that survives** (22). The residual is a first-class step, not an
implicit leftover. Making it explicit is what stops a model silently losing
cash: if a waterfall does not declare where the remainder goes, that should be a
compile error rather than a quiet zero.

## 3. Proposed surface

```cfdl
waterfall abs.distribution on entity asset.trust {
  from state.available_funds

  pay servicing        to party.servicer     amount state.servicing_fee
  pay trustee_fees     to party.trustee      amount state.trustee_fees
                                             cap inputs.trustee_annual_cap
  pay class_a_interest to party.class_a      amount state.class_a_accrued
  pay class_a_target   to party.class_a      down to state.pool_balance
                                             measuring state.class_a_balance
  pay class_a_final    to party.class_a      amount state.class_a_balance
                                             when time.date >= inputs.class_a_final_date
  ...
  pay reserve_topup    to account.reserve    up to inputs.specified_reserve_amount
  pay oc_build         to party.class_a      down to state.oc_target
                                             measuring state.total_note_balance
  pay trustee_excess   to party.trustee      overflow of trustee_fees
  pay residual         to party.certificate  remainder
}
```

Six forms, one per rule, plus `when` as a modifier available to any of them:

| form | meaning |
|---|---|
| `amount <expr>` | pay this, or what is left, whichever is smaller |
| `cap <expr>` | pay no more than this; record the shortfall as overflow |
| `overflow of <step>` | pay the shortfall an earlier capped step recorded |
| `down to <expr> measuring <expr>` | pay `max(0, measuring − down_to)`, bounded by what is left |
| `up to <expr>` | top the payee up to this level |
| `remainder` | everything still in the pot |

`when <expr>` gates any step on a condition read from period-open state, the
same rule events and options already follow.

## 4. Evaluation

A waterfall runs **after** streams and states for the period are known, and
before results are published. It reads period-close state, because the pot it
allocates is this period's cash and the balances it measures are this period's
balances.

Steps evaluate in declaration order. The pot is a single running value; each
step takes `min(what it is owed, what remains)` and reduces it. A step that is
gated off by `when` takes nothing and does not consume.

Every step emits a stream, so a waterfall is not a new kind of output: it is a
declaration that lowers to per-step cash flows, categorised and attributable to
a payee. That keeps statements, metrics and the results schema unchanged.

## 5. What has to be decided before implementation

1. **Does the pot ever go negative?** No — a step takes at most what remains. A
   shortfall is a fact to publish, not an overdraft. Whether an unpaid step
   accrues (a deferred-interest balance on a junior note) is a per-step property
   and probably a second form, not a default.
2. **Where does the pot come from?** `from <expr>` above is a single value. A
   real deal has an interest waterfall and a principal waterfall that cross-link
   — interest diverted into principal redemption on a trigger failure. Two
   coupled waterfalls is the roadmap's own requirement, and one pot does not
   express it. Options: two declarations with an explicit cross-link step, or a
   named-pot construct. **Unresolved.**
3. **How is a shortfall published?** Per step, per period, in results. It is the
   thing an analyst reads first, so it should not have to be derived by
   differencing.
4. **Does `down to` need to see other steps' effects?** **Settled in §12: yes,
   and it costs nothing.** The clause is a read of prior steps' payments, which
   an ordered evaluation already has. No model state is mutated and the stream
   model is untouched.

Questions 2 and 3 remain open, and no longer depend on question 4.

## 6. Validating it

Expressiveness first: encode all 22 steps and have the compiler accept them,
with a fixture and a golden. That proves the surface covers a mainstream
consumer ABS deal.

Numbers second, and against a different source. Two catalogued engines publish
full tranche cash flows under permissive licences — **AbsBox** (Apache-2.0) and
**Hastructure** (BSD 3-Clause). Unlike every reference the benchmark suite uses
today, those may be vendored and wired into CI, so a securitisation case can be
checked continuously rather than against numbers copied once. An SEC Exhibit
99.4 with published weighted-average-life tables per class anchors the endpoint.

## 7. What this unlocks

Named by the roadmap as gated on this construct: CLO/ABS/CDO tranching (rank 1),
private-fund carry waterfalls (rank 2), CMBS and RMBS, and the toll-road and
project-finance cascades. It also retires two existing workarounds — the LBO
exit waterfall built by hand in `benchmarks/opco/lbo_option_pool_exit`, and the
equity-first construction draw in `benchmarks/cre/one_lincoln_street` that the
CRE pack cannot express.

## 8. Templates or freeform

Both, layered — and the layering already exists in this language.

A **stream** is the primitive; a **contract** is a pack template that lowers to
streams. The user guide states the rule that falls out of it: *use contracts for
what the pack understands and streams for everything else*, and the two mix
freely in one model. Waterfalls take the same shape:

| layer | waterfall | precedent |
|---|---|---|
| primitive | ordered allocation over a pot | `stream` |
| template | `credit.sequential_pay`, `opco.american_carry` | `contract` |
| escape | write the steps out | a hand-written stream |

Neither half works alone. **Templates only** fails on the first deal that
reorders two steps, and reordering is exactly what an indenture does — the steps
are bespoke per deal even though the rules are few. **Freeform only** means
every ABS model is twenty-two hand-written steps, which is unreadable, and it
abandons the pack's whole proposition: declare business terms, not mechanics.

The design consequence is a bar, and it is stronger than "can it express the
AmeriCredit deal":

> **Every template must lower to the primitive with no escape hatch.** A
> template that needs a special case in the engine is a sign the primitive is
> wrong.

A second rule follows from what the roadmap actually contains: **a template
parameterises the ordering, it does not hide it.** `sequential_pay` taking an
ordered list of classes is useful. A template that reduces the twenty-two steps
to three terms is a trap, because the next deal differs in the ordering and
nothing about it can be reused.

## 9. What the ABS deal hides

Encoding one deal was the right start and it is not sufficient. Across the
roadmap there are **31 waterfall-shaped requirements**, and the consumer ABS
deal is the easiest of them. Three generalisations do not appear in it at all:

| structure | pot | target test | coupling |
|---|---|---|---|
| ABS sequential pay | cash | period ratio | none |
| CLO | cash | period ratio | interest and principal |
| CMBS | cash | per-loan status | shortfalls written up from the bottom |
| Aircraft ABS | cash | DSCR/LTV | **permanent** regime change |
| Private fund carry | cash | **cumulative IRR** | none |
| GP-led continuation | cash | **cumulative MOIC** | none |
| GP stakes | cash | — | **nested**: fund → firm → holder |
| Film recoupment | cash | — | per title, then portfolio |
| Water rights | **volume** | seniority | none |

**The pot is not always money.** Water rights allocate a physical supply to
senior rights first — the same primitive over a quantity, not a currency. That
is cheap to decide now and expensive to retrofit: the construct should be
generic in what it allocates, and money should be one instantiation.

**Waterfalls nest.** GP stakes runs a fund waterfall, rolls its carry into a
firm-level line, and splits that to a stakeholder. Film recoups per title and
then at portfolio level. So a waterfall's output must be able to be another
waterfall's pot, which makes composition a requirement rather than a
convenience.

**Cumulative targets are a second shape**, measured since inception rather than
for the period: capital returned, a compounded preferred return, a MOIC
threshold, an IRR hurdle. §12 works through whether any of them needs a solver.
The answer is no, but the reasoning is not obvious and it is the difference
between a primitive and a numerical library.

**Regime change is a lifecycle, not a waterfall feature.** An aircraft ABS
trigger that permanently reorders priority is an asset changing state, which
this language already has. A waterfall gated with `active in state` reuses
lifecycles, events and the transition log rather than inventing a parallel
mechanism — and the transition log then records *when* the deal flipped to rapid
amortisation, which is exactly what an analyst asks.

## 10. When a waterfall runs

A waterfall is a **post-free-cash-flow distribution**, and it happens on a
cadence of its own. Two shapes cover the roadmap:

- **Every period.** An ABS distribution date, a CLO payment date, a project
  finance cash cascade, a fund's quarterly distribution.
- **Once, at the end of a hold.** An LBO exit waterfall, a fund liquidation, a
  film's final recoupment. `benchmarks/opco/lbo_option_pool_exit` is exactly
  this, written by hand.

So a waterfall takes a **`schedule`**, the same construct a stream takes, and
the two shapes are `schedule every month …` and `schedule on <date>`. That is
reuse rather than a new mechanism, and it settles a question §4 left vague by
saying only "after streams and states are known":

    waterfall opco.exit on entity asset.target {
      schedule on 2021-01
      from state.exit_equity
      ...
    }

Three consequences.

**Ordering within a period is now explicit.** Streams and states resolve, the
period's free cash flow is known, and *then* waterfalls run in declaration
order. A waterfall never feeds a stream in the same period, which is what keeps
it out of the dependency graph — the boundary `docs/14` §5 drew.

**A periodic waterfall and an exit waterfall can coexist in one model**, which
is the normal case: a deal sweeps cash to lenders every period and splits the
residual to equity once at exit. Two declarations on two schedules, no special
case.

**An end-of-hold waterfall is where the cumulative targets live.** A preferred
return, a catch-up and a carry split are evaluated against cash flows *since
inception*, not against this period's. That is why the root-finding targets in
§9 cluster on the once-at-exit shape, and it means the primitive needs access to
a cumulative series, not just the current period.

## 11. Revised sequence

1. ~~Settle §5 question 4~~ — **done, §12**. No mutation; no solver.
2. Fix the primitive's shape against **four** structures, not one: the ABS deal
   (ordering and caps), a fund carry tier (cumulative root-find target), a
   nested split (composition), and an exit waterfall (once-at-end schedule,
   cumulative targets). If one primitive expresses all four, it is probably
   right. `benchmarks/opco/lbo_option_pool_exit` is the fourth, already written
   by hand, so it doubles as the test that the construct earns its place.
3. Implement, with the ABS deal as the expressiveness fixture.
4. Pack templates on top, each with a test that it lowers to the primitive and
   produces what the hand-written steps produce.

## 12. Question 4, settled — and the solver question

### Question 4 does not require mutation

Steps 7, 10, 13 and 16 read "to the extent necessary, **after giving effect to
any payments made under clauses 4, 5 …**, to reduce the combined principal
balance of the Class A and Class B Notes to the pool balance".

Written out, step 7's payment is:

    min(remaining, max(0, (A_open − p4 − p5) + B_open − pool_balance))

where `p4` and `p5` are the amounts steps 4 and 5 paid. Those are known by the
time step 7 evaluates, because the waterfall is ordered. So the clause is a
**read of prior steps' payments**, not a mutation of anything a stream can see.

Two implementations are available and they produce the same number: carry the
prior payments and subtract them, or have the waterfall keep a running ledger of
each payee's balance. The ledger reads better for an author, and it is internal
to the waterfall's own evaluation either way.

**So there is no departure from the stream model.** A waterfall reads
period-close state, allocates a pot, and emits streams. Nothing it does is
visible to a stream in the same period, and `docs/14` §5's boundary holds
unchanged. Question 4 is closed, and questions 2 and 3 no longer depend on a
contested answer.

The remaining design choice is presentational: whether the surface exposes
`paid(step_name)` or a running `balance_of(payee)`. Both are expressible; the
ledger is the recommendation, because every one of these clauses is written
about a balance rather than about a payment.

### No solver — and the reason is not the one I expected

The tempting conclusion from §9 was that cumulative tiers need root-finding,
because IRR is a root of a polynomial in the cash flows. Worked through, every
tier is **linear in the payment**:

| tier | solve for the payment X | |
|---|---|---|
| return of capital | `X = min(pot, unreturned_balance)` | a balance |
| compounded preferred | `X = min(pot, accrued_pref_balance)` | an accrual, i.e. state |
| GP catch-up to 20% of profit | `X / (pref + X) = 0.20` → `X = pref / 4` | closed form |
| 80/20 split | `X = 0.20 × remaining` | arithmetic |
| MOIC ratchet threshold | `(D + X) / C = m` → `X = m·C − D` | closed form |
| IRR hurdle | see below | closed form |

**An IRR hurdle is not an IRR calculation.** The tier says *pay until the LP has
achieved an 8% IRR*. The hurdle rate is **given**, so nothing solves for a rate.
What is wanted is the payment that makes the present value of all distributions,
discounted at that known rate, equal the capital contributed — and present value
is linear in a payment at a fixed rate. One division:

    X = (C − PV(distributions so far at h)) × (1 + h)^t

Checked: capital 100, an 8% hurdle, distributions of 10, 15 and 20 at years 1–3,
and a final distribution at year 5 gives `X = 91.104238`, at which the present
value of every flow at 8% is exactly 100.

The engine already carries a bisection solver for `model.irr`, and that is the
right tool for its job — *reporting* an IRR when the rate is the unknown. It is
not needed here, where the rate is an input.

### Where a search does remain, and what to use

One roadmap item is genuinely a search: the tax-equity **yield flip**, where the
flip date is whenever the investor's after-tax return crosses a target. That is
a search over **dates**, which are discrete and ordered — the same shape as the
option ladder in `benchmarks/opco/lbo_option_pool_exit`, which enumerates over
prefixes and needs no solver either.

If a continuous root find is ever unavoidable, it should be **bisection or
Brent, not Newton-Raphson**, and the reason is this language's central promise.
Newton needs a derivative, can diverge or oscillate on a polynomial with several
sign changes, and its iterate path depends on floating-point detail — so the
same model could converge differently on two machines. Byte-reproducibility is
the property everything else here is built on. A bracketed method converges
deterministically or fails deterministically, which is the behaviour to want,
and it is what `irr_with_offsets` already does with a fixed bracket and a fixed
iteration count.

**Recommendation: build no solver for the waterfall.** Closed forms cover every
tier catalogued. If one is later found that does not reduce, add a bracketed
search with a declared tolerance and a named failure, and make it visible in the
syntax so a reader can see that a model is paying for iteration.