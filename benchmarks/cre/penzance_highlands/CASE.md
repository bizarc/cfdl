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

The distribution is **once at the end**, at the final condominium closing in
2024-06. A development JV does not distribute while the deal is live, so cash
accumulates from inception and the preferred return and the capital are
cumulative balances the venture carries.

What is allocated is the venture's whole cash position: the equity the partners
contributed, plus everything the deal earned on it, less every cost. Each
partner's capital and each partner's distributions are tracked separately,
which is what makes a per-partner return measurable. The preference accrues
from construction start, not from the 2011 land purchase — compounding the land
for 12.75 years consumes the entire promote.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Declared | 4 curves, 8 entities, 28 streams, 3 accounts, 1 waterfall, 7 tiers, 4 metrics, 8 field recurrences |
| What the deal requires | carrying a balance across periods, a recorded schedule of closings, an ordered priority of payments, cash that accumulates between distributions, a return measured per partner, where in the period the cash falls |
| Conventions | equity-first funding, capitalized construction interest, a facility retired out of disposal proceeds, sale in lease-up |

The facility carries five balances — equity funded, interest, draw, repayment
and the outstanding balance — each advancing from the month before it, so every
month resolves from one already settled. It is built directly rather than from
the pack's construction loan, and the behavior is the same.

Every cost schedule states a value for **every** month, including the quiet
ones, so a run of zeros is declared rather than inferred.

## The result

The facility figures tie to the workbook exactly, and the deal's return agrees
to the sixth decimal:

| | |
|---|---|
| Peak debt | 370,411,950.94 |
| Peak equity | 186,245,280.59 |
| Capitalized interest | 48,448,594.10 |
| Lifetime net cash, levered | 196,361,512.48 |
| Return over the hold | 11.0161% |
| Multiple on invested capital | 2.04664 |

The distribution allocates 382,606,793.06 across seven tiers, leaving nothing
behind, and each partner's own record then answers what that partner earned:

| | contributed | distributed | MoIC | IRR |
|---|---|---|---|---|
| Baupost (90%) | 167,620,752.53 | 328,472,611.96 | 1.959618 | 8.1203% |
| Penzance (10%, promote included) | 18,624,528.06 | 54,134,181.10 | 2.906607 | 12.7552% |

Penzance's figure is all-in: its 10% investor share and the 17,637,224.21
promote together. Each tier is reported on its own and asserted per period, so
the promote can be read separately from the investor return.

Interest is stated **gross** — an interest outflow against a matching funding
draw — rather than folded into the balance. The two net to zero in cash and the
balance grows by the accrual either way, but the published debt service then
carries the real interest.

The payoff is reported as reversion rather than as debt principal. It is retired
entirely out of sale and condominium proceeds, and treating a $394M repayment as
debt service would make every coverage ratio in the disposal period meaningless.
The pack treats a permanent loan's balloon the same way.

## The delta

**No present value is asserted.** The deal is financed, so a present value here
is a levered one and would need a cost of equity rather than a project rate —
the unlevered figure at 10% is −171,050 while the financing contributes
+9,229,459, so almost all of a reported figure would be that artifact. And no
source document for this deal states a discount rate: the site plan record and
both guidebooks publish *capitalization* rates, which value a stabilized year
rather than a stream. Run across rates, the present value swings from
+84,206,257 at 4.46% to −46,033,231 at 20% while the return and the multiple do
not move at all, both being solved from the cash flows. The other financed CRE
cases assert no present value for the same reason.

**Cash falls at the open of each period.** Every recurring schedule here is
expense-like — construction capex, operating revenue and opex, funding draws,
condominium closings — and each is placed at the period's open. It is also what
makes the case tie: placed at the close instead, the deal returns 11.1454%
against the workbook's 11.0161%. The totals are identical either way, because a
sum does not care where inside a period its cash sits, which is why the
per-period figures are asserted alongside the lifetime ones.

**Both towers sold in lease-up.** Delivery is mid-2021 and the recorded exits
are May 2022, so on any plausible pace neither tower had stabilized. The
model carries the ramp rather than a stabilized year.
