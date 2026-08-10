# A tolled highway concession — no pack, and a subsidy the model solves for itself

## The case

A 125 km, 2x2-lane tolled highway, built over four years and operated for
forty-six more under a fifty-year concession. Ten thousand vehicles a day use
it at opening, split evenly between two categories paying 0.13 and 0.25 USD per
vehicle per kilometre, and traffic grows 3% a year for the life of the deal.

Almost every mechanic in it is a phase change. Construction draws on three debt
tranches at once — 80% at 4.0% over twenty years, 10% at 4.5% over fifteen, 10%
at 5.0% over ten — with interest capitalising into each balance rather than
being paid, and each tranche's grace period ending in a different year, so the
first tranche starts repaying in 2014 and the other two in 2015. Operating cost
is a regressive scale: the first ten thousand vehicles a day cost nothing to
serve, the next ten thousand cost 0.60 each, the next 0.30, everything above
0.15 — and traffic crosses two of those thresholds before the concession ends.
Corporate tax is levied on the smaller of the year's profit and the profit
accumulated to date, and paid a year late.

And the road does not pay for itself. The contracting authority tops it up each
year with an availability subsidy sized to hold the annual debt service cover
ratio at exactly 1.30x. It pays 21.7m in 2014, rises to 64.9m by 2017, then
falls away as traffic growth outruns the fixed costs and the two short tranches
retire — and stops entirely after 2023, five years before the last tranche is
repaid.

## The reference

The World Bank and PPIAF's *Numerical Model for Financial Simulation of Highway
PPP Projects*, run at the case-study defaults that ship inside it. The workbook
is the toolkit's own teaching model for exactly this deal, and it carries a
complete set of cached values: a fifty-year cash flow waterfall, income
statement, three per-tranche repayment schedules, a funding-during-construction
table and a results sheet. Every figure asserted here is one of those cached
values, so the comparison is period by period rather than against a single
answer.

**Not vendored.** The workbook and the user guide are freely downloadable from
the toolkit, but neither carries an explicit reuse grant, so neither is
committed here. They were fetched once outside the repository and only their
output numbers were carried across. See SOURCE.md.

## What it exercises

| | |
|---|---|
| Pack | **none** — written from the bare language |
| Declared | five entities, nine declared fields, twenty-one native streams |
| Language features | declared state with `init`/`next`, cross-field `prev` reads, a state that snapshots and then holds, `min`/`max`/`pow` |
| Conventions | mid-year drawdown with capitalised interest, constant P+I annuities off three different grace periods, VAT stripped from an inclusive toll, tax in arrears with loss carryforward, a regressive cost scale, an ADSCR-targeted subsidy |

**This is the first case in the suite with no pack**, and that is half the
point of it. A toll road is none of the four: it has no generation and no
offtaker, so `energy` does not describe it; no rent roll, so `cre` does not; no
pool of obligors, so `credit` does not; and its revenue is a traffic count
times a distance times a tariff rather than a margin on sales, so `opco` does
not. Every other benchmark demonstrates that a pack works. This one
demonstrates something no pack-based case can: that the language underneath is
enough to build an asset class nobody has written a pack for.

**The ADSCR-targeted subsidy needs no solver.** The reference computes the
subsidy as an output — the amount that makes cover come out at 1.30x — and read
naively that is a fixed point, because the subsidy sits inside cash available
for debt service, cash available for debt service is net of corporate tax, and
tax is charged on a profit that includes the subsidy. It is not circular,
because tax is paid one year in arrears:

    subsidy(t)  = max(0, 1.30 * debt_service(t) - (revenue(t) - opex(t) - tax_paid(t)))
    tax_paid(t) = 30% * min(pbt(t-1), cumulative_pbt(t-1))

Everything on the right is finished before period *t* is evaluated, so the
subsidy falls out arithmetically once a period. This is the same move as the
tax equity flip, where an IRR hurdle became a discounted running sum: the
circularity is in how the spreadsheet is wired, not in the deal.

## The result

**Exact.** Twenty-five series across fifty-one periods and seven financing-plan
totals reproduce the workbook's cached values.

| | model | reference |
|---|---:|---:|
| total uses / sources | 796.229877 | 796.229877 |
| 1st tranche at financial close | 577.459550 | 577.459550 |
| 2nd tranche | 72.852262 | 72.852262 |
| 3rd tranche | 73.526569 | 73.526569 |
| 1st tranche annuity (P+I) | 51.937347 | 51.937347 |
| subsidy, 2014 | 21.697430 | 21.697430 |
| subsidy, nominal, whole concession | 351.951289 | 351.951289 |
| ADSCR, 2013 (unsubsidised) | 1.769033 | 1.769033 |

Asserted: the works, equity, fee and per-tranche drawdown lines through
construction; all three tranche balances across all fifty years; per-tranche
interest and principal; both toll revenue lines; five operating cost lines; the
subsidy; corporate tax; profit before tax; and the depreciable capital base —
1,322 figures in total.

## The delta

The declared state agrees to **2.7e-12** — machine epsilon over a fifty-year
recursion. The cash streams agree to **8.9e-7**, which is not a modelling
difference: the results file publishes stream amounts rounded to six decimal
places, and these are USD millions, so 8.9e-7 is fifty cents on figures in the
hundreds of millions. The per-period tolerance is set at 1e-5 to sit just above
that rounding floor.

One thing the case does **not** assert is the reference's equity IRR, project
IRR and NPV. Those need the dividend policy — distributable reserves are the
lesser of the cash balance and cumulated retained profit — and a balance sheet
to carry cash between years, neither of which is modelled here. The spine that
determines them is: revenue, cost, tax, subsidy and all three debt schedules
are all asserted, so anything downstream would be arithmetic on numbers that
already agree.
