---
id: grammar
title: "Grammar (EBNF)"
slug: "/docs/specification/grammar"
description: "The complete EBNF grammar for CFDL v0.1."
source: docs/02_grammar.md
generated: full
layer: specification
---

> This grammar is intentionally pragmatic for v0.1: it captures the surface syntax needed by the Core spec. It is suitable as the basis for a hand-written parser or parser-generator input after minor adaptation (token rules, whitespace/comment handling).

---

## 1) Lexical tokens (informative)

Implementations MUST support:
- **Whitespace**: spaces, tabs, newlines (ignored except as separators)
- **Line comments**: `// ...` to end of line
- **Block comments**: `/* ... */`

Recommended tokens:
- `IDENT`      → `[A-Za-z_][A-Za-z0-9_]*`
- `QNAME`      → `IDENT ('.' IDENT)*`
- `STRING`     → `"..."` with escapes
- `INT`        → digits with optional `_` separators
- `DECIMAL`    → digits `.` digits with optional `_`
- `DATE`       → `YYYY-MM` or `YYYY-MM-DD`

---

## 2) Grammar

The formal EBNF grammar is maintained as a standalone file for use with grammar tooling (railroad diagram generators, parser generators, etc.):

📄 **[Download the grammar (EBNF)](/schemas/CFDL_v0_1_Grammar.ebnf)**

---

## 3) Notes & implementation guidance (informative)

1. **Line breaks** are not significant; blocks and keywords provide structure.
2. `money_lit` is syntactic sugar; implementations should normalize to `Money` with `currency`.
3. `DATE` accepts `YYYY-MM` and `YYYY-MM-DD`; normalize `YYYY-MM` to `YYYY-MM-01` during parsing.
4. `schedule_opts` are order-independent; parsers should accept any order.
5. `activate contract` / `deactivate contract` were removed: a contract is a collection of streams, and one switch cannot express forbearance or an early termination. Gate the streams a contract produced by name, or with `active in state`. The retired spelling is `E0006_RETIRED_SYNTAX`.
