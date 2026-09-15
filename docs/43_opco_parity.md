# OpCo parity — the real items, and what SaaS adds

Status: informative, 14 September 2026. Not published; repository-only, like
the backlog and the other three parity documents.

**The scope boundary first, because for this segment it is unusual.** Argus and
Intex are applications: a parity document can say what the product does that we
do not. Private equity and growth-equity valuation has **no dominant tool**. The
incumbent is Excel plus a house template, maintained by the team that uses it,
and differing between two firms doing the same deal. That changes what parity
means here — there is no feature list to match, so this document asks a
different question: **what does every source we have reconciled against state
that a modeller cannot say in the pack's own words?**

Every claim about CFDL below was verified by reading the pack sources or citing
a benchmark, not by reading feature lists. The comparison side is the nine
`benchmarks/opco/*` sources and `docs/41`'s agreement survey.

---

## What is already at parity or ahead

Recorded so the items below are read at their true size. The valuation spine is
done and externally checked; the gaps are operating vocabulary and financing
mechanics.

- **A banker's disclosed valuation reproduces.** `banker_dcf_conventions`
  against a sell-side DCF in a public merger filing: all nine cells of the
  answer grid within $1.2mm on $19bn. It also found two outright engine
  defects — mid-period discounting had no spelling, and `on day <n>` divided by
  a literal 30 on every calendar — both fixed.
- **Four terminal methods, each against its own source.** Exit multiple
  (`dcf_exit_multiple_nwc`), EBITDA multiple, perpetuity, and Gordon growth on
  a regulated utility (`gordon_growth_coned`), plus FCFF cross-industry
  (`damodaran_fcff`, now on `opco.reinvestment`, the derived capital line).
- **The LBO runs end to end.** `lbo_buyout` (five-year, 8.0x entry),
  `lbo_financing_cases` ($720m at 8.0x LTM), and `lbo_option_pool_exit` — an
  accruing convertible preferred and a management option pool resolved as an
  exit waterfall.
- **Circular interest needs no solver, and this is the segment's standing
  objection answered.** `lbo_circular_interest`: interest on the average
  balance is affine in the closing balance, so collecting terms solves it in
  one substitution. The reference model ships a literal `CIRC` switch and
  enables Excel's iterative calculation; we do not need one. `docs/26` records
  the general rule — a sized loan does not need a solver — with the caveat that
  it holds for the affine loops the programme has met, which is empirical
  rather than proven.
- **The most contested convention in software valuation is modelled both
  ways.** `saas_sbc_convention_fork`: stock-based compensation before and
  after, $331m against $198m of first-year free cash flow from one source on
  one page.
- **Participant-level returns are in the language.** `irr(party.<p>)` and
  `moic(party.<p>)` fold a party's own account, so a sponsor's and a
  management pool's returns are declared metrics rather than hand arithmetic
  (§7.72, `docs/31` W4 phase 2).
- **Working capital is a policy, not a plug** — `opco.working_capital` and
  `opco.working_capital_policy`, with `dcf_exit_multiple_nwc` as the check.
- **Ahead of a workbook:** the journal as a causal audit trail; text under
  version control; byte-comparable deterministic runs; scenarios; declared
  metrics and statements versioned with the deal; and per-assumption Monte
  Carlo with a p01–p99 distribution for every declared metric.

---

## Item 1 — the revolver, the cash sweep and the NOL carryforward

**What cannot be said:** three mechanics `docs/41` records as forced by *every
LBO source* it surveyed, and none is in the pack. A revolver that draws when
cash is short and repays when it is not; a cash sweep that applies excess free
cash to debt on a stated priority; and a net-operating-loss carryforward that
shelters taxable income until exhausted.

**What the language already has:** all three are balances that move — the
generalized account of `docs/42`, with `Contract.Debt` refinements over it. The
revolver is an account with a draw rule and a limit; the sweep is a priority
over free cash, which is an ordered waterfall; the NOL is a balance consumed by
taxable income, which is a field recurrence. None needs a construct.

**Why it is Item 1:** a leveraged buyout without a revolver is not a leveraged
buyout. This is the single largest vocabulary gap in the pack, and it is
vocabulary rather than capability — which is why it is cheap and why leaving it
open is expensive.

**Shape:** `opco.revolver`, `opco.cash_sweep`, `opco.nol_carryforward` as Debt
and Tax refinements. Coupled mechanics where a sweep priority interacts with a
covenant are `docs/38` Item 1's territory, not this one.

## Item 2 — a depreciation schedule the tax rule can read

**What is broken, not merely missing.** `opco.cash_taxes` reads `da_monthly`
as a bare term **no rule produces** (`docs/41`). So the tax computation depends
on a number the modeller must supply by hand, on a contract whose whole purpose
is to compute tax. Depreciation is not an agreement — it is family H, an
expense line — so the shape is a schedule the tax rule reads by name, not a
contract.

MACRS is already a function (`macrs_rate` in `cfdl-calc`), so the arithmetic
exists and the vocabulary does not.

**Shape:** `opco.depreciation` producing a D&A series, with `opco.cash_taxes`
reading it by name. Bonus depreciation and multi-class basis are the energy
pack's `docs/39` item 3 and should share whatever shape this settles.

## Item 3 — the exit struck on a forward multiple

