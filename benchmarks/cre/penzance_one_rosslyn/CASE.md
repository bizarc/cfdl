## The case

One Rosslyn is a ground-up mixed-use development in Rosslyn, Virginia. Penzance
develops it with The Baupost Group. It sits on Arlington County site plan
**SP #419**, amendment SPLA24-00040, approved 2025-07-19. The County recorded
the land in November 2023. Construction runs a projected 42 months from late
2026.

Three towers stand over one five-story podium. They carry for-sale and rental
product on a single construction basis:

| | Units | Stories | Tenure |
|---|---|---|---|
| NE Tower | 73 | 23 | for sale |
| NW Tower | 311 | 27 | rental |
| S Tower | 461 | 30 | rental |

The program is 845 dwelling units. Residential gross floor area is 957,306 sq
ft and retail is 14,584 sq ft, at 9.92 FAR. The site plan allows 429 parking
spaces.

**Nobody has built this project.** The program and the land are recorded fact.
The economics are forecast.

The companion case, `penzance_highlands`, reconstructs a deal that completed
four blocks away. Same sponsor, same equity partner, same submarket. There, the
record fixes the answer. Read the two together. One case shows what a model
does when the facts constrain it. The other shows what a model does when they
do not.

## The reference

An independent Excel implementation. The reference came first. Both
implementations read the same frozen input set, so the two tie by construction
rather than by transcription. Both scenarios agree to under a dollar.

Arlington County supplies the facts. The Board Report for SP #419 gives the
program, unit mix, GFA, FAR and parking. Deed 20230100013266 records the land
basis of **$52,000,000** on 2023-11-14. Its sales code is "4-Multiple RPCs", so
the price covers all three parcels.

Public obligations total **$16,218,313**. The County ties each AHIF tranche to the first
partial certificate of occupancy for its tower, so the model pays each one on that
tower's date.

The operating guidelines are the County's own. They come from the **2026
Guidebook**, MARKET Apartment Guidelines, High-Rise 9+ (PCC 313), effective age
2010+, Metro. The loaded cap is 5.45%. Expenses are $9,466 per unit. Vacancy
and collection is 5%. Guideline land value is $78,000 per rental unit. That
figure puts the 772 rental units at $60,216,000. The sponsor paid $52,000,000,
which is 13.6% below the County's own basis.

The model derives rent rather than assumes it, by the County's method. A
comparable's assessed value times the guideline loaded cap gives the assessor's
own net operating income. Add back guideline expenses. Gross up for vacancy.
The result is $3,789 per unit per month in 2026 dollars.

Construction is anchored to the companion case's actual spend. Buildings cost
$255.00/sf of GFA and parking costs $32,000 per space. The two rates stay
separate on purpose. Highlands built 1.17 spaces per unit, and this project
builds 0.51.

The BLS producer price index for new multifamily construction escalates that
spend. Series WPUIP2312001 rose 53.4% from 2019-11 to 2026-07. It measures
**output** prices. The inputs index measures materials, and assumes contractor
margin moves with them, so it overstates.

The case assumes construction duration, debt pricing, lease-up pace,
condominium pricing, growth, and the JV tiers.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | 6 curves, 8 entities, 16 streams, 8 field recurrences, 3 accounts, 2 waterfalls, 14 tiers, 4 metrics, 1 slice, 6 scenarios, a 12-month valuation tail |
| What the deal requires | carrying a balance across periods, an ordered priority of payments, cash that accumulates between distributions, a return measured per partner, two exits in one model |
| Language features | `curve` lookups, entity field recurrences (`init`/`next`/`prev`), a scenario switch that weights streams and tiers through `inputs.*`, `pow` escalation from a stated index base, `series_sum` over a `project` tail for a forward-income valuation, `account` and `moves`, a `waterfall` paying `from` an account, `irr`/`moic` over a party, a `slice`, `part of` roll-up, `start` and `end` placement |
| Conventions | equity-first funding, capitalized construction interest, a facility retired out of disposal proceeds, permanent refinance at stabilization, sale in lease-up, pro rata return of capital and preference, one distribution per strategy |

The model carries **two exit strategies over one set of facts**. The input
`inputs.scenario_b` selects one.

- **A, merchant build.** The venture sells in lease-up, roughly ten months
  after delivery. Penzance and Baupost did this at The Highlands. Both towers
  sold on 2022-05-17 at 83% and 60% occupancy. No permanent loan existed.
- **B, build to core.** The venture stabilizes the asset. It then refinances
  into permanent debt at 60% of stabilized value over a 30-year amortization,
  and holds five years.

Both strategies run from one model, on one set of costs and one construction facility. The
two columns below differ only in when the venture sells.

**One strategy per named run.** The six runs in `run.json` are the case's
scenario set. Nothing outside it is asserted.

| run | strategy | exit | market factor | what it answers |
|---|---|---|---|---|
| `build_to_core` (deterministic) | stabilize, refinance, hold five years | 2037-06 | 1.00 | the base case: the hold |
| `merchant_build` | sell in lease-up, as at The Highlands | 2031-10 | 1.00 | the comparison |
| `core_at_2026_discount` | hold | 2037-06 | 0.884 | the hold, priced where Central Place traded |
| `core_at_2022_premium` | hold | 2037-06 | 1.321 | the hold, priced where The Highlands traded |
| `merchant_at_2026_discount` | sell | 2031-10 | 0.884 | the sale, at the discount |
| `merchant_at_2022_premium` | sell | 2031-10 | 1.321 | the sale, at the premium |

