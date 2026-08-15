---
id: specification
title: "Specification"
slug: "/docs/specification"
description: "The normative definition of CFDL: the grammar, the expression environment, the compiler's obligations, the diagnostic register, the pack interface, and the schemas."
generated: none
---

# Specification

The normative definition of CFDL: the grammar, the expression environment, the
compiler's obligations, the diagnostic register, the pack interface, and the two
JSON schemas the toolchain reads and writes.

These pages exist so that a second implementation could be written from them,
and so that anything consuming CFDL's output has a contract to hold it to. They
describe behavior that most people modeling a deal never need to think
about.

**If you are building a model, you want [Reference](/docs/reference) instead.**
It covers the same ground at the altitude of the work — what a thing does, when
you would reach for it, and how to get a particular result.

## What is here

| | |
|---|---|
| [Language specification](/docs/specification/language-spec) | Types, declarations, evaluation, and the rules a compiler must enforce |
| [Grammar](/docs/specification/grammar) | The EBNF |
| [Expression environment](/docs/specification/expression-environment) | Every binding and builtin available to an expression |
| [Compiler specification](/docs/specification/compiler-spec) | Determinism, canonical ordering, and stable identity |
| [Diagnostics](/docs/specification/diagnostics) | The complete code register, with severities |
| [Pack interface](/docs/specification/pack-interface) | How a domain pack extends the language |
| [IR schema](/docs/specification/ir-schema) | The compiled intermediate representation |
| [Results schema](/docs/specification/results-schema) | The run output document |

The two JSON schemas are also served directly, for validation in your own
tooling: [IR](/schemas/CFDL_v0_1_IR.schema.json) ·
[Results](/schemas/CFDL_v0_1_Results.schema.json) ·
[Grammar](/schemas/CFDL_v0_1_Grammar.ebnf).

## Stability

The schemas carry their own version field, and a run records the engine version
that produced it. Where a specification page and the engine disagree, the engine
is the defect — these documents are normative.
