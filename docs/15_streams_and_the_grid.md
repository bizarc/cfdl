# Streams and the model grid — design

Status: **REJECTED.** Considered, not taken. The reasoning is below; the
proposal itself is kept intact beneath it because a design that was weighed and
declined is worth more on the record than deleted.

---

## Why this was not taken

This proposed retiring `E2108_SCHEDULE_FINER_THAN_CALENDAR` and adding an
occurrence layer, so a stream could emit flows at times finer than the model
grid and have them bucketed afterwards.

The alternative that was taken instead is the **grain rule**, written into
`docs/01_language_spec.md` beside the `E2108` definition:

> Model at the finest grain at which anything varies; report at any coarser
> grain by folding.

Three reasons it wins:

1. **It costs nothing.** A model that needs monthly mechanics declares a monthly
   timeline. Nothing is lost, because reporting at a coarser grain is a
   regrouping of the same ledger — a statement, an annual rollup and a valuation
   each name the grain they report at, and all coexist in one run.

2. **This proposal's benefit is not the benefit reporting needed.** Its stated
   payoff was drill-down to individual sub-period payments. What a statement
   actually needs is drill-down to the CONTRIBUTING STREAMS and their contract
   terms, which the statement layer delivers without touching the evaluation
   path.

3. **The cost is concentrated in the most load-bearing code there is.**
   `evaluate_stream` and the per-period environment are what every number in
   every benchmark passes through. Changing them to support a modeling choice
   the grain rule handles better is a large blast radius for no gain.

Backlog **7.16** describes the underlying limitation and is answered by the
grain rule rather than by this design: the case it describes becomes
unconstructable, and `E2108` is what makes it so.

---

## The original proposal follows, unchanged

The intended model: **the grid is fixed and always exists; a stream is not
related to it.** A stream emits inflows and outflows at *times*, from its own
rules. Those flows are then bucketed into grid periods, traceably.

The engine is already half-way there, and the half that is missing is smaller
than it looks.

---

## 1. A stream's schedule is already grid-free

```rust
fn occurrences(from: &Date, to: &Date, interval: &str) -> Result<Vec<Date>, EngineError>
```

`crates/cfdl-engine/src/lib.rs:1982`. It takes no timeline. A stream's
occurrence dates are generated purely from its own `from`, `to` and `interval`,
and nothing about the model calendar reaches it.

So the premise is already true at the point it matters most. What follows is
where it stops being true.

## 2. Where the grid gets in

All of this is one function, `apply_schedule_indices`, `:2131-2195`.

| line | step | needs the grid? |
|---|---|---|
| 2131 | `occurrences(from, to, interval)` → the stream's own dates | **no** |
| 2133 | `period_index(timeline, start)` → accrual becomes an index | yes |
| 2141-2156 | which period does this interval *close* in | yes — by searching the timeline |
| 2170 | billing date = `period_end(timeline, pay_idx)` | yes — **and this is the substantive one** |
| 2171 | day rule = `place_in_interval(&timeline[pay_idx], …)` | the helper is date-based; it is *fed* a grid date |
| 2178-2181 | `net <n>` — add days or months | **no**, pure date arithmetic |
| 2183 | business-day roll | **no**, pure date arithmetic |
| 2194 | `period_index(timeline, &rolled)` → settlement bucket | yes — **and legitimately so** |
| 2195 | `out[settled].push(accrual_idx)` | the dates die here |

Exactly **one** of those steps genuinely needs the grid: the last
`period_index`, which is bucketing and belongs there. Every other grid reference
is incidental — and note the bucketing is already many-to-one. `out[settled]` is
a `Vec`, so several occurrences landing in one period is the normal case, not an
error case. Measured: weekly payments on a monthly grid bucket 4, 4, 4, 5, 4 per
month and sum correctly. `E2108` rejects that model before the engine sees it.

### Why the dates cannot survive: two jobs in one pass

`apply_schedule_indices` interleaves *resolving the stream's schedule* with
*placing it on the grid*. Because they are interleaved, intermediate dates are
read **off the grid** instead of computed from the occurrence:

```rust
let billed = match (has_terms, schedule.on_rule.as_ref()) {
    (true, None) => period_end(timeline, pay_idx),          // the GRID period's end
    _ => place_in_interval(&timeline[pay_idx], …),          // the GRID period's date
};
```

An invoice should be dated from the end of **its own accrual interval**. Here it
is dated from the end of whichever grid period that interval happened to land
in. For a stream at the grid's cadence the two coincide, which is why this has
never produced a wrong number — `E2108` guarantees the cadences are compatible,
and the compatible case is the one where the substitution is harmless.

