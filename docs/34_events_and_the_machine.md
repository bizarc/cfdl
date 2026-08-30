# Events and the machine — scope

Status: **decisions settled** — reviewed and answered 2026-08-28; D1–D8
resolved, D2a added in review. Not published; repository-only.

## The principle

**An event is something that happens.** Time is an event, a default is an
event, a cure is an event, a payment is an event. There is no restriction on
an event happening once — a unit that defaults, cures and defaults again has
had three events, and a model that can only record the first is wrong.

The ontology, stated fully:

- **Events — including time — are the only things that can change the state
  of an entity.** Status moves only when an event fires; fields move only
  by their recurrence (time) or an event's write. This already holds in the
  shipped engine, and the walk's ordering is what proves it.
- **An event need not be named.** It can be described by the entity it
  impacts and the conditions that must be true — which is exactly the
  machine's guarded edge: `current -> defaulted when <conditions>`.
- **An event can be named** and carry logic and conditions — the `event`
  declaration.
- **The journal is the event log.** Every settled payment, every
  transition, every action's outcome is a dated occurrence in
  `deterministic.journal` and `deterministic.transitions`. Constructs are
  event *sources*.

Measured against that ontology, the shipped language is two deltas short,
both small and both on constructs that otherwise already match it:

1. **The anonymous event cannot act.** A guarded edge fires every
   occurrence — re-armed by re-entry, journaled — but arrives empty-handed:
   no behavior rides on the transition. Per-arrival bookkeeping (record the
   shortfall, reset the cure counter, strike the prevailing market rent)
   has no home.
2. **The named event cannot recur.** `event` latches — the engine skips a
   fired event forever (`event_fired`, `crates/cfdl-engine/src/state.rs`)
   and `docs/01` §13 declares the latch a definition. Under the ontology it
   is a special case: a one-shot is an event whose nature happens to be
   once, and it should say so.

In one sentence: the construct that repeats cannot act, and the construct
that acts cannot repeat. This plan closes both halves, under one rule:
**informal events are described by guard conditions; canonized events use
the `event` keyword — and both paths work identically.** Identical means:
same firing semantics, same evaluation environment, same action
vocabulary, same journaling.

## What this unblocks, concretely

- **Per-arrival behavior** — §7.77's DSCR cash-trap cure window (a counter
  that resets on each breach), §7.74's step-shortfall and
  servicer-advance bookkeeping.
- **Endogenous re-striking** — look up the prevailing market rent or rate
  at the transition's own instant and hold it for the cycle: chained
  rollover (`docs/33` Item 1), delinquency repricing.
- **Recurring named events** — a default event that can happen twice
  without spelling every recurring condition as a state pair when no state
  is otherwise needed.
- **The vocabulary heals** — "event" in the language means what it means
  to a practitioner.

---

## Design decisions

Numbered; each was answered individually in review, 2026-08-28. The
decisions below are settled.

### D1 — when does an event fire? On each occurrence (rising edge).

An event fires **each time its conditions become true having been false**,
and re-arms when they fall. Recommended over the two alternatives:

- *Latch (status quo):* fires once ever. Rejected as the definition — it
  is a special case, not the concept.
- *Level-triggered:* fires every period the conditions hold. Rejected: a
  DSCR below trigger for twelve months is one breach event and twelve
  breach-months, not twelve breaches. "The conditions hold" is a *state*;
  `active in state` and edge guards already express it.

Rising-edge is the semantics the machine's edges already have — taking the
edge disarms it, leaving the from-state re-arms it — so the named and
anonymous spellings of an event stop disagreeing about what an occurrence
is.

**There is no `once` keyword.** The language never restricts an event to
firing once; once-ness is always a property of the world the model
declared, and it already has two spellings:

- a **schedule whose occurrence is singular** — `schedule on 2027-03`
  fires once because the date occurs once (D1a);
- a **topology with no way back** — a refinance fires once not because the
  event is latched but because afterwards the loan IS refinanced:
  `current -> refinanced` with no returning edge. If the model declares a
  return edge, it declared that re-firing is possible.

The latch was a hidden state — "has fired" — living outside the machine,
unjournaled and undeclarable. Removing it puts all memory where memory
lives: in schedules and states, the two things a model can see.

