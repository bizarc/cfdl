# Changelog

All notable changes to this project will be documented in this file.

This project follows Semantic Versioning: https://semver.org/

---

## [Unreleased]

### Added: master types, containers, and the relation vocabulary (§7.88, §7.89, §7.92)

The ontology promised refinement and did not record it: "a pack may refine
`Asset.Real` into `CRE.Asset.RealProperty`" was a naming convention the system
could not read, so no metric or validation could say "all debt" — each named
concrete pack types and broke when a pack added one.

Three changes, one type system. A pack type states what it specializes
(`refines`), checked at load — exists in pack or language base, same family,
same class, acyclic — and `is_a` walks the recorded chain. The language base
ships eleven abstract contract masters (`Contract.Debt`, `Contract.Lease`,
`Contract.Purchase`, `Contract.Sale`, `Contract.Offtake`, `Contract.Service`,
`Contract.Tax`, `Contract.Option`, `Contract.Construction`,
`Contract.Derivative`, `Contract.Insurance`); a master binds no lowering rule
and cannot be instantiated. And the family roster is restored to its own
comment plus one: `entity` declares `asset`, `party` or `container` (a fund, a
portfolio, an SPV, a transaction — groupings that scope cash without producing
it), while relations range over the five node families — the two above plus
`contract` and `reference` — which is what lets `secured_by`,
`guarantees` and `is_counterparty_to` join the base vocabulary beside the
widened `part_of` and `owns`.

All four packs declare their refinements (33 entity, 33 contract). Fixture
`valid/container_entity` pins the container rollup;
`invalid/entity_unknown_type`'s gold moves because E1311's known-types hint
now lists the Container base types. Specs: `docs/01` §7,
`docs/07_pack_interface.md` §6.1, `docs/13` §7.88/§7.89/§7.92.


### Added: a participant's realized return (§7.72)

The model computed `model.irr` on the deal's net cash, and a waterfall
attributed each step's payment to a payee — and there the trail stopped. To
measure what one party earned, an analyst hand-assembled the payee's cash,
capital in and distributions out, and ran the arithmetic outside the language
against results the language already held.

```cfdl
account lp_capital {
  owner party.lp
  from if(time.t == 0, -1000.0, 0.0)     // the capital call
}
...
metric lp_irr  = irr(party.lp)           // 0.218623
metric lp_moic = moic(party.lp)          // 1.6
```

**Folded over the party's own ACCOUNT, never over payee streams.** An account's
journal already separates the directions — a contribution is a negative
`inflow`, a receipt is an `allocate_in` — so the sign change an IRR needs is
RECORDED rather than inferred from stream names, which is the attribution trap
§7.43 records.

**The party is a reference, not text.** A party is an entity, named the way the
language names entities everywhere else, and the reference is what the compiler
can act on:

| written | before the run |
|---|---|
| `irr(party.lpp)` | `E1301` — not a declared entity |
| `irr(asset.deal)` | `E1356` — not a party |
| `irr(party.ghost)` | `E1356` — owns no account, and how to declare one |
| `irr("party.lp")` | `E1356` — write it as a reference |
| `irr(…)` in a stream | `E1355` — a fold over the finished projection |

Only what cannot be known until the run — flows that never change sign —
refuses at run time, naming the party, because a metric the author declared
must not silently go missing.

`E1355` closed a live silent path: `irr()` in a stream amount compiled AND ran,
substituting zero under `status: ok`, because a stream amount that fails to
evaluate warns and carries on. It is now refused across stream amounts,
activations, event guards, waterfall steps and account inflows.

`docs/13` §7.72 closes and `docs/31` W4 phase 2 is done, which leaves the
waterfall calculator a benchmark case and a surface.

### Added: a model may declare a metric (§7.25)

Metric keys were minted in two places: the engine (`model.*`) and a pack
(`domain.*`). A case computing a deal-specific figure — a class weighted
average life on the deal's own axis, a crossover date, an overcollateralisation
ratio — had nowhere to name it, so the number the case existed to check sat
unnamed in an `expected.csv` column instead of in `expected_metrics.json`
beside the published figure it reproduces.

```cfdl
metric gross_revenue = series_sum("ops.revenue", 0, 4)
metric total_cost    = series_sum("ops.cost", 0, 4)
metric margin        = metric.gross_revenue + metric.total_cost
```

- **`metric.<name>`** is a third namespace beside `model.*` and `domain.*`, so
  a results file says who minted every number in it.
- Evaluated ONCE, at the horizon, over the finished projection — the valuation
  plane of `docs/28` §2. A metric is a fold over a completed projection, never
  a recurrence, so nothing it computes feeds back into the walk. It may read
  the projection tail, which is what a forward-looking figure needs.
- It may read series (including streams a CONTRACT lowered — §7.50), entity
  fields, `inputs`, `cfg`, the engine's `model.*`, and `metric.<name>` for any
  metric **declared above it**. Metrics compose in declaration order, the rule
  waterfalls already follow, which makes the dependency an order rather than a
  graph: **`E1354_METRIC_FORWARD_REF`** refuses a forward reference and, with a
  different hint, a self-reference. **`E1008_DUPLICATE_METRIC`** refuses a
  repeated name.
- Every declared metric reaches every scenario summary, at no extra cost:
  scenarios and the deterministic block publish the same map, so a scenario
  grid can assert a derived figure per column and not only the engine's
  built-ins.

`docs/01` §15.3 is new and normative; §15.2's "CFDL models do not declare
output metrics" is no longer true and is gone. `metric` joins the reserved
words (§18.1, 86). This unblocks §7.72 (participant-level returns) and
completes `docs/31` W4 phase 1, the commercial plan's only critical path.

Two parser tidies fell out of it. A `metric` declared AFTER a contract was
silently dropped: the contract parser asks `is_statement_start` while the
`assume` statement carried a private copy of the same list, so a new keyword
ended one statement and not the other — the scan now delegates to the one
definition. And `ExprEnv::empty()` already existed, so the derived `Default`
added alongside the constant-folding check was a second way to say it.

### Fixed: the diagnostic register is now the codes the tools emit

Nothing compared `docs/08` against what anything emits, and the drift ran both
ways: **198 documented against 192 emitted**, 18 of them documented by nothing
that could produce them, three the packs emit missing from the page, and two
pack READMEs naming a number under a name its validation does not use.

The register is the repair catalogue an agent reads. A promised diagnostic that
never fires teaches it to wait for a code that will not come.

**Nine checks built** — each probed first, and each silent before:

| code | what compiled before |
|---|---|
| `E0005_INVALID_DATE_LITERAL` | `from 2026-13` reached the IR as `"2026-13-01"`; only the run refused it |
| `E1002_DUPLICATE_CONTRACT` | reported only as `E5007`'s downstream symptom, and only under a pack |
| `E1004_DUPLICATE_PHASE` | unreported |
| `E1006_DUPLICATE_OPTION` | both options reached the IR; `exercise option` resolved by position |
| `E1007_DUPLICATE_EVENT` | both events reached the IR and both fired |
| `E2201_EVENT_WHEN_NOT_BOOL` | `when 42` ran `status: ok`, the guard taken as false |
| `E2202_STREAM_ACTIVE_NOT_BOOL` | `active when 7` ran `status: ok`, the stream never paying |
| `E3003_EXPR_TYPE_ERROR` | `"100" + 1` ran `status: ok`, the amount taken as 0 |
| `E3004_EXPR_ILLEGAL_OP` | `10 and 3` the same |

The duplicate family was dead because the symbol table was: `phases`,
`contracts`, `options` and `events` were declared on it and never written by
anything. `E0005` moved into the LEXER, where the token is minted, because
`try_lex_date` checked shape and never the calendar.

The last four are the **constant-folding subset only**. `cfdl-expr` has no type
inference, so a general check is a feature and not a missing diagnostic; an
expression built only from literals is decided by evaluating it, and anything
naming a binding is left to the run. `EXPR_UNKNOWN_NAME` vs `EXPR_EVAL` draws
that line, so "depends on a binding" is never read as "the arithmetic is wrong".

**Ten entries deleted**, each a condition already caught elsewhere or one that
cannot arise: `E1305` (`E2106`), `E4001` (`E1311`), `E3002` (`E5002`), `E2203`
(open-world fields; a declared field already refuses an ill-typed write),
`E2003`, `E4002`, `E4003`, `E5001`, and the `E6004`/`E6021` retirement
tombstones. **Three documented**: `E6030`, `E6033`, `E7011`.

**`docs/08` §8 now says what a pre-release language should**: before 1.0 a
retired code is deleted and its number returns to the pool; the never-reuse
rules take effect at 1.0, when saved artifacts start outliving the release that
produced them.

**`make diagnostic-parity`** holds it: `docs/08`, the crates, the pack
validations and the pack READMEs must agree, both directions, with test-only
codes listed by name and reason.

### Removed: `activate` / `deactivate contract` (§7.73)

A contract is not one behaviour; it is a COLLECTION OF STREAMS. `cre.lease`
lowers into base rent, recoveries and abatement, and an all-or-nothing switch
gives one answer where the real cases need a per-stream one: forbearance stops
principal while interest accrues, an early termination stops rent while a fee
flows and recoveries continue, a facility at the end of its draw period stops
drawing and amortizes.

The action could not be reached from a model in any case — a contract carries
only its type, so there was no instance to resolve — and the engine had no
runtime for it, journaling `ignored`. Both better spellings now exist: name the
stream the contract produced (§7.50, shipped) or declare a lifecycle state and
gate each stream with `active in state` (§6.1), which is checked, level-
triggered so it can end as well as begin, and journaled.

- The productions leave the grammar and the parser. No new diagnostic marks
  the absence: `E0004_EXPECTED_TOKEN` already answers the retired spelling with
  "Expected 'stream' after activate/deactivate", spanned on the offending
  token.
- `E1303_UNRESOLVED_CONTRACT_REF` resolved only this action's target and is
  DELETED, not entombed. `docs/08` §8's never-reuse rule is written for a
  released language; pre-1.0, with no installed base, a code no one has seen
  is removed rather than carried.
- The IR's `Action` drops both kinds and its now-unused `contract` property.
- The engine's contract arm goes. The `ignored` outcome stays, because an
  action kind hand-written IR carries and no compiler emits still needs one —
  which is what the engine unit test now pins.

### Fixed: a model may name the streams its own contracts produce (§7.50)

`docs/01` §13.2 gives the modeller `deactivate stream <name>`, and §9.1's own
example of a stream name — `cre.lease.base_rent` — is a name a CONTRACT
produces. The modeller could not use it. The symbol table is built before the
pack is chosen, so at name resolution a contract's streams do not exist, and
the check could not tell "not yet lowered" from "misspelled": both were
`E1302`. A loan repaid early kept taking debt service, and the same model
expressed the stop correctly the moment the pack was dropped — a pack was a
trade rather than an addition.

The check moved to the compiler's post-lowering position, where every stream
that will exist is known — the same place `exercise option` is already checked.
Typo detection is unchanged, since a misspelling still matches nothing, and the
hint now lists every stream in the model, declared and lowered alike.

`fixtures/valid/event_stops_lowered_stream` is the case: `cre.permanent_debt`
runs 27,500.00 through its interest-only months and 36,845.249537 while it
amortizes, and the prepayment event takes debt service to 0.00 from the period
it fires. `fixtures/invalid/event_stream_typo` pins the typo.

`docs/04` §1.1 now records what its stage list omitted — lowering is the one
GENERATIVE stage, creating names no statement wrote, which is why a check over
lowered names cannot sit at name resolution.

### Fixed: a waterfall must say when it distributes (§7.45)

`docs/01` §10.1 has required a waterfall's `schedule` in normative text since
the waterfall entered the spec. The compiler did not enforce it: a missing
schedule lowered to `on <time.start>`, so the waterfall distributed once, in
the first period, of whatever that period happened to produce — a pot of 500
across five periods paid `500, 0, 0, 0, 0` and said nothing about the 2,000 it
never distributed. The engine believed the opposite (no schedule meant every
period) and could never act on it, because the compiler emitted no such IR.

The omission is now **`E1348_WATERFALL_NO_SCHEDULE`**, and the engine's
unreachable branch is gone — one component states the rule. The schedule is
half of what a distribution says: between its scheduled periods the pot
accumulates, so "every quarter" and "once at exit" are different deals rather
than two spellings of one.

No shipped model, benchmark, example or exercise relied on the default; the
four fixtures that omitted a schedule are `invalid`/`repairs` pairs pinning
unrelated diagnostics, and now declare one. `fixtures/invalid/waterfall_no_schedule`
pins the new code.

### Added: the agent-eval harness (docs/32 Phase 3)

`tools/agent-eval/runner.py`: the benchmark suite becomes the grader.
Three tiers — repair (the 70 fixture/fix pairs; scored on compile),
transcribe (the 42 cases: CASE.md and permitted reference material, never
`expected.csv`; scored compiles/runs/matches with partial credit by
asserted column and metric), extend (declared assertions; format defined,
public set empty until independently-derived assertions exist). Agents
plug in as the scripted `replay`, a `cmd:` subprocess, or an HTTP
endpoint — the provider-agnostic seam. Grading imports the benchmark
runner's own comparison, so the eval and `make bench` cannot grade
differently. `make agent-eval-selftest` (sampled replay at 100%) joins
`ci-gates`; `make agent-eval-replay` is the full gate. `--benchmarks-dir`
points the transcribe tier at a private held-out split.

### Added: benchmark exemplars in the machine surface

`docs/machine/exemplars.md` (served at `/machine/exemplars.md`, and inlined
in `llms-full.txt`): 18 of the 42 benchmark cases, curated so every
core-language mechanism and every meaningfully-composed pack pattern
appears at least once — full-deal models, each matched to an external
reference in CI, each carrying its case's "what it exercises" grid.
Near-duplicates (rate sweeps, twins) stay out; the full suite remains the
grader. The set doubles as the transcribe-tier nucleus for docs/32 Phase 3.

### Added: the documentation surface for machines (docs/32 Phase 2)

`tools/gen-machine-docs.py` generates `docs/machine/` from the same sources
the site renders: `llms.txt` (normative links plus the course chapters as
Optional), the machine docs bundle (language spec, full EBNF grammar,
expression environment, IR and results schemas, pack interface, diagnostics
catalog, controlled-English authoring contract, glossary, terminology
register, pack rosters), `llms-full.txt` (the bundle plus every learn
chapter), and the diagnostics -> repair catalog (per invalid fixture: the
minimal failing example, its golden diagnostics, and — where recorded under
`fixtures/repairs/` — the minimal fix, compile-verified by the generator).
`sync-content.mjs` stages the artifacts to `site/public/`, so cfdl.dev serves
`/llms.txt`, `/llms-full.txt`, and `/machine/*`. `make machine-docs-check`
joins `ci-gates`. Also in this pass: a valid-examples corpus
(`docs/machine/valid-examples.md`, every golden fixture model) joins the
served artifacts; all 70 invalid fixtures now carry a compile-verified
repair in `fixtures/repairs/`; five pack validation codes the compiler
emits joined the docs/08 §7 register (`E6031`, `E6032`, `E6040`, `E6041`,
`E7010`); the expression parser now emits its registered code
`E3001_EXPR_PARSE_ERROR` instead of the bare `EXPR_PARSE` (two diagnostics
goldens re-blessed); the orphaned `gold/diag/expr_type_error.diag.json` is
removed; `check-site-voice.py` validates `ste-allow:` rule ids against
docs/22 §3; and backlog 7.80–7.82 record the remaining measured gaps
(unexemplified codes, runtime expression code naming, CE tier mapping).