The interval's own close is *already computed*, two lines earlier:

```rust
let next = starts.get(k + 1).cloned().unwrap_or_else(|| step_once(start, interval));
```

`next` is used to *search the timeline* rather than used directly. The value the
grid is being consulted for is already in hand.

## 3. The occurrence is never a value

This is the substantive gap, and it is not about bucketing.

```rust
let amount = eval_amount_expr(&amount_expr, &env, …);
values[pay_idx] += amount * direction_sign;
```

The amount is computed and folded into a period vector in the same statement.
It is never paired with its date — by that line the occurrence's date is not
even in scope, only `idx` and `pay_idx`. Upstream, `out[settled].push(accrual_idx)`
stored a `usize`; the ordinal, accrual date, billing date, due date and settle
date all existed inside the resolution loop and none survived it.

`evaluate_stream` returns `Vec<f64>`. A stream goes from *authored* to *a period
vector* in one pass, and the intermediate steps are loop bookkeeping rather than
values.

And nothing occurrence-level reaches the output. `DeterministicSection` is
`{ status, metrics, series, errors, annual_rollup }` — every field is
period-shaped.

So a reported period total of 2,500 cannot be traced to five payments of 500 on
five dates. The payments were never represented.

## 4. The pipeline, and where it breaks

| step | today |
|---|---|
| 1. stream authored | parser → IR |
| 2. **resolved into occurrences** | dates generated by `occurrences()`, then **consumed** — no artefact, and no amount attached |
| 3. occurrences mapped to the grid | happens, but only the period index survives |
| 4. inflows/outflows netted per period | `values[pay_idx] +=`, then `record_stream` → `model_series` |
| 5. cash flows discounted | by period index plus one offset per stream |
| 6. NPV, IRR | over those period vectors |

Steps 3–6 are sound. Step 2 has no output, and that is what removes audit,
transparency, per-occurrence reporting, and counts in one go.

## 5. The design: make step 2 produce a value

A stream resolves to its own **ledger** — occurrences with amounts, before any
grid is involved:

```
Flow { ordinal, accrual_date, settle_date, amount, direction }
```

Bucketing is then a fold over the ledger, not a side effect of building it:

```
series[p] = sum of flows whose settle_date falls in period p
```

The period total is traceable **by construction**: it is the sum of a known,
addressable list. Counts fall out for free — a bucket's length. So does "which
payments make up this number".

### The ledger has to be an output, not an internal

This is the part that makes it a real change rather than a refactor. Audit and
transparency mean the ledger must be reachable from the results, so
`DeterministicSection` gains an occurrence-level section alongside `series`.
That is a **published contract change**, versioned and gated by
`check-results-schema` across three copies.

Two things to decide with it:

- **Size.** A daily stream over thirty years is ~11,000 flows. Per stream. The
  ledger is bounded by occurrence count, not period count, and a results file
  that today is period-shaped becomes occurrence-shaped. Whether it is always
  emitted, emitted on request, or emitted at a summarized grain is a real
  decision, not a detail.
- **What `series` becomes.** It stays exactly as it is — one value per stream per
  grid period — and is now *derived* from the ledger rather than accumulated
  directly. The reporting shape does not change; its provenance does.

### The grid is unchanged

- `time calendar … for N` — unchanged.
- `results.deterministic.series` — unchanged in shape and value.
- `series_sum(name, from_t, to_t)` — still windows bucketed series by grid index.
- States, metrics, the annual rollup, Monte Carlo, scenarios — unchanged.

## 6. What changes for a stream

| | |
|---|---|
| `schedule_accruals` | returns occurrences, not `Vec<Vec<usize>>` |
| `evaluate_stream` | returns a **ledger** of dated flows, not `Vec<f64>` |
| bucketing | a fold over the ledger, performed after evaluation rather than during it |
| results | gains an occurrence-level section — a published contract change |
| `E2108` | retires: a stream's cadence stops being the grid's business |
| counts, audit, traceability | fall out of the ledger; unavailable today at any price |

### The open question: what does an expression see per flow?

`time.t` is the grid period index and is named after the grid. When one grid
period holds twelve flows, all twelve are in period `t`.

**Recommendation: leave `time.t` as the grid period.** It is backward compatible,
including for coarser-than-grid streams that exist today — a quarterly stream on
a monthly grid has ordinals 0, 1, 2 at periods 0, 3, 6, and changing `time.t` to
the ordinal would silently move every one of them.

