# Events and the machine — scope

Status: **draft scope, for review.** 2026-08-28. Not published;
repository-only.

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

Numbered so review can answer them individually. Each carries a
recommendation; none is settled until this document's status changes.

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
condition) and/or a **`when` clause** (its state and cash conditions);
both present means the event fires on scheduled occurrences where the
conditions hold.

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
forces them with provenance.

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
rewritten to rising-edge plus `once`; the "machine does not latch" doctrine
survives with its conclusion stated plainly: nothing latches unless it says
`once`.

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
3. No deprecation period: the audit is the deprecation period.

## Phases

| phase | delivers | gate |
|---|---|---|
| 1 | Semantics settled: D1–D8 answered; `docs/01` §7.3/§13, `docs/02` grammar, `docs/05` IR schema updated together; `docs/28` §6 amended | spec/grammar/IR reviewed as one change |
| 2 | Parser + IR + validation: entry-action and edge-action blocks, the event schedule clause (D1a), entity-relative field resolution; diagnostics — unknown field on the bound entity (named set), action expression reading same-period series refused (E1134's argument, one construct over) | per-code validate tests from invalid fixtures |
| 3 | Engine: rising-edge firing once per period, scheduled occurrences (D1a), entry-then-edge action execution in the walk, journal children, `results_version` 0.6 | fixtures: re-fire on cure/re-default; action write visible to a same-period stream; declaration-order writes; one-shot via no-return topology |
| 4 | Corpus audit + golden re-bless per Migration | collapse property holds across the blessed corpus |
| 5 | Pack surface: `types.toml` machines carry `on enter` blocks (and per-transition actions); `cre.unit` gains re-let entry actions (re-strike, counter reset); pack validations for action fields | doc-examples gate; pack READMEs |
| 6 | Vocabulary sweep: glossary, `docs/22`, `docs/10` rows, `docs/33` Item 1 marked closed by this plan; learn-site chapters flagged for the next curriculum pass | glossary-check; site gates when published |

Phases 1–4 are one arc (the semantics are not shippable half-changed); 5–6
follow independently. The showcase fixtures are the chained-rollover
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

M2-shaped work: it stands on the walk, the machine and the journal, and it
is the mechanism §7.77 (cure window) and §7.74 (shortfall, advances) were
waiting to spell. §7.73 composes: state-gating gates the streams, and this
plan makes the states act. A backlog entry referencing this document is
appended as §7.79.