### Added: cfdl-mcp — the agent toolkit (docs/32 Phase 1)

A new `cfdl-mcp` crate serves the authoring loop over MCP stdio: `compile`,
`run`, `diff`, `explain`, `lookup`, and `skeleton`, every result a typed
schema-carrying structure. `diff` is a Rust port of the benchmark runner's
comparison discipline (same column resolution, absolute tolerances, and null
semantics); `lookup` answers from the embedded terminology register and
re-derives pack contract coverage by scanning benchmark declarations;
`skeleton` compiles its own output before returning it. Post-engine
enrichment (domain metrics, statements) moved into a shared `cfdl-run`
facade, so the CLI, HTTP server, and MCP server call one function and cannot
drift. The Phase-1 gate is `crates/cfdl-mcp/tests/self_test.rs`: the tool
loop rebuilds `benchmarks/cre/office_two_tenant` and matches its
expectations, and localizes a seeded divergence.

### Added: M1 — the period walk, shipped end to end

`docs/28` restated from proposed to shipped; `docs/29` is the phase record.
The engine evaluates one period at a time — state settles, streams evaluate,
waterfalls the schedule names distribute — so realised cash feeds the
model's own logic with a one-period lag, and the whole corpus computes the
same numbers it did under the column order (the collapse property, asserted
over every blessed fixture in `cargo test`).

What the walk carries, phase by phase:

- **Backward reads** (`docs/28` §4): logic reads settled series at or before
  the previous period; `E1134` narrowed from a prohibition to that rule. A
  unit goes delinquent because rent was not received.
- **The account** (§5.1): a declared cash location under the balance law —
  `from <account>` pots, `pay … to account` credits, `prev.<account>` in
  logic, no floor on the balance, every movement journaled.
  `results_version` 0.4.
- **The lifecycle machine** (§6.1): a core-language finite state machine
  with guarded edges declared only as used; no latch — edge availability is
  the memory; event status writes validated against the relation.
  `results_version` 0.5 (`transition` journal action).
- **`state_enter`** (§6.2): the third schedule anchor; each entry opens its
  own window and a re-entered state re-anchors.
- **The priced exception** (§7): a forward window in an amount is a
  valuation setting a causal amount, evaluated after the causal cells
  settle; the cycle is refused with the path named; the expense stop
  declares itself in the valuation plane (decided by the MIT Rentleg
  reference tie, recorded in `docs/26`).

Not one shipped golden moved in any number across the milestone; the
re-blessings were `results_version` strings, `model_hash`, and the additive
machine publication in IR.

### Added: which models can be walked, and the forward reads that answer it

Phase 2.0 of `docs/29_period_walk_implementation.md`. A period walk cannot
serve a read that reaches forward — at period 3 there is no period 24 — so
before the walk is built, the corpus is measured for reads ahead of `time.t`.
`cfdl_expr::series_windows` extracts each read with its window bounds and
`window_bound_is_backward` classifies them; `cfdl_engine::walk_eligibility`
answers the question for a model.

**Asked of model source the answer was two benchmarks. Asked of the compiled
IR it is six fixtures and three causes**, and the difference is the finding:
the forward-income exit is the CRE pack's `cre.exit_forward` LOWERING, reading
`[time.t + 1 .. time.t + 12]`, so every model using that contract reads
forward and no scan of model source can see it. That widens `docs/28` §7's
migration from one benchmark to one pack contract and every model that uses
it. A waterfall's STEPS read series too, and are not in `ir.streams`, which is
how an absolute `[0..5]` window looked walkable.

Two shapes had to be taught before the cumulative windows came out right, and
both reported as forward until they were: the compiler normalises `0` to
`0.0`, and the OpCo pack writes a trailing-twelve-month window as
`time.t - 12.0 + 1.0`, which is `t - 11`. Reading only the first term refused
the walk for every LBO model in the corpus.

The classifier is conservative by construction — an unrecognised shape is
forward, because a walk that guesses wrong reads a cell that does not exist
yet — and `only_the_known_models_read_forward` pins the measured set so the
plan's table cannot go stale.

### Added: Monte Carlo says when each act happened, and how often — 7.18 closed

`monte_carlo.journal` publishes one row per distinct act with the share of
trials in which it occurred and the distribution over the period it FIRST did.
7.18 ruled out the shape everyone reaches for first — a per-trial log is
trials x acts of output, and nobody reads ten thousand copies of the same
sequence — and asked for the distribution instead. This is that: bounded by
the model's acts rather than by the trial count.

`fixtures/valid/monte_carlo_journal` exercises both halves in one model. A
balance falls 40 a period from 1,000, reaching 560 over a twelve-month
horizon, against a covenant level sampled between 200 and 800: a draw above
560 breaches inside the horizon and a draw below never does. The covenant
breaks in 13 of 40 trials (32.5%, against a theoretical 40% — within sampling
noise at that count), and where it breaks the first breach spans periods 6 to
11 with a median of 8.

Quantiles are nearest-rank rather than interpolated, because a quantile of
periods should be a period: "the covenant first broke around month 9", not
month 9.5. The mean stays fractional, being explicitly an average.

Closes backlog 7.18. Backlog: 40 items.

### Backlog: 7.73 — `activate contract` is the wrong grain

Found while journaling action outcomes: the `ignored` outcome that
`activate`/`deactivate contract` produces cannot be reached from a model,
because a contract carries only its type and the reference does not resolve
(§7.63). The deeper problem is the grain — a contract is a COLLECTION of
streams, and forbearance (principal stops, interest accrues), termination with
a fee, and the end of a draw period all need per-stream answers that one
switch cannot give.

The granular mechanism already exists twice over: `deactivate stream`, blocked
only by §7.50's unaddressable generated names; and better, a lifecycle state
with each stream declaring the states it is active in — declarative, checked,
level-triggered, and journaled under `docs/28` §6.1. The entry recommends
retiring the action rather than building a runtime for it, which also retires
the `ignored` outcome the journal carries.

Backlog: 40 items.

### Added: the journal — what the model did, and whether each thing happened

`deterministic.journal` records every causal act with its outcome:
`applied`, `declined`, `overridden`, `ignored`, `failed`. Phase 1 of
`docs/29_period_walk_implementation.md`, and `docs/28` §8.

`transitions` records field CHANGES, so an action that changed nothing
appeared nowhere. The case that motivated this: an event activates a stream
whose own `active when` is false. Both gates must pass, so the activation does
not turn the stream on — and the modeller got a zero series, no warning, and
nothing in the results saying an activation had been refused. It is now one
row, at the period the refusal began, with the count of scheduled periods it
covered.

A waterfall step carries the pot **before and after** it took, so a payee that
got less than it was owed is visible as a short pot rather than inferable from
the amount: `300,000,000 → 276,066,457.50 → 60,664,575 → …` reads as the
cascade it is. Option elections say whether the guard held or an event forced
them, and a force outside the `exercisable in` window is `declined` with the
reason.

Flat on purpose — one row per act. A golden asserts on lines, a reviewer greps
for a stream name, and the schema checks one row type.

**Additive, and nothing else moved.** The key is omitted when a model has no
events, options or waterfalls, so 91 of 108 blessed fixtures are untouched.
Across the 17 that gained it, the diff is the new key and nothing else: zero
non-journal lines changed in any results golden.

`fixtures/valid/journal_action_outcomes` pins five outcomes in one model. The
sixth, `ignored`, cannot be reached from a model at all: `activate contract`
needs a contract to name, and a contract carries only its TYPE, so the
reference is `E1303_UNRESOLVED_CONTRACT_REF`. That sequences backlog **7.63**
(instance naming) before **7.40i**'s contract runtime — the runtime needs
something to name — and the outcome is covered by an engine unit test against
hand-written IR.

### Added: the engine's blessed corpus in `cargo test`, and three gaps it closed

`crates/cfdl-engine/tests/golden_corpus.rs` runs all 108 blessed fixtures
in-process — `gold/ir` in, `gold/results` compared — in about three seconds,
inside `make ci`. Before it, `cargo test -p cfdl-engine` was 27 unit tests
over a 2,200-line engine in 0.01 seconds, and the engine's real suite was
reachable only from `tools/golden-runner`. This is what makes `docs/28`'s
collapse property — every blessed number unchanged — checkable on every
commit.

Three real gaps are closed with it:

- **The annual valuation grain's dispatch had no end-to-end coverage.** The
  arithmetic under it was unit-tested; nothing set `"valuation_grain":
  "annual"` in any fixture or benchmark, so the match arm reading it could be
  deleted with every blessed number unmoved. `valid/valuation_grain_annual`
  pins it — two years of level cash, where the grain moves the NPV 4.3%
  (145,785.12 against 152,351.97), and the annual figure is hand-checkable as
  84,000/1.1 + 84,000/1.21.
- **A model-declared Monte Carlo run had none.** `run monte_carlo trials N
  seed S` is specified in `docs/01` §15.1 and read from the IR when the run
  config asks for no Monte Carlo of its own; every existing MC fixture
  supplied the mode in `run.json` instead, so the pickup path was exercised by
  no model. `valid/run_declared_monte_carlo` declares everything in source —
  `assume ~` distributions, trials and seed — leaving `run.json` a discount
  rate.
- **`trials 0` was accepted by a parser whose own message says positive.**
  `parse::<u64>()` took it, the engine's `trials > 0` guard then declined to
  set the run up, and a model asking for Monte Carlo compiled, ran, and
  published no Monte Carlo section with nothing saying why. Now refused
  (`invalid/run_monte_carlo_zero_trials`), with the engine's IR-level guard
  covered by a unit test since no model can reach it any more.

### Removed: the mutation-testing pre-work, and what it taught before it went

`docs/29` phase 0.2 specified a blessed mutation baseline over the engine.
Removed, with the reasoning recorded rather than dropped. The concern was
right — "the goldens pass" is only evidence if the goldens would notice a
change — but 860 mutants, hours of wall-clock and a 2.4 GB tree copy per
parallel job do not fit the machines this project is built on, and the
technique answers only whether the TESTS notice, where the external benchmarks
tied to published figures answer whether the NUMBERS are right. No target, no
baseline register, no gate.

Its findings are kept, as fixtures and tests: the three above. And the
question it asked is kept in the plan — when a change's success criterion is
the absence of a difference, check that something would have noticed; during
phase 2 that is done by hand, per hypothesis, in minutes.

### Added: logic that reads a stream is refused, and 7.71's "silent" claim is corrected

`E1134_SERIES_READ_IN_LOGIC` refuses `series_sum`/`series_avg` in an event's
guard or action value, a field's `init`/`next`, and an option's election or
payoff. All are evaluated before any stream has a value, so the read binds
nothing: a guard gets `false` and never fires, a rule gets `0` and `prev`
carries the collapse for the rest of the run. Phase 0.1 of
`docs/29_period_walk_implementation.md`, and the gate the period walk's
reorder needs before it is attempted. The three probe models from `docs/13`
§7.71 land as `fixtures/invalid/`, including the lagged spelling that §7.71
and `docs/28` §4 make legal later — it is refused now rather than inert, and
moves to `valid/` when the walk lands.

**And the entry it implements was wrong about one thing.** 7.71 claimed these
failures were silent. They are not: the engine emits one warning per period
naming the read and the substitution. What is true is narrower and is now
what the entry says — the run reports `status: ok`, the exit code is 0, the
CLI prints nothing, and the warnings live inside the results document.
`tools/benchmark-runner.py` fails on engine warnings; the golden runner does
not, and `fixtures/valid/evaluation_order` carried four of them in its
blessed golden for the life of the fixture, with a comment presenting the
guard that never fired as expected behaviour. That fixture's dead event is
removed: `cash_flag` stayed at its `init` either way, so no published number
moves — the IR shrinks, and the four warnings go with it. The correction is
recorded in 7.71, `docs/28` §1 and `docs/29` §0.1.

### Added: docs/29 — the period walk's implementation plan, and backlog 7.72

The plan that realizes docs/28, in seven dependency-ordered phases: the
loud-fail gate and the engine's Rust-side guard; the journal (independent,
may ship first); the walk itself, gated on the collapse property — full
golden suite byte-identical, wall-clock within noise; the read
rules; the account; the state machine; the migrations; the documentation
surface. Three decisions stay open and each names who settles it.

New backlog entry 7.72: a participant's realized return has no construct —
measuring what a payee actually earned means hand-assembling their streams
outside the language. Gated on the account (docs/28 §5.1), whose per-party
journal is the input the metric folds over; rides on 7.25's declared
metrics. Backlog: 40 items.

### Changed: docs/28 §5 — the account, where carried cash lives

The pot-as-balance paragraph becomes the account construct: a declared cash
location — general, or owned by a party for cash allocated but not yet paid
— with a per-period balance law. A negative inflow lowers the balance with
no floor, because an account fed a deal's whole net cash IS the deal's
cumulative position; what is floored is the draw. Steps may pay to an
account (the reserve pattern), waterfalls may draw from one, and logic
reads `prev.<account>` strictly backward. Owed is not held: receivables
stay entity fields. Carryover is opt-in by declaration — `available` and
`remaining` keep their shipped indenture meanings, so the collapse property
holds by construction. Fixture list extended: reserve fund-and-release,
trapped-cash cure, the cumulative-sum identity, Highlands via an account.

### Added: docs/28 — the period walk, M1's evaluation-order specification

The causal stages (state, events, streams, distributions) advance one period
at a time and settle in a fixed order within each period; the results stage
is named the valuation plane and keeps forward reads. Logic reads realised
cash strictly backward, so the cell graph is acyclic by construction and
cycles stay refused, never iterated. Waterfalls keep schedule sovereignty,
and the pot becomes carried state: it grows by the declared inflow each
period, scheduled distributions draw it down, and residue carries. The
lifecycle gains its edges: declared, re-enterable transitions where an
undeclared edge is refused, driven by events whose latch becomes a trigger
policy, with a third schedule anchor — a state entry — so a delayed
construction hangs its window off the transition. The journal becomes the
execution trace, with every action's outcome recorded. For every model with
no cash-into-logic edge the results must be byte-identical — the golden
suite is the proof obligation. Pre-work unchanged: loud failure for
unbindable series reads (7.71), and a Rust-side guard for the engine.

### Backlog: an event cannot see realised cash, and the failure is silent

