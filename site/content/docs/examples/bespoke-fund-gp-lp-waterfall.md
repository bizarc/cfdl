---
id: benchmark-bespoke-fund-gp-lp-waterfall
title: "A fund waterfall, with each partner's own return"
slug: "/docs/examples/bespoke-fund-gp-lp-waterfall"
description: "A closed-end fund distributing 39,973,982.80 over twenty-nine months against 31,000,000 committed, split by return of capital, an 8% preferred return and three IRR-hurdle tiers, with each partner's return from its own account."
source: benchmarks/bespoke/fund_gp_lp_waterfall
---

# A fund waterfall, with each partner's own return

A closed-end fund distributing 39,973,982.80 over twenty-nine months against 31,000,000 committed, split by return of capital, an 8% preferred return and three IRR-hurdle tiers, with each partner's return from its own account.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

A closed-end fund and the agreement that divides its cash. Thirty-one million
is committed at the fund's first month — ninety per cent by the limited
partner, ten by the general partner — and over the following twenty-nine months
the fund returns 39,973,982.80.

The agreement divides that cash in five steps. Capital comes back first, pro
rata. Then a preferred return of 8 per cent, accrued on the capital still
outstanding. Then three hurdle tiers at 8.5, 9 and 100 per cent, each drawing
on a stated share of what is left.

A hurdle is a measurement rather than a share. It compares what a partner has
earned so far against a rate, and the tier pays the difference. Two partners in
the same fund therefore earn different returns on the same cash, and the size
of that difference is what the agreement is for.

## The reference

**Not redistributable.** A private fund model held in the research corpus. The
workbook is not committed. The case carries the fund's monthly cash flow and
the partnership's stated economics as a frozen input set, and asserts the
figures the source publishes: a per-period amount for each partner in each
tier, and a summary giving each partner's total and its annual return.

No party, property, fund or manager name from the source is carried. The
partners here are the roles the tiers name.

The source leaves two conventions implicit, and this case fixes both:

**The preferred accrues on capital outstanding at the open of the period**,
before that period's repayment lands. Over the fund's life the choice of
measurement point moves the preferred by 199,454.

**Each hurdle pays a partner up to that partner's own shortfall**, capped by
that partner's share of the tier. The share is a ceiling on what a partner may
draw, not the proportion in which the tier is divided. Here the ceilings are
never reached, so each partner takes what it is short, and the amounts land in
the ratio of the partners' capital — 90/10 — while the stated shares are 50/50
and 45/55.

## What it exercises

| | |
|---|---|
| Pack | none — written from the bare language |
| Declared | 1 curve, 3 entities, 2 streams, 3 accounts, 1 waterfall, 10 tiers, 4 metrics, 10 carried balances |
| What the deal requires | carrying a balance across periods, cash accumulating into one place before it is divided, an ordered priority of payments, a tier measured against what a partner has already earned, a return measured per partner |
| Conventions | pro-rata return of capital, a preferred accrued on capital outstanding, hurdle tiers struck on each partner's own return |

Each partner's capital is recorded as it is committed and each distribution as
it is made, so the return published for a partner is measured over that
partner's own position.

## The result

The fund's own figures reproduce exactly:

| | model | reference |
|---|---|---|
| Lifetime net cash | 8,973,982.800209 | 8,973,982.800209 |
| Fund return | 19.8049% | 19.8049% |

And so do both partners' returns, each measured from that partner's own record:

| | contributed | distributed | multiple | return |
|---|---|---|---|---|
| Limited partner (90%) | 27,900,000.00 | 33,315,452.19 | 1.19x | 14.0641% |
| General partner (10%) | 3,100,000.00 | 6,658,530.61 | 2.15x | 57.1144% |

Every tier is asserted for each partner in each of the thirty months.

The general partner earns four times the limited partner's return on a tenth of
the capital. That difference is made in the third hurdle, which is set at a
return the fund does not reach and so carries everything above the tiers below
it.

## The delta

**Every asserted tier agrees to within $1.36**, on 37,073,982.80 allocated.

The residual is a rounding convention. The source rounds the cash-flow vectors
it runs its own return test against to whole dollars, and this model carries
them unrounded. The difference moves a hurdle boundary by a few cents in the
months where a tier is close to exhausted; the largest single divergence in the
case is $1.36, in one month, in one tier. The per-period tolerance carries it.

**The scope is the distribution.** The case asserts what the agreement does
with a monthly total, and takes that total as its input. The source builds the
total from twenty-five property models, each with a capital stack, two debt
tranches whose payments it derives, an interest-only period and a sale. The
property layer is a case of its own.

**The figures for the general partner are what the fund pays it.** The source
divides those proceeds again, between two classes of the general partner's own
members, on a second set of hurdles at 10, 15, 20 and 100 per cent. That
division is a case of its own.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.08}}
version 0.1
model "fund-gp-lp-waterfall" currency USD
time calendar monthly from 2017-08 for 30

