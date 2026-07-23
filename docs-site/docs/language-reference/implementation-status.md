---
id: implementation-status
title: "Implementation Status"
slug: "/language-reference/implementation-status"
---

> This page is generated from `docs/10_implementation_status.md`.
> Source: https://github.com/bizarc/cfdl/blob/main/docs/10_implementation_status.md

Status: informative; updated with each engine increment.
The 1.0 launch gate requires every row below to be **Implemented** or removed
from the grammar — no silent no-ops at 1.0.

Legend: ✅ implemented end-to-end (parse → IR → engine) · 🟡 partial (see
notes) · ❌ declared in the EBNF but not parsed.

| Construct | Status | Notes |
|---|---|---|
| `version`, `model "<name>"` | ✅ | |
| `model ... currency <code>` (model-level currency clause) | ❌ | declared in the grammar; not parsed — IR model currency defaults to USD |
| `use pack`, `import ... as` | ✅ | |
| `time calendar <freq> from <date> for <n>` | ✅ | daily/monthly/quarterly/annual |
| `time ... project <n>` (valuation projection tail) | ✅ | computed for series lookups; excluded from cash/NPV |
| `series_sum` / `series_avg` (cross-stream references) | ✅ | two-phase evaluation; phase-2 streams cannot reference each other |
| `phase <name> from .. to ..` | ✅ | named in IR; gates option exercise |
| `entity <ns> <name>` | ✅ | basic form |
| `entity ... : <type> { attrs }` (typed block) | ❌ | typed entities + kv attrs not parsed |
| `assume <name> = <expr>` | ✅ | evaluated into `inputs.*` |
| `assume <name> ~ Normal/LogNormal/Uniform/Triangular(..., clip=[..])` | ✅ | per-assumption seeded Monte Carlo; central values in deterministic runs |
| `curve <name> [step\|linear] { <date>: <num>, ... }` | ✅ | date-indexed value curves; `curve_value(name, date)` lookup (step = flat-forward, linear = calendar-day interpolation) |
| `contract` (subject, `term A..B`, `terms { k = v }`) | ✅ | terms feed `{{contract.*}}` lowering templates |
| `contract` `effects` / `parties` / `tags` blocks | 🟡 | tolerated by the parser (block-skipped); not represented in IR |
| `stream` (owner, direction, currency, amount, active when) | ✅ | bare native expressions |
| `schedule on <date>` / `every <freq> from .. to ..` | ✅ | |
| `schedule on phase_enter(..)` / `every .. phase_start()/phase_end()` | ✅ | |
| `schedule ... on day <n>` | ✅ | day-of-month rule |
| `schedule ... on eom` / weekday lists | ❌ | not parsed |
| `schedule ... convention <roll>` | ✅ | following/modified_following/preceding/modified_preceding |
| `schedule ... calendar "<name>"` | ✅ | weekend, us, target, uk (computed holidays) |
| `schedule ... except [dates]` / `also [dates]` | ✅ | roll-adjusted point dates |
| `schedule ... stub <policy>` | ❌ | not parsed |
| `event <name> when <expr> { actions }` | ✅ | latch semantics: fires once, declaration order |
| — `set entity <ref>.<field> = <expr>` | ✅ | state visible as `entity.<field>` / `entity.state.*` (null before set) |
| — `activate/deactivate stream` | ✅ | persists forward |
| — `activate/deactivate contract` | 🟡 | parsed + lowered; engine warns-and-skips (no contract runtime yet) |
| — `exercise option` | ✅ | forces exercise at the firing period |
| `option <name> type <t> [exercisable in <phase>] { exercise when / payoff }` | ✅ | rule-based exercise only (optimal-exercise out of scope for v0.1) |
| `run deterministic` / `run monte_carlo trials N seed N` | ✅ | lowered to IR `runs`; run-config JSON still overrides at execution |
| Literals: `money_lit` (`42000 USD`), `list`, `map_inline` | ❌ | contract term values are single literals; expression lists/maps not in the dialect |
| Expressions (`cfdl-calc` dialect) | ✅ | see docs/03; decimal-first, excel_compat mode |

## Tooling status

- `cfdl parse <root>` dumps the typed AST as JSON (spans included).
- Parser robustness: deterministic fuzz tests (random soup, mutation, and
  truncation sweeps) assert lex+parse never panic; run in normal `cargo test`.
- `cfdl-validate` has per-code unit tests derived from the invalid fixtures.
