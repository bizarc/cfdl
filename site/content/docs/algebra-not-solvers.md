---
id: algebra-not-solvers
title: Algebra instead of solvers
description: "Circular references in a spreadsheet are usually circular wiring, not circular deals. A method for telling the difference, and four worked cases across four domains that needed no solver."
generated: none
---

# Algebra instead of solvers

A financial model built in a spreadsheet reaches for iteration often enough that
the habit is invisible. Debt sizing gets Goal Seek. Capitalized construction
interest gets a `CIRC` switch and Excel's iterative calculation. A subsidy sized
to hold a cover ratio gets a circular reference and a tolerance.

CFDL has no solver, and so far it has not needed one. Every sizing and every
circularity the validation programme has met has fallen to algebra — four
worked cases across four domains, each reconciled against the external model
that solved the same problem by iterating.

That is not a claim that no financial problem needs iteration. It is a narrower
and more useful claim: **the circularity is usually in how the spreadsheet is
wired, not in the deal being modeled**, and the two are easy to confuse when the
wiring is what you have in front of you.

This page gives a method for telling them apart, then works the four cases with
their derivations and verification figures.

## Why the confusion is so common

A spreadsheet cell holds one value and refers to other cells. When a quantity
depends on itself, the only tool the grid offers is to compute it repeatedly
until it stops changing. That is a general method: it works whether the
dependence is linear, non-linear, discontinuous or genuinely simultaneous.

Being general is exactly what hides the structure. Iteration does not care
*why* the loop closes, so the modeler never has to find out — and a loop that
would collapse to one line of algebra looks identical, in the grid, to one that
would not.

Three things tend to be true of the loops that appear in deal models:

- **They are linear in the unknown.** Interest is a rate times a balance.
  A reserve is a multiple of a payment. Very little in a debt schedule squares
  or divides one unknown by another.
- **They close over a single unknown.** The loop passes through five
  quantities, but four of them are determined once the fifth is.
- **They often are not loops at all.** A term that looks circular is settled in
  an earlier period — tax paid in arrears, a balance struck at the prior
  period's close — and the dependence points backward in time rather than
  around a ring.

Each of those has a move that answers it, and none of the moves is iteration.

## The method

Run this on the problem in front of you, in order. Stop at the first step that
applies.

### 1. Write the loop out, as equations

Not as cell references. Name the unknown, and write each step as an equation in
it. Most of the work is here: a loop that survives being written out as
equations is usually already halfway solved.

The loop in a leveraged buyout, for example:

```
interest(t)  ->  net income(t)  ->  free cash flow(t)  ->  closing balance(t)  ->  interest(t)
```

Five arrows, but only one unknown: the closing balance. Everything else is a
function of it.

### 2. Ask whether a lag breaks it

Does any term on the right refer to an earlier period? If the quantity that
seems to close the loop is settled before this period begins, there is no loop —
there is a sequence, and it evaluates in order.

This is worth checking first because it is the cheapest answer and the easiest
to miss. A spreadsheet shows `=D14` whether `D14` is this year's tax or last
year's, and a circular reference warning appears either way if the wiring is
sloppy.

### 3. Ask whether every step is affine in the unknown

Affine means the unknown appears only multiplied by constants and added to
constants — no products of two unknowns, no powers of it, no `min`, `max`,
absolute value or threshold applied to it.

If every step is affine, the whole loop is affine, and an affine equation in one
unknown has a closed form. Write the unknown on both sides, collect it, and
divide:

```
X = a + k*X        =>        X = a / (1 - k)
```

That single form solves three of the four cases below. The work is finding `a`
and `k`, which is bookkeeping rather than mathematics.

**A threshold does not always break this.** `max(0, ...)` around the unknown is
not affine — but if you can determine which side of the threshold you are on
before solving, the branch is a constant and the remaining expression is affine
again. Solve within the branch.

### 4. Ask whether the constraint is a definition in disguise

