# Intex parity — the real items

Status: informative, 2026-09-01; sole home since 11 September 2026. Not
published; repository-only, like the backlog. Promoted from `docs/13` §7.74,
which was then kept as a backlog anchor pointing here while this document
pointed back at it — so neither was the home. The anchor is closed and THIS
document is where these items live. Each item's own section below is its
citation; there is no backlog entry to keep in sync with it.

Intex is the reference engine for structured-finance cash flow projection —
the Intex/Trepp category: collateral pools feeding tranche waterfalls with
triggers and reserve accounts, plus bond analytics over the result. This
document records what separates CFDL from that scope **at the level of
modeling mechanics** — the language and engine, not the application. The
deal-library coverage (every CUSIP, modeled and maintained), the terminal UI
and the data subscriptions are out of scope by decision: the agent substrate
(`docs/32`) and surfaces built on the results contract are the answer to
those, and they consume the language as it is.

Every claim about CFDL below was verified by probing the current build,
reading the pack sources, or citing a benchmark — not by reading feature
lists. The Intex side is domain knowledge; no benchmark reconciles against an
Intex run (see "The benchmark this document wants," below).

---

## What is already at parity or ahead

Recorded so the items below are read at their true size. The collateral side
is the larger half of the category's work, and it is largely done; the open
items concentrate in liability-side mechanics and analytics.

- **Collateral runs to published-schedule parity.** CPR prepayments, CDR
  defaults, loss severity and a recovery lag; PSA, SDA and ABS ramps indexed
  from origination with `age_months` seasoning; selectable day count
  (`30/360`, `30e/360`, `act/360`, `act/365`), with amortization allowed its
  own basis; and cross-grain agreement to the cent — the same pool on a
  39-period monthly grid and an 1186-period daily book (`packs/credit/README.md`,
  each convention with the error it prevents quantified).
- **Sequential-pay tranching runs as an ordered waterfall** —
  `benchmarks/credit/auto_abs_tranches`, per-class principal columns off a
  43-sub-pool collateral exhibit (`auto_abs_wal` reconciles the collateral
  itself, all 43 balances exact).
- **The intricate liability structures run too.** AmeriCredit 2017-1: all
  twenty-two priority clauses written out — parity steps, the
  overcollateralization-target turbo, the step-down release, and clause 19's
  reserve as an `account` funded at closing with the top-up as its own step —
  reconciled against a reference that reproduces the prospectus
  percent-outstanding tables at all four published ABS speeds
  (`benchmarks/credit/americredit_2017_1`; the model asserts the 1.50%
  column, and §2.3 of `docs/20` is the grid gap that leaves the other three
  to the reference).
- **Loan-level and pool-level agree.** The same pool as one contract and as
  four loans belonging to a pool that holds no contract, every pool figure an
  aggregate (`benchmarks/credit/mbs_pool_by_loan`).
- **The REMIC family reconciles at six PSA speeds** —
  `benchmarks/credit/fnma_remic_2019_2_g3` and its `psa000` through `psa1000`
  variants, a coupon-strip identity asserted through the interest legs after
  mutation testing showed the residual-only form was one-sided
  (`docs/20` §3.2). The class-type ground covered across the two REMIC deals
  read for the suite — PAC, TAC, SEQ, AD, Z, PT, NTL — is recorded in
  `docs/20` §2.4, alongside what is not (see Item 8).
- **Logic reads settled cash, strictly backward** (`docs/28` §4–§5, shipped):
  triggers test realized collections, not scheduled ones, and the refusal to
  iterate within a period is a guarantee, not a limitation (see the
  non-items).
- **Reserve mechanics are core** — fund to target, top up, release, trapped
  cash across a failed test, interest on the prior balance
  (`docs/13` §7.76; `fixtures/valid/reserve_interest_on_balance`).
- **Ahead of Intex:** the journal as a causal audit trail — every trigger
  test and every clamped step traceable to the entries that produced it; text
  source under version control; byte-comparable deterministic runs;
  per-assumption Monte Carlo, with a p01–p99 distribution for every declared
  metric since §7.87 shipped; and model-declared metrics and statements
  (`docs/13` §7.25, §7.55), so the output surface is versioned with the deal
  rather than configured in a terminal.

