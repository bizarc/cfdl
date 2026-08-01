# CFDL v0.1 — Implementation Status

Status: informative. Every row is verified by compiling a probe model against
the current build rather than read off the grammar — the rows that were wrong
in earlier revisions were wrong because they described intent.

The standard this page holds the language to: **nothing is accepted and
silently discarded.** A construct either works end to end, or it is rejected
with a diagnostic that says so. Several rows below moved from "accepted, does
nothing" to "rejected" for exactly that reason; a construct that quietly does
nothing produces wrong numbers with no signal, which is worse than one that
refuses to compile.

Legend: ✅ works end to end (parse → IR → engine) · 🟡 partial, see notes ·
❌ rejected with a diagnostic.

| Construct | Status | Notes |
|---|---|---|
| `version`, `model "<name>"` | ✅ | |
| `model ... currency <code>` | ✅ | reporting currency for every metric; defaults to `USD`. Streams and pack rules must agree (`E2107`) |
| `use pack`, `import ... as` | ✅ | |
| `time calendar <freq> from <date> for <n>` | ✅ | daily/monthly/quarterly/annual |
| `time ... project <n>` (valuation projection tail) | ✅ | computed for series lookups; excluded from cash/NPV |
| `series_sum` / `series_avg` (cross-stream references) | ✅ | two-phase evaluation; phase-2 streams cannot reference each other |
| `phase <name> from .. to ..` | ✅ | named in IR; gates option exercise |
| `entity <ns> <name>` | ✅ | basic form |
| `entity ... : <type> { attrs }` (typed block) | ❌ | rejected; declare entities in the bare form |
| `assume <name> = <expr>` | ✅ | evaluated into `inputs.*` |
| `assume <name> ~ Normal/LogNormal/Uniform/Triangular(..., clip=[..])` | ✅ | per-assumption seeded Monte Carlo; central values in deterministic runs |
| `curve <name> [step\|linear] { <date>: <num>, ... }` | ✅ | date-indexed value curves; `curve_value(name, date)` lookup (step = flat-forward, linear = calendar-day interpolation) |
| `contract` (subject, `term A..B`, `terms { k = v }`) | ✅ | terms feed `{{contract.*}}` lowering templates |
| `contract effects { ... }` | 🟡 | the block is **required** (`E2002`) but its contents are block-skipped: a stream declared inside is never emitted. Declare streams at top level or via a pack |
| `contract parties` / `tags` blocks | 🟡 | accepted and discarded; absent from the IR |
| `stream` (owner, direction, currency, amount, active when) | ✅ | bare native expressions. An unrecognised item in the body is rejected (`E0004`); it used to be bumped and discarded, so `payment net 60 days` on its own line — and every typo'd key — compiled clean and did nothing |
| `schedule on <date>` | ✅ | settles on its own date, undiscounted for the period it lands in |
| `schedule every <interval> from .. to ..` | ✅ | `day`, `week`, `month`, `quarter`, `year`. Distinct from the calendar cadence: a stream may pay quarterly on a monthly grid |
| `schedule every <interval> due` | ✅ | annuity due — payment at the interval's start. Without it the schedule is an ordinary annuity, paid at the end. See `12_payment_timing.md` |
| `schedule on phase_enter(..)` / `every .. phase_start()/phase_end()` | ✅ | |
| `schedule ... on day <n>` | ✅ | places the payment within its interval, clamped to the month's length |
| `contract ... payment net <n> [days\|months]` | ✅ | applies to every stream the contract lowers; billing at period close, due date rolled |
| `schedule ... net <n> [days\|months]` | ✅ | overrides the contract's terms for one stream; rejected on `schedule on <date>` |
| `schedule ... on eom` | ✅ | last day of the month; 28 February, not 30 |
| `schedule ... on <weekday list>` | ❌ | rejected; removed from the grammar |
| `schedule ... convention <roll>` | ✅ | following/modified_following/preceding/modified_preceding |
| `schedule ... calendar "<name>"` | ✅ | weekend, us, target, uk (computed holidays) |
| `schedule ... except [dates]` / `also [dates]` | ✅ | roll-adjusted point dates |
| `schedule ... stub <policy>` | ❌ | rejected with a diagnostic and removed from the grammar. It was previously accepted and discarded, so a model could ask for a stub period and silently get a full one |
| `event <name> when <expr> { actions }` | ✅ | latch semantics: fires once, declaration order |
| — `set entity <ref>.<field> = <expr>` | ✅ | state visible as `entity.<field>` / `entity.state.*` (null before set) |
| — `activate/deactivate stream` | ✅ | persists forward |
| — `activate/deactivate contract` | 🟡 | parsed + lowered; engine warns-and-skips (no contract runtime yet) |
| — `exercise option` | ✅ | forces exercise at the firing period |
| `option <name> type <t> [exercisable in <phase>] { exercise when / payoff }` | ✅ | rule-based exercise only (optimal-exercise out of scope for v0.1) |
| `run deterministic` / `run monte_carlo trials N seed N` | ✅ | honoured by the engine; an explicit run config still wins |
| Literals: `money_lit` (`42000 USD`), `list`, `map_inline` | ❌ | rejected |
| `terms { k = <literal> }` | ✅ | one literal per term; trailing tokens are rejected (`E0004`) rather than discarded |
| `terms { k = inputs.<name> }` | ✅ | defers to a declared input, so scenarios and Monte Carlo drive it. Unknown input → `E5010` |
| Expressions (`cfdl-calc` dialect) | ✅ | see docs/03; decimal-first, excel_compat mode |

## Tooling status

- `cfdl parse <root>` dumps the typed AST as JSON (spans included).
- Parser robustness: deterministic fuzz tests (random soup, mutation, and
  truncation sweeps) assert lex+parse never panic; run in normal `cargo test`.
- `cfdl-validate` has per-code unit tests derived from the invalid fixtures.
