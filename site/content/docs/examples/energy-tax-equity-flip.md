---
id: benchmark-energy-tax-equity-flip
title: "Energy: a tax-equity flip, with the date derived"
slug: "/docs/examples/energy-tax-equity-flip"
source: benchmarks/energy/tax_equity_flip
---

# Energy: a tax-equity flip, with the date derived

A tax-equity partnership whose flip date is derived from the investor's return rather than stated, reconciled against an external model.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A 100 MW-ac solar project financed through a tax-equity partnership. A tax
investor funds 98% of the equity and takes 98% of the cash, the depreciation
and the investment credit. When its after-tax return reaches 8%, the structure
**flips**: the investor drops to 5% and the sponsor takes the rest.

The flip is not a date. It is a test, and when it lands depends on how the
project performs — so the model derives it rather than stating it.

## The reference

A national laboratory's open-source project-finance model, run in its
leveraged-partnership-flip configuration. It publishes the flip year alongside
both partners' cash, so the derived date is checkable and not only the split.

**Not vendored.** The tool was run once outside the repository and only its
output numbers were carried across.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Declared | two typed assets, two parties, three states, one event, one waterfall |
| Language features | **a declared lifecycle**, **an event whose guard is a computed value**, the transition log, a waterfall reading its owner's state |
| Conventions | 98/2 pre-flip and 5/95 post-flip sharing, an investment credit at 30%, MACRS on a basis reduced by half the credit, level-pay debt |

The lifecycle sits on the partnership **interest** rather than the plant: the
panels do not change when the structure flips, the claim on their cash does.

**The test needs no solver.** The criterion is an internal rate of return
reaching 8%, which this language cannot compute mid-model. It does not need to
— at a fixed hurdle the two statements are one:

    IRR through period n >= 8%   <=>   NPV at 8% through period n >= 0

A discounted running sum is a recurrence, so the test is arithmetic evaluated
once a period. Nor can it be circular: the test at period *t* reads flows
through *t-1*, and every one of those periods is still pre-flip by
construction, so the sharing percentages it depends on are settled before it
is evaluated.

## The result

**The flip date is derived, and it agrees.** The transition fires at period 4
against the reference's stated flip in year 3 — the same instant under the
deal's own convention, where the year's books close, the return is tested, and
the new sharing applies to the year that follows.

Both partners' cash reproduces across all 25 operating periods, through the
flip and through the debt cliff at periods 18 and 19.

Asserted: two columns period by period, plus the transition itself.

## The delta

Worst disagreement across all 25 periods and both columns: **1.0e-6 dollars**,
which is the engine's own publication precision rather than a convention
difference.

Period 0 is not asserted. The reference books a sponsor development fee at
close that this case does not model, and it has no bearing on the flip.

## A variant the reference does not publish

The same deal on a **monthly** grid flips ten months earlier, in the second
month of year 3.

By the end of year 2 the investor is $445,000 short of its hurdle, and two
months of operating cash clear it — but an annual grid has no period between
month 24 and month 36 in which to notice, so the event cannot fire until the
next year end. The investor keeps 98% of the cash for ten months it was no
longer entitled to, worth about $3.5mm here.

The grid is therefore an economic assumption whenever an event decides who
gets paid, not a presentation choice. No external source publishes the monthly
answer, so it is carried as a fixture rather than as a benchmark.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "tax-equity-flip"
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
  state operating
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
                     * ( prev.project_cash
                     - inputs.tax_rate
                     * ( prev.project_cash
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

// The equipment is $100mm; the reference capitalises $3.1mm of financing into
// the installed cost, so the credit and depreciation are taken on the larger
// figure. Both bases follow from it: the credit on all of it, depreciation on
// it less half the credit, which is the rule that catches people out.
assume installed_cost  = 103100000.0
assume itc_rate        = 0.30
assume tax_rate        = 0.21

assume preflip_share   = 0.98
assume postflip_share  = 0.05

assume hurdle          = 0.08
assume investor_equity = 42238000.0      // 98% of $43.1mm of equity

// ---------------------------------------------------------------------------
// The project, before anybody is paid
// ---------------------------------------------------------------------------

state project_cash {
  init 0.0
  next inputs.energy_year_one * inputs.ppa_price
        * pow(1.0 + inputs.ppa_escalation, time.t - 1.0)
        * pow(1.0 - inputs.degradation, time.t - 1.0)
       - inputs.capacity_kw * inputs.om_per_kw
        * pow(1.0 + inputs.om_escalation, time.t - 1.0)
       - if(time.t <= inputs.debt_term,
            0.0 - pmt(inputs.debt_rate, inputs.debt_term, inputs.debt_amount),
            0.0)
}

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

waterfall partnership.distribution on entity asset.partnership {
  schedule every year from 2027-01 to 2051-01
  from state.project_cash

  pay investor to party.tax_investor = remaining * asset.partnership.investor_share
  pay sponsor  to party.sponsor = remaining
}
```

## Run configuration

```json
{ "deterministic": { "annual_discount_rate": 0.08 } }
```

## Verified results

Checked period by period: **2 series** across **25 periods**, each within ±0.01 of the reference.

- `partnership.distribution.investor`
- `partnership.distribution.sponsor`

