<!-- GENERATED valid examples corpus by tools/gen-machine-docs.py — do not edit by hand. Regenerate: make machine-docs -->

# CFDL valid examples — the golden corpus

CFDL 0.9.0. Every model below compiles, and its IR and
results are byte-asserted against goldens in CI (`fixtures/valid/`,
157 models.

`gold/ir/`, `gold/results/`). Each is single-purpose: the directory name
says what it exercises. This is what right looks like — positive few-shot
material to pair with the diagnostics repair catalog.

## account_after_assume

```cfdl
version 0.1
model "account-after-assume"
time calendar monthly from 2026-01 for 4

// AN ACCOUNT DECLARED AFTER AN ASSUMPTION.
//
// `assume <name> = <expr>` has no terminator of its own, so the parser scans
// to the next top-level statement — and `is_statement_start` is the list that
// says where that is. `account` was missing from it, so the scan ran straight
// through this declaration and swallowed it: no diagnostic at the swallow
// site, and the account simply did not exist. The only symptom was
// `E1347_UNRESOLVED_ACCOUNT_REF` at whatever referred to it later, pointing at
// the reference rather than at the cause — and a model that referred to the
// account nowhere lost it in total silence.
//
// The same list already carries the same scar: the comment at
// `scan_to_next_top_level_statement` records a `metric` declared after a
// contract vanishing the same way. Found by `docs/13` §7.76 —
// `benchmarks/credit/americredit_2017_1` states its reserve as an assumption
// and declares the account against it, which is this exact order.
//
// The pin is that the account carries its balance. `lifecycle` is the other
// statement the list was missing; it is an ordinary identifier rather than a
// keyword, and `lifecycle_after_assume` pins it separately.

entity asset co { name = "Co" }
entity party sponsor { name = "Sponsor" }

assume required = 100.0

account reserve {
  from if(time.t == 0.0, inputs.required, 0.0)
}

stream ops.revenue on entity asset.co inflow currency USD {
  schedule every month from 2026-01 to 2026-04
  amount = 50
}

// Funded at inception by the account's own inflow, so the top-up finds the
// balance already at target and pays nothing — every period, on purpose.
waterfall funding on entity asset.co {
  schedule every month from 2026-02 to 2026-04
  from available
  pay topup    to account reserve = max(0.0, inputs.required - prev.reserve)
  pay residual to party.sponsor   = remaining
}
```

## account_cumulative_identity

```cfdl
version 0.1
model "account-cumulative-identity"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// THE CUMULATIVE-SUM IDENTITY, PINNED — docs/28 §5.1.
//
// An account fed a deal's whole net cash IS the deal's cumulative position:
// `balance(t) = series_sum("cre.*", 0, t)`, every period. Capex makes the
// first quarter net-negative, so the balance goes NEGATIVE THROUGH THE
// J-CURVE — a balance has no floor, because the language models returns and
// the position through the J-curve is the truth of the deal. What is floored
// is only what a step may take.
//
// Nothing distributes here: the account observes. The identity is the whole
// assertion — position [-2000, -4000, -6000, -5100, ...] against cumulative
// net cash, equal in every cell.

entity asset suite : CRE.Asset.Unit {
  rentable_area = 1000
  state leased
}

stream cre.capex on entity asset.suite outflow currency USD {
  schedule every month from 2026-01 to 2026-03
  category investing.capital.capex
  amount = 2000
}

stream cre.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-04 to 2026-12
  category operating.revenue.base_rent
  amount = 900
  active in state leased
}

account position {
  from series_sum("cre.*", time.t, time.t)
}
```

## account_moved_by_streams

```cfdl
version 0.1
model "account-moved-by-streams"
time calendar monthly from 2026-01 for 8

// THE BALANCE IS AN ACCOUNT THE STREAMS MOVE (docs/42 §3). A loan the
// borrower owes, rolled by the engine from what happened: principal paid
// (cash, lowers it), interest capitalized (an accrual, raises it with no
// money moving), a loss written off (lowers it with no money moving), and
// interest that READS the opening balance — the prior close, or the init
// in the first period. Nothing writes the balance.
entity asset loan : Asset.Financial {
  account balance owed init 1000000
}

// The lender's mirror: the same claim as a receivable, DUE. The same
// direction word moves it the other way, because the side decides.
entity party lender {
  account receivable due init 1000000
}

stream loan.interest on entity asset.loan outflow currency USD {
  schedule every month from 2026-01 to 2026-08
  amount = prev.balance * 0.06 / 12
}

stream loan.principal on entity asset.loan outflow currency USD {
  schedule every month from 2026-01 to 2026-08
  amount = 10000
  moves balance
}

stream loan.pik on entity asset.loan accrual currency USD {
  schedule every month from 2026-01 to 2026-08
  amount = prev.balance * 0.01
  moves balance
}

stream loan.loss on entity asset.loan writeoff currency USD {
  schedule every month from 2026-06 to 2026-06
  amount = 5000
  moves balance
}

stream lender.principal on entity party.lender inflow currency USD {
  schedule every month from 2026-01 to 2026-08
  amount = 10000
  moves receivable
}
```

## account_reserve_cycle

```cfdl
version 0.1
model "account-reserve-cycle"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// A RESERVE IS FUNDED TO TARGET, RELEASED, AND REFILLED — docs/28 §5.1's
// reserve pattern, as one step form and one account.
//
// The monthly waterfall funds the reserve to 300 and tops it up when short:
// `target - prev.<account>` is a strictly-backward read, the balance as it
// stood when the period opened. The release waterfall runs ONCE, on its own
// schedule, drawing `from reserve` — the schedule stays sovereign, and on
// every other period the account just carries.
//
// June: the release drains the reserve to zero. July: the top-up sees the
// empty account through `prev.reserve` and refills it. The balance column
// reads 300 through May, 0 in June, 300 from July — funded, released,
// refunded, with every movement journaled.

entity asset suite : CRE.Asset.Unit {
  rentable_area = 1000
  state leased
}

entity party sponsor : Party { name = "Sponsor" }

account reserve { }

stream cre.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  category operating.revenue.base_rent
  amount = 1000
  active in state leased
}

waterfall dist on entity asset.suite {
  schedule every month from 2026-01 to 2026-12
  from available
  pay top_up   to account reserve = if(time.t == 0, 300.0, max(0.0, 300.0 - prev.reserve))
  pay residual to party.sponsor   = remaining
}

waterfall release on entity asset.suite {
  schedule on 2026-06
  from reserve
  pay released to party.sponsor = remaining
}
```

## account_restates_the_window

```cfdl
version 0.1
model "account-restates-the-window"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// HIGHLANDS' SHAPE, RESTATED THROUGH AN ACCOUNT — docs/28 §5.1.
//
// penzance_highlands distributes once, at the end of the hold, from
// `series_sum("cre.*", 0, time.t)`: the hand-written cumulative window. The
// account is that window as a construct: `from collection` where the account's
// inflow is each period's net cash. Both waterfalls run here, at the same end
// date, over the same deal — and every mirrored step must take the SAME
// number, which the golden pins cell for cell. A declared cumulative window
// stays legal and means what it says (docs/28 §5.2); the account is the same
// cash as a location instead of an expression.

entity asset suite : CRE.Asset.Unit {
  rentable_area = 1000
  state leased
}

entity party investor : Party { name = "Investor" }
entity party sponsor  : Party { name = "Sponsor" }

account collection {
  from series_sum("cre.*", time.t, time.t)
}

stream cre.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  category operating.revenue.base_rent
  amount = 1000
  active in state leased
}

waterfall windowed on entity asset.suite {
  schedule on 2026-12
  from series_sum("cre.*", 0, time.t)
  pay preferred to party.investor = 8000.0
  pay promote   to party.sponsor  = remaining * 0.20
  pay residual  to party.investor = remaining
}

waterfall accounted on entity asset.suite {
  schedule on 2026-12
  from collection
  pay preferred to party.investor = 8000.0
  pay promote   to party.sponsor  = remaining * 0.20
  pay residual  to party.investor = remaining
}
```

## active_in_state

```cfdl
version 0.1
model "active-in-state"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 5

entity asset suite : CRE.Asset.Unit {
  rentable_area = 10000
  state leased
}

event expiry when time.t >= 2 {
  set entity asset.suite.status = "vacant"
}

event reletting when time.t >= 4 {
  set entity asset.suite.status = "leased"
}

// The state name is CHECKED against the unit lifecycle. A misspelling is a
// compile error, where `asset.suite.status == "leasd"` would just be false
// forever and say nothing.
stream cre.rent on entity asset.suite inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  category operating.revenue.base_rent
  amount = 100
  active in state leased, holdover
}
```

## active_when_series

```cfdl
version 0.1
model "active-when-series"
time calendar annual from 2026-01 for 3

entity asset co : Asset.Financial

stream base.revenue on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 100
}

// The amount uses no series; the GUARD does. Guards participate in wave
// classification too — once they did not, and the stream silently produced nothing.
stream bonus.fee on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  active when series_sum("base.revenue", 0, time.t) > 150
  amount = 25
}
```

## arrival_actions_entry_and_edge

```cfdl
version 0.1
model "arrival-actions-entry-and-edge"
time calendar monthly from 2026-01 for 8

// Both grains, and the conflict between them. `on enter` carries what is true
// of the STATE however reached; the edge carries what is true of the PATH.
// Entry runs first and the edge refines it, so the entry's value is
// journalled `overridden` and the edge's stands.
//
// The write settles the field store, so the stream reads 1200 in the SAME
// period the transition fires, and the recurrence resumes from it after.
lifecycle unit {
  initial vacant
  state vacant, leased

  on enter leased {
    set in_place_rent = 1000.0
  }

  vacant -> leased when time.t >= 2 {
    set in_place_rent = 1200.0
  }
}

entity asset suite {
  lifecycle unit
  in_place_rent init 900.0 next prev
}

stream core.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-08
  amount = asset.suite.in_place_rent
}
```

## assume_monte_carlo

```cfdl
version 0.1
model "assume-monte-carlo"
time calendar monthly from 2026-01 for 6

entity asset borrower : Asset.Financial

assume rent ~ Uniform(min=900, max=1100)
assume vacancy ~ Triangular(min=0.0, mode=0.05, max=0.15)

stream lease.rent on entity asset.borrower inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = inputs.rent * (1 - inputs.vacancy)
}
```

## assume_smoke

```cfdl
version 0.1
model "assume-smoke"
time calendar monthly from 2026-01 for 6

entity asset borrower : Asset.Financial

assume base_fee = 200 + 50
assume growth ~ Normal(mean=0.03, stdev=0.01, clip=[0.0, 0.08])

// Deterministic run: growth resolves to its central value (the mean, 0.03).
stream fee.management on entity asset.borrower inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = inputs.base_fee * (1 + inputs.growth)
}
```

## assumption_derived

```cfdl
version 0.1
model "assumption-derived"
time calendar annual from 2026-01 for 3

// A DERIVED ASSUMPTION — one assumption computed from others.
//
// "Net rentable is gross times efficiency" is ordinary modeling, and stating
// it once beats restating 8,500 wherever the number is needed: change the
// efficiency and everything downstream follows.
//
// Assumptions once evaluated in name order against an empty environment, so a
// read of another assumption found nothing, the assumption was skipped, and
// every read of it resolved to nothing. They now evaluate in dependency
// order — the same ordering the stream layer does, one layer up — with the
// same single rejection: a circular derivation, which no order can satisfy.
//
// Note the ordering is NOT alphabetical: `net_sf` is declared last but reads
// two assumptions declared before it, while `rent_psf` reads none.

entity asset co : Asset.Financial

assume gross_sf   = 10000.0
assume efficiency = 0.85
assume rent_psf   = 2.0
assume net_sf     = inputs.gross_sf * inputs.efficiency

// 10,000 x 0.85 = 8,500 SF at $2.00 = $17,000 per period.
stream base.rent on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = inputs.net_sf * inputs.rent_psf
}
```

## bespoke_oil_gas_ep

```cfdl
version 0.1
model "bespoke-oil-gas-ep"
time calendar monthly from 2026-01 for 60

phase drilling   from 2026-01 to 2026-06
phase ramp       from 2026-07 to 2026-12
phase production from 2027-01 to 2030-12

entity asset asset : Asset.Real
entity asset drillco : Asset.Financial
entity asset opco : Asset.Financial

// Initial capex: spud-to-TD, casing, wellhead at t=0
stream capital.well_capex on entity asset.drillco outflow currency USD {
  schedule on phase_enter("drilling")
  amount = 3500000
}

// Monthly drilling & completion costs during drilling phase (t=0..5)
stream capital.drilling_costs on entity asset.drillco outflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 250000
}

// Gross production revenue: $180k/mo at peak, 6%/mo exponential decline
// t=12: $180,000; t=24: $84,843; t=36: $37,592; t=48: $18,827
stream rev.gross_production on entity asset.asset inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 180000 * pow(0.94, time.t - 12.0)
  active when time.t >= 12
}

// Royalty burden: 18.75% of gross revenue (landowner + overriding royalties)
// NRI = 81.25%; royalty declines with production at same 6%/mo rate
stream rev.royalty_burden on entity asset.asset outflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 33750 * pow(0.94, time.t - 12.0)
  active when time.t >= 12
}

// LOE: pumping unit, chemicals, water disposal, field labor — begins at ramp phase
stream opex.loe on entity asset.opco outflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 28000
  active when time.t >= 6
}

// G&A overhead: corporate allocation, land, accounting (all 60 months)
stream opex.ga on entity asset.opco outflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 12000
}
```

## bespoke_saas

```cfdl
version 0.1
model "bespoke-saas"
time calendar monthly from 2026-01 for 36

phase ramp   from 2026-01 to 2026-06
phase growth from 2026-07 to 2027-12
phase scale  from 2028-01 to 2028-12

entity asset product : Asset.Financial
entity asset venture : Asset.Financial

// Seed: one-time outflow at phase entry (t=0)
stream ops.seed_investment on entity asset.venture outflow currency USD {
  schedule on phase_enter("ramp")
  amount = 500000
}

// MRR: monthly revenue with 15% annual compound growth via pow (registered in A.6)
// Uses concrete date range (2026-07 to 2028-12) for full growth+scale span.
// t=6  => 50000 * pow(1.15, 6/12.0)  ~= 53619
// t=24 => 50000 * pow(1.15, 24/12.0) = 66125
stream saas.mrr on entity asset.product inflow currency USD {
  schedule every month from 2026-07 to 2028-12
  amount = 50000 * pow(1.15, time.t / 12.0)
}

// Team cost: fixed $30k/month for all 36 months
stream ops.team_cost on entity asset.venture outflow currency USD {
  schedule every month from 2026-01 to 2028-12
  amount = 30000
}

// Infra cost: linearly growing, suppressed during ramp (t < 6)
// t=6  => 2000 + 6*50  = 2300
// t=24 => 2000 + 24*50 = 3200
stream saas.infra_cost on entity asset.product outflow currency USD {
  schedule every month from 2026-01 to 2028-12
  amount = 2000 + time.t * 50
  active when time.t >= 6
}
```

## conditional_window_bound_walks

```cfdl
version 0.1
model "conditional-window-bound-walks"
time calendar monthly from 2026-01 for 4

// A CONDITIONAL WINDOW BOUND IS BACKWARD WHEN BOTH BRANCHES ARE.
//
// `walk_ineligible_reason` asks whether a series window can reach past the
// current period; if it can, the model keeps the column order, and the column
// order has no periods to carry an account balance through. The check reads
// the bound's SOURCE TEXT and recognised numeric literals, `time.t` plus a
// chain of signed literals, and `max`/`min` of those. It did not recognise
// `if`, so a bound that is backward down both branches was reported forward.
//
// The cost was not theoretical. `benchmarks/credit/americredit_2017_1` draws
// its pot with the lower bound `if(time.t == 1.0, 0.0, time.t)` — the first
// distribution collects two periods, because six of the twelve assumed pools
// have a January cutoff and six a February one. That one conditional held the
// corpus's most intricate waterfall on the column order, so the reserve
// account it needed could be declared and could never carry a balance. The
// engine said so rather than publishing zeros, which is how it was found.
//
// The condition is NOT examined and does not need to be: whichever branch it
// selects, the bound is one of the two, so both being backward settles it. A
// forward branch makes the whole bound forward — `window_bound_is_backward`'s
// own unit tests pin that direction.
//
// The pin here is the CONSEQUENCE, not the predicate: this model both draws
// from an if-bounded window and declares an account, so if the bound were
// read as forward the run would warn and `account.reserve` would be absent.

entity asset co { name = "Co" }
entity party sponsor { name = "Sponsor" }

account reserve { }

stream ops.revenue on entity asset.co inflow currency USD {
  schedule every month from 2026-01 to 2026-04
  amount = 100
}

// The first distribution draws two periods of revenue and every later one
// draws its own — the AmeriCredit pot's shape, at four periods.
waterfall funding on entity asset.co {
  schedule every month from 2026-02 to 2026-04
  from series_sum("ops.revenue", if(time.t == 1.0, 0.0, time.t), time.t)
  pay to_reserve to account reserve = 30.0
  pay residual   to party.sponsor   = remaining
}
```

## container_entity

```cfdl
version 0.1
model "container-entity"
time calendar annual from 2026-01 for 3

// A CONTAINER GROUPS AND SCOPES; IT DOES NOT PRODUCE.
//
// A fund is not an asset — `Asset.Financial` claims "a claim on cash," which
// a grouping is not. The `container` family says what the thing IS, and the
// language base ships the types (`Container.Fund`, `Container.Portfolio`,
// `Container.SPV`, `Container.Transaction`) for a model to use pack-free and
// for a pack to refine (docs/13 §7.88).
//
// Containment reuses `part_of` — one hierarchy concept, widened endpoints —
// so the parent's cash aggregates BY THE RELATION, exactly as an asset
// parent's does: `entity.container.fund.net_cash_flow` below carries both
// holdings' cash, and counts as cash nowhere, because a fold of the cash
// never counts AS cash.

entity container fund : Container.Fund

entity asset alpha : Asset.Financial { part of container.fund }
entity asset beta  : Asset.Financial { part of container.fund }

stream alpha.income on entity asset.alpha inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.other
  amount = 100
}

stream beta.income on entity asset.beta inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.other
  amount = 40
}
```

## container_field_read

```cfdl
version 0.1
model "container-field-read"
time calendar annual from 2026-01 for 3

// A CONTAINER'S FIELD READS LIKE ANY OTHER ENTITY'S. `container.fund.fee_rate`
// is the same spelling an asset's field answers to — the family joined the
// declaration surface in one release and the expression environment lagged a
// list (the fourth-list defect #222 named; spec audit, 31 August 2026). This
// fixture is the read that failed.

entity container fund : Container.Fund { fee_rate = 0.02 }
entity asset alpha : Asset.Financial { part of container.fund }

stream alpha.income on entity asset.alpha inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.other
  amount = 100
}

stream fund.fee on entity container.fund outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.expense.management_fee
  amount = 100 * container.fund.fee_rate
}
```

## container_folds_member_accounts

```cfdl
version 0.1
model "container-folds-member-accounts"
time calendar monthly from 2026-01 for 8

// THE RELATION FOLDS BALANCES AS IT FOLDS CASH (docs/42 §3.4). Two loans a
// trust holds, each carrying the claim it is due; the trust declares no
// balance of its own and has one anyway — `container.trust.balance`, the
// sum of its members', opening and closing. A field on the trust reads it
// as `prev.container.trust.balance`, and a clean-up call is written once
// against the trust, never against a member.
entity container trust : Container.SPV

entity asset loan_a : Asset.Financial {
  part of container.trust
  account balance due init 600000
}
entity asset loan_b : Asset.Financial {
  part of container.trust
  account balance due init 400000
}

// What the trust holds at the open of each period, carried as a field so a
// statement row can show it beside the cash.
entity container trust_view : Container.SPV {
  held init 0 next prev.container.trust.balance
  called init 0
}

stream loan_a.principal on entity asset.loan_a inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 100000
  moves balance
}
stream loan_b.principal on entity asset.loan_b inflow currency USD {
  schedule every month from 2026-01 to 2026-08
  amount = 50000
  moves balance
}

// The call: once the trust's opening balance is under a quarter of the
// initial, the deal is called. One event, at the trust, reading the fold —
// the seventh period, when the opening is 100,000 against 1,000,000.
event clean_up_call when prev.container.trust.balance < 250000 {
  set entity container.trust_view.called = 1
}
```

## contract_inherits_master_field

```cfdl
version 0.1
model "contract-inherits-master-field"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset target : OpCo.Asset.Enterprise

// TERMS INHERIT DOWN THE CHAIN (docs/40 §3). The OpCo pack declares no
// `day_count` of its own; `opco.term_debt` refines `Contract.Debt`, which
// does, so the term is accepted against the effective roster and read by the
// accrual divisor as on every debt. The IR records the resolved type and its
// master.
contract opco.term_debt on entity asset.target {
  term 2026-01..2027-12
  terms {
    principal = 5000000
    interest_rate = 0.08
    amortization_months = 84
    day_count = "act/360"
  }
}
```

## contract_two_token

```cfdl
version 0.1
model "contract-two-token"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty
entity party landlord_co : Party
entity party acme : Party

// THE INSTANCE IS ITS OWN TOKEN (docs/13 §7.63; docs/40 §8): the type is
// `cre.lease_unit` and the instance is `tenant_a`, and the compiler checks
// the type instead of recovering it from a prefix. The fully qualified name
// is the same `cre.lease_unit.tenant_a` the fused spelling produces, so the
// lowered streams, the IR and every reference are unchanged.
//
// The parties bind the PACK's roles: `landlord` is the CRE specialization of
// the master's `lessor`, and the IR records both words.
contract cre.lease_unit tenant_a on entity asset.tower {
  term 2026-01..2027-12
  terms {
    rent_year = 480000
    escalation = 0.03
  }
  parties {
    landlord = party.landlord_co
    tenant = party.acme
  }
}

// The fused spelling keeps working, and resolves to the same type.
contract cre.lease_unit.tenant_b on entity asset.tower {
  term 2026-07..2027-12
  terms {
    rent_year = 360000
  }
}
```

## cre_derived_lines

```cfdl
version 0.1
model "cre-derived-lines"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 84

// THREE CRE LINES DERIVED FROM OTHER LINES.
//
// The backlog called these "three ordinary CRE requirements, one cause": a
// vacancy that tracks the rent roll, a management fee that is a percentage of
// effective gross income, and an expense stop that resets to a later year's
// ACTUAL opex. Each needs one line to read another, and each was written as a
// restated constant instead — a number that goes stale the moment an
// assumption changes.
//
// The cause was the engine's two-phase stream split, which allowed a reader to
// read only non-readers. Streams now evaluate in dependency-ordered waves, so
// a chain is ordinary: opex, then the recoveries that read it, then the exit
// that reads them. NO PACK CHANGE WAS NEEDED for any of the three — a contract
// term already holds an expression, and the expression may name another
// stream. This fixture is the proof, and the pattern a modeller copies.

entity asset strip_center : CRE.Asset.RealProperty

// The operating expense the other lines are measured against.
contract cre.opex_line on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    amount_year = 240000
    growth_rate = 0.03
  }
}

// (1) AN EXPENSE STOP THAT RESETS TO ACTUAL OPEX.
//
// Recoveries are tested against what the building actually spent, not against
// a restatement of how the spend is computed, and the stop resets to the 2028
// actual (period 24) rather than a hand-computed constant. Opex books signed
// negative, hence the negation; `time.ppy` annualizes a monthly figure because
// the terms are stated per year, and follows the model's calendar rather than
// hard-coding twelve.
//
// The read names `cre.opex.line` EXACTLY. A `cre.opex.line.*` selector would
// also pull in the management fee below, which reads these recoveries — a
// genuine cycle, which the engine refuses by name rather than answering.
contract cre.lease_unit.anchor on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    rent_year = 540000
    escalation = 0.02
    opex_year = (0 - series_sum("cre.opex.line", time.t, time.t)) * time.ppy
    opex_escalation = 0
    expense_stop_year = (0 - series_sum("cre.opex.line", 24, 24)) * time.ppy
    gross_up_factor = 1.0
    pro_rata_share = 0.60
    ti_total = 150000
    lc_total = 60000
  }
}

// (2) VACANCY THAT TRACKS THE RENT ROLL, AND STEPS AT A CLIFF.
//
// Potential gross rent grows with escalation and changes as suites come and
// go, and the loss follows it. The rate is an expression too, so the 46% step
// at a 2030 affordability cliff is stated where it happens.
contract cre.vacancy_loss on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    rate = if(time.date >= date(2030, 1, 1), 0.46, 0.03)
    potential_gross_year = series_sum("cre.unit.base_rent.*", time.t, time.t) * time.ppy
  }
}

// (3) A MANAGEMENT FEE THAT IS A PERCENTAGE OF EGI.
//
// Effective gross income is rent plus recoveries less vacancy — all revenue,
// so the fee reads no opex line and cannot read itself. The fee falls when
// vacancy rises, which is the whole point of stating it as a percentage.
//
// This is the deepest chain here: rent, then vacancy and recoveries that read
// rent and opex, then this fee that reads them, then the exit that reads every
// opex line including this one.
contract cre.opex_line.management on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    amount_year = (series_sum("cre.unit.base_rent.*", time.t, time.t)
                   + series_sum("cre.unit.recoveries.*", time.t, time.t)
                   + series_sum("cre.vacancy.loss", time.t, time.t)) * time.ppy * 0.04
    growth_rate = 0
    pct_fixed = 1.0
  }
}

contract cre.exit on entity asset.strip_center {
  term 2032-12..2032-12
  terms {
    income = 640000
    cap_rate = 0.0675
    selling_costs = 0.015
  }
}
```

## cre_developer_scenarios

```cfdl
version 0.1
model "cre-developer-scenarios"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset property : CRE.Asset.RealProperty

contract cre.construction_stub {
  term 2026-01..2026-06
  terms {
    amount = 45000
  }
}

contract cre.lease {
  term 2026-07..2031-12
  terms {
    rent = 25000
  }
}

contract cre.revenue_line {
  term 2028-01..2031-12
  terms {
    amount = 30000
  }
}

contract cre.opex_line {
  term 2028-01..2031-12
  terms {
    amount = 12000
  }
}

contract cre.exit_cap {
  term 2031-12..2031-12
  terms {
    cap_rate = 0.06
    income = 180000
  }
}
```

## cre_developer_smoke

