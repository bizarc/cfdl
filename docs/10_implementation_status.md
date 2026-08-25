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
| `series_sum` / `series_avg` (cross-stream references) | ✅ | dependency-ordered waves to any depth; genuine cycles rejected with the named path |
| `phase <name> from .. to ..` | ✅ | named in IR; gates option exercise |
| `entity <ns> <name>` | ✅ | basic form |
| `entity ... : <type> { attrs }` (typed block) | ❌ | rejected; declare entities in the bare form |
| `assume <name> = <expr>` | ✅ | evaluated into `inputs.*` |
| `assume <name> ~ Normal/LogNormal/Uniform/Triangular(..., clip=[..])` | ✅ | per-assumption seeded Monte Carlo; central values in deterministic runs |
| `curve <name> [step\|linear] { <date>: <num>, ... }` | ✅ | date-indexed value curves; `curve_value(name, date)` lookup (step = flat-forward, linear = calendar-day interpolation) |
| entity field rules (`<field> init <expr> next <expr>` in an entity block) | ✅ | a named number per period, owned by the entity it describes and read as `<family>.<entity>.<field>`. `init` is mandatory; `next` sees `prev`, `prev.<family>.<entity>.<field>`, `time.*`, `inputs.*` and curves — never a same-period value, which is what keeps cycles impossible by construction. Read from stream amounts, guards, events and options. The retired top-level `state` declaration is rejected (`E1125`) with a message pointing here. See 03 §3.1 and 18_entity_owned_properties.md |
| — pack-side (`field_name` / `field_init` / `field_next` / `field_every` / `field_from` / `field_to` on a lowering rule) | ✅ | a rule may declare a field on the contract's subject entity; `field_name` must expand to a single identifier, so per-instance fields use `{{contract.suffix_ident}}` |
| `entity <ns> <name> : <Type> { <field> = <lit>, part of <ref>, state <name> }` | ✅ | typed against the active ontology, or the language's base vocabulary when no pack is active. `part of` is optional at every grain. `state` sets the lifecycle state the entity opens in |
| `contract` (subject, `term A..B`, `terms { k = v }`) | ✅ | terms feed `{{contract.*}}` lowering templates |
| `contract ... parties { <role> = <party> }` | ✅ | roles are declared by the contract TYPE, not the entity — the same party is lessor in one agreement and lender in another |
| `terms { k = v <unit> }` | ✅ | an optional unit is an ASSERTION about what the number means; the rule declares the truth and a mismatch is `E5024`. Units are never converted |
| `stream ... active in state <name>[, <name>]` | ✅ | lowers to a comparison on the lifecycle state, with the name checked against the owner's declared lifecycle — which a string comparison cannot be |
| `contract effects { ... }` | 🟡 | the block is **required** (`E2002`) but its contents are block-skipped: a stream declared inside is never emitted. Declare streams at top level or via a pack |
| `contract parties` / `tags` blocks | 🟡 | accepted and discarded; absent from the IR |
| `stream` (owner, direction, currency, amount, active when) | ✅ | bare native expressions. An unrecognized item in the body is rejected (`E0004`); it used to be bumped and discarded, so `payment net 60 days` on its own line — and every typo'd key — compiled clean and did nothing |
| `stream ... { category <path> }` | ✅ | what the stream IS, economically — a dotted path into the cash flow statement (`operating.deduction.abatement`). Aggregation reads this rather than pattern-matching the stream's name. Must name a category the active pack declares (`E5022`); the pack's own vocabulary must be rooted in `operating`, `investing` or `financing` |
| `schedule on <date>` | ✅ | settles on its own date, undiscounted for the period it lands in |
| `schedule every <interval> from .. to ..` | ✅ | `day`, `week`, `month`, `quarter`, `year`. Distinct from the calendar cadence: a stream may pay quarterly on a monthly grid |
| `schedule <on \| every> ... [start\|mid\|end]` | ✅ | one placement axis, three positions, at most one. `start` is an annuity due (interval's start), `mid` halfway, `end` its close. A recurrence defaults to `end` (ordinary annuity), a one-shot to `start`. See `12_payment_timing.md` |
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
| `event <name> when <expr> { actions }` | 🟡 | latch semantics: fires once, at the first period its condition holds, in declaration order — there is no repeating or level-triggered form. The guard reads entity fields and lifecycle state by qualified path (`entity.asset.tower.status`, `asset.tower.status`), as the period OPENED; it cannot read a stream |
| — `set entity <ref>.<field> = <expr>` | ✅ | visible as `entity.<field>` on the owner and `<family>.<entity>.<field>` from anywhere. An entity whose type declares a lifecycle opens in its declared initial state rather than null. Every write is published in `deterministic.transitions` |
| — `activate/deactivate stream` | ✅ | persists forward |
| — `activate/deactivate contract` | 🟡 | parsed + lowered; engine warns-and-skips (no contract runtime yet) |
| — `exercise option` | ✅ | forces the option's own ELECTION at the firing period. It does not bypass `exercisable in`: an option outside its window is not one anyone holds, and forcing one warns and declines |
| `option <name> [on entity <ref>] type <t> [exercisable in <phase>] { parties / exercise when / payoff }` | 🟡 | an option is a contract with an election, so it carries an owner and parties. `exercise when` reads entity fields by qualified path and its owner's entity state, as the period OPENED; it cannot read a stream. Rule-based exercise only — optimal exercise is out of scope for v0.1. Every declared option publishes a series, zero where it did not exercise |
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
