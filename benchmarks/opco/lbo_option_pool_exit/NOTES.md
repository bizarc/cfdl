# LBO option pool — the discrete fixed point, and a silent option failure

`lbo_circular_interest` showed that an LBO's debt schedule circularity is
**linear**, so it collects into a closed form. Its notes named the case it could
not reach: one where a constraint binds and the recursion goes piecewise.

This is that case, on the same deal, at the exit.

## The circularity

A management option tranche is exercised if it is **in the money** — if the
exit consideration per share exceeds its strike. But exercising a tranche adds
*both* its strike proceeds *and* its shares to the pool, which **moves the
value per share**. So:

```
which options exercise  ->  value per share  ->  which options exercise
```

No algebra collects this, because **the unknown is a set, not a number**. The
linear substitution that solved the debt schedule has nothing to grab.

## Why it is still closed

**The strikes are ordered.** If a $20.00 option is in the money then so is every
cheaper one, so any exercising set must be a **prefix** of the tranches sorted
by strike. That collapses 2^7 = 128 candidate subsets to **8 candidate
prefixes**, and exactly one of them is self-consistent:

```
V(j) = (exit equity + cumulative strike proceeds through j)
       / (preferred shares + rollover shares + cumulative option shares)

take the largest j whose own strike is below its own V(j)
```

Written descending, that is a plain `if` chain — which is what
`state.value_per_share` is in `model.cfdl`. No iteration, no solver.

Verified against **all six** published exit multiples; the consistent prefix is
unique at every one:

| exit multiple | tranches exercised | value/share | reference |
|---|---:|---:|---:|
| 7.0x | 3 | 16.7670 | 16.7670 |
| 7.5x | 4 | 18.8397 | 18.8397 |
| 8.0x | 5 | 20.8771 | 20.8771 |
| 8.5x | 6 | 22.8697 | 22.8697 |
| 9.0x | 6 | 24.7967 | 24.7967 |
| 9.5x | 7 | 26.6568 | 26.6568 |

Taken with the debt schedule case, the combined claim is sharper than either
alone: **neither of the two circularities a sponsor LBO is famous for actually
requires a solver.** One is linear and collects; the other is discrete but
ordered, and enumerates.

## Finding 1 — options could not see model state, and failed silently

**This case found a defect, and the fix is now part of the case.**

An option's `exercise when` was evaluated in the engine's discrete event/option
pre-pass. That pass built its own environment, and the environment carried
`inputs.` but **not** `state.`. Writing `state.value_per_share` in an
`exercise when` did **not** fail the build — the engine warned and evaluated the
condition to **false**:

```
Stream 'mgmt_options_12_50' exercise when evaluation failed [EXPR_EVAL]:
unknown variable `state.value_per_share`; using false.
```

so every option silently declined to exercise and **$12.9mm of intrinsic value
disappeared** from a model that still ran clean to anyone not reading warnings.
This case only caught it because the reference publishes the answer.

**Fixed by computing states before events and options.** The reorder is sound
because the graph is a strict DAG: a state's `next` reads only `prev`, curves,
inputs and time — never a stream, an event or an option — so nothing an option
does can reach back into a state. The options now read
`state.value_per_share` directly, the constant that stood in for it is gone,
and **every figure below is unchanged**. That is the strongest evidence
available that the two agree, because only one of them derives the number.

Three things the defect separated out, all now closed:

1. The asymmetry (`inputs.` yes, `state.` no) was undocumented, and the spec's
   "deterministic exercise triggers" reads as a restriction on *search*, not on
   which variables resolve.
2. **The severity was wrong.** A missing variable in a stream `amount` is a
   compile error; in an `exercise when` it degraded to a warning and a wrong
   answer.
3. The diagnostic called an option a "Stream".

## Finding 2 — a non-exercised option was absent, not zero

The $22.50 and $25.00 tranches are out of the money at 8.0x, and the engine
published **no series at all** for them — not a zero series. A consumer could
not tell "did not exercise" from "does not exist", and the case could not
assert a non-exercise, which is half of what an option model has to prove.

Also fixed: every declared option now publishes a series, zero where it did not
exercise. `expected.csv` asserts both tranches at 0, which is the regression
test.

Both are deliberately kept in the model. An option model in which every option
fires is not tested, and the boundary at $20.00 (in, by $0.88) against $22.50
(out, by $1.62) is where an off-by-one in the prefix walk would show.

## The result

| line | CFDL | reference |
|---|---:|---:|
| value per share | 20.877119 | 20.877119 |
| exit equity value | 575.615845 | 575.615845 |
| option strike proceeds | 44.500000 | 44.500 |
| sponsor proceeds | 487.546139 | 487.546139 |
| management proceeds | 132.569706 | 132.569706 |

Option intrinsic value by tranche: 4.188560, 3.438560, 2.938560, 1.688560,
0.657839 — $12.912mm in total across the five exercised tranches.

A convention worth recording: management proceeds are published **gross** of
the strikes paid in, because those strike proceeds are already inside total
cash to shareholders. Netting them would double-count. Sponsor 487.546 +
management 132.570 = 620.116, the published total.

The preferred is a third mechanic the case pins for free: $158.9375mm accruing
8% for five years, converting one-for-one at $10.00 — 158.9375 x 1.08^5 =
233.53133, giving 23.353133m shares. The conversion is itself an option (take
the preference or convert), in the money throughout here.

## What is out of scope

- **MoIC and IRR.** Published as 3.0675x and 25.13% at this multiple, and they
  reconcile on the proceeds above, but they are arithmetic on the result rather
  than cash flow.
- **The other five exit multiples**, and the three financing cases. Each needs a
  separate run; all six multiples are reconciled in the table above.
- **The debt schedule** that produces the $378.73mm of net debt at exit — that
  is `lbo_circular_interest`, which asserts it period by period.

## The source

The same publicly downloadable seven-step LBO teaching model as
`lbo_circular_interest` — free, no registration, "All Rights Reserved" with no
open licence. Neither vendored nor in CI: downloaded once outside this repo,
cached values read, only numbers carried across.

## The split became a waterfall, and what that changed

The sponsor's and management's shares were two independent streams. Both
reproduced their published figures, and **nothing checked that they added up to
the cash available** — the total appeared only in a comment. Two amounts that
were wrong together would have passed every gate.

They are now the two steps of an exit waterfall over a pot of exit equity plus
the strike proceeds, with management taking `remaining`. The published figures
are unchanged:

| | CFDL | published |
|---|---:|---:|
| sponsor | 487.546139 | 487.54613944 |
| management | 132.569706 | 132.56970572 |

The constraint now binds. Perturbing the sponsor's share count from 23.3531 to
23.4 moves its figure to 488.524585 **and management's to 131.591260** — so the
published management figure fails, where before only the sponsor's would have.
An error can no longer hide in a line nobody cross-checks.

**What the waterfall did not do**, and the design note claiming it would was
wrong: it does not retire the discrete fixed point. Resolving which option
tranches exercise is a search over ordered prefixes, and it stays exactly as it
was. The waterfall distributes cash once the value per share is known; it has
nothing to say about how that value is found. Two different mechanisms, and only
one of them is an ordered allocation.
