---
id: getting-started
title: Getting started
generated: none
---

# Getting started

Ten minutes from nothing to a running model with a probability distribution
around its NPV. No install needed for the first half.

## Try it in the browser

Open the [Playground](/playground) and paste this model:

```cfdl
version 0.1
model "first-model"
time calendar monthly from 2026-01 for 24

entity asset company : Asset.Financial

assume growth ~ Normal(mean=0.02, stdev=0.01, clip=[0.0, 0.05])

stream company.revenue on entity asset.company inflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 10000 * pow(1 + inputs.growth, time.t / 12.0)
}
```

Click **Run**. The compiler and engine execute entirely in your browser.

What each line does:

- `time calendar monthly from 2026-01 for 24` — every stream lands on this
  24-month grid.
- `entity asset company : Asset.Financial` — the thing that owns the cash flow.
  `asset` is its family and `Asset.Financial` its type. See [the object
  model](/docs/object-model).
- `assume growth ~ Normal(...)` — a stochastic assumption: a growth rate
  drawn from a (clipped) normal distribution. Swap `~ Normal(...)` for
  `= 0.02` and it becomes a plain constant.
- `stream ... { schedule ... amount = ... }` — a monthly revenue stream
  whose amount is an ordinary formula; `inputs.growth` reads the assumption
  and `time.t` is the period index.

## Run it with the CLI

Install the CLI (see [Install the CLI](/docs/install/cli) — from a release binary,
or `cargo build -p cfdl-cli` from a checkout). Put the model in  <!-- site-allow: install and support still route through the public repository; revisit when it goes private -->
`first-model/model.cfdl`, then:

```bash
cfdl compile first-model --out first-model/ir.json
cfdl run first-model/ir.json --out first-model/results.json --rate 0.08
```

`results.json` now holds the deterministic answer — per-stream cash flows
plus core metrics. For this model: `model.npv ≈ 227,331 USD` at an 8%
annual discount rate.

## Add the distribution

Create `first-model/run.json`:

```json
{
  "deterministic": { "annual_discount_rate": 0.08 },
  "monte_carlo": { "trial_count": 500, "seed": 42 }
}
```

```bash
cfdl run first-model/ir.json --config first-model/run.json --out first-model/results.json
```

The `monte_carlo` section of the results now summarizes every metric across
500 seeded trials — mean, stdev, min/max, and percentiles. Same model file,
point estimate *and* the distribution around it. Because the seed is
explicit and every assumption has its own draw stream, this output is
byte-reproducible on any machine.

## What you just produced

- `ir.json` — the canonical compiled model
  ([IR schema](/docs/specification/ir-schema)); commit it, diff it, hash it.
- `results.json` — cash flows + metrics
  ([Results schema](/docs/specification/results-schema)); everything
  downstream (pandas, dashboards, reports) reads this.

## Where to go next

**Modeling a deal** (analyst path):

1. Read [the object model](/docs/object-model), then work through the
   [Language guide](/docs/language-guide) and the eight
   [lessons](/docs/examples).
2. Pick your domain's [pack guide](/docs/packs) — energy, CRE, credit, or opco —
   and start from its quickstart.
3. Set up [VS Code](/docs/install/vscode) for diagnostics and hover docs.

**Integrating CFDL** (developer path):

1. [Install the Python SDK](/docs/install/python) and open the
   [notebooks](/docs/python-sdk#notebooks).
2. Or run the [API server](/docs/install/api-server) and hit
   `POST /v1/run`.
3. Read [How CFDL works](/docs/concepts) for the IR/Results contract.