New entry 7.71, from three probe models. A guard reading a stream by bare path
is refused loudly at IR load; the same read spelled `series_sum(name, t, t)`
compiles, runs, and the event is silently inert — and so is the strictly
backward spelling, `series_sum(name, t-1, t-1)`, which reads only settled
history and is cycle-free by the `prev` argument. A field recurrence reading a
series nulls its whole expression and the run reports ok. The cause is the
stage order: state and events complete over the whole timeline before any
stream value exists, so at guard time even the past has not been computed. The
entry asks for a loud refusal now (7.38's argument, one environment over) and
records the per-period interleave under which the lagged spelling becomes
legal.

Backlog: 39 items.

### Backlog: three items closed, three narrowed to what actually remains

Closed. **7.15** — both stated reasons are false: One Lincoln Street exercises
`cre.construction_loan` against its own external figures, and the retail source
it called unregistrable is vendored. **7.8** — supersedes itself and records no
provenance, saying outright "Nothing currently needs it". **7.42** — the
specification no longer teaches a model-level discount rate; the one surviving
`assume discount_rate = 0.10`, in `docs/01`'s multi-file example, was also a
DEAD assumption never read, and is now `base_rent`, which the example's
contract actually uses.

Narrowed rather than closed, because each had real residue under a stale
headline. **7.41** was five invariants, four now gated; what remains is that
nothing checks a waterfall's `from` for naming a pack-lowered family, which the
pack-series gate could cover. **7.44** claimed the engine is one file of 5,341
lines; the split shipped and `lib.rs` is about 2,200 across ten modules, so
what remains is the crate boundary, with its cost stated. **7.51** shipped its
schema gate; what remains is that a `parameter_overrides` key is never checked
against the model, so a typo overrides nothing and the run reports ok — the
same family as the unresolved-name work, one layer out.

Backlog: 38 items.


### Removed: backlog 6.3, a symmetry gap with no case behind it

6.3 wanted a one-shot flow whose PERIOD differs from its contract term — a sale
agreed in one period and settling in another. Removed under this file's own
standard: it recorded no provenance, saying plainly that "no live case forced
this; it is a symmetry gap noticed while fixing the disposal discounting", and
the header requires that each entry say what could not be expressed and what
forced the discovery, because "a backlog item with no provenance is a guess".

The neighbouring facts also make it a thin case: a one-shot placed on the
SETTLEMENT date already says when the cash moves, `start`/`mid`/`end` now place
it within that period, and a single-occurrence recurrence accepts `net <n>`
where a one-shot correctly refuses it for having no accrual period.

Also de-stales the "Where these came from" section, which cited items 1.4-1.7
and 5.1 by number after those items had been closed and removed. It now names
the benchmarks and their external references instead, which is the part that
does not dangle.

Backlog: 41 items.


### Rewritten: backlog 5.2 — a recurrence cannot read the model's own streams

5.2 said cash sweeps and revolver draws were "still blocked" and needed an
ordered allocation pass. Both halves were wrong: a sweep is expressible today
(`benchmarks/opco/lbo_financing_cases` sweeps a Term Loan B and reproduces the
reference's MoIC and IRR across three financing cases), and the allocation pass
exists as the waterfall.

What remains is narrower and is a duplication cost rather than an absence: a
field's `next` reads no series, so a balance whose movement is determined by
realised cash must restate how that cash is computed. Much of that is avoidable
today by letting a FIELD own the quantity and having the published stream read
the same field — verified to the cent, with the free-cash-flow build stated
once. `docs/26` carries the shape.

Corrects two pack notes that repeated the false claim: `packs/opco`'s lowering
rules and README both said sweeps "need per-period persistent state" and were
unavailable, which conflated a pack's choice not to lower them with a language
limit.

Backlog: 42 items.


### Changed: the PV benchmark derives its depreciable basis (backlog 4.2)

`benchmarks/energy/utility_pv_singleowner` carried `basis = 85000000` with the
arithmetic — `100m - 0.5 * 30m` — in a comment beside it, and the credit as a
second hardcoded `30000000`. Installed cost and the credit rate are now stated
once as assumptions, and the capex, the credit and the reduced basis all derive
from them. Reproduces the reference exactly; 40/40 benchmarks.

4.2 asked for an `itc_basis_reduction` term. None is needed: a term holds an
expression, so the adjustment can be stated rather than pre-computed. The pack
still declines to derive the basis, which is right — adjustments are
jurisdictional — but that is a reason for a model to say which one applies, not
to paste in the answer.

Corrects two stale claims in `packs/energy/README.md`: it told readers to
"state the adjusted figure" (now: state the adjustment), and it still said
"`energy.ptc` does not round" when the rule computes the statutory staircase
via `round_to` with `round_step` defaulting to 1.00.

Backlog: 42 items.


### Closed: backlog 4.1, already shipped

`round_to(x, step)` is in the expression vocabulary and `energy.ptc` already
computes the statutory credit as a staircase, gated on a `round_step` term
that defaults to 1.00 — whole dollars per MWh, since 0.1 cent/kWh is $1.00/MWh
and rounding a per-MWh figure to 0.10 would be indistinguishable from not
rounding. The item's "general case", a recurrence over already-rounded
figures, is live in `benchmarks/cre/hud_home_multifamily` on five expense
lines: an entity field whose `next` reads `prev`.

Corrects a stale comment on `round_to` claiming the recurrence "needs a stream
to read its own prior period, which the language cannot do". A stream cannot;
a field can, and the comment now shows the shape. Recorded in `docs/26`,
including the unit trap and why an omitted staircase survives reconciliation
(the error alternates sign rather than drifting).

Backlog: 43 items.


### Closed: backlog 3.3, a documented convention rather than a defect

`net <n>` discounts a payment from the period it lands in, not from its exact
due date. `docs/12` already names this — "Discounting is at bucket
granularity… This is a stated convention, not an oversight" — quantifies it at
roughly 0.5% of an affected flow on a monthly grid at 12%, and notes that the
first-order effect, moving the cash two periods later, IS captured.

The item also described the error backwards: measured, `net 30` and `net 45`
land in the same bucket, so the residual costs a `net 45` flow nothing against
a `net 30` one. Recorded in `docs/26` with the measurements, including the two
ways this is easy to reason about wrongly. Removing the convention would be an
architectural change — per-payment discount offsets in `npv_with_offsets` —
that moves numbers in every model using `net`, and belongs as its own item if
ever wanted.

Backlog: 44 items.


### Changed: schedule placement is one axis — `start` / `mid` / `end`

**Breaking, pre-release: `due` is removed.** Write `start`.

Where a flow sits in its period was three independent booleans (`due`, `mid`,
`at_period_end`) repeated across the parser, IR, engine and pack interface. It
is now a single `Placement { Start, Mid, End }`, spelled `start`/`mid`/`end`
everywhere, including pack rules as `schedule_placement = "start"|"mid"|"end"`
(replacing `schedule_at_period_end`).

Three things this fixes:

- **A one-shot can now settle at its period's close.** `schedule on <date> end`
  is what a disposal needs — a reversion is taken at the end of the holding
  period, so a year-5 sale discounts five periods, not four. A pack rule could
  already reach it; a hand-written model could not. Closes backlog 3.2.
- **Two placements can no longer be stated**, because the positions are
  alternatives rather than flags. `E2109_SCHEDULE_CONFLICTING_PLACEMENT` is
  narrowed to clashes across different axes — a placement against a day rule
  or against `net` payment terms — and no longer covers placement-vs-placement,
  which is now a parse error.
- **The grammar file was stale** and is corrected: `schedule_on` omitted the
  `mid` modifier that the parser accepted and the spec documented.

Defaults are unchanged — a recurrence still defaults to `end`, a one-shot to
`start` — but every position is now nameable in both forms, so no model has to
rely on a default that differs by form.

**No number moved.** 17 IR goldens changed shape and the 17 results goldens
that differ do so only in hash fields; 163 goldens and 40 benchmarks pass.
15 shipped models migrated from `due` to `start`.

Also closes backlog 2.4 and 3.1, both already expressible — see `docs/26`.


### Closed: backlog 2.4 and 3.1, both already expressible

**2.4 (sequential-pay note classes).** The liability stack is not missing —
`benchmarks/credit/americredit_2017_1` models a real deal as one waterfall of
22 prospectus clauses in 30 `pay` steps and reproduces the published grid,
including all 48 weighted average lives. What the item calls for is pack
surface, not capability. `docs/26` records the shape: accumulate free cash,
cascade on the distribution date, state a claim rather than a payment, and
carry a class balance as a field ONLY when what is distributed diverges from
what is produced (a step-down amount, an overcollateralization redirection,
losses). Where payments track production, the outstanding is a derived stream
and no balance exists at all.

**3.1 (a stub first period).** The calendar is a neutral coordinate grid, not
the deal's fiscal year, so a leading stub belongs nowhere near it. The grid
takes any start date, the schedule carries placement (`due`/default/`mid`, day
rules, and its own `short_front`/`long_front` stub policies), and discounting
reads `(period + offset) / ppy` continuously. A 30 September valuation with
30 June fiscal years lands at 0.75, 1.75, 2.75, 3.75 and 4.75 years out on a
plain monthly grid. The item's premise came from omitting `due`, which places
cash at the END of each period — the documented default, not drift.

Backlog: 46 items.


### Research: the A.CRE Retail Development Model catalogued as source 107

Salvaged from a stale branch. The catalogue now carries 107 sourced models,
63 of them with full period-by-period numeric output, and the A.CRE Retail
Development Model (v2.2) is flagged as the sharpest available test of
solve-to-target over a circular reference — the shape `docs/14` §5 rules out
of the language and would need an explicit, bounded, convergence-checked
construct if ever built.

A work-in-progress benchmark for that model was NOT salvaged: it has no
`case.toml`, so the benchmark runner cannot read it. It is preserved at
`d197c5768a867eaec44cb6aec4c5f78ec3d06272` (branch `docs/remediate-backlog`, deleted) and can be recovered from
there if the case is picked up.


### Closed: SMM and MDR are expressible today (backlog 2.3)

2.3 asked for `smm`/`mdr` terms because a 1% SMM pool had to be entered as
`cpr = 0.11361512828387077`, "computed by hand and unrecognisable to a
reader." A term holds an expression, so the conversion can be stated instead:
`cpr = 1 - pow(1 - 0.01, time.ppy)` — legible, and cadence-correct via
`time.ppy` rather than a literal 12, which the item itself asked for. Verified
byte-identical to the hand-computed constant across every stream and all 361
periods of a 30-year pool.

Closed rather than built: the remaining ask was vocabulary, not capability, and
the expression is a complete statement with no staleness risk. The idiom is
recorded in `docs/26` for any quoted-cadence mismatch.


### Added: E5027_ACTUAL_AMORTIZATION_BASIS

`amortization_day_count` chooses what a level payment is struck on, and an
Actual convention expands to `(360 / time.days_in_period)` — a period-local
value that the annuity then applies to every remaining period. January struck a
payment as if all remaining months had 31 days, February as if all had 28, so
the payment moved with month length. Measured on a single 1,200,000 loan at 6%
with no pool, no prepayment and no defaults: a **460.68 swing** over twelve
months. Now refused, for every pack and every instrument.

The pairing a loan document states is unaffected and still compiles: accrue on
`act/360` via `day_count`, strike the payment on `30/360`, and the payment
holds at 7,194.61 while interest moves 6,200.00 to 5,594.43 with month length.
`day_count` itself is untouched — a per-period divisor is exactly right for a
per-period accrual.

This is the sibling of a failure already measured on the accrual divisor
(697k-754k in `benchmarks/credit/mbs_pool_conventions`); splitting the two
divisors fixed that spelling and left this one, and the shipped fixture pairs
`act/360` accrual with `30/360` amortization, so the broken combination was
never exercised. No shipped model changes.

Backlog 2.2 is rewritten rather than closed: it diagnosed this as a pool-factor
limit and proposed a gate for pools, and both were wrong — the defect is in the
closed form and applies to a single loan. Two stale claims in the credit
README are corrected alongside (age-varying hazards ARE expressible, and
loan-level heterogeneity ships as `benchmarks/credit/mbs_pool_by_loan`).

### Changed: an expiring rent restriction is an event, not a hand-written switch

`benchmarks/cre/hud_home_multifamily` carried its affordability cliff as an
`if(time.t < n, restricted, market)` restated in both the rent line and the
vacancy line. It now carries a `restricted` field on the property, cleared once
by an `affordability_expires` event, with vacancy reading the rent stream
rather than restating how rent is computed. Reproduces the source workbook to
the cent (40/40 benchmarks).

The reversion is now published as a transition record — period 14,
`2038-01-01` — so the workbook's own off-by-one (its switch fires a year before
its "15-year" label reads) is auditable against the source instead of surviving
only as a comment.

**No pack gained a term for this**, deliberately. The shape recurs in every
pack — CRE restriction to market, energy PPA to merchant, credit fixed to
floating — so a per-pack `reverts_after` would be built four times, each with a
sentinel for "never", each less expressive than the expression it replaced.
Events latch, which is exactly a transition that happens once; `docs/26`
records the shape as the mirror of the guard-based recurring regime.

Closes backlog 1.7.


### Housekeeping: closed backlog items removed, their lessons kept

`docs/13` states that closed items are removed, not archived — a shipped
capability is described in the language documentation, and reasoning that
turned out to be wrong goes to `docs/26_lessons_learned.md`. Items 1.2, 1.6
and 7.65 were marked closed in place rather than removed, which left the count
misleading. They are now removed and the file holds 50 open items.

`docs/26` gains what they were worth keeping: a corrected-reasoning entry (the
three CRE requirements did NOT need new pack terms — a capability gap and a
demonstration gap look identical from the backlog) and two how-to entries (a
line derived from other lines, an assumption derived from other assumptions).


### Added: CRE lines derived from other lines (vacancy, management fee, expense stop)

The three requirements the backlog called "three ordinary CRE requirements, one
cause" — a vacancy that tracks the rent roll, a management fee that is a
percentage of effective gross income, and an expense stop that resets to a
later year's actual opex — all work now, and none needed a pack change. A
contract term already holds an expression and the expression may name another
stream; what was missing was the engine ordering, which dependency-ordered
waves supply.

Two templates make the patterns discoverable: `cre.vacancy_loss.tracking` and
`cre.opex_line.management_egi`. The fixed-fee management template stays — some
agreements do state one — and is no longer described as a workaround.
`fixtures/valid/cre_derived_lines` pins all three, including the deepest chain
(rent, then vacancy and recoveries reading rent and opex, then the fee reading
those, then the exit reading every opex line) and a 46% vacancy cliff.

Closes backlog 1.2 and 1.6.


### Added: an assumption may be derived from other assumptions

`assume net_sf = inputs.gross_sf * inputs.efficiency` compiled and then failed
at run: assumptions evaluated in name order against an empty environment, so a
read of another assumption found nothing, the assumption was skipped, and every
read of it resolved to nothing. They now evaluate in dependency order, with
random assumptions resolved first as leaves. A circular derivation is refused
with the cycle named (`'gross_sf' -> 'net_sf' -> 'gross_sf'`), on the same
principle as cross-stream reads one layer down: no order satisfies it, and the
engine does not iterate toward a fixed point. Closes backlog 7.65.

### Changed: MIT Rentleg Plaza reads actual opex instead of restating it

The benchmark's expense reimbursements rebuilt "actual opex per SF" from the
inputs inside each recovery stream — base, trend, fixed share, that year's
occupancy — because reading the opex stream made a recovery a reader, and
`cre.exit_forward` reads the recoveries. Dependency-ordered waves permit that
chain, so both halves of the formula (this period's opex, and the 2004 reset
stop) are now the same `series_sum("cre.opex.line", ...)` read with different
windows. Every reimbursement reproduces to the cent and the net exit price is
unchanged at 3,051,540.54. Closes the language half of backlog 1.2.


### Changed: streams evaluate in dependency-ordered waves

The engine's two-phase stream split — streams that read no series, then
streams that do, against a store sealed between the rounds — banned every
chain of series reads deeper than one to get acyclicity. Waves get exactly
acyclicity: `series_references` extracts each read as written,
`selector_matches` resolves it to the streams it names (the same edges the
old phase guard walked to *reject* depth-two chains), and each stream
evaluates one wave past the deepest stream it reads, against a store in
which everything it names is finished. A genuine circular read is the only
rejection left — `SeriesCycle` names the path (`'a' -> 'b' -> 'a'`) and the
engine never iterates toward a fixed point (docs/14 §5). A stream whose
series names are computed at runtime keeps its old semantics: it evaluates
after every literally-named stream and cannot itself be read. All existing
goldens are byte-identical; `fixtures/valid/series_depth_chain` pins the new
depth with hand-checkable arithmetic. Unblocks backlog 1.2, 1.6 and the
percentage-of-EGI management fee (migrations are their own PRs).

### Added: E1346_STREAM_READS_WATERFALL_STEP

A stream's `series_sum`/`series_avg` naming a waterfall step aggregated to
zero in silence — the step counted as a known producer, so no warning fired,
and the store never holds a step, so the read found nothing. docs/03 §3.2
always said a step's series is visible to a later waterfall's `from` and to
nothing else; the compiler now says so too, beside E1341/E1342.


### Fixed: a name that resolves to nothing is refused, not read as zero

`docs/03` §2 has always said *"Unknown variables are hard errors (EXPR_EVAL),
not nulls."* Every layer honoured it except the engine, which caught the error
and substituted zero: `inputs.typo` or `time.typo` compiled clean, warned once
per period, produced a column of zeros, and the run reported `status: ok`.
Entity fields were the exception — `E1131` has always refused those — so one
of three namespaces was checked.

The fix splits by what each layer can know.

**`time.` is closed**, so the compiler refuses it:
`E1133_UNKNOWN_TIME_READ` names the five bindings the engine actually binds
(`t`, `date`, `days_in_period`, `phase`, `ppy`), at the reader's own source
span, beside the `E1131` check it extends.

**`inputs.` cannot be checked at compile time** — an input may be supplied
entirely by the run configuration, which the compiler never sees, and
`run_dists_full` is exactly that model. The engine refuses it instead, where
every source is known, with a message naming the distinct unresolved names
rather than repeating one per period.

The distinction that makes this safe is DECLARED SOMEWHERE versus BOUND HERE.
An input declared only as a Monte Carlo distribution is unbound in the
deterministic pass and stays a warning; a name nothing declares is fatal.
`cfdl-expr` now separates the two failures at the source, giving an
unresolvable name its own `EXPR_UNKNOWN_NAME` code so arithmetic that merely
failed is still tolerated.

Goldens: 157 pass, zero numeric differences — the only golden movement is the
error code inside one fixture's warning text.

### Fixed: the Python extension's freshness guards no longer hash packs into it

`tools/py-stamp.py` hashed `packs/` into the extension's build stamp and
`test_native_is_fresh` globbed `packs/**/*.toml`, both on the premise that
cfdl-pack `include_str!`s every pack TOML into the binary. It does — but only
under `#[cfg(feature = "embedded-packs")]`, and crates/cfdl-py takes cfdl-pack
without it. The extension says so itself: *"No pack directory was provided and
this build has no embedded packs."* The SDK reads packs from disk at run time,
so a pack edit cannot stale the binary.

The cost was a false alarm on one of the most common actions in this
repository. Editing a lowering rule failed `py-check`, which demanded `make
py-develop`; the rebuild was a no-op because cargo correctly saw no dirty
input, the `.so` kept its mtime, and the mtime test then failed too — the only
way out being to touch a crate source to force a rebuild that changed nothing.

Both guards now stay quiet through a pack edit and speak on an engine one,
verified both ways.

The second defect the first one hid is closed as well. `make py-develop` ran
`pip install -e` and then stamped unconditionally, so a rebuild that did not
happen was certified fresh regardless; it was caught only because a second
guard disagreed. The stamp now records the artefact's identity beside the
source digest, so a stamp written without a build no longer matches the file
it claims to describe — and `--write` with no extension present refuses
instead of writing.

The notebook render stamp keeps its `packs/` entry, correctly: pack data
changes what a notebook prints, because the SDK reads it at run time.

Closes backlog 7.21.


### Added: every pack ships contract templates, and a gate keeps them working

`templates.toml` was populated for CRE's two line-item contracts and nowhere
else — opco, energy and credit had none, and the instruments decomposed this
week (a mortgage with eight terms, a project loan, three exit forms) had none
either, despite the design rules saying the conventional vocabulary ships as
templates.

Now 39 templates across four packs: CRE gains its five instruments beside the
fifteen line items; opco, energy and credit get their line items and
instruments from scratch. Each carries the terms its rule requires, with a
conventional value where a convention exists.

`tools/check-pack-templates.py` renders every template from its declared
defaults — what the editor inserts when no parameter is supplied — and
compiles it. It caught two real defects on its first run: `principal = 0` and
`balance = 0` defaults that the packs' own validations reject (E6051, E8022,
E9001), and declared ranges that outran the model they were placed in. A
template that does not compile teaches a shape the language rejects and leaves
the modeller debugging the pack's own snippet.