Some "solved" quantities are not solved at all. A loan sized to an LTV is
`value * ratio`. A loan sized to a DSCR on level-pay terms is the present value
of the `NOI / target` annuity. Both are stated in a document as a test to be
met, and both are a closed-form expression of the inputs. The iteration in the
spreadsheet is searching for a number that arithmetic already determines.

### 5. If none of the above, take the stated figure

Where a workbook's sizing is genuinely a solved artifact and the deal documents
state the answer anyway, take the stated figure as the input it is, and
**assert the resulting ratio against the source**. The model
then checks the sizing rather than reproducing the search.

That is not a defeat. A model that recomputes a number the credit agreement
already states has added a way to disagree with the agreement.

---

## Case one — capitalized construction interest

**Domain:** commercial real estate. **The spreadsheet's answer:** a circular
reference.

Interest accrues on a construction loan balance that is itself growing by the
interest. The classic Excel circularity.

```
B(t) = B(t-1) + D(t) + r * f * B(t)
```

where `D(t)` is the period's draw, `r` the rate and `f` the period fraction.
The balance appears on both sides, and it appears **linearly** — step 3
applies. Collect it:

```
B(t) - r*f*B(t) = B(t-1) + D(t)
B(t) * (1 - r*f) = B(t-1) + D(t)
B(t) = (B(t-1) + D(t)) / (1 - r*f)
```

That is a recurrence, not a fixed point: each period's balance is a closed-form
function of the previous period's, which the language evaluates in order. The
`cre.construction_loan` contract carries exactly this form.

## Case two — a leveraged buyout on average-balance interest

**Domain:** operating companies. **The spreadsheet's answer:** an explicit
`CIRC` switch that turns on iterative calculation.

Interest is charged on the average of the opening and closing balance, which is
the standard convention. The closing balance depends on how much cash swept the
debt down, and that cash is net of interest.

```
B(t)        = B(t-1) - LFCF(t)
LFCF(t)     = (1 - tax) * (EBIT(t) - interest(t)) + C(t)
interest(t) = rate(t) * (B(t-1) + B(t)) / 2 + K(t)
```

Three equations, one unknown, every one affine in `B(t)`. Substituting and
collecting, with `k = (1 - tax) * rate(t) / 2`:

```
B(t) = [ B(t-1) * (1 + k) - (1 - tax) * (EBIT(t) - K(t)) - C(t) ] / (1 - k)
```

`K(t)` is the interest that does *not* depend on the swept balance — the
commitment fee on the undrawn revolver, fixed-rate senior notes, the PIK
subordinated coupon, amortized financing fees, less interest earned on the
minimum cash balance. All of it is known before `B(t)` is, which is what makes
collecting `B(t)` legitimate. `C(t)` is the non-EBIT cash flow.

**Verification.** Against the reference workbook's own unrounded cached values,
across all sixteen balance and interest figures:

| year | term loan balance | reference | term loan interest | reference |
|---|---:|---:|---:|---:|
| 2017 | 238.517440443 | 238.517440443 | 8.986555208 | 8.986555208 |
| 2018 | 199.519287769 | 199.519287769 | 8.979752928 | 8.979752928 |
| 2019 | 156.762561123 | 156.762561123 | 8.016341600 | 8.016341600 |
| 2020 | 120.484780576 | 120.484780576 | 7.139119049 | 7.139119049 |

Worst disagreement: **2.8e-14**, which is machine epsilon. The closed form and
the iteration agree to the limit of double precision — as they must, because
they are answers to the same linear equation.

See the worked case: [LBO with circular interest](/docs/examples/opco-lbo-circular-interest).

## Case three — a subsidy sized to a cover ratio

**Domain:** infrastructure. **The spreadsheet's answer:** a circular reference.

A contracting authority tops a toll road up each year with whatever subsidy
holds the annual debt service cover ratio at exactly 1.30x. Read naively this is
a fixed point: the subsidy sits inside cash available for debt service, cash
available for debt service is net of corporate tax, and tax is levied on a
profit that includes the subsidy.

