# State and recurrence — design

Status: **shipped.** The construct is `state <name> { init … next … }`; see
`fixtures/valid/state_smoke` for the smallest example and `compute_states` in
`crates/cfdl-engine/src/lib.rs` for the evaluation. This document is the design
that produced it, kept because the reasoning is still the reasoning — with
corrections appended in §8 and §9 rather than edited in, so a reader can see
what was believed and what measurement changed.

Filled the gap noted in `docs/13_feature_backlog.md` 5.1/5.2 and referenced four
times in `packs/opco/lowering/rules.toml` as "H3-style state" with no design
written down.

---

## 1. The problem

A quantity whose value in one period depends on its value in the previous one
cannot be expressed. Three shapes, all from real sources:

```
revenue(t)  = revenue(t-1) * (1 + g_t)          growth that decays        (Damodaran)
expense(t)  = round(expense(t-1) * 1.025)       escalation that rounds    (HUD)
balance(t)  = balance(t-1) * (1 - hazard_t)     survival under a ramp     (PSA/SDA/ABS)
```

Today the only compounding primitive is `pow(1 + r, t)`, which applies **one
period's rate as though it had held from the start**. Exact when the rate is
constant; wrong the moment it moves. Measured against Damodaran's published
forecast, that is −2.4% on revenue by year 10; against HUD's, 12.26 on 204,655
at year 29.

## 2. Approaches tried and rejected

Recorded because the design converged through elimination, and a reader deserves
to see why each door is closed rather than take it on trust.

| approach | why rejected |
|---|---|
| **Period-major evaluation with dependency ordering and cycle detection** | Replaces the published guarantee *"cycles are impossible by construction"* (`docs/03_expression_environment.md:136`) with "cycles are possible and detected". A property enforced by analysis is weaker than one enforced by structure, and the detector can be wrong. |
| **Self-reference through existing `series_sum` with a runtime guard** | Same weakening, plus: reads cost O(t) per period → O(n²); the initial condition is a silent `0` (an unmatched or clamped window returns zero, not an error); evaluation order becomes observable semantics. |
| **`exp`/`ln` log-sum trick** | Built and probed. Works numerically — reproduces all ten Damodaran years exactly — but needs a helper stream to hold `ln(1+r_t)`, and **every stream is a cash stream**. The probe's net cash flow came to 22,853.7188 against a true 22,853.6700: a dimensionless logarithm added to dollars, corrupting `model.total`, `model.npv` and `model.wal_years`. Also escapes to `f64`, and cannot reach a *rounded* recurrence, since rounding does not commute with logs. |
| **Fold expression (`cumprod`, `fold`)** | Safe by construction and decimal-exact, but recomputes from scratch each period — O(n²) — and can only fold an expression of the loop index. It cannot accumulate a *stream's* values, so it never reaches a sweep, a reserve or a carryforward. |

The `exp`/`ln` builtins were kept: they are correct, tested, and 2.1's ramps
will want them. They are not the answer to this problem.

## 3. The design

A **declared state variable**: named, seeded, updated once per period, readable
by streams.

```cfdl
state revenue_index {
  init  1.0
  next  prev * (1 + curve_value("growth", time.date))
}
```

- `init` — the value at period 0. **Mandatory.** Not defaultable.
- `next` — the value at every later period. `prev` is bound to this state's
  value at `t-1`.

### 3.0 It is a general construct, not a revenue one

A state has **no entity, no direction, no currency and no schedule**. It is a
named number per period. It sits at the language level alongside `curve` and
`assume` — not in any pack — so every model can declare one regardless of which
pack it uses.

The recurrence shape is whatever the `next` expression says, and the operator
matters as much as the construct:

```cfdl
state survival        { init 1.0  next prev * (1 - hazard(time.t)) }
state opex_index      { init 1.0  next round_to(prev * 1.025, 1) }
state degradation     { init 1.0  next prev * (1 - 0.005) }
state discount_factor { init 1.0  next prev / (1 + curve_value("wacc", time.date)) }
state cum_capex       { init 0    next prev + inputs.capex_per_period }
state high_water      { init 0    next max(prev, curve_value("distributions", time.date)) }
```

