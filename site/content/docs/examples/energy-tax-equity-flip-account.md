---
id: benchmark-energy-tax-equity-flip-account
title: "Energy: a tax-equity flip, distributing from an account"
slug: "/docs/examples/energy-tax-equity-flip-account"
description: "The twin of tax_equity_flip, with the project's cash as streams settling into an account rather than a hand-carried field."
source: benchmarks/energy/tax_equity_flip_account
---

# Energy: a tax-equity flip, distributing from an account

The twin of tax_equity_flip, with the project's cash as streams settling into an account rather than a hand-carried field.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

The twin of `tax_equity_flip`: the same leveraged partnership flip, against the
same external anchors, with the project's cash rebuilt as streams settling into
an account instead of a hand-carried field.

The original says what it is waiting for, in a comment on its own waterfall:
"the project carries it as a field the deal itself tracks. Rehoming it as
streams would move the case's asserted figures, so it stays until the case is
rebuilt." This is that rebuild, carried as a twin so the claim can be checked
rather than asserted.

## The reference

The same one the original uses — the national laboratory's open-source
project-finance model in its leveraged-partnership-flip configuration, run once
outside this repo. See `../tax_equity_flip/NOTES.md` for the version, inputs and
command. The anchors in `expected.csv` are that model's outputs, unchanged.

## What it exercises

The distribution pot as an **account** rather than a field. The plant's revenue,
O&M and debt service are three streams under one name family; the account draws
them with a single glob; the distribution takes the whole balance each period,
so the account returns to zero and carries nothing forward.

The flip test moves with the pot. `return_position` previously read
`prev.asset.project.cash` — the field. It now reads
`series_sum("energy.project.*", time.t - 1, time.t - 1)`: the same quantity,
taken from settled cash strictly backward.

What that buys is visibility. In the original the project's cash exists only
inside an entity, where the distribution can see it and a reviewer cannot. Here
it is in the ledger, published per period, and the figures that decide the flip
are the figures the statement shows.

## The result

Both partners' columns reconcile against the same anchors as the original, at
the same one-cent tolerance, across all 25 operating periods.

Against the ORIGINAL's own output the agreement is tighter still: 50 of 50
cells within tolerance, largest absolute difference 0.0047 dollars on figures
of about four million — one part in 10^9.

## The delta

Where this case is looser than its twin, and why.

The original reconciles to 1.0e-6 dollars against the reference. This one
reconciles to 4.7e-3. That is a reassociation difference, not a modeling one:
the same quantities are summed in a different order — through the ledger and an
account balance, rather than inside one field expression — and floating-point
addition is not associative. Both are far inside the case's one-cent tolerance,
and neither is more correct than the other about the deal.

It is recorded because a twin exists to make a substitution checkable, and
"the numbers moved by 5 milli-dollars" is part of what the check found.

## A variant the reference does not publish

None. The original carries that section; this twin exists to check a
substitution, not to extend the case.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "tax-equity-flip-account"
use pack "energy" version "0.1.0"
time calendar annual from 2026-01 for 26

// A TAX-EQUITY PARTNERSHIP FLIP, where the flip date is DERIVED.
//
// A tax investor funds most of the equity and takes 98% of the cash and the
// tax attributes. When its after-tax return reaches a target, the structure
// flips: it drops to 5% and the sponsor takes the rest. The flip is not a date
// in a contract — it is a test, and when it lands depends on how the project
// performs.
//
// THE FLIP IS AN EVENT WRITING A NUMBER, and the date is an output. The
// investor's share of cash and tax is a term of the contract between the
// parties: stated at signing, changed when a condition is met. So it is a
// field of the stake, and the event sets it.
//
// It was first built as a lifecycle, `pre_flip` -> `post_flip`, with the
// percentages looked up from whichever state the stake was in. That is two
// facts — a state name and the number it implies — kept in step by nobody. A
// lifecycle earns its place when the phases differ in WHICH RULES APPLY, as a
// building under construction differs from one in operation. Here they differ
// by a number, and a number is a field.
//
// THE TEST NEEDS NO SOLVER. The criterion is an internal rate of return
// reaching 8%, and this language cannot compute an IRR mid-model. It does not
// need to: at a fixed hurdle the two statements are the same one.
//
//     IRR through period n >= 8%   <=>   NPV at 8% through period n >= 0
//
// A discounted running sum is a recurrence, which is a declared state, so the
// test is arithmetic evaluated once a period — a discrete test rather than a
// search, the same shape as an ordered waterfall's tiers.
//
// AND IT CANNOT BE CIRCULAR. The test at period t reads flows through t-1, and
// every one of those periods is by construction still pre-flip: the flip has
// not fired yet, or the test would not still be running. So the sharing
// percentages the test depends on are settled before it is evaluated.

