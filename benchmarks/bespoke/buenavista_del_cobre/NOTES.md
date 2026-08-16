# Maintainer's notes

Not published. How this case was built and what it cost to establish.

## The case was rebuilt, and why

The first version consumed Table 19.1 directly: payable metal was the published
revenue divided by the published price, and the cost lines were the published
cost rows. Every input was a function of an output, so agreement to 1e-5 was
guaranteed and proved nothing about the asset. Review caught it — the curve
`cu_payable` held 883.939394 for 2025, which is exactly 2917 / 3.30.

This version consumes only inputs. The distinction that made it work: sort every
input by WHERE THE DOCUMENT PUTS IT, which is visible without seeing any answer.
Section 18 is the cost-input section and section 19.1 states the economic
assumptions, so those are inputs. Section 12's cut-off parameters and section
14's historical operating results are published for other purposes, and using
them is a substitution to declare. Nothing published is a recovery for the cash
flow, so those four numbers are ours.

## Reading the tables out of the filing

Table 19.1 sits on report page 19-3, PDF page 296, a rotated landscape page.
`pdftotext -layout` returns it as an empty string and `pdfimages -list` shows
only a logo, so every text-based check says the page is blank. Render it:

```
pdftoppm -f 296 -l 296 -r 300 -png scco-20241231xex96d6.pdf out
```

then rotate -90 and slice into bands with the label column pasted alongside
each. The sibling report for La Caridad has the same defect on its page 336.
This cost a wrong conclusion once: the case was declared unusable on the
strength of the empty extraction.

## What the metallurgy work established, and what it did not

Several derivations were tried and are worth recording because they are sound
even though the case does not use all of them:

- **Mill recovery from a constant tailings grade.** A flotation circuit's tail
  is set by the circuit, not the feed, so recovery is `(g - t)/g`. The tail
  inferred from Table 14.1's three operating years is 0.0678, 0.0713 and 0.0718
  percent Cu — a spread of 5.8%, which is what constant looks like in a real
  plant. That gives 85.9% at 0.50% feed falling to 82.4% at 0.40%, which is the
  grade dependence section 12.2.4 asserts and never quantifies.
- **Leach recovery from the routing rule.** The stated rule rearranges to
  `0.65 x SI + 0.30 x oxide_fraction`, a function of the solubility index. The
  index is bounded below at 0.30 by the routing threshold and per zone by the
  Table 11.7 regressions.
- **Mining escalation from depletion.** Haul distance rises as the pit deepens,
  so unit cost scales with cumulative material moved, with the scale solved to
  satisfy the published life-of-mine average of $2.71/t. This reproduced the
  published mining line to +0.4% over 41 years and is the only curve in the
  investigation with a rule that holds across the whole life.

What none of it established is the leach number the operator actually used.
Section 11 states that no primary sulfide reaches a leach pad, so the feed is
mixed and secondary ore, whose chemistry implies 36% to 57%. The operator's own
revenue implies about 22%, below even the mixed-zone floor. There is one
equation and two unknowns — total copper revenue does not separate mill from
leach — and putting leach at its chemistry floor requires a mill recovery of
96.4%, which is impossible. So the model carries 26% as a declared assumption
and the case reports the contradiction rather than resolving it.

## Two hazards this case hit

**A guard on a status no event writes is false forever, silently.** While
splitting the curves into `inputs.cfdl`, a regex removed the two lifecycle
events along with them. The model compiled, ran with zero warnings, and simply
never paid the closure outlay. Only the harness caught it. This is the failure
mode recorded at `docs/13` section 7.36.

**A field rule cannot read a series.** The shelter first read EBITDA through
`series_sum` and evaluated to zero with 41 warnings. State sees only settled
things, which is what keeps recurrences free of cycles, so the rule restates
EBITDA from the curves. That is the only place in the model where a definition
appears twice.

## Tolerance

`period_tolerance` is 1e-5. `expected.csv` holds the reference implementation's
output, so CFDL and the reference compute the same claims over the same inputs
and any daylight is a defect rather than rounding. Curves in `inputs.cfdl` are
serialised to nine decimals; at three decimals the capital line failed by
1.2e-4.

## Not done

- The comparison against the operator's Table 19.1 is printed by
  `reference_gen.py` and reported in `CASE.md`, but nothing fails if it drifts.
  Asserting it would require a tolerance wide enough to be meaningless, and
  would turn a finding into a target.
- Table 19.2's 78 published sensitivity points are transcribed in
  `published_sensitivities.csv` and unused. They are after-tax NPVs under the
  operator's own model, so they are a comparison for our Figure 19-1 rather
  than an input.
- The market price curves are transcribed in the source notes and unused, for
  the reason `CASE.md` gives: they cover 10 and 5 years of a 41-year life.
