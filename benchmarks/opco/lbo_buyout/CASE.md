## The case

A five-year leveraged buyout of a services business. Entry at 8.0x a $33.6m
run-rate EBITDA, funded with term debt; the business grows, pays the debt down
out of operating cash flow, and is sold at an exit multiple. Working capital
moves on a days-based policy — receivables, payables and inventory — rather than
as a stated figure, and cash taxes are paid on the levered result.

## The reference

Sponsor buyout conventions: sources and uses at entry, a debt schedule paid from
operating cash flow, working capital driven by days outstanding, and an exit at a
multiple of trailing earnings.

**Not redistributable.** The source cannot be published, so its conventions are
recreated independently of the model and compared period by period.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Contract types | `opco.revenue_line`, `opco.opex_line`, `opco.working_capital_policy`, `opco.term_debt`, `opco.cash_taxes`, `opco.acquisition`, `opco.exit_ebitda` |
| Language features | pack contracts across an entry, a hold and an exit |
| Conventions | entry at a multiple, days-based working capital, debt amortization from operating cash flow, an exit on trailing EBITDA |

The widest span of the opco pack's contract surface: entry, hold and exit
rather than one mechanic.

## The result

Present value **13,883,137.75**, multiple on invested capital **3.467004** and
lifetime revenue **69,485,786.14**.

Asserted: net cash flow per period, plus the three summary figures.

## The delta

None: every period agrees inside a one-cent tolerance. The multiple carries a
basis-point tolerance, being computed from an iterative root.
