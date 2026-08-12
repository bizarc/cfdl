# One Lincoln Street, through the contract

## Why a second case rather than a conversion

The obvious move was to convert the existing case to the new contract. That
would have destroyed the more valuable of the two artefacts.

A case built from primitives proves the LANGUAGE can express the deal — no
domain vocabulary, nothing predefined, just a curve, a recurrence and three
streams. That is the stronger claim, and it is the one a reader evaluating CFDL
as a language should be able to check. A pack contract is an ergonomic layer on
top: it makes the next development model quick to write for a practitioner who
should not have to derive an equity-first waterfall from scratch.

So both ship, and the contract is asserted against the language rather than only
against the exhibit. A contract validated solely against its own source is the
pack marking its own homework; a contract that reproduces, to the cell, what the
primitives already produced is a convenience proven not to have changed the
answer.

If the two ever disagree, the contract is wrong.

## What the contract could not do, and what that forced

The design went through the constraints rather than around them:

- **A field's `next` cannot read a stream.** So `loan_balance next prev + draw`
  is not expressible, and the natural-looking design — a balance fed by the draw
  streams — is unavailable. The contract carries CUMULATIVE FUNDING instead and
  re-derives the opening balance as `max(0, cum - draw - commitment)`.
- **No stream may read another stream in the same period.** The equity draw and
  the loan draw are therefore both written as expressions over the same two
  quantities rather than one reading the other's residual.
- **A rule gets one schedule**, from the contract's own term.

The result is that all three rules share one field and one closed form, which is
also why the mid-quarter crossover needs no special case.

## The term that is a convention, and the one that is data

`draw_accrual_fraction` is a term because where in a period the money lands
genuinely varies between deals and moves every interest figure. The exhibit
states its own: funding "assumed to occur ratably throughout the quarter", so
0.5.

The draw SCHEDULE is not a term. It is per-deal data and it is a curve. A
contract carrying a curve's shape — a steepness parameter, a flat-or-S-curve
enum — states an implementation choice as though the parties had agreed it, and
the next deal disagrees. The contract names the curve; the model declares it.

## What this case does not cover

Capitalised interest. The exhibit pays it from the equity budget as a stated
line, so the contract models the paid form. A capitalising facility is a
different recurrence — affine in the closing balance, so it collects to
`next (prev * (1 + r*f) + D) / (1 - r*f)` rather than needing a solver — and is
a follow-on.

The JV waterfall on page 7 of the same case: an 11% annually compounded
cumulative preferred to MSGW and STRS pari passu, then a 34/51/15 residual split
with CPA subordinated to both preferred and return of capital. Fully specified,
and the case publishes no returns for it — it is the student's assignment. That
makes it an expressiveness fixture rather than a benchmark, and it is the
obvious next thing to build.
