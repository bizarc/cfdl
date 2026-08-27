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

**0.2 Mutation testing over the engine.** `cargo-mutants` (or equivalent)
over `cfdl-engine`, a `make mutants` target, and a blessed baseline: the
surviving-mutant list is reviewed and recorded, not zero by fiat. The point
is that phase 2's reorder cannot pass the suite by accident.

- Gate: baseline recorded in the repo; CI runs the target on the engine
  crate at least on demand.

## Phase 1 — the journal (independent of the walk; may ship first)

Extend `deterministic.transitions` into the journal of §8: event firings
with the values their guards read; every action with its outcome (applied /
declined / overridden — a stream activation losing to `active when` is
recorded); waterfall allocations per step with pot before and after; option
elections. Additive keys in the results schema, so existing consumers keep
working; goldens re-blessed deliberately for the new keys, nothing else
moving. Monte Carlo emits per-trial aggregates over the same record, which
closes `docs/13` §7.18.

- Files: `crates/cfdl-engine/src/{state,streams,distributions,results}.rs`,
  `docs/06` and `docs/schemas` (results schema), `tools/check-results-schema`.
- Gate: a golden asserts on the journal itself; the schema gate validates
  it; all numeric series byte-identical.

## Phase 2 — the walk (the reorder; blocks all new expressiveness)

**2.1 Dependency extraction over guards and recurrences.** The existing
`series_references`/`selector_matches` machinery runs over event guards and
field `next` expressions, producing the cell-level edges of §4.

**2.2 The schedule.** A static builder classifies the model: subgraphs with
no cash-into-logic edge evaluate as columns (today's waves, unchanged);
coupled subgraphs get the per-period interleave. Cross-time cycles are
refused with the path named. The schedule is computed once and replayed per
scenario and per trial.

**2.3 The store.** Series storage represents partially-built columns; a
read of a not-yet-computed cell is a loud engine error, never a substituted
null — phase 0.1's discipline applied inside the engine.

**2.4 The fold.** Streams and scheduled waterfalls move inside the walk
that `state.rs` already performs: per period — state settles, streams
evaluate, waterfalls whose schedule names the period distribute, journal
appends. The `EventSim` seam becomes two-way: realised per-period amounts
flow back into the walk's record.

- Files: `crates/cfdl-engine/src/{lib,state,streams,distributions}.rs`.
- Gate — **the collapse property, and it is hard**: the full golden suite
  byte-identical with no re-blessing; the mutation baseline does not
  regress; benchmark wall-clock within noise of the shipped engine on the
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
