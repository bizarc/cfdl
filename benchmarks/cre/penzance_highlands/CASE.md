# The Highlands — Rosslyn, Virginia

Penzance and The Baupost Group. Arlington County Site Plan **SP #445**.
Land acquired 2011-09-30, delivered 2021, both rental towers sold to Cortland
2022-05-17, condominium sellout completed 2024-06.

## Why this deal

Two towers over one shared podium, carrying **both for-sale and rental product
on a single construction basis**:

| | Units | Tenure | Exit |
|---|---|---|---|
| East / North Tower — **Pierce** | 104 | for sale | 102 recorded closings, 34 months |
| East / South Tower — **Evo** | 455 | rental | $334,642,240, 2022-05-17 |
| West — **Aubrey** | 331 | rental | $266,455,000, 2022-05-17 |

The costs are not separable between the condominium and the rental product,
which is the modelling problem worth having. The site also carries in-kind
public obligations — Fire Station 10, Rosslyn Highlands Park and a new public
street — that are pure cost with no revenue, and seven of the twelve parcels
are **ground-leased from the County**, not owned.

## What is fact

Program, unit counts and mix by tower, GFA, parking, FAR and heights; the
entitlement chronology; the quantified public obligations and their triggers —
all from the SP #445 record (2017 approval and the 2018 and 2020 amendments).

Land basis **$67,000,000** (recorded 2011-09-30, deed 4491/2257) and the full
**102-closing condominium sellout** (dates and prices, unit by unit) from the
Arlington assessment roll. Both tower sale prices, and the **5.15% guideline
loaded cap** for high-rise / Metro / 2010+, from the County's published
Commercial Guidebook. Operating expense of $7,511/unit, 8% vacancy and garage
parking at $150/space come from the same source.

Rents are **derived**, not assumed: each tower's 1/1/2022 assessed value times
the guideline loaded cap gives the assessor's own NOI, solved back to
$3,522/unit (Aubrey) and $3,273/unit (Evo).

## What is assumed

Construction cost, debt pricing, the lease-up pace, and the JV tiers. The
Penzance/Baupost terms are private; the waterfall in this model is a **stated
placeholder**, not the real split.

## Constructs this case exercises

- entity **field recurrences** (`init` / `next` / `prev`) carrying the facility's
  equity draw, interest, principal draw, repayment and balance
- **`curve`** for the recorded sellout and the deterministic cost path — a step
  curve with *every* period declared, because flat-forward interpolation would
  otherwise hold the last construction draw forward for ever
- **`waterfall`** with `available`, `remaining` and a promote step
- **`part of`** roll-up: `asset.project` aggregates east and west
- pack **categories** driving `domain.cre.*`

## Payment placement

Every recurring schedule here is `start` — an annuity due, cash at the period's
open. That is what `12_payment_timing.md` §6 asks for: revenue, opex, capex and
funding draws fall due in the period they belong to, and only annuity-like
streams (scheduled debt service, coupons) take the `end` default.

It is also what makes this case tie. The reference workbook places every flow
at its bare period index, which is offset 0.0 — the same axis `start` puts it
on. Left at the recurrence default the model returns an IRR of 11.1454%
against the workbook's 11.0161%; on `start` it returns **11.0161%**. The totals
tie either way, because a sum does not care where inside a period its cash
sits — which is exactly why a benchmark that checks only totals would never
have caught this.

Note the spelling. The keyword is **`start`**; `due` appears in the language
spec §11.2.3, the EBNF, `12_payment_timing.md` §3 and
`10_implementation_status.md` row 44, but is not a token — it lexes as a bare
identifier and fails with `E0004_EXPECTED_TOKEN`. The parser and the IR
schema's `placement` enum both say `start | mid | end`.
