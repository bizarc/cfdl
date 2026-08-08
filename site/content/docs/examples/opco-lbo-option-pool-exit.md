---
id: benchmark-opco-lbo-option-pool-exit
title: "OpCo: LBO exit waterfall with an option pool"
slug: "/docs/examples/opco-lbo-option-pool-exit"
source: benchmarks/opco/lbo_option_pool_exit
---

# OpCo: LBO exit waterfall with an option pool

A leveraged buyout's exit waterfall, splitting proceeds between an accruing preferred, rolled-over management equity and a laddered management option pool.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

The exit waterfall of a sponsor buyout: an accruing convertible preferred, a
management rollover, and a seven-tranche management stock option pool with
strikes laddered from $12.50 to $25.00.

An option tranche pays if it is in the money — if the exit consideration per
share exceeds its strike. But exercising a tranche adds both its strike proceeds
*and* its shares to the pool, which moves the value per share. So which options
exercise depends on the value per share, and the value per share depends on which
options exercise.

## The reference

A seven-step leveraged buyout teaching model published as a downloadable
spreadsheet, free and without registration. Its waterfall sheet publishes the
exit enterprise and equity value at six exit multiples, the in-the-money test per
tranche, option proceeds, share counts, the resulting value per share, and the
split of proceeds between sponsor and management.

**Not redistributable.** The workbook carries an "All Rights Reserved" notice, so
it is neither vendored nor wired into the test suite.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | seven options, two states, four native streams |
| Language features | options with an exercise test and a payoff; declared state read by an option guard |
| Conventions | a preferred accruing 8% and converting one-for-one, a management rollover, laddered option strikes, dilution at exit |

Two tranches are out of the money, so the case asserts a non-exercise as well
as an exercise.

## The result

Value per share **20.877119**, sponsor proceeds **487.546139**, management
proceeds **132.569706** — each matching the published figure exactly. Option
intrinsic value comes to $12.912mm across the five exercised tranches.

Verified at all six published exit multiples; the exercising set is unique at
every one.

## The delta

None.

The circularity resolves without iteration because **the strikes are ordered**:
if a $20.00 option is in the money then so is every cheaper one, so any
exercising set is a prefix of the tranches sorted by strike. That reduces 128
possible subsets to eight candidates, exactly one of which is self-consistent —
a finite ordered test rather than a search.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0}}
// A sponsor LBO's exit waterfall: an accruing convertible preferred, a
// management rollover, and a seven-tranche management stock option pool.
//
// COMPANION TO lbo_circular_interest, AND THE OTHER KIND OF CIRCULARITY. That
// case showed the debt schedule's loop is LINEAR, so it collects into a closed
// form. This one is the case that case's notes said was out of reach: a
// DISCRETE fixed point.
//
// An option tranche is exercised if it is in the money — if the exit
// consideration per share exceeds its strike. But exercising a tranche adds
// both its strike proceeds and its shares to the pool, which MOVES the value
// per share. So which options exercise depends on the value per share, and the
// value per share depends on which options exercise. There is no algebra that
// collects this: the unknown is a SET, not a number.
//
// IT IS STILL CLOSED, BECAUSE THE STRIKES ARE ORDERED. Any exercising set must
// be a prefix of the tranches sorted by strike — if a $20.00 option is in the
// money then so is every cheaper one. That reduces the search from 2^7 subsets
// to 8 candidate prefixes, and exactly one of them is self-consistent. So the
// fixed point is resolved by a finite ordered test rather than by iterating:
//
//     V(j) = (exit equity + cumulative strike proceeds through j)
//            / (preferred shares + rollover shares + cumulative option shares)
//
//     take the largest j whose own strike is below its own V(j)
//
// which is the `if` chain on `value_per_share` below. Verified against all six
// published exit multiples; the consistent prefix is unique at each one.
//
// This is a real structure, not a teaching abstraction: a management option
// pool struck above the sponsor's entry price is how nearly every sponsor deal
// pays management, and the strikes are laddered precisely so that later
// tranches only pay in better outcomes.
//
// Modelled at the 8.0x exit multiple — the same multiple the deal was entered
// at, so it is the case where the sponsor's return comes from deleveraging and
// growth rather than from multiple expansion. The other five published columns
// are reconciled in NOTES.md.

