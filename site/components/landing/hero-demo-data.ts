/**
 * Real output, not illustration.
 *
 * Produced by compiling and running `heroModel` below with the actual CFDL
 * toolchain (cfdl 0.3.0, seed 42, 2000 trials, 8% annual discount rate) and
 * copying the numbers out of `results.json`. The hero shows what the engine
 * actually returns for this model; regenerate with:
 *
 *   cfdl compile <dir> --out ir.json
 *   cfdl run ir.json --config run.json --out results.json
 */
export const heroModel = `version 0.1
model "solar-ppa"
time calendar monthly from 2026-01 for 120

entity project solar

assume degradation ~ Normal(mean=0.005, stdev=0.002, clip=[0.0, 0.012])

stream project.ppa_revenue on entity project.solar inflow currency USD {
  schedule every monthly from 2026-01 to 2035-12
  amount = 4200 * 85 / 12 * pow(1 - inputs.degradation, time.t / 12.0)
}

stream project.om on entity project.solar outflow currency USD {
  schedule every monthly from 2026-01 to 2035-12
  amount = 70000 / 12
}`;

export const heroRunConfig = `{
  "deterministic": { "annual_discount_rate": 0.08 },
  "monte_carlo": { "trial_count": 2000, "seed": 42 }
}`;

export const heroResults = {
  deterministic: { npv: 1_954_958.82 },
  monteCarlo: {
    trials: 2000,
    seed: 42,
    mean: 1_956_195.15,
    stdev: 20_312.84,
    p05: 1_921_963,
    p50: 1_956_580,
    p95: 1_989_638,
    min: 1_889_412,
    max: 2_008_263,
    /** 15 equal-width bins across [min, max]. */
    histogram: [1, 11, 26, 56, 82, 180, 231, 265, 311, 296, 229, 152, 87, 39, 34],
  },
} as const;
