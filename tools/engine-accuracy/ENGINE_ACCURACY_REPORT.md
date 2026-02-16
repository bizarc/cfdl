# CFDL Engine Accuracy Report

## Summary

- **Total runs attempted**: 17
- **Compile/run succeeded**: 17
- **Compile/run failed**: 0
- **Validated** (comparison run): 17
- **Match**: 17
- **Mismatch**: 0

## Methodology

- **Recomputed metrics**:
  - `model.total` = sum of `deterministic.series["model.net_cash_flow"].values[].amount`
  - `model.npv` = sum of `cf[i] / (1 + discount_rate)^i` for each period (period-end discounting, matching engine).
- **Optional**: For each `stream.<name>` series, sum of `values[].amount` compared to `stream.<name>.total` in metrics.
- **Tolerance**: `1e-06` (consistent with engine 6-decimal rounding).

## Accuracy

- **Runs where all checked metrics matched**: 17 / 17 (100.0%).

## Issues

No mismatches. All validated runs matched recomputed metrics.

## Failures

No compile or run failures.

## Recommendations

- Consider adding unit tests in cfdl-engine for NPV and model.total from series.
- Optionally lock golden results for example runs to catch regressions.
