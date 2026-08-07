---
id: benchmark-credit-auto-abs-speed-150
title: "Credit: auto ABS at 1.5x prepayment speed"
slug: "/docs/examples/credit-auto-abs-speed-150"
source: benchmarks/credit/auto_abs_speed_150
---

# Credit: auto ABS at 1.5x prepayment speed

The same auto loan pool at 1.5 ABS, three times the prepayment speed, showing how the collection profile shortens.

Every number below is checked against an independent reference
implementation on every commit — period by period, and on each metric,
inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).

## The case

The same subprime auto receivables as the zero-speed case, prepaying at
**1.50% ABS**. Under the absolute prepayment speed convention, prepayments are a
constant share of the *original* pool each month, so the balance follows a
running product rather than a constant hazard — and the collection profile
shortens as the speed rises.

## The reference

An issuer's own prepayment-speed exhibit, filed publicly with a securities
regulator. It tabulates the Class A-2 percent-outstanding at every monthly
distribution date, for each of seven prepayment speeds.

**Not redistributable.** Public filings are freely readable and citable, but the
filer retains copyright, so figures are asserted against rather than reproduced.

## What it exercises

| | |
|---|---|
| Pack | `credit` |
| Contract types | `credit.pool_level_pay`, 43 instances |
| Language features | many instances of one contract type; a per-period pool factor carried as state |
| Conventions | absolute prepayment speed, level-pay amortisation, a promotional 0% rate |

This column was unreachable until the pool factor became per-period state. Every
pool factor in the pack computed `pow(k, p)`, which is correct only for a
constant hazard and wrong under a ramp.

## The result

Principal collections reproduce the exhibit's published percent-outstanding
column at every distribution date, worst **0.0036 percentage points** against a
source that rounds to 0.01.

The aggregate pool balance is asserted directly:
`domain.credit.principal` = **537,640,787.96**.

## The delta

The 0.0036-point residual is inside the exhibit's own rounding: it publishes to
two decimal places, so a difference smaller than 0.01 cannot be distinguished
from the printed figure.

## The model