The hold is the base. The sale is kept because the same partners chose it four
blocks away, and because the two answer different questions about the same
building. The hold length is five years past stabilization and is fixed by
the generator; a longer hold is a different run, not a different input.

**The engine derives the exit rather than states it.** The model declares a 12-month
projection tail. The sale is valued on the twelve months of income that follow it,
divided by the County's guideline loaded cap and adjusted by a stated market factor.
Change the rent growth and the exit changes with it.

The factor is **1.00**, which is the guideline basis itself.

Two observations bracket that factor, and the case carries both as a range. The
Highlands sold at **+32.1% and +32.3%** to this basis in 2022. Central Place
sold at **-11.6%** in 2026. Either one as the base would import a market call
that the record does not support.

At the scenario-A exit the south tower is roughly 58% leased, so that sale is priced on
stabilized income. In-place income would understate it by about $140M.

## The result

Both scenarios tie to the workbook to under a dollar.

| | A, merchant build | B, build to core |
|---|---:|---:|
| Exit | 2031-10 | 2037-06 |
| Construction cost | 555,953,894 | 555,953,894 |
| Capitalized interest | 58,451,635 | 70,146,419 |
| Permanent loan | — | 347,217,832 |
| Exit value | 567,404,298 | 691,127,415 |
| Exit per unit | 734,980 | 895,243 |
| `slice.deal.total` | 28,591,606 | 223,799,127 |
| `slice.deal.moic` | 1.094 | 1.738 |
| `slice.deal.irr` | 2.02% | 6.57% |

Scenario A derives an exit of **$734,980 per unit**. Evo recorded **$735,385**,
within 0.1%. Same submarket, same method, comparable product. That is a
cross-check rather than a coincidence.

**The holding period is the deal.** The land, the building, the cost and the
guideline basis are identical across both scenarios. A sale in lease-up returns
1.09x. A five-year hold past stabilization returns 1.74x.

The levered figures are the deal's own cash with the partners' contributions
left out, which is what the workbook reports. The project's whole cash,
contributions included, is 473,468,010 under the hold and 278,260,488 under
the sale, and each strategy's distribution allocates exactly that.

**The split.** The partners fund pro rata, 90% Baupost and 10% Penzance, on
the dates the facility draws equity, and each contribution moves that
partner's own account.
Cash accrues to the venture and is split once per strategy: at the last
condominium closing in 2032-05 under the sale, and at the exit in 2037-06
under the hold. The split is seven tiers: capital back, an 8% preference
compounded from construction start, a 20% promote, and the residual 90/10.

Capital and the preference come back pro rata. Below the promote the partners
are pari passu, so a pot too small to pay the whole preference shorts both in
proportion.

| | A, sell in lease-up | B, hold five years |
|---|---:|---:|
| Distributed | 278,260,488 | 473,468,010 |
| Preference owed | 115,348,264 | 297,775,808 |
| Preference paid | 28,591,606 | 223,799,127 |
| Promote | 0 | 0 |
| Baupost, contributed | 224,701,994 | 224,701,994 |
| Baupost, distributed | 250,434,439 | 426,121,209 |
| Penzance, contributed | 24,966,888 | 24,966,888 |
| Penzance, distributed | 27,826,049 | 47,346,801 |
| MoIC, both partners | 1.1145 | 1.8964 |
| IRR, both partners | 2.00% | 6.22% |

**At the guideline basis, neither strategy clears the preference.** The hold
returns 6.57% on the deal's cash and the preference is 8%, so the profit is
paid out as preference and stops short of it. The promote is zero, and both
partners earn the same return, because nothing above the preference exists to
split unevenly.

The promote appears at the 2022 premium. Priced where The Highlands traded, the
hold pays a promote of 29,575,044 and the sale 19,076,024, and the sponsor's
return separates from the investor's: 13.43% against 9.64% under the hold. Each
named run pins its party figures in `expected_scenarios.json`.

## The delta

**The case does not assert `model.npv`.** The model carries financing streams,
so its NPV is a levered one. A levered NPV needs a cost of equity rather than a
project rate. No source document for this project states a discount rate, and
the rate in `run.json` is a stated placeholder. The companion case and
`one_lincoln_street` omit it for the same reason.

**The project is entitled but unbuilt.** The exit is therefore a derived value
rather than a transaction. The market has traded away from the guideline basis
in both directions inside four years: +32% in 2022, and -11.6% in 2026. Move
the market factor first. It moves the answer more than any other single input.

**Condominium pricing is the weakest input, and it carries the most weight.**
The NE tower's 73 units average 2,273 sq ft. The units that anchor them are far
smaller. No Rosslyn condominium of that size has traded recently.

**Scenario B exits at $895,243 per unit, above every recorded comparable.** The figure is 2037 dollars, after eleven years of growth. The engine derives it from the twelve months of income after the sale, over the guideline cap. It is the figure most exposed to the growth rate and the market factor.

**The JV tiers are placeholders.** The Penzance and Baupost terms are private.
The tier percentages state a structure, not the partnership's economics. The
structure is real: each partner's capital and distributions are one record, and
the return per partner is model output. Replacing the three rates is a
three-line change, and every partner figure recomputes.

**The tier and party figures are the model's own.** The workbook carries no
split, so nothing external asserts them. They are pinned as regression cover,
and `NOTES.md` recomputes every tier outside the engine from the pot and the
balances, to the cent.

**The holding period is a strategy, not a fact.** Scenario A follows the
companion deal, where both towers sold in lease-up. Scenario B holds five years
past stabilization. Nothing in the record fixes which one applies here. The two
differ by more than half a turn of equity multiple.
