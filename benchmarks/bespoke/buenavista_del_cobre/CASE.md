## The case

Buenavista del Cobre is an open-pit copper mine in Sonora, Mexico. It has
operated since 1899 and is among the largest copper mines in the world. Its
operator publishes a reserve: how much rock remains, at what grade, and where
each class of rock is processed.

A mine is a set of depleting stocks. Each period draws what its processing
capacity allows, or the remainder if less, and carries the balance forward.
Mine life is therefore a result rather than an input: the mill runs until its
stock is gone.

This case derives the mine's 41-year production plan from the reserve, values
it, and compares both against what the operator published. The comparisons are
reported. Neither drives the model.

## The reference

The inputs come from the S-K 1300 Technical Report Summary for the mine,
prepared by WSP USA for Southern Copper Corporation, dated 11 February 2025 and
filed as Exhibit 96.6 to the FY2024 Form 10-K.

About twenty numbers: four reserve tonnages and their contained metal (Table
12.8), two mill capacities, the strip ratio, the head-grade policy (Table
12.5), unit costs (section 18), and prices, the discount rate and the fiscal
rates (section 19).

Two published tables are comparisons rather than inputs. The operator's
production schedule is `published_production_schedule.csv` and its discounted
cash flow is `published_grid.csv`. The model reads neither.

An independent implementation of the same claims over the same inputs produces
the expectations this case asserts.

## What it exercises

| | |
|---|---|
| Pack | none — written from the bare language |
| Entities | four depleting stocks, each with tonnage and contained metal |
| Language features | stock recurrences, capacity limits, streams reading the period's result through `series_sum`, a stochastic assumption, a seeded Monte Carlo, run-config knobs |
| Conventions | duty on EBITDA, profit share on EBITDA net of depreciation and duty, income tax net of a duty credit, loss carryforward, first year undiscounted |

The second case in the suite written without a pack, and the first to assert a
Monte Carlo result.

**The stocks are the model.** Each of the four reserve classes is an entity
carrying two balances: tonnage and contained metal. A period draws
`min(capacity, remaining)`, and both balances fall together. The copper
concentrator steps from 74 to 43 million tonnes a year when Concentrator I is
taken offline, which is a property of that stock's processing route.

**Grade is a consequence.** Because contained metal is a stock alongside
tonnage, the grade of any period is the remaining metal over the remaining
rock. The exception is the mine's own published head-grade policy, which sets
the first three years; the balance of the reserve carries the rest of the life.
Nothing about grade is fitted.

**The strip ratio is a distribution.** Waste is what must move to reach ore.
The reserve statement pins the life-of-mine ratio at 0.83. An operating pit
strips overburden before it reaches ore and the ratio climbs as the pit
deepens, so the ratio is drawn from a triangular distribution across the range
an operating pit exhibits, with the published figure as its mode. The seed is
declared, so the draws are reproducible.

## The result

All 41 periods reproduce across fifteen columns, two metrics reproduce, and
five Monte Carlo aggregates reproduce, to 1e-5 against the reference.

**The derived plan against the operator's**, over the life of the mine:

| line | ours Mt | theirs Mt | difference |
|---|---:|---:|---:|
| copper mill feed | 2,104 | 2,104 | 0.0% |
| zinc mill feed | 287 | 287 | 0.0% |
| crushed leach | 1,077 | 1,080 | −0.3% |
| ROM leach | 1,041 | 1,039 | +0.2% |
| waste | 3,742 | 3,769 | −0.7% |
| contained copper, kt | 16,083 | 16,192 | −0.7% |

Capacity against a depleting stock reproduces both mill schedules exactly, year
for year. Mass balance holds contained copper to within a percent.

**The valuation against the operator's**, in US$ M:

| line | ours | theirs | difference |
|---|---:|---:|---:|
| total revenue | 79,527 | 76,951 | +3.3% |
| total operating cost | 59,167 | 57,887 | +2.2% |
| capital | 8,277 | 8,317 | −0.5% |
| after-tax NPV at 10% | 3,120 | 3,405 | −8.4% |

Over 500 trials on the strip ratio, the after-tax NPV has a mean of 3,132, a
median of 3,236, and a standard deviation of 820, ranging from 1,110 to 4,632.
The operator's 3,405 sits inside that range.

## The delta

**The plan derives; the pit sequence does not.** Both mill schedules are exact
because they are capacity against a stock, and both capacities are published.
The leach and waste lines match in total and not in shape: ours are smooth,
while the operator's leach tonnage swings between 8 and 154 million tonnes a
year. Nothing in the report describes the pit sequence that produces that
swing.

**That swing is worth a quarter of the valuation.** The strip ratio observed
across an operating pit spans 0.31 to 2.08. Drawn across that range, the
after-tax NPV has a standard deviation of 820 against a mean of 3,132. How much
waste moves in which year is the largest single uncertainty in this asset, and
it is larger than any question about metallurgy or price.

**Four recovery numbers are ours.** The report states no recovery for its cash
flow. Mill copper, leach copper, molybdenum and zinc recovery are declared with
their basis and are run-config knobs, so they can be moved without editing the
model. Across their full published ranges, molybdenum and zinc together move
the valuation by less than 50; leach copper moves it by up to 2,592.

**The price is the operator's.** Section 19.1 records that the deck of
US$3.30 per pound of copper was provided by the operator. The Wood Mackenzie
market study the same report contains averages US$3.87 per pound over its
published years. Moving to that study's own base case raises the valuation by
about 2,589.

## What the case does not claim

The additional royalty on precious-metal receipts is not modeled: this mine's
published revenue carries only copper, molybdenum and zinc. The market price
curves are not used, because they cover ten and five years of a 41-year life.
Working capital is not modeled, because the stated day counts net to zero over
the life. The pit sequence is not modeled, because the report does not describe
it; its effect is measured instead.
