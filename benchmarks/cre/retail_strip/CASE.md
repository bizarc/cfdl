## The case

A retail strip center held for ten years. An anchor tenant pays base rent plus
percentage rent above a stated breakpoint; inline shops sit on net leases with
staggered expiries. Operating expense recoveries run off a base-year stop that is
grossed up to 95% occupancy, so the landlord recovers as though the center were
nearly full even when it is not. The center is sold on forward net operating
income.

## The reference

Retail center valuation conventions — base-year gross-ups and percentage rent
as practiced in institutional retail underwriting.

**Not redistributable.** The source cannot be published, so the reference is an
independent recreation of its conventions, built separately from the model and
compared against it period by period.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.lease_unit` (two instances), `cre.percentage_rent`, `cre.vacancy_loss`, `cre.opex_line`, `cre.exit` |
| Language features | multiple instances of one contract type |
| Conventions | a base-year expense stop with a 95% gross-up, percentage rent over a breakpoint, net leases, staggered rollover |

A gross-up implemented as a flat recovery understates income in every year the
center sits below the gross-up threshold.

## The result

Present value **8,328,491.23**, net operating income **4,012,080.73** and leasing
costs **340,000.00**.

Asserted: effective gross income, net operating income and net cash flow per
period across 120 months, plus the three lifetime figures.

Percentage rent, the base-year gross-up and the recovery stop can each be wrong
in offsetting ways that a lifetime NOI would not show, so assertion is per
period.

## The delta

None: every period agrees inside a one-cent tolerance across all 120 months.
