---
id: guide-scenarios
title: Scenarios and run configurations
slug: /docs/guides/scenarios-and-run-configs
description: "Evaluate one compiled model under many run configurations: base case, stress case, and sensitivities, without editing the model."
generated: none
---

# Scenarios and run configurations

A compiled model (IR) is evaluated under a **run configuration** — so the same
model answers base case, stress case, and sensitivity questions without
editing the model file.

## Base + stress in one run

`run.json`:

```json
{
  "deterministic": {
    "annual_discount_rate": 0.10,
    "parameters": { "stream.cre.lease.base_rent:amount": 1000.0 }
  },
  "scenarios": {
    "stress": {
      "annual_discount_rate": 0.12,
      "parameters": { "stream.cre.lease.base_rent:amount": 800.0 }
    }
  }
}
```

```bash
cfdl run my-deal/ir.json --config run.json --out results.json
```

Results carry the deterministic block plus one block per scenario — same
metrics, directly comparable.

## Override keys

- `stream.<dotted_stream_name>:amount` — replace a stream's amount.
- `cfg.<path>` — set values expressions read via `cfg.<path>`; this is the
  idiomatic knob for parameters the model deliberately externalizes.

## Sensitivity sweeps

Generate configurations programmatically (they are plain JSON) or drive sweeps from
the [Python SDK](/docs/python-sdk), where `results.scenarios()` lands each
scenario as a DataFrame row.

## Monte Carlo

The `monte_carlo` section (trials, seed, optional distribution overrides)
turns the same IR into a distribution — see
[Stochastic modeling](/docs/stochastic-modeling) and the
[run-config reference](/docs/reference/run-config).

## Reference links

- [Run-config reference](/docs/reference/run-config)
- [CLI reference — cfdl run](/docs/reference/cli)