---

## Item 1 — the waterfall mechanics `docs/17` §5 left open

**What could not be expressed:** three related things, all
write-up-from-the-bottom mechanics that CMBS and CLO documents assume.

- ~~**Coupled interest/principal waterfalls.**~~ **EXPRESSIBLE TODAY, probed
  14 September 2026.** This bullet said one pot does not express a trigger
  diversion and the answer "is not designed". The answer was designed
  afterwards and this row was never revisited: the ACCOUNT closed it. An
  account carries a `from` expression, a waterfall draws `from <account>`, and
  a step may pay TO an account — so the interest ladder's diversion step credits
  the principal ladder's source account, and the principal waterfall sees it in
  the SAME period. Two pots, two ladders, one cross-link, no new construct:

      account interest_collections  { from series_sum("coll.int",  time.t, time.t) }
      account principal_collections { from series_sum("coll.prin", time.t, time.t) }

      waterfall trust.interest on entity legal.trust {
        schedule every month from 2026-01 to 2026-04
        from interest_collections
        pay senior_int to legal.classA = 400
        pay oc_divert  to account principal_collections = if(<oc fails>, remaining, 0)
        pay residual   to legal.equity = remaining
      }
      waterfall trust.principal on entity legal.trust {
        schedule every month from 2026-01 to 2026-04
        from principal_collections
        pay classA_prin to legal.classA = remaining
      }

  Class A principal pays 5,000 in a passing period and 5,600 in a failing one,
  the 600 arriving from the interest ladder that period. `auto_abs_tranches`
  already carries the two-account half of this shape. What remains is a pack
  spelling and a case, not a language question.
- **A step's shortfall cannot LEAVE the waterfall.** The earlier statement —
  "a step's shortfall as a published series" — understates it, and the entry's
  own suggestion of deriving it by differencing does not work. Probed:

  1. The shortfall is exactly what its name says: a claim meeting available
     cash. Senior claims 700 and sub claims 300 against 600 of collections;
     senior takes 600, sub takes 0, and the sub's 300 shortfall exists.
     `owed.sub_coupon - paid.sub_coupon` computes it correctly INSIDE the
     waterfall.
  2. A STEP cannot record it. A step moves cash and is clamped by `remaining`,
     and a shortfall exists precisely when `remaining` is zero — so
     `pay deficiency to account pik = owed.x - paid.x` pays 0.0 in every
     period. Not a bug: a step is a cash instrument and a shortfall is the
     absence of cash.
  3. A NON-CASH ACCRUAL is the right instrument and cannot read the step.
     `E5031_UNRESOLVED_NAME` at run time — loudly, which is §7.97's fix
     working.

  So the item is: **a waterfall has no non-cash exit.** One sentence, and it is
  the root cause of PIK below and of the available-funds cap in Item 8.
- **Deferred/PIK interest on an unpaid step** (`docs/17` §5, question 1 —
  "probably a second form, not a default"): the unpaid amount accrues as a
  balance rather than vanishing. It is the bullet above wearing a name — PIK
  IS a shortfall reaching an account across periods.

**A narrower repair than a new construct, and the one to try first.** A stream
reading a step one period BACK is refused by `E1346_STREAM_READS_WATERFALL_STEP`,
whose stated reason is that "steps publish when their waterfall finishes, and
every waterfall runs after the causal plane — so this read could only ever
aggregate to zero". That is true of a same-period read and false of a backward
one: at `time.t - 1` the waterfall has finished and the period is closed. The
engine already proves a window backward — numeric literals, `time.t` plus a
chain of signed literals, `max`/`min`/`if` over those (`docs/10`, priced
amounts) — so the machinery to narrow this check exists and is not applied to
it. Narrowing `E1346` to same-period-or-unprovable would make the lagged
shortfall, PIK and the available-funds cap expressible with no new construct.
A same-period non-cash exit is the larger question and can stay open.