entity asset project : Energy.Asset.GenerationFacility {
  technology         = "solar_pv"
  nameplate_capacity = 100000.0
  state in_service

  // The plant's cash is STREAMS now, not a field. What it throws off is
  // published per period in the ledger, where a reviewer can read it, rather
  // than computed inside the entity and visible only to the distribution.
}

// THE PARTNERSHIP, which is the thing that allocates the project's cash. Not
// the plant: when the flip happens nothing about the solar farm changes — same
// panels, same output — only who has a right to the money.
//
// There is no separate "stake" object between the partnership and its terms.
// The sharing percentage is a term of the partnership, so it is a field of the
// partnership.
entity asset partnership : Energy.Asset.ProjectInterest {
  interest_type = "tax_equity"

  // The investor's share of cash and of tax attributes, as the contract
  // states it at signing.
  investor_share init 0.98

  // How far the investor is toward its target, as the discounted value of
  // everything the partnership has returned it. The test that moves the share.
  return_position init 0.0 - inputs.investor_equity
                  next prev
                     + if(time.t >= 2.0 and prev < 0.0,
                     inputs.preflip_share
                     * ( series_sum("energy.project.*", time.t - 1.0, time.t - 1.0)
                     - inputs.tax_rate
                     * ( series_sum("energy.project.*", time.t - 1.0, time.t - 1.0)
                     + if(time.t - 1.0 <= inputs.debt_term,
                     (0.0 - pmt(inputs.debt_rate, inputs.debt_term, inputs.debt_amount))
                     + ipmt(inputs.debt_rate, time.t - 1.0, inputs.debt_term,
                     inputs.debt_amount),
                     0.0)
                     - macrs_rate(time.t - 2.0, 5)
                     * (inputs.installed_cost
                     - 0.5 * inputs.itc_rate * inputs.installed_cost) )
                     + if(time.t - 1.0 == 1.0, inputs.itc_rate * inputs.installed_cost, 0.0) )
                     / pow(1.0 + inputs.hurdle, time.t - 1.0),
                     0.0)
}

entity party sponsor      : Party { name = "Sponsor" }
entity party tax_investor : Party { name = "Tax investor" }

// ---------------------------------------------------------------------------
// The deal
// ---------------------------------------------------------------------------

assume energy_year_one = 250000000.0     // kWh in the first operating year
assume ppa_price       = 0.045           // $/kWh
assume ppa_escalation  = 0.02
assume degradation     = 0.005

assume capacity_kw     = 100000.0
assume om_per_kw       = 15.0
assume om_escalation   = 0.02

assume debt_amount     = 60000000.0
assume debt_rate       = 0.06
assume debt_term       = 18.0

// The equipment is $100m; the reference capitalizes $3.1m of financing into
// the installed cost, so the credit and depreciation are taken on the larger
// figure. Both bases follow from it: the credit on all of it, depreciation on
// it less half the credit, which is the rule that catches people out.
assume installed_cost  = 103100000.0
assume itc_rate        = 0.30
assume tax_rate        = 0.21

assume preflip_share   = 0.98
assume postflip_share  = 0.05

assume hurdle          = 0.08
assume investor_equity = 42238000.0      // 98% of $43.1m of equity

