# Notebook walkthrough scripts

Narration scripts for the two website walkthrough videos. Each scene is one
notebook cell: the capture executes the cell on screen while the narration
line is read (or synthesized). Timings assume an unhurried read; both cuts
land near two minutes. The walkthroughs are agent-driven — the same model
loop the site describes is what is visibly operating the notebook.

## CRE — office acquisition (`02_cre_office_acquisition.ipynb`)

| scene | cell | on screen | narration |
|---|---|---|---|
| 1 | title | Notebook title, one glance at the model directory | "This is an institutional two-tenant office DCF — free rent, expense stops, tenant improvements, probability-blended rollover. Not a toy: every number here is asserted against an independent reference, in CI, with stated tolerances." |
| 2 | compile | `cfdl_sdk.compile(...)` output | "The deal is about a hundred lines of CFDL. One call compiles it. Contracts lower to streams — the compiler is the first verifier." |
| 3 | run | `model.run(config, pack="cre")` | "One call runs it: ten years, monthly, with the CRE pack computing domain metrics — NOI, debt service, coverage." |
| 4 | cashflows | `cf.head()` — 120×26 frame | "Results are a pandas DataFrame. One column per stream, a real period index. From here on, nothing is our UI — it's your pandas." |
| 5 | annual rollup | groupby table with computed `dscr` | "A one-line annual rollup, with a coverage column we compute ourselves. Look at year one: zero-point-five-four coverage. That's lease-up — and a lifetime DSCR of one-point-oh-seven would have hidden it completely." |
| 6 | covenant screen | print + trailing-12 chart | "Which months breach a one-times covenant? Thirty-six of a hundred and twenty — the screen is a filter expression, and the trailing-twelve NOI is a rolling window." |
| 7 | composition | stacked area by tenant | "The lease-by-lease grain survives into results: tenant A, tenant B, recoveries, rollover — stacked straight from the stream columns." |
| 8 | sensitivity | NPV across four discount rates | "Value sensitivity is four engine re-runs in a dict comprehension. Deterministic engine, so this table is reproducible to the byte." |
| 9 | metrics | `metrics_frame()` | "And the deal metrics — a twenty-nine percent IRR — each one traceable to the series that produced it. A verified engine underneath; open pandas on top." |

## Energy — solar PPA microgrid (`01_energy_solar_microgrid.ipynb`)

| scene | cell | on screen | narration |
|---|---|---|---|
| 1 | title | Notebook title | "A two-megawatt solar-plus-storage microgrid on a twenty-five-year power purchase agreement — with storage arbitrage, a capacity payment, an investment tax credit, and project debt." |
| 2 | compile + run | both calls | "Compile, run — three hundred monthly periods, energy-pack metrics included." |
| 3 | verify | `pct_change().unique()` → `[0.0149]` | "Before analyzing a series, interrogate it. Annual PPA revenue should grow at escalation net of degradation. One line of pandas recovers exactly one-point-four-nine percent — two percent escalation times half-a-percent degradation. The convention, verified from the output." |
| 4 | stack | revenue stacked area | "Revenue decomposition: contracted PPA, storage margin, capacity payments — the merchant story and the contracted story, separable because they're separate streams." |
| 5 | coverage | annual DSCR line | "Lender's view: CFADS against debt service, annually, over the life of the loan." |
| 6 | payback | cumulative net + print | "Equity payback: the cumulative line crosses zero here — and the engine's own payback metric agrees. Two derivations, one answer." |
| 7 | metrics | `metrics_frame()` | "The full metric set, sourced and labeled. This is what it looks like when the model is a program: everything computed, everything checkable, nothing trapped in a cell." |

## Capture notes

- Drive Jupyter in a browser at ~1280×800; execute scenes top to bottom with
  a beat on each output; charts get two beats.
- The agent-driven capture is the point: the operator is the model, not a
  human — say so in the page copy next to the video.
- Export: silent screen capture (GIF or mp4) + narration track from this
  script; final mp4 lives in `site/public/`.
