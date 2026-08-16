# Maintainer's notes

Not published. How this case was built and what it cost to establish.

## Reading the tables out of the filing

Two of the tables this case depends on sit on rotated landscape pages that
`pdftotext -layout` returns as empty strings, with `pdfimages -list` showing
only a logo. Every text-based check reports the page as blank. Render them:

```
pdftoppm -f 296 -l 296 -r 300 -png scco-20241231xex96d6.pdf out
```

then rotate -90 and slice into bands with the label column pasted alongside
each, so no cell is read by row position alone. The producer is "Microsoft:
Print To PDF", which is where the broken text layer comes from.

## The depletion arithmetic, and its off-by-one

A field's `next` computes period t from period t-1, so anything it reads must
be evaluated at t-1. Three expressions in the stock rules got this wrong at
first: the capacity switch, the grade policy lookup, and the tonnage draw. The
symptom was subtle -- the model ran with no warnings, drew 96% of contained
metal instead of 97.4%, and lost one year of full-capacity milling at the
Concentrator I step.

The grade policy needs a second curve on lagged dates, because a rule that must
know period t-1's grade cannot shift a date. The streams read the unlagged
curve; only the field reads the lagged one. A regex fix once changed both, and
the model then ran a period behind with no warning at all.

## Two implementations, and what they caught

The reference and the model disagreed twice, and both times the reference was
right about one thing and the model about another:

- The model's `assume rate_crushed_leach = 26.3` was rounded where the
  reference computed 1077/41. Every line differed in the sixth figure.
- The deterministic run of a stochastic assumption takes the distribution
  MEAN, not its mode. The reference used 0.83, the published life-of-mine
  strip ratio, and had to be moved to the triangular mean of 1.0733 to be
  comparable.

## The harness gained monte carlo support for this case

`tools/benchmark-runner.py` had no monte carlo handling and no case used it.
This case adds `expected_monte_carlo.json`, asserting the aggregates the engine
already publishes -- mean, p50, stdev, min, max -- with tolerances, mirroring
how `expected_metrics.json` treats the deterministic run.

Section 12.4 of the language spec makes the seed mandatory, so the draws are
reproducible and the tolerances can be as tight as any other metric. What is
being checked is this engine on this seed, not a statistical claim.

## Cost

500 trials takes about 15 seconds on a release build and about 70 on the debug
build the suite uses, against a suite that otherwise runs in 81 seconds. That
is a real cost and it buys the only distributional assertion in the suite.

## Not done

- The leach and waste phasing is smooth where the operator's is lumpy. The
  totals match; the shape does not. The pit sequence that produces the swing is
  not described anywhere in the report.
- The published sensitivity matrix is transcribed and unused. It is an
  after-tax NPV under the operator's own model, so it is a comparison for our
  own distribution rather than an input.
- The market price curves are unused: they cover ten and five years of a
  41-year life.