```cfdl run={"deterministic":{"annual_discount_rate":0.03}}
// Auto-receivables collateral at 1.50% ABS, reconciled against the same
// issuer exhibit as benchmarks/credit/auto_abs_wal — one of its seven
// prepayment-speed columns rather than the zero-speed one.
//
// THE ABSOLUTE PREPAYMENT MODEL IS INDEXED FROM ORIGINATION. ABS states a
// constant fraction of the ORIGINAL number of receivables prepaying each
// month, so the implied SMM rises as the pool shrinks:
//
//     SMM(t) = ABS / (1 - ABS * (t - 1)),  t counted from ORIGINATION
//
// This pool is seasoned — weighted average ages run 11 to 42 months — so `t`
// is the loan's age, not the months since closing. `age_months` carries that.
// Measured against this exhibit, using months-since-closing instead is out by
// 20 percentage points of note balance by the fourth distribution at 1.50% ABS.
//
// The exhibit's own stated assumptions: no defaults, losses or repurchases
// (hence cpr = cdr = 0 alongside the ABS speed), payments on the last day of
// each month with 30-day months, and the clean-up call not exercised.

version 0.1
model "auto-abs-speed-150"
use pack "credit" version "0.1.0"
time calendar monthly from 2018-10 for 64

entity asset trust : Credit.Asset.LoanPool

contract credit.pool_level_pay.p01 on entity asset.trust {
  term 2018-10..2020-03
  terms {
    balance = 5616021.32
    rate = 0.00000
    term_months = 18
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 42
  }
}

contract credit.pool_level_pay.p02 on entity asset.trust {
  term 2018-10..2021-01
  terms {
    balance = 2616054.82
    rate = 0.00000
    term_months = 28
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 35
  }
}

contract credit.pool_level_pay.p03 on entity asset.trust {
  term 2018-10..2022-06
  terms {
    balance = 4635948.89
    rate = 0.00000
    term_months = 45
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 27
  }
}

contract credit.pool_level_pay.p04 on entity asset.trust {
  term 2018-10..2022-12
  terms {
    balance = 2205909.75
    rate = 0.00000
    term_months = 51
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 21
  }
}

contract credit.pool_level_pay.p06 on entity asset.trust {
  term 2018-10..2019-11
  terms {
    balance = 147440.15
    rate = 0.00915
    term_months = 14
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 40
  }
}

contract credit.pool_level_pay.p07 on entity asset.trust {
  term 2018-10..2021-03
  terms {
    balance = 216238.15
    rate = 0.00992
    term_months = 30
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 41
  }
}

contract credit.pool_level_pay.p08 on entity asset.trust {
  term 2018-10..2022-07
  terms {
    balance = 354043.75
    rate = 0.00907
    term_months = 46
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 26
  }
}

contract credit.pool_level_pay.p09 on entity asset.trust {
  term 2018-10..2022-12
  terms {
    balance = 342126.24
    rate = 0.00905
    term_months = 51
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 21
  }
}

contract credit.pool_level_pay.p11 on entity asset.trust {
  term 2018-10..2020-02
  terms {
    balance = 610459.31
    rate = 0.01906
    term_months = 17
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 41
  }
}

contract credit.pool_level_pay.p12 on entity asset.trust {
  term 2018-10..2021-04
  terms {
    balance = 1144291.74
    rate = 0.01951
    term_months = 31
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 32
  }
}

contract credit.pool_level_pay.p13 on entity asset.trust {
  term 2018-10..2022-02
  terms {
    balance = 699535.89
    rate = 0.01949
    term_months = 41
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 23
  }
}

contract credit.pool_level_pay.p14 on entity asset.trust {
  term 2018-10..2022-12
  terms {
    balance = 201897.47
    rate = 0.01869
    term_months = 51
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 21
  }
}

contract credit.pool_level_pay.p16 on entity asset.trust {
  term 2018-10..2020-02
  terms {
    balance = 13918351.08
    rate = 0.02594
    term_months = 17
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 40
  }
}

contract credit.pool_level_pay.p17 on entity asset.trust {
  term 2018-10..2021-04
  terms {
    balance = 26181002.53
    rate = 0.02626
    term_months = 31
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 31
  }
}

contract credit.pool_level_pay.p18 on entity asset.trust {
  term 2018-10..2022-02
  terms {
    balance = 28740527.64
    rate = 0.02684
    term_months = 41
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 24
  }
}

contract credit.pool_level_pay.p19 on entity asset.trust {
  term 2018-10..2022-12
  terms {
    balance = 9735143.46
    rate = 0.02794
    term_months = 51
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 21
  }
}

contract credit.pool_level_pay.p21 on entity asset.trust {
  term 2018-10..2020-02
  terms {
    balance = 14533243.98
    rate = 0.03678
    term_months = 17
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 40
  }
}

contract credit.pool_level_pay.p22 on entity asset.trust {
  term 2018-10..2021-04
  terms {
    balance = 26195374.46
    rate = 0.03667
    term_months = 31
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 32
  }
}

contract credit.pool_level_pay.p23 on entity asset.trust {
  term 2018-10..2022-03
  terms {
    balance = 37348352.52
    rate = 0.03671
    term_months = 42
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 26
  }
}

contract credit.pool_level_pay.p24 on entity asset.trust {
  term 2018-10..2023-01
  terms {
    balance = 19509631.08
    rate = 0.03673
    term_months = 52
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 20
  }
}

contract credit.pool_level_pay.p26 on entity asset.trust {
  term 2018-10..2020-02
  terms {
    balance = 12183065.19
    rate = 0.04661
    term_months = 17
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 42
  }
}

contract credit.pool_level_pay.p27 on entity asset.trust {
  term 2018-10..2021-04
  terms {
    balance = 20323443.61
    rate = 0.04674
    term_months = 31
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 33
  }
}

contract credit.pool_level_pay.p28 on entity asset.trust {
  term 2018-10..2022-03
  terms {
    balance = 32071657.98
    rate = 0.04690
    term_months = 42
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 27
  }
}

contract credit.pool_level_pay.p29 on entity asset.trust {
  term 2018-10..2023-01
  terms {
    balance = 20332473.43
    rate = 0.04674
    term_months = 52
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 20
  }
}

contract credit.pool_level_pay.p31 on entity asset.trust {
  term 2018-10..2020-02
  terms {
    balance = 6428613.14
    rate = 0.05572
    term_months = 17
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 43
  }
}

contract credit.pool_level_pay.p32 on entity asset.trust {
  term 2018-10..2021-05
  terms {
    balance = 16325861.98
    rate = 0.05566
    term_months = 32
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 35
  }
}

contract credit.pool_level_pay.p33 on entity asset.trust {
  term 2018-10..2022-04
  terms {
    balance = 34020451.15
    rate = 0.05608
    term_months = 43
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 28
  }
}

contract credit.pool_level_pay.p34 on entity asset.trust {
  term 2018-10..2023-01
  terms {
    balance = 22175932.04
    rate = 0.05615
    term_months = 52
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 20
  }
}

contract credit.pool_level_pay.p36 on entity asset.trust {
  term 2018-10..2020-03
  terms {
    balance = 4214767.90
    rate = 0.06583
    term_months = 18
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 44
  }
}

contract credit.pool_level_pay.p37 on entity asset.trust {
  term 2018-10..2021-05
  terms {
    balance = 10197295.25
    rate = 0.06567
    term_months = 32
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 35
  }
}

contract credit.pool_level_pay.p38 on entity asset.trust {
  term 2018-10..2022-04
  terms {
    balance = 28511150.24
    rate = 0.06580
    term_months = 43
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 28
  }
}

contract credit.pool_level_pay.p39 on entity asset.trust {
  term 2018-10..2023-01
  terms {
    balance = 21518975.29
    rate = 0.06583
    term_months = 52
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 21
  }
}

contract credit.pool_level_pay.p40 on entity asset.trust {
  term 2018-10..2024-01
  terms {
    balance = 210992.57
    rate = 0.06671
    term_months = 64
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 9
  }
}

contract credit.pool_level_pay.p41 on entity asset.trust {
  term 2018-10..2020-02
  terms {
    balance = 2314366.62
    rate = 0.07537
    term_months = 17
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 45
  }
}

contract credit.pool_level_pay.p42 on entity asset.trust {
  term 2018-10..2021-04
  terms {
    balance = 6049009.56
    rate = 0.07527
    term_months = 31
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 35
  }
}

contract credit.pool_level_pay.p43 on entity asset.trust {
  term 2018-10..2022-04
  terms {
    balance = 17752272.88
    rate = 0.07538
    term_months = 43
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 28
  }
}

contract credit.pool_level_pay.p44 on entity asset.trust {
  term 2018-10..2023-02
  terms {
    balance = 17560641.20
    rate = 0.07526
    term_months = 53
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 20
  }
}

contract credit.pool_level_pay.p45 on entity asset.trust {
  term 2018-10..2024-01
  terms {
    balance = 133227.13
    rate = 0.07709
    term_months = 64
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 8
  }
}

contract credit.pool_level_pay.p46 on entity asset.trust {
  term 2018-10..2020-02
  terms {
    balance = 4089106.53
    rate = 0.09923
    term_months = 17
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 43
  }
}

contract credit.pool_level_pay.p47 on entity asset.trust {
  term 2018-10..2021-04
  terms {
    balance = 9761650.69
    rate = 0.09773
    term_months = 31
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 35
  }
}

contract credit.pool_level_pay.p48 on entity asset.trust {
  term 2018-10..2022-05
  terms {
    balance = 26285138.49
    rate = 0.09619
    term_months = 44
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 26
  }
}

contract credit.pool_level_pay.p49 on entity asset.trust {
  term 2018-10..2023-02
  terms {
    balance = 29949234.04
    rate = 0.09622
    term_months = 53
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 20
  }
}

contract credit.pool_level_pay.p50 on entity asset.trust {
  term 2018-10..2023-11
  terms {
    balance = 279866.82
    rate = 0.09836
    term_months = 62
    cpr = 0
    cdr = 0
    abs_speed = 0.015
    age_months = 11
  }
}
```

## Run configuration

```json
{"deterministic":{"annual_discount_rate":0.03}}
```

## Verified results

Checked period by period: **1 series** across **14 periods**, each within ±0.01 of the reference.

- `net_cash_flow`

Summary metrics:

| Metric | Value | Tolerance |
|---|---:|---:|
| `domain.credit.principal` | 537,640,787.96 | ±0.01 |