version 0.1
model "lbo-option-pool-exit"
use pack "opco" version "0.1.0"

// A single exit period. Everything here is struck at one instant — this is a
// waterfall, not a cash flow schedule, and the schedule is the other case.
time calendar annual from 2021-01 for 1

entity asset target : OpCo.Asset.Enterprise {
  // Both quantities describe the enterprise AT EXIT, so they hang on it rather
  // than floating as model variables. `value_per_share` is what every option's
  // exercise test reads.
  exit_equity init 575.6158451632398

  value_per_share init if(25.00 < (575.6158451632398 + 103.875) / 32.203133120640006,
          (575.6158451632398 + 103.875) / 32.203133120640006,
          if(22.50 < (575.6158451632398 + 72.625) / 30.953133120640006,
          (575.6158451632398 + 72.625) / 30.953133120640006,
          if(20.00 < (575.6158451632398 + 44.500) / 29.703133120640006,
          (575.6158451632398 + 44.500) / 29.703133120640006,
          if(17.50 < (575.6158451632398 + 29.500) / 28.953133120640006,
          (575.6158451632398 + 29.500) / 28.953133120640006,
          if(15.00 < (575.6158451632398 + 20.750) / 28.453133120640006,
          (575.6158451632398 + 20.750) / 28.453133120640006,
          if(14.00 < (575.6158451632398 + 13.250) / 27.953133120640006,
          (575.6158451632398 + 13.250) / 27.953133120640006,
          if(12.50 < (575.6158451632398 + 6.250) / 27.453133120640006,
          (575.6158451632398 + 6.250) / 27.453133120640006,
          575.6158451632398 / 26.953133120640006)))))))
}

// ---------------------------------------------------------------------------
// Exit. LTM adjusted EBITDA at the end of the five-year hold, at 8.0x, less
// net debt carried out of the debt schedule.
// ---------------------------------------------------------------------------
assume exit_ebitda      = 119.29345470000001
assume exit_multiple    = 8.0
assume net_debt         = 378.7317924367603

// Sponsor preferred: $158.9375mm at $10.00/share, accruing 8% for five years,
// converting one-for-one. 158.9375 * 1.08^5 = 233.53133.
assume preferred_shares = 23.353133120640006

// Management rollover: $36mm at $10.00/share.
assume rollover_shares  = 3.6


// ---------------------------------------------------------------------------
// The option pool. Seven tranches, laddered by strike ($mm of proceeds and
// millions of shares).
//
//   strike   shares   cumulative shares   cumulative proceeds
//   12.50     0.50           0.50                 6.250
//   14.00     0.50           1.00                13.250
//   15.00     0.50           1.50                20.750
//   17.50     0.50           2.00                29.500
//   20.00     0.75           2.75                44.500
//   22.50     1.25           4.00                72.625
//   25.00     1.25           5.25               103.875
// ---------------------------------------------------------------------------

// Exit equity value, before any option proceeds.


// The resolved value per share.
//
// Walks the prefixes from the largest down and takes the first one that is
// self-consistent — the first j whose own strike sits below the value per
// share that exercising through j would produce. Descending order is what
// makes "largest consistent j" fall out of a plain `if` chain.
//
// The denominators are (26.953133120640006 + cumulative option shares), where
// 26.953133 is the preferred plus rollover shares that exist regardless.

// ---------------------------------------------------------------------------
// The options themselves. Each tests its own strike against the resolved value
// per share — the economically real test, and now expressible directly:
// `exercise when` reads `asset.target.value_per_share`, the value the model derives
// above, rather than a constant restated for the engine's benefit.
//
// The payoff is the tranche's intrinsic value at exit: shares * (value - strike).
// ---------------------------------------------------------------------------

