## The case

Buenavista del Cobre is an open-pit copper mine in Sonora, Mexico. It has
operated since 1899 and is among the largest copper mines in the world. Its
operator publishes a 41-year plan: what rock is moved each year, at what grade,
what it costs to move and treat, and what the resulting cash is worth.

This case does not reproduce that cash flow. It takes the operator's inputs,
states our own claims about how the asset behaves, and produces our own
statement. The operator's answer is then a comparison, and the difference is
the finding.

The claims are ordinary. A mine produces metal, which varies by year and splits
between three products. Each sells at its own price. Some costs scale with rock
moved, some with rock milled, some with metal sold, and some are fixed. What is
left is EBITDA. Depreciation, a mining duty, an employee profit share and
income tax take EBITDA to net income.

## The reference

The inputs come from the S-K 1300 Technical Report Summary for the mine,
prepared by WSP USA for Southern Copper Corporation, dated 11 February 2025 and
filed as Exhibit 96.6 to the FY2024 Form 10-K. The 41-year production schedule
is Table 13.3, transcribed as `published_production_schedule.csv`. Unit costs
are section 18, prices and the discount rate are section 19.1, and the fiscal
rates are section 19.2.

The comparison target is Table 19.1, the operator's own discounted cash flow,
transcribed as `published_grid.csv`. No part of the model consults it.

An independent implementation of the same claims over the same inputs produces
the expectations this case asserts, so the check is between two implementations
rather than against the operator's answer. The comparison to that answer is
reported below and is deliberately not asserted: it moves whenever the recovery
assumptions move, and pinning it would turn a finding into a target.

## What it exercises

| | |
|---|---|
| Pack | none — written from the bare language |
| Entities | one real asset, carrying its own lifecycle and its one memory |
| Language features | streams reading the period's result through `series_sum`, open-world lifecycle events with published transitions, declared phases, a carryforward recurrence, run-config knobs driving scenarios, a two-file model |
| Conventions | duty on EBITDA, profit share on EBITDA net of depreciation and duty, income tax net of a duty credit, loss carryforward, first year undiscounted |

The second case in the suite written without a pack, after
`ppiaf_toll_highway`. A mine fits none of the four: no generation and no
offtaker, no rent roll, and no pool of obligors. Its revenue is contained metal
at a price, not a margin on sales.

**The model separates data from claims.** `inputs.cfdl` holds the published
physical drivers and nothing else — eleven curves of tonnes, grades and
capital, with a header stating that nothing in the file is a modeling choice.
`model.cfdl` holds the claims: the rates, the streams, the lifecycle and the
fiscal stack.

**EBITDA is a result, not an input.** The fiscal charges read it from the
period's realized streams through `series_sum`. Cross-stream reads are one hop
deep, so each charge derives from EBITDA in closed form rather than chaining
off another charge.

**Four recovery numbers are ours.** The report states no recovery for its cash
flow. They are run-config knobs rather than constants, so the case's declared
uncertainty is explorable rather than buried, and scenarios walk each to its
alternative published basis.

## The result

All 41 periods reproduce across sixteen columns, and three metrics reproduce,
to 1e-5 against the reference.

Against the operator's own statement, life of mine, in US$ M:

| line | ours | theirs | difference |
|---|---:|---:|---:|
| Total revenue | 79,937 | 76,951 | +3.9% |
| Total operating cost | 56,398 | 57,887 | −2.6% |
| EBITDA | 23,539 | 19,062 | +23.5% |
| Income tax | 3,105 | 2,415 | +28.6% |
| Capital | 8,317 | 8,317 | 0.0% |
| **After-tax NPV at 10%** | **3,689** | **3,405** | **+8.3%** |

Revenue lands within 4% and operating cost within 3% of a statement built by
the operator's own consultants from the same physical plan. Capital matches
exactly, because it is a published total apportioned by material moved.

## The delta

**The cost side reconstructs; the revenue side does not.** Mining, processing
and general costs all fall within a few percent of the published lines, using
nothing but published unit rates and published tonnages. That is evidence the
report discloses enough to rebuild what it costs to run this mine.

**Copper is the whole difference, and its levers are not equal.** Moving each
input across its full published range moves our after-tax NPV by:

| lever | move |
|---|---:|
| copper price, US$3.30 to the market study's US$3.87 | +2,589 |
| leach recovery, 26% to the secondary-zone chemistry of 57% | +2,592 |
| leach recovery, 26% to the mixed-zone floor of 36% | +838 |
| mill recovery, 83.6% to 78.3% | −666 |
| payability, 96.7% to 95% | −226 |
| molybdenum and zinc recovery, across their whole published ranges | under 50 |

Molybdenum and zinc do not matter. Mill recovery matters modestly. **Price and
leach recovery each move the valuation by roughly US$2.6 bn**, and both are
choices rather than measurements.

**Two unstated judgments carry the case.** The price deck of US$3.30 per pound
was, in the report's own words, "provided by SCC" — the operator — while the
Wood Mackenzie market study the same report contains averages US$3.87 over its
published years. And the leach circuit treats 35% of the contained copper at a
recovery the report never states; the soluble-species chemistry in Table 11.7
implies 36% to 57%, while the operator's economics imply materially less.

Our model is 8.3% above the operator's valuation. Set leach recovery to what
the published chemistry supports and the difference grows rather than closes.
Take the market study's own price and it grows further. So the difference is
not arithmetic. The operator's valuation rests on a price below its own market
study and a leach recovery below its own ore chemistry, and the report explains
neither.

## What the case does not claim

The 0.5% additional royalty on precious-metal receipts, confirmed in the parent
Form 10-K, is not modeled: this mine's published revenue carries only copper,
molybdenum and zinc. The market price curves are not used, because Table 16.2
runs to 2034 and Table 16.4 to 2029 against a mine that runs to 2065, and
extending them would mean inventing three decades. Working capital is not
modeled, since the stated day-counts net to zero over the life. The annual
capital programme is published only as life-of-mine totals, so it is
apportioned by material moved.