// ===========================================================================
// A CLOSED-END FUND'S DISTRIBUTION WATERFALL.
//
// 31,000,000 is committed at the fund's first period, 90 per cent by the
// limited partner and 10 by the general partner. Over the following
// twenty-nine months the fund returns 39,973,982.80. This model allocates
// that cash: capital back first, then a preferred return, then three
// hurdle tiers, each paying a partner only until that partner's own return
// reaches the hurdle.
//
// Terms and the monthly cash flow are a frozen input set. Provenance is in
// reference/PROVENANCE.md.
// ===========================================================================

assume commitment  = 31000000.0
assume lp_share    = 0.90
assume gp_share    = 0.10
assume pref_m      = 0.00643403011000343
assume h1_m        = 0.006821493365962272
assume h2_m        = 0.007207323316136716
assume h3_m        = 0.05946309435929531
assume h1_pool_lp  = 0.50
assume h2_pool_lp  = 0.45
assume h3_pool_lp  = 0.40

// The fund's distributable cash, by month. Every period is declared.
curve fund_cf {
  2017-08: 0.0000000000
  2017-09: 329583.3333333333
  2017-10: 330132.6388888889
  2017-11: 330682.8599537037
  2017-12: 331233.9980536265
  2018-01: 331786.0547170492
  2018-02: 332339.0314749110
  2018-03: 191751.0901390536
  2018-04: 192070.6752892854
  2018-05: 192390.7930814342
  2018-06: 2952374.7015722967
  2018-07: 1563971.6206677407
  2018-08: 2953990.9988077767
  2018-09: 1548423.0194330842
  2018-10: 2944186.7637892212
  2018-11: 2941425.7620103741
  2018-12: 2938696.2231653449
  2019-01: 2935998.3123942427
  2019-02: 1513107.4245575145
  2019-03: 2937935.3238998223
  2019-04: 71488.0789977869
  2019-05: 71607.2257961165
  2019-06: 1515432.3413555855
  2019-07: 1514269.9182054941
  2019-08: 1513124.0457543989
  2019-09: 2971041.3125505750
  2019-10: 1496294.6398919225
  2019-11: 19208.6528222681
  2019-12: 19208.6528222681
  2020-01: 2990227.3067840915
}

entity asset fund : Asset.Financial {
  // Each partner's account one period back, so a month's distribution can be
  // differenced without reaching two periods behind.
  lp_lag init 0.0 next prev.lp_capital
  gp_lag init 0.0 next prev.gp_capital

  // Capital still outstanding at the OPEN of the period. The preferred accrues
  // on this balance, which is the convention the reference uses: measured at
  // the close it misses by 199,454 over the fund's life.
  equity_bal init 31000000.0
             next max(0.0, prev.asset.fund.equity_bal - min(curve_value("fund_cf", edate(time.date, -1)), prev.asset.fund.equity_bal))

  // The preferred return, accrued on capital outstanding and carried until paid.
  pref_bal init 0.0
           next max(0.0, prev.asset.fund.pref_bal + max(0.0, prev.asset.fund.equity_bal - min(curve_value("fund_cf", edate(time.date, -1)), prev.asset.fund.equity_bal)) * inputs.pref_m
                         - min(max(0.0, curve_value("fund_cf", edate(time.date, -1)) - min(curve_value("fund_cf", edate(time.date, -1)), prev.asset.fund.equity_bal)),
                               prev.asset.fund.pref_bal + max(0.0, prev.asset.fund.equity_bal - min(curve_value("fund_cf", edate(time.date, -1)), prev.asset.fund.equity_bal)) * inputs.pref_m))

  // LP's balance at the h1 rate: capital accreted at the hurdle, less
  // everything already distributed to that party.
  lp_h1 init 27900000.0
        next max(0.0, (prev.asset.fund.lp_h1 - (prev.lp_capital - prev.asset.fund.lp_lag + if(time.t == 1, 27900000.0, 0.0))) * (1.0 + inputs.h1_m))
  // LP's balance at the h2 rate: capital accreted at the hurdle, less
  // everything already distributed to that party.
  lp_h2 init 27900000.0
        next max(0.0, (prev.asset.fund.lp_h2 - (prev.lp_capital - prev.asset.fund.lp_lag + if(time.t == 1, 27900000.0, 0.0))) * (1.0 + inputs.h2_m))
  // LP's balance at the h3 rate: capital accreted at the hurdle, less
  // everything already distributed to that party.
  lp_h3 init 27900000.0
        next max(0.0, (prev.asset.fund.lp_h3 - (prev.lp_capital - prev.asset.fund.lp_lag + if(time.t == 1, 27900000.0, 0.0))) * (1.0 + inputs.h3_m))
  // GP's balance at the h1 rate: capital accreted at the hurdle, less
  // everything already distributed to that party.
  gp_h1 init 3100000.0
        next max(0.0, (prev.asset.fund.gp_h1 - (prev.gp_capital - prev.asset.fund.gp_lag + if(time.t == 1, 3100000.0, 0.0))) * (1.0 + inputs.h1_m))
  // GP's balance at the h2 rate: capital accreted at the hurdle, less
  // everything already distributed to that party.
  gp_h2 init 3100000.0
        next max(0.0, (prev.asset.fund.gp_h2 - (prev.gp_capital - prev.asset.fund.gp_lag + if(time.t == 1, 3100000.0, 0.0))) * (1.0 + inputs.h2_m))
  // GP's balance at the h3 rate: capital accreted at the hurdle, less
  // everything already distributed to that party.
  gp_h3 init 3100000.0
        next max(0.0, (prev.asset.fund.gp_h3 - (prev.gp_capital - prev.asset.fund.gp_lag + if(time.t == 1, 3100000.0, 0.0))) * (1.0 + inputs.h3_m))
}