```cfdl
version 0.1
model "cre-developer-smoke"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset property : CRE.Asset.RealProperty

contract cre.construction_stub {
  term 2026-01..2026-06
  terms {
    amount = 45000
  }
}

contract cre.lease {
  term 2026-07..2031-12
  terms {
    rent = 25000
  }
}

contract cre.revenue_line {
  term 2028-01..2031-12
  terms {
    amount = 30000
  }
}

contract cre.opex_line {
  term 2028-01..2031-12
  terms {
    amount = 12000
  }
}

contract cre.exit_cap {
  term 2031-12..2031-12
  terms {
    cap_rate = 0.06
    income = 180000
    income = 216000
  }
}
```

## cre_development_with_financing_smoke

```cfdl
version 0.1
model "cre-development-with-financing"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset property : CRE.Asset.RealProperty
entity asset construction : Asset.Financial
entity asset permanent : Asset.Financial

contract cre.construction_stub on entity asset.property {
  term 2026-01..2026-06
  terms {
    amount = 45000
  }
}

contract cre.lease on entity asset.property {
  term 2026-07..2031-12
  terms {
    rent = 25000
  }
}

contract cre.revenue_line on entity asset.property {
  term 2028-01..2031-12
  terms {
    amount = 30000
  }
}

contract cre.opex_line on entity asset.property {
  term 2028-01..2031-12
  terms {
    amount = 12000
  }
}

contract cre.exit_cap on entity asset.property {
  term 2031-12..2031-12
  terms {
    cap_rate = 0.06
    income = 180000
  }
}

stream loan.construction_interest on entity asset.construction {
  schedule every month from 2026-01 to 2027-06
  category financing.debt.service
  amount = 40000
}

stream loan.permanent_debt_service on entity asset.permanent {
  schedule every month from 2027-07 to 2031-12
  category financing.debt.service
  amount = 55000
}
```

## cre_lease_up

```cfdl
version 0.1
model "cre-lease-up"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset property : CRE.Asset.RealProperty

contract cre.lease {
  term 2026-07..2027-12
  terms {
    rent = 25000
    lease_up_months = 18
  }
}
```

## cre_mixed_use_io_construction

```cfdl
version 0.1
model "cre-mixed-use-io-construction"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 96

entity asset property : CRE.Asset.RealProperty
entity asset construction : Asset.Financial
entity asset permanent : Asset.Financial

// Operating expenses during stabilized operations (2029-01 to 2033-12)
// mgmt + maintenance + insurance + RE taxes + utilities = $55,000/mo
contract cre.opex_line on entity asset.property {
  term 2029-01..2033-12
  terms {
    amount = 55000
  }
}

// Exit month 95: stabilized NOI = ($120k + $35k + $45k - $55k) * 12 = $1,740,000/yr
// Cap rate 5.50% → sale price = $1,740,000 / 0.055 = $31,636,363.64
contract cre.exit_cap on entity asset.property {
  term 2033-12..2033-12
  terms {
    cap_rate = 0.055
    income = 1740000
  }
}

// Construction IO interest: $12M loan @ 7.5% annual = $75,000/mo (months 0-35)
stream loan.io_interest on entity asset.construction outflow currency USD {
  schedule every month from 2026-01 to 2028-12
  category financing.debt.service
  amount = 75000
}

// Office lease revenue: $120,000/mo stabilized with 18-month lease-up ramp
// t=36: $120k * (1/18) = $6,667; t=53: $120k * (18/18) = $120,000
stream office.base_rent on entity asset.property inflow currency USD {
  schedule every month from 2029-01 to 2033-12
  category operating.revenue.base_rent
  amount = 120000 * clamp((time.t - 36.0 + 1.0) / 18.0, 0.0, 1.0)
}

// Retail lease revenue: $35,000/mo stabilized with 6-month lease-up ramp
// t=36: $35k * (1/6) = $5,833; t=41: $35k * (6/6) = $35,000
stream retail.base_rent on entity asset.property inflow currency USD {
  schedule every month from 2029-01 to 2033-12
  category operating.revenue.base_rent
  amount = 35000 * clamp((time.t - 36.0 + 1.0) / 6.0, 0.0, 1.0)
}

// Residential lease revenue: $45,000/mo stabilized with 3-month lease-up ramp
// t=36: $45k * (1/3) = $15,000; t=38: $45k * (3/3) = $45,000
stream resi.base_rent on entity asset.property inflow currency USD {
  schedule every month from 2029-01 to 2033-12
  category operating.revenue.base_rent
  amount = 45000 * clamp((time.t - 36.0 + 1.0) / 3.0, 0.0, 1.0)
}

// Permanent debt service replaces IO after construction close (months 36-95)
// $12M perm loan @ 6.5% / 25-yr amortization → ~$68,500/mo P&I
stream loan.perm_debt_service on entity asset.permanent outflow currency USD {
  schedule every month from 2029-01 to 2033-12
  category financing.debt.service
  amount = 68500
}
```

## cre_multifamily_100unit

```cfdl
version 0.1
model "cre-multifamily-100unit"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity asset property : CRE.Asset.RealProperty

// 100 units × $1,850/mo market rent, 5% vacancy → $175,750/mo effective rental income
contract cre.revenue_line on entity asset.property {
  term 2026-01..2030-12
  terms {
    amount = 175750
  }
}

// Ancillary income: parking + laundry + storage ($7,200/mo)
stream ancillary.income on entity asset.property inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  category operating.revenue.other
  amount = 7200
}

// Operating expenses: mgmt 6% ($11,100) + maintenance ($18,500) + insurance ($6,500)
// + RE taxes ($14,200) + utilities ($8,800) + capex reserves ($4,500) = $63,600/mo
contract cre.opex_line on entity asset.property {
  term 2026-01..2030-12
  terms {
    amount = 63600
  }
}

// Exit at month 60: NOI = ($175,750 + $7,200 - $63,600) * 12 = $1,432,200/yr
// Cap rate 5.20% → sale price = $1,432,200 / 0.052 = $27,542,307.69
contract cre.exit_cap on entity asset.property {
  term 2030-12..2030-12
  terms {
    cap_rate = 0.052
    income = 1432200
  }
}
```

## cre_office_two_tenant

```cfdl
version 0.1
model "office-two-tenant"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 120 project 12

entity asset tower : CRE.Asset.RealProperty

// Tenant A: 5-year lease, 3 months free, 3% anniversary escalations,
// recoveries above a full stop at 40% pro-rata, $200k TI/LC.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2030-12
  terms {
    rent_year = 480000
    free_rent_months = 3
    escalation = 0.03
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 300000
    pro_rata_share = 0.40
    ti_total = 120000
    lc_total = 80000
  }
}

// Tenant A rollover: window starts AT EXPIRY; during the 3 downtime months
// only the renewal scenario (70%) pays. Runs through the projection tail so
// exit valuation sees a full forward year.
contract cre.rollover.tenant_a on entity asset.tower {
  term 2031-01..2036-12
  terms {
    renewal_probability = 0.7
    renewal_rent_year = 520000
    market_rent_year = 560000
    market_escalation = 0.03
    downtime_months = 3
    renewal_ti_lc = 100000
    new_ti_lc = 350000
  }
}

// Tenant B: 7-year lease from mid-2026, 2.5% escalations, $180k stop at 30%.
contract cre.lease_unit.tenant_b on entity asset.tower {
  term 2026-07..2033-06
  terms {
    rent_year = 360000
    escalation = 0.025
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 180000
    pro_rata_share = 0.30
    ti_total = 100000
    lc_total = 50000
  }
}

contract cre.vacancy_loss on entity asset.tower {
  term 2026-01..2036-12
  terms {
    rate = 0.02
    potential_gross_year = 900000
  }
}

contract cre.opex_line on entity asset.tower {
  term 2026-01..2036-12
  terms {
    amount_year = 300000
    growth_rate = 0.025
  }
}

// Sale at the end of the hold; NOI for the valuation year is DERIVED from
// the modeled streams over the 12 projection months after the sale date.
contract cre.exit_forward on entity asset.tower {
  term 2035-12..2035-12
  terms {
    cap_rate = 0.065
    selling_costs = 0.02
  }
}

// Permanent debt (25-year amortization, 10-year hold window).
stream loan.permanent_debt_service on entity asset.tower outflow currency USD {
  schedule every month from 2026-01 to 2035-12
  category financing.debt.service
  amount = -pmt(0.055 / 12, 300, 6000000)
}
```

## cre_percentage_rent_expected

```cfdl
version 0.1
model "cre-percentage-rent-expected"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 3

entity asset store : CRE.Asset.RealProperty

// A tenant's annual sales as a DISTRIBUTION. The trapezoids average to exactly
// 1,000,000, which is the point estimate used below — so the two contracts
// differ in the shape of the payoff and in nothing else. Any gap between them
// is the option, not two disagreeing forecasts.
quantile store_sales linear {
  0.00:  400000.0
  0.25:  700000.0
  0.50:  950000.0
  0.75: 1250000.0
  1.00: 1800000.0
}

// THE SAME LEASE, PRICED TWICE.
//
// The breakpoint is 1,200,000 — above expected sales, which is the ordinary
// shape of a natural breakpoint. The point-estimate contract therefore pays
// EXACTLY ZERO: max(0, 1,000,000 - 1,200,000) is zero however much of the
// distribution lies above the breakpoint. Roughly 29% of it does.
contract cre.percentage_rent.point on entity asset.store {
  term 2026-01..2028-01
  terms {
    sales_year      = 1000000
    sales_growth    = 0.0
    breakpoint_year = 1200000
    overage_pct     = 0.06
  }
}

contract cre.percentage_rent_expected.dist on entity asset.store {
  term 2026-01..2028-01
  terms {
    sales_quantile  = store_sales
    sales_growth    = 0.0
    breakpoint_year = 1200000
    overage_pct     = 0.06
  }
}
```

## cre_permanent_debt_smoke

```cfdl
version 0.1
model "cre-permanent-debt-smoke"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity asset tower : CRE.Asset.RealProperty

// A commercial mortgage in its three phases, all in one contract:
//
//   periods  0-23   interest only        6,000,000 * 0.055/12 = 27,500.00
//   periods 24-58   level payment        pmt(0.055/12, 300, 6,000,000)
//   period     59   payment + balloon    the unamortized balance falls due
//
// The balloon is opted into here precisely because it is OFF by default —
// a fixture that only exercised the default would never see it. Debt service
// coverage is measured on periodic debt service, so the standard pro forma
// repays the balance out of the sale rather than as a debt service line.
//
// amort_months (300) is deliberately longer than the term (60). That split is
// the defining feature of a commercial mortgage and the reason the balloon
// exists at all.
contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2030-12
  terms {
    principal = 6000000
    interest_rate = 0.055
    amortization_months = 300
    interest_only_months = 24
    balloon_at_maturity = 1
  }
}
```

## cre_retail_strip

```cfdl
version 0.1
model "retail-strip"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 84

entity asset strip_center : CRE.Asset.RealProperty

// Anchor grocer: base-year stop (stop = year-0 grossed-up opex), 95% gross-up,
// 60% pro-rata, plus percentage rent above a $12M breakpoint.
contract cre.lease_unit.anchor on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    rent_year = 540000
    escalation = 0.02
    opex_year = 240000
    opex_escalation = 0.03
    expense_stop_year = 228000
    gross_up_factor = 0.95
    pro_rata_share = 0.60
    ti_total = 150000
    lc_total = 60000
  }
}

contract cre.percentage_rent.anchor on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    sales_year = 11500000
    sales_growth = 0.03
    breakpoint_year = 12000000
    overage_pct = 0.02
  }
}

// Inline shops: single lease with net recoveries (no stop), 30% share.
contract cre.lease_unit.shops on entity asset.strip_center {
  term 2026-07..2031-06
  terms {
    rent_year = 288000
    free_rent_months = 2
    escalation = 0.025
    opex_year = 240000
    opex_escalation = 0.03
    gross_up_factor = 0.95
    pro_rata_share = 0.30
    ti_total = 90000
    lc_total = 40000
  }
}

contract cre.vacancy_loss on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    rate = 0.03
    potential_gross_year = 850000
  }
}

contract cre.opex_line on entity asset.strip_center {
  term 2026-01..2032-12
  terms {
    amount_year = 240000
    growth_rate = 0.03
  }
}

contract cre.exit on entity asset.strip_center {
  term 2032-12..2032-12
  terms {
    income = 640000
    cap_rate = 0.0675
    selling_costs = 0.015
  }
}
```

## cre_stochastic_rollover

```cfdl
version 0.1
model "cre-stochastic-rollover"
time calendar monthly from 2031-01 for 24

entity asset tower : Asset.Real

// The conventional expected-value blend replaced by a BINARY renewal outcome per
// Monte Carlo trial: one uniform draw per trial decides renew vs re-lease,
// so trial results show the true two-cluster distribution (rent level,
// downtime, and turnover cost all consistent within a scenario) instead of
// a single probability-weighted average.
assume renewal_draw ~ Uniform(min=0, max=1)

stream asset.rollover_rent on entity asset.tower inflow currency USD {
  schedule every month from 2031-01 to 2032-12
  amount = if(inputs.renewal_draw < 0.7, 520000 / 12, 560000 / 12)
}

// Re-lease scenario: 3 months of vacancy before the new tenant pays.
stream asset.downtime_loss on entity asset.tower outflow currency USD {
  schedule every month from 2031-01 to 2031-03
  amount = if(inputs.renewal_draw < 0.7, 0, 560000 / 12)
}

stream asset.turnover_costs on entity asset.tower outflow currency USD {
  schedule on 2031-01
  amount = if(inputs.renewal_draw < 0.7, 100000, 350000)
}
```

## cre_subject_implicit_fallback

```cfdl
version 0.1
model "cre-subject-implicit-fallback"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset first_entity : CRE.Asset.RealProperty
entity asset second_entity : CRE.Asset.RealProperty

contract cre.opex_line {
  term 2026-01..2027-12
  terms {
    amount = 12000
  }
}
```

## cre_subject_non_first_entity

```cfdl
version 0.1
model "cre-subject-non-first-entity"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset primary : CRE.Asset.RealProperty
entity asset annex : CRE.Asset.RealProperty

contract cre.revenue_line on entity asset.annex {
  term 2026-01..2027-12
  terms {
    amount = 30000
  }
}
```

## credit_float_smoke

```cfdl
version 0.1
model "credit-float-smoke"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 15

// Linear-interpolated index path: rides down from 5.0% to 3.8% over a year.
curve sofr linear {
  2026-01: 0.050
  2027-01: 0.038
}

entity asset buyer : Credit.Asset.LoanPool

// Small floating IO pool: SOFR + 300, floor 7.25% (binds late), 12-month
// bullet, prepay/default/severity with a 3-month recovery lag.
contract credit.pool_float_io_bullet.smoke on entity asset.buyer {
  term 2026-01..2027-03
  terms {
    principal = 1200000
    index_curve = "sofr"
    margin = 0.03
    rate_floor = 0.0725
    rate_cap = 0.10
    term_months = 12
    cpr = 0.10
    cdr = 0.03
    severity = 0.50
    recovery_lag_months = 3
  }
}

contract credit.purchase.smoke on entity asset.buyer {
  term 2026-01..2026-01
  terms {
    price = 1200000
  }
}
```

## credit_note

```cfdl
version 0.1
model "credit-note"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 15

entity container trust : Container.SPV
entity asset pool : Credit.Asset.LoanPool {
  part of container.trust
}
entity party holders : Credit.Party.Investor { name = "Class A noteholders" }

contract credit.pool_level_pay.smoke on entity asset.pool {
  term 2026-01..2027-03
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 12
    cpr = 0.10
    cdr = 0.03
    severity = 0.50
    recovery_lag_months = 3
  }
}

// A STRUCTURED NOTE is the second refinement of Contract.Security (docs/40
// §4.13), and the opposite shape from the participation: both of its lines
// are allocated, so the contract lowers no cash. It lowers two claims as
// fields on the trust — face less what the holder's principal account had
// received, and the interest due on it — and the priority of payments pays
// them. Each step names the contract and line it pays, so the results
// attribute the allocation to the note and a slice by type and line reaches
// it.
contract credit.note a on entity container.trust {
  term 2026-01..2027-03
  terms {
    face = 1000000
    coupon = 0.05
    principal_account = a_principal
  }
  parties {
    holder = party.holders
  }
}

account interest_collections {
  from series_sum("credit.pool.interest.*", time.t, time.t)
     + series_sum("credit.pool.servicing.*", time.t, time.t)
}

account principal_collections {
  from series_sum("credit.pool.sched_principal.*", time.t, time.t)
     + series_sum("credit.pool.prepay.*", time.t, time.t)
     + series_sum("credit.pool.recoveries.*", time.t, time.t)
}

account a_principal { owner party.holders }
account a_interest { from 0.0 }

waterfall notes.interest on entity container.trust {
  schedule every month from 2026-01 to 2027-03
  from interest_collections
  pay a_interest to account a_interest for contract credit.note.a line interest
        = min(remaining, container.trust.credit_note_interest_due_a)
}

waterfall notes.principal on entity container.trust {
  schedule every month from 2026-01 to 2027-03
  from principal_collections
  pay a_principal to party.holders for contract credit.note.a line principal
        = min(remaining, container.trust.credit_note_claim_a)
}

slice note_principal {
  type Contract.Security
  line principal
}
```

## credit_participation

```cfdl
version 0.1
model "credit-participation"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 15

entity asset pool : Credit.Asset.LoanPool
entity party issuer : Credit.Party.Servicer { name = "Issuer" }
entity party investors : Credit.Party.Investor { name = "Certificate holders" }

contract credit.pool_level_pay.smoke on entity asset.pool {
  term 2026-01..2027-03
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 12
    cpr = 0.10
    cdr = 0.03
    severity = 0.50
    recovery_lag_months = 3
    servicing_fee = 0.005
  }
}

// A PARTICIPATION IS THE FIRST REFINEMENT OF Contract.Security (docs/40
// §4.13): a pro rata interest in the pool's cash, passed through to the holder
// each period — scheduled and unscheduled principal alike, interest net of
// servicing — with nothing ranked and nothing chosen, so both lines are
// lowered from the collateral rather than allocated by a priority of
// payments. It carries the pool's suffix, and at a share of one its two
// streams equal the pool's net collections, which is what this fixture checks.
contract credit.participation smoke on entity asset.pool {
  term 2026-01..2027-03
  terms {
    face = 1200000
    share = 1.0
  }
  parties {
    issuer = party.issuer
    holder = party.investors
  }
}

slice passed_through {
  type Contract.Security
}

slice pool_interest_net {
  type Contract.Debt
  line interest
  line servicing
}
```

## credit_pool_smoke

```cfdl
version 0.1
model "credit-pool-smoke"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 15

entity asset buyer : Credit.Asset.LoanPool

// Small level-pay pool: 12-month amortization, prepay/default/severity with
// a 3-month recovery lag (term spans 12 + 3 months).
contract credit.pool_level_pay.smoke on entity asset.buyer {
  term 2026-01..2027-03
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 12
    cpr = 0.10
    cdr = 0.03
    severity = 0.50
    recovery_lag_months = 3
  }
}

contract credit.purchase.smoke on entity asset.buyer {
  term 2026-01..2026-01
  terms {
    price = 1200000
  }
}
```

## credit_zero_rate_pool

```cfdl
version 0.1
model "credit-zero-rate-pool"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset lender : Credit.Asset.LoanPool

// A 0% APR pool — promotional financing, ordinary in auto and retail credit
// and about 3% of a real published auto-ABS collateral table.
//
// The level-pay closed form is 0/0 at r = 0, so this was unsupported: the
// balance factor S(p) = ((1+r)^n - (1+r)^p)/((1+r)^n - 1) has no value there.
// Expressed as a ratio of annuity present values it does — pv() already
// carries the r = 0 limit — and the whole thing collapses to straight line:
//
//   scheduled principal   1,200,000 / 12 = 100,000 every month
//   interest              0
//   balance factor S(p)   (12 - p) / 12
//
// The `rate` validation only ever required non-negative, so before this a zero
// rate was accepted and produced NaN rather than an answer or a refusal.
contract credit.pool_level_pay on entity asset.lender {
  term 2026-01..2026-12
  terms {
    principal = 1200000
    interest_rate = 0
    term_months = 12
    cpr = 0
    cdr = 0
  }
}
```

## currency_inr

```cfdl
version 0.1
model "currency-inr" currency INR
time calendar monthly from 2026-01 for 12

entity asset plant : Asset.Real

// A model declares the currency it reports in. Every metric is denominated
// in it; streams must agree, since cash flows are summed period by period.
stream plant.revenue on entity asset.plant inflow currency INR {
  schedule every month from 2026-01 to 2026-12
  amount = 1500000
}

stream plant.opex on entity asset.plant outflow currency INR {
  schedule every month from 2026-01 to 2026-12
  amount = 200000
}
```

## declared_metrics

```cfdl
version 0.1
model "declared-metrics"
time calendar annual from 2026-01 for 5

// A MODEL MAY NAME THE FIGURE IT SOLVED FOR.
//
// Metric keys were minted in two places only: the engine (`model.*`) and a
// pack (`domain.*`). A case computing a deal-specific number — a class
// weighted average life on the deal's own axis, a crossover date, an
// overcollateralisation ratio — had nowhere to put it, so the number a case
// exists to check sat unnamed in an `expected.csv` column instead of in
// `expected_metrics.json` beside the published figure it reproduces.
//
// A metric is a fold over the FINISHED projection, evaluated once at the
// horizon in the valuation plane. It never feeds back into the walk.
//
// Metrics compose in declaration order, the rule waterfalls already follow, so
// `margin` reads the two above it rather than repeating their expressions. The
// composition is checkable here: 5,000 + (-2,000) = 3,000, which is exactly
// what the engine publishes as model.total.

entity asset co : Asset.Financial

stream ops.revenue on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  amount = 1000
}

stream ops.cost on entity asset.co outflow currency USD {
  schedule every year from 2026-01 to 2030-01
  amount = 400
}

metric gross_revenue = series_sum("ops.revenue", 0, 4)
metric total_cost    = series_sum("ops.cost", 0, 4)
metric margin        = metric.gross_revenue + metric.total_cost
```

## dscr_cash_trap_cure_period

```cfdl
version 0.1
model "dscr-cash-trap-with-cure-period"
time calendar monthly from 2026-01 for 18

// A PROJECT FINANCE CASH TRAP, WITH THE CURE PERIOD A CREDIT AGREEMENT
// ACTUALLY WRITES (`docs/13` §7.77). Below the trigger, distributions stop
// and cash accumulates in the trap; the trap releases only after the ratio
// has held AT OR ABOVE the trigger for the stated cure period. ONE GOOD
// MONTH IS NOT A CURE, and that is the part `trapped_cash_cure` could not
// say: it cures on the next good period, because a duration measured from
// the last breach had no spelling until arrival actions (§7.79).
//
// `on enter trapped { set good_periods = 0 }` is what makes the cure period
// expressible. A plain recurrence counts consecutive good periods, but it
// has no way to start over at each new breach; the arrival does that.
//
// The pin: NOI drops to 12,000 against 15,000 of debt service at t=4..6, so
// DSCR is 0.80 against a 1.20 trigger. The machine reads SETTLED cash
// strictly backward, so it traps at t=5. Cash accumulates once NOI recovers
// (5,000 at t=7, 10,000 at t=8) because the trapped months have nothing to
// distribute. Two consecutive good periods land at t=9 and the trap
// releases 10,000 in full.
entity asset plant {
  lifecycle covenant
  // Consecutive periods at or above the trigger, on SETTLED cash. Reset to
  // zero on every entry into `trapped`, so each breach starts its own clock.
  good_periods init 0.0 next if(
    series_sum("ops.noi", time.t - 1, time.t - 1) >= 1.20 * (0.0 - series_sum("debt.service", time.t - 1, time.t - 1)),
    prev + 1.0,
    0.0
  )
}

entity party sponsor { name = "Sponsor" }

account trap { }

lifecycle covenant {
  initial compliant
  state compliant, trapped

  on enter trapped { set good_periods = 0 }

  compliant -> trapped when
    series_sum("ops.noi", time.t - 1, time.t - 1) < 1.20 * (0.0 - series_sum("debt.service", time.t - 1, time.t - 1))

  // The cure is the ratio held for two consecutive periods, not one.
  trapped -> compliant when asset.plant.good_periods >= 2
}

stream ops.noi on entity asset.plant inflow currency USD {
  schedule every month from 2026-01 to 2027-06
  amount = if(time.t >= 4 and time.t <= 6, 12000, 20000)
}

stream debt.service on entity asset.plant outflow currency USD {
  schedule every month from 2026-01 to 2027-06
  amount = 15000
}

waterfall distribution on entity asset.plant {
  schedule every month from 2026-01 to 2027-06
  from available
  pay trapped   to account trap    = if(asset.plant.status == "trapped", remaining, 0.0)
  pay residual  to party.sponsor   = remaining
}

// The release: once the covenant is cured, the accumulated trap balance is
// handed back. A second waterfall drawing FROM the account, because the trap
// is a location cash sits in, not a step in the distribution.
waterfall release on entity asset.plant {
  schedule every month from 2026-01 to 2027-06
  from trap
  pay released to party.sponsor = if(asset.plant.status == "compliant", remaining, 0.0)
}
```

## dscr_smoke

```cfdl
version 0.1
model "dscr-smoke"
time calendar monthly from 2026-01 for 24

entity asset prop : Asset.Real
entity asset permanent : Asset.Financial

// Revenue: 30,000/month inflow  (24 periods → total +720,000)
stream cre.ops.revenue on entity asset.prop inflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 30000
}

// Operating expense: 10,000/month outflow  (24 periods → total -240,000)
stream cre.opex.line on entity asset.prop outflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 10000
}

// Debt service: 15,000/month outflow  (24 periods → total -360,000)
// NOI = 720,000 + (-240,000) = 480,000
// DSCR = 480,000 / 360,000 = 1.333333
stream loan.permanent_debt_service on entity asset.permanent outflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 15000
}
```

## energy_construction_cod

```cfdl
version 0.1
model "energy-construction-cod"
time calendar monthly from 2026-01 for 48

entity asset microgrid : Energy.Asset.GenerationFacility

// Construction: 12 months of capex draws before COD. No revenue.
phase construction from 2026-01 to 2026-12
phase operations from 2027-01 to 2029-12

use pack "energy" version "0.1.0"

contract energy.capex on entity asset.microgrid {
  term 2026-01..2026-01
  terms { amount = 500000 }
}

contract energy.capex.final_draw on entity asset.microgrid {
  term 2026-12..2026-12
  terms { amount = 700000 }
}

// PPA begins at COD (contract term_start = 2027-01). Degradation and
// escalation anniversaries count from COD, not from model start.
contract energy.ppa on entity asset.microgrid {
  term 2027-01..2029-12
  terms {
    quantity = 2400
    price = 90
    escalation = 0.02
    degradation = 0.01
  }
}

contract energy.om on entity asset.microgrid {
  term 2027-01..2029-12
  terms {
    fee_year = 36000
    escalation = 0.02
  }
}
```

