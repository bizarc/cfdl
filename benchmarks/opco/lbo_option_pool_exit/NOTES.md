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

## Finding 1 — options cannot see model state, and fail silently

**This is the important finding, and it is a defect rather than a number.**

An option's `exercise when` is evaluated in the engine's discrete event/option
pre-pass. That pass builds its own environment, and the environment exposes
`inputs.` but **not** `state.`. So an option cannot test against anything the
model computes as state.

The failure mode is what makes it serious. Writing `state.value_per_share` in
an `exercise when` does **not** fail the build. The engine emits a warning and
**evaluates the condition to false**:

```
Stream 'mgmt_options_12_50' exercise when evaluation failed [EXPR_EVAL]:
unknown variable `state.value_per_share`; using false.
```

so every option silently declines to exercise and its entire value disappears
from the model. A valuation that should carry $12.9mm of intrinsic option value
simply reports zero, with the model still "running clean" to anyone not reading
warnings. This case found it because the reference publishes the answer;
without an external number it would have looked like a plausible result.

Three things worth separating:

1. **The asymmetry itself** — `inputs.` yes, `state.` no — is undocumented. The
   language spec says v0.1 "supports only deterministic exercise triggers",
   which reads as a restriction on *search*, not on which variables resolve.
2. **The severity of the failure.** A missing variable in a stream `amount` is a
   compile error (`E5010_TERM_UNKNOWN_INPUT` exists precisely for this). In an
   `exercise when` it degrades to a warning and a wrong answer. These should
   agree, and the strict one is right.
3. **The diagnostic calls an option a "Stream".** Cosmetic next to the above,
   but it sends a reader looking for a stream that does not exist.

The workaround here is to restate the resolved value as an `assume` for the
options to read. The two are tied by the **test** rather than by the model:
`expected.csv` asserts `state.value_per_share` against the published figure, so
if the derivation drifts from the constant the case fails.

## Finding 2 — a non-exercised option is absent, not zero

The $22.50 and $25.00 tranches are out of the money at 8.0x, and the engine
publishes **no series at all** for them — not a zero series. So they cannot be
asserted as zero; their non-exercise is established by the value per share
(20.8771, below both strikes) and by the proceeds and share totals, which only
reconcile if exactly five tranches exercised.

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