**What forced the discovery:** the AmeriCredit waterfall work (`docs/17`) and
the survey this document carries. The 22-clause deal happened not to need any of the three;
the next tier of deals (CLO OC/IC diversion, CMBS appraisal reduction) is
built from them.

**The shape:** question 2 is CLOSED by the account (see the first bullet);
`docs/17` §5 should be read with that correction. What is open is the non-cash
exit, with narrowing `E1346` as the cheap half. Design home: `docs/17` §5.

## Item 2 — the trigger that fails and cures, benchmarked

**The construct shipped; the credit case did not.** An OC/IC test that fails
and cures was a bare field flipping both ways until the machine could act;
§7.79 closed that (events fire per rising edge, states carry `on enter`
actions), and §7.77's fixture runs the whole covenant — breach, trap,
accumulate, two consecutive good periods, release
(`fixtures/valid/dscr_cash_trap_cure_period`, cure counter reset by
`on enter`). What remains is the same remainder as §7.77's: a fixture
asserted against its own engine is the suite marking its own homework
(`docs/20` §5.1). The ask is a shipped deal whose OC/IC trigger breaches and
cures against published figures — a case-authoring item with a sourcing
problem, not a language gap. Backlog: `docs/13` §7.77, `docs/20` §5.1.

## Item 3 — servicer advances

**What could not be expressed:** nothing — the item is that P&I advancing and
stop-advance appear nowhere in the docs or the suite.

**Two corrections to how this was written, 14 September 2026.** The shape given
here was a deal-level state pair (`advancing`, `stopped`). That is wrong about
what carries the state: the servicer advances on a PARTICULAR DELINQUENT LOAN
and stops when THAT loan's advance is deemed non-recoverable. The state belongs
to the obligation on a loan, not to the deal and certainly not to a security —
a deal-level pair is a pool approximation of a loan-level decision, and it is
wrong on any deal where some loans are advanced and others are not.

And the reason to care was overstated. For a GUARANTEED security — Ginnie Mae,
where the issuer is on the hook — the certificate holder is paid on schedule
whatever the collateral did, so advances change neither the security's cash nor
its value. They are the issuer's economics, and they reach the SECURITY only
through behaviour: an issuer carrying advances has an incentive to buy the loan
out, which moves prepayment. For private-label paper with no guarantee,
advances do change the timing the certificates see, and that is the case worth
modelling.

`docs/30` §1 already names the recoverable-advances balance as one of the
reserves every domain has under a different name. The item is naming that shape
— per loan — in the credit pack and shipping a case, not new machinery.

## Item 4 — the clean-up call, exercised. **SHIPPED.**

**What could not be expressed:** nothing — the gap was a missing case, and the
case now exists. The pack-lifecycle review retired `called` as a state ("a
clean-up call is an occurrence, not a condition a pool sits in," `docs/36`
§2.2, landed in `packs/credit/ontology/types.toml` — the pool machine is now
`warehouse`, `revolving`, `amortizing`, `rapid_amortization`, `retired`), so
the election is an occurrence and not a condition.