## energy_solar_ppa

```cfdl
version 0.1
model "solar-ppa-microgrid"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 300

entity asset microgrid : Energy.Asset.GenerationFacility

// 2 MW solar + storage microgrid, 25-year PPA.
contract energy.capex on entity asset.microgrid {
  term 2026-01..2026-01
  terms { amount = 2400000 }
}

contract energy.itc on entity asset.microgrid {
  term 2026-12..2026-12
  terms { amount = 720000 }
}

contract energy.ppa on entity asset.microgrid {
  term 2026-01..2050-12
  terms {
    quantity = 4200
    price = 85
    escalation = 0.02
    degradation = 0.005
  }
}

contract energy.storage_arbitrage on entity asset.microgrid {
  term 2026-01..2050-12
  terms {
    quantity = 500
    price = 30
  }
}

contract energy.capacity on entity asset.microgrid {
  term 2026-01..2050-12
  terms { price = 60000 }
}

contract energy.om on entity asset.microgrid {
  term 2026-01..2050-12
  terms {
    fee_year = 70000
    escalation = 0.025
  }
}

contract energy.debt_service on entity asset.microgrid {
  term 2026-01..2045-12
  terms {
    interest_rate = 0.06
    term_months = 240
    principal = 1600000
  }
}
```

## energy_wind_ptc

```cfdl
version 0.1
model "wind-ptc-macrs"
use pack "energy" version "0.1.0"
time calendar monthly from 2027-01 for 240

entity asset windfarm : Energy.Asset.GenerationFacility

// 30 MW wind: merchant revenue with availability, 10-year PTC,
// 5-year MACRS shield, level-pay debt.
contract energy.capex on entity asset.windfarm {
  term 2027-01..2027-01
  terms { amount = 42000000 }
}

contract energy.merchant on entity asset.windfarm {
  term 2027-01..2046-12
  terms {
    quantity = 105000
    price = 42
    escalation = 0.015
    degradation = 0.007
    availability = 0.95
  }
}

contract energy.ptc on entity asset.windfarm {
  term 2027-01..2036-12
  terms {
    quantity = 105000
    amount = 27.5
    escalation = 0.02
    degradation = 0.007
    availability = 0.95
  }
}

contract energy.macrs_shield on entity asset.windfarm {
  term 2027-01..2046-12
  terms {
    basis = 42000000
    tax_rate = 0.21
    life = 5
  }
}

contract energy.om on entity asset.windfarm {
  term 2027-01..2046-12
  terms {
    fee_year = 1300000
    escalation = 0.02
  }
}

contract energy.debt_service on entity asset.windfarm {
  term 2027-01..2041-12
  terms {
    interest_rate = 0.055
    term_months = 180
    principal = 25000000
  }
}
```

## entity_attributes_read

```cfdl
version 0.1
model "entity-attributes-read"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 3

// A DECLARED ATTRIBUTE IS A PROPERTY OF THE THING, and a model can read it.
//
// These were parsed, checked against the ontology — misspell one and E1313
// rejects it — and carried into the IR, and then the engine deserialised past
// them. `entity.<field>` was 0 in every expression that touched
// it, so a rent struck per square foot came out as nothing at all.
entity asset tower : CRE.Asset.RealProperty {
  asset_class    = "office"
  rentable_area  = 30000
}

entity asset suite : CRE.Asset.Unit {
  rentable_area = 12000
  part of asset.tower
}

// Numeric where it looks numeric, so arithmetic works.
stream cre.rent.tower on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = asset.tower.rentable_area * 2.50
}

// And per entity: the suite reads its own area, not its parent's.
stream cre.rent.suite on entity asset.suite inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = asset.suite.rentable_area * 3.00
}

// A string attribute stays a string.
stream cre.opex on entity asset.tower outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.expense.opex
  amount = if(asset.tower.asset_class == "office", 1000.0, 500.0)
}
```

## entity_field_rule

```cfdl
version 0.1
model "entity-field-rule"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 4

// A FIELD THAT MOVES, OWNED BY THE THING IT DESCRIBES.
//
// `balance` is a fact about this tranche, so it is declared on the tranche and
// read as `asset.tlb.balance` — the same spelling a stated field answers to. A
// reader does not have to know whether a field holds or moves to know how to
// read it.
//
// A model-level `state tlb_balance` said only that a variable existed. This
// says what the number IS, which is what lets the ontology check it, what lets
// a parent aggregate it by relation, and what a waterfall means when it draws
// `from asset.trust.available_funds`.
//
// The rule is the same recurrence a state carries and is evaluated in the same
// pass — one place a recurrence is solved, one set of rules about what it can
// see.

entity asset tlb : Credit.Asset.Tranche {
  seniority = 1
  original_balance = 275.0
  balance init 275.0 next max(0.0, prev - 25.0)
}

entity party lender : Party { name = "Term lender" }

stream credit.repayment on entity asset.tlb outflow currency USD {
  schedule every year from 2026-01 to 2029-01
  category operating.collection.principal
  amount = 25.0
}

// The stream reads the field by name, and gets the period's computed value.
stream credit.balance_probe on entity asset.tlb inflow currency USD {
  schedule every year from 2026-01 to 2029-01
  category operating.collection.principal
  amount = asset.tlb.balance
}
```

## entity_property_bare_path

```cfdl
version 0.1
model "entity-property-bare-path"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 2

// AN ENTITY'S PROPERTY, NAMED DIRECTLY.
//
// `asset.class_a.original_balance` and `entity.asset.class_a.original_balance`
// are the same read. Properties are bound under their family, so the bare form
// is an alias rather than a second mechanism, and only a declared family —
// asset, party, contract, reference — is aliased.
//
// It matters most in a waterfall, where a step names the thing it is paying
// and then names what that thing is owed. Before the alias the bare spelling
// resolved to nothing, and because the entity root is open-world it returned
// null and paid ZERO rather than failing — in the one construct whose job is
// deciding who gets paid. Every waterfall example in the documentation was
// written that way.

entity asset trust   : Credit.Asset.LoanPool { collateral_type = "auto" }
entity asset class_a : Credit.Asset.Tranche  { original_balance = 5000.0 seniority = 1 }
entity party residual : Party { name = "Residual holder" }

waterfall abs.distribution on entity asset.trust {
  schedule on 2026-01
  from 10000.0

  // The bare form and the long form, side by side, on the same fact.
  pay senior_bare to asset.class_a  = asset.class_a.original_balance / 2.0
  pay senior_long to asset.class_a  = entity.asset.class_a.original_balance / 2.0
  pay rest        to party.residual = remaining
}
```

## evaluation_order

```cfdl
version 0.1
model "evaluation-order"
use pack "credit" version "0.1.0"
time calendar monthly from 2017-01 for 4

// WHAT EACH LAYER CAN SEE, pinned as a test rather than asserted in a document.
//
// The engine evaluates in layers, each complete before the next begins:
//
//   1. fields      — every field, every period
//   2. events      — every period, reading fields; writes are visible later
//   3. streams     — in dependency-ordered waves (readers after what they read)
//   4. subtotals   — folds of stream categories, for statements
//   5. waterfalls  — read fields, event writes, streams, earlier waterfalls
//
// A layer sees what finished before it and nothing after, and this fixture is
// four readings that show where the boundaries fall:
//
//   `reads_fee_rate`  — an event fires on a FIELD crossing a threshold, and
//                       the waterfall sees the write: 20000 for two periods,
//                       then 50000.
//   `reads_cash_flag` — an event guard reading CASH never fires, because no
//                       stream exists when events are simulated. Zero
//                       throughout, with a warning naming the series.
//   `domain.credit.*` — subtotals carry the pool's collections and none of the
//                       waterfall's payments: subtotals are folded before the
//                       waterfall runs, so a distribution never reaches a
//                       statement.
//   `dist.from`       — a waterfall CAN read a stream, which is the one
//                       direction across the boundary that works.
//
// See `docs/03_expression_environment.md` §3.1 and §3.2 for the rules, and
// `docs/13_feature_backlog.md` §7.37 for what the ordering costs.

entity asset pool : Credit.Asset.LoanPool {
  collateral_type = "auto"
  trigger_level = 700000.0
  cash_trigger = 200000.0

  // A field moves on its own values, which is the one recurrence that works.
  running init 1000000.0 next max(0.0, prev - 150000.0)
  fee_rate init 0.02
  cash_flag init 0.0
}

entity party investor : Credit.Party.Investor { name = "Investor" }

contract credit.pool_level_pay.one on entity asset.pool {
  term 2017-01..2017-04
  terms { principal = 1000000 interest_rate = 0.12 term_months = 4 cpr = 0 cdr = 0 }
}

// Fires at period 2, when the field reaches the level.
event field_threshold when asset.pool.running <= asset.pool.trigger_level {
  set entity asset.pool.fee_rate = 0.05
}

// A GUARD READING A STREAM used to live here, and never fired: events are
// simulated before any stream is evaluated, so the read bound nothing and the
// comparison was false in every period. It is now refused
// (`E1134_SERIES_READ_IN_LOGIC`, pinned by `invalid/series_read_in_guard`)
// rather than pinned as behavior — a limitation documented by a fixture that
// runs clean is a limitation a model author meets by getting wrong numbers.
//
// `cash_flag` stays at its `init` of 0.0, which is what it did with the event
// present, so this fixture's numbers are unchanged and `reads_cash_flag` still
// pays nothing. When the period walk lands (`docs/28` §4), the SETTLED form of
// that read — at or before the previous period — becomes legal, and the
// same-period form above stays refused.

waterfall dist on entity asset.pool {
  schedule every month from 2017-01 to 2017-04
  // The pot is the entity's available cash, bound by the engine — the
  // boundary crossing the language provides for.
  from available

  pay reads_fee_rate  to party.investor = asset.pool.fee_rate * 1000000.0
  pay reads_cash_flag to party.investor = asset.pool.cash_flag * 111.0
  pay residual        to party.investor = remaining
}
```

## event_guard_reads_field

```cfdl
version 0.1
model "event-guard-reads-field"
time calendar annual from 2026-01 for 5

// AN EVENT GUARD READS A FIELD, and a rule may read a constant one.
//
// Both were broken and neither failed loudly. `bind_states` bound an entity's
// field values, then `bind_all_entity_state` REPLACED that entity's map with
// one holding only lifecycle status — so a guard reading a field found a key
// that was not there. Under the open-world entity root that is null, the guard
// compared null to a number, and the event never fired.
//
// ONE THING DELIBERATELY NOT PROVEN HERE, recorded rather than blessed:
//
//   * A RULE READING A LITERAL FIELD. The validator now allows it — a constant
//     is readable at any period — but the engine's rule environment does not
//     bind literals, so it resolves to nothing.
//
// A STREAM READING AN EVENT-WRITTEN FIELD is now the same read as anyone
// else's: the write settles the field store itself, so the published series,
// every reader, and next period's `prev` agree. One value per path.

entity asset pool : Asset.Financial {
  // Stated facts. Constants, so a rule may read them.
  trigger_level = 60.0

  balance init 100.0 next max(0.0, prev - 10.0)
  fee_rate init 0.02
}

entity party lender : Party { name = "Lender" }

// The guard reads a field that moves; the action writes one that does not.
event covenant_breach when asset.pool.balance <= asset.pool.trigger_level {
  set entity asset.pool.fee_rate = 0.05
}

// The balance is a field a rule moves, read by a stream at its own name.
stream credit.amortization on entity asset.pool outflow currency USD {
  schedule every year from 2027-01 to 2030-01
  amount = asset.pool.balance * 0.02
}
```

## event_refires_on_each_occurrence

```cfdl
version 0.1
model "event-refires-on-each-occurrence"
time calendar monthly from 2026-01 for 10

// AN EVENT IS SOMETHING THAT HAPPENS, and a unit that defaults, cures and
// defaults again has had three events. Under the latch this fired once and
// the second breach was invisible; it now fires on each rising edge, and the
// journal carries one row per occurrence.
//
// `breached` is a marker the event writes, not a state: what makes the
// condition RE-RISE is `paying` falling, recovering and falling again.
entity asset suite {
  paying init 1.0 next if(time.t == 3 or time.t == 7, 0.0, 1.0)
  breach_count init 0.0 next prev
}

event breach when asset.suite.paying < 0.5 {
  set entity asset.suite.breach_count = prev.asset.suite.breach_count + 1.0
}

stream core.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-10
  amount = 100 * asset.suite.paying
}
```

## event_reseeds_recurrence

```cfdl
version 0.1
model "event-reseeds-recurrence"
time calendar annual from 2026-01 for 6

// ONE VALUE PER PATH, and the law of motion resumes from an intervention.
//
// A loan amortizes 100 a year. In year 2 an unscheduled principal payment of
// 250 arrives — a partial liquidation. Standard finance: the next period
// amortizes FROM THE REDUCED BALANCE, not the contractual schedule.
//
//   contractual   1000, 900, 800, 700, 600, 500
//   settled       1000, 900, 550, 450, 350, 250
//                             ^ event overwrites; recurrence resumes
//
// The published series, every reader, and next period's `prev` all see the
// settled value. There is no second store to go stale.

entity asset loan : Asset.Financial {
  balance init 1000.0 next max(prev - 100.0, 0.0)
}
entity party lender : Party { name = "Lender" }

// The guard reads the period's candidate; the write settles the column.
event partial_liquidation when time.t == 2.0 {
  set entity asset.loan.balance = asset.loan.balance - 250.0
}

// A stream reading the balance sees the settled value in the write period and
// the resumed amortization after it.
stream loan.balance_report on entity asset.loan inflow currency USD {
  schedule every year from 2026-01 to 2031-01
  amount = asset.loan.balance
}
```

## event_scheduled_occurrences

```cfdl
version 0.1
model "event-scheduled-occurrences"
time calendar monthly from 2026-01 for 12

// A schedule SUPPLIES the occurrences and `when` FILTERS them: a quarterly
// covenant test is four tests a year, and four consecutive failures are four
// breach events, because the model declared quarterly testing. The schedule
// is the same sub-language a stream's is, so `every quarter` tests at the
// quarter's close.
entity asset plant {
  covered init 1.0 next if(time.t >= 4, 0.0, 1.0)
  tests_failed init 0.0 next prev
}

event covenant_test schedule every quarter from 2026-01 to 2026-12 when asset.plant.covered < 0.5 {
  set entity asset.plant.tests_failed = prev.asset.plant.tests_failed + 1.0
}

stream core.revenue on entity asset.plant inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 100
}
```

## event_stops_lowered_stream

```cfdl
version 0.1
model "event-stops-lowered-stream"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 60

// A MODEL MAY NAME THE STREAMS ITS OWN CONTRACTS PRODUCE.
//
// `cre.permanent_debt` lowers into three streams — proceeds, interest and
// principal — and the IR carries them under the names used below. Those names
// were unaddressable: the symbol table is built before the pack is chosen, so
// a contract's streams did not exist when the reference was checked, and
// `deactivate stream cre.debt.principal` read as a misspelling. A loan repaid
// early kept taking debt service, and the same model expressed the stop
// correctly the moment the pack was dropped.
//
// The prepayment below is the plain case: at period 36 the borrower repays,
// and the loan's cash stops. Debt service runs 27,500.00 through the
// interest-only months, 36,845.249537 while it amortizes, and 0.00 from
// period 36 — the period the event fires.
//
// The stop is a modeling decision and is stated in the model. It does not
// belong in the contract, which records what was agreed, nor in the lowering
// rule, which would have to guess the vocabulary a modeller will use.

entity asset tower : CRE.Asset.RealProperty

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2030-12
  terms {
    principal = 6000000
    interest_rate = 0.055
    amortization_months = 300
    interest_only_months = 24
    balloon_at_maturity = 1
  }
}

event prepaid when time.t >= 36 {
  deactivate stream cre.debt.principal
  deactivate stream cre.debt.interest
}
```

## events_options_smoke

```cfdl
version 0.1
model "events-options-smoke"
time calendar monthly from 2026-01 for 12

entity asset senior : Asset.Financial

// Debt service runs until the refinance event fires at t=6, which both
// deactivates the stream and flips entity state (belt and braces: the
// active_when clause would stop it too).
stream loan.debt_service on entity asset.senior outflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 500
  active when entity.status != "refinanced"
}

event refi_trigger when time.t >= 6 {
  set entity asset.senior.status = "refinanced"
  deactivate stream loan.debt_service
  exercise option refi_savings
}

option refi_savings type Option.Refinance {
  exercise when false
  payoff 10000 - 250
}
```

## exit_cap_smoke

```cfdl
version 0.1
model "exit-cap-smoke"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset property : CRE.Asset.RealProperty

// Exit via cap rate: terminal value = noi_value / exit_cap = 100000 / 0.05 = 2000000
contract cre.exit_cap on entity asset.property {
  term 2026-12..2026-12
  terms {
    cap_rate = 0.05
    income = 100000
  }
}
```

## expr_smoke

```cfdl
version 0.1
model "expr-smoke"
time calendar monthly from 2026-01 for 4

entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower {
  schedule every month from 2026-01 to 2026-04
  amount = cfg.base + time.t * 10
  active when time.t < 3
}
```

## flip_monthly_grain

```cfdl
version 0.1
model "flip-monthly-grain"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 312

// THE SAME PARTNERSHIP FLIP, TESTED MONTHLY.
//
// `benchmarks/energy/tax_equity_flip` runs this deal on an annual grid and
// reproduces its reference exactly: the tax investor's after-tax return clears
// its 8% hurdle during year 3, and the sharing moves from 98/2 to 5/95.
//
// This is the same deal, the same lifecycle and the same test, with one line
// changed — the calendar. Nothing else about the economics moves.
//
// The flip lands TEN MONTHS EARLIER, in the second month of year 3.
//
// It is not a rounding difference. By the end of year 2 the investor is
// $445,000 short of its hurdle, and two months of operating cash clear it. An
// annual grid has no period between month 24 and month 36 in which to notice,
// so the event cannot fire until the next year end — and the investor keeps
// 98% of the cash for ten months it was no longer entitled to. On this deal
// that is about $3.5m.
//
// SO THE GRID IS AN ECONOMIC ASSUMPTION, not a presentation choice, whenever
// an event decides who gets paid. A model that states its flip date hides
// that; one that derives it cannot.
//
// WHAT IS DELIBERATELY NOT MONTHLY: the credit and the tax benefits. Those are
// realized on a return, once a year — spreading them monthly would move the
// flip earlier for a reason that is not real. Only the operating cash sits on
// the monthly grid, which is where it actually sits.

entity asset interest : Energy.Asset.ProjectInterest {
  interest_type = "tax_equity"
  state pre_flip

  month_of_year init 0.0
                next if(prev >= 12.0, 1.0, prev + 1.0)
  operating_year init 1.0
                 next if(prev.asset.interest.month_of_year == 12.0, prev + 1.0, prev)
  investor_npv_closed init 0.0 - inputs.investor_equity
                      next prev
       + if(time.t >= 2.0 and prev < 0.0,
            inputs.preflip_share
             * ( ( inputs.energy_year_one * inputs.ppa_price
                    * pow(1.0 + inputs.ppa_escalation, prev.asset.interest.operating_year - 1.0)
                    * pow(1.0 - inputs.degradation, prev.asset.interest.operating_year - 1.0)
                   - inputs.capacity_kw * inputs.om_per_kw
                    * pow(1.0 + inputs.om_escalation, prev.asset.interest.operating_year - 1.0)
                   - if(prev.asset.interest.operating_year <= inputs.debt_term,
                        0.0 - pmt(inputs.debt_rate, inputs.debt_term, inputs.debt_amount),
                        0.0) ) / 12.0
                 + if(prev.asset.interest.month_of_year == 12.0,
                      0.0 - inputs.tax_rate
                       * ( inputs.energy_year_one * inputs.ppa_price
                            * pow(1.0 + inputs.ppa_escalation, prev.asset.interest.operating_year - 1.0)
                            * pow(1.0 - inputs.degradation, prev.asset.interest.operating_year - 1.0)
                           - inputs.capacity_kw * inputs.om_per_kw
                            * pow(1.0 + inputs.om_escalation, prev.asset.interest.operating_year - 1.0)
                           + if(prev.asset.interest.operating_year <= inputs.debt_term,
                                ipmt(inputs.debt_rate, prev.asset.interest.operating_year,
                                     inputs.debt_term, inputs.debt_amount),
                                0.0)
                           - macrs_rate(prev.asset.interest.operating_year - 1.0, 5)
                             * (inputs.installed_cost
                                - 0.5 * inputs.itc_rate * inputs.installed_cost) )
                      + if(prev.asset.interest.operating_year == 1.0,
                           inputs.itc_rate * inputs.installed_cost, 0.0),
                      0.0) )
             / pow(1.0 + inputs.hurdle, (time.t - 1.0) / 12.0),
            0.0)
}

entity party sponsor      : Party { name = "Sponsor" }
entity party tax_investor : Party { name = "Tax investor" }

assume energy_year_one = 250000000.0
assume ppa_price       = 0.045
assume ppa_escalation  = 0.02
assume degradation     = 0.005

assume capacity_kw     = 100000.0
assume om_per_kw       = 15.0
assume om_escalation   = 0.02

assume debt_amount     = 60000000.0
assume debt_rate       = 0.06
assume debt_term       = 18.0

assume installed_cost  = 103100000.0
assume itc_rate        = 0.30
assume tax_rate        = 0.21

assume preflip_share   = 0.98
assume postflip_share  = 0.05

assume hurdle          = 0.08
assume investor_equity = 42238000.0

// The calendar has no integer division, so the model counts. `month_of_year`
// cycles 1..12 and `operating_year` steps when it wraps.


// THE TEST, monthly. Identical in shape to the annual model's: the investor's
// discounted after-tax position through the period before this one, stopping
// the moment it turns non-negative.
//
// Operating cash is a twelfth of its year. The credit and the tax benefit land
// whole, in the month the year closes.

event flip when asset.interest.investor_npv_closed >= 0.0 {
  set entity asset.interest.status = "post_flip"
}

// Operating cash is ACTIVITY, so it is a stream; the waterfall allocates the
// interest's available cash rather than recomputing revenue inside the pot.
// A NET figure — PPA revenue less O&M — carried on one stream, so it is
// categorised as energy revenue and EBITDA reads it whole.
stream interest.operating_cash on entity asset.interest inflow currency USD {
  schedule every month from 2026-02 to 2051-12
  category operating.revenue.energy
  amount = ( inputs.energy_year_one * inputs.ppa_price
              * pow(1.0 + inputs.ppa_escalation, asset.interest.operating_year - 1.0)
              * pow(1.0 - inputs.degradation, asset.interest.operating_year - 1.0)
             - inputs.capacity_kw * inputs.om_per_kw
              * pow(1.0 + inputs.om_escalation, asset.interest.operating_year - 1.0)
             - if(asset.interest.operating_year <= inputs.debt_term,
                  0.0 - pmt(inputs.debt_rate, inputs.debt_term, inputs.debt_amount),
                  0.0) ) / 12.0
}

waterfall interest.distribution on entity asset.interest {
  schedule every month from 2026-02 to 2051-12
  from available

  pay investor to party.tax_investor =
        remaining * if(asset.interest.status == "post_flip",
                       inputs.postflip_share,
                       inputs.preflip_share)
  pay sponsor  to party.sponsor = remaining
}
```

## hierarchy_rollup

```cfdl
version 0.1
model "hierarchy-rollup"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 3

entity asset tower : CRE.Asset.RealProperty { asset_class = "office" }
entity asset suite_a : CRE.Asset.Unit { rentable_area = 10000  part of asset.tower }
entity asset suite_b : CRE.Asset.Unit { rentable_area = 5000   part of asset.tower }

// Rent sits on the SUITES. The building's cash is its suites' cash because
// they are its suites — not because the stream names share a prefix.
stream cre.rent.suite_a on entity asset.suite_a inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 100
}

stream cre.rent.suite_b on entity asset.suite_b inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 50
}

// A building-level cost sits on the building itself.
stream cre.opex.building on entity asset.tower outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.expense.opex
  amount = 30
}
```

## inherited_field

```cfdl
version 0.1
model "inherited-field"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 2

// A FIELD LEARNED FROM THE MASTER HOLDS ON EVERYTHING THAT IS ONE.
//
// `year_built` is declared by CRE.Asset.RealProperty. A Unit refines
// RealProperty (docs/13 §7.92), so a unit carries the field without the
// Unit type restating it — and a reader who learned the field from the
// master is not lied to by the refinement. `rentable_area` shows the other
// direction: the master declares it optional, the Unit strengthens it to
// required, and this model must state it.

entity asset suite : CRE.Asset.Unit {
  rentable_area = 1200.0
  year_built = 1987.0
}

stream unit.age_probe on entity asset.suite inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  category operating.revenue.other
  amount = 2026.0 - asset.suite.year_built
}
```

## instance_categories

```cfdl
version 0.1
model "instance-categories"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 2

// A PACK STATES WHAT ITS CONTRACTS ARE. IT CANNOT STATE EVERY LEAF.
//
// `cre.opex_line` lowers to `operating.expense.opex` — one category however
// many instances a model declares, which is right for a pack that cannot know
// the deal. A hotel's statement is departmental, so these three instances say
// what they are and the pack's default gives way.
//
// The three roots are the only gate, so `operating.expense.rooms` is valid
// without the pack listing it. It is not the pack's conventional spelling, so
// the run reports W5023 naming it. Both facts are the point of this fixture.

entity asset hotel : CRE.Asset.RealProperty

contract cre.opex_line.rooms on entity asset.hotel {
  term 2026-01..2027-01
  category operating.expense.rooms
  terms {
    amount = 1000
  }
}

contract cre.opex_line.food_beverage on entity asset.hotel {
  term 2026-01..2027-01
  category operating.expense.food_beverage
  terms {
    amount = 400
  }
}

// No override: takes the rule's `operating.expense.opex`, which is what the
// pack's own vocabulary recommends.
contract cre.opex_line.property_tax on entity asset.hotel {
  term 2026-01..2027-01
  terms {
    amount = 200
  }
}
```

## journal_action_outcomes