**What cannot be said:** a sale at an NTM multiple struck at a point before
model end. `opco.exit_multiple`, `opco.exit_ebitda_multiple` and
`opco.exit_perpetuity` all value on trailing or terminal figures. `docs/41`
records the gap as forced by `banker_dcf_conventions`.

The valuation plane's forward window is the mechanism — a priced amount reading
ahead — so this is a pack rule over machinery that shipped in M1.

**Shape:** `opco.exit_forward_multiple`, beside the three that exist.

## Item 4 — the SaaS operating build, which does not exist at all

**What cannot be said:** everything a software business is actually forecast
on. Not one term of it is in the pack:

| the industry states | today |
|---|---|
| ARR / MRR as the stock, and net new ARR as the flow | a revenue line with no notion of a recurring base |
| Cohort retention — a cohort acquired in a period, decaying on its own curve | expressible as one entity per cohort with a field recurrence; no vocabulary, and the grain question is unsettled |
| Gross and net revenue retention, expansion and contraction | arithmetic over series a modeller assembles |
| Logo churn against revenue churn | no distinction |
| CAC, CAC payback, and sales-and-marketing capitalisation | an expense line |
| Seats, price and usage as separate drivers of one revenue line | a single `amount_year` |

**This is the segment's Item 1 and the only place in any parity document where
the answer is "nothing exists".** The valuation spine above is checked against
real sources; the OPERATING model under it, for the industry that currently
buys the most valuation work, has no vocabulary whatever.

**What is unsettled and should be decided before terms are written:** the
COHORT GRAIN. A cohort is naturally an entity with a decay recurrence, and
`part of` already folds a container from its members — so a hundred monthly
cohorts is expressible and may be the right spelling, or may be a hundred
entities where a curve would do. `docs/13` §7.3's "two axes" argument applies:
core-spelled first, vocabulary once a case forces the shape.

**Shape:** a case before terms. `saas_sbc_convention_fork` is a VALUATION of a
software company, not an operating model of one, so the suite has no cohort
build to reason from.

## Item 5 — covenants and leverage tests on the opco side

**What cannot be said:** a leverage covenant (net debt / EBITDA), an interest
coverage test, and the consequence of breaching one — as terms on the debt
rather than as a rule a modeller writes.

The mechanism is shipped and demonstrated on the CRE side:
`fixtures/valid/dscr_cash_trap_cure_period` runs breach, trap, accumulate, two
good periods and release, with the cure counter reset by `on enter`. What is
missing here is the same thing missing there — covenant terms on the contract
and a published breach series — so this item and `docs/33`'s covenant row
should be solved once.

## Item 6 — the three-statement articulation

**What cannot be said, and the one place a house workbook is genuinely ahead.**
A PE model is usually three statements that articulate: income statement, cash
flow statement, and a balance sheet that balances, with the check itself as the
control that the model is right.

CFDL publishes cash and declares statements over categories (§7.55), so the
income statement and the cash flow statement are expressible. A BALANCE SHEET
is different in kind: it is a position at a point, not a flow over a period,
and the balancing identity is an assertion across three statements rather than
a subtotal within one. The account (`docs/42`) is the first half of the
machinery — a balance that rolls from the lines that move it — and every asset
and liability would need to be one.

**Whether this belongs in v1 is a scope question, not a design one.** A
cash-flow language that publishes a balance sheet is a larger claim than the
project has made so far, and the audit argument cuts both ways: the balancing
check is the workbook's own control, and a language where balances are derived
from journaled movements does not need it in the same way.

---

## Non-items, recorded so they are not rediscovered

| candidate gap | resolution |
|---|---|
| An iterative solver for circular interest | not needed — affine, one substitution (`lbo_circular_interest`, `docs/26`) |
| Goal Seek for debt sizing | not needed — `docs/26`, "a sized loan does not need a solver" |
| The equity bridge (EV to equity value) | not a contract: a statement over accounts and fields (`docs/41`) |
| A diluting share count | not a contract: an entity field (`docs/41`) |
| Excel-identical float artefacts for reconciliation | shipped — `arithmetic: "excel_compat"` in the run config |
| Management incentive resolved at exit | shipped — `lbo_option_pool_exit`, `OpCo.Contract.EquityOption` |
| Comparable-company multiples as a valuation method | out of scope by the same line as Argus's report library: a comps set is data, not deal mechanics |

---

## The benchmarks this document wants

**A SaaS operating model against a published source.** An ARR build with
cohorts, retention and expansion, reconciled against a disclosed figure. This
is the one benchmark that would settle Item 4's grain question, and it is the
largest hole in the programme's coverage of the segment that buys the most
valuation work. Sourcing is the difficulty: operating builds are rarely
disclosed at cohort grain, so the realistic source is an S-1 or a public
company's disclosed ARR roll-forward rather than a banker's model.

**An LBO with a revolver and a sweep.** Item 1's demonstration, and probably
the cheapest of the three — every LBO source the survey read carries both.

**A three-statement model, if Item 6 is taken.** The balancing check is the
assertion, and it is unlike any assertion the suite currently makes.

---

## What this document is not

It is not a claim that CFDL cannot value an operating company — nine
benchmarks say otherwise, one of them against a banker's own answer grid. It is
a list of what a modeller in this segment would have to write by hand that a
CRE or credit modeller would find waiting for them in the pack. The language
gap is narrow. The vocabulary gap, for SaaS specifically, is total.