Pinned: conditions are evaluated **once per period** per event, in the
walk's state stage, in declaration order; an event fires at most once per
period; at most one transition per entity per period is unchanged.
Rising-edge detection is this period's evaluation compared against the
last — nothing re-evaluates within a period.

### D1a — time conditions use the schedule language

The schedule sub-language is already the language of when things occur —
dates, intervals, anchors (`state_enter` included), roll conventions,
calendars — and schedules are already event sources for payments. An event
whose occurrence is time-driven says so in that language rather than in
grid-fragile arithmetic: `schedule on 2027-03`, not `when time.t == 15`.
An event — named or anonymous — may carry a **schedule clause** (its time
condition) and/or a **`when` clause** (its state and cash conditions).

**Settled: the schedule supplies the occurrences; `when` filters them.**
With both present, the event fires at EACH scheduled occurrence where the
conditions hold — `schedule every quarter` + `when dscr < 1.2` is a
quarterly covenant test, and four consecutive failing tests are four
breach events, because the model declared quarterly testing. This is not
the level-triggering D1 rejected: that was rejected as a DEFAULT, because
it made every grid period an implicit occurrence nobody declared. Here
the occurrences are declared. Rising-edge applies only when no schedule
is present — occurrences then come from the condition's own dynamics.
A model wanting "only on entry into breach" has the honest spelling
already: `ok -> breach` on a machine, which is what states are for.

### D2 — the anonymous event acts: transitions carry behavior

```cfdl
current -> defaulted when series_sum("unit.pay", time.t - 1, time.t - 1) < inputs.due {
  set shortfall       = inputs.due - series_sum("unit.pay", time.t - 1, time.t - 1)
  set months_in_state = 0
}
```

An edge may carry an action block. Actions run **every time the edge is
taken, whatever took it** — its own conditions, or a named event's status
write moving the entity across a permission edge. Field names are
entity-relative (`set shortfall`, not `set asset.unit_a.shortfall`): one
lifecycle is bound by many entities, and the behavior belongs to the entity
that transitioned.

### D2a — a model may augment a pack machine's actions, additively

Per-edge and entry actions must be reachable from all three positions a
practitioner occupies: authoring a pack, modeling with no pack, and — the
common middle — modeling ON a pack whose machine is right but whose
actions stop short. A model may therefore attach entry and edge actions
to a pack-declared machine's states and edges, **additively only**: it
cannot add states or edges, remove the pack's actions, or alter the
topology — the pack's machine stays the checkable contract, and the
model's actions run after the pack's (the specific refines the general,
same-field conflicts journaling `overridden` per D5). This is what makes
per-edge actions an extensibility surface rather than a pack-author
convenience.

**Storage, settled in phase 1.** The pack's and the model's actions share ONE
list per state and per edge, ordered pack-first, and every action carries a
REQUIRED `author` of `pack` or `model`. Three shapes were compared: author
inferred from the presence of `generated_by`; this one; and a separate
top-level `lifecycle_augmentations` node keyed by `lifecycle_id`.

The inferred form was rejected on the §7.38 argument — a lowering path that
forgot the stamp would journal a pack action as the model's, silently, and an
`overridden` line that cannot name the author is the one thing the record
exists to prevent. The separate node keeps the pack's `Lifecycle` node
byte-identical to what the pack declared, which is the stronger guarantee,
and it was rejected only on weight: the IR already flattens pack machines
into `lifecycles` wholesale, so the seam it protects was already gone, and it
costs a top-level concept plus a join in the engine's arrival path.

A required discriminator buys what that seam was for — attribution that
cannot be forgotten — at no structural cost. The additive rule stays
enforceable: validation asserts every `pack` action precedes every `model`
action in each list.

**A model addresses a pack machine by qualified name.** `lifecycle
opco.enterprise { … }` augments; a block naming no existing machine declares
one. The grammar's `lifecycle_stmt` therefore takes a `qname`, not an
`IDENT` — a gap found in phase 1, since a dotted pack machine could not be
named at all.

### D3 — entry actions are first-class, and the primary domain spelling

Both grains exist because both are real, and `cre.unit` shows the split.
Three edges arrive in `leased` — `vacant -> leased` (lease-up),
`holdover -> leased` (renewal), `downtime -> leased` (re-let):