```cfdl
version 0.1
model "journal-action-outcomes"
time calendar monthly from 2026-01 for 12

// WHAT DID THE MODEL DO, AND DID EACH THING HAPPEN.
//
// `deterministic.transitions` records field CHANGES. An action that was
// declined, ignored, or done and then overridden changes nothing, so it
// appeared nowhere — and the worst of those was silent. This fixture pins one
// row of each outcome the journal can report (`docs/28` §8).
//
// The case that motivated it: an event activates a stream whose own
// `active when` is false. Both gates must pass, so the activation does not
// turn the stream on — measured before the journal existed, the modeller got a
// zero series, no warning, and nothing in the results saying an activation had
// been refused.

phase early from 2026-01 to 2026-06
phase late  from 2026-07 to 2026-12

entity asset bldg : Asset.Financial {
  expanded init 0.0
}

// APPLIED: a field write, which `transitions` also records.
// APPLIED: a stream deactivation, which takes effect.
// OVERRIDDEN: an activation against a stream that declares itself off.
// DECLINED: an option forced outside its exercisable window.
//
// The sixth outcome, IGNORED, cannot be reached from a model at all. It is
// what the engine journals for an action KIND it does not know, and every kind
// a model can write is known. Only hand-written IR can carry one — IR that
// still spells `DeactivateContract`, retired from the language with the action
// itself (backlog 7.73), is the case in hand. An engine unit test covers it.
event expand when time.t >= 6 {
  set entity asset.bldg.expanded = 1.0
  deactivate stream ops.baseline
  activate stream capex.floor2
  exercise option renewal
}

// Turns on when the event writes the field — the ordinary path, and the
// contrast that makes the overridden row meaningful.
stream rent.floor2 on entity asset.bldg inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 5000
  active when asset.bldg.expanded >= 1.0
}

// Declares itself off for the whole run. The event's activation cannot
// override that, and the journal says so once, with a count.
stream capex.floor2 on entity asset.bldg outflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 20000
  active when 0 > 1
}

// Runs until the event deactivates it.
stream ops.baseline on entity asset.bldg inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 1000
}

// Exercisable only in `early`, and the event fires in `late` — so the forced
// exercise is declined rather than performed.
option renewal on entity asset.bldg type Option.Renewal exercisable in early {
  exercise when 0 > 1
  payoff 250
}
```

## lifecycle_after_assume

