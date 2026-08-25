## The case

A ground-up mixed-use development in Rosslyn, Virginia — The Highlands, by
Penzance with The Baupost Group, on Arlington County site plan **SP #445**. Land
was bought in September 2011, construction ran 39 months, and the deal exited
in three pieces: two rental towers sold on one day in May 2022 and a
condominium sold unit by unit over the 34 months to June 2024.

Two towers sit over one shared podium and carry **both for-sale and rental
product on a single construction basis**:

| | Units | Tenure | Exit |
|---|---|---|---|
| East / North Tower — Pierce | 104 | for sale | 102 recorded closings |
| East / South Tower — Evo | 455 | rental | $334,642,240 |
| West — Aubrey | 331 | rental | $266,455,000 |

Seven of the twelve parcels are **ground-leased from the County**, not owned,
and the site carries in-kind public obligations — a fire station, a public park
and a new public street — that are pure cost with no revenue.

## The reference

Public record, and an independent spreadsheet implementation built from it.
Both read the same frozen input set, so they tie by construction rather than by
transcription.

Fact, from Arlington County: the program, unit mix by tower, GFA, parking, FAR
and the quantified public obligations (site plan SP #445, 2017 approval with
2018 and 2020 amendments); the land basis of **$67,000,000** recorded
2011-09-30 and all 102 condominium closings (assessment roll); both tower sale
prices and the operating parameters — 5.15% guideline loaded cap, $7,511/unit
expenses, 8% vacancy, $150/space parking (Commercial Guidebook, 2022 and 2023).

Rents are **derived, not assumed**: each tower's 1/1/2022 assessed value times
the guideline loaded cap gives the assessor's own NOI, solved back to
$3,522/unit for Aubrey and $3,273/unit for Evo.

Assumed: construction cost, debt pricing, lease-up pace, and the JV tiers. The
Penzance/Baupost terms are private, so the tier percentages here are stated
placeholders rather than the real split.

The distribution is a **once-at-end** waterfall, at the final condominium
closing in 2024-06. A development JV does not distribute while the deal is
live, so the preferred return and the capital are cumulative balances the
venture carries, per `docs/17` §10. Two consequences follow from the pot rather
than from the deal. `available` is *this period's* netted cash, which on a
once-at-end schedule is one month rather than the deal, so the pot is
`series_sum("cre.*", 0, time.t)` — the streams' own running sum. And there is no
return-of-capital tier: contributions are outflows inside those streams, so the
running sum has already recovered the capital, and what survives to the end is
profit. The preference accrues from construction start, not from the 2011 land
purchase — compounding the land for 12.75 years consumes the entire promote.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | 4 curves, 8 entities, 28 streams, 1 waterfall, 8 field recurrences |
| Language features | entity field recurrences (`init`/`next`/`prev`), `curve` lookups, a once-at-end `waterfall` drawing a cumulative pot with `series_sum`, `part of` roll-up, `start` placement |
| Conventions | equity-first funding, capitalized construction interest, a facility retired out of disposal proceeds, sale in lease-up |

The facility is five recurrences on one entity — `equity_funded`, `interest`,
`draw`, `repay`, `balance` — each reading only `prev` and the cost curves, never
a stream, which is what keeps it acyclic. It is hand-built rather than the
pack's `cre.construction_loan` contract, and so does not use that contract's
`capitalize_interest` election; the behavior is the same, the implementation
independent.

Every cost curve declares **every** period, including the zeros. A step curve is
flat-forward, so omitting the quiet months holds the last construction draw
forward for ever and the balance never stops compounding.

## The result

The facility ties to the workbook to the cent:

| | |
|---|---|
| Peak debt | 370,411,950.94 |
| Peak equity | 186,245,280.59 |
| Capitalized interest | 48,448,594.10 |
| `model.total` (levered) | 196,361,512.48 |
| `model.irr` | 0.110161 |
| `model.moic` | 2.04664 |

Interest is stated **gross** — an outflow at `financing.interest` against a
matching draw at `financing.debt_proceeds` — rather than folded silently into
the balance. The legs net to zero in cash, so the balance grows by the accrual
either way, but `domain.cre.debt_service` then sees the real interest.

The payoff is categorized `investing.reversion`, not
`financing.debt_principal`. It is retired entirely out of sale and condominium
proceeds, and `financing.debt_principal` folds into `domain.cre.debt_service`,
where a $394M bullet makes every coverage ratio in the disposal period
meaningless. The pack says the same of a permanent loan's balloon.

## The delta

**`model.npv` is deliberately not asserted.** The model carries financing
streams, so its NPV is levered and would need a cost of equity rather than a
project rate — the unlevered PV at 10% is −171,050 while the financing streams
contribute +9,229,459, so essentially all of a reported NPV would be that
artifact. And no source document for this deal states a discount rate: the site
plan record and both guidebooks publish *capitalization* rates, which value a
stabilized year rather than a stream. Run as scenarios, NPV swings from
+84,206,257 at 4.46% to −46,033,231 at 20% while `model.irr` and `model.moic`
do not move at all, both being solved from the cash flows. `one_lincoln_street`
and `hud_home_multifamily` omit `model.npv` for the same reason.

**Placement is `start` throughout.** Every recurring schedule here is
expense-like — construction capex, operating revenue and opex, funding draws,
condominium closings — which `12_payment_timing.md` §6 places at the period's
open. It is also what makes the case tie: at the `end` default the model returns
an IRR of 11.1454% against the workbook's 11.0161%. The totals are identical
either way, because a sum does not care where inside a period its cash sits,
which is why the per-period series are asserted alongside the metrics.

**Both towers sold in lease-up.** Delivery is mid-2021 and the recorded exits
are May 2022, so on any plausible pace neither tower had stabilized. The
model carries the ramp rather than a stabilized year.