Multiplication gives a compounding factor — a pool's survival under a ramp
(credit), an escalating expense index (CRE), module decay (energy), a discount
factor under a varying cost of capital. Addition gives an accumulator. `max`
gives a running high-water mark, which is what a GP catch-up or a preferred
return tests against.

The worked example below happens to be a revenue index because that is the case
`benchmarks/opco/damodaran_fcff` needs. Nothing about the construct is specific
to revenue, to opco, or to a pack.

Streams read it as an ordinary environment path:

```cfdl
stream firm.revenue on entity asset.firm inflow currency USD {
  schedule every year from 2026-01 to 2035-01
  amount = 21765.4 * state.revenue_index
}
```

### 3.1 The restriction that makes it safe

Inside `next`, the environment contains:

- `prev` — this state's previous value
- `time.*`, `inputs.*`, curves — the ordinary expression environment
- other states' **previous** values, as `prev.<name>` (§3.1.1)
- stream series **up to and including `t-1`**

It does **not** contain any stream value at period `t`, and it does not contain
any state's value at `t`.

This is not a check that can fail. Same-period values are **absent from the
environment**, exactly as `series` is absent today when a phase-1 stream is
evaluated (`evaluate_stream` receives `None`; phase-2 receives
`Some(&full_series)`). The mechanism already exists and is already load-bearing.

Because every edge a state can create points strictly backward in time, **no
cycle can close.** The guarantee survives, and survives for a better reason than
before: not because streams are forbidden to see each other, but because
everything a state can see is already finished.

### 3.1.1 Reading another state: `prev.<name>`

`prev` is a **namespace**, not just a binding. Bare `prev` is shorthand for this
state's own previous value; `prev.<name>` reads another state's.

```cfdl
state high_water {
  init 0
  next max(prev, prev.distributions)      // prev == prev.high_water
}
```

Two prefixes, two meanings, no overlap — and each is available in exactly one
place:

| prefix | resolves to | present in |
|---|---|---|
| `state.<name>` | that state at the **current** period | stream expressions |
| `prev.<name>` | that state at the **previous** period | `next` expressions |

Enforced by absence, as everything else here is. `next` gets a `prev` map and no
`state` map; a stream gets a `state` map and no `prev` map. Neither is a check
that can fail — the entry is not there to be found. Same mechanism as `series`
being `None` when a phase-1 stream evaluates.

Implementation cost is nil: another `BTreeMap` in `ExprEnv`, resolved by the
existing `lookup(path)`, exactly like `inputs.<name>` and `time.<field>`.

**States may reference each other freely, including mutually.** Because every
state at period `t` is computed from the *completed* `t-1` column, this is
well-founded:

```cfdl
state a { init 1  next prev + prev.b }
state b { init 1  next prev + prev.a }
```

No cycle, no ordering requirement, declaration order irrelevant. It is ordinary
double-buffering — read the previous column, write the current one — so the
engine needs no dependency analysis among states at all.

`prev` has no meaning in `init`, bare or namespaced, since there is no period
-1.

### 3.2 Evaluation

Interleaved, period by period:

```
for t in 0..n:
    for each state:  value[t] = (t == 0) ? init : next(prev = value[t-1], series ≤ t-1)
    for each stream: evaluate at t, with state values at t available
```

O(1) per state per period; O(n) overall. States are computed before streams
within a period, and read only completed data, so **no ordering among states is
needed at all** — a state sees only the completed `t-1` column, so declaration
order is irrelevant and mutual reference between states is well-founded.

The projection tail is included: states run to `periods + projection`, so a
phase-2 stream reading forward finds them populated.

### 3.3 Why the seed is syntax, not a term

This is the single largest failure mode and it is designed out rather than
documented.

Today an out-of-range or unmatched series read returns **0 silently**
(`crates/cfdl-expr/src/lib.rs`, `matched.is_empty()` → `0`; window clamped by
`from.max(0)`). A recurrence with a missing base case would therefore evaluate
to zero for every period, with no diagnostic — the whole series wrong, quietly.

Making `init` a required clause means the compiler rejects a recurrence whose
period 0 is unstated. There is nothing to forget.

## 4. Industry alignment

The design is not novel, which is the point. Five independent traditions
converge on the same three rules — explicit declaration, mandatory seed,
backward-only access:

