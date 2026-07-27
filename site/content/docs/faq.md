---
id: faq
title: FAQ
slug: /docs/faq
---

# FAQ

## Is CFDL open source?

No — CFDL is **source available** under the Business Source License 1.1.
You can read, modify, and use it in production; offering CFDL itself to
third parties as a hosted or embedded commercial product requires a
commercial license. Each released version converts to Apache 2.0 four years
after release. Details: [Licensing](/docs/licensing).

## How is this different from spreadsheets and appraisal-grade DCF tools?

Three ways. **Declarative and deterministic**: a CFDL model is a text file
that always produces the same IR and results — diffable, hashable,
CI-testable. **Natively stochastic**: swap a constant for a distribution
and the same model yields percentile bands around every metric, with seeded,
reproducible draws ([Stochastic Modeling](/docs/stochastic-modeling)). **Parity
is proven, not claimed**: every domain pack is gated by
[benchmark suites](/docs/benchmarks) diffed against independent Excel-grade
references — schedule math held decimal-exact.

## Will my results match Excel?

Schedule math uses decimal arithmetic with documented rounding (Excel's
half-away-from-zero `round()`), and financial builtins are unit-tested
against Excel-computed values. An `excel_compat` mode evaluates in IEEE-754
float64 to reproduce Excel's artifacts exactly where that matters.

## Can I contribute?

External pull requests are not accepted — CFDL is maintained by a small
internal team. Bug reports are welcome via
[GitHub issues](https://github.com/bizarc/cfdl/issues).

## How stable is the language?

Pre-1.0: the language/IR spec is v0.1 and interfaces may change until 1.0
freezes the IR and Results schemas (additive-only after that). The
[implementation status](/docs/language-reference/implementation-status) page
tracks exactly what is implemented.

## Which surface should I use?

Zero-install experiments → [Playground](/playground). Files/git/CI →
[CLI](/docs/install/cli). Notebooks and pandas → [Python SDK](/docs/install/python).
Service integration → [API server](/docs/install/api-server). All embed the same
engine and produce byte-identical results.
