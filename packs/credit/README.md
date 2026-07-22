# credit pack v0.1

Credit / lending pack: fixed-rate loan pools with CPR prepayments, CDR
defaults, loss severity and a recovery lag. Benchmarked in
`benchmarks/credit/` against independent month-by-month reference
implementations (LAUNCH_PLAN §6D).

## Contract types

### `credit.pool_level_pay`

Homogeneous level-pay (fully amortizing) pool. The engine's expression
dialect has no loops, so streams use the exact closed form for a homogeneous
pool under constant SMM/MDR (the standard pool-factor decomposition) — see
the convention block at the top of `lowering/rules.toml`.

Terms:

| term | meaning | default |
|---|---|---|
| `balance` | original pool balance | required |
| `rate` | annual note rate (must be > 0) | required |
| `term_months` | amortization term in months | required |
| `cpr` | annual conditional prepayment rate | `0` |
| `cdr` | annual conditional default rate | `0` |
| `severity` | loss severity on defaulted balance | `0` |
| `recovery_lag_months` | months from default to recovery cash | `0` |

Streams (all suffixed by contract instance): `credit.pool.interest`,
`credit.pool.sched_principal`, `credit.pool.prepay`,
`credit.pool.recoveries`.

The contract `term` must span `term_months + recovery_lag_months` schedule
periods so the recovery tail has periods to land in; the expressions gate
themselves, so a longer term is harmless.

### `credit.pool_io_bullet`

Interest-only pool with a principal bullet at maturity. Balance declines
only through prepayment and default; the final period pays no SMM
prepayment — the whole surviving balance pays as the bullet. Same terms as
`pool_level_pay` (`rate` may be anything, including 0). Adds stream
`credit.pool.bullet`.

### `credit.purchase`

Acquisition price paid at `term_start` (`price` term), stream
`credit.purchase.price`.

## Conventions

- Defaults leave the pool at the start of the period and earn no interest
  that period.
- `cpr`/`cdr` are annualized; converted with `cpr_to_smm(x) = 1 - (1-x)^(1/12)`.
- Prepayment applies SMM to the performing balance net of scheduled
  principal.
- Recoveries return `(1 - severity)` of defaulted face,
  `recovery_lag_months` later. Defaulted-balance write-offs are not cash
  and emit no stream.

## Metrics

`domain.credit.interest`, `.principal` (scheduled + prepay + bullet),
`.recoveries`, `.collections`, `.purchase`, `.collections_multiple`
(collections / purchase).

## Not in v0.1

- **Floating-rate loans** — needs the `curve` input concept and
  mean-reverting rate paths (LAUNCH_PLAN stochastic roadmap item 4).
- Zero note rate on `pool_level_pay` (closed form divides by `r`).
- Delinquency states, servicer advances, loan-level heterogeneity.
