# Intex parity — the real items

Status: informative, 2026-09-01. Not published; repository-only, like the
backlog. Promoted from `docs/13` §7.74, which remains the backlog anchor and
now points here.

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

- **Coupled interest/principal waterfalls.** Interest diverted into principal
  redemption on a trigger failure crosses two waterfalls, and one pot does
  not express it (`docs/17` §5, question 2 — "two declarations with an
  explicit cross-link step, or a named-pot construct. **Unresolved.**"). The
  account and the walk are the machinery an answer would use, but the answer
  is not designed.
- **A step's shortfall as a published series** (`docs/17` §5, question 3).
  It is the thing an analyst reads first, and today it must be derived by
  differencing — which `docs/20` §3.2 showed is also an assertion hole: a
  clamped step and a satisfied one are indistinguishable from the residual.
- **Deferred/PIK interest on an unpaid step** (`docs/17` §5, question 1 —
  "probably a second form, not a default"): the unpaid amount accrues as a
  balance rather than vanishing.

**What forced the discovery:** the AmeriCredit waterfall work (`docs/17`) and
the §7.74 survey. The 22-clause deal happened not to need any of the three;
the next tier of deals (CLO OC/IC diversion, CMBS appraisal reduction) is
built from them.

**The shape:** open — question 2 has two candidate forms and no decision.
Backlog: `docs/13` §7.74 (this document's anchor); design home `docs/17` §5.

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

**What could not be expressed:** nothing, probably — the item is that P&I
advancing and stop-advance appear nowhere in the docs or the suite. Under the
machine they are a state pair (`advancing`, `stopped`) with streams gated on
it and a recoverable-advances account; `docs/30` §1 already names the
recoverable-advances balance as one of the reserves every domain has under a
different name. The item is naming that shape in the credit pack and shipping
a case, not new machinery. Backlog: `docs/13` §7.74.

## Item 4 — the clean-up call, exercised

**What could not be expressed:** nothing — the gap is a missing case. The
pack-lifecycle review retired `called` as a state ("a clean-up call is an
occurrence, not a condition a pool sits in," `docs/36` §2.2, landed in
`packs/credit/ontology/types.toml` — the pool machine is now `warehouse`,
`revolving`, `amortizing`, `rapid_amortization`, `retired`), so the election
is an event driving `amortizing -> retired` whose guard reads pool factor —
expressible today. No shipped case exercises it. A benchmark deal with a call
is the ask, not a construct. Backlog: `docs/13` §7.74.

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
priced exception plus a PV expression, not a new solver. Backlog:
`docs/13` §7.74; adjacent: §7.4 (the discount curve, `docs/33` Item 2).

## Item 6 — per-period stochastic draws

**What could not be expressed:** a rate path. `assume ~ Dist` draws one
scalar per trial; a path is a field recurrence whose innovation must differ
per period, and there is no per-period draw. **The shape:** a per-period draw
stream, seeded per (assumption, period, trial) the way per-assumption streams
are seeded today — additive, journaled, replayable. Correlation stays
excluded (`docs/01` §1.1.10) until a document forces it; a rate-dependent CPR
is a recurrence reading the rate path and needs no correlation construct.
Backlog: `docs/13` §7.74.

## Item 7 — the output surface an analyst reads

Two published-figure classes still cannot be asserted, and both are recorded
at length elsewhere; referenced, not duplicated.

- **Per-class WAL** (`docs/13` §7.22; `docs/20` §3.1): 709 published lives
  for one deal and seven for another, all reproduced, none assertable —
  a per-class WAL needs a payment stream paired with the class's original
  balance, a fold keyed to an entity rather than to a stream pattern.
- **The settlement axis** (`docs/13` §7.26): time-weighted metrics measure
  from model start on period fractions; a prospectus WAL measures from
  settlement to stated payment days, and the 400% PSA column of FNMA 2019-2
  is the falsifying case.
- **The speed grid** (`docs/13` §7.23; `docs/20` §2.3): a published decrement
  table is five to seven speeds per class, and a case can assert one.

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
shortfall accrual wearing a class-type name. Backlog: `docs/13` §7.74;
`docs/20` §2.4.

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
room. Backlog: `docs/13` §7.74.

## Item 11 — loan-level scale, measured

**Undemonstrated, not disproven.** Four loans tie to the single-pool model at
0.0 over 372 periods (`benchmarks/credit/mbs_pool_by_loan`, `docs/13` §2.2);
43 sub-pool entities run in the auto-ABS cases
(`benchmarks/credit/auto_abs_wal`) — the largest entity count measured.
Nothing has run thousands of entities, and the per-(stream, period)
environment rebuild (`docs/29` §2.3, whose performance half was deliberately
not built when the measurement said the correctness half sufficed) is the
known hot spot to profile first. The ask is a measurement, then the fix if
the measurement demands one. Backlog: `docs/13` §7.74.

---

## Non-items, recorded so they are not rediscovered

| candidate gap | resolution |
|---|---|
| Same-period circular conventions (a fee on an ending balance that includes the fee) | out on purpose — spreadsheet artifacts, not indenture mechanics; priorities are ordered and the causal plane's refusal to iterate is the guarantee, not the gap (`docs/13` §7.74, closing paragraph; `docs/28` §4) |
| Reserve accounts, trapped cash, interest on a funded balance | shipped — `docs/28` §5.1, `docs/13` §7.76, `benchmarks/credit/americredit_2017_1` |
| An event that recurs, an action on arrival | shipped — `docs/13` §7.79, `docs/34` |
| Deterministic scenario grids (the category's dominant workflow) | scenarios plus curves plus options, today (`docs/13` §7.74) |
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