**It is not circular, because tax is paid one year in arrears.** Step 2, not
step 3. The tax settled in year `n` is a rate on year `n-1`'s profit, and that
profit is finished before year `n` is evaluated:

```
subsidy(t)  = max(0, 1.30 * debt_service(t) - (revenue(t) - opex(t) - tax_paid(t)))
tax_paid(t) = rate * min(pbt(t-1), cumulative_pbt(t-1))
```

Every term on the right is settled. The subsidy falls out arithmetically, once
per period.

Note also the `max(0, ...)`: a threshold, which is not affine. It does not
matter here, because nothing downstream of the subsidy feeds back into it — the
branch is evaluated, not solved through.

See the worked case: [PPIAF toll highway](/docs/examples/bespoke-ppiaf-toll-highway).

## Case four — a debt service reserve, sized against the debt it reserves

**Domain:** energy. **The spreadsheet's answer:** the reference model solves it
internally.

A project funds a debt service reserve at close, sized at six months of debt
service. The reserve is capitalized into the installed cost; the debt is sized
as a percentage of installed cost; the debt service follows from the debt. So
the reserve helps determine its own size:

```
reserve -> installed cost -> debt size -> debt service -> reserve
```

Four arrows, one unknown, all affine. With `C0` the cost before financing, `p`
the debt fraction, `m` the reserve expressed in years, and `a` the annuity
factor `r / (1 - (1+r)^-n)`:

```
T = m * a * D
D = p * (C0 + T)
```

Substituting the second into the first and collecting `T`:

```
T = m*a*p*(C0 + T)
T - m*a*p*T = m*a*p*C0
T * (1 - m*a*p) = m*a*p*C0

T = k*C0 / (1 - k),      k = m*a*p
```

The same `X = a / (1 - k)` shape as case one and case two. Here `k` is the
share of the basis the reserve consumes — 2.77% on the figures below — and
`1/(1-k)` is the gross-up.

**Verification.** Against the national laboratory's open-source
project-finance model, at 100 MW, a 60% debt fraction, 6% over 18 years and a
six-month reserve:

| quantity | closed form | reference |
|---|---:|---:|
| annuity factor `a` | 0.092356540553 | — |
| `k = m*a*p` | 0.027706962166 | — |
| reserve target `T` | 2,849,651.400115 | 2,849,651.400115 |
| installed cost `C0 + T` | 102,849,651.400115 | 102,849,651.4001 |
| debt size `p*(C0+T)` | 61,709,790.840069 | 61,709,790.8401 |
| annual debt service | 5,699,302.800230 | 5,699,302.800230 |

Worst disagreement on the reserve target: **7e-9**.

## When the method does not apply

Where it stops, so the method is usable rather than merely encouraging.

**A genuine simultaneous system.** Two or more unknowns that determine each
other, where no lag and no substitution separates them. Nothing in the
validation programme has produced one yet, which is evidence about the deals
met so far and not a theorem.

**A threshold the unknown must be solved through.** `max(0, ...)` is fine when
the branch can be determined first. It is not fine when which branch you are in
depends on the answer — then the problem is piecewise, and the treatment is to
solve each branch in closed form and state which one governs.

**A search over a policy, not a quantity.** Optimal exercise of an option asks
what a decision-maker would choose, which needs a model of the decision-maker.
That is outside a language whose promise is that every behavior traces to a
stated claim, and no amount of algebra converts it.

**Non-linear dependence.** A rate that depends on a ratio that depends on the
rate — a pricing grid, a margin ratchet — is not affine, and collecting terms
does not close it. A grid with stated breakpoints is piecewise affine, which the
threshold treatment above covers; a smooth non-linear dependence is not.

## The pattern worth remembering

Three of the four cases collapse to the same expression:

```
X = a + k*X        =>        X = a / (1 - k)
```

and the fourth was not circular at all. The work in every case was writing the
loop as equations and noticing that the unknown appeared linearly. That is a
reading exercise, not a numerical one, and it is worth doing before reaching for
iteration — because the closed form is exact, evaluates once, and states the
relationship the deal actually has.
