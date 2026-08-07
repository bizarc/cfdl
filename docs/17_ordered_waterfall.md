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
4. **Does `down to` need to see other steps' effects?** Steps 7, 10, 13 and 16
   each say "after giving effect to any payments made under clauses 4, 5 …".
   That is satisfied if `measuring` reads the balance as it stands mid-waterfall
   rather than as it stood at period open — which means a waterfall mutates the
   balances it measures. That is a real departure from the stream model and is
   the single hardest question here.

Question 4 is the one to settle first. Questions 2 and 3 follow from it.

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