### Found: an option's type resolves against nothing (backlog 7.67)

`PackOntology` carries entities, contracts, lifecycles, references and
relations — no options. No pack declares, lowers or validates an option type,
so the three type names shipping across five models are free text the compiler
accepts unexamined. Entities and contracts both have the surface options lack.


### Changed: the teaching and examples adopt expression terms

The adoption pass over learn, training and the examples — pack-using models
only; hand-written core models are untouched:

- The learn quick reference and chapters 6 and 19 teach the capability: a
  signed formula ("CPI plus 50 basis points") is stated in the term, and the
  hand-written boundary moves to STRUCTURE the contract does not have.
- `examples/opco_with_growth` showcases it: the growth plan is a step curve
  read by the term — `growth_rate = curve_value("growth_plan", time.date)` —
  5% through the ramp, 3% mature.
- `mit_rentleg_plaza` states its opex agreement in the term
  (`amount_year = inputs.opex_psf_full * inputs.building_sf`) and restates the
  2004 expense stop FROM THE INPUTS, killing the parameter staleness backlog
  1.2 records. Reconciliation to MIT 11.431J holds at the same tolerance.

### Found: an assumption cannot reference another assumption (backlog 7.65)

`assume b = inputs.a * 2` compiles and then fails at run — assumptions have no
dependency ordering, the assumption is "ignored", and every read degrades to a
warned zero with the run reporting ok. Found because a benchmark rewrite tried
it and the reconciliation caught the zeros.


### Removed (breaking): the curve-selector twin terms

`escalation_curve`/`occupancy_curve` (cre.opex_line), `growth_curve`
(opco.revenue_line/opex_line/capex_line) and `tax_rate_curve`
(opco.cash_taxes) are gone, with the `if("" == "", scalar,
curve_value(...))` splices they required. A varying rate is stated in the
term itself — `escalation = curve_value("cpi", time.date) + 0.005` — which is
what expression terms exist for. The canonical curve semantics are unchanged
(a model-declared `curve`, read flat-forward by `curve_value` at the period's
date), and a mistyped curve name is an evaluation error rather than a silent
scalar fallback, which is stricter than the twin was.

`draw_curve` (construction) and `index_curve` (floating pools) remain: the
curve IS those instruments' primary input, not a selector beside a scalar.

Migration: `x_curve = "name"` becomes `x = curve_value("name", time.date)`.
The two shipped models using twins migrate that way; results are unchanged in
every golden — zero numeric differences across all 98 results files — and all
40 benchmarks pass.


### Changed (breaking): pack streams follow `[domain].[category].[line]`

The pattern is now stated normatively in the pack interface (docs/07 §6.4):
a pack-lowered stream is named `[domain].[category].[line]{.[instance]}`, a
contract's several streams share their `[domain].[category]`, and contract
TYPES may keep underscores — they are authoring surface. Hand-written streams
are the modeller's own names and carry no pattern.

Four streams violated it and rename, values identical everywhere:

- `loan.permanent_debt.*` → `cre.debt.*` (wrong domain, inherited from the
  retired netted stream)
- `opco.capex{.id}` → `opco.capex.line{.id}`
- `opco.taxes{.id}` → `opco.taxes.cash{.id}`
- `cre.pct_rent{.id}` → `cre.pct_rent.overage{.id}`

Metric selectors match the new names by prefix and are unchanged.
damodaran_fcff's two asserted columns rename with their streams; all 40
benchmarks and 22 exercises pass with identical values.


### Changed (breaking): one CRE revenue line item, `cre.revenue_line`

`cre.ops_revenue` — a singleton with a raw per-period amount and no growth —
is replaced by `cre.revenue_line`, mirroring `cre.opex_line`: instanced per
statement line, the vocabulary in the instance name
(`cre.revenue_line.parking`), a blended figure when unsuffixed, `amount` or
`amount_year` with `escalation`.

It is the first rule designed for expression terms: there is no
`escalation_curve` twin, because `escalation = curve_value("cpi", time.date)`
states the agreed formula directly. Five conventional templates ship —
parking, storage, antenna, laundry/vending, blended.

`E6020`/`E6021` retire with the old contract (E6064 replaces the amount
check; a revenue term legitimately reaches the projection tail so forward NOI
has revenue to read). The forward-NOI windows read the instanced family.
Eighteen models migrate by contract name alone; every golden value is
identical under the stream rename, and all 40 benchmarks and 22 training
exercises pass unchanged.


### Changed: exit contracts state the sale gross, with selling costs as their own line

Four exit rules folded selling costs into the proceeds via `* (1 -
selling_costs)` — `cre.exit`, `cre.exit_forward`, `opco.exit_ebitda`,
`opco.exit_perpetuity`. A pro forma shows the gross sale value and the
transaction costs; a netted figure can show neither. The proceeds streams are
now GROSS and each contract lowers a sibling `*.selling_costs` outflow
(`investing.selling_costs`), so gross less costs is the old net: net cash
flow and NPV are unchanged to the digit on every fixture, verified
numerically. Statements gain a "Less: selling costs" row.

`mit_rentleg_plaza` now asserts both columns — the gross at ten times forward
NOI and the 5% commission are each the source's own stated quantities; the
net_cash_flow column is untouched. All 40 benchmarks pass.


### Changed (breaking): `energy.debt_service` lowers to the whole instrument

Same decomposition as `cre.permanent_debt`, same reason: one netted stream,
no draw. Now `energy.debt.proceeds{.<id>}` (financing.debt_proceeds,
`funded_at_close` default 1), `.interest{.<id>}` and `.principal{.<id>}`.
The legs sum to the level payment exactly and fold into
`domain.energy.debt_service_periodic` unchanged, so DSCR holds to the digit.

The five levered energy benchmarks state `funded_at_close = 0` with the reason
in each model — their references net operations against debt service and never
book the draw. Three expected.csv files asserted the netted stream by name;
the column now asserts the positive-signed periodic subtotal instead — same
magnitudes, per period, against the same sources; only the sign convention
moved. All 40 benchmarks pass.


### Changed (breaking): `cre.permanent_debt` lowers to the whole instrument

The most common CRE financing instrument lowered to ONE netted stream,
`loan.permanent_debt_service` — and emitted no proceeds at all: a $6m mortgage
that funded nothing. The netting foreclosed the interest/principal split — a
tax line, interest coverage, an amortization schedule — and the missing draw
made every levered return wrong even while the net line reconciled.

Per the contract design rules (docs/07 §6.4) it now lowers to three streams:
`loan.permanent_debt.proceeds{.<id>}` (financing.debt_proceeds, at closing),
`.interest{.<id>}` (financing.interest) and `.principal{.<id>}`
(financing.debt_principal, balloon folded in when opted on). All three carry
`dot_suffix`, so a deal may hold more than one mortgage.

The split is exact — ipmt + ppmt equals the level payment identically — and
`domain.cre.debt_service` now folds the two legs where it folded the netted
line, so DSCR is unchanged to the digit (verified: the subtotal series is
byte-identical on the smoke fixture). The operating statement gains Interest
and Principal amortization rows.

`funded_at_close` (default `1`) draws the principal at `term_start`. A
reconciliation whose source's cash flow starts post-financing states
`funded_at_close = 0` and says so — office_two_tenant does, because its
reference nets rents against debt service and never books the draw.

The learn capstones had hand-written a `harbor.perm_proceeds` stream to work
around the missing draw. The workaround is deleted from all five capstone
models and the chapter; the contract funds itself with the same $9.5m, and
every expected exercise metric is unchanged.


### Added: a contract term may hold an expression

A term was a literal or one reference to a declared input, and nothing else —
`escalation = inputs.cpi + 0.01` was a parse error directing the modeller to an
`assume`. But what was agreed is often itself an expression: a lease escalating
at "CPI plus 50 basis points", a coupon of "SOFR + 225bp". Forcing those
through a model-level `assume` moved the agreement out of the contract and left
a reference behind. The grammar always said `literal_or_expr`; the
implementation now conforms to it.

```cfdl
terms {
  base_rent  = 42000
  escalation = curve_value("cpi", time.date) + 0.005
}
```

An expression term is compiled at its own site (`E5025_TERM_EXPR_INVALID` when
it does not parse) and substituted into lowering rules PARENTHESISED — template
expansion is a textual splice, and `a + b` into `{{x}} * {{y}}` would otherwise
silently associate as `a + (b * y)`. Literals and input references substitute
verbatim, byte-for-byte as before: across 98 IR and 98 results goldens, every
pre-existing file is unchanged.

An expression is valid where the rule uses the term in an expression — an
amount, a field's `init`/`next`. A term a rule reads as a name, date,
frequency, or period count must stay literal:
`E5026_TERM_EXPR_IN_LITERAL_SLOT`, and `E5017_PERIOD_TERM_NOT_LITERAL` now
covers expressions and non-numeric literals (the latter previously misreported
as a missing term). A non-integer expansion of a rule's net-days no longer
falls back silently to the contract's payment terms — it is an error.

