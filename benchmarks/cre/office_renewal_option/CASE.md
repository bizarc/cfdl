## The case

The two-tenant office building of `office_two_tenant`, held for ten years,
with one change. Tenant A's five-year lease carries a renewal option: five more
years at $520k a year, escalating 3% on the renewal term's anniversaries, with
$100k of tenant improvements and leasing commissions at renewal. At expiry the
tenant renews if the market rent then exceeds the option rent. If the option
lapses, the space is re-let at market after three months of downtime, on a new
lease with $350k of leasing costs. Tenant B, the vacancy allowance, the
operating expenses, the permanent mortgage and the sale on forward net
operating income are the original's, unchanged.

## The reference

Institutional lease-by-lease office DCF conventions, as practiced by the
commercial valuation software this kind of model is built in. The original case
handles Tenant A's expiry as a probability-weighted rollover. This case handles
it as the right the lease grants: a renewal at stated terms, exercised or not,
with each outcome a lease of its own.

**Not redistributable.** The source cannot be published, so the reference is an
independent recreation of its conventions, built separately from the model and
compared against it period by period. It is the original case's recreation with
the rollover replaced by the option and its two outcomes.

## What it exercises

| | |
|---|---|
| Pack | `cre` |
| Contract types | `cre.lease_unit` (four instances), `cre.vacancy_loss`, `cre.opex_line`, `cre.permanent_debt`, `cre.exit_forward`, and the election `CRE.Contract.RenewalOption` |
| Language features | an option written on a contract, an option's own terms, an option tested on a stated date, actions on exercise, a scheduled event, a run scenario |
| Conventions | free rent, anniversary escalation, recoveries above an expense stop, tenant improvements and leasing commissions, a renewal option at stated terms, downtime and re-letting at market, a forward-NOI exit |

The renewal is an election, not a probability. The option is written on the
lease it extends and states its own terms:

```cfdl
option renewal on contract cre.lease_unit.tenant_a type CRE.Contract.RenewalOption {
  parties { landlord = party.landlord_co, tenant = party.acme }
  terms {
    renewal_rent_year = inputs.renewal_rent_year
    renewal_term_months = 60
    renewal_ti_lc = 100000
  }
  schedule on 2031-01
  exercise when inputs.market_rent_year > contract.renewal_rent_year
  payoff 0
  set entity asset.tower.tenant_a_renewed = 1
  deactivate stream cre.unit.base_rent.tenant_a_market
  deactivate stream cre.unit.abatement.tenant_a_market
  deactivate stream cre.unit.recoveries.tenant_a_market
  deactivate stream cre.unit.ti_lc.tenant_a_market
}
```

Both outcomes are leases the model declares: the renewal lease from January
2031, and the market lease from April 2031. The schedule tests the election
once, in the month after expiry. On exercise the option records the renewal on
the building and switches off the market lease; a scheduled event with the
opposite test switches off the renewal lease when the option lapses. The exit
values the building on the lease left standing, because the forward net
operating income is derived from the leases' own streams.

The market rent is an assumption, so the outcome the option does not take is a
run scenario rather than a second model. In the base run the market rent is
$560k and the tenant renews. In the `soft_market` scenario the market rent is
$480k, the option lapses, and the space is re-let.

## The result

Base run: present value **1,362,611.39**, net operating income
**4,697,224.27**, leasing costs **450,000.00** and debt service
**4,421,429.94**. Soft-market scenario: present value **595,382.95** and total
cash **1,495,550.50**.

Asserted: seven per-period series across 120 months, the renewal lease's rent,
the market lease's rent, effective gross income, net operating income, debt
service, the coverage ratio and net cash flow, plus the five lifetime figures
and the two scenario figures. Every asserted cell agrees with the reference to
the cent.

Against the original case's present value of 1,424,273.80, the renewal
outcome is **61,662.41** lower and the re-let outcome **828,890.85** lower.
The two figures bound what the original's 70/30 blend stands for.

## The delta

None: every period agrees inside a one-cent tolerance across all 120 months,
and both scenario figures agree to the cent.