| system | construct | seed |
|---|---|---|
| **Lustre / SCADE** | `pre(x)`, with `->` for the first instant: `n = 0 -> pre(n) + 1` | mandatory, in the syntax |
| **VHDL / Verilog** | clocked register; combinational loops are an error | reset value |
| **Analytica** | `Dynamic(init, expr)` — literally declared state with a seed | mandatory argument |
| **Elm / Rx** | `foldp(step, seed)` / `scan(seed, step)` | mandatory argument |
| **Anaplan** | `PREVIOUS()`, circularity otherwise rejected | line-item initial |

SCADE's variant is certified for DO-178B avionics, so this discipline is trusted
where a wrong answer kills people. Its causality rule is exactly ours: every
feedback path must pass through a delay.

**The counter-example is the spreadsheet.** Excel permits arbitrary references,
detects circularity at runtime, and offers an "iterative calculation" checkbox
with a max-iteration count. Circular references are among the best-documented
sources of spreadsheet error, and the mechanism is invisible in the file. That
is the design to avoid, and it is what "allow same-period references and detect
cycles" would have reproduced.

## 5. What this does not solve

**Same-period cross-stream dependency.** A cash sweep needs to know how much
cash remains *after this period's* debt service. That is an instantaneous
dependency; no backward-only construct reaches it. It stays 5.2, and the right
shape there is an **ordered allocation pass** — a waterfall is an author-declared
priority over a pot, not a dependency graph to be solved, so it needs no cycle
detection either.

