## The case

An institutional two-tenant office building held for ten years. Tenant A takes a
five-year lease with three months free and 3% anniversary escalations; Tenant B
takes seven years from mid-year one at 2.5%. Both recover operating expenses
above their own expense stop, at different pro-rata shares. Tenant A's expiry is
modeled as a probability-weighted rollover — 70% renewal at one rent, otherwise
a new tenant at market after three months of downtime, with different tenant
improvement and leasing commission costs on each branch. A permanent mortgage
runs underneath, and the building is sold on forward net operating income.

## The reference

Institutional lease-by-lease office DCF conventions, as practiced by the
commercial valuation software this kind of model is built in.

**Not redistributable.** The source cannot be published, so the reference is an
independent recreation of its conventions — built separately from the model and
compared against it period by period.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.lease_unit` (two instances), `cre.rollover`, `cre.vacancy_loss`, `cre.opex_line`, `cre.permanent_debt`, `cre.exit_forward` |
| Language features | multiple instances of one contract type, per-period subtotals |
| Conventions | free rent, anniversary escalation, recoveries above an expense stop, tenant improvements and leasing commissions, probability-blended rollover with downtime, a forward-NOI exit |

More of the CRE pack's contract surface than any other case.

## The result

Present value **1,424,273.80**, net operating income **4,718,933.90**, leasing
costs **525,000.00** and debt service **4,421,429.94**.

Asserted: five per-period series across 120 months — effective gross income, net
operating income, debt service, the coverage ratio and net cash flow — plus the
four lifetime figures.

Assertion is per period rather than on the totals: a lifetime coverage ratio of
1.4 can contain a year at 0.9.

## The delta

None: every period agrees inside a one-cent tolerance across all 120 months.
