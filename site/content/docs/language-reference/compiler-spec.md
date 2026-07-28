---
id: compiler-spec
title: "Compiler Spec (v0.1)"
slug: "/docs/language-reference/compiler-spec"
source: docs/04_compiler_spec.md
---

This page is a usability-focused digest of the compiler spec for model authors and SDK integrators.

## Use this page for

- Understanding compiler stage responsibilities
- Knowing deterministic guarantees and ordering rules
- Finding the validation/diagnostic sections quickly

## Compiler flow at a glance

1. Load and normalize files
2. Lex and parse with spans
3. Resolve imports and symbols
4. Validate structure/types/schedules
5. Lower normalized AST into deterministic IR
6. Emit canonical IR JSON

## Determinism guarantees

- Same sources + pack version + compiler version must emit deterministic IR.
- Arrays in IR are canonically ordered (entities/contracts/streams/etc).
- Deterministic IDs are derived from stable keys.

## Sections to read in the full spec

- AST model and spans
- Validation rules and required statements
- Lowering and normalization rules
- IR assembly and canonical ordering
- Diagnostics contract and error code guide

## Full compiler spec

- [Open full compiler spec source](https://github.com/bizarc/cfdl/blob/main/docs/04_compiler_spec.md)

If you need strict implementation-level details, use the full source spec above as authoritative.
