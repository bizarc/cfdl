---
id: pack-credit
title: "Credit"
slug: "/docs/packs/credit"
description: "The credit pack: loan pools amortizing to schedule while prepaying and defaulting, and what reaches each note holder."
generated: regions
---

# Credit

Loan pools and structured credit: what a pool collects, and what reaches a note holder.

## What it models

A pool amortizing to schedule while prepaying and defaulting against market-standard curves — CPR, SMM, PSA and SDA — with recoveries arriving on a lag and a servicing fee taken off the top. Fixed and floating coupons, the latter reset off a declared curve.

## Contracts

Declare a contract and the pack expands it into the streams those terms imply,
each classified so it lands on the right line of a
[statement](/docs/reference/statements).

<!-- cfdl:generated contracts-credit -->
| Contract | Terms it reads | Streams it emits |
|---|---|---|
| `credit.loan` | `abs_speed`, `age_months`, `cdr`, `cpr`, `index_curve`, `interest_rate`, `margin`, `payment_frequency`, `prepay_penalty_rate`, `principal`, `psa_speed`, `rate_cap`, `rate_floor`, `sda_speed`, `servicing_fee`, `severity` | `credit.loan.interest[.suffix]`, `credit.loan.sched_principal[.suffix]`, `credit.loan.prepay[.suffix]`, `credit.loan.recoveries[.suffix]`, `credit.loan.servicing[.suffix]`, `credit.loan.penalty[.suffix]`, `credit.loan.bullet[.suffix]` |
| `credit.purchase` | `price` | `credit.purchase.price[.suffix]` |
| `credit.participation` | `share` | `credit.participation.interest[.suffix]`, `credit.participation.principal[.suffix]` |
| `credit.note` | `coupon`, `face`, `payment_frequency`, `principal_account` |  |
<!-- /cfdl:generated contracts-credit -->

A contract can be declared more than once by giving it a suffix, so the pieces
stay separable in the results.

## Reporting

Three views of one pool, because the asset class has more than one convention: a collections statement, an agency-style remittance report splitting principal scheduled from unscheduled, and a statement of operations reporting total and net investment income.

## Related

- [Statements](/docs/reference/statements) — the pro forma this pack produces
- [Metrics](/docs/reference/metrics) — what it reports over the whole model
- [Validation](/docs/benchmarks) — the reference models it is gated against