// ---------------------------------------------------------------------------
// The project, before anybody is paid
// ---------------------------------------------------------------------------



// THE TEST, as one recurrence.
//
// At period t this holds the investor's discounted after-tax position through
// period t-1: its share of cash, of the tax saved on the loss depreciation
// creates, and of the credit in the first operating year.
//
// It is one state rather than two because a state's `next` may read another
// state's PREVIOUS value and not its current one — so the flow of period t-1
// is exactly what is reachable here, and that is the flow the closed test
// wants. The lag is the deal's own convention: the year's books close, the
// return is tested, and the new sharing applies to the year that follows.
//
// Computed at the PRE-FLIP shares, and it stops accumulating the moment it
// turns non-negative — which is the period the flip fires. A test that has
// passed has no further question to answer, and stopping it keeps the series
// readable: its final value is the position that triggered the flip, not a
// running total at shares that stopped applying.
//
// Interest comes from `ipmt` rather than from a balance carried alongside.
// A balance state would hold the CLOSING figure, and interest is charged on
// the opening one — an off-by-one this states outright rather than works
// around.


// When the investor's return reaches its target, its share drops to 5% and the
// sponsor takes the rest — from the following period, which is the deal's own
// convention: the year's books close, the return is tested, the new split
// applies to the year that follows.
event flip when asset.partnership.return_position >= 0.0 {
  set entity asset.partnership.investor_share = 0.05
}

// ---------------------------------------------------------------------------
// What each partner receives
//
// The waterfall is owned by the INTEREST, so its steps read the lifecycle that
// governs the split. The investor takes its share and the sponsor takes the
// residual, which is what "the sponsor gets the rest" means.
// ---------------------------------------------------------------------------

// THE POT IS AN ACCOUNT the project's own streams settle into — the rebuild
// the original case said it was waiting for (`docs/25` category C, `docs/13`
// §7.76). The distribution takes the whole balance each period, so the account
// returns to zero and carries nothing forward; what changed is that the cash
// is now IN THE LEDGER, published per period, rather than computed inside an
// entity where only the waterfall could see it.
account project_cash {
  from series_sum("energy.project.*", time.t, time.t)
}

waterfall partnership.distribution on entity asset.partnership {
  schedule every year from 2027-01 to 2051-01
  from project_cash

  pay investor to party.tax_investor = remaining * asset.partnership.investor_share
  pay sponsor  to party.sponsor = remaining
}

// ---------------------------------------------------------------------------
// WHAT THE PLANT THROWS OFF, as streams. One name family, `energy.project.*`,
// so the account's `from` picks them up with a single glob and the flip test
// reads the same settled total the distribution does.
// ---------------------------------------------------------------------------

stream energy.project.ppa_revenue on entity asset.project inflow currency USD {
  schedule every year from 2027-01 to 2051-01
  category operating.revenue.energy
  amount = inputs.energy_year_one * inputs.ppa_price
         * pow(1.0 + inputs.ppa_escalation, time.t - 1.0)
         * pow(1.0 - inputs.degradation, time.t - 1.0)
}

stream energy.project.om on entity asset.project outflow currency USD {
  schedule every year from 2027-01 to 2051-01
  category operating.expense.om
  amount = inputs.capacity_kw * inputs.om_per_kw
         * pow(1.0 + inputs.om_escalation, time.t - 1.0)
}

stream energy.project.debt_service on entity asset.project outflow currency USD {
  schedule every year from 2027-01 to 2051-01
  category financing.debt.service
  amount = if(time.t <= inputs.debt_term,
               0.0 - pmt(inputs.debt_rate, inputs.debt_term, inputs.debt_amount),
               0.0)
}
```

## Run configuration

```json
{ "deterministic": { "annual_discount_rate": 0.08 } }
```

## Verified results

Checked period by period: **2 series** across **25 periods** — **50 values** in all, each within ±0.01 of the reference.

- `partnership.distribution.investor`
- `partnership.distribution.sponsor`