```cfdl
version 0.1
model "lifecycle-after-assume"
time calendar monthly from 2026-01 for 4

// A LIFECYCLE DECLARED AFTER AN ASSUMPTION — the second half of the swallow
// `account_after_assume` records.
//
// `lifecycle` is CONTEXTUAL: an ordinary identifier that starts a top-level
// statement, deliberately, so that declaring one costs no new reserved word
// (`docs/13` §7.19). `is_statement_start` matched on keywords only, so it
// could not see this one either, and an assumption's expression scan ran
// through the declaration below.
//
// This half was never silent — the scan stopped at the `{` and the next
// parse reported `E3001_EXPR_PARSE_ERROR: unexpected character '{'`, which
// names a brace three lines away from the assumption that actually swallowed
// the statement. Loud and misdirecting rather than silent and absent; the
// same bug, and the same fix.

assume threshold = 2.0

lifecycle unit {
  initial vacant
  state vacant, leased
  vacant -> leased when time.t >= inputs.threshold
}

entity asset suite {
  lifecycle unit
}

stream core.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-04
  amount = 100
}
```

## lifecycle_breach_and_cure

```cfdl
version 0.1
model "lifecycle-breach-and-cure"
time calendar monthly from 2026-01 for 12

// Rent flows in leased, stops when the tenant defaults (an event drops
// collections), the machine sees the shortfall in settled cash and moves to
// delinquent; collections resume, the cure edge sees them, and the unit is
// current again — twice.
lifecycle unit {
  initial leased
  state leased, delinquent
  leased -> delinquent when series_sum("core.rent", time.t - 1, time.t - 1) < 50
  delinquent -> leased when series_sum("core.rent", time.t - 1, time.t - 1) >= 50
}

entity asset suite {
  lifecycle unit
  paying init 1.0 next if(time.t == 3 or time.t == 8, 0.0, 1.0)
}

stream core.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 100 * asset.suite.paying
}
```

## logic_reads_settled_cash

```cfdl
version 0.1
model "logic-reads-settled-cash"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// A UNIT GOES DELINQUENT WHEN LAST PERIOD'S RENT CAME IN SHORT.
//
// The case this whole reorder exists for, and for most of the language's life
// it could not be written. Events settled over the whole timeline before any
// stream had a value, so a guard reading cash bound nothing: the engine warned
// once per period, substituted `false`, and published a run in which the event
// simply never fired (`docs/13` §7.71).
//
// Under the period walk (`docs/28` §3) a period's state settles with every
// earlier period's cash already in the store, so the guard reads history that
// has actually happened. Rent stops after June; the guard sees July's shortfall
// when it settles August, and the unit moves to downtime — which the rent
// stream's own `active in state` then keeps off.
//
// STRICTLY BACKWARD, and that is what keeps it sound. `time.t - 1` is settled;
// `time.t` is not, because logic settles before this period's cash exists. At
// period 0 the window is empty and reads nothing, which is the right answer:
// no rent has been received before the model began.

entity asset suite : CRE.Asset.Unit {
  rentable_area = 1000
  state leased
}

stream cre.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  category operating.revenue.base_rent
  amount = 100
  active in state leased
}

event delinquent when series_sum("cre.rent", time.t - 1, time.t - 1) < 50 {
  set entity asset.suite.status = "vacant"
}
```

## metric_folds_slice

```cfdl
version 0.1
model "metric-folds-slice"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2027-12
  terms {
    principal = 3000000
    interest_rate = 0.06
    amortization_months = 300
  }
}

slice debt_interest {
  type Contract.Debt
  line interest
}

// A METRIC FOLDS A SLICE (docs/13 §7.90: a named selection that functions
// consume). `slice.debt_interest` is the slice's net, so the figure is
// selected by type and line rather than by naming a stream or a pack's
// category — the cross-pack reading of docs/40 stage 5.
metric interest_paid = -series_sum("slice.debt_interest", 0, 23)
```

## metric_reads_published_results

```cfdl
version 0.1
model "metric-reads-published-results"
time calendar annual from 2026-01 for 3

// A METRIC CAN READ WHAT THE VALUATION PLANE PUBLISHES — INCLUDING ITS OWN
// AGGREGATES.
//
// `docs/13` §7.85. `docs/01` §15.3 has said in normative text since metrics
// entered the spec that a metric's expression MAY read series, ENTITY FIELDS,
// `inputs`, `cfg` and the engine's `model.*`. Entity fields it could not read:
// the metric environment was built from stream and waterfall series and
// nothing else, and `bind_states` — called for streams, distributions and
// state evaluation alike — was never called for it.
//
// The same absence hid every computed aggregate. `entity.<symbol>.net_cash_flow`
// is computed, published, and was unreachable; so was a field's own series, an
// account balance, and a pack's money subtotal. Each read as ZERO, in silence,
// because `series_sum` returns 0.0 for a selector that matches nothing.
//
// Worse, the two dialects disagreed about the same cash: `ops.rev` read 300
// while `stream.ops.rev` — the key the results document publishes — read 0.
// Both spellings now name the same series. The metric environment is the only
// place this binding is added: a pot's window and a guard's read are untouched,
// because a metric reads the FINISHED projection and they read the walk.

entity asset proj : Asset.Financial {
  drawn init 10000.0 next prev + 1000.0
}

stream ops.rev on entity asset.proj inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 100
}

// An entity field, both spellings, read at the HORIZON: 10,000 then two
// increments of 1,000.
metric field_bare  = asset.proj.drawn
metric field_long  = entity.asset.proj.drawn

// The same field as a SERIES, under the key results publishes it under.
metric field_at_t2 = series_sum("asset.proj.drawn", 2, 2)

// The two dialects for one stream's cash. These must agree.
metric bare        = series_sum("ops.rev", 0, 2)
metric qualified   = series_sum("stream.ops.rev", 0, 2)
metric agree       = metric.bare - metric.qualified

// The aggregates the valuation plane computes beside the metric and could not
// hand to it: the entity rollup and the model's own net cash flow.
metric entity_cash = series_sum("entity.asset.proj.net_cash_flow", 0, 2)
metric model_cash  = series_sum("model.net_cash_flow", 0, 2)
```

## minimal_model

```cfdl
version 0.1
model "minimal-model"
time calendar monthly from 2026-01 for 12

entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower {
  schedule every month from 2026-01 to 2026-12
  amount = 1000
}
```

## monte_carlo_journal

```cfdl
version 0.1
model "monte-carlo-journal"
time calendar monthly from 2026-01 for 12

// WHEN DOES IT HAPPEN, AND HOW OFTEN — the question a stochastic run asks of
// the journal, and the one a per-trial log answers badly.
//
// `docs/13` §7.18 ruled out the obvious shape: trials x acts of output, and
// nobody reads ten thousand copies of the same sequence. What a reader wants
// is the DISTRIBUTION over the period each act first occurred, and the share
// of trials in which it occurred at all. `monte_carlo.journal` publishes that,
// one row per distinct act, bounded by the model rather than the trial count.
//
// The balance falls 40 a period from 1,000, reaching 560 by the end of the
// twelve-month horizon. The covenant level is sampled between 200 and 800, so
// a draw above 560 breaches inside the horizon and a draw below it never does.
// Both halves of the summary are therefore exercised by one model: the breach
// occurs in SOME trials, not all, and when it occurs the period varies with
// the draw. The seed is stated, so it is reproducible and can be a golden.

assume covenant_level ~ Uniform(min=200, max=800)

entity asset pool : Asset.Financial {
  balance  init 1000.0 next max(0.0, prev - 40.0)
  fee_rate init 0.02
}

event breach when asset.pool.balance <= inputs.covenant_level {
  set entity asset.pool.fee_rate = 0.05
}

stream credit.fee on entity asset.pool inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = asset.pool.balance * asset.pool.fee_rate
}

run monte_carlo trials 40 seed 20260827
```

## monte_carlo_metric_distribution

```cfdl
version 0.1
model "monte-carlo-metric-distribution"
time calendar annual from 2026-01 for 5

// A TRIAL IS A COMPLETE RUN, AND EVERY FIGURE IT COMPUTED SURVIVES IT.
//
// `docs/13` §7.87: the trial loop used to build a fresh one-entry map holding
// `model.npv` and publish that as the trial's whole record. Every other figure
// — the IRR, the MoIC, each entity's total, each stream's total, and every
// metric the model declared — was computed inside the trial and dropped on the
// way out. Since per-trial SERIES are (reasonably) not retained, the metric
// map is the only window into a trial: what the loop did not carry was
// unrecoverable after it, and the figure a stochastic case exists to assert
// existed in no trial.
//
// This model is that claim, in one page. The exit price is drawn, so every
// figure below MOVES with the draw and a distribution over it means something:
//
//   - `metric.exit_multiple` — a declared metric, the thing §7.25 built and
//     the thing §7.87 was throwing away;
//   - `model.irr` and `model.moic` — the acquisition in 2026 is the sign
//     change an IRR needs;
//   - `entity.container.fund.total` — the ROLLED-UP total, which is the join
//     to the results-plane graph: `graph.entities[].symbol` is
//     `container.fund`, so a consumer holding results alone can read a
//     per-entity distribution rather than inferring ownership from names
//     (§7.43, §7.91).
//
// The seed is stated, so every published percentile is reproducible: a Monte
// Carlo run IS a golden when its seed is.

entity container fund : Container.Fund

entity asset holding : Asset.Financial { part of container.fund }

assume exit_price ~ Uniform(min=900, max=1500)

stream deal.acquisition on entity asset.holding outflow currency USD {
  schedule every year from 2026-01 to 2026-01
  category investing.acquisition.purchase
  amount = 1000
}

stream deal.rent on entity asset.holding inflow currency USD {
  schedule every year from 2027-01 to 2030-01
  category operating.revenue.other
  amount = 80
}

stream deal.exit on entity asset.holding inflow currency USD {
  schedule every year from 2030-01 to 2030-01
  category investing.disposition.sale
  amount = inputs.exit_price
}

// An outflow's total is NEGATIVE — it is cash leaving — so the capital
// invested is its negation, and `exit_multiple` then reproduces `model.moic`
// in the language. Two computations of the same figure by different routes,
// one the engine's and one the model's, and the trial rows now carry both.
metric invested      = 0 - series_sum("deal.acquisition", 0, 4)
metric proceeds      = series_sum("deal.rent", 0, 4) + series_sum("deal.exit", 0, 4)
metric exit_multiple = metric.proceeds / metric.invested

run monte_carlo trials 32 seed 20260831
```

## monte_carlo_partial_metric

```cfdl
version 0.1
model "monte-carlo-partial-metric"
time calendar annual from 2026-01 for 3

// NOT EVERY TRIAL PUBLISHES EVERY METRIC, WHICH IS WHY A SUMMARY COUNTS.
//
// `model.irr` exists only where the flows solve for a rate. The swing below is
// drawn across zero, so in the trials that draw it negative there is no sign
// change and no IRR — and a mean over the trials that DID solve would read
// exactly like a mean over all of them, if the summary did not say how many it
// was taken over. `trials` on a metric summary is that count (`docs/13` §7.87).
//
// `model.npv` is published by every trial and `model.irr` by some, so this one
// model shows both cases in one results document.

entity asset co : Asset.Financial

assume swing ~ Uniform(min=-60, max=140)

stream ops.outlay on entity asset.co outflow currency USD {
  schedule every year from 2026-01 to 2026-01
  amount = 100
}

stream ops.swing on entity asset.co inflow currency USD {
  schedule every year from 2027-01 to 2028-01
  amount = inputs.swing
}

run monte_carlo trials 24 seed 20260831
```

## monte_carlo_smoke

```cfdl
version 0.1
model "monte-carlo-smoke"
time calendar monthly from 2026-01 for 6

entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower {
  schedule every month from 2026-01 to 2026-06
  amount = 1000
}
```

## noi_smoke

```cfdl
version 0.1
model "noi-smoke"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 13

entity asset property : CRE.Asset.RealProperty

// Ops streams: revenue $5k/mo, expense $2k/mo, 12 months
// Monthly ops net = $3,000; annual = $36,000
contract cre.revenue_line {
  term 2026-01..2026-12
  terms {
    amount = 5000
  }
}

contract cre.opex_line {
  term 2026-01..2026-12
  terms {
    amount = 2000
  }
}

// Terminal value: noi_value / exit_cap = 36000 / 0.09 = 400000
// noi_value equals the annualized ops net from the contracts above
contract cre.exit_cap on entity asset.property {
  term 2027-01..2027-01
  terms {
    cap_rate = 0.09
    income = 36000
  }
}
```

## obs_smoke

```cfdl
version 0.1
model "obs-smoke"
time calendar monthly from 2026-01 for 3

entity asset borrower : Asset.Financial

stream debt.rate_payment on entity asset.borrower {
  schedule every month from 2026-01 to 2026-03
  amount = obs.rate
}
```

## opco_basic_smoke

```cfdl
version 0.1
model "opco-basic-smoke"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset business : OpCo.Asset.Enterprise

contract opco.revenue_line {
  term 2026-01..2031-12
  terms {
    amount = 120000
    growth_rate = 0.0
  }
}

contract opco.opex_line {
  term 2026-01..2031-12
  terms {
    amount = 70000
  }
}

contract opco.working_capital {
  term 2026-01..2031-12
  terms {
    amount = 3000
  }
}

contract opco.exit_multiple {
  term 2031-12..2031-12
  terms {
    multiple = 6.5
    base = 800000
  }
}
```

## opco_exit_perpetuity_smoke

```cfdl
version 0.1
model "opco-exit-perpetuity-smoke"
use pack "opco" version "0.1.0"
time calendar annual from 2026-01 for 5

entity asset firm : OpCo.Asset.Enterprise

// A five-year forecast ending in a growing perpetuity — the shape every
// intrinsic valuation takes, and one the pack could not express before:
// exit_multiple and exit_ebitda both apply a MULTIPLE to something.
stream firm.fcff on entity asset.firm inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  category operating.revenue.recurring
  amount = 1000 * pow(1.04, time.t)
}

// Struck on the terminal year's flow, at a terminal cost of capital that is
// deliberately NOT the run's discount rate: a business in steady state is
// capitalized at a steady-state rate. The contract applies the (1 + g) step,
// so base_value is the terminal flow itself.
//
// The exit settles at the END of its period, so it discounts the full five
// years. `on_date` alone would place it a year early on an annual model.
contract opco.exit_perpetuity on entity asset.firm {
  term 2030-01..2030-01
  terms {
    base = 1169.858560
    growth_rate = 0.025
    discount_rate = 0.085
  }
}
```

## opco_growth_smoke

```cfdl
version 0.1
model "opco-growth-smoke"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset business : OpCo.Asset.Enterprise

contract opco.revenue_line {
  term 2026-01..2027-12
  terms {
    amount = 100000
    growth_rate = 0.12
  }
}

contract opco.opex_line {
  term 2026-01..2027-12
  terms {
    amount = 60000
  }
}
```

## opco_lbo_smoke

```cfdl
version 0.1
model "opco-lbo-smoke"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset target : OpCo.Asset.Enterprise

contract opco.revenue_line on entity asset.target {
  term 2026-01..2027-12
  terms {
    amount = 200000
    growth_rate = 0.05
  }
}

contract opco.opex_line on entity asset.target {
  term 2026-01..2027-12
  terms {
    amount = 120000
    growth_rate = 0.03
  }
}

contract opco.working_capital_policy on entity asset.target {
  term 2026-01..2027-12
  terms {
    ar_days = 40
    ap_days = 25
    inv_days = 15
    release_at_end = 1
  }
}

contract opco.capex_line on entity asset.target {
  term 2026-01..2027-12
  terms {
    amount = 5000
    pct_of_revenue = 0.02
  }
}

// 6-month IO, then amortizing over 36 months, balloon at maturity.
contract opco.term_debt on entity asset.target {
  term 2026-01..2027-12
  terms {
    principal = 3000000
    interest_rate = 0.09
    interest_only_months = 6
    amortization_months = 36
  }
}

contract opco.cash_taxes on entity asset.target {
  term 2026-01..2027-12
  terms {
    tax_rate = 0.25
    da_monthly = 20000
  }
}

contract opco.acquisition on entity asset.target {
  term 2026-01..2026-01
  terms {
    price = 5000000
  }
}

contract opco.exit_ebitda on entity asset.target {
  term 2027-12..2027-12
  terms {
    multiple = 7.0
    selling_costs = 0.02
  }
}
```

## opco_professional_services

```cfdl
version 0.1
model "opco-professional-services"
time calendar monthly from 2026-01 for 60

entity asset firm : Asset.Financial
entity asset equity : Asset.Financial

// Recurring client retainers: $280k/mo base, 8% annual compound growth
stream rev.retainers on entity asset.firm inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 280000 * pow(1.08, time.t / 12.0)
}

// Project & transaction fees: $95k/mo base, 12% annual compound growth
stream rev.project_fees on entity asset.firm inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 95000 * pow(1.12, time.t / 12.0)
}

// Software licensing / IP royalties: $45k/mo base, 18% annual compound growth
stream rev.licensing on entity asset.firm inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 45000 * pow(1.18, time.t / 12.0)
}

// Compensation & benefits (billable staff + management team)
stream opex.compensation on entity asset.firm outflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 230000
}

// G&A: finance, legal, HR, executive overhead
stream opex.ga on entity asset.firm outflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 45000
}

// Facilities, IT infrastructure, software licenses
stream opex.facilities on entity asset.firm outflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 22000
}

// Sales & marketing, business development
stream opex.sales_marketing on entity asset.firm outflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 18000
}

// Working capital requirement
stream opex.working_capital on entity asset.firm outflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 14000
}

// PE exit: Y5 annualized EBITDA ~$4.27M; 8.5x multiple → $36,300,000
stream exit.pe_sale on entity asset.equity inflow currency USD {
  schedule on 2030-12
  amount = 36300000
}
```

## option_as_contract

```cfdl
version 0.1
model "option-as-contract"
use pack "opco" version "0.1.0"
time calendar annual from 2026-01 for 2

entity asset target : OpCo.Asset.Enterprise
entity party sponsor : OpCo.Party.Sponsor { name = "Sponsor LLC" }
entity party mgmt : OpCo.Party.Management { name = "Management" }

// An option is a contract with an election: it is written ON something and is
// BETWEEN parties, exactly as every other contract is.
option mgmt_pool on entity asset.target type OpCo.Contract.EquityOption {
  parties { grantor = party.sponsor, holder = party.mgmt }
  exercise when time.t >= 1
  payoff 250.0
}

stream opco.revenue on entity asset.target inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  category operating.revenue.recurring
  amount = 1000
}
```

## option_reads_state

```cfdl
version 0.1
model "option-reads-state"
time calendar annual from 2026-01 for 4

// The capability this fixture exists to pin: an option's `exercise when` can
// read a value the MODEL computes. It could not before — the guard was
// evaluated in a pass that ran before any state existed, and the failure was a
// warning and `false`, so the option silently never fired.

entity asset plant : Asset.Real {
  // The plant's carrying value — what an option's exercise test reads.
  book_value init 100.0
             next prev * 1.10
}
entity party holder : Party { name = "Holder" }

// A value that MOVES, so the period the option fires is a fact about the
// recurrence rather than a coincidence of the timeline.

// Fires the first period book value crosses 120: 100, 110, 121 -> period 2.
option call_at_120 on entity asset.plant type Option.Call {
  parties { holder = party.holder }
  exercise when asset.plant.book_value > 120.0
  payoff asset.plant.book_value - 120.0
}

// Never in the money over this horizon (peaks at 133.1), so it publishes a
// zero series rather than no series — a non-exercise is assertable.
option call_at_500 on entity asset.plant type Option.Call {
  exercise when asset.plant.book_value > 500.0
  payoff asset.plant.book_value - 500.0
}

stream plant.revenue on entity asset.plant inflow currency USD {
  schedule every year from 2026-01 to 2029-01
  amount = 10
}
```

## pack_amortization_day_count

```cfdl
version 0.1
model "pack-amortization-day-count"
use pack "credit" version "0.1.0"
time calendar monthly from 2025-01 for 13

entity asset buyer : Credit.Asset.LoanPool

// A commercial Actual/360 loan: interest accrues on actual days, but the
// payment is struck once on a 30/360 schedule and held constant, with
// principal absorbing the month-length variation. Recomputing both legs from
// one varying divisor makes the payment swing with the length of the month,
// which no loan document does.
//
// No prepayments or defaults, so this isolates the two rate bases.
contract credit.pool_level_pay.p on entity asset.buyer {
  term 2025-01..2025-12
  terms {
    principal = 100000000
    interest_rate = 0.08
    term_months = 360
    cpr = 0
    cdr = 0
    severity = 0
    recovery_lag_months = 0
    day_count = "act/360"
    amortization_day_count = "30/360"
  }
}
```

## pack_cadence_cre_annual

```cfdl
version 0.1
model "pack-cadence-cre-annual"
use pack "cre" version "0.1.0"
time calendar annual from 2025-01 for 3 project 1

entity asset tower : CRE.Asset.RealProperty

// One lease on three calendars: 480,000/yr, five months free at the start,
// 3% anniversary escalation, recoveries above a 300,000 stop at 40%.
//
// The five free months are the interesting part. They are FIVE CALENDAR
// MONTHS on every grid — a fact about the lease, not the modeller's choice —
// so they pro-rate: 5 periods monthly, 1.667 quarterly, 0.417 annually. Only
// exact pro-rating makes the annual figures agree.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2025-01..2028-01
  terms {
    rent_year = 480000
    free_rent_months = 5
    escalation = 0.03
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 300000
    pro_rata_share = 0.40
  }
}

contract cre.opex_line on entity asset.tower {
  term 2025-01..2028-01
  terms {
    amount_year = 300000
    growth_rate = 0.025
  }
}

// The operating terms run through the projection tail so the exit has a
// forward year to read. A hand-written stream could not reach the tail
// before E2103 was widened; a pack contract always could.
// Exit on forward NOI derived from the modeled streams over the next YEAR,
// which is `project 1` periods on this grid — the window that used to be a
// hardcoded twelve.
contract cre.exit_forward on entity asset.tower {
  term 2027-01..2027-01
  terms {
    cap_rate = 0.065
    selling_costs = 0.02
  }
}
```

## pack_cadence_cre_monthly

```cfdl
version 0.1
model "pack-cadence-cre-monthly"
use pack "cre" version "0.1.0"
time calendar monthly from 2025-01 for 36 project 12

entity asset tower : CRE.Asset.RealProperty

// One lease on three calendars: 480,000/yr, five months free at the start,
// 3% anniversary escalation, recoveries above a 300,000 stop at 40%.
//
// The five free months are the interesting part. They are FIVE CALENDAR
// MONTHS on every grid — a fact about the lease, not the modeller's choice —
// so they pro-rate: 5 periods monthly, 1.667 quarterly, 0.417 annually. Only
// exact pro-rating makes the annual figures agree.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2025-01..2028-12
  terms {
    rent_year = 480000
    free_rent_months = 5
    escalation = 0.03
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 300000
    pro_rata_share = 0.40
  }
}

contract cre.opex_line on entity asset.tower {
  term 2025-01..2028-12
  terms {
    amount_year = 300000
    growth_rate = 0.025
  }
}

// The operating terms run through the projection tail so the exit has a
// forward year to read. A hand-written stream could not reach the tail
// before E2103 was widened; a pack contract always could.
// Exit on forward NOI derived from the modeled streams over the next YEAR,
// which is `project 12` periods on this grid — the window that used to be a
// hardcoded twelve.
contract cre.exit_forward on entity asset.tower {
  term 2027-12..2027-12
  terms {
    cap_rate = 0.065
    selling_costs = 0.02
  }
}
```

## pack_cadence_cre_quarterly

```cfdl
version 0.1
model "pack-cadence-cre-quarterly"
use pack "cre" version "0.1.0"
time calendar quarterly from 2025-01 for 12 project 4

entity asset tower : CRE.Asset.RealProperty

// One lease on three calendars: 480,000/yr, five months free at the start,
// 3% anniversary escalation, recoveries above a 300,000 stop at 40%.
//
// The five free months are the interesting part. They are FIVE CALENDAR
// MONTHS on every grid — a fact about the lease, not the modeller's choice —
// so they pro-rate: 5 periods monthly, 1.667 quarterly, 0.417 annually. Only
// exact pro-rating makes the annual figures agree.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2025-01..2028-10
  terms {
    rent_year = 480000
    free_rent_months = 5
    escalation = 0.03
    opex_year = 300000
    opex_escalation = 0.025
    expense_stop_year = 300000
    pro_rata_share = 0.40
  }
}

contract cre.opex_line on entity asset.tower {
  term 2025-01..2028-10
  terms {
    amount_year = 300000
    growth_rate = 0.025
  }
}

// The operating terms run through the projection tail so the exit has a
// forward year to read. A hand-written stream could not reach the tail
// before E2103 was widened; a pack contract always could.
// Exit on forward NOI derived from the modeled streams over the next YEAR,
// which is `project 4` periods on this grid — the window that used to be a
// hardcoded twelve.
contract cre.exit_forward on entity asset.tower {
  term 2027-10..2027-10
  terms {
    cap_rate = 0.065
    selling_costs = 0.02
  }
}
```

## pack_cadence_credit_daily_monthly_pay

```cfdl
version 0.1
model "pack-cadence-credit-daily_monthly_pay"
use pack "credit" version "0.1.0"
time calendar daily from 2025-01 for 1186

entity asset buyer : Credit.Asset.LoanPool

// The SAME pool: a 36-month level-pay loan with a 3-month recovery lag, once
// on a monthly grid and once on a DAILY book that still pays monthly.
//
// A 30-year mortgage carried on a daily book is still 360 monthly payments,
// not 10,950 daily ones — the daily grid is there for accrual, day-count and
// daily-reset floaters. `payment_frequency` says so, and periods-per-year
// then comes from the payment rhythm (12) rather than the calendar (365).
//
// So these two must agree EXACTLY on annual totals: they are literally the
// same 36 payments. That is the sharpest available proof that the payment
// grid and the model grid are separated correctly.
contract credit.pool_level_pay.book on entity asset.buyer {
  term 2025-01..2028-03
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 36
    cpr = 0.10
    cdr = 0.03
    severity = 0.50
    recovery_lag_months = 3
    payment_frequency = "month"
  }
}
```

## pack_cadence_credit_monthly

```cfdl
version 0.1
model "pack-cadence-credit-monthly"
use pack "credit" version "0.1.0"
time calendar monthly from 2025-01 for 39

entity asset buyer : Credit.Asset.LoanPool

// The SAME pool: a 36-month level-pay loan with a 3-month recovery lag, once
// on a monthly grid and once on a DAILY book that still pays monthly.
//
// A 30-year mortgage carried on a daily book is still 360 monthly payments,
// not 10,950 daily ones — the daily grid is there for accrual, day-count and
// daily-reset floaters. `payment_frequency` says so, and periods-per-year
// then comes from the payment rhythm (12) rather than the calendar (365).
//
// So these two must agree EXACTLY on annual totals: they are literally the
// same 36 payments. That is the sharpest available proof that the payment
// grid and the model grid are separated correctly.
contract credit.pool_level_pay.book on entity asset.buyer {
  term 2025-01..2028-03
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 36
    cpr = 0.10
    cdr = 0.03
    severity = 0.50
    recovery_lag_months = 3

  }
}
```

## pack_cadence_energy_annual

```cfdl
version 0.1
model "pack-cadence-energy-annual"
use pack "energy" version "0.1.0"
time calendar annual from 2025-01 for 3

entity asset microgrid : Energy.Asset.GenerationFacility

// One three-year PPA + O&M deal, written identically on four calendars. Only
// `time calendar` differs, so the annual figures must agree exactly.
//
// The window is 2025-2027 for a reason: all three are non-leap years. On a
// daily grid an annual figure is spread as X_year / 365 every day, which is
// the Act/365-Fixed convention — so a 366-day year genuinely pays 366/365 of
// the annual amount. That is correct, not drift, but it means the annual
// parity identity does not hold across a leap year on a daily calendar. See
// packs/energy/README.md.
//
// Terms start on 2025-01 and run whole years so contract anniversaries line up
// with the rollup boundaries; I1 is an annual identity, not a within-year one.
contract energy.ppa on entity asset.microgrid {
  term 2025-01..2027-01
  terms {
    quantity = 4200
    price = 85
    escalation = 0.02
    degradation = 0.005
  }
}

contract energy.om on entity asset.microgrid {
  term 2025-01..2027-01
  terms {
    fee_year = 48000
    escalation = 0.025
  }
}

contract energy.capacity on entity asset.microgrid {
  term 2025-01..2027-01
  terms { price = 60000 }
}
```

## pack_cadence_energy_daily

```cfdl
version 0.1
model "pack-cadence-energy-daily"
use pack "energy" version "0.1.0"
time calendar daily from 2025-01 for 1095

entity asset microgrid : Energy.Asset.GenerationFacility

// One three-year PPA + O&M deal, written identically on four calendars. Only
// `time calendar` differs, so the annual figures must agree exactly.
//
// The window is 2025-2027 for a reason: all three are non-leap years. On a
// daily grid an annual figure is spread as X_year / 365 every day, which is
// the Act/365-Fixed convention — so a 366-day year genuinely pays 366/365 of
// the annual amount. That is correct, not drift, but it means the annual
// parity identity does not hold across a leap year on a daily calendar. See
// packs/energy/README.md.
//
// Terms start on 2025-01 and run whole years so contract anniversaries line up
// with the rollup boundaries; I1 is an annual identity, not a within-year one.
contract energy.ppa on entity asset.microgrid {
  term 2025-01..2027-12-31
  terms {
    quantity = 4200
    price = 85
    escalation = 0.02
    degradation = 0.005
  }
}

contract energy.om on entity asset.microgrid {
  term 2025-01..2027-12-31
  terms {
    fee_year = 48000
    escalation = 0.025
  }
}

contract energy.capacity on entity asset.microgrid {
  term 2025-01..2027-12-31
  terms { price = 60000 }
}
```

## pack_cadence_energy_monthly

```cfdl
version 0.1
model "pack-cadence-energy-monthly"
use pack "energy" version "0.1.0"
time calendar monthly from 2025-01 for 36

entity asset microgrid : Energy.Asset.GenerationFacility

// One three-year PPA + O&M deal, written identically on four calendars. Only
// `time calendar` differs, so the annual figures must agree exactly.
//
// The window is 2025-2027 for a reason: all three are non-leap years. On a
// daily grid an annual figure is spread as X_year / 365 every day, which is
// the Act/365-Fixed convention — so a 366-day year genuinely pays 366/365 of
// the annual amount. That is correct, not drift, but it means the annual
// parity identity does not hold across a leap year on a daily calendar. See
// packs/energy/README.md.
//
// Terms start on 2025-01 and run whole years so contract anniversaries line up
// with the rollup boundaries; I1 is an annual identity, not a within-year one.
contract energy.ppa on entity asset.microgrid {
  term 2025-01..2027-12
  terms {
    quantity = 4200
    price = 85
    escalation = 0.02
    degradation = 0.005
  }
}

contract energy.om on entity asset.microgrid {
  term 2025-01..2027-12
  terms {
    fee_year = 48000
    escalation = 0.025
  }
}

contract energy.capacity on entity asset.microgrid {
  term 2025-01..2027-12
  terms { price = 60000 }
}
```

## pack_cadence_energy_quarterly

```cfdl
version 0.1
model "pack-cadence-energy-quarterly"
use pack "energy" version "0.1.0"
time calendar quarterly from 2025-01 for 12

entity asset microgrid : Energy.Asset.GenerationFacility

// One three-year PPA + O&M deal, written identically on four calendars. Only
// `time calendar` differs, so the annual figures must agree exactly.
//
// The window is 2025-2027 for a reason: all three are non-leap years. On a
// daily grid an annual figure is spread as X_year / 365 every day, which is
// the Act/365-Fixed convention — so a 366-day year genuinely pays 366/365 of
// the annual amount. That is correct, not drift, but it means the annual
// parity identity does not hold across a leap year on a daily calendar. See
// packs/energy/README.md.
//
// Terms start on 2025-01 and run whole years so contract anniversaries line up
// with the rollup boundaries; I1 is an annual identity, not a within-year one.
contract energy.ppa on entity asset.microgrid {
  term 2025-01..2027-10
  terms {
    quantity = 4200
    price = 85
    escalation = 0.02
    degradation = 0.005
  }
}

contract energy.om on entity asset.microgrid {
  term 2025-01..2027-10
  terms {
    fee_year = 48000
    escalation = 0.025
  }
}

contract energy.capacity on entity asset.microgrid {
  term 2025-01..2027-10
  terms { price = 60000 }
}
```

## pack_cadence_opco_annual

```cfdl
version 0.1
model "pack-cadence-opco-annual"
use pack "opco" version "0.1.0"
time calendar annual from 2025-01 for 3

entity asset business : OpCo.Asset.Enterprise

// One operating business on three calendars. Stated with the ANNUAL siblings
// (`amount_year`), which is what makes the deal grid-independent: the bare
// `amount` term is per-period by definition and would mean different
// economics on each calendar.
//
// Growth is held at zero on purpose. opco compounds growth CONTINUOUSLY on
// the model clock — (1+g)^(t/ppy), a documented convention — so a finer grid
// genuinely captures more intra-year compounding: 5% annual growth yields
// 1,472,709 of year-one revenue monthly against 1,440,000 annually. That is a
// real economic difference, not a cadence defect, and it would mask the thing
// this fixture exists to test. See packs/opco/README.md.
//
// Term debt and policy-driven working capital are deliberately absent. Both
// are correct but not annual-rollup invariant: nominal rate accrual differs
// by cadence by design (a 6% loan is 0.5%/month and 1.5%/quarter), and the
// working-capital delta telescopes so only its sum over life is invariant.
// Those are covered by the benchmark reference generators instead.
contract opco.revenue_line {
  term 2025-01..2027-01
  terms {
    amount_year = 1440000
    growth_rate = 0.0
  }
}

contract opco.opex_line {
  term 2025-01..2027-01
  terms {
    amount_year = 840000
    growth_rate = 0.0
  }
}

contract opco.capex_line {
  term 2025-01..2027-01
  terms {
    amount_year = 60000
    pct_of_revenue = 0.02
  }
}

contract opco.cash_taxes {
  term 2025-01..2027-01
  terms {
    tax_rate = 0.25
    da_year = 120000
  }
}
```

## pack_cadence_opco_monthly

```cfdl
version 0.1
model "pack-cadence-opco-monthly"
use pack "opco" version "0.1.0"
time calendar monthly from 2025-01 for 36

entity asset business : OpCo.Asset.Enterprise

// One operating business on three calendars. Stated with the ANNUAL siblings
// (`amount_year`), which is what makes the deal grid-independent: the bare
// `amount` term is per-period by definition and would mean different
// economics on each calendar.
//
// Growth is held at zero on purpose. opco compounds growth CONTINUOUSLY on
// the model clock — (1+g)^(t/ppy), a documented convention — so a finer grid
// genuinely captures more intra-year compounding: 5% annual growth yields
// 1,472,709 of year-one revenue monthly against 1,440,000 annually. That is a
// real economic difference, not a cadence defect, and it would mask the thing
// this fixture exists to test. See packs/opco/README.md.
//
// Term debt and policy-driven working capital are deliberately absent. Both
// are correct but not annual-rollup invariant: nominal rate accrual differs
// by cadence by design (a 6% loan is 0.5%/month and 1.5%/quarter), and the
// working-capital delta telescopes so only its sum over life is invariant.
// Those are covered by the benchmark reference generators instead.
contract opco.revenue_line {
  term 2025-01..2027-12
  terms {
    amount_year = 1440000
    growth_rate = 0.0
  }
}

contract opco.opex_line {
  term 2025-01..2027-12
  terms {
    amount_year = 840000
    growth_rate = 0.0
  }
}

contract opco.capex_line {
  term 2025-01..2027-12
  terms {
    amount_year = 60000
    pct_of_revenue = 0.02
  }
}

contract opco.cash_taxes {
  term 2025-01..2027-12
  terms {
    tax_rate = 0.25
    da_year = 120000
  }
}
```

## pack_cadence_opco_quarterly

```cfdl
version 0.1
model "pack-cadence-opco-quarterly"
use pack "opco" version "0.1.0"
time calendar quarterly from 2025-01 for 12

entity asset business : OpCo.Asset.Enterprise

// One operating business on three calendars. Stated with the ANNUAL siblings
// (`amount_year`), which is what makes the deal grid-independent: the bare
// `amount` term is per-period by definition and would mean different
// economics on each calendar.
//
// Growth is held at zero on purpose. opco compounds growth CONTINUOUSLY on
// the model clock — (1+g)^(t/ppy), a documented convention — so a finer grid
// genuinely captures more intra-year compounding: 5% annual growth yields
// 1,472,709 of year-one revenue monthly against 1,440,000 annually. That is a
// real economic difference, not a cadence defect, and it would mask the thing
// this fixture exists to test. See packs/opco/README.md.
//
// Term debt and policy-driven working capital are deliberately absent. Both
// are correct but not annual-rollup invariant: nominal rate accrual differs
// by cadence by design (a 6% loan is 0.5%/month and 1.5%/quarter), and the
// working-capital delta telescopes so only its sum over life is invariant.
// Those are covered by the benchmark reference generators instead.
contract opco.revenue_line {
  term 2025-01..2027-10
  terms {
    amount_year = 1440000
    growth_rate = 0.0
  }
}

contract opco.opex_line {
  term 2025-01..2027-10
  terms {
    amount_year = 840000
    growth_rate = 0.0
  }
}

contract opco.capex_line {
  term 2025-01..2027-10
  terms {
    amount_year = 60000
    pct_of_revenue = 0.02
  }
}

contract opco.cash_taxes {
  term 2025-01..2027-10
  terms {
    tax_rate = 0.25
    da_year = 120000
  }
}
```

## pack_cadence_probe_annual

```cfdl
version 0.1
model "pack-cadence-probe-annual"
use pack "testpack" version "0.1.0"
time calendar annual from 2026-01 for 3

entity asset borrower : Asset.Financial

// One deal, three grids. 120,000/yr with five months free at the start, a
// three-year term and 10% annual escalation, expressed identically each time —
// only `time calendar` differs. Their ANNUAL figures must agree exactly:
// that is the cadence-neutrality invariant, and the five free months have to
// pro-rate (5 periods monthly, 1.667 quarterly, 0.417 annually) for year one
// to come out at 70,000 = 120,000 x 7/12 on all three.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2028-01
  terms {
    amount_year = 120000
    free_months = 5
    term_months = 36
    escalation = 0.10
  }
}
```

## pack_cadence_probe_monthly

```cfdl
version 0.1
model "pack-cadence-probe-monthly"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 36

entity asset borrower : Asset.Financial

// One deal, three grids. 120,000/yr with five months free at the start, a
// three-year term and 10% annual escalation, expressed identically each time —
// only `time calendar` differs. Their ANNUAL figures must agree exactly:
// that is the cadence-neutrality invariant, and the five free months have to
// pro-rate (5 periods monthly, 1.667 quarterly, 0.417 annually) for year one
// to come out at 70,000 = 120,000 x 7/12 on all three.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2028-12
  terms {
    amount_year = 120000
    free_months = 5
    term_months = 36
    escalation = 0.10
  }
}
```

## pack_cadence_probe_quarterly

```cfdl
version 0.1
model "pack-cadence-probe-quarterly"
use pack "testpack" version "0.1.0"
time calendar quarterly from 2026-01 for 12

entity asset borrower : Asset.Financial

// One deal, three grids. 120,000/yr with five months free at the start, a
// three-year term and 10% annual escalation, expressed identically each time —
// only `time calendar` differs. Their ANNUAL figures must agree exactly:
// that is the cadence-neutrality invariant, and the five free months have to
// pro-rate (5 periods monthly, 1.667 quarterly, 0.417 annually) for year one
// to come out at 70,000 = 120,000 x 7/12 on all three.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2028-10
  terms {
    amount_year = 120000
    free_months = 5
    term_months = 36
    escalation = 0.10
  }
}
```

## pack_currency_inr

```cfdl
version 0.1
model "pack-currency-inr" currency INR
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

// A pack contract in a non-USD model. Lowering rules leave `currency` unset,
// so the streams they emit inherit the model's currency rather than asserting
// the instrument is American — a PPA in Rajasthan is not a USD contract.
contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = 5000
    price = 3000
    escalation = 0.0
    degradation = 0.005
    availability = 1.0
  }
}
```

## pack_day_count_act360

```cfdl
version 0.1
model "pack-day-count-act360"
use pack "credit" version "0.1.0"
time calendar monthly from 2025-01 for 13

entity asset buyer : Credit.Asset.LoanPool

// Actual/360, the USD credit convention. A nominal annual rate is divided by
// the rule's accrual divisor, which the default reads as periods-per-year —
// every period is 1/ppy of a year, the 30/360 reading — and which act/360
// scales by the period's real days.
//
// So interest is 6,200 in a 31-day January and 5,600 in a 28-day February,
// against a flat 6,000 under 30/360, and 73,000 over a 365-day year rather
// than 72,000. That 365/360 uplift is the point of the convention.
contract credit.pool_io_bullet.p on entity asset.buyer {
  term 2025-01..2025-12
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 12
    cpr = 0
    cdr = 0
    severity = 0
    recovery_lag_months = 0
    day_count = "act/360"
  }
}
```

## pack_quarterly_interval

```cfdl
version 0.1
model "pack-quarterly-interval"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset borrower : Asset.Financial

// A lowering rule may declare its own interval, so a pack can express a
// quarterly coupon on a monthly model. Rules that omit it pay at the
// calendar cadence, which is what every shipped rule does.
contract test.coupon_contract on entity asset.borrower {
  term 2026-01..2027-12
  terms { note = "quarterly" }
}
```

## pack_rule_reads_prev_field

```cfdl
version 0.1
model "pack-rule-reads-prev-field"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 4

entity asset borrower : Asset.Financial

// A LOWERING RULE READING BOTH ENDS OF THE FIELD IT DECLARES.
//
// The rule declares `testpack_balance` (100 at the first tick, +100 each
// period after) and strikes interest on the average of its opening and closing
// value — the accessor a construction facility needs and every commercial draw
// schedule reproduces.
//
// The balance runs 100, 200, 300, 400. The contract starts a period after the
// model so there is a previous close to read, so interest is:
//   (100 + 200) / 2 * 0.01 = 1.5
//   (200 + 300) / 2 * 0.01 = 2.5
//   (300 + 400) / 2 * 0.01 = 3.5
//
// This is a regression fixture. `prev.field.<name>` lowered to a path nothing
// bound, so the stream evaluated to zero at every period with a warning rather
// than an error, and the model still reported ok.
contract test.avg_balance_contract on entity asset.borrower {
  term 2026-02..2026-04
  terms {
    draw = 100
    rate = 0.01
  }
}
```

## pack_template_terms

```cfdl
version 0.1
model "pack-template-terms"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 6

entity asset borrower : Asset.Financial

// rate comes from the contract; units falls back to the rule default (10).
contract test.fee_contract {
  term 2026-01..2026-06
  terms {
    rate = 25
  }
}
```

## participant_returns

```cfdl
version 0.1
model "participant-returns"
time calendar annual from 2026-01 for 5

// WHAT DID THIS PARTICIPANT EARN?
//
// The model computes `model.irr` on the deal's net cash, and a waterfall
// attributes each step's payment to a payee — and there the trail stopped. To
// measure a party's own return an analyst had to hand-assemble the payee's
// cash, capital in and distributions out, and run the arithmetic outside the
// language against results the language already held.
//
// The vector is the party's own ACCOUNT, never the payee's streams: a step's
// payee says who was paid, but attributing through stream names is the trap
// docs/13 §7.43 records. An account's journal already separates the two
// directions — a contribution is a NEGATIVE inflow, a receipt is an
// allocation in — so the sign change an IRR needs is recorded, not inferred.
//
// 1,000 called at t=0, then 400 a year for four years:
//   moic = 1,600 / 1,000            = 1.6
//   irr  solves -1000 + 400/(1+r) + ... + 400/(1+r)^4 = 0

entity asset deal : Asset.Financial
entity party lp   : Party { name = "Limited Partner" }

account lp_capital {
  owner party.lp
  from if(time.t == 0, -1000.0, 0.0)
}

stream deal.income on entity asset.deal inflow currency USD {
  schedule every year from 2027-01 to 2030-01
  amount = 400
}

waterfall deal.distribution on entity asset.deal {
  schedule every year from 2027-01 to 2030-01
  from available
  pay to_lp to account lp_capital = remaining
}

metric lp_irr  = irr(party.lp)
metric lp_moic = moic(party.lp)
```

## payment_terms

```cfdl
version 0.1
model "payment-terms"
time calendar monthly from 2026-01 for 18

entity asset co : Asset.Financial

// Cash arrives when the contract says it does, not when the activity happened.
// Billing is at period close, so January's is invoiced 31 January and net-45
// falls in mid-March.
stream co.on_time on entity asset.co inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 1000
}

// net 0 is the historical behavior, stated explicitly.
stream co.net0 on entity asset.co inflow currency USD {
  schedule every month net 0 from 2026-01 to 2026-06
  amount = 1000
}

// Under net-30 January and February both settle in March; their amounts sum
// rather than one displacing the other.
stream co.net30 on entity asset.co inflow currency USD {
  schedule every month net 30 from 2026-01 to 2026-06
  amount = 1000
}

stream co.net45 on entity asset.co inflow currency USD {
  schedule every month net 45 from 2026-01 to 2026-06
  amount = 1000
}

// Months step by the calendar, not by 30 days.
stream co.net2m on entity asset.co inflow currency USD {
  schedule every month net 2 months from 2026-01 to 2026-06
  amount = 1000
}
```

## payment_terms_contract

```cfdl
version 0.1
model "payment-terms-contract"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset plant : Energy.Asset.GenerationFacility

// A contract states its payment terms once, and every stream it lowers
// settles on them — the commercial reality that this clause exists to
// express. The amount is still evaluated in the period that earned it; only
// the cash moves.
contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  payment net 45
  terms {
    quantity = 6000
    price = 3000
    escalation = 0.0
    degradation = 0.005
    availability = 1.0
  }
}
```

## per_stream_category

```cfdl
version 0.1
model "per-stream-category"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// A CONTRACT LOWERS ONE OR MORE STREAMS, AND EACH CARRIES ITS OWN CATEGORY.
//
// The pack states what a permanent mortgage's interest, principal and proceeds
// are, and is right about all three. Where a deal needs one of them classified
// differently, the override names the stream: the other two keep the pack's
// answer. A clause that could not name one would flatten all three.
entity asset tower : CRE.Asset.RealProperty

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2026-12
  category cre.debt.interest = operating.expense.interest
  terms {
    principal = 1000000
    interest_rate = 0.05
    amortization_months = 300
  }
}

// The single-stream case needs no name: there is nothing to disambiguate.
contract cre.opex_line.rooms on entity asset.tower {
  term 2026-01..2026-12
  category operating.expense.rooms
  terms {
    amount_year = 12000
  }
}
```

## phase_schedule_smoke

```cfdl
version 0.1
model "phase-schedule-smoke"
time calendar monthly from 2025-01 for 24

entity asset property : Asset.Real

phase construction from 2025-01 to 2025-12
phase operations from 2026-01 to 2026-12

stream development.construction_draw on entity asset.property outflow {
  schedule on phase_enter("construction")
  amount = 500000
}

stream operations.revenue on entity asset.property inflow {
  schedule every month from phase_start("operations") to phase_end("operations")
  amount = 10000
}
```

## pool_retires_by_role

```cfdl
version 0.1
model "pool-retires-by-role"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 15

entity asset pool : Credit.Asset.LoanPool

contract credit.pool_level_pay.smoke on entity asset.pool {
  term 2026-01..2027-03
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 12
    cpr = 0.10
    cdr = 0.03
    severity = 0.50
    recovery_lag_months = 3
  }
}

// STATE OWNED BY THE AGREEMENT (docs/40 §3, stage 6). The pool's machine
// carries `on enter retired { set balance = 0 }`, where `balance` is the
// field ROLE the pack's rules fill — the pool's balance every stream reads
// and its lagged twin — so the model retires the pool with one status
// write and names no instance's field. Retirement is an occurrence: a
// clean-up call, here at the sixth distribution. From the next period every
// stream the pool lowers reads zero, with no per-entity write in the model.
event clean_up_call when time.t == 6 {
  set entity asset.pool.status = "retired"
}
```

## prev_on_a_field

```cfdl
version 0.1
model "prev-on-a-field"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 4

// READING A FIELD ONE PERIOD BACK.
//
// `asset.senior.balance` is that field at period CLOSE.
// `prev.asset.senior.balance` is the same field at the close before this one.
//
// THIS RETIRES THE `_open` PATTERN. A model that needed both ends of a period
// used to declare the quantity twice — `tlb_balance` and `tlb_balance_open` —
// where the second was not a quantity at all but a missing accessor wearing a
// name. Average-balance interest, which is the standard convention in a debt
// schedule, is the case that forced it.
//
// WHY DOTTED AND NOT `prev asset.senior.balance`. `prev` is an ordinary
// identifier, so a prefix form would put two operands side by side — which is
// exactly where an expression ENDS. Reusing the dot keeps one rule for where
// expressions stop and one spelling for "the period before", the same
// `prev.<name>` a declared state already uses.

entity asset senior : Credit.Asset.Tranche {
  seniority = 1
  balance init 100.0 next max(0.0, prev - 10.0)
}

entity asset junior : Credit.Asset.Tranche {
  seniority = 2
  // The balance this period OPENED with: last period's close.
  opening init 0.0 next prev.asset.senior.balance
}

entity party lender : Party { name = "Lender" }

// Interest on the AVERAGE of the period's opening and closing balance — the
// convention that needed two states before, and needs one field now.
stream credit.interest on entity asset.senior outflow currency USD {
  schedule every year from 2027-01 to 2029-01
  category operating.collection.interest
  amount = (asset.junior.opening + asset.senior.balance) / 2.0 * 0.06
}
```

## quantile_slices

```cfdl
version 0.1
model "quantile-slices"
time calendar monthly from 2026-01 for 3

entity asset battery : Asset.Financial

// A price stack written worst-first, the way a duration curve reads. The
// compiler stores it ascending, so the order word never reaches the IR.
quantile prices linear by exceedance ref energy.power_price {
  1.00: 512.0
  0.98: 340.0
  0.50:  28.0
  0.00:  11.0
}

// The same points, written ascending and read as steps. Declared to pin that
// the two interpolations give DIFFERENT integrals over the same slice — the
// shape is a declaration, not a detail.
quantile prices_step step {
  0.00:  11.0
  0.50:  28.0
  0.98: 340.0
  1.00: 512.0
}

// A dispersion payoff: the top 2% of hours against the bottom half, at 85%
// round-trip efficiency. This is the shape a battery earns and the reason a
// scalar spread cannot express it.
stream battery.arbitrage on entity asset.battery inflow currency USD {
  category operating.revenue.energy
  schedule every month from 2026-01 to 2026-03
  amount = (quantile_mean("prices", 0.98, 1.0) - quantile_mean("prices", 0.0, 0.5) / 0.85) * 100.0
}

// Step over the same slice, to hold the two apart.
stream battery.arbitrage_step on entity asset.battery inflow currency USD {
  category operating.revenue.energy
  schedule every month from 2026-01 to 2026-03
  amount = quantile_mean("prices_step", 0.0, 0.5) * 100.0
}

// quantile_of inverts quantile_at: a stated THRESHOLD becomes a share, which
// is what a lease breakpoint or a tranche attachment point needs.
stream battery.inverse on entity asset.battery inflow currency USD {
  category operating.revenue.energy
  schedule every month from 2026-01 to 2026-03
  amount = quantile_of("prices", 28.0) * 1000.0 + quantile_at("prices", 0.5)
}

// A COMPUTED SLICE IS STILL RECORDED.
//
// The lower bound comes from an input, so the compiler cannot resolve this
// call to a number. It is published WITHOUT a value rather than dropped: a
// call site missing from the audit record would read as a model that never
// made one, which is the failure mode the record exists to prevent.
assume tail_start = 0.9

stream battery.computed_slice on entity asset.battery inflow currency USD {
  category operating.revenue.energy
  schedule every month from 2026-01 to 2026-03
  amount = quantile_mean("prices", inputs.tail_start, 1.0) * 10.0
}
```

## recurrence_reads_settled_cash

```cfdl
version 0.1
model "recurrence-reads-settled-cash"
time calendar monthly from 2026-01 for 12

// A FIELD'S RULE READS THE CASH THAT ARRIVED.
//
// Occupancy rises with realised revenue. Before the period walk this nulled
// the whole expression — the series read bound nothing, the substituted zero
// propagated through `prev`, and occupancy collapsed to 0.0 from period 1 with
// the run still reporting ok (`docs/13` §7.71).
//
// Now the rule reads settled history, strictly backward: at period `t` the
// cash of `t - 1` has already been computed by the walk. The circularity that
// makes this look dangerous is not there — occupancy at `t` depends on revenue
// at `t - 1`, which depended on occupancy at `t - 1`. Every edge points
// backward, so there is nothing to solve and nothing to iterate.

entity asset property : Asset.Financial {
  occupancy init 0.80 next prev + series_sum("rent.total", time.t - 1, time.t - 1) / 10000000.0
  base_rent = 100000.0
}

stream rent.total on entity asset.property inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = asset.property.occupancy * asset.property.base_rent
}
```

## reserve_interest_on_balance

```cfdl
version 0.1
model "interest-on-a-reserve-balance"
time calendar monthly from 2026-01 for 12

entity asset project {
  name = "Project"
  // A FIELD's rule reads an account strictly backward — the prior close,
  // or the init (zero here) in the first period. The field carries the
  // interest forward and the stream spends it.
  reserve_interest init 0.0 next prev.dsra * 0.005
}
entity party sponsor { name = "Sponsor" }

// INTEREST EARNED ON A FUNDED RESERVE (`docs/13` §7.76, part three). The
// CREST reconciliation line this closes: `crest_solar_cost_based/NOTES.md`
// records that the reference EBITDA "includes interest earned on funded
// reserve accounts (~$4,606 in year one), which CFDL does not model."
//
// THE SPELLING MATTERS. `prev.<account>` is the OPENING balance — the prior
// close, or the account's init in the first period (`docs/42` §7), so the
// top-up funds from period 0 with nothing guarded. A stream may read it
// too since the balance generalized (`docs/42` §3.3); this fixture keeps the
// field so the interest is a rule the reader can see carried forward.
//
// The pin: the reserve funds toward 3,000 out of 1,000/month of revenue from
// the first period, and interest accrues on the PRIOR balance at 0.5% — 0
// in the first period, 5.00 on the first 1,000, 10.03 on 2,005, then 15.00
// a month once the target is held. Reading the balance strictly backward is
// what keeps the reserve and the interest it earns from being mutually
// circular.

// A debt service reserve funded to target out of operating cash.
account dsra { }

stream ops.revenue on entity asset.project inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 1000
}

// Interest EARNED on the reserve balance. Reads the account strictly
// backward, which is what makes it legal and cycle-free.
stream ops.reserve_interest on entity asset.project inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = asset.project.reserve_interest
}

waterfall funding on entity asset.project {
  schedule every month from 2026-01 to 2026-12
  from available
  pay top_up   to account dsra   = max(0.0, 3000.0 - prev.dsra)
  pay residual to party.sponsor  = remaining
}
```

## rule_reads_literal_field

```cfdl
version 0.1
model "rule-reads-literal-field"
time calendar annual from 2026-01 for 4

// A RULE READS A STATED FACT.
//
// `amortization` is a literal: a constant, the same in every period. A rule may
// read it, because there is no ordering question and nothing to sequence.
//
// This is what a field is for. Written the other way —
//
//     balance init 100.0 next prev - 10.0
//
// — the amount is stated twice, in the literal and in the rule, and nothing
// keeps them in step. The whole argument for declaring `amortization` once is
// that everything reads it.
//
// WHAT A RULE STILL MAY NOT READ is a field that MOVES. Its period-close value
// has not been computed yet inside another rule, and `E1127` rejects that at
// compile time rather than letting it resolve to nothing. Binding literals and
// only literals is what makes that diagnostic honest: the validator permits
// exactly what the engine resolves.

entity asset pool : Asset.Financial {
  amortization = 10.0
  balance init 100.0 next max(0.0, prev - asset.pool.amortization)
}

entity party lender : Party { name = "Lender" }

stream credit.principal on entity asset.pool outflow currency USD {
  schedule every year from 2027-01 to 2029-01
  amount = asset.pool.amortization
}
```

## run_declared_monte_carlo

```cfdl
version 0.1
model "run-declared-monte-carlo"
time calendar monthly from 2026-01 for 6

// A MODEL DECLARES ITS OWN RUN MODE, and the engine honours it.
//
// `run monte_carlo trials N seed S` is specified in `docs/01` §15.1, and the
// engine reads it from the IR's `runs` when the run configuration does not
// ask for a Monte Carlo run of its own. Nothing exercised that: every other
// Monte Carlo fixture supplies the mode in `run.json`, so the pickup path was
// reachable only from a spelling no model used — the construct was specified,
// implemented, and untested.
//
// Found by mutation testing (`docs/30`): five mutants in the pickup condition
// survived — its `&&` flipped to `||`, its `==` to `!=`, and its `trials > 0`
// three ways — because no blessed fixture could tell the difference.
//
// EVERYTHING IS IN THE MODEL. The distributions are `assume ~` declarations
// and the trials and seed are declared here, so `run.json` carries only a
// discount rate. The seed is stated, so the draws and every published
// quantile are reproducible: a Monte Carlo run IS a golden when its seed is.

entity asset borrower : Asset.Financial

assume rent    ~ Uniform(min=900, max=1100)
assume vacancy ~ Triangular(min=0.0, mode=0.05, max=0.15)

stream lease.rent on entity asset.borrower inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = inputs.rent * (1 - inputs.vacancy)
}

run monte_carlo trials 8 seed 424242
```

## run_dists_full

```cfdl
version 0.1
model "run-dists-full"
time calendar monthly from 2026-01 for 12

entity asset plant : Asset.Real

// Drivers supplied entirely by run.json, exercising every distribution kind
// the language offers plus clip.
stream plant.revenue on entity asset.plant inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = inputs.n + inputs.u + inputs.t + inputs.l + inputs.f
}
```

## scenario_compare

```cfdl
version 0.1
model "scenario-compare"
time calendar monthly from 2026-01 for 6

entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower {
  schedule every month from 2026-01 to 2026-06
  amount = 1000
}
```

## schedule_annual_stride

```cfdl
version 0.1
model "schedule-annual-stride"
time calendar monthly from 2026-01 for 24
entity asset co : Asset.Financial

stream co.true_up on entity asset.co outflow currency USD {
  schedule every year from 2026-01 to 2027-12
  amount = 5000
}
```

## schedule_conventions

```cfdl
version 0.1
model "schedule-conventions"
time calendar daily from 2026-07-01 for 10

entity asset borrower : Asset.Financial

// Jul 4 2026 is a Saturday; "following" on the US calendar rolls the payment
// to Monday Jul 6 (Fri Jul 3 is the observed Independence Day holiday).
stream fee.rolled_payment on entity asset.borrower inflow currency USD {
  schedule on 2026-07-04 convention following calendar "us"
  amount = 100
}

// Daily accrual with one explicit skip and one explicit extra date.
stream fee.daily on entity asset.borrower outflow currency USD {
  schedule every day from 2026-07-01 to 2026-07-05 except [2026-07-03] also [2026-07-08]
  amount = 10
}
```

## schedule_day_rules

```cfdl
version 0.1
model "schedule-day-rules"
time calendar daily from 2026-01-01 for 90
entity asset co : Asset.Financial

// Day-of-month places the occurrence inside its interval.
stream co.rent on entity asset.co inflow currency USD {
  schedule every month on day 15 from 2026-01-01 to 2026-03-31
  amount = 100
}

// End-of-month clamps to each month's real length (February = 28).
stream co.sweep on entity asset.co outflow currency USD {
  schedule every month on eom from 2026-01-01 to 2026-03-31
  amount = 50
}
```

## schedule_mid

```cfdl
version 0.1
model "schedule-mid"
time calendar annual from 2026-01 for 4
entity asset co : Asset.Financial

// Mid-period: cash summarized at the midpoint of the period that earned it.
// Half a period on every calendar, because it is a convention and not a date.
stream co.flow_mid on entity asset.co inflow currency USD {
  schedule every year mid from 2026-01 to 2029-01
  amount = 100
}

// The default is the period's end — the same series, discounted further.
stream co.flow_end on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2029-01
  amount = 100
}

// `start` is the other end of the axis: the period's open.
stream co.flow_due on entity asset.co inflow currency USD {
  schedule every year start from 2026-01 to 2029-01
  amount = 100
}

// A one-shot takes the same modifier. A price struck at a point in time would
// not use it; a lump treated as arriving across its period does.
stream co.oneshot_mid on entity asset.co inflow currency USD {
  schedule on 2028-01 mid
  amount = 1000
}
```

## schedule_placement_axis

```cfdl
version 0.1
model "schedule-placement-axis"
time calendar annual from 2026-01 for 4

// WHERE IN ITS PERIOD A FLOW SITS — one axis, three positions, both forms.
//
// `start`, `mid` and `end` are alternatives, not flags, so a schedule states
// at most one and stating two is a parse error rather than a diagnostic. The
// engine reads a single `placement`, which is why the contradictory state is
// unwritable instead of rejected.
//
// The forms differ only in which position they DEFAULT to, and that is why
// every position is nameable in both. A recurrence defaults to `end` — an
// ordinary annuity, where the interval elapses and then payment falls. A
// one-shot defaults to `start`: it settles on the date stated, not after
// waiting through a period it never waited through.
//
// The one-shot `end` is what a disposal needs and what no surface syntax
// could say until now. A reversion is taken at the close of the holding
// period, so a year-5 sale discounts five periods rather than four — on an
// annual model a whole year, and 9% of the reversion at 12%. A pack rule
// could reach it through `schedule_placement`; a hand-written model could not.
//
// Offsets are the assertion: 0.0 at the open, 0.5 halfway, 1.0 at the close.

entity asset co : Asset.Financial

stream one.start on entity asset.co inflow currency USD {
  schedule on 2027-01 start
  amount = 100.0
}

stream one.mid on entity asset.co inflow currency USD {
  schedule on 2027-01 mid
  amount = 100.0
}

stream one.end on entity asset.co inflow currency USD {
  schedule on 2027-01 end
  amount = 100.0
}

stream rec.start on entity asset.co inflow currency USD {
  schedule every year start from 2026-01 to 2028-01
  amount = 100.0
}

stream rec.mid on entity asset.co inflow currency USD {
  schedule every year mid from 2026-01 to 2028-01
  amount = 100.0
}

stream rec.end on entity asset.co inflow currency USD {
  schedule every year end from 2026-01 to 2028-01
  amount = 100.0
}
```

## schedule_quarterly_grid

```cfdl
version 0.1
model "schedule-quarterly-grid"
time calendar quarterly from 2026-01 for 8

entity asset co : Asset.Financial

// No pack, no domain logic: this fixture exists so that a regression on a
// quarterly calendar is attributable to the calendar itself rather than to
// pack lowering. Until the cadence work there was no quarterly model in the
// repo at all, so nothing exercised quarterly period stepping end to end.
stream co.rent on entity asset.co inflow currency USD {
  schedule every quarter from 2026-01 to 2027-10
  amount = 30000
}

// An annual true-up on a quarterly grid: coarser than the calendar, which is
// allowed, and lands once per four periods.
stream co.true_up on entity asset.co outflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 5000
}
```

## schedule_quarterly_stride

```cfdl
version 0.1
model "schedule-quarterly-stride"
time calendar monthly from 2026-01 for 24
entity asset co : Asset.Financial

// A quarterly stream on a monthly grid pays 8 times over two years, not 24.
stream co.coupon on entity asset.co inflow currency USD {
  schedule every quarter from 2026-01 to 2027-12
  amount = 1000
}
```

## schedule_weekly_daily_grid

```cfdl
version 0.1
model "schedule-weekly-daily-grid"
time calendar daily from 2026-01-01 for 120

entity asset co : Asset.Financial

// A weekly schedule is representable on a daily grid: each occurrence gets
// its own period. This is the case the interval vocabulary exists for — a
// weekly grid is not representable, a weekly schedule on a daily one is.
stream co.payroll on entity asset.co outflow currency USD {
  schedule every week from 2026-01-01 to 2026-03-31
  amount = 5000
}
```

## series_depth_chain

```cfdl
version 0.1
model "series-depth-chain"
time calendar annual from 2026-01 for 3

// A CHAIN OF SERIES READS, TWO DEEP — the shape the two-phase engine refused
// outright and dependency-ordered waves evaluate. `mid` reads `base`, and
// `top` reads `mid`: three waves, each stream evaluating against a store in
// which everything it names is already finished.
//
// The arithmetic is chosen to prove the ordering, not just permit it.
//   base = 100 each period            -> [100, 100, 100], total 300
//   mid  = running total of base      -> [100, 200, 300], total 600
//   top  = running total of mid       -> [100, 300, 600], total 1000
// `top` reaches 1000 only if `mid` was COMPLETE when `top` evaluated. The old
// engine's sealed store would have handed `top` nothing — which is exactly why
// it refused the chain rather than report a plausible zero.

entity asset co : Asset.Financial

stream base.revenue on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 100
}

stream mid.cumulative on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = series_sum("base.revenue", 0, time.t)
}

stream top.cumulative on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = series_sum("mid.cumulative", 0, time.t)
}
```

## series_reductions

```cfdl
version 0.1
model "series-reductions"
time calendar annual from 2026-01 for 4

// SUM AND MEAN WERE THE WHOLE REDUCTION VOCABULARY.
//
// `docs/13` §7.86. Peak outstanding debt, the worst period, a compounding
// factor, a count of periods that paid — none was expressible over a series,
// and the miss LOOKED LIKE A HIT: `min`/`max`/`sum`/`avg` are variadic SCALAR
// folds, so `max(series_sum("dbt.*", 0, 3))` compiles, runs, and returns the
// lifetime net as a one-element fold, silently labelled as a peak. The tell is
// below: `peak_wrong` and `lifetime` are the same number.
//
// EVERY REDUCTION FOLDS THE PER-PERIOD AGGREGATE. When a selector matches
// several streams they are added together within each period first, and the
// fold runs over that one series. Addition is associative so the order never
// showed for a sum; for a maximum it decides the answer.
//
// A PEAK BALANCE IS A FOLD OVER THE BALANCE. `series_max("dbt.*")` is the
// largest per-period NET FLOW, which is a different question. Peak outstanding
// debt is `series_max` over the series that CARRIES the balance — an entity
// field, readable in a metric under the key results publishes it under, which
// is §7.85. The two entries close this together.

entity asset tlb : Asset.Financial {
  // 6,000 drawn, 4,000 more, then repaid 3,500 a year: 6,000 / 10,000 /
  // 6,500 / 3,000. The peak is 10,000 and it is in the middle, so a fold
  // that took the first or the last value would not find it.
  balance init 6000.0 next max(0.0, prev + if(time.t == 1, 4000.0, 0.0) - if(time.t >= 2, 3500.0, 0.0))
  // A VARYING growth path: 1.05 then 1.03 thrice. pow(1.03, 4) is not this
  // number, which is the whole reason `series_prod` is not `pow`.
  growth  init 1.05 next 1.03
}

stream dbt.draw on entity asset.tlb inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = if(time.t == 0, 6000, 4000)
}

stream dbt.repay on entity asset.tlb outflow currency USD {
  schedule every year from 2028-01 to 2029-01
  amount = 3500
}

// THE TRAP, pinned so it cannot come back silently. Both are the lifetime net.
metric lifetime   = series_sum("dbt.*", 0, 3)
metric peak_wrong = max(series_sum("dbt.*", 0, 3))

// THE ANSWER: a fold over the balance the model carries. 10,000.
metric peak_debt   = series_max("asset.tlb.balance", 0, 3)
metric trough_debt = series_min("asset.tlb.balance", 0, 3)

// Over the FLOWS, which is a different and also useful question: the largest
// single period's net movement, and the worst.
metric largest_flow  = series_max("dbt.*", 0, 3)
metric smallest_flow = series_min("dbt.*", 0, 3)

// A cumulative product under a varying rate: 1.05 * 1.03 * 1.03 * 1.03.
// Decimal, with no helper stream — the `exp(series_sum(ln(...)))` identity
// needs one, and a helper stream must be inflow or outflow, so it is CASH.
metric compounded = series_prod("asset.tlb.growth", 0, 3)

// How many periods carried a non-zero aggregate. The draw stream pays in two
// of the four; the two debt streams together leave no period at zero.
metric draw_periods = series_count("dbt.draw", 0, 3)
metric busy_periods = series_count("dbt.*", 0, 3)

// A SELECTION THAT MATCHES NOTHING HAS NO MAXIMUM — and that is not an error.
//
// Each fold has its own honest answer for an empty selection: nothing sums to
// 0, multiplies to 1, and no period carries a non-zero aggregate. A maximum
// has none, so it publishes NULL, which is the language's existing word for
// absent (an entity state no event has set is one; a ratio's undefined period
// publishes as one) and carries exactly the right guard rails — ordering and
// arithmetic on a null are errors, so the absence can never quietly become a
// number.
//
// Null rather than a hard error, because an error leaves the model no way to
// SAY that a selector may legitimately be empty. `guarded` below is that
// sentence, and it is why this is a value and not a refusal.
metric empty_max   = series_max("nothing.*", 0, 3)
metric empty_sum   = series_sum("nothing.*", 0, 3)
metric empty_count = series_count("nothing.*", 0, 3)
metric guarded     = if(series_count("nothing.*", 0, 3) == 0, 0, series_max("nothing.*", 0, 3))
```

## slice_by_line

```cfdl
version 0.1
model "slice-by-line"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

contract cre.lease_unit tenant_a on entity asset.tower {
  term 2026-01..2027-12
  terms {
    rent_year = 480000
    free_rent_months = 2
  }
}

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2027-12
  terms {
    principal = 3000000
    interest_rate = 0.06
    amortization_months = 300
  }
}

// A LINE IS SELECTED BY ROLE (docs/40 §6). The master names what every
// debt produces — interest — and each pack rule names the line it emits,
// so `line interest` reaches the interest of every debt whichever pack
// lowered it and whatever category that pack spelled. The clauses
// intersect: with `type Contract.Debt` beside it, only debt's interest.
slice debt_interest {
  type Contract.Debt
  line interest
}

// A line alone, across every type that produces one.
slice all_rent {
  line rent
}

// Type alone, as before: every stream lowered from anything that is_a Debt.
slice all_debt {
  type Contract.Debt
}
```

## slice_by_type

```cfdl
version 0.1
model "slice-by-type"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 60

// BASE-TYPE SELECTION, THE REFINEMENT'S PAYOFF (docs/13 §7.92, §7.90).
// `type Contract.Debt` names a MASTER; the compiler expands it through the
// recorded refinement — CRE.Contract.PermanentDebt is_a Contract.Debt — to
// every stream that contract lowers. The slice survives a new pack
// unchanged, because it names what the thing IS, not how a pack spells it.

entity asset tower : CRE.Asset.RealProperty

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2030-12
  terms {
    principal = 6000000
    interest_rate = 0.055
    amortization_months = 300
    interest_only_months = 24
    balloon_at_maturity = 1
  }
}

stream ops.noi on entity asset.tower inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  category operating.revenue.other
  amount = 55000
}

slice debt {
  type Contract.Debt
}
```

## slice_window

```cfdl
version 0.1
model "slice-window"
time calendar annual from 2026-01 for 5

// A SLICE MAY BOUND THE PERIODS IT REPORTS.
//
// `docs/13` §7.55. A slice selects streams — by entity, type, category or name
// — and until now it selected all of their periods. "The 2027 to 2028 result
// for this asset" was not expressible, so a reader wanting it left the language
// and did the arithmetic somewhere else.
//
// A WINDOW IS NOT A PHASE. A phase is a lifecycle anchor: `phase_start()` and
// `phase_end()` drive schedules, and the model's own periods are named by it.
// A window is a reporting bound, applied after the walk has finished. One
// construct doing both jobs would mean neither could move without the other.
//
// Dates, not period indices. An index is a fact about one grid; a window that
// survives a change of calendar has to be stated in dates, which is how every
// other range in the language is stated.
//
// The rent is 100 a year for five years. `all_years` totals 500; `middle`
// bounds itself to 2027-2028 and totals 200. The window is published in the
// slice's own selection, so a reader can see WHY the two differ rather than
// having to infer it.

entity asset co : Asset.Financial

stream ops.rent on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  category operating.revenue.base_rent
  amount = 100
}

slice all_years {
  entity asset.co
}

slice middle {
  entity asset.co
  window from 2027-01 to 2028-01
}
```

## slices

```cfdl
version 0.1
model "slices"
time calendar annual from 2026-01 for 3

// A SLICE IS A NAMED, DELIBERATELY PARTIAL SELECTION (docs/13 §7.90).
// Clause kinds intersect, values within a kind union, `except` subtracts —
// and a slice carries NO reconciliation block, because the absence is what
// the declaration means: a partial number must not dress as a complete one.
//
// Two artists under one label. Slicing is the question the results plane
// could not answer before the graph: one artist's cash, one artist's
// royalties only, and the label minus a discontinued line.

entity container label : Container.Portfolio
entity asset artist_a : Asset.Intangible { part of container.label }
entity asset artist_b : Asset.Intangible { part of container.label }

stream a.royalties on entity asset.artist_a inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.royalty
  amount = 100
}
stream a.merch on entity asset.artist_a inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.merchandise
  amount = 40
}
stream b.royalties on entity asset.artist_b inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.royalty
  amount = 70
}

// All of artist A: entity clause selects the entity and its descendants.
slice artist_a {
  entity asset.artist_a
}

// Artist A's royalties only: kinds intersect.
slice artist_a_royalties {
  entity asset.artist_a
  category "operating.revenue.royalty"
}

// The whole label, minus merchandise: include by entity, subtract by
// category. The label is a container; its scope arrives through part_of.
slice label_ex_merch {
  entity container.label
  except category "operating.revenue.merchandise"
}
```

## stable_identity

```cfdl
version 0.1
model "stable-identity"
time calendar annual from 2026-01 for 2

// A STABLE IDENTITY IS CARRIED, NEVER INTERPRETED (docs/13 §7.91). The
// layer above the model assigns canonical ids to real-world things; the
// literal field `id` is where a model carries one. The engine ignores it
// entirely and republishes it in the results graph, which is what makes a
// package's numbers attributable to canonical things — a consumer joins on
// the id, not on symbol names and hope.
//
// The graph beside it is §7.43's answer: symbol, family, type, parent —
// the hierarchy view derivable from results alone.

entity container fund : Container.Fund { id = "evs:container/fund-274" }

entity asset alpha : Asset.Financial {
  id = "evs:asset/alpha-91"
  part of container.fund
}

stream alpha.income on entity asset.alpha inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  category operating.revenue.other
  amount = 100
}
```

## state_anchored_schedule

```cfdl
version 0.1
model "state-anchored-schedule"
time calendar monthly from 2026-01 for 24

// "18 MONTHS OF CONSTRUCTION FROM WHENEVER CONSTRUCTION STARTS" — the third
// schedule anchor (docs/28 §6.2), pinned as §9's delayed, re-anchored
// construction schedule. The machine delays the start: nothing accrues while
// waiting. Entry at t=4 opens a six-period window; the pause at t=8 masks
// the tail through `active in state` — the window is presence, the state is
// activity, and the two compose. Re-entry at t=10 RE-ANCHORS: a fresh
// six-period window, not the remnant of the old one, which is what a second
// entry means.
lifecycle project {
  initial waiting
  state waiting, building, paused
  waiting -> building when time.t == 4
  building -> paused  when time.t == 8
  paused -> building  when time.t == 10
}

entity asset site {
  lifecycle project
}

stream capex.build on entity asset.site outflow currency USD {
  schedule every month from state_enter(asset.site, building) for 6 periods
  amount = 100
  active in state building
}
```

## statement_authored_rows

```cfdl
version 0.1
model "statement-authored-rows"
time calendar annual from 2026-01 for 3

// A STATEMENT MAY STATE ITS OWN ROWS.
//
// `docs/13` §7.55. A generated statement names a `structure` and a `depth` and
// the rows follow from the tree, which is right when the tree IS the
// presentation. A pro forma is not that: its rows carry curated labels, its
// expenses are shown positive under "Less:", and it has a coverage ratio at the
// bottom that is a node of no hierarchy at all.
//
// AUTHORED OR GENERATED, NEVER BOTH (`E1369`). A generated statement partitions
// the cash by construction; an authored one partitions it by the author's care.
// Mixed, neither holds — an authored line beside generated rows would claim
// streams the generated rows already claimed, and the bottom line would
// double-count.
//
// THE DISPLAY SIGN NEVER CHANGES WHAT IS SUMMED. `values` carries the signed
// amount, so a consumer that ignores `display_sign` still adds up correctly;
// the sign is how a reader is shown the number, not what it is.
//
// A RATIO DIVIDES TWO DECLARED SLICES. A slice is already a named selection
// with a per-period net, so a ratio needs no row identifiers — and a coverage
// ratio in a period with no debt service publishes `null`, not zero, which is
// the rule a pack ratio already follows.

entity asset property : Asset.Financial

stream ops.rent on entity asset.property inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 1000
}

stream ops.opex on entity asset.property outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.expense.other
  amount = 300
}

stream fin.debt_service on entity asset.property outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category financing.debt.service
  amount = 400
}

slice noi           { category "operating.*" }
slice debt_service  { category "financing.debt.service" }

statement operating {
  label "Operating statement"

  line     "Base rental revenue"    { category "operating.revenue.base_rent" }
  line     "Less: operating costs"  { category "operating.expense.*" display positive }
  subtotal "Net operating income"   { category "operating.*" depth 0 }
  spacer
  line     "Debt service"           { slice debt_service display positive }
  // DEBT SERVICE IS STORED NEGATIVE, because an outflow is negative cash, so
  // NOI over it is arithmetically -1.75. No practitioner writes a coverage
  // ratio negative. `display` flips how the number is SHOWN and leaves
  // `values` signed, which is the same rule every other row follows: a
  // consumer that ignores the display sign still reads the arithmetic.
  ratio    "DSCR"                   { of noi to debt_service display positive }
}
```

## statement_by_entity

```cfdl
version 0.1
model "statement-by-entity"
time calendar annual from 2026-01 for 3

// A MODEL DECLARES HOW ITS RESULTS ARE ORGANISED.
//
// `docs/13` §7.55. A statement is the ORGANISING STRUCTURE — whether the
// presentation is an entity hierarchy, a category hierarchy, or something else.
// It is NOT a list of rows: the rows come from the structure, and `depth` sets
// the level of aggregation, so an interior node IS a subtotal by virtue of
// where it sits. That is how a statement carries dozens of rows without dozens
// of declarations, and it is why a model needs no subtotal construct.
//
// The hierarchy here is the one `graph` already publishes: `part of`. The fund
// holds two properties, each with its own rent. At depth 1 the fund is one
// line; at depth 2 the properties appear beneath it and the fund becomes their
// subtotal — the same tree, two levels of aggregation.

entity container fund : Container.Fund

entity asset north : Asset.Financial { part of container.fund }
entity asset south : Asset.Financial { part of container.fund }

stream north.rent on entity asset.north inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 100
}

stream south.rent on entity asset.south inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 60
}

statement portfolio {
  label     "Portfolio by property"
  structure entity
  depth     2
}

// THE SAME TREE, ONE LEVEL UP. The fund's children are now cut off by `depth`,
// so the fund is a LINE carrying all of their cash rather than a subtotal over
// rows beneath it. That rule — a node whose children are shown is a subtotal, a
// node whose children are cut off is a line — is what keeps the bottom line
// reconciling at EVERY depth: the lines always partition the cash, whichever
// level you cut at.
statement portfolio_summary {
  label     "Portfolio, one line"
  structure entity
  depth     1
}

// THE OTHER HIERARCHY, over the same cash. A category is a dotted path, so its
// levels are structural and need no declaring either.
statement by_category {
  label     "By category"
  structure category
  depth     3
}

// A FILTER, ORTHOGONAL TO THE STRUCTURE. A slice narrows WHICH cash; the
// statement decides HOW it is organised. The same entity hierarchy over the
// north property alone, bounded to the first two years — 100 a year, so 200
// against the portfolio's 480.
//
// The bottom line reconciles against the MODEL, so a filtered statement shows
// a residual: that is the completeness guarantee doing its job rather than
// failing. A statement that hides cash should say how much.
slice north_early {
  entity asset.north
  window from 2026-01 to 2027-01
}

statement north_only {
  label     "North, first two years"
  structure entity
  depth     2
  slice     north_early
  metrics   total_rent
}

metric total_rent = series_sum("north.rent", 0, 2) + series_sum("south.rent", 0, 2)
```

## statement_generated_order

```cfdl
version 0.1
model "statement-generated-order"
time calendar annual from 2026-01 for 2

// A GENERATED STATEMENT READS AS A HIERARCHY, AND IN THE RIGHT ORDER.
//
// `docs/13` §7.55. Two things were wrong, and a single-root single-category
// fixture could show neither.
//
// DEPTH FIRST. Sorting rows by (depth, symbol) is breadth first: two funds
// holding two properties each came out as both funds followed by all four
// properties in one flat block, with nothing saying which belonged to which.
// A parent is now followed by its own subtree.
//
// THE CATEGORY ROOTS HAVE A CANONICAL ORDER and it is not alphabetical.
// `CATEGORY_ROOTS` is operating, investing, financing — the order a cash flow
// statement is read in — and iterating the set alphabetically put financing
// first. Below a root there is no canonical order, so siblings sort
// alphabetically: arbitrary, but stated and stable.
//
// LABELS ARE DERIVED. A generated statement is meant to need no declarations,
// and a row reading `operating.revenue.base_rent` is a presentation that has
// not been presented. The last segment, underscores opened out, capitalised.

entity container north_fund : Container.Fund
entity container south_fund : Container.Fund

entity asset alpha : Asset.Financial { part of container.north_fund }
entity asset beta  : Asset.Financial { part of container.north_fund }
entity asset gamma : Asset.Financial { part of container.south_fund }

stream alpha.base_rent on entity asset.alpha inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  category operating.revenue.base_rent
  amount = 100
}

stream beta.base_rent on entity asset.beta inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  category operating.revenue.base_rent
  amount = 50
}

stream gamma.sale on entity asset.gamma inflow currency USD {
  schedule every year from 2027-01 to 2027-01
  category investing.disposition.sale
  amount = 900
}

stream gamma.debt on entity asset.gamma outflow currency USD {
  schedule every year from 2026-01 to 2027-01
  category financing.debt.service
  amount = 40
}

statement by_entity  { structure entity   depth 2 }
statement by_section { structure category depth 2 }
```

## statement_grain

```cfdl
version 0.1
model "statement-grain"
time calendar monthly from 2026-01 for 12

// A STATEMENT REPORTS AT ITS OWN GRAIN, AND A RATIO IS RECOMPUTED AT IT.
//
// `docs/13` §7.55. Two defects, both latent: no fixture used `grain` on a
// model statement, so nothing in the corpus exercised either.
//
// THE ROWS WERE NOT BUCKETED. An annual statement over a monthly model
// published two grain labels against twenty-four monthly values — a reader
// lining values up against labels read one month as the year, and had
// twenty-two values with no label to sit under. `total` is the lifetime figure
// over the RAW periods; `values` is the series the labels describe.
//
// A RATIO IS NOT RE-BUCKETED, IT IS RECOMPUTED. An annual coverage ratio is
// annual NOI over annual debt service. It is NOT the mean of twelve monthly
// ratios and no function of a column of ratios gives it. Debt service here is
// 20 a month for six months and 100 for six, so the two answers differ and the
// fixture can tell them apart:
//
//   annual NOI / annual DS  = 1,440 / -720 = -2.0   <- correct
//   mean of monthly ratios  = (6*-6 + 6*-1.2) / 12  = -3.6
//
// The pack renderer takes the subtotal SPECS for exactly this reason: handing
// an arm the ratio values and a grain makes averaging them the obvious thing
// to write. Here the inputs are two slices, so the same discipline is to
// divide AFTER bucketing.

entity asset co : Asset.Financial

stream ops.noi on entity asset.co inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  category operating.revenue.base_rent
  amount = 120
}

stream fin.ds on entity asset.co outflow currency USD {
  schedule every month from 2026-01 to 2026-12
  category financing.debt.service
  amount = if(time.t < 6, 20, 100)
}

slice noi { category "operating.*" }
slice ds  { category "financing.debt.service" }

statement by_month {
  label "Monthly"
  line  "Net operating income" { slice noi }
  ratio "DSCR"                 { of noi to ds display positive }
}

statement by_year {
  label "Annual"
  grain annual
  line  "Net operating income" { slice noi }
  ratio "DSCR"                 { of noi to ds display positive }
}
```

## statement_residual

```cfdl
// A statement must never lose cash quietly.
//
// `misc.windfall` carries no category, so no row of the CRE operating
// statement can claim it. The statement therefore emits a visible
// `residual` row holding it, and W3500 names the stream — rather than
// omitting the money and leaving a bottom line that is short with no signal.
//
// The reconciliation still closes, because the residual row is part of the
// bottom line. That is the point: the gate makes the omission VISIBLE, it does
// not paper over it.
version 0.1
model "residual-probe"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 3
entity asset tower : CRE.Asset.RealProperty

// Classified: lands in the statement's revenue row.
stream cre.unit.base_rent.a on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 1000
}

// Classified, and claimed by no ROW. That is the residual case that remains
// reachable: `E5029` makes an uncategorised stream an error while a pack is
// active, so a residual can no longer be produced by saying nothing. It is
// produced by saying something the statement has no row for — which is the
// case a modeller actually hits, naming their own instance (docs/35 §1.4).
stream misc.windfall on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.windfall
  amount = 250
}
```

## statement_row_by_type

```cfdl
version 0.1
model "statement-row-by-type"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

contract cre.lease_unit tenant_a on entity asset.tower {
  term 2026-01..2027-12
  terms {
    rent_year = 480000
  }
}

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2027-12
  terms {
    principal = 3000000
    interest_rate = 0.06
    amortization_months = 300
  }
}

// AN AUTHORED ROW DRAWS BY TYPE AND LINE, as a slice does: the compiler
// expands the clauses to exact stream names and the row claims those, so
// the bottom line reconciles the same way a category row's does. A row
// written this way survives a change of pack: "every debt's interest"
// names no pack's category.
statement lender {
  label    "Lender view"
  line     "Rent"           { type Contract.Lease line rent }
  line     "Interest"       { type Contract.Debt line interest display positive }
  line     "Principal"      { type Contract.Debt line principal display positive }
  subtotal "Debt service"   { type Contract.Debt line interest line principal display positive }
}
```

## statement_series_row

```cfdl
version 0.1
model "statement-series-row"
time calendar annual from 2026-01 for 3

// A ROW MAY DRAW A PUBLISHED SERIES (`docs/13` §7.55).
//
// A series row presents a fold OF the ledger — `model.net_cash_flow` here —
// rather than cash in it, so it claims no streams and its figure stays out of
// the bottom line whatever kind the author gives it. The reconciliation below
// proves it: the claimed rows alone account for the model's cash, the memo
// line repeats the model's own net beside them, and no W3502 fires.
//
// AN UNCLAIMED STREAM NAMED BY NOTHING falls to the residual. `misc.sundry`
// carries no category and no row claims it, so it lands in the residual row
// under W3500 — a named stream is drawn only through a claim clause, never
// implied.

entity asset property : Asset.Financial

stream ops.rent on entity asset.property inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 1000
}

stream ops.opex on entity asset.property outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.expense.other
  amount = 300
}

stream misc.sundry on entity asset.property inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 50
}

statement memo {
  label "Operating memo"

  line     "Rental revenue"       { category "operating.revenue.*" }
  line     "Less: operating costs" { category "operating.expense.*" display positive }
  spacer
  // A MEMO OF THE MODEL'S OWN NET. Drawn as a series, so it reaches no
  // bottom line and doubles nothing.
  subtotal "Net cash flow (memo)" { series "model.net_cash_flow" }
  // A key this run does not publish renders no values and no total — never
  // a column of fabricated zeros.
  subtotal "NOI (pack memo)"      { series "domain.cre.noi" }
}
```

## term_expression

```cfdl
version 0.1
model "term-expression"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

// A term may hold an expression. This exact model was the INVALID fixture
// `term_trailing_tokens`: the arithmetic used to be discarded in silence, so
// it compiled as mwh_year = 1000 — then it became a parse error — and now it
// means what it says. The results must equal the same model stating 1500.
contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = 1000 + 500
    price = 3000
    escalation = 0.0
    degradation = 0.005
    availability = 1.0
  }
}
```

## term_expression_dynamic

```cfdl
version 0.1
model "term-expression-dynamic"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

assume esc_base = 0.02

// Volume varies by season; the contract reads the curve directly instead of
// carrying a *_curve twin term for the pack to compose.
curve production step {
  2026-01: 1400
  2026-04: 1700
  2026-10: 1300
}

// What was agreed is itself an expression: an escalator of "base plus 50bp",
// and a volume that follows the production curve. Neither needs a
// model-level assume to hold the arithmetic.
contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = curve_value("production", time.date)
    price = 3000
    escalation = inputs.esc_base + 0.005
    degradation = 0.0
    availability = 1.0
  }
}
```

## term_input_ref

```cfdl
version 0.1
model "term-input-ref"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset plant : Energy.Asset.GenerationFacility

// A contract term may defer to a declared input instead of stating a literal.
// The contract still records what was signed; the quantities that vary are
// named, and their values arrive from a scenario or a Monte Carlo draw.
assume annual_yield ~ Normal(mean=5000, stdev=350, clip=[4000, 6000])
assume degradation ~ Triangular(min=0.004, mode=0.005, max=0.007)

contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2027-12
  terms {
    // Contractual facts stay literal.
    price = 3000
    escalation = 0.0
    availability = 1.0
    // Drivers defer to inputs. `degradation` is range-checked by the energy
    // pack, so this also covers a validated term deferring.
    quantity = inputs.annual_yield
    degradation = inputs.degradation
  }
}
```

## term_units

```cfdl
version 0.1
model "unit-annotations"
use pack "energy" version "0.1.0"
time calendar annual from 2026-01 for 3

entity asset plant : Energy.Asset.GenerationFacility

// The unit is an assertion about what the number means. It agrees here.
contract energy.ptc on entity asset.plant {
  term 2026-01..2028-01
  terms {
    quantity        = 250000 "MWh/yr"
    amount  = 27.50 "USD/MWh"
  }
}
```

## transition_log

```cfdl
version 0.1
model "transition-log"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 5

// Entity state used to be UNOBSERVABLE in results. Nothing distinguished "the
// event fired and its target was misspelled" from "the event never fired", and
// a case could not assert that a transition happened at all. The transition
// log is that audit trail: period, entity, field, from, to, and the event that
// caused it.
//
// The building opens in its lifecycle's declared initial state, so the first
// transition has a real `from` rather than a null — before the ontology,
// `status` did not exist until an event wrote one.
entity asset tower : CRE.Asset.RealProperty {
  asset_class = "office"
  state stabilized
}

event start_repositioning when time.t >= 2 {
  set entity asset.tower.status = "repositioning"
}

// WHEN A WRITE BECOMES VISIBLE, which is two rules and not one:
//
//   - an EVENT or OPTION guard reads the state as the period OPENED, so every
//     guard in a period sees the same thing and declaration order cannot
//     change an answer;
//   - a STREAM reads the state as the period CLOSED, so a transition takes
//     effect in the period it fires.
//
// That is the synchronous discipline: transitions all evaluate against the
// current state, the state commits, then outputs read the committed result.
// Rent therefore stops in the SAME period the event fires, which is what the
// model reads as saying.
stream cre.rent on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  category operating.revenue.base_rent
  amount = 100
  active when asset.tower.status != "repositioning"
}
```

## trapped_cash_cure

```cfdl
version 0.1
model "trapped-cash-cure"
time calendar monthly from 2026-01 for 12

// TRAPPED CASH, CURED — accounts, the machine, and backward guard reads in
// one pin (docs/28 §9). A collections shortfall trips the trigger: the
// machine reads SETTLED cash strictly backward and moves to trapped. While
// trapped, the waterfall allocates the residual into the trap account
// instead of to the sponsor — the step reads the status the machine settled
// THIS period, stage 1 before stage 3. Collections resume, the cure edge
// sees them, and the release step hands the accumulated balance back. The
// trap's balance is the pin: 0 until the breach, the trapped months' cash
// while it holds, 0 again after the cure.
lifecycle trigger {
  initial normal
  state normal, trapped
  normal -> trapped when series_sum("ops.rent", time.t - 1, time.t - 1) < 50
  trapped -> normal when series_sum("ops.rent", time.t - 1, time.t - 1) >= 50
}

entity asset suite {
  lifecycle trigger
  paying init 1.0 next if(time.t == 3, 0.0, 1.0)
}

entity party sponsor { name = "Sponsor" }

account trap { }

stream ops.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 100 * asset.suite.paying
}

waterfall dist on entity asset.suite {
  schedule every month from 2026-01 to 2026-12
  from available
  pay trapped  to account trap  = if(asset.suite.status == "trapped", remaining, 0.0)
  pay residual to party.sponsor = remaining
}

waterfall release on entity asset.suite {
  schedule every month from 2026-01 to 2026-12
  from trap
  pay released to party.sponsor = if(asset.suite.status == "normal", remaining, 0.0)
}
```

## typed_entities

```cfdl
version 0.1
model "typed-entities"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 3

// Old form still parses — every model written before types existed.
entity asset legacy : CRE.Asset.RealProperty

// Typed, with attributes.
entity asset tower : CRE.Asset.RealProperty {
  asset_class = "office"
  rentable_area = 30000
  state stabilized
}

// Optional hierarchy: a unit inside the building. Never required.
entity asset suite_a : CRE.Asset.Unit {
  rentable_area = 12000
  part of asset.tower
  state leased
}

// A party — someone to contract with.
entity party acme : CRE.Party.Tenant {
  name = "Acme Corp"
}

stream cre.rent on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 100000
}
```

## use_pack_smoke

```cfdl
version 0.1
model "use-pack-smoke"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset borrower : Asset.Financial

contract test.lease_contract {
  term 2026-01..2026-12
}
```

## valuation_grain_annual

```cfdl
version 0.1
model "valuation-grain-annual"
time calendar monthly from 2026-01 for 24

// A RUN VALUES AT THE ANNUAL GRAIN, and the engine dispatches on it.
//
// `"valuation_grain": "annual"` sums cash into calendar years and discounts
// each bucket once at the ANNUAL rate — the convention published sources use,
// and the capability `mit_rentleg_plaza` needed when a 1.3% gap turned out to
// be the model's calendar silently deciding its valuation convention.
//
// The arithmetic under it was unit-tested (`npv_at_grain` against the two
// grains). The WIRING was not: no fixture and no benchmark run configuration
// set the key, so the match arm that reads it could be DELETED and every
// blessed number stayed the same. Mutation testing found exactly that
// (`docs/30`) — a published capability whose dispatch nothing exercised.
//
// Two years of level monthly cash makes the grain's effect visible rather
// than incidental: 24 payments discounted at their own fractional years is a
// different number from two annual buckets discounted once each.

entity asset property : Asset.Financial

stream ops.rent on entity asset.property inflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 10000
}

stream ops.expense on entity asset.property outflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 3000
}
```

## waterfall_abs_22_step

```cfdl
version 0.1
model "waterfall-abs-22-step"
use pack "credit" version "0.1.0"
time calendar monthly from 2017-02 for 6

// A REAL 22-STEP PRIORITY OF PAYMENTS, encoded in full.
//
// The structure is a consumer auto ABS: a servicer and a trustee paid ahead of
// the notes, five rated classes paid interest then principal in strict
// seniority, a reserve account topped to a specified level, an
// overcollateralization target that excess spread turbos toward, and a
// certificateholder taking whatever survives.
//
// This fixture is about EXPRESSIVENESS, not about numbers: the source
// specifies the waterfall and publishes no tranche schedule, so what it can
// prove is that the language says all twenty-two steps without an escape
// hatch. Numeric agreement is a separate case against a runnable reference.
//
// The twenty-two steps use seven rules, and every one is an ordinary
// expression:
//
//   a stated amount              = <expr>
//   capped, with a later overflow  = min(<expr>, <cap>)  /  owed.x - paid.x
//   pay down to a target         = <balance> - <target>
//   remaining balance on a date  = if(time.t >= <n>, <balance>, 0)
//   top an account to a level    = <level> - <balance>
//   everything that survives     = remaining

// THE DEAL'S OWN CAPITAL STRUCTURE, as the prospectus publishes it. The class
// sizes, coupons and final scheduled distribution dates are the transaction's;
// the pool and collection figures below are not, and are marked where they
// appear.
entity asset trust    : Credit.Asset.LoanPool { collateral_type = "auto"  original_balance = 984243280
  pool_balance init 960000000.0
               next prev * 0.96
  reserve_balance init 18000000.0
                  next prev
}
entity asset class_a1 : Credit.Asset.Tranche  { seniority = 1  original_balance = 182000000 }
entity asset class_a2a: Credit.Asset.Tranche  { seniority = 2  original_balance = 230000000 }
entity asset class_a2b: Credit.Asset.Tranche  { seniority = 2  original_balance =  75000000 }
entity asset class_a3 : Credit.Asset.Tranche  { seniority = 3  original_balance = 189000000 }
entity asset class_b  : Credit.Asset.Tranche  { seniority = 4  original_balance =  73370000 }
entity asset class_c  : Credit.Asset.Tranche  { seniority = 5  original_balance =  91080000 }
entity asset class_d  : Credit.Asset.Tranche  { seniority = 6  original_balance =  89550000 }
// The Class E Notes exist in the indenture and are retained by the depositor,
// so the prospectus states no size for them. Zero here, and steps 15-17 still
// appear: a step that pays nothing is a step the deal has.
entity asset class_e  : Credit.Asset.Tranche  { seniority = 7  original_balance =         0 }
entity asset reserve  : Credit.Asset.Tranche  { seniority = 9  original_balance =  19684866 }

entity party servicer    : Credit.Party.Servicer { name = "Servicer" }
entity party trustee     : Credit.Party.Issuer   { name = "Trustee" }
entity party sponsor     : Credit.Party.Issuer   { name = "Sponsor" }
entity party certificate : Credit.Party.Investor { name = "Certificateholder" }

// ILLUSTRATIVE, NOT THE DEAL'S. Available funds, the pool balance and the
// reserve balance are inputs a collateral engine would produce, and the
// prospectus publishes no period-by-period schedule for them. They are stated
// here so the waterfall has something to allocate; the capital structure above
// is the transaction's own.

assume servicing_fee   = 1845456.0     // 2.25%/12 on the pool
assume trustee_fee     = 90000.0
assume trustee_cap     = 75000.0       // the stated annual limit, per period
assume specified_reserve   = 19684866.0
assume principal_distributable = 2500000.0
assume oc_target       = 940000000.0

// COLLECTIONS, ILLUSTRATIVE. The prospectus publishes no period-by-period
// schedule, so the amounts are stated here as a stream — the trust's cash,
// which `available` hands the waterfall. The decay stands in for a pool
// amortizing; the capital structure above is the transaction's own.
// Undifferentiated collections, categorised as principal: the waterfall retires
// notes with them.
stream trust.collections on entity asset.trust inflow currency USD {
  schedule every month from 2017-02 to 2017-07
  category operating.collection.principal
  amount = 12500000.0 * pow(0.97, time.t)
}

waterfall abs.distribution on entity asset.trust {
  schedule every month from 2017-02 to 2017-07
  from available

  //  1. the servicer, and amounts the sponsor is entitled to retain
  pay servicing_fee     to party.servicer  = inputs.servicing_fee
  pay sponsor_amounts   to party.sponsor   = 0.0

  //  2. trustee, owner trustee, collateral agent and asset representations
  //     reviewer, subject to a stated annual limit. Step 21 pays whatever the
  //     cap held back, which is `owed - paid` and nothing else.
  pay trustee_fees      to party.trustee   = min(inputs.trustee_fee, inputs.trustee_cap)

  //  3. interest due on the Class A Notes — all four subclasses together, at
  //     the published coupons: A-1 0.92%, A-2-A 1.51%, A-2-B one-month LIBOR
  //     + 0.30%, A-3 1.87%.
  pay class_a_interest  to asset.class_a1  = 790350.0

  //  4. principal to reduce the Class A balance to the pool balance
  pay class_a_target    to asset.class_a1  = entity.asset.class_a1.original_balance
                                             + entity.asset.class_a2a.original_balance
                                             + entity.asset.class_a2b.original_balance
                                             + entity.asset.class_a3.original_balance
                                             - asset.trust.pool_balance

  //  5. the remaining Class A balance at its final scheduled distribution date
  pay class_a_final     to asset.class_a1  = if(time.t >= 5,
                                                entity.asset.class_a1.original_balance, 0.0)

  //  6. interest due on the Class B Notes, 2.30%
  pay class_b_interest  to asset.class_b   = 140625.83

  //  7. principal to reduce the COMBINED A+B balance to the pool balance,
  //     after giving effect to clauses 4 and 5 — which is what `paid.` reads.
  pay class_b_target    to asset.class_b   = entity.asset.class_a1.original_balance
                                             + entity.asset.class_a2a.original_balance
                                             + entity.asset.class_a2b.original_balance
                                             + entity.asset.class_a3.original_balance
                                             + entity.asset.class_b.original_balance
                                             - paid.class_a_target
                                             - paid.class_a_final
                                             - asset.trust.pool_balance

  //  8. the remaining Class B balance at its final scheduled date
  pay class_b_final     to asset.class_b   = if(time.t >= 5, entity.asset.class_b.original_balance, 0.0)

  //  9. interest due on the Class C Notes, 2.71%
  pay class_c_interest  to asset.class_c   = 205689.0

  // 10. principal, after clauses 4, 5, 7 and 8
  pay class_c_target    to asset.class_c   = entity.asset.class_a1.original_balance
                                             + entity.asset.class_a2a.original_balance
                                             + entity.asset.class_a2b.original_balance
                                             + entity.asset.class_a3.original_balance
                                             + entity.asset.class_b.original_balance
                                             + entity.asset.class_c.original_balance
                                             - paid.class_a_target
                                             - paid.class_a_final
                                             - paid.class_b_target
                                             - paid.class_b_final
                                             - asset.trust.pool_balance

  // 11. the remaining Class C balance at its final scheduled date
  pay class_c_final     to asset.class_c   = if(time.t >= 5, entity.asset.class_c.original_balance, 0.0)

  // 12. interest due on the Class D Notes, 3.13%
  pay class_d_interest  to asset.class_d   = 233576.25

  // 13. principal, after clauses 4, 5, 7, 8, 10 and 11
  pay class_d_target    to asset.class_d   = entity.asset.class_a1.original_balance
                                             + entity.asset.class_a2a.original_balance
                                             + entity.asset.class_a2b.original_balance
                                             + entity.asset.class_a3.original_balance
                                             + entity.asset.class_b.original_balance
                                             + entity.asset.class_c.original_balance
                                             + entity.asset.class_d.original_balance
                                             - paid.class_a_target
                                             - paid.class_a_final
                                             - paid.class_b_target
                                             - paid.class_b_final
                                             - paid.class_c_target
                                             - paid.class_c_final
                                             - asset.trust.pool_balance

  // 14. the remaining Class D balance at its final scheduled date
  pay class_d_final     to asset.class_d   = if(time.t >= 5, entity.asset.class_d.original_balance, 0.0)

  // 15. interest due, IF ANY, on the Class E Notes. Retained and sized at
  //     zero here, so the step pays nothing and still exists.
  pay class_e_interest  to asset.class_e   = 0.0

  // 16. principal, after clauses 4, 5, 7, 8, 10, 11, 13 and 14
  pay class_e_target    to asset.class_e   = entity.asset.class_a1.original_balance
                                             + entity.asset.class_a2a.original_balance
                                             + entity.asset.class_a2b.original_balance
                                             + entity.asset.class_a3.original_balance
                                             + entity.asset.class_b.original_balance
                                             + entity.asset.class_c.original_balance
                                             + entity.asset.class_d.original_balance
                                             + entity.asset.class_e.original_balance
                                             - paid.class_a_target
                                             - paid.class_a_final
                                             - paid.class_b_target
                                             - paid.class_b_final
                                             - paid.class_c_target
                                             - paid.class_c_final
                                             - paid.class_d_target
                                             - paid.class_d_final
                                             - asset.trust.pool_balance

  // 17. the remaining Class E balance at its final scheduled date
  pay class_e_final     to asset.class_e   = if(time.t >= 5, entity.asset.class_e.original_balance, 0.0)

  // 18. the Noteholders' Principal Distributable Amount
  pay principal_distributable to asset.class_a1 = inputs.principal_distributable

  // 19. the reserve account, up to its specified amount
  pay reserve_topup     to asset.reserve   = inputs.specified_reserve - asset.trust.reserve_balance

  // 20. principal to achieve the specified overcollateralization
  pay oc_build          to asset.class_a1  = asset.trust.pool_balance - inputs.oc_target

  // 21. trustee amounts in excess of the cap that held them at step 2
  pay trustee_excess    to party.trustee   = owed.trustee_fees - paid.trustee_fees

  // 22. everything that survives
  pay residual          to party.certificate = remaining
}
```

## waterfall_after_contract

```cfdl
version 0.1
model "waterfall-after-contract"
use pack "credit" version "0.1.0"
time calendar monthly from 2018-10 for 6

// DECLARATION ORDER MUST NOT CHANGE WHAT A MODEL MEANS.
//
// A block that scans forward for the end of its own body stops at the next
// declaration. When `waterfall` was missing from that set, a contract ran past
// its own closing brace and absorbed everything after it — the `assume` and the
// waterfall both — and the model compiled clean, ran clean, and paid nobody.
//
// The give-away was that moving the party ABOVE the contract fixed it: the
// contract then stopped at `entity`, which was on the list. Same declarations,
// same numbers, different order, different answer.
//
// This fixture is that order: a contract, then an assume, then a waterfall,
// with nothing after them to stop an over-eager scan.

entity asset trust : Credit.Asset.LoanPool { collateral_type = "auto" }
entity asset pool  : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity party holders : Credit.Party.Investor { name = "Noteholders" }

contract credit.pool_level_pay.pool on entity asset.pool {
  term 2018-10..2019-03
  terms {
    principal = 1200000.0
    interest_rate = 0.06
    term_months = 6
    cpr = 0
    cdr = 0
  }
}

assume note_balance = 500000.0

waterfall notes.principal on entity asset.trust {
  schedule every month from 2018-10 to 2019-03
  // NARROWER THAN `available`, deliberately: this waterfall allocates the
  // principal collections the exhibit tabulates, not the deal's whole cash. `docs/03` §3.2
  // keeps the `from` expression free for exactly this.
  from series_sum("credit.pool.sched_principal.pool", time.t, time.t)

  // THE CAP IS PER-PERIOD, NOT CUMULATIVE, AND THAT IS THE HONEST SPELLING.
  //
  // This step used to subtract what it had already paid —
  // `inputs.note_balance - series_sum("notes.principal.note_principal", ...)` —
  // to cap the note at its balance over the whole deal. A step publishes when
  // its waterfall finishes, so that read saw nothing, subtracted nothing, and
  // the cap never bound: a $500,000 note paid out $1,200,000 across six
  // periods and the golden agreed with it. `E1342_WATERFALL_SERIES_NOT_VISIBLE`
  // now refuses the spelling.
  //
  // A running total is a BALANCE the distribution draws down, which needs a
  // distribution that can post to a field — `docs/13` §7.37. Until then the
  // cumulative cap is not expressible, and stating a per-period one is better
  // than stating a cumulative one that does nothing.
  pay note_principal to party.holders = min(remaining, inputs.note_balance)
}
```

## waterfall_available

```cfdl
version 0.1
model "waterfall-available"
time calendar monthly from 2017-01 for 3

// A WATERFALL'S POT, BY NAME. `available` is the netted stream cash of the
// entity the waterfall hangs on, children rolled up by `part of` — the
// quantity the documentation calls "this period's cash". The engine supplies
// it the way it supplies `remaining`; no model declares a field for it.
//
// The trust here produces 300 + 150 - 50 = 400 a period, across two children,
// and the waterfall allocates exactly that: 250 senior, 150 residual.

entity asset trust : Asset.Financial { }
entity asset pool_a : Asset.Financial { part of asset.trust }
entity asset pool_b : Asset.Financial { part of asset.trust }
entity party investor : Party { name = "Investor" }

stream a.collections on entity asset.pool_a inflow currency USD {
  schedule every month from 2017-01 to 2017-03
  amount = 300.0
}
stream b.collections on entity asset.pool_b inflow currency USD {
  schedule every month from 2017-01 to 2017-03
  amount = 150.0
}
stream b.fee on entity asset.pool_b outflow currency USD {
  schedule every month from 2017-01 to 2017-03
  amount = 50.0
}

waterfall dist on entity asset.trust {
  schedule every month from 2017-01 to 2017-03
  from available
  pay senior   to party.investor = 250.0
  pay residual to party.investor = remaining
}
```

## waterfall_cre_jv_promote

```cfdl
version 0.1
model "waterfall-cre-jv-promote"
time calendar annual from 2000-01 for 4

// A CRE DEVELOPMENT JOINT VENTURE — the distribution priority of the One
// Lincoln Street venture, as its case states it.
//
// Three partners, and the shape is not the fund waterfall this repository
// already carries. TWO claimants share the preference PARI PASSU, and a third
// is subordinated to both their preferred return AND the return of their
// capital before it sees a dollar.
//
//   * MSGW and STRS each earn an annually compounded 11% cumulative preferred
//     on invested equity, and each is entitled to full repayment of that
//     capital — including earned but unpaid preference — from sale proceeds
//     before any payment is made to CPA;
//   * those two preferences rank PARI PASSU with each other;
//   * what survives splits 34% / 51% / 15% to MSGW / STRS / CPA.
//
// CPA is the development consortium holding the site designation. It
// contributes no capital and takes a promote — which is why it ranks last and
// why its share is not proportional to anything.
//
// WHAT PARI PASSU COSTS IN AN ORDERED WATERFALL. Steps pay in sequence, so
// writing the two preferences as two ordinary steps would pay MSGW in full
// before STRS saw anything — sequential, not pari passu, and identical to the
// correct answer whenever the pot is deep enough to hide the difference. The
// first step therefore caps itself at its PRO-RATA share of what is there:
//
//     min(E_m, remaining * E_m / (E_m + E_s))
//
// When the pot covers both, that minimum is E_m and both are paid in full.
// When it does not, it is exactly the pro-rata share, and the second step's
// `min(E_s, remaining)` collects the rest of the same proportion. One
// expression covers both regimes with no branch.
//
// EQUITY IS THE VENTURE'S STATED MINIMUM, split as the case states: the greater
// of $175m or 50% of total development cost, contributed 10% by MSGW and 90%
// by STRS. Preference accrues through the four construction years, when the
// development distributes nothing because it earns nothing.
//
// THE SALE PRICE IS AN ASSUMPTION, NOT A PUBLISHED FIGURE. The case is a
// student assignment: it specifies the structure completely and asks the reader
// to compute the returns, so there is no published answer to check against.
// This is an EXPRESSIVENESS fixture — it shows the structure can be said — and
// the two scenarios below are chosen to exercise both regimes rather than to
// forecast the building.

entity asset jv : Asset.Financial {
  // Capital plus the compounding preference, which is the whole entitlement
  // ranking ahead of CPA. Annually compounded, per the case's own wording.
  msgw_preference init inputs.msgw_capital next prev * (1.0 + inputs.pref_rate)
  strs_preference init inputs.strs_capital next prev * (1.0 + inputs.pref_rate)
}

entity party msgw : Party { name = "MSGW III" }
entity party strs : Party { name = "Midwest State Teachers Retirement System" }
entity party cpa  : Party { name = "Columbia Plaza Associates" }

assume msgw_capital = 17500000.0          // 10% of the $175m venture equity
assume strs_capital = 157500000.0         // 90%
assume pref_rate    = 0.11
assume net_sale_proceeds = 300000000.0    // overridden by the shortfall scenario

// The equity actually going in, so the model carries cash rather than only a
// distribution.
stream jv.equity_msgw on entity asset.jv outflow currency USD {
  schedule every year from 2000-01 to 2000-01
  amount = inputs.msgw_capital
}

stream jv.equity_strs on entity asset.jv outflow currency USD {
  schedule every year from 2000-01 to 2000-01
  amount = inputs.strs_capital
}

// The sale. The venture receives the proceeds in the exit year; the equity
// went out in year zero, so `available` at the distribution is the proceeds.
stream jv.sale_proceeds on entity asset.jv inflow currency USD {
  schedule on 2003-01
  amount = inputs.net_sale_proceeds
}

waterfall jv.distribution on entity asset.jv {
  schedule on 2003-01
  from available

  // 1-2. Capital and accrued preference, PARI PASSU between the two funders.
  pay msgw_preference to party.msgw = min(asset.jv.msgw_preference,
                                          remaining * asset.jv.msgw_preference
                                          / (asset.jv.msgw_preference + asset.jv.strs_preference))
  pay strs_preference to party.strs = min(asset.jv.strs_preference, remaining)

  // 3-5. The residual, 34 / 51 / 15. Each share is struck on the SAME base:
  // `remaining` falls as the steps pay, so the second share reconstructs the
  // base from what the first one actually paid rather than re-reading a pot
  // that has already shrunk.
  pay msgw_residual to party.msgw = remaining * 0.34
  pay strs_residual to party.strs = paid.msgw_residual / 0.34 * 0.51
  pay cpa_residual  to party.cpa  = remaining
}
```

## waterfall_fund_carry

```cfdl
version 0.1
model "waterfall-fund-carry"
time calendar annual from 2020-01 for 6

// A PRIVATE FUND CARRY WATERFALL — the ordered kind.
//
// $10m called, $30m returned five years later, split on a whole-of-fund
// waterfall: capital back, then a compounding 8% preferred return, then a full
// GP catch-up, then 80/20. Each tier is paid only out of what the tier above
// left, which is what makes this an ordered allocation rather than a set of
// shares.
//
// WHAT THIS EXERCISES THAT THE ABS CASE DOES NOT:
//
//   * CUMULATIVE targets — a preferred return is measured since inception, not
//     for the period, so the tier is a balance rather than a periodic accrual;
//   * a ONCE-AT-EXIT schedule, `schedule on`, rather than a distribution date;
//   * a catch-up, which is the tier most often said to need a solver.
//
// IT NEEDS NO SOLVER. The catch-up pays the GP 20% of everything distributed
// in the preferred and catch-up tiers combined:
//
//     X / (pref + X) = 0.20   ->   X = pref / 4
//
// one division, and the same is true of every other tier here. `docs/17` §12
// works through why an IRR HURDLE is closed-form too: the hurdle rate is an
// input, so nothing solves for a rate — only for the payment that reaches it.

entity asset fund : Asset.Financial {
  // Capital plus the compounding preferred — the fund's own hurdle.
  lp_preference init inputs.called_capital * pow(1.0 + inputs.pref_rate, inputs.hold_years)
                next prev
}
entity party lp   : Party { name = "Limited Partners" }
entity party gp   : Party { name = "General Partner" }

assume called_capital = 10000000.0
assume pref_rate      = 0.08
assume hold_years     = 5.0
assume gp_carry       = 0.20
assume proceeds       = 30000000.0

// WHAT THE CATCH-UP IS COMPUTED ON is the only thing separating three
// published structures, so it is a run-config knob rather than three models:
//
//   0                        no catch-up tier at all — straight 80/20
//   the preferred only       GP catches up to 20% of the PREF
//   the whole first tier     GP catches up to 20% of ALL distributions
//
// The deterministic run is the third; two scenarios are the others.

// Capital plus the compounding preferred, measured from inception — the tier
// one hurdle in money rather than in rate.

// The exit: the fund receives the proceeds, and the waterfall allocates the
// fund's available cash rather than re-reading the assumption.
stream fund.sale_proceeds on entity asset.fund inflow currency USD {
  schedule on 2025-01
  amount = inputs.proceeds
}

waterfall fund.distribution on entity asset.fund {
  schedule on 2025-01
  from available

  // 1. Return of capital and the 8% cumulative preferred, to the LPs.
  pay lp_preference to party.lp = asset.fund.lp_preference

  // 2. Full GP catch-up: 20% of everything paid in tiers 1's preferred
  //    component and this one together. Closed form, X = pref / 4.
  pay gp_catchup    to party.gp = cfg.catchup_base
                                  / (1.0 - inputs.gp_carry) * inputs.gp_carry

  // 3. The 80/20 split of what survives.
  pay gp_promote    to party.gp = remaining * inputs.gp_carry
  pay lp_residual   to party.lp = remaining
}
```

## waterfall_irr_hurdles

```cfdl
version 0.1
model "waterfall-irr-hurdles"
time calendar annual from 2020-01 for 6

// AN IRR-HURDLE WATERFALL, and a claim about it tested rather than repeated.
//
// Three participants — management, a sponsor promote and the limited partners.
// Management and the sponsor vest a percentage of equity that STEPS UP as the
// LP's IRR crosses each of eight hurdles. The source catalog describes this
// structure as "requiring a circular solve: the split determines cash flows
// which determine the IRR which determines the split".
//
// IT DOES NOT, FOR TWO REASONS, AND BOTH ARE VISIBLE IN THE ARITHMETIC.
//
// FIRST, the deal's IRR does not depend on the split. Capital goes in once and
// proceeds come back once, so the IRR is (P/C)^(1/n) - 1 — a closed form over
// the deal's own totals. Who receives the proceeds cannot change what the deal
// returned. The circularity people expect is not there to begin with.
//
// SECOND, each hurdle's vesting threshold is precomputable. A tier states an LP
// IRR, so the LP needs C*(1+h)^n; the LP takes only (1 - mgmt - sponsor) of the
// pot, so the pot must reach that grossed up; and the rate that pot implies is
// again a closed form. Eight thresholds, computed before any cash moves.
//
// What remains is choosing the tier, and that is an ORDERED DISCRETE test —
// the largest tier whose implied rate the deal beats. The same shape as the
// option ladder in benchmarks/opco/lbo_option_pool_exit, and it enumerates
// rather than iterating.
//
// So the waterfall itself is three plain steps.

entity asset deal : Asset.Financial {
  // The deal's own realized return — a fact about the deal.
  deal_irr init pow(inputs.proceeds / inputs.called_capital, 1.0 / inputs.hold_years) - 1.0
           next prev
}
entity party mgmt    : Party { name = "Management Team" }
entity party sponsor : Party { name = "Sponsor Promote" }
entity party lp      : Party { name = "Limited Partners" }

assume called_capital = 10000000.0
assume proceeds       = 30000000.0
assume hold_years     = 5.0

// The deal's own return. Two points, so this is arithmetic, not a search.

// THE VESTED PERCENTAGES, chosen in the waterfall rather than in a state.
//
// A state's `init` may not read another state — the rule `docs/14` sets, and
// the reason a first draft of this file returned zero for every share without
// complaining. A waterfall step reads period-close state, so the tier test
// belongs here, next to the payment it decides.
//
// Written descending, so "the largest tier the deal reached" falls out of a
// plain if-chain: eight ordered comparisons, no iteration.

stream deal.sale_proceeds on entity asset.deal inflow currency USD {
  schedule on 2025-01
  amount = inputs.proceeds
}

waterfall fund.distribution on entity asset.deal {
  schedule on 2025-01
  from available

  pay mgmt_proceeds    to party.mgmt    = inputs.proceeds *
     if(asset.deal.deal_irr >= 0.28148859901472223,
        0.08,
        if(asset.deal.deal_irr >= 0.2604763708480864,
           0.07,
           if(asset.deal.deal_irr >= 0.24011821572791225,
              0.06,
              if(asset.deal.deal_irr >= 0.22339827653178967,
                 0.05,
                 if(asset.deal.deal_irr >= 0.20696650711711273,
                    0.04,
                    if(asset.deal.deal_irr >= 0.15163318845914975,
                       0.03,
                       if(asset.deal.deal_irr >= 0.10299934260814592,
                          0.02,
                          0.0)))))))

  pay sponsor_proceeds to party.sponsor = inputs.proceeds *
     if(asset.deal.deal_irr >= 0.28148859901472223,
        0.2,
        if(asset.deal.deal_irr >= 0.2604763708480864,
           0.18,
           if(asset.deal.deal_irr >= 0.24011821572791225,
              0.16,
              if(asset.deal.deal_irr >= 0.22339827653178967,
                 0.15,
                 if(asset.deal.deal_irr >= 0.20696650711711273,
                    0.14,
                    if(asset.deal.deal_irr >= 0.15163318845914975,
                       0.1,
                       if(asset.deal.deal_irr >= 0.10299934260814592,
                          0.08,
                          0.0)))))))

  pay lp_proceeds      to party.lp      = remaining
}
```

## waterfall_nested_split

```cfdl
version 0.1
model "waterfall-nested-split"
time calendar annual from 2020-01 for 6

// A NESTED SPLIT — one waterfall's output is the next one's pot.
//
// A GP stakes structure has three of them in a row. The fund pays its carry to
// the general partner; the management company splits that carry with the deal
// team; and what the firm keeps is split again between its founders and the
// passive minority investor who bought a strip of it.
//
// Each of those is an ordered priority of payments over a pot that only exists
// once the one above has run. That is COMPOSITION, and it is a requirement
// rather than a convenience: without it the second waterfall's pot has to be
// restated as an assumption, which is the number the first waterfall computes.
//
// The rule is DECLARATION ORDER. A waterfall may read the steps of any
// waterfall declared before it, exactly as a step may read the steps above it.
// It is an order, not a graph, so there is nothing to solve and nothing to
// cycle.
//
// TIER ONE IS THE VERIFIED CASE. The fund waterfall is `waterfall_fund_carry`
// unchanged — $10m called, $30m back after five years, an 8% compounding
// preferred, a full catch-up, then 80/20 — and its GP carry comes to exactly
// $4,000,000, which is 20% of the $20m profit. A full catch-up is defined by
// producing that number, so the pot the firm-level waterfalls draw on is
// checkable by hand before any of them run.

entity asset fund     : Asset.Financial {
  lp_preference init inputs.called_capital * pow(1.0 + inputs.pref_rate, inputs.hold_years)
                next prev
}
entity asset mgmt_co  : Asset.Financial
entity party lp       : Party { name = "Limited Partners" }
entity party gp       : Party { name = "General Partner" }
entity party team     : Party { name = "Deal team carry pool" }
entity party founders : Party { name = "Founding partners" }
entity party stakes   : Party { name = "GP stakes investor" }

assume called_capital = 10000000.0
assume pref_rate      = 0.08
assume hold_years     = 5.0
assume gp_carry       = 0.20
assume proceeds       = 30000000.0

assume team_pool_pct  = 0.40
assume stakes_pct     = 0.20


// 1. THE FUND. Capital and preferred to the LPs, catch-up and promote to the
//    general partner, residual to the LPs.
stream fund.sale_proceeds on entity asset.fund inflow currency USD {
  schedule on 2025-01
  amount = inputs.proceeds
}

waterfall fund.distribution on entity asset.fund {
  schedule on 2025-01
  from available

  pay lp_preference to party.lp = asset.fund.lp_preference
  pay gp_catchup    to party.gp = 4693280.768
                                   / (1.0 - inputs.gp_carry) * inputs.gp_carry
  pay gp_promote    to party.gp = remaining * inputs.gp_carry
  pay lp_residual   to party.lp = remaining
}

// 2. THE MANAGEMENT COMPANY. The pot is the carry the fund just paid — both
//    tiers of it — and the deal team's pool comes off the top.
waterfall firm.carry_allocation on entity asset.mgmt_co {
  schedule on 2025-01
  // `0, time.t` rather than `0, 5`: the pot is every dollar of carry paid up
  // to this distribution, and the waterfall runs at one date. The constant 5
  // was that date's period index, so the two are the same number here — but
  // written as a constant it reads as a window that could reach past the
  // period being distributed, which a pot never does.
  from series_sum("fund.distribution.gp_catchup", 0, time.t)
        + series_sum("fund.distribution.gp_promote", 0, time.t)

  pay team_pool  to party.team = remaining * inputs.team_pool_pct
  pay firm_share to asset.mgmt_co = remaining
}

// 3. THE FIRM'S OWN CARRY, split a second time. The stakes investor's strip is
//    on what the firm keeps, not on the fund's carry, so this pot is the step
//    above rather than the one two levels up — which is the whole reason the
//    structure needs three waterfalls instead of three lines in one.
waterfall firm.owner_split on entity asset.mgmt_co {
  schedule on 2025-01
  from series_sum("firm.carry_allocation.firm_share", 0, time.t)

  pay stakes_strip   to party.stakes   = remaining * inputs.stakes_pct
  pay founder_share  to party.founders = remaining
}
```

## waterfall_partial_catchup

```cfdl
version 0.1
model "waterfall-partial-catchup"
time calendar annual from 2020-01 for 6

// A PARTIAL CATCH-UP — the tier that pays the GP some of each dollar rather
// than all of it.
//
// Under a full catch-up the GP takes every dollar above the preferred return
// until it holds its 20% of profits. Under a 50/50 catch-up it takes half of
// each dollar, so the tier is twice as long and the LPs keep receiving cash
// throughout it. 80/20 splits everything below.
//
// THE RATE IS AN INPUT, SO THE TIER IS STILL ARITHMETIC. Let P be the
// preferred return, k the carry rate, c the catch-up rate, and g what the GP
// is owed in the tier. The tier total is g/c, and the tier ends when the GP
// holds k of everything distributed above capital:
//
//     g = k * (P + g/c)     ->     g = k*P*c / (c - k)
//
// One expression, and c = 1 reduces it to the full catch-up's P/4 — which is
// how this model checks itself. Its `full_catchup` scenario returns the same
// $4,000,000 / $26,000,000 split as `waterfall_fund_carry`, to the cent.
//
// A catch-up rate at or below the carry rate never catches up: c = k makes the
// denominator zero and the tier infinite. That is a mis-specified deal, not a
// case the language has to represent.
//
// WHERE THE RATE ACTUALLY CHANGES THE ANSWER is when the money runs out inside
// the tier, which is the whole reason the structure is negotiated. The
// `early_exit` scenarios stop there, on $15.5m of proceeds. A full catch-up
// hands the GP the whole $806,719 left above the preferred; a 50/50 catch-up
// splits it, and the GP takes $403,360. Same deal, same money, one negotiated
// number between them.

entity asset fund : Asset.Financial {
  lp_preference init inputs.called_capital * pow(1.0 + inputs.pref_rate, inputs.hold_years)
                next prev
}
entity party lp   : Party { name = "Limited Partners" }
entity party gp   : Party { name = "General Partner" }

assume called_capital = 10000000.0
assume pref_rate      = 0.08
assume hold_years     = 5.0
assume gp_carry       = 0.20
assume proceeds       = 30000000.0

// Capital plus the compounding preferred, measured from inception.

stream fund.sale_proceeds on entity asset.fund inflow currency USD {
  schedule on 2025-01
  amount = cfg.proceeds
}

waterfall fund.distribution on entity asset.fund {
  schedule on 2025-01
  from available

  // 1. Return of capital and the cumulative preferred, to the LPs.
  pay lp_preference to party.lp = asset.fund.lp_preference

  // 2. The catch-up, at whatever rate the deal negotiated. The GP takes its
  //    share OF THE TIER — `remaining * c` — capped at what the tier owes it,
  //    so a pot that runs out mid-tier is split at the negotiated rate rather
  //    than handed to the GP whole. That cap is the entire difference between
  //    a full and a partial catch-up.
  pay gp_catchup to party.gp =
        min((asset.fund.lp_preference - inputs.called_capital)
              * inputs.gp_carry * cfg.catchup_rate
              / (cfg.catchup_rate - inputs.gp_carry),
            remaining * cfg.catchup_rate)

  // 3. The LPs' side of the same tier, sized off what the GP was ACTUALLY
  //    paid. Reading `paid.` rather than recomputing keeps the two sides of
  //    one tier consistent when the pot clamps the step above.
  pay lp_catchup to party.lp =
        paid.gp_catchup * (1.0 - cfg.catchup_rate) / cfg.catchup_rate

  // 4. The 80/20 split of what survives.
  pay gp_promote  to party.gp = remaining * inputs.gp_carry
  pay lp_residual to party.lp = remaining
}
```

## waterfall_smoke

```cfdl
version 0.1
model "waterfall-smoke"
use pack "credit" version "0.1.0"
time calendar monthly from 2017-02 for 4

entity asset trust : Credit.Asset.LoanPool {
  original_balance = 1000
  pool_balance    init 900.0
                  next prev
}
entity asset class_a : Credit.Asset.Tranche { seniority = 1  original_balance = 950 }
entity party servicer : Credit.Party.Servicer { name = "Servicer" }
entity party trustee : Credit.Party.Issuer { name = "Trustee" }
entity party certificate : Credit.Party.Investor { name = "Certificateholder" }


// The pot is the cash the trust receives. `available` binds it — the trust's
// netted stream cash for the period — so the waterfall consumes a result
// rather than a hand-maintained field.
// Undifferentiated collections, categorised as principal because that is what
// the waterfall below retires with them.
stream trust.collections on entity asset.trust inflow currency USD {
  schedule every month from 2017-02 to 2017-05
  category operating.collection.principal
  amount = 1000.0
}

// A priority of payments, and the seven rules a real 22-step consumer ABS
// waterfall needs, each written as an ORDINARY EXPRESSION rather than its own
// syntax:
//
//   a stated amount            = 12.5
//   a capped amount            = min(4.0, 3.0)
//   pay down to a target       = <balance> - <target>
//   an earlier step's overflow = owed.x - paid.x
//   everything that survives   = remaining
//
// Steps take min(max(0, owed), remaining), so the pot cannot go negative
// however a step is written, and the six lines below allocate exactly 1000.
waterfall abs.distribution on entity asset.trust {
  schedule every month from 2017-02 to 2017-05
  from available

  pay servicing        to party.servicer    = 12.5
  pay trustee_fees     to party.trustee     = min(4.0, 3.0)
  pay class_a_interest to asset.class_a      = 6.25
  pay class_a_target   to asset.class_a      = entity.asset.class_a.original_balance - asset.trust.pool_balance
  pay trustee_excess   to party.trustee     = owed.trustee_fees - paid.trustee_fees
  pay residual         to party.certificate = remaining
}
```