Pack bounds apply where the value is knowable: literal terms are checked at
compile time, input references against their clip, expression terms not at all
— the same tier the specification already granted input references.

Two latent defects closed on the way: `inputs.cpi * 2` would have classified as
a reference to an input named "cpi * 2" (terms are now kind-classified by the
parser, and an input reference requires exactly one identifier); and
`x = -(a + b)` was silently DROPPED, the pack default applying in its place.

`fixtures/invalid/term_trailing_tokens` — the fixture that pinned the old
restriction — is now `fixtures/valid/term_expression`, and its `1000 + 500`
produces results identical to stating `1500`. docs/01 §8.2.1 is rewritten:
a term records what was agreed, and what was agreed is often a formula.


### Fixed: the published grammar now describes the language the parser accepts

`docs/schemas/CFDL_v0_1_Grammar.ebnf` is normative — `docs/02` says
implementations MUST support it and offers it for download as parser-generator
input. Checked by hand against the parser and against every shipped model, five
productions were wrong:

- `contract_stmt` wanted two qnames, a mandatory `on entity`, and `term` as a
  clause BEFORE the block. It is one qname, an optional `on entity`, and `term`
  is an item inside the block. As written the grammar rejected 519 of the 520
  contract declarations in this repository.
- `entity_stmt` made the block mandatory; it is optional. `entity_block` allowed
  only `IDENT literal_or_expr`, and the real block holds four item forms: a
  field, a rule-bearing field with `init`/`next`, `part of`, and `state`. The
  `entity_field` production was defined and referenced by nothing; it is now the
  field form.
- `option_stmt` had no `on entity`; it is optional and used.
- `map_entry` said a contract term is `literal_or_expr`. A term is a literal or
  one declared input — never a compound expression — which is why a pack
  composes `curve_value` from a term holding a curve name.
- `stream_effect_stmt` and `tags_block` describe features that do not exist.
  Both are now marked NOT IMPLEMENTED, matching §18.2, which lists `owner`,
  `direction` and `tags` as reserved for features not yet built.

Closes backlog 7.49. Filed 7.61 for the standing check that would stop this
recurring.


### Fixed: the reserved-word list is the lexer's list, and says which words do nothing

`docs/01` §18 is the published statement of what a modeller may not name a
thing. The lexer reserved 95 words and §18 listed 57, so 38 identifiers stopped
working with nothing to explain why. Fourteen reserved words are read by no
production at all — they render in error messages and nowhere else.

§18 now splits into words a production reads (81) and words reserved for a
feature not yet built (14), and `tools/check-keyword-register.py` holds the
split to the lexer and the parser. Adding a keyword now fails CI until §18 says
which it is.

Closes backlog 7.47 and 7.48.

### Found: `Mon` through `Sun` describe syntax that has never existed

The weekday keywords were documented as weekly-schedule anchors. No production
reads them, `on` accepts only `day <n>` or `eom`, and `weekly` is not a calendar
frequency. The specification described a feature that was never built, which is
exactly what the gate above exists to prevent. Filed as backlog 7.60.

Backlog 7.49 gains a second instance of the same drift in the grammar itself: a
contract term is documented as `literal_or_expr` and is in fact a literal or one
declared input.


### Added: a statement row may itemise a stream family, and the CRE operating statement does

A `line` row has always accepted `streams` selectors as well as `categories` —
"for what a category cannot express" — but they were undeclarable in practice.
A statement is checked at pack load for completeness: every category a lowering
rule emits must be claimed by some line row. Stream selectors contributed
nothing to that set, so a stream row could not replace the category row it
duplicated, and keeping both double-claims every stream.

The loader now resolves a stream selector to the category of the rule that emits
that family, which it can do because the pack declares both. So a stream row
claims a category exactly as a category row does, and claiming one both ways is
now an error rather than a double count discovered at runtime.

`cre.opex_line` is the case that needed it: one contract instanced per expense,
every instance carrying the same category, so a category row could only ever
render one number however many lines a model declared. The CRE operating
statement now itemises the nine conventional expense lines that
`templates.toml` ships, with `domain.cre.opex_total` as the total.

A modeller naming their own instance is not lost — it matches no row and the
evaluator emits a `residual` row for it. Verified end to end: a model declaring
`property_tax`, `insurance` and a `moat_maintenance` line of its own renders
-240,000 and -60,000 on their own rows, -15,000 under "Unclassified", and
-315,000 as the total.

Closes backlog 7.59.


### Changed (breaking): one CRE operating expense contract, `cre.opex_line`

`cre.property_opex` and `cre.ops_expense` are removed. They differed only in
escalation — an annual figure compounding against a raw per-period amount — and
in whether they could be instanced, and on that second point their names said
the opposite of the truth: `cre.property_opex` carried `{{contract.dot_suffix}}`
and repeated, while `cre.ops_expense`, the generic-sounding one, was a singleton.
A modeller could not be expected to pick correctly and nothing documented it.

`cre.opex_line` replaces both and spans the range from a single blended figure
to a fully itemised schedule. Grain is how many instances a model declares;
LEVEL is the entity each hangs on, with `part_of` doing the rollup, because the
CRE ontology already says a building may be modeled as one asset, as unit types
or as individual suites — a `level` term would restate that where the two could
disagree.

New terms: `amount` / `amount_year`, `escalation` / `escalation_curve`,
`pct_fixed`, `occupancy` / `occupancy_curve`. Escalation and occupancy are each
a PAIR rather than a toggle, because a contract term is a literal or one
declared input and never an expression — so a time-varying rate cannot arrive
through the term and the pack composes the `curve_value` call from a curve name.
`escalation = 0` is the default and is "no escalation".

The occupancy factor `pct_fixed + (1 - pct_fixed) * occupancy` closes backlog
1.1. `benchmarks/cre/mit_rentleg_plaza` now runs on the pack contract instead of
a hand-written stream and still reconciles to MIT 11.431J at a one-cent period
tolerance: $4.81/SF on 30,000 SF at 81% fixed gives $135,161 of 2001 opex, not
$144,300, and the published answer depends on it.

`templates.toml` — a real loaded structure the LSP reads for snippet
completion, empty in this pack since it was created — now ships nine
conventional expense lines plus a blended one.

### Fixed: opex dropped out of forward NOI when the contract was instanced

`cre.exit_forward` summed `series_sum("cre.property.opex", …)` by BARE name
while `cre.property_opex` was instanceable. A `.*` pattern matches the bare name
and its children; a bare pattern matches only itself. So
`cre.property_opex.taxes` emitted a stream the exit rule did not see: opex fell
out of forward NOI, NOI was overstated, and the exit price struck off it was too
high. Silent, and worth real money. It is the same defect found and fixed for
`cre.pct_rent`, one term over in the same expression.

Both opex terms are now one `series_sum("cre.opex.line.*", …)`.

### Migration

Rename the contract, and `opex_year` to `amount_year`. Note `cre.lease_unit`
also has an `opex_year` term — its expense stop — which is unrelated and
unchanged.

`E6020_CRE_OPS_MISSING_AMOUNT` and `E6021_CRE_OPS_INVALID_SCHEDULE` no longer
apply to the expense side. E6020 demanded a bare `amount`, which `E6061` now
replaces with "either amount term"; E6021 rejected a term reaching into the
projection tail, which an expense legitimately does so that forward NOI has
expenses to read.

New codes: `E6061_CRE_OPEX_LINE_MISSING_AMOUNT`,
`E6062_CRE_OPEX_LINE_PCT_FIXED_RANGE`, `E6063_CRE_OPEX_LINE_OCCUPANCY_RANGE`.

Published stream names change: `cre.property.opex` and `cre.ops.expense` both
become `cre.opex.line{.<id>}`. Scenario overrides keyed
`stream.cre.ops.expense:amount` move with them.

Goldens were re-blessed. Across 98 results files and 88,863 numeric leaves,
every number is identical under the rename — only names moved.


### Changed (breaking for external packs): a validation's `match` field is gone

A pack validation matched a contract by EXACT name unless it declared
`match = "instance"`. A model must suffix a contract whenever a deal has more
than one of something — two tenants are `cre.lease_unit.tenant_a` and
`.tenant_b` — so a validation that omitted the flag was silently skipped on the
form models actually use. It never fired, and nothing said so. Two thirds of
the shipped validations were dead that way: `E7001_OPCO_LINE_MISSING_AMOUNT`
rejected `opco.revenue_line` with no amount and accepted
`opco.revenue_line.core` with no amount, one character apart.

That was fixed by requiring the declaration and gating it. The gate reads
`packs/*/validations.toml` in THIS repository, and packs are a published
extension point — so an author writing a pack elsewhere still got the silent
default, from a field the pack interface documentation never mentioned.

Lowering never offered the choice. `rule_matches_contract` has always matched
instances unconditionally, for the case that decides what cash a contract
produces, and its six lines were a byte-for-byte copy of the validation path's
`Instance` arm. Validations were the outlier and no reason was ever recorded.

So `ContractMatch` is deleted, matching is unconditional, and both callers share
one predicate. The 68 `match = "instance"` declarations are removed, and so is
the gate that required them — there is nothing left to state.

**Migration:** delete `match` from your validations. `PackValidation` sets
`deny_unknown_fields`, so a pack that still carries it fails to load with an
error naming the key rather than loading with it ignored. `match = "exact"` has
no replacement; if you have a case that needs it, it is worth hearing about.

Golden outputs are byte-identical: every shipped validation already declared
`instance`, so nothing in this repository changes behaviour.


### Added: the run configuration schema is a gate

`docs/schemas/run.schema.json` describes what a `run.json` may contain, and
nothing read it. Not the engine, which parses run configs with serde and applies
its own rules; not the CLI; not a test.

It drifted, in the direction silence always allows. `valuation_grain` had been
accepted by the engine and documented in the user guide for as long as it
existed, and the schema never listed it — and because `DeterministicCase` sets
`additionalProperties: false`, a run stating its own grain would have been
rejected by the schema it is supposed to conform to. Nobody noticed, because
nothing checked.

`tools/check-run-schema.py` validates every run config in `benchmarks/`,
`fixtures/`, `training/` and `examples/` — 123 of them — and `make ci` runs it.
It follows `check-ir-schema.py`, including the rule that a gate which can pass
without running is not a gate: `CFDL_REQUIRE_SCHEMA_GATE=1` turns a missing
`jsonschema` into a failure rather than a skip.

It catches what the schema's own description says the design prevents — *"a
misspelled key would otherwise produce a clean run with wrong numbers"*:

```
deterministic: Additional properties are not allowed ('anual_discount_rate' was unexpected)
deterministic/arithmetic: 'float' is not one of ['decimal', 'excel_compat']
```

What it does not yet catch is an override key that resolves to nothing. Two of
the four key shapes could be checked — `inputs.<name>` names an assumption and
`stream.<name>:amount` names a stream, both declared — and two could not, since
`cfg.<path>` and `obs.<path>` name nothing the model declares. Recorded as the
open half of `docs/13` §7.51.

### Added: a run selects its arithmetic, and `act/act` joins the day counts

Two items from `docs/13` §6, both of which had a capability sitting one step out
of reach.

**A run may state `"arithmetic": "excel_compat"`.** `Mode::ExcelCompat` existed
in `cfdl-calc` and was exercised only by that crate's own tests: the engine
always called plain `eval`, and there was no run-config key and no flag.

```json
{ "deterministic": { "arithmetic": "excel_compat" } }
```

```
(0.1 + 0.2) * 1e15    decimal       300000000000000.0
                      excel_compat  300000000000000.06
```

Decimal stays the default and is what every published number uses. The mode is
carried on `ExprEnv` rather than threaded through each evaluation site, so no
signature changed; 152 goldens are byte-identical, which is the check that the
default moved nothing. It is run-wide — scenarios and Monte Carlo trials inherit
it, because a scenario varies the deal's drivers and the rate it is valued at,
not the arithmetic every scenario shares. An unrecognized value is refused.

**`year_frac` accepts `act/act` (ISDA).** The span is split at calendar-year
boundaries and each part measured against its own year's length, returned over
the common denominator 365*366 so the result stays one exact integer ratio like
every other basis:

```
2024-07-01 -> 2025-07-01   act/act  0.998622651   = 184/366 + 181/365
                           act/365  1.000000000   = 365/365
2024-01-01 -> 2025-01-01   act/act  1.000000000   a leap year is still one year
```

The item had recorded the blocker as the expression environment not exposing the
days in a period's year. That holds for the pack's `{{model.accrual_divisor}}`,
which resolves to one number per period and so cannot carry a denominator that
changes with the year — but not for `year_frac`, which receives both endpoints.
Two paths of different difficulty, recorded as one. The divisor remains open.

`docs/schemas/run.schema.json` gained `arithmetic`, and `valuation_grain` with
it: the engine has always accepted that key and the schema had never listed it.
The schema sets `additionalProperties: false`, so a run stating its grain would
have failed validation had anything validated against the schema. Nothing does.

### Fixed: `time.phase` answers with the phase name

The specification requires the expression environment to support `time.phase`
(`docs/01` §16.2), the expression guide lists it in the `time` namespace, and
the course introduces it in chapter 5 as "the current phase's name" and repeats
it in the quick reference. It was never computed. `crates/cfdl-engine/src/env.rs`
inserted `ExprValue::Optional(None)` under `phase` at both env-building sites, so
it was null in every period of every model.

Null fails quietly in both directions, and one of them reads as success: a null
equals no string, so an inclusion test never fired, and a null differs from every
string, so `active when time.phase != "construction"` — the natural way to write
"not during construction" — was true during construction too and the guard did
nothing at all. Neither form warned. The value also reached results as the Rust
debug form of the empty optional.

It now answers with the name of the phase covering the period. The membership
test is the one `state.rs` already applied to an option's `exercisable_in_phase`,
lifted into `phase_at` and called from both sites, so a phase means the same
thing to a guard as it does to an option rather than being written twice. Where
no declared phase covers the date the answer is still null. Where two phases
cover it — overlapping phases compile today — the FIRST DECLARED wins, so the
answer is a stated order rather than map iteration.

Running the course's chapter 9 model with phases added and `time.phase`
published:

```
period  0 (2026-01)  construction
period 11 (2026-12)  operations
period 35 (2028-12)  operations
```

No model in `packs/`, `fixtures/`, `benchmarks/`, `examples/` or `training/`
read `time.phase`, so nothing depended on the old null; 151 goldens are
unchanged.

### Fixed: a waterfall step cannot read its own payments — backlog 7.41 item 3

A step that reads its own waterfall through `series_sum` was answered with a
silent zero. Steps publish when their waterfall finishes, so the read sees
nothing; the arithmetic around it then quietly does nothing.

`fixtures/valid/waterfall_after_contract` was doing exactly that. It capped a
note at its balance by subtracting what it had already paid, the subtraction
took nothing away, and a $500,000 note paid out $1,200,000 across six periods
with a golden that agreed. That is the failure `docs/13` §7.41 predicted as a
preferred return paid in full six times, found in the repository rather than in
a report.

