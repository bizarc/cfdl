# 29 — The Period Walk: Implementation Plan

Status: proposed. The plan that realizes `docs/28`. Phases are ordered by
dependency; each lands as its own PR with `make ci` green, and no phase
starts before the gate of the phase it depends on. Section references are
into `docs/28` unless said otherwise.

---

## Phase 0 — the gates (pre-work; blocks everything)

**0.1 Loud failure for unbindable series reads.** The three inert spellings
of `docs/13` §7.71 — each of which today warns per period and publishes wrong
numbers under `status: ok` — become refusals: `series_sum`/`series_avg` in
an event guard or a field recurrence is refused at compile where the
validator can see it, and at IR load otherwise, with the read named — the
same family as the bare-path `E5002`. New diagnostics, registered in the
code register (both directions, per the 7.11 gate). The three probe models
from §7.71 land as `fixtures/invalid/`, which is the test.

- Files: `crates/cfdl-compile` (validator), `crates/cfdl-engine/src/env.rs`
  (load-time backstop), `docs/08` (diagnostics).
- Gate: the probes fail loudly; full suite green.

**0.2 A Rust-side guard for the engine — the gate phase 2 actually needs.**

*Revised. This was specified as "mutation testing over the engine, with a
blessed baseline". That was the wrong instrument, and the revision records
why rather than quietly dropping it.*

The concern behind it is real: phase 2's collapse property says every blessed
number is unchanged, and "the goldens pass" only means that if the goldens
would notice. But mutation testing is an expensive way to ask that question —
860 mutants in `cfdl-engine`, hours of wall-clock, and a 2.4 GB tree copy per
parallel job, which does not fit on the machines this project is built on. It
also answers only whether the TESTS notice a change; the benchmarks tied to
published figures answer whether the NUMBERS are right, which matters more and
which mutation testing cannot do.

What phase 2 needs, and what this phase delivers, is the guard itself:
`crates/cfdl-engine/tests/golden_corpus.rs` runs all 108 blessed fixtures
in-process — `gold/ir` in, `gold/results` byte-compared — in about three
seconds, inside `make ci`. Before it, `cargo test -p cfdl-engine` was 27 unit
tests over a 2,200-line engine in 0.01 seconds, and the engine's real suite
was reachable only from a shell script. That is the durable deliverable, and
it makes the collapse property checkable on every commit rather than once.

**What one exploratory run found, before the tooling was removed.** It is not
in the repository and is not runnable here — a 2.4 GB tree copy per parallel
job does not fit these machines — but three real gaps it surfaced are now
closed by fixtures and tests, which is where the value belongs:

- the annual valuation grain's dispatch had no end-to-end coverage, though its
  arithmetic was unit-tested → `valid/valuation_grain_annual`;
- the model-declared `run monte_carlo` path had none, because every Monte
  Carlo fixture supplied the mode in `run.json` →
  `valid/run_declared_monte_carlo`;
- `trials 0` was accepted by a parser whose own message said positive, so a
  model could ask for Monte Carlo, compile, run, and publish no Monte Carlo
  section → `invalid/run_monte_carlo_zero_trials`, plus a unit test for the
  engine's IR-level guard, which no model can reach any more.

The lesson worth keeping is not the technique but the question it asked: when
a change's success criterion is the ABSENCE of a difference, check that
something would have noticed. The cheap way to do that during phase 2 is by
hand and per hypothesis — break the reorder deliberately in the specific way
you fear (evaluate events after streams; put the lag off by one) and confirm a
golden fails. That is minutes, tests exactly the risk, and needs no tooling.

- Gate: `cargo test --all` covers the engine against the whole blessed corpus,
  and `make ci` runs it.

## Phase 1 — the journal (independent of the walk; may ship first)

Extend `deterministic.transitions` into the journal of §8: event firings
with the values their guards read; every action with its outcome (applied /
declined / overridden — a stream activation losing to `active when` is
recorded); waterfall allocations per step with pot before and after; option
elections. Additive keys in the results schema, so existing consumers keep
working; goldens re-blessed deliberately for the new keys, nothing else
moving. Monte Carlo summarises the same record — one row per act with the
share of trials it occurred in and the distribution over the period it FIRST
did, rather than the per-trial log §7.18 rules out — which closes `docs/13`
§7.18.

**Shipped.** `deterministic.journal` and `monte_carlo.journal`, with
`fixtures/valid/journal_action_outcomes` pinning five outcomes and
`fixtures/valid/monte_carlo_journal` pinning both halves of the summary. The
sixth outcome, `ignored`, is unreachable from a model — see §7.73, which
recommends retiring the action rather than building its runtime.