entity party lp : Party { name = "Limited Partner" }
entity party gp : Party { name = "General Partner" }

// The fund draws its capital once and returns cash monthly. Stated as streams,
// so the fund's own return is measured on the same vector the reference uses.
stream fund.capital on entity asset.fund outflow currency USD {
  schedule on 2017-08
  category investing.capital.capex
  amount = inputs.commitment
}

stream fund.receipts on entity asset.fund inflow currency USD {
  schedule every month start from 2017-09 to 2020-01
  category investing.reversion
  amount = curve_value("fund_cf", time.date)
}

// Each period's receipts accumulate here and the waterfall allocates them.
account fund_cash {
  from series_sum("fund.receipts", time.t, time.t)
}

// What each partner put in, so what each got back can be measured against it.
account lp_capital {
  owner party.lp
  from 0.0 - if(time.t == 0, inputs.commitment * inputs.lp_share, 0.0)
}

account gp_capital {
  owner party.gp
  from 0.0 - if(time.t == 0, inputs.commitment * inputs.gp_share, 0.0)
}

waterfall fund.distribution on entity asset.fund {
  schedule every month start from 2017-09 to 2020-01
  from fund_cash

  // 1. Capital back, pro rata, until the fund's equity is repaid.
  pay roc_lp to party.lp = min(asset.fund.equity_bal * inputs.lp_share, remaining * inputs.lp_share)
  pay roc_gp to party.gp = min(asset.fund.equity_bal * inputs.gp_share, remaining)

  // 2. The preferred return, on capital outstanding, split as capital was.
  pay pref_lp to party.lp = min(asset.fund.pref_bal * inputs.lp_share, remaining * inputs.lp_share)
  pay pref_gp to party.gp = min(asset.fund.pref_bal * inputs.gp_share, remaining)

  // 3-5. Each hurdle pays a partner only up to its own shortfall, and only out
  //      of that partner's share of the tier.
  pay h1_lp to party.lp = min(max(0.0, asset.fund.lp_h1 - paid.roc_lp - paid.pref_lp), remaining * inputs.h1_pool_lp)
  pay h1_gp to party.gp = min(max(0.0, asset.fund.gp_h1 - paid.roc_gp - paid.pref_gp), remaining)

  pay h2_lp to party.lp = min(max(0.0, asset.fund.lp_h2 - paid.roc_lp - paid.pref_lp - paid.h1_lp), remaining * inputs.h2_pool_lp)
  pay h2_gp to party.gp = min(max(0.0, asset.fund.gp_h2 - paid.roc_gp - paid.pref_gp - paid.h1_gp), remaining)

  pay h3_lp to party.lp = min(max(0.0, asset.fund.lp_h3 - paid.roc_lp - paid.pref_lp - paid.h1_lp - paid.h2_lp), remaining * inputs.h3_pool_lp)
  pay h3_gp to party.gp = remaining
}

// What each partner earned, folded over that partner's own account.
metric lp_irr   = irr(party.lp)
metric lp_moic  = moic(party.lp)
metric gp_irr   = irr(party.gp)
metric gp_moic  = moic(party.gp)
```

## Run configuration

```json
{
  "deterministic": {
    "annual_discount_rate": 0.08
  }
}
```

## Verified results

Checked period by period: **10 series** across **30 periods** — **300 values** in all, each within ±2.0 of the reference.

- `fund.distribution.roc_lp`
- `fund.distribution.roc_gp`
- `fund.distribution.pref_lp`
- `fund.distribution.pref_gp`
- `fund.distribution.h1_lp`
- `fund.distribution.h1_gp`
- `fund.distribution.h2_lp`
- `fund.distribution.h2_gp`
- `fund.distribution.h3_lp`
- `fund.distribution.h3_gp`

Summary metrics for the base run:

| Metric | Value | Tolerance |
|---|---:|---:|
| `model.total` | 8,973,982.8 | ±0.01 |
| `model.irr` | 0.198049 | ±0.000001 |
| `metric.lp_irr` | 0.140641 | ±0.000001 |
| `metric.gp_irr` | 0.571144 | ±0.000001 |
| `metric.lp_moic` | 1.194102 | ±0.00001 |
| `metric.gp_moic` | 2.147913 | ±0.00001 |
| `entity.party.lp.total` | 33,315,452.19 | ±1 |
| `entity.party.gp.total` | 6,658,530.61 | ±1 |