- **Entry actions** (`on enter <state> { … }`) carry what is true of the
  STATE, however it was reached: "on entering `leased`, reset
  `months_in_state` to 0" holds for all three arrivals, and for the fourth
  edge someone adds later. This is the primary domain spelling — a pack's
  `types.toml` machine declares the state's meaning once and every model
  using the type inherits it.
- **Edge actions** (D2) carry what is true of the PATH taken. A renewal
  and a re-let both land in `leased`, but the rent is struck differently
  because of how you arrived — bump the existing rent on
  `holdover -> leased`, strike at market on `downtime -> leased`. An entry
  action cannot express that; it does not know which edge fired.

Entry-action blocks are declared in model lifecycles and in pack
`types.toml` machines alike. Execution order on arrival: entry actions
first (the state's own setup), then the taken edge's actions (the
specific refines the general) — a same-field write journals the earlier
one `overridden`.

### D4 — the action vocabulary on edges is `set` only

Stream gating is already declarative (`active in state`) and schedule
anchoring already follows entry (`state_enter`); imperative
`activate`/`deactivate` on an edge would duplicate the declared pattern.
`exercise option` stays with named events. If a case forces more verbs, it
forces them with provenance — the settled starting point is `set`, with
more verbs added as domain packs require them.

**Pinned: an entry or edge action may `set` FIELDS only, never `status`.**
An action writing status would fire a second transition inside the same
period — violating one-transition-per-entity-per-period and inviting
same-period cascades. A transition that should cause another transition
is topology: an edge from the target state, taken next period. Status
writes remain the named event's privilege, validated against the edge
relation as today.

### D5 — evaluation discipline (unchanged, extended)

- Actions evaluate in the **same environment as the guard**: state as the
  period opened, series strictly backward, `inputs`, curves, `time.*`. No
  new cycle risk — it is the environment the walk already proves acyclic.
- Writes follow the existing event-`set` law (ONE VALUE PER PATH,
  `state.rs`): the write settles the field for the period, the recurrence
  resumes from it next period, streams later in the same period read the
  settled value. `before`/`after` journaled; a losing same-period write
  journals `overridden`.
- **At most one transition per entity per period** is unchanged; a taken
  edge's actions run in declaration order after the move.

### D6 — the named event, re-founded

To describe an event informally, use guard conditions on the machine; to
canonize a formal, named event, use the `event` keyword — **and both paths
work identically.** A named event fires on each occurrence under D1; a
one-shot expresses its once-ness in a singular schedule or a
no-return topology, never a latch. Its `set … status`
writes stay validated against the declared edge relation and can drive the
same permission edge repeatedly — and a status write that moves the entity
triggers the target state's entry actions exactly as the anonymous path
would. One semantics, two spellings: the anonymous form for entity-local
conditions (described by the entity it impacts and the conditions that
must be true), the named form for occurrences worth canonizing — referenced
from elsewhere, or spanning entities.

### D7 — the journal and results

**The journal is an ordered, unversioned log of everything that occurred,
in order.** Nothing versions events. What is versioned is the results FILE
FORMAT: `results_version` is the schema stamp on `run.json` (0.4 added the
account fields, 0.5 the transitions), so a consumer knows the shape it is
reading.

- Each named-event firing is journaled per occurrence (the latch merely
  stopped occurrences existing).
- A transition record gains its actions' outcomes as children — the
  transition is the event, the actions are what it did. That shape change
  is why the stamp bumps (0.5 → 0.6); `docs/06` and the results-schema
  gate carry it.
- Noted for later, out of this scope: `journal` and `transitions` are two
  arrays today; the ontology says there is ultimately ONE event log with
  typed rows. Unification is its own item when a consumer forces it.

### D8 — the vocabulary

Glossary and `docs/22`: **an event is something that happens; the journal
is the event log; edges, event declarations and schedules are event
sources; an anonymous event is described by the entity it impacts and the
conditions that must be true.** `docs/01` §13's latch paragraphs are
rewritten to rising-edge (D1). The "machine does not latch" doctrine
survives with its conclusion stated plainly and generally: **nothing
latches.** Once-ness is declared, never engine policy — a singular
schedule or a topology with no way back, per D1.

*Corrected during phase 1:* this decision previously read "rising-edge plus
`once`" and "nothing latches unless it says `once`", written before D1
settled. D1 rejects a `once` keyword explicitly and D6 agrees; the phrase
was stale and is removed rather than reconciled.

---

## Migration

D1 changes the meaning of every existing named event whose conditions
re-rise. Pre-1.0 this is a semantics fix, not a compatibility promise
broken — but it is measured, not assumed:

1. **Corpus audit first.** Instrument a run over every fixture, golden and
   benchmark model counting, per event, the rising edges of its condition.
   Count = 1: unaffected. Count > 1: read one by one — either the re-fire
   is the correct meaning (the latch was masking a bug; each becomes a
   fixture pinning the new behavior) or the model genuinely means once —
   and gains the state or singular schedule it was implicitly relying on,
   which makes the one-shot-ness visible in the model rather than latent
   in the engine.
2. **Golden re-blessing** follows the audit under the full cadence; the
   collapse property over the blessed corpus must hold before and after.
3. No deprecation period: the audit is the deprecation period. Settled in
   review as a clean cut — no compatibility flag, no surviving latch.

## Phases

| phase | delivers | gate |
|---|---|---|
| 1 | Semantics settled: D1–D8 answered; `docs/01` §7.3/§13, `docs/02` grammar, `docs/05` IR schema updated together; `docs/28` §6 amended | spec/grammar/IR reviewed as one change |
| 2 | Parser + IR + validation: entry-action and edge-action blocks, the event schedule clause (D1a), entity-relative field resolution, model-side augmentation of pack machines (D2a); diagnostics — unknown field on the bound entity (named set), action expression reading same-period series refused (E1134's argument, one construct over) | per-code validate tests from invalid fixtures |
| 3 | Engine: rising-edge firing once per period, scheduled occurrences (D1a), entry-then-edge action execution in the walk, journal children, `results_version` 0.6 | fixtures: re-fire on cure/re-default; action write visible to a same-period stream; declaration-order writes; one-shot via no-return topology |
| 4 | Corpus audit + golden re-bless per Migration | collapse property holds across the blessed corpus |
| 5 | Pack surface: `types.toml` machines carry `entry_actions` blocks and per-transition actions; pack-load validation for them; `docs/07` documents the surface. **The shipped packs declare actions as cases need them, not up front** — see the note below | pack load tests; `docs/07` |
| 6 | Vocabulary sweep: glossary, `docs/22`, `docs/10` rows, `docs/33` Item 1 marked closed by this plan; learn-site chapters flagged for the next curriculum pass | glossary-check; site gates when published |

Phases 1–4 are one arc (the semantics are not shippable half-changed); 5–6
follow independently.

**Phase 5 landed as mechanism, not content.** The plan named `cre.unit`'s
re-let actions as its deliverable, on the assumption that declaring them was
the same act as building the surface. It is not. A pack's fields are populated
by its LOWERING RULES, which run per contract instance — so an action can only
rely on a field where the contract that lowers it is present. None of the eight
models binding `CRE.Asset.Unit` carries a `cre.lease_unit` contract; they type
an entity as a unit while exercising accounts, hierarchy and typed entities.
Declaring the re-let actions now would warn on all eight and write nothing.

So the packs ship the capability and declare actions when a case needs them,
which is the same discipline the rest of this repository follows: worth a case
before it is worth a construct. `docs/33` Item 1's chained-rollover case is the
one that will force `cre.unit`'s, and it will carry a lease contract, so the
field will lower with it. The showcase fixtures are the chained-rollover
re-strike from `docs/33` — the probe that found the gap becomes the fixture
that closes it — and a §7.77 cure-counter.

## Non-goals

- No exit actions (`on leave <state>`) until a case forces them; entry
  actions (D3) cover the observed need.
- No new action verbs (D4) without a forcing case.
- No unification of `journal` and `transitions` into one log (D7's note)
  in this scope.
- No change to schedule semantics, `state_enter` or `active in state` —
  they are correct and this plan composes with them.
- No backward induction or optimal exercise; deterministic triggers only,
  as v0.1 states.

## Relation to the roadmap

**First priority within M2** (settled in review): it stands on the walk,
the machine and the journal, and it is the mechanism §7.77 (cure window),
§7.74 (shortfall, advances) and §7.76's counters were waiting to spell —
three other M2 entries consume it, so it goes first, ahead of the
independent §7.50/§7.73 pair. §7.73 composes: state-gating gates the streams, and this
plan makes the states act. A backlog entry referencing this document is
appended as §7.79.