Packs are already occurrence-oriented and need no change:
`{{time.elapsed_periods}}` counts the *rule's own payment periods*
(`crates/cfdl-compile/src/lib.rs:2079`), not grid periods. It is native models
using `time.t` that are grid-oriented, and they should stay that way.

**`time.date` needs its own decision.** It is currently the accrual *period's*
start date, so under `on day 15` an expression sees the 1st. Making it the
flow's own date is more honest and is a real behavior change for day-rule
schedules that exist today.

## 7. One thing that must move with it

`{{time.periods_to_term_end}} == 0` is the balloon idiom in
`cre.permanent_debt` and `opco.term_debt`. It is a *last grid period* test, and
it is true for **every** flow in that period — twelve balloons on an annual
model. It has to become a last-*flow* test before `E2108` can retire, or the
change is worse than the gap.

## 8. What atomic streams unlock, beyond the obvious

Going back over the items the validation runs produced, the ledger reaches more
of them than "occurrences are indistinguishable" suggests.

**Directly solved**

- **7.16** occurrences inside a period are indistinguishable — by construction.
- **6.3** a flow settling in a period other than its term's. *"A sale agreed in
  one period and settling in another"* is inexpressible today because `on_date`
  has one date. A flow carries its accrual date and its settle date
  independently, so this stops being a case.

**Unlocked, where the ledger is the prerequisite rather than the fix**

- **6.1** `act/act` day count. Blocked because *"it needs the days in the year
  the period falls in, which the expression environment does not expose"*. A
  dated flow knows its own date, hence its year. Depends on the `time.date`
  decision in §6.
- **2.2** Actual-basis pool amortization. The remaining half needs a per-period
  divisor; with dated occurrences the accrual interval is a property of the flow
  rather than of the grid period it landed in.
- **7.8** a stream cannot be non-cash. If a flow carries a classification, a
  non-cash quantity is a flow kind excluded from cash aggregation rather than a
  new stream kind. (Mostly superseded by declared states, but the general form
  falls out here.)
- **1.3** abatements as a first-class NOI line. *"You can report it as a line OR
  have it counted in NOI, not both"* — because a line's presentation and its
  arithmetic role are the same object. Flows carrying a classification separate
  the two. The consuming half is reporting.
- **3.1, 3.2, 3.3** stub periods, one-shot placement, and the settlement-lag
  remainder. All need dated flows to reach valuation, which the ledger provides
  and this document does not spend.

**Not touched by it** — cross-stream reads (1.1), contract shapes
(1.5, 1.7, 7.5, 7.9), ordered allocation (5.2, 2.4), and everything that needs a
source rather than a capability.

## 9. Where this sits in the reporting story

**Reporting is its own capability — backlog 7.17 — and this is one of five gaps
in it.** For a line-item pro forma it is not the largest. Measured against
`benchmarks/cre/office_two_tenant`: the line items already exist as thirteen
per-period series; what is missing is per-period **subtotals** (`domain.cre.noi`
is one number for a ten-year hold), statement **structure**, **drill-down**
(this document), reporting **grain**, and **counts**.

Per-period subtotals can land without the ledger, and would immediately let
`benchmarks/cre/hud_home_multifamily` assert four published coverage ratios it
currently reproduces by hand. Drill-down and counts need the ledger. Statement
structure needs subtotals underneath it.

This document is the ledger only. It is not a reporting design.

## 10. What this deliberately excludes

Discounting, weighted average life and payback also measure time in grid indices
(`(1+r)^-i`, `(t + offset)/ppy`), and `RunConfig.as_of` is parsed and published
but never enters a discount factor — backlog items 3.1, 3.2, 3.3, 6.3 and 7.4
describe the consequences. A dated ledger is the prerequisite for addressing
them, not the fix, and what valuation then does with dated flows is a separate
decision.

## 11. Verification

- **`git diff gold/` must be empty.** Every model that exists today has a stream
  cadence at or coarser than the grid, which is exactly the case where the
  substitutions in §2 are harmless. If a golden moves, the restructure changed
  behavior it should not have.
- **The billing substitution is the one to probe.** Construct a coarser-than-grid
  stream with `net <n>` — a quarterly accrual on a monthly grid — and confirm the
  billing date is the quarter's close both before and after.
- **Then the case that could not be built:** a monthly-paying loan on an annual
  model. `benchmarks/cre/hud_home_multifamily` needs it, and measured with
  `E2108` bypassed the contract already returns the workbook's published
  13,314.3827 (7.14).
- **Traceability is testable:** a period's reported value must equal the sum of
  the flows bucketed into it, by construction rather than by assertion.