`benchmarks/credit/americredit_2017_1` exercises it (#308, `docs/40` stage 7):
`Credit.Contract.CleanUpCall` written on the trust and held by the servicer,
`call_threshold = 0.10` against the cutoff balance, `exercise when
prev.balance <= contract.call_threshold * contract.initial_balance` reading
the trust's own fold as its claim, and each loan's machine writing the balance
off on `repurchased` (`docs/42` §3.5). Every cell of the deal is unchanged by
it.

This row previously read "no shipped case exercises it" for a week after one
did — the hazard of a status line in a survey document, which is the same
failure `docs/26` records for `docs/40`'s stage header.

## Item 5 — valuation solvers and the make-whole

**What could not be expressed:** yield from price, price from yield, and
discount margin — the bond-analytics layer of the category. `model.irr` is
the shipped precedent: a bracketed bisection over the completed projection,
deterministic and replayable. These are the same computation with a different
objective, and they belong in the valuation plane as declared metrics
(`docs/13` §7.25, shipped — the construct they ride on), bracketed bisection
or Brent per `docs/17` §12 — never in the causal core, where a solver would
cost provenance and replay.

The **make-whole** is the one causal cash amount in this cluster: its size is
a discounting computation, and the priced exception of `docs/28` §7 is the
sanctioned mechanism, as with the direct-cap reversion — the primitive is the
priced exception plus a PV expression, not a new solver. Adjacent:
`docs/13` §7.4 (the discount curve, `docs/33` Item 2).

## Item 6 — per-period stochastic draws

**What could not be expressed:** a rate path. `assume ~ Dist` draws one
scalar per trial; a path is a field recurrence whose innovation must differ
per period, and there is no per-period draw. **The shape:** a per-period draw
stream, seeded per (assumption, period, trial) the way per-assumption streams
are seeded today — additive, journaled, replayable. Correlation stays
excluded (`docs/01` §1.1.10) until a document forces it; a rate-dependent CPR
is a recurrence reading the rate path and needs no correlation construct.

**What it is NOT, recorded because both were proposed and neither fits.** Not a
CURVE: a curve is a deterministic table, and the ask is a value that is random
and whose next draw depends on the last. Not a QUANTILE either —
`quantile_at` / `quantile_mean` / `quantile_of` (`docs/27`) query a DECLARED
distribution by closed-form integral, deterministically, so they describe the
shape of an uncertainty without ever drawing from it; two runs give the same
number by construction, which is the property that makes them auditable. The
per-period draw is the missing third thing:

    r(t) = r(t-1) + kappa * (theta - r(t-1)) * dt + sigma * sqrt(dt) * eps(t)

where the mean reversion is a field recurrence we already have and `eps(t)` is
what we do not. Adding a DISTRIBUTION FAMILY is a separate and much smaller
change — a name in `dist_name` and an arm in `cfdl-calc` — and is not this: the
gap is the per-period draw, not the menu it is drawn from.

**Who else wants it.** Energy, for merchant price paths.
`benchmarks/energy/merchant_storage_arbitrage` is fully DETERMINISTIC today —
every `assume` is `= <value>`, no `~`, no `monte_carlo` block — and takes its
per-period variation from a price curve. That case is complete as written and
is not blocked by this. What a per-period draw would add is a different
question of the same deal: the distribution of arbitrage margin across price
PATHS, rather than the margin under one path.

## Item 7 — the output surface an analyst reads

Two published-figure classes still cannot be asserted, and both are recorded
at length elsewhere; referenced, not duplicated.

- ~~**Per-class WAL**~~ **SHIPPED** 6 September 2026: `wal(<series>)` folds
  the life of what a series paid on the model's axis, so a class's life is
  one line over the principal its step pays. The seven FNMA 2019-2 cases
  assert Class AB's published life per speed, and the auto ABS pilot asserts
  all six classes to maturity inside the exhibit's print floor.
- **The settlement axis** (`docs/13` §7.26): time-weighted metrics measure
  from model start on period fractions; a prospectus WAL measures from
  settlement to stated payment days, and the 400% PSA column of FNMA 2019-2
  is the falsifying case.
- ~~**The speed grid**~~ **SHIPPED** 6 September 2026: `expected_<scenario>.csv`
  asserts a scenario's own per-period column, so a published decrement table
  is one case with a scenario per speed (`docs/20` §2.3).

## Item 8 — class types nothing exercises

From the Fannie Mae class-type taxonomy (`docs/20` §2.4): `NAS`/`AS`
(non-accelerated and its accelerated mirror), `JMP`/`SJ`/`NSJ` (priority that
changes on a trigger — permanently, temporarily, or on compound triggers),
`CPT` (components), `SEG` (segment groups), `AFC` (available funds, shortfall
carrying over and itself accruing), `SP`/`SPS`.

`JMP`/`SJ`/`NSJ` are the priority-model question and the ones to watch: a
trigger that REORDERS a waterfall is the shape `docs/17` §5 left open, and a
deal exercising it would settle whether declaration order plus `when` is
enough or whether priority needs to be first-class. `AFC` is Item 1's
shortfall accrual wearing a class-type name. See `docs/20` §2.4.

**Worked through against the constructs, 14 September 2026 — reasoned, not
probed. A case demonstrates a claim here; it does not establish it.** `NAS` is
a step gated on elapsed periods with a step-up curve: expressible. `CPT` is one
class declared as its own notes and steps per component: expressible. `SEG` is
a separate pool entity with its own collection account and waterfall, which is
`auto_abs_tranches`' shape already: expressible. `JMP`/`SJ`/`NSJ` are
expressible by declaring the class at each position it can occupy under
complementary guards — `if(trigger, full, 0)` early, `if(trigger, 0, full)`
late — which works and costs a duplicated step per position; that duplication
IS the evidence for whether priority should be first-class, and it is available
without waiting for a deal. `AFC` is the one that does not work: the capped
coupon is `min(stated, available)` and trivial, but the CAP CARRYOVER is a
shortfall accruing to a balance — Item 1's missing non-cash exit, blocked by
the same `E1346` reading. Four expressible, one blocked, and the blocked one is
not a class-type question at all.

## Item 9 — structured collateral

An instrument whose collateral is **another instrument's output**, where the
pot is supplied rather than derived — Fannie Mae's `SC` type, present in both
REMIC deals read for the suite. The language needs nothing new: a waterfall
takes `from <expr>` and a `curve` can carry a declared principal series
(`docs/20` §2.1, with the sketch). Nothing demonstrates it, and until
something does the suite implicitly claims a model must own its collateral.
A case saying otherwise out loud is worth more than the deal it uses.
Backlog: `docs/20` §2.1.

## Item 10 — multi-currency

No mechanism has landed; the account was shaped so the currency clause is
additive (`docs/28` §5.1). Blocked on a document that needs it, not on design
room.

## Item 11 — loan-level scale, measured

**Undemonstrated, not disproven.** Four loans tie to the single-pool model at
0.0 over 372 periods (`benchmarks/credit/mbs_pool_by_loan`, `docs/13` §2.2);
43 sub-pool entities run in the auto-ABS cases
(`benchmarks/credit/auto_abs_wal`) — the largest entity count measured.
Nothing has run thousands of entities, and the per-(stream, period)
environment rebuild (`docs/29` §2.3, whose performance half was deliberately
not built when the measurement said the correctness half sufficed) is the
known hot spot to profile first. The ask is a measurement, then the fix if
the measurement demands one.

---

## Non-items, recorded so they are not rediscovered

| candidate gap | resolution |
|---|---|
| Same-period circular conventions (a fee on an ending balance that includes the fee) | out on purpose — spreadsheet artifacts, not indenture mechanics; priorities are ordered and the causal plane's refusal to iterate is the guarantee, not the gap (the non-items here; `docs/28` §4) |
| Reserve accounts, trapped cash, interest on a funded balance | shipped — `docs/28` §5.1, `docs/13` §7.76, `benchmarks/credit/americredit_2017_1` |
| An event that recurs, an action on arrival | shipped — `docs/13` §7.79, `docs/34` |
| Deterministic scenario grids (the category's dominant workflow) | scenarios plus curves plus options, today |
| Metric distributions per trial | shipped — `docs/13` §7.87, `results_version` 0.9 |

## The benchmark this document wants

An Intex tie would be the highest-value structured-credit benchmark in the
programme — it is the number the market settles on. It cannot be a public
case, and the bind is the same one this document's sibling records for Argus:
Intex output is licensed and not redistributable, and producing it needs a
subscription seat. `docs/36` §6 states the general position — such sources
"are usable as specifications to reconcile against and to cite; neither can
be vendored — the position `docs/33` already takes on Argus output." It
belongs in the **private held-out case set** that `docs/32` Phase 3 already
contemplates, alongside engagement-derived cases from `docs/31` W2.

Meanwhile the public suite keeps doing what a license cannot forbid:
reconciling against the issuer's own published tables — which is what every
credit case above already does.