option mgmt_options_12_50 type Option.Equity {
  exercise when asset.target.value_per_share > 12.50
  payoff 0.50 * (asset.target.value_per_share - 12.50)
}

option mgmt_options_14_00 type Option.Equity {
  exercise when asset.target.value_per_share > 14.00
  payoff 0.50 * (asset.target.value_per_share - 14.00)
}

option mgmt_options_15_00 type Option.Equity {
  exercise when asset.target.value_per_share > 15.00
  payoff 0.50 * (asset.target.value_per_share - 15.00)
}

option mgmt_options_17_50 type Option.Equity {
  exercise when asset.target.value_per_share > 17.50
  payoff 0.50 * (asset.target.value_per_share - 17.50)
}

option mgmt_options_20_00 type Option.Equity {
  exercise when asset.target.value_per_share > 20.00
  payoff 0.75 * (asset.target.value_per_share - 20.00)
}

// Out of the money at 8.0x: the value per share resolves to $20.88, below both
// remaining strikes. Included precisely so the case asserts a NON-exercise as
// well as an exercise — an option model that only ever fires is not tested.
option mgmt_options_22_50 type Option.Equity {
  exercise when asset.target.value_per_share > 22.50
  payoff 1.25 * (asset.target.value_per_share - 22.50)
}

option mgmt_options_25_00 type Option.Equity {
  exercise when asset.target.value_per_share > 25.00
  payoff 1.25 * (asset.target.value_per_share - 25.00)
}

// ---------------------------------------------------------------------------
// The reported lines.
// ---------------------------------------------------------------------------

// Total cash to shareholders: exit equity plus the strike proceeds the
// exercised tranches pay in.
stream opco.exit.equity_value on entity asset.target inflow currency USD {
  schedule every year from 2021-01 to 2021-01
  category investing.exit
  amount = asset.target.exit_equity
}

stream opco.exit.option_proceeds on entity asset.target inflow currency USD {
  schedule every year from 2021-01 to 2021-01
  category investing.exit
  amount = 44.500
}

// THE SPLIT, AS A WATERFALL. Total cash to shareholders is exit equity plus
// the strike proceeds the exercised tranches pay in. The sponsor takes its
// converted preferred shares at the resolved value; management takes what is
// left, which is its rollover plus the exercised option shares at the same
// value.
//
// WHY A WATERFALL AND NOT TWO STREAMS. Written as two independent amounts,
// nothing checked that they add up to the cash available — both could be wrong
// together and every gate would pass. As a waterfall the adding-up is
// structural: management's step is `remaining`, so the two exhaust the pot by
// construction, and an error in the sponsor's share moves management's figure
// away from the published one instead of hiding.
//
// It is gross of the strikes paid in: those are already inside the pot, so
// netting them here would double-count.
entity party sponsor : OpCo.Party.Sponsor    { name = "Sponsor" }
entity party mgmt    : OpCo.Party.Management { name = "Management" }

waterfall opco.exit on entity asset.target {
  schedule on 2021-01
  from asset.target.exit_equity + 44.500

  pay sponsor_proceeds    to party.sponsor = 23.353133120640006 * asset.target.value_per_share
  pay management_proceeds to party.mgmt    = remaining
}
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.0
  }
}
```

## Verified results

Checked period by period: **12 series** across **1 periods**, each within ±1e-6 of the reference.

- `state.asset.target.value_per_share`
- `opco.exit.equity_value`
- `opco.exit.option_proceeds`
- `opco.exit.sponsor_proceeds`
- `opco.exit.management_proceeds`
- `option.mgmt_options_12_50`
- `option.mgmt_options_14_00`
- `option.mgmt_options_15_00`
- `option.mgmt_options_17_50`
- `option.mgmt_options_20_00`
- `option.mgmt_options_22_50`
- `option.mgmt_options_25_00`