`E1342_WATERFALL_SERIES_NOT_VISIBLE` now refuses the spelling at compile time,
beside `E1341_WATERFALL_FORWARD_REF` — the same failure one spelling over, so
the two answer the same reference the same way. Reading an EARLIER waterfall is
the documented composition and still compiles;
`fixtures/valid/waterfall_nested_split` pins that. The message names the model
that works: `paid.<step>` for this period's payment, and for a running total a
balance the distribution moves — a field the step pays and the balance
subtracts, which works today (`docs/26_lessons_learned.md`).

The fixture now states the per-period cap it was computing all along. Its
`ledger_hash` is unchanged and the other 148 goldens are byte-identical, so
every published number was already this one — only the expression became
honest.

`series_references` moved to `cfdl-expr` so the compiler and the engine read
series names with one scanner rather than two that could drift.

### Added: AmeriCredit 2017-1 — an auto ABS that builds its own enhancement

`benchmarks/credit/americredit_2017_1` reproduces the percent-outstanding grid
a sub-prime auto ABS publishes for six note classes at four ABS speeds, plus a
weighted average life to call and to maturity for each. It is the first case in
the suite whose notes have to build credit enhancement rather than simply
receive collections: excess cash accelerates principal toward 14.75%
overcollateralization net of the reserve, and principal beyond the target is
retained as a Step-Down Amount rather than paid, subject to a floor of 0.50% of
the initial pool.

The reference implementation reproduces **184 of 195 informative cells** inside
the grid's own whole-percent rounding floor — mean error 0.2479 against the
0.25 a correct model predicts, maximum 0.4990 against 0.4973 — and **46 of the
48 published lives** exactly. The CFDL model agrees with it to 4.4 cents on a
$305m class across every class and period.

Four conventions the prospectus does not state had to be recovered, each by
testing candidate readings against all four published speeds: a January-cutoff
pool pays twice before the first distribution; ABS runs from origination, which
retires four seasoned pools outright at 2.00%; the step-down floor is 0.50% of
the initial pool; and weighted average life runs 30E/360 from closing to the
18th with a 25-day stub. Eleven cells remain outside the floor, all Class A-1
or A-2 in the first six months; three candidate explanations were tested and
rejected rather than fitted.

**Found by building it:** the case states its distribution twice, once in the
waterfall and once across seven balance recurrences, and the two can disagree —
which is how a servicing fee came to be right in one and wrong in the other.
The duplication is avoidable: state the amount once as a field, and let the step
pay it while the balance subtracts it. Recorded in `docs/26_lessons_learned.md`,
where the first reading — that a waterfall cannot tell a balance what it paid,
and that the engine needed restructuring — is corrected.

### Added: a writing standard, and the documentation held to it — backlog 7.28–7.35

The documentation estate — cfdl.dev, learn.cfdl.dev, and every source that
feeds them — now has what the numbers have had all along: a standard, a
measurement, a remediation, and a gate. The audit is `docs/21`, the standard is
`docs/22` (CFDL-CE, derived from ASD-STE100 and tiered by content type), the
terminology register is `docs/terminology.toml`, and the accessibility
assessment is `docs/23`.

**Measured first.** 70,438 words of published prose, sentence by sentence. The
findings were concrete: the same words published in two spellings (41
conflicting forms once the generating sources were read, not the 7 the rendered
pages showed), one object under three names, 143 RFC 2119 keywords across the
three specifications with no definition anywhere, no glossary, and not one page
with a meta description.

**Then fixed.** US spelling throughout — 537 replacements, identifiers renamed
with their dependents and four goldens re-blessed label-for-label with every
numeric token verified unchanged. The specs define their normative keywords by
BCP 14. Every page states what it is (generated pages derive their description
from sources that already exist, so there is no second wording to go stale).
`/docs/glossary` publishes 47 terms generated from the register. All 22
exercise prompts are numbered imperative steps — mean sentence length
19.8 → 11.4 words — and the chapters' procedures instruct instead of asking.

**Then gated.** `check-site-voice.py` enforces the mechanical subset — retired
spellings and synonyms load from the register at run time, so the standard, the
glossary, and the enforcement cannot drift apart. Judgment rules (sentence
length, voice) stay in review, deliberately: a gate that flags judgment gets
disabled. The specifications are now read by a prose gate for the first time.

**And assessed for accessibility.** WCAG 2.2 AA, on production builds and the
deployed sites, both themes: the muted-text token failed contrast in both
themes and was split per theme, the playground splitter could not report its
value to a screen reader, and tables, code blocks, and the results panel were
unreachable by keyboard. All fixed; axe reports zero violations on every swept
page. Conformance is **not** claimed until the human assistive-technology pass
runs — that is backlog 7.35, the one item the program leaves open.

### Fixed: streams are line items — backlog 1.3, 1.5, and the reporting half of 7.14

A stream is the atom a statement reports, so a stream that is secretly an
aggregate is a row a statement cannot show. Three of them were.

**A property may now have more than one expense line (1.5).**
`cre.opex_line` takes a suffix and `domain.cre.noi` selects
`cre.opex.line.*`. `benchmarks/cre/hud_home_multifamily` carries its four
published sub-lines as four streams and **asserts all four independently**
against the Sample workbook's Operating Pro Forma rows 18–21, where it
previously asserted only their total. The four states already existed — split
for the rounding reason — so this moved nothing: their sum reproduces the
previously asserted total at every anchor year.

**Free rent is its own deduction (1.3).** `cre.lease_unit` emits
`cre.unit.abatement.<id>` and publishes base rent GROSS; the abatement family
sits in `domain.cre.noi`'s denominator, so the two net to the rent collected.
Previously a model could report the line OR have it counted in NOI, never both.
Verified as an exact decomposition — gross + abatement equals the previous net
to 0.00e+00, and NPV, NOI and DSCR are unchanged.

**HUD's mortgage separates P&I from MIP (7.14).** The pro forma's debt line is
one number and the workbook defines it as P+I+MIP. Both legs are now grounded
in the First Mortgage Sizing tab rather than inferred: MIP is the stated 0.450%
of the stated $150,000 principal (675.00, flat, exact), and debt service is the
residual of the published "Calculated Monthly P+I+MIP Payment" of 1,165.7819 —
which reconstructs the 13,314.3828 that backlog 7.14 had recorded by hand.
`domain.cre.debt_service` carries the MIP because coverage there is measured
against the whole published line, which is what the workbook's own DSCR uses.

**No expectation moved.** An intermediate version of this change used the sizing
tab's unrounded 13,989.3828 and moved the lifetime figure to 195,851.36, on the
reasoning that the pro forma's 13,989 was a rounded display. It is not: that
cell is `=ROUND(...,0)`, so 13,989 is what the workbook COMPUTES, and its
published DSCR is that rounded line divided into a rounded NOI. Using the
unrounded payment would have been more precise and less accurate. The model
applies the workbook's own round — via the `round_to` it already uses for the
expense recurrence — rather than restating 13,989 as a constant, so the
derivation stays visible.

Every native stream in every pack-using model is now classified, so the
completeness gate that Stage 8 turns on starts from zero unclassified streams.

**Invariants hold across all ten changed results goldens**: `model.total`,
`model.npv`, `model.irr`, `model.moic`, `domain.cre.noi`, `domain.cre.dscr` and
every `model.net_cash_flow` period are identical. What changed is that
aggregates became lines.

### Added: provenance, resolved inputs, and a ledger hash — `results_version` 0.3

A published line item can now be traced back to the term that struck it.

**`inputs.streams`** records, per stream, the contract terms a pack rule
actually consumed. Not the contract's whole term map: a contract lowers to
several streams and each reads a different subset, so "the contract's terms" is
not an answer to "what struck this line". One `cre.lease_unit` contract produces
three streams with three different term sets:

    cre.unit.base_rent.tenant_a   <- rent_year, escalation
    cre.unit.recoveries.tenant_a  <- opex_year, opex_escalation, expense_stop_year,
                                     pro_rata_share, gross_up_factor (pack default)
    cre.unit.ti_lc.tenant_a       <- ti_total, lc_total

`defaults_applied` separates the values the model stated from the ones the pack
assumed, because "the model said 0" and "the pack assumed 0" are different facts
and a reader tracing a number needs to tell them apart.

Note what this was NOT: `crates/cfdl-compile/src/lib.rs` emits `terms: {}` on
every contract and always has, so nothing was being un-dropped. The terms are
read from the rule's own templates *before* expansion — afterwards the keys are
gone and only their values remain, indistinguishable from literals.

**`inputs.resolved`** publishes evaluated `assume` values. Worth having on the
page rather than only in the model source: in a deterministic run a random
assumption resolves to its clipped CENTRAL value, not to a draw, and publishing
it is what stops that being invisible.

**`ledger_hash`** is a SHA-256 over the deterministic ledger — the series and
the annual rollup. Together with `model_hash` and `engine` it closes the chain:
identical inputs on an identical engine must reproduce an identical ledger. A
golden diff can say "this document changed"; it cannot say whether that was a
real behavioural difference or a run-to-run wobble, and a wobble would surface
as a flapping test rather than as the defect it is.

It deliberately covers the ledger and not the metrics. NPV and IRR are folds OF
the ledger, so including them would make the hash move for a reason the ledger
did not — and it means `ledger_hash` is **invariant to the discount rate**,
which is correct: the ledger is cash before discounting. There is a test
asserting exactly that, alongside reproducibility and the fact that changing a
model's cash does move it.

The engine passes `stream_inputs` through as opaque JSON. `IrStream` is not
widened and the per-period evaluation path is untouched.

**No numbers move.** 116 goldens change: 1,384 IR `stream_inputs` leaves, the
same republished under `inputs.streams`, 72 `ledger_hash` values, 7 resolved
assumptions, plus the `results_version` bump and the 44 `model_hash` values that
follow the IR change. Zero numeric leaves differ.

### Added: stream categories

Every stream may now declare what it IS, economically, and aggregation reads
that rather than pattern-matching its name:

    stream cre.abatement.suite_200 on entity asset.rentleg outflow currency USD {
      schedule every year from 2001-01 to 2006-01
      category operating.deduction.abatement
      amount = ...
    }

A name is an address; a category is a meaning. Deciding that `cre.vacancy.loss`
is a deduction by reading its spelling means every metric, fold and statement
re-derives the same judgement independently — and they drift, which is exactly
how two `.*` selector dialects came to disagree.

**Why direction is not enough.** CRE emits seven outflow rules; three sit above
the NOI line (`ops.expense`, `vacancy.loss`, `property.opex`) and four below it
(`unit.ti_lc`, `rollover.ti_lc`, `construction.draws`, `permanent_debt_service`).
`direction` says "outflow" to all seven. The split already existed — as nine
hand-listed stream names in `domain.cre.noi`, restated in
`cre_exit_forward_noi_derived` and again in a benchmark's reference generator.
Categories do not add a concept; they move one to where it cannot drift.

**Categories are hierarchical paths, rooted in the cash flow statement.**
`operating.revenue.base_rent`, `investing.capital.leasing`,
`financing.debt_service`. Every system that solves this converged on the same
shape — IAS 7's three sections, a chart of accounts' five root types,
beancount's `Expenses:Rent:Office`, XBRL's calculation linkbase: a small
universal root, then an arbitrary domain tree, with the rollup defined by the
tree. So a subtotal is a prefix query over the selector streams already use —
NOI is `operating.*` — and a generic statement works against a pack it has
never seen.

CFDL enforces the root vocabulary and nothing below it. WHICH root a category
takes is the pack's call, because that genuinely varies: interest paid is
operating under IFRS and financing under US GAAP, and a lender's interest
*received* is operating revenue rather than financing at all.

All 58 lowering rules across the four packs are classified, and
`benchmarks/cre/mit_rentleg_plaza` now classifies its ten native streams —
including the abatement line that backlog 1.3 is about, which the pack has no
contract for and which a name-based selector could never have reached.

New diagnostic `E5022_UNKNOWN_STREAM_CATEGORY`. A pack whose vocabulary is not
rooted in a known section fails to load.

**No numbers move.** 81 goldens change: 169 added `category` fields, the 40
`model_hash` values that follow, and one parser message now advertising the new
item. Checked leaf by leaf — zero numeric values differ, and all 21 benchmarks
still reconcile.

The wasm budget moved 600 → 640 KB gzipped. It had been sitting at exactly
600/600, so the next addition of any kind was going to trip it; categories cost
~9 KB raw / 3 KB gzipped. Recorded in `build-wasm.sh`, along with the thing that
did *not* work: the pack TOMLs are `include_str!`-embedded so their comments do
ship, but cutting 2 KB of comment prose recovered 0 KB gzipped.

### Fixed: one selector dialect, and two metrics that were quietly wrong

There were two implementations of the `.*` stream selector and they disagreed
about whether it reaches the BARE name. That matters because a lowering rule
writing `energy.ppa.revenue{{contract.dot_suffix}}` emits the bare name for an
unsuffixed contract and `energy.ppa.revenue.plant_a` for a suffixed one, so a
selector reaching only one form silently drops the other — an absent stream
contributes 0 rather than raising.

Neither defect was caught, for the same reason: none of the affected fixtures
runs with `--pack`, so `domain_metrics` is absent from every golden that would
have shown them.

- **`domain.credit.wal_years` omitted unsuffixed pools.** `wal_years` matched
  `stream.<prefix>.` against series keys, which carry no `.total`, so a bare
  `credit.pool.prepay` failed the prefix test. It selects sched_principal,
  prepay, bullet and recoveries this way and goldens ship all four bare, so an
  unsuffixed pool reported a weighted average life computed over a subset of its
  own principal. (`sum` reached the bare name too — but only because its keys
  end in `.total`, which supplied the separating dot by coincidence rather than
  by decision.)
- **Every energy metric omitted suffixed contracts.** All fourteen selectors in
  `packs/energy/metrics.toml` named their stream exactly, while all ten energy
  lowering rules template the name. A suffixed PPA therefore contributed nothing
  to revenue, EBITDA, DSCR or tax benefits. Three goldens ship
  `energy.ppa.revenue.plant_a`, one carrying $29.9m.
- **`cre.exit_forward` double-counted an unsuffixed percentage rent.** Its
  forward-NOI expression summed both `cre.pct_rent` and `cre.pct_rent.*`, and
  the glob already includes the bare name — so the stream entered twice and
  inflated the exit price it strikes. Latent: every shipped model suffixes that
  contract.

Matching now lives in one place, `cfdl_expr::selector_matches`, and matches
NAMES rather than storage keys, so the key format cannot be load-bearing again.
`.*` reaches the bare name and its children both; the path-segment boundary is
unchanged, so `cre.pct_rent.*` still does not reach `cre.pct_rent_extra`.

**No shipped model's numbers move.** Ten goldens change — four IR expression
texts, the four `model_hash` values that follow, and 28 lineage selector
strings. Checked leaf by leaf: zero numeric values differ.

`check-pack-validations.py` gains a fourth check, because "pick the right
selector by reading the file" is exactly what failed here: a metric that names a
templated stream exactly is now rejected. Verified both ways — it reports all
fourteen energy selectors against the pre-fix file.