- Files: `crates/cfdl-engine/src/{state,streams,distributions,results}.rs`,
  `docs/06` and `docs/schemas` (results schema), `tools/check-results-schema`.
- Gate: a golden asserts on the journal itself; the schema gate validates
  it; all numeric series byte-identical.

## Phase 2 — the walk (the reorder; blocks all new expressiveness)

**2.0 What actually reads forward — measured, and then measured again.**

A period walk cannot serve a forward read: at period 3 there is no period 24.
So the first question is how much of the corpus reads ahead of itself.

Asked first of model SOURCE, the answer looked like two benchmarks — the
forward-income exit in `penzance_one_rosslyn` and the expense stop's base year
in `mit_rentleg_plaza`, exactly the two constructs `docs/28` §7 migrates. Asked
of the compiled IR, by `cfdl_engine::walk_eligibility` over the blessed corpus,
it is **five fixtures and two causes** — and the difference is the finding:

| cause | models | window |
| --- | --- | --- |
| the CRE pack's `cre.exit_forward` lowering | `cre_office_two_tenant`, `pack_cadence_cre_{annual,monthly,quarterly}` | `[time.t + 1 .. time.t + 12]` |
| an absolute base year | `cre_derived_lines` | `cre.opex.line[24..24]` |

A third cause was found and removed rather than recorded: `waterfall_nested_split`
read an absolute window from a waterfall pot, which `docs/17` §4 forbids. See
below.

**The forward-income exit is a PACK CONTRACT, not a benchmark.** A scan of
model source cannot see it, because the read lives in the pack's lowering rule
rather than in anything a modeller wrote — so every model using
`cre.exit_forward` reads forward, present and future. That widens `docs/28`
§7's migration from "one benchmark" to "one pack contract, and every model that
uses it", which is a different piece of work and is why it is written down here
rather than discovered in phase 6.

**A waterfall never reads forward, and the reference says so.** `docs/17` §4:
a waterfall "reads period-close state, because the pot it allocates is THIS
PERIOD'S cash and the balances it measures are this period's balances." The
schema does not enforce it — `Waterfall.source` and `WaterfallStep.amount` are
plain `Expr` — and the engine cannot detect it, because by the time the
distribution stage runs the whole series column exists, so a forward window
succeeds silently. `waterfall_nested_split` had a pot written `[0..5]`, an
absolute window, where the constant was its single distribution date's period
index; it now reads `[0..time.t]` and says what it means, with no published
number moved. The walk closes the gap by construction: under a period walk a
forward pot read is impossible rather than merely disallowed.

A waterfall's pot and steps do have to be CHECKED, though, and they are not in
`ir.streams` — a streams-only eligibility check called that fixture walkable.

**A cumulative window is not a forward read.** `[0..time.t]` — the Highlands
pot, the auto-ABS cumulative prepayment — reads only what has already
happened, and a walk serves it exactly. Two shapes had to be taught before
that came out right, and both were reported as forward until they were:
the compiler normalises `0` to `0.0`, and the OpCo pack writes a
trailing-twelve-month window as `time.t - 12.0 + 1.0`, which is `t - 11`.
Reading only the first term refused the walk for every LBO model in the corpus.

The classifier is conservative by construction: a shape it does not recognise
is forward, because a walk that guesses wrong reads a cell that does not exist
yet. A positive literal is forward for the same reason — `24` is behind `t`
from period 25 and ahead of it before, and static analysis cannot know which.

`only_the_known_models_read_forward` in `golden_corpus.rs` pins the set, so
this table cannot quietly go stale and a new forward-reading model cannot
arrive unnoticed.

**2.1 Dependency extraction over guards and recurrences.** The existing
`series_references`/`selector_matches` machinery runs over event guards and
field `next` expressions, producing the cell-level edges of §4.

