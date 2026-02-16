# Engine Accuracy: Recommended Next Steps

This document outlines recommended next steps to continue enhancing and improving the accuracy of the CFDL engine. It builds on the engine accuracy validation tooling in `tools/engine-accuracy/`, which runs example models, recomputes key metrics from results, and compares against engine output (see `tools/engine-accuracy/README.md` and `ENGINE_ACCURACY_REPORT.md`).

---

## 1. Strengthen in-repo tests

- **Unit tests for core math**  
  Add tests in `crates/cfdl-engine` that:
  - Compute `model.total` as the sum of `model.net_cash_flow` series and assert it equals the engine’s `model.total` metric.
  - Compute NPV from the same series using the documented formula `sum(cf[i] / (1 + r)^i)` and assert it equals the engine’s `model.npv`.
  - Use the same tolerance as the validation script (e.g. 1e-6) and the same rounding policy as `round_amount`.

- **Property-style checks**  
  For a small set of IR fixtures (or generated IR), run the engine and assert internal consistency: stream totals equal sum of series values; model total equals sum of net cash flow; NPV matches recomputation from series and discount rate.

---

## 2. Lock golden results for examples

- **Example goldens**  
  Extend the golden suite (or add a separate step) so that selected **example** runs produce locked golden results (e.g. under `gold/results/examples/` or similar).  
  - Run `tools/engine-accuracy/run_all.sh` (or equivalent), then promote `out/*.results.json` to gold only when intended (e.g. via an env flag like `CFDL_GOLD_UPDATE=1` for fixtures).  
  - This catches regressions in engine output for real-world example models, not just fixtures.

- **CI**  
  In CI, after building the CLI, run the example golden comparison so that any change to example results is explicit and reviewed.

---

## 3. Independent recomputation (from IR + config)

- **Beyond internal consistency**  
  Current validation recomputes from the engine’s **output** (series and metrics). A stronger check is to recompute from **IR + run config** only (timeline, schedules, CEL evaluation, then NPV/totals).  
  - Options: (a) Implement a small “reference” calculator in Python/Rust that reads IR and run config, evaluates stream amounts (e.g. for constant or override-only cases first), builds net cash flow, then applies the same NPV/total formulas. (b) Or add a “dump expected series” mode to the engine and compare against a hand-maintained or script-generated expected file for a few small models.  
  - Start with one or two minimal models (e.g. minimal_model, first_stream) where amounts are constants or overrides, then expand.

- **Document formulas**  
  Document the exact NPV and total formulas (and rounding) in `docs/` or in the engine crate so that any reference implementation and validation script stay aligned.

---

## 4. Scenarios and Monte Carlo

- **Scenario NPV**  
  For run configs that define scenarios, validate that each scenario’s `model.npv` (and optionally totals) can be recomputed from the same formulas applied to that scenario’s effective config (e.g. discount rate, parameter overrides).

- **Monte Carlo**  
  For Monte Carlo runs, add checks that:  
  - Trial-level NPVs are consistent with the same period-end NPV formula.  
  - Summary stats (mean, median, stddev) match recomputation from the trial NPVs (with documented tolerance for floating point).  
  - Optionally, validate distribution sampling (e.g. fixed seed and known distributions) against expected moments for a small test.

---

## 5. Expression and schedule coverage

- **CEL / expression coverage**  
  Ensure engine-accuracy (or fixture) runs cover:  
  - All CEL built-ins used in production (e.g. `pow`, `min`, `max`, `round`, date functions).  
  - Edge cases: zero periods, single period, large period index, `active_when` toggling streams off.  
  - Parameter overrides (`stream.<name>:amount`, `cfg.*`) so that overrides are applied and reflected in series and metrics.

- **Schedule and timeline**  
  Add or tag fixtures that stress:  
  - Different calendars (monthly, quarterly, annual).  
  - Schedule boundaries (first/last period, `from`/`to`).  
  - Multi-stream models where model net cash flow is the sum of several streams.  
  Validate that timeline generation and schedule masking match documented behavior (e.g. in `docs/` or engine comments).

---

## 6. Documentation and process

- **Accuracy report in CI**  
  Run `tools/engine-accuracy/run_all.sh` in CI and publish or archive `ENGINE_ACCURACY_REPORT.md` (and optionally `validation_report.json`) so that regressions (new failures or mismatches) are visible.

- **Changelog**  
  When changing engine formulas, rounding, or semantics, document the change and update any reference expectations or goldens; mention in release notes if user-facing.

- **Tolerance and rounding**  
  Keep a single, documented tolerance (e.g. 1e-6) and rounding policy (e.g. 6 decimals) for all numeric comparisons and engine output; reference it from this doc and from `tools/engine-accuracy/`.

---

## 7. Priorities (suggested order)

1. Add engine unit tests for NPV and model.total from series (low effort, high value).  
2. Run engine-accuracy in CI and fail on new failures or mismatches.  
3. Lock golden results for example runs and add CI comparison.  
4. Document NPV/total formulas and rounding in `docs/`.  
5. Implement independent recomputation from IR for one or two minimal models.  
6. Extend validation to scenarios and Monte Carlo summaries.  
7. Add coverage for more CEL/schedule edge cases and document expected behavior.

---

## References

- **Engine accuracy tooling**: `tools/engine-accuracy/README.md`  
- **Current report**: `tools/engine-accuracy/ENGINE_ACCURACY_REPORT.md`  
- **Engine implementation**: `crates/cfdl-engine/src/lib.rs` (e.g. `npv`, `round_amount`, `run_deterministic`)  
- **User/run config**: `docs/USER_GUIDE.md`  
- **Results schema**: `docs/cfdl_v_0_1_results_schema.md` (if applicable)