### Added: schema `--write`, warning codes in the gates, and a determinism lint

`check-results-schema.py` and `check-ir-schema.py` gained `--write`, which
regenerates the site mirror and the embedded docs block from the source schema.
Both gates could previously say the three copies disagreed but not make them
agree, so keeping them in step was a three-way paste — and `docs/06` is the copy
that fell four releases behind. The canonical serialisation now lives in one
place, `tools/schema_sync.py`, rather than being re-derived by hand.

`check-pack-validations.py`'s code-uniqueness checks matched an `E` followed by
digits, so warning codes were invisible to both of them: a `W3500` could be
added twice, or added without ever being documented, and nothing looked. Widened
to `[EWI]`, keyed on letter-plus-number so `E3500` and `W3500` stay distinct.

`clippy.toml` disallows `HashMap`/`HashSet`, making determinism in the numeric
path a property of the type rather than of anyone remembering. Float sums
reassociate, so a map with unspecified iteration order there would produce
results that differ between runs of the same model — and the golden suite would
report it as a flapping test rather than as the nondeterminism it is. `cfdl-lsp`
and one never-iterated map in `cfdl-calc` are exempt at the declaration, with
reasons.

### Added: `cre.permanent_debt`

A commercial mortgage on a stabilized property — the CRE pack previously had no
debt contract at all, so every model hand-wrote its mortgage and
`domain.cre.dscr` worked only because the metric matched a stream *name* by
convention.

    contract cre.permanent_debt on entity asset.tower {
      term 2026-01..2035-12
      terms { principal = 6000000  rate = 0.055  amort_months = 300 }
    }

`amort_months` strikes the payment and is normally longer than the term — the
30-year-amortization-on-a-10-year-loan structure is what a commercial mortgage
is. Optional interest-only period; the balloon is opt-in via
`balloon_at_maturity` and defaults OFF, because coverage is measured on periodic
debt service and the standard pro forma repays the balance from the sale.

One combined stream, `loan.permanent_debt_service`, matching the exact name the
metric selects. Diagnostics `E6050`–`E6056`.

### Added: `opco.exit_perpetuity`

Terminal value as a growing perpetuity — the Gordon form. The pack could
previously express only a *multiple* of something, so the largest single
component of value in a DCF had no contract.

    TV = base_value * (1 + growth_rate) / (discount_rate - growth_rate)

`base_value` is the terminal-period flow **before** the `(1 + g)` step; the
contract applies it. `discount_rate` is a contract term, not the run's NPV rate:
a terminal cost of capital is the rate for a business in steady state, and the
published models that state these terminals build it from their own CAPM inputs.
The run's rate discounts the result; this one capitalizes it.

Diagnostics `E7025`–`E7029`. `E7025` guards `r > g`, below which the perpetuity
has no finite value.

### Added: two externally reconciled benchmark cases

- **`benchmarks/cre/one_lincoln_street`** — a real named Boston development.
  Reconciles the construction period funding and interest schedule across
  sixteen quarters: equity and loan draws exact to the dollar, interest within
  the source's own thousand-rounding. The equity commitment depletes mid-quarter,
  and that split falls out of a declared state rather than being stated.
- **`benchmarks/opco/gordon_growth_coned`** — nine published values across nine
  growth rates, spanning a sign change. **The first case in this repo to assert
  a value** rather than a cash flow or a ratio.

Both sources are redistributable and are committed under `reference/`, bringing
to four the number of sources a reader can open and check directly.

### Note: adding a CRE contract and a CRE source did not improve CRE coverage

Externally reconciled pack-contract coverage moved opco 4/10 → 5/11 and left CRE
at 1/13 — worse as a ratio, since the denominator grew. `cre.permanent_debt`'s
only user is an in-house case, and One Lincoln Street's funding waterfall needs
a construction-loan contract that does not exist. Recorded as backlog 7.15
rather than left to be inferred from the numbers.


### Added: a state has its own schedule

A `state` now takes the same `schedule` clause a stream does:

    state pool_survival {
      schedule every quarter from 2026-01 to 2031-01
      init 1.0
      next prev * (1 - hazard)
    }

The recurrence STEPS on that cadence and HOLDS between ticks and outside its
window. It does not fall to zero — that is what separates a schedule from
`active when`, which a state deliberately does not have.

This corrects the original design. `docs/14_state_and_recurrence.md` said "a
state has no schedule", conflating cadence with activity and dropping both, so
every state advanced once per MODEL period. Since a lowering rule's
`{{time.elapsed_periods}}` counts its own PAYMENT periods, a pool on a daily
book paying monthly would have compounded 365 times a year instead of 12. §8 of
that document records the correction.

Absent, a state steps every model period over the whole timeline, so nothing
already written changes. Pack rules gain `state_every` / `state_from` /
`state_to`.

### Added: PSA, SDA and the ABS prepayment model in the credit pack

`psa_speed`, `sda_speed` and `abs_speed`, each a MULTIPLE of the published
curve, plus `age_months` for a pool's seasoning at closing. All default to `0`,
selecting the existing flat `cpr`/`cdr` path.

The pool factor is now a per-period state rather than `pow(k, p)` — the closed
form of the running product only while the hazard is constant. Three externally
reconciled cases were blocked on this and now land:

  - `benchmarks/credit/auto_abs_speed_050`   0.0048 percentage points
  - `benchmarks/credit/auto_abs_speed_150`   0.0036 percentage points
  - `benchmarks/credit/mbs_pool_ramped`      within the source's rounding floor

New diagnostics `E9016`–`E9019`. Closes backlog 2.1.

Two convention defects were found by those external references after every
in-house identity already passed: all three ramps index from loan ORIGINATION
rather than the deal's closing (20 percentage points on a seasoned pool at
1.50% ABS), and the lagged pool factor the recoveries rules read was consuming
the hazard one lag too late (7.6% on recoveries by month 60). Both are recorded
in the cases' NOTES.

### Added: `make rule-fragments`

`tools/check-rule-fragments.py` asserts that repeated expression fragments in a
pack's lowering rules are byte-identical, normalising the age argument. Every
committed golden runs at a constant hazard, so nothing in the suite evaluates a
ramp branch; a typo in one of eighteen copies is invisible to it. Measured: a
10x typo in a shared `state_next` is caught by `E5021`, but the same typo in one
rule's `amount_expr` passes gold, benchmarks and analytic checks.

### Changed: pool factors are no longer decimal-exact

`pow(k, p)` was one decimal exponentiation; a state is `p` sequential
multiplications stored as `f64`. Measured at 4.6e-16 relative over 360 periods,
which publication rounding at six decimals absorbs — no committed golden moved.
Recorded because it is a real, if tiny, loss of exactness.


### Added: declared state variables

A `state` is a named number per period defined by a recurrence — the one shape
`pow(1 + r, t)` cannot express, since that applies a single period's rate as
though it had held from the start:

    state revenue_index {
      init  1.0
      next  prev * (1 + curve_value("growth", time.date))
    }

    stream firm.revenue on entity legal.firm inflow currency USD {
      schedule every year from 2026-01 to 2035-01
      amount = 21765.4 * state.revenue_index
    }

Language-level, not pack-level: a state has no entity, direction, currency or
schedule, and any model may declare one regardless of which pack it uses (or
none). Inside `next`, bare `prev` is this state's previous value and
`prev.<name>` is another's.

`init` is mandatory. An unstated base case would evaluate as a silent zero for
every period, since an unmatched lookup returns 0.

The safety property is preserved by ABSENCE rather than by a check: a `next`
environment carries no `state` map and a stream environment carries no `prev`
map, so a same-period read is not there to be found. Everything a state can see
is already finished, so no reference can close a cycle — "cycles are impossible
by construction" is intact, and states may reference each other mutually with
declaration order carrying no meaning.

Six diagnostics, `E1120`–`E1125`, each probed against a fixture that violates it.

States are published in `results.deterministic.series` as `state.<name>`, as
bare numbers with no currency and no offset. They are **not cash**: excluded
from `model_series`, `model.total`, `model.npv`, the annual rollup and every
domain metric, with an analytic identity asserting it.

Pack lowering rules may declare a state too (`state_name`, `state_init`,
`state_next`, plus a `{{contract.suffix_ident}}` placeholder), with
`E5020_LOWERED_STATE_INVALID` and `E5021_DUPLICATE_LOWERED_STATE`. The three
opco growth rules now compound through a running product; no model needed
editing.

Verified against two independent published sources:

  - the FCFF forecast: revenue drifted -2.4% by year 10 and years 6-10 were
    unasserted; now all ten years agree to floating-point noise
  - the HUD multifamily pro forma: a 12.26 residual under `period_tolerance = 13`
    is now exact, with the tolerance at 0.5 — the whole-dollar rounding floor

Across all 110 goldens the only movement from the pack change is
`7365967.000481 -> 7365967.00048` (1.4e-13 relative) and a `-0.0 -> 0.0`.

Closes backlog 5.1 and 7.2; supersedes most of 7.8. See
`docs/14_state_and_recurrence.md`.

### Changed: `Series.values` may hold a bare number

`MoneySeries` is renamed `Series`, and its `values` becomes
`Money | number` — cash carries a currency, a state does not. The results
schema always permitted a number here, so no published shape changed.

Consumers that weight or sum cash take `SeriesValue::money_amount()`, which
returns `None` for a non-money series.

### Added: `ln` and `exp`

Two builtins that turn a cumulative **product** into a cumulative **sum**:

    PROD(1 + r_i)  ==  exp(series_sum("ln_one_plus_r", 0, t))

A survival factor or growth path under a *varying* rate has no closed form, and
`pow(1 + r, t)` is not it — that applies one period's rate as though it had held
throughout. Verified end to end: the identity reproduces all ten years of a
published forecast with a decaying growth rate exactly, where `pow` drifts to
-2.4% by year 10.

Both escape to float64, as `pow` already does for fractional exponents, so they
are **not decimal-exact**. Prefer a closed form where one exists.

Note the technique is not yet usable from a pack rule: the helper stream
carrying `ln(1 + r_t)` is counted as cash. See backlog 7.8.


### Breaking: three diagnostic codes renumbered

A diagnostic code is an identifier — what a user greps for and what a tool
matches on. Three named two different checks each:

| was | is now | check |
|---|---|---|
| `E7010` | **`E7013`** | `OPCO_WC_MISSING_AMOUNT_OR_RULE` |
| `E7011` | **`E7014`** | `OPCO_WC_INVALID_SCHEDULE` |
| `E6030` | **`E6033`** | `CRE_UNIT_INVALID_ESCALATION` |

The ambiguity checks keep `E7010`, `E7011` and `E6030`; they form a family and
are documented as such. Anyone matching on the three old codes for the
working-capital or unit-escalation meanings needs to update.

### Two thirds of pack validations were never running

A validation matches a contract by exact name unless it declares
`match = "instance"`, and contracts are routinely written suffixed
(`opco.revenue_line.core`). 33 of 48 shipped validations lacked the flag and
were silently skipped on the form models actually use — `E7001` rejected
`opco.revenue_line` with no amount and accepted `opco.revenue_line.core` with no
amount.

All 48 now declare it. No golden moved: eight previously-dormant checks are live
and every shipped model already satisfied them.

`tools/check-pack-validations.py` joins `make ci`, enforcing that codes are
unique **and** that every validation states its match mode explicitly. `exact`
remains available; it just has to be written, because defaulting was the trap.


### Breaking: WAL and payback are measured on the discounting time axis

`model.wal_years`, `domain.credit.wal_years` and `model.payback_years` weighted
a period-0 cash flow at **t = 0**. The market convention — the one a prospectus
states as "the number of years from the closing date to the related
distribution date" — puts an ordinary annuity's first monthly collection at
1/12 of a year. Credit models put their first collection in period 0, so every
WAL this engine has ever reported was one period short.

Reconstructed from an issuer-published auto-ABS schedule, the effect is not
academic: a class with a published WAL of 0.37 years came out at 0.286, a 23%
understatement. Short amortizing deals are hit hardest, because one period is a
larger share of a shorter life.

A flow's time is now `(period + offset) / ppy`, where `offset` is the same
placement `npv_with_offsets` discounts on (`docs/12_payment_timing.md`). So NPV,
IRR, WAL and payback now agree about when a dollar arrived. Consequences:

- a bullet's WAL is exactly its term (it reported term − 1 period);
- an annuity due's WAL is exactly one period shorter than the equivalent
  ordinary annuity's (they were identical);
- `mid` sits exactly halfway between the two (it was indistinguishable);
- the same deal has the same WAL on any calendar (an annual grid was a full
  year out).

All four, plus a payback identity, are now asserted in
`tools/analytic-checks.py` — they fail on the previous engine and pass on this
one. Nothing else could have caught this: the three credit benchmarks asserted
WAL against reference generators that restated the same off-by-one, so both
sides agreed for as long as they existed. The generators are fixed here
independently of the engine, and their agreement afterwards is the check.

Time-weighted metrics now net **within** an offset rather than across one: two
flows in one period at different points in it are not the same cash at the same
moment, so a purchase settling on its date no longer cancels that period's
collections. Where every stream shares a placement this is exactly the previous
behavior. `model.moic` is deliberately unchanged — it is a ratio of cash in to
cash out and does not depend on when the cash moved.

Numbers that move:

| benchmark | `model.wal_years` | `domain.credit.wal_years` |
|---|---|---|
| `credit/level_pay_pool` | 3.817027 → 3.843940 | 3.973633 → 4.056967 |
| `credit/io_bullet_loan` | 3.812188 → 3.864922 | 4.244941 → 4.328274 |
| `credit/float_bridge_pool` | 2.313942 → 2.367044 | 2.456847 → 2.540180 |

The domain metric moves by exactly 1/12; `model.wal_years` moves by less,
because period 0's collections were being annihilated by the purchase and now
re-enter the denominator at 1/12 year. 56 goldens move `model.wal_years` and 16
move `model.payback_years`; no golden gains or loses a metric key.

### The published results schema is a gate, and was wrong

Every one of the 67 committed results goldens violated
`docs/schemas/results.schema.json`, and had since 0.3.0 — four releases:

- `results_version` declared `const "0.1"` while the engine has emitted `"0.2"`
  since 0.3.0. The one field whose entire job is to say which shape a document
  has was itself wrong in every document;
- `deterministic.annual_rollup` was emitted by 62 goldens and undeclared;
- the root-level `domain_metrics` was emitted by 8 and undeclared.

Fixed, and gated: `tools/check-results-schema.py` joins `make ci`, the sibling
`check-ir-schema.py` has had since the IR schema drifted the same way.

`docs/06_results_schema.md` was an independently maintained copy of the same
JSON and had drifted further. It is now generated from the schema, and the gate
checks all three copies agree — the site mirror, the doc page, and the source of
truth. Three copies of one contract, only ever one of them read, is how this
happened in the first place.

### CI ran five fewer gates than `make ci`