**2.2 The schedule.** A static builder classifies the model: subgraphs with
no cash-into-logic edge evaluate as columns (today's waves, unchanged);
coupled subgraphs get the per-period interleave. Cross-time cycles are
refused with the path named. The schedule is computed once and replayed per
scenario and per trial.

**2.3 Two halves, and only one of them was real.**

This section asked for two things and read as one. The correctness half is
built; the performance half is not, and the measurement says it should not be.

**BUILT — a cell that has not been computed is not a zero.** Under a walk the
columns are allocated for the whole grid and filled as it advances, so a read
reaching past the current period finds the zero the column was allocated with:
a plausible number with nothing to see, which is exactly the defect `E1134`
refuses one layer out. `ExprEnv::series_available_to` marks how far the columns
are filled — `None` for the column order, whose columns are finished, and
`Some(t)` under the walk — and a window reaching past it is refused rather
than clamped. Clamping is what would make a forward read look like a small
number instead of a mistake. Three tests in `cfdl-expr` pin it: refused past
the watermark, served within it, and unchanged when there is no watermark.

**NOT BUILT — the store.**

*Revised twice, and the second revision is the measurement. This asked for
series storage representing a partially built column, so the walk would not
copy the store per (period, wave). The copying is real; its cost is not.*

**The controlled measurement.** One model, 5,000 trials, run twice: once as
written, once with its cross-stream reads replaced by a constant so no snapshot
is ever consulted. **10.78s against 10.74s.** The snapshot copying — the whole
justification for a store — is inside the noise.

An earlier comparison appeared to show cross-stream reads costing 12x. It was
confounded: the two models differed in size (6 periods x 1 stream against
36 x 4), so it measured cells, not copying. Recorded because the wrong number
was persuasive.

**Where the time actually goes.** Dropping 4 streams to 1 takes 10.74s to
5.66s, so about 9.4 microseconds per accrual and roughly 63% of the run in the
per-accrual path; the remaining ~4s is per-trial work outside it. The suspect
in that path is `build_expr_env`, which constructs the WHOLE environment for
one stream at one period: it re-parses every curve point's date string,
rebuilds every quantile, re-clones every input name, and re-scans every
parameter override — then throws it away.

**And the deeper point is not that this is repeated, but that it is
UNCONDITIONAL.** A stream whose amount is `5000` needs no curves, no
quantiles and no inputs, and is handed all of them. What an expression
references is already known — it is how the wave graph is built — so a plan
can carry an environment holding only the roots its stream actually names,
built once. That is a different and better fix from hoisting invariants,
because it makes the cost proportional to what a model uses rather than to
what it declares.

**None of this is built, and none of it is blocking.** The walk is not on the
production path, so nothing is slow because of it. The note exists so the
question is not reasoned about a fourth time: the store is not the cost, the
environment is, and the fix is a per-stream minimal environment rather than a
new storage type.

**2.4 The fold — built.**

The period loop of `docs/28` §3 exists and is driven: `walk_periods` settles
state for period `t`, then evaluates that period's streams against it, then
advances. Three refactors made it possible and none of them changed a number:

* `evaluate_stream` split into `plan_stream` and `StreamPlan::step`;
* the state stage split into `prepare_state_walk`, `StateWalk::step` and
  `StateWalk::finish`, so it can be advanced a period at a time rather than
  run to completion first;
* a stream stopped taking a finished `EventSim` and started taking the two
  pieces it reads — the settled entity state and the stream-active mask — so a
  walk can supply them while the state stage is still advancing and the column
  order can supply them from a completed one.

Both orders are loops over the same `step`, so they cannot compute different
numbers by construction, and `walk_matches_the_column_order` shows it:
**exactly equal on 105 models**, every stream and every period, with the five
forward-reading models skipped as inapplicable.

**The seam is two-way, and it needs no interior mutability.** Within a period
the state stage reads the store and the streams stage writes it — but
sequentially, because state settles completely before streams evaluate. That
is why §2.3's store was unnecessary for correctness as well as for speed.

**What this makes possible, and does not yet do.** State at period `t` now
settles with every earlier period's cash already in the store. Nothing reads
it: `E1134` still refuses a series read in logic, so no model can ask. Phase 3
narrows that refusal to FORWARD reads only, and this loop is what makes it
narrowable rather than absolute.

**Waterfalls are their own stage, and now run at their own period.** Those are
different things, and only the second changed: a waterfall still allocates cash
the streams already produced (`docs/17` §4) and stream evaluation still does not
know it exists. What moved is WHEN the stage runs — at its place inside the
period rather than in a pass after all time.

It has to. Under a short pot a tranche is paid less than it is owed, and its
balance must reduce by what it RECEIVED rather than by what it was scheduled;
that is how sequential-pay works, and for a balance at `t` to read a payment
from `t-1` the waterfall at `t-1` must already have run. Backlog 5.2 records
`americredit_2017_1` duplicating its step-down expression across seven class
balances for exactly this reason. `prev.<waterfall>.<step>` is what closes it,
and it stays strictly backward, so no cycle appears.

Composition survives the reorder: an earlier waterfall's steps join the visible
store as they are paid, so a later one reads them at the same period —
`waterfall_nested_split` distributes three waterfalls at one date and still
ties. A waterfall cannot read its OWN steps, which `E1342` refuses at compile
time, so runtime visibility cannot be abused to get around it.

- Files: `crates/cfdl-engine/src/{lib,state,streams,distributions}.rs`.
- Gate — **the collapse property, and it is hard**: the full golden suite
  byte-identical with no re-blessing, asserted by `golden_corpus.rs` in
  `cargo test` as well as by the shell runner; every external benchmark still
  ties to its published figures; benchmark wall-clock within noise on the
  REMIC fleet (the deepest schedules) and the Monte Carlo cases (the
  hottest loop).

## Phase 3 — the read rules (new expressiveness, first tranche)

Guards and recurrences may read series strictly at `≤ t − 1` (§4); phase
0.1's refusal narrows to same-period-and-forward reads. `prev.<waterfall>.<step>`
lands as the symmetric extension of `prev.<entity>.<field>`. The §7.71
probes flip from invalid fixtures to valid ones — the delinquency guard now
fires — and the lagged-sweep and later-balance fixtures of §9 pin the rest.
Grammar and lexer changes are audited against `docs/02` (the §7.19 and
§7.61 cautions: reserved words and the grammar-parser gap).

- Files: parser, compile validator, engine env; `docs/02`, `docs/03`,
  `docs/07`, `docs/10`.
- Gate: new fixtures pass; §5.2's rule holds — no shipped model changes.

## Phase 4 — the account (§5.1)

Parser and IR for the declaration (`account <name> [owned by <party>]`,
`from`, `currency`), `from <account>` as a waterfall pot, `pay ... to
account <name>` as a step form, and `prev.<account>` as a read. Engine
carries balances in the walk under the balance law; draws floored at
`max(balance, 0)` with the refusal named; negative inflows lower the
balance. Balances publish in the valuation plane as non-cash series;
journal entries per §5.2. The `docs/13` §7.41 residue closes here: an
account's `from` names what flows in, checked.

- Gate fixtures (§9): reserve fund-to-target and release; the
  cumulative-sum identity; Highlands restated through an account, tied to
  the same numbers.

## Phase 5 — the state machine (§6.1–6.2)

Transition edges declared beside lifecycle states in pack `types.toml` and
in model-declared lifecycles; undeclared transitions refused (compile where
statable, run otherwise); the trigger policy (`once`, the default latch,
vs. every-period); `state_enter` as the third schedule anchor, resolved
during the walk, re-anchoring on re-entry.

- Files: `crates/cfdl-parser`, `crates/cfdl-pack` (lifecycle loading),
  `crates/cfdl-engine/src/{state,timeline}.rs`, pack `types.toml` files.
- Gate fixtures (§9): the delinquency machine breaching and curing twice on
  realised rent; the trapped-cash cure (accounts + machine + backward
  guards in one pin); the delayed, re-anchored construction schedule.

## Phase 6 — the migrations (§7)

The forward-income exit re-homes under the priced exception, with the
acyclicity check live: a priced amount whose valuation window is touched by
the flow it prices is refused with the path named. The expense stop's open
decision — causal true-up vs. valuation-plane declaration — is settled
against MIT Rentleg, the shipped case that exercises it, and recorded in
`docs/26` whichever way it goes. Affected goldens re-blessed deliberately,
with the delta explained in the changelog.

- Gate: `one_rosslyn` and `mit_rentleg` tie to their references at their
  existing tolerances.

## Phase 7 — the surface (closes the milestone)

Documentation walked end-to-end: `docs/01`–`03`, `07`, `10`, `17` (the
waterfall doc absorbs the account), glossary and `terminology.toml`,
`docs/28` restated from proposed to shipped, the changelog release notes.
The training site is checked for chapters the walk contradicts (events,
waterfalls) and corrected. `docs/13` closes or narrows §5.2, §7.10, §7.36,
§7.40i's trigger half, §7.41, §7.45, §7.71 — each under the file's own
standard: closed items removed, residue restated under an accurate
headline.

## Decisions still open, and who settles them

1. **The expense stop's plane** (§7) — settled by MIT Rentleg in phase 6.
2. **Concrete syntax** for the account declaration, the pay-to-account
   step, the trigger policy, and `state_enter` — settled at each phase's
   parser PR, against `docs/02` and the §7.19 reserved-word caution.
3. **Whether the journal is a golden by default** or only where a fixture
   opts in — settled in phase 1 by the size of what it produces.

## What this plan does not contain

The contract runtime behind `activate contract` (M2), multiple instances of
one pack contract type (F.3), typed pack-declared actions, participant-level
return metrics (`docs/13` §7.72 — gated on the account, built on §7.25's
declared metrics in M4), and optimal exercise (past v1). Each is listed in
`docs/28` §10 with its home.
