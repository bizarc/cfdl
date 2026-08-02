# State and recurrence — design

Status: **proposal, not implemented.** Fills the gap noted in
`docs/13_feature_backlog.md` 5.1/5.2 and referenced four times in
`packs/opco/lowering/rules.toml` as "H3-style state" with no design written down.

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

Streams read it as an ordinary environment path:

```cfdl
stream firm.revenue on entity legal.firm inflow currency USD {
  schedule every year from 2026-01 to 2035-01
  amount = 21765.4 * state.revenue_index
}
```

### 3.1 The restriction that makes it safe

Inside `next`, the environment contains:

- `prev` — this state's previous value
- `time.*`, `inputs.*`, curves — the ordinary expression environment
- other states' **previous** values
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

### 3.2 Evaluation

Interleaved, period by period:

```
for t in 0..n:
    for each state:  value[t] = (t == 0) ? init : next(prev = value[t-1], series ≤ t-1)
    for each stream: evaluate at t, with state values at t available
```

O(1) per state per period; O(n) overall. States are computed before streams
within a period, and read only completed data, so no ordering among states is
needed beyond declaration order — and even that is unnecessary, since a state
may only see *other* states' previous values.

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
| **`active when`** | A stream inactive at `t` yields 0. A *state* has no schedule and always updates; say so explicitly, because the alternative reading is defensible and the two differ. |

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
