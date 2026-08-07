/**
 * Real output, not illustration.
 *
 * Produced by compiling and running `heroModel` below with the actual CFDL
 * toolchain (seed 42, 2000 trials, 8% annual discount rate) and copying the
 * numbers out of `results.json`. The hero shows what the engine actually
 * returns for this model.
 *
 * `tools/check-doc-examples.py` compiles and runs the model on every commit,
 * so a syntax change cannot leave a model on the landing page that does not
 * run — which is how `every monthly` and an untyped entity survived here after
 * both stopped being valid.
 */
export const heroModel = `version 0.1
model "solar-ppa"
time calendar monthly from 2026-01 for 120

entity asset solar : Asset.Real

assume degradation ~ Normal(mean=0.005, stdev=0.002, clip=[0.0, 0.012])

stream solar.ppa_revenue on entity asset.solar inflow currency USD {
  schedule every month from 2026-01 to 2035-12
  amount = 4200 * 85 / 12 * pow(1 - inputs.degradation, time.t / 12.0)
}

stream solar.om on entity asset.solar outflow currency USD {
  schedule every month from 2026-01 to 2035-12
  amount = 70000 / 12
}`;

export const heroRunConfig = `{
  "deterministic": { "annual_discount_rate": 0.08 },
  "monte_carlo": { "trial_count": 2000, "seed": 42 }
}`;

export const heroResults = {
  deterministic: { npv: 1_942_460.97 },
  monteCarlo: {
    trials: 2000,
    seed: 42,
    mean: 1_943_689.4,
    stdev: 20_182.98,
    p05: 1_909_865,
    p50: 1_944_120,
    p95: 1_976_918,
    min: 1_877_333,
    max: 1_995_425,
    /** 15 equal-width bins across [min, max]. */
    histogram: [1, 11, 26, 56, 82, 180, 231, 265, 311, 296, 229, 152, 87, 39, 34],
  },
} as const;
