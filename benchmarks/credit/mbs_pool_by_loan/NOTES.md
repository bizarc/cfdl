# A pool modelled loan by loan — what building it found

## Why this case exists next to `mbs_pool_conventions`

The two carry the same published anchors, which normally means one of them is
redundant. They are not, and the difference is the whole design.

`mbs_pool_conventions` declares one contract on one entity and asserts the
published schedule against the streams that contract emitted. It proves the
CONVENTIONS: level-pay amortisation, SMM applied to the gross balance, MDR, a
twelve-month recovery lag.

Here the pool holds no contract. Four loans do, and the pool is their parent.
Every asserted figure is an aggregate the engine produced by walking `part of`.
The conventions are held constant on purpose — they are already proven — so the
only thing that can move a number is the aggregation.

Before this, `part of` appeared in **no benchmark at all**. Hierarchy rollups
shipped with a fixture and a golden and had never been checked against a figure
CFDL did not produce.

## The two columns are two mechanisms

| Column | Computed by |
|---|---|
| `entity.asset.pool.net_cash_flow` | the engine's entity rollup, walking `part of` |
| `domain.credit.gross_collections` | the pack's category fold over contract instances |

They share no code. Asserting both against the same published figures means a
defect in either surfaces as a disagreement between them, not just as a
disagreement with the source — which is a cheaper signal to read.

## The strongest result is the one not in expected.csv

Against the single-pool model, over all 372 periods:

    max |entity.asset.pool.net_cash_flow − model.net_cash_flow| = 0.0

Zero. Not within a tolerance. Splitting $100mm into loans of $40mm, $30mm, $20mm
and $10mm changes nothing about the pool's cash, which is what a correct rollup
means and what an incorrect one could not fake — the four loans amortise on four
different schedules and only their sum is invariant.

That comparison is not committed as an assertion because both sides are ours.
`expected.csv` carries only figures the reference published.

## Uneven balances, deliberately

40/30/20/10, not 25/25/25/25. Equal loans would agree with the pool under any
aggregation that happened to divide by four, including several wrong ones.

## The defect it found: a declared field no model can set

`Credit.Asset.Loan` declares three fields — `original_balance`, `coupon` and
`term`. Writing the third is a parse error:

    entity asset loan_a : Credit.Asset.Loan {
      term = 360          // ERROR[E0004_EXPECTED_TOKEN]
    }

`term` is a keyword, so the lexer never offers it as an attribute name, and the
entity block cannot accept it. The ontology declares a field that is unreachable
from the language.

Not fixed here, because the fix is a language decision rather than a pack one:
either attribute positions accept keyword-shaped identifiers, or the field is
renamed and the ontology loader rejects field names that collide with keywords.
The second is smaller and catches the next one at load time rather than leaving
it to be discovered by someone writing a model. Recorded in the backlog.

The case is unaffected: the term the schedule uses is the contract's
`term_months`, and the entity attribute is descriptive.

## Tolerance

`period_tolerance = 2.01`, and the bound is arithmetic rather than a judgement.
The reference publishes each column rounded to the whole dollar; a period's cash
is up to four of them added, so two dollars bounds the residual before any model
runs. Largest observed: 1.76.

The other case carries 0.51 because it asserts the columns individually — one
rounding each. Adding them adds their rounding, and the tolerance says so
instead of quietly absorbing it.