**Genuine fixed points.** Eleven catalogued sources mention "solve" or
"circular"; at least one (A.CRE's mezzanine capital stack) is explicitly
iterative, and Damodaran's LBO re-prices cost of equity at post-LBO leverage.
Mining reports levy royalties on earnings *before* tax, circular with cost and
depreciation. These need iteration to convergence and cannot be by construction.
If ever built, it must be an explicit, bounded, convergence-checked construct —
never a silent checkbox.

## 6. Cost and risk

| | |
|---|---|
| **Language surface** | New: `state` declaration, `init`/`next` clauses, `prev` binding, `state.<name>` path. Larger than a builtin. |
| **Engine** | Interleaved period loop. Bounded — no dependency ordering among streams, which keeps the existing phase split intact. |
| **IR / results** | States need representing in the IR and probably reporting in `results.series`. Both are versioned contracts with gates (`check-ir-schema`, `check-results-schema`), so the change is visible and tested. |
| **Performance** | O(n), against O(n²) for the fold. The existing `env.series = series.clone()` per accrual is already the hot spot and should move to a borrowed view as part of this. |
| **Monte Carlo** | States recompute per trial. A recurrence propagates a per-period error forward rather than keeping it local — worth a note in the docs. |
| **`active when`** | A stream inactive at `t` yields 0. A state HOLDS instead — see §8, which corrects what this row originally said. |

## 7. Verification

- **Identities first**, in `tools/analytic-checks.py`: a state seeded at `b` with
  `next = prev * (1+r)` must equal `b * pow(1+r, t)` for constant `r` — the new
  path and the closed form agreeing is the check. And a varying-rate state must
  equal the running product computed independently.
- **The seed is mandatory**: a `state` without `init` must be a diagnostic, with
  a probe model proving it fires. Every check added this session that was not
  probed turned out not to fire.
- **`git diff gold/` must be empty** until a model uses a state.
- **The two rechecks are the acceptance test.** `benchmarks/opco/damodaran_fcff`
  asserts years 1–5 and blanks 6–10; with this it must assert all ten and the
  −2.4% drift must go to zero. `benchmarks/cre/hud_home_multifamily`'s rounded
  escalation must close its 12.26 residual. Two independent published sources
  confirming one mechanism is the strongest evidence available.

---

## 8. Correction — a state does have a clock

**This document originally said "a state has no schedule."** That was wrong, and
wrong in a way worth recording rather than quietly editing out, because the
error was a conflation rather than an oversight.

A stream's `schedule` carries two independent things:

| axis | question it answers |
|---|---|
| **cadence** | how often does this advance? |
| **activity** | does this contribute anything in period `t`? |

§6 reasoned about the *activity* axis — "a stream inactive at `t` yields 0, and
a state should not" — concluded correctly that `active when` does not belong on
a state, and then dropped **both** axes. So every state stepped once per *model*
period.

That is not a stylistic gap. `{{time.elapsed_periods}}` counts a lowering rule's
**payment** periods, and the credit pack lets a contract set its own
`payment_frequency`. On a daily calendar with monthly payments — a shipped,
tested configuration — a hazard recurrence would compound 365 times a year
instead of 12: `k^1096` rather than `k^36` at month 36. Not a rounding
regression; a total loss of the number.

**The fix is the cadence axis, and only that.** A state takes the same
`schedule` clause a stream does, steps on its accrual periods, and **holds**
between ticks and outside its window:

```cfdl
state pool_survival {
  schedule every quarter from 2026-01 to 2031-01
  init 1.0
  next prev * (1 - hazard)
}
```

Holding rather than zeroing is precisely the distinction §6 was reaching for,
now placed on the axis where it belongs. `active when` stays out.

Two off-by-ones, both found by building the fixture rather than by reasoning:

1. **Step on accruals, not settlements.** A quarterly schedule accrues at
   periods 0, 3, 6 and settles at 2, 5, 8. A stream's amount is evaluated at the
   accrual, which is also where the payment index is counted. Ticking on
   settlements puts the recurrence a whole interval from the index that reads it.
2. **`init` belongs to the first tick, not to model period 0.** Otherwise the
   first payment reads `F(1)` where it should read `F(0)` — an off-by-one
   against every published amortisation schedule.

`fixtures/valid/state_cadence` pins both. It uses `next prev * 2` so the step
count is readable straight off the value: a wrong cadence is unmissable rather
than subtle.

The general lesson is the one this design already relies on elsewhere: when a
construct borrows a concept from another, borrow the *mechanism* too. A state's
cadence goes through the same `apply_schedule_indices` a stream's does, so the
two cannot drift.

## 9. Correction — not every fixed point needs iteration

§5 says of genuine fixed points that they *"need iteration to convergence and
cannot be by construction."* Two benchmarks falsified that as a general claim,
and one of them is the very family §5 names as its example.

**A linear loop collects.** `benchmarks/opco/lbo_circular_interest` reproduces a
sponsor LBO's debt schedule, where interest accrues on the AVERAGE debt balance
— so interest depends on the closing balance, the closing balance depends on the
cash that swept it down, and that cash is net of interest. The reference model
solves it by switching on Excel's iterative calculation; it ships a literal
`CIRC` toggle.

But every step is affine in the closing balance, so collecting terms solves it in
one substitution. With `k = (1 - tax) * rate / 2`:

    B(t) = [ B(t-1) * (1 + k) - (1 - tax) * (EBIT(t) - K(t)) - C(t) ] / (1 - k)

That is an ordinary `next` clause. It agrees with the iterated answer to
**2.8e-14** across every balance and every interest figure, and it compiled and
reproduced the schedule on the first run.

**An ordered discrete loop enumerates.** `benchmarks/opco/lbo_option_pool_exit`
is the harder shape: a management option tranche exercises if it is in the
money, but exercising adds both its strike proceeds and its shares to the pool,
which MOVES the value per share. The unknown is a SET, not a number, so no
algebra collects it.

It is still closed, because the strikes are ORDERED: if a $20.00 option is in the
money then so is every cheaper one, so any exercising set is a PREFIX. That
reduces 2^7 subsets to 8 candidates, exactly one of which is self-consistent —
a finite ordered test, verified unique at all six published exit multiples.

**The claim as it should read.** Iteration is needed for fixed points that are
neither linear nor ordered. A loop that is affine in its unknown collects; a
discrete loop over an ordered set enumerates. Both are expressible today, and
§5's own example — Damodaran's LBO re-pricing at post-LBO leverage — belongs in
the first category rather than the third.

Worth checking rather than assuming for the remaining examples: mining royalties
levied on earnings *before* tax are circular with cost and depreciation, and may
well be linear too.

**The honest limit**, from `lbo_circular_interest/NOTES.md`: the collection holds
because no constraint binds. That deal never draws its revolver and never fully
repays its term loan, so the recursion stays linear throughout. A `min` or `max`
in the loop makes it piecewise — solvable branch by branch, but not by the one
substitution above, and CFDL cannot express the branch selection today.