`bench`, `analytic`, `cadence-parity`, `ir-schema` and `results-schema` existed
only locally, so they fired when someone remembered. That is how the weighted
average life defect above survived — the identity that catches it lives in
`analytic-checks`, which the workflow never executed. All five now run in CI.

### The compiled Python extension has a freshness gate

`cfdl_sdk` is half editable Python and half a compiled Rust extension. The
Python half tracked the working tree; the compiled half was rebuilt only on
`make py-develop` and nothing said when it had gone stale. It went stale, and
`make notebooks-render` failed with

    E4004_MISSING_PACK: unknown variant `terms_mutually_exclusive`

which reads like a broken pack and was nothing of the sort — the extension
predated the commit that added that validation kind. `tools/py-stamp.py` hashes
the sources the extension is built from, `make py-develop` stamps it, and
`notebooks-render` / `notebooks-check` check it first and name the remedy.

A source hash rather than a version check, for the same reason the wasm bundle
uses one: the commit that broke this shipped no version bump, and it changed a
pack TOML rather than any Rust source.

### Added

- `MoneySeries.offset` in the results document — a series' placement in its
  period, published so a consumer holding `results.json` can recompute the
  time-weighted metrics the engine reported. Optional and additive; absent on
  aggregates, which sum streams whose placements differ.

---

## [0.7.0] - 2026-07-28

Schedules, contract terms and the published surface. Breaking: see below.

### Schedules honour what they declare

A stream's recurrence interval was discarded at parse time, so every stream
paid in every period. This release completes the fix end to end.

- An interval finer than the model's calendar is rejected
  (`E2108_SCHEDULE_FINER_THAN_CALENDAR`) rather than collapsed. A weekly
  schedule on a monthly grid paid twelve times a year instead of about
  fifty-two: several occurrences fall in one period, and a period holds one
  payment. This is section 10.3's own rule, finally implemented.
- A lowering rule may declare `schedule_every`, so a pack can express a
  quarterly coupon or an annual true-up rather than being pinned to the
  calendar cadence. Unset means the cadence, which is every shipped rule.
- `stub` is rejected instead of accepted and discarded — a model could ask
  for a short front stub and silently receive a full period.
- The doc-examples gate counts payments: a stream may not pay in more periods
  than its schedule declares. That is the check that would have caught the
  original defect.

### Contract terms

A term is a literal or a reference to one declared input. Trailing tokens are
rejected: `rent_year = 12 * 8500` compiled as `12`, silently, in any pack.

A term naming an input defers to it, so scenarios and Monte Carlo drive it
through the one channel they already write to. Terms were previously baked
into lowered expressions as literals, so a Monte Carlo run sampled a variable
the expression did not contain and returned a degenerate distribution with no
warning.

### Currencies

`model "x" currency INR` parses, and every metric reports in it. Pack
lowering rules no longer hardcode USD — a PPA in Rajasthan is not a USD
contract — and a stream whose currency differs from the model's is rejected
rather than summed as though the units matched.

### The published surface describes the language that exists

The EBNF splits `cadence` from `interval`, documents `due`, and drops the
`stub` and weekday productions nothing implements.

The IR schema was public at cfdl.dev/schemas and checked against nothing. It
listed `metrics` as required though no compiler emits it, declared `stub` and
weekday rules that are never produced, and used `oneOf` for a union whose
members overlap, which could never be satisfied. `tools/check-ir-schema.py`
validates every IR golden against it and is part of `make ci`; it immediately
caught an `on eom` rule emitting `day: 0` against its own 1..31 bound.

The pack manifest documentation described a format the loader never read — a
pack written to it would have loaded with no entrypoints at all.

### Tooling

- The four standard packs are built into the CLI, so `cfdl compile my-model`
  resolves `use pack` with no flag and no download. A packs directory that
  holds packs stays authoritative.
- A pack present at a different version says so, naming both versions,
  instead of reporting "not found".
- `cfdl validate` applies the same `./packs` default as compile and run.
- Object ids no longer depend on the compiler version. Every release rewrote
  every id, churning goldens and making a downstream store treat the same
  entity as new after an upgrade. Ids move once here and should not again.
- `run.json` gained a JSON Schema, all five distributions, `clip`, and
  rejection of unknown keys — which found twelve example configs running
  undiscounted while claiming 0.1.

### Breaking

- `stub`, schedules finer than the calendar, mixed-currency models, and terms
  with trailing tokens no longer compile.
- Schedule intervals are singular nouns: `every month`, not `every monthly`.
- Object ids change once, as described above.
- A pack rule pinning a currency the model does not declare is rejected.

---

## [0.6.0] - 2026-07-28

Packs work outside the United States. Breaking: see below.

### Lowering rules inherit the model's currency

All 58 lowering rules across the four packs hardcoded `currency = "USD"`, and
the compiler fell back to the model's currency only when a rule left the field
empty. An INR model using the energy pack therefore reported INR metrics over
USD-labeled streams — a PPA in Rajasthan is not a USD contract.

Nothing caught it. `E2107_STREAM_CURRENCY_MISMATCH` lives in `cfdl-validate`,
which runs on the AST and so sees only hand-written streams; pack-lowered
streams are generated afterwards and bypassed it. The guarantee 0.5.0 made —
that currencies cannot be silently mixed — held for hand-written models and
not for pack-based ones, which is every serious model. The check now also runs
where lowered streams are built.

Rules omit `currency` rather than defaulting it to USD, because the default
already exists one level up: an unset rule currency takes the model's, and a
model that declares none takes USD. Two defaults would shadow each other and
reinstate the bug. An empty value is a deferral, not a missing value — the same
shape as a term deferring to a declared input. Pin a currency only when the
instrument is genuinely fixed to one, and the model must then agree.

No golden moved, which is the check that the fallback is wired correctly:
every model in the repository is USD, so the inherited value is identical.

### The packs archive ships what the docs promise

`package_packs.sh` archived only `cre` and `opco` while the install page
promised all four, so the flagship energy pack was undownloadable for anyone
without a checkout. It now discovers packs by their manifests rather than
listing them, so a new pack ships automatically.

`verify_release_assets.py` previously checked only that the archive's filename
existed. It now looks inside — a tarball missing half its packs passed three
releases undetected.

### Breaking

- A pack lowering rule that pins a currency the model does not declare is
  rejected with `E2107_STREAM_CURRENCY_MISMATCH`. No shipped rule pins one, so
  this affects third-party packs only.
- `LoweringRule.currency` is optional. Packs that omit it now inherit the
  model's currency instead of failing to parse.

---

## [0.5.2] - 2026-07-28

Release-pipeline fixes. No behavior change: the compiler, engine and packs
are identical to 0.5.0.

- `distribution/scripts/package_docs.sh` named three documents that were
  renamed in the docs restructure, so `tar` exited non-zero and the docs
  archive failed to build on every tagged release. It now archives the docs
  tree wholesale, which cannot drift as files are renamed.
- The server image failed at `cargo build -p cfdl-server`:
  `utoipa-swagger-ui`'s build script downloads the Swagger UI bundle at
  compile time and shells out to `curl` when its reqwest feature is off, and
  `rust:1-slim` ships neither `curl` nor CA certificates. Both are now
  installed in the builder stage.
- Adds a `.dockerignore`. The image builds from the repository root with
  `COPY . .` and had no ignore file, so `target/`, `node_modules/` and `.git`
  were all being sent as build context.

Together with 0.5.1 this makes the full release pipeline green for the first
time — the VS Code extension lockfile, the docs archive and the server image
had each been failing independently since v0.3.0 or earlier.

---

## [0.5.1] - 2026-07-28

Release-pipeline fixes. No behavior change: the compiler, engine and packs
are identical to 0.5.0.

- The VS Code extension's `package-lock.json` still declared `0.0.1` while
  `package.json` tracks the project version, so `npm ci` refused to install
  and the Extension lint step failed on every tagged release from v0.3.0
  onward. The lockfile now carries the real version and is bumped with it.
- Playground examples were stale against the repo's models: the schedule
  syntax migration in 0.4.0 changed the `.cfdl` sources without regenerating
  them, and the site workflow had been failing on `main` as a result.
- Monte Carlo dispersion is asserted as a property in
  `tools/analytic-checks.py` rather than as a golden. A long run over a
  pack-lowered expression containing `pow()` is not bit-identical across
  platforms, so it passed locally and failed on Windows CI. The golden keeps
  its deterministic scenario sweep.

---

## [0.5.0] - 2026-07-28

Contract terms, stochastic layering, and currencies. Breaking: see below.

### Contract terms are a literal or one declared input

A term kept only the first token after `=` and silently discarded the rest, so
`rent_year = 12 * 8500` compiled as `12` — no diagnostic, and no validation
caught it because `12` parses cleanly. That is now an error
(`E0004_EXPECTED_TOKEN`), and a term is defined as either a literal or a
reference to one declared input:

```cfdl
assume annual_yield ~ Normal(mean=5000, stdev=350, clip=[4000, 6000])

terms {
  ppa_price = 3000                 // contractual fact
  mwh_year  = inputs.annual_yield  // driver, supplied per run
}
```

Contracts stay declarative records of what was signed; anything that varies is
named and supplied from outside. Because `inputs.*` is the single channel that
scenarios and Monte Carlo already write to, one declaration serves a fixed
case, a scenario sweep and a stochastic run alike.

This also fixes Monte Carlo through pack contracts. Terms were baked into
lowered expressions as literals, so a trial sampled a variable the expression
did not contain and returned a degenerate distribution with no warning.

- `E5010_TERM_UNKNOWN_INPUT` — a term naming an input that was never declared.
- `E5011_TERM_CLIP_OUT_OF_BOUNDS` — a deferred term's value cannot be checked
  at compile time, but its distribution's `clip` states the range it can reach,
  so where a pack declares bounds the clip is checked against them.
- `E5009_LOWERED_EXPR_INVALID` — pack-lowered amount expressions are now
  compile-checked. The engine evaluates a failed expression as zero with only a
  warning, so a malformed expansion became a silently empty stream.

### Model currency

`model "x" currency INR` now parses; every metric is denominated in it, and it
defaults to USD when omitted. Streams must agree with it: cash flows are summed
period by period, so a 500 USD outflow in an INR model was being subtracted as
500 INR, producing a total in no currency at all
(`E2107_STREAM_CURRENCY_MISMATCH`). Cross-currency models require an explicit
conversion in the amount expression — the language applies no implicit FX.

### Run configuration

- All five distributions (`fixed`, `normal`, `uniform`, `log_normal`,
  `triangular`) and `clip` now work from `run.json`, matching what
  `assume x ~ Dist(...)` offers. `stdev` is accepted alongside `stddev`.
- Unknown keys are rejected. Parsing was lenient and the override consumers
  ignore unrecognized keys, so a misspelling produced a clean run with wrong
  numbers and no warning.
- `docs/schemas/run.schema.json` — the format had no schema at all.
- An in-source `run monte_carlo trials N seed S` is honoured. It was parsed and
  lowered, then dropped by the engine, so a model asked for trials and got a
  single deterministic pass. An explicit run config still wins.

### Breaking

- Terms with trailing tokens, mixed-currency models, and unknown run-config
  keys now fail to compile or run.
- Twelve example run configs set `discount_rate`, which is not the wire name
  (`annual_discount_rate`) and was therefore ignored — those examples ran
  undiscounted while claiming 0.1. Migrated rather than aliased, so the
  correction is visible; their numbers change.

---

## [0.4.0] - 2026-07-28

Payment timing. Breaking: discounted metrics change for every model.

### Schedules honour the declared interval

A stream's recurrence interval was discarded — the parser dropped the token and
the compiler substituted the model's calendar frequency — so every stream paid
in every period. A model written `every quarterly` on a monthly grid paid twelve
times a year, silently. Intervals are now parsed, required, and honoured.

Interval and cadence became separate words because they are separate concepts:
a calendar is adjectival and describes the grid (`time calendar monthly`); an
interval is a noun and describes how far apart one stream's payments fall
(`every month`). Only intervals have a weekly member.

`on day <n>` and `on eom` work for the first time. The compiler had always
emitted the rule; the engine had no field for it and dropped it on
deserialization.

### Payment timing is specified and discounted correctly

A payment belongs to the period that earned it. What separates the two annuity
conventions is where in that period the cash falls, and therefore how far it is
discounted — one mechanism rather than three special cases:

| Schedule | Position | Discounted from |
|---|---|---|
| `due` | start | period start |
| default, `on eom` | end | period end |
| `on day <n>` | day n | that point in the period |

This is Excel's convention, matching `pmt(rate, nper, pv, [fv], [due])` in the
expression library. Mid-period discounting follows from the same rule.

Written honestly, a five-year par bond now returns an NPV of exactly zero — the
identity that exposed the defect, since the first coupon previously landed
undiscounted and the final year fell off the end of the range.

See `docs/12_payment_timing.md`.

### Verification against closed-form finance

`tools/analytic-checks.py` asserts identities drawn from the definition of
present value, so they hold for any correct implementation and cannot be
satisfied by making two implementations agree: a par bond is worth par, a level
annuity matches `(1-(1+i)^-n)/i`, an annuity due is worth `(1+i)` times the
ordinary annuity, and a fully-amortizing loan is worth its principal. Part of
`make ci`.

The benchmark suite compares each model against a reference implementation,
which cannot detect a convention both sides share — that is how the original
defect survived eight passing benchmarks. Every reference was corrected to
separate one-shot flows from recurring ones.

### Breaking

- Discounted metrics (NPV, IRR, and anything derived) change for every model.
  Undiscounted cash flows are unchanged for models scheduling at their calendar
  frequency, which was every model in the repository.
- Schedule intervals are spelled as singular nouns: `every month`, not
  `every monthly`. The interval is now required after `every`.

---

## [0.3.0] - 2026-07-27

First public release. CFDL is pre-1.0: the language and IR spec is v0.1, and
interfaces may change until 1.0 freezes the IR and Results schemas.

### Language and engine

- Deterministic compilation: the same sources, pack version and compiler
  version emit byte-identical IR, enforced by a golden suite.
- Native `cfdl-calc` expression engine with decimal-exact money arithmetic and
  an Excel-compatible function library (annuities, day counts, business-day
  calendars, MACRS, prepayment conversions).
- Deterministic DCF, scenarios, and seeded Monte Carlo, emitting
  schema-governed Results JSON.

### Domain packs

- `energy`, `cre`, `credit` and `opco`, each supplying contract types,
  template-driven lowering rules, domain metrics, and declarative validations.
- Every pack is gated by a parity suite: each model is diffed period-by-period
  against an independent reference implementation.

### Surfaces

- CLI (`cfdl compile`, `cfdl run`, `cfdl validate`).
- Python SDK (`cfdl_sdk`) with pandas result accessors.
- WebAssembly build powering the in-browser playground at cfdl.dev.
- HTTP API server, and a VS Code extension with LSP diagnostics.

### Licensing

- Business Source License 1.1 (source available, not open source). Each
  released version converts to Apache-2.0 four years after its release.
