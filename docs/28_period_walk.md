# 28 — The Period Walk

Status: proposed. This is the specification for milestone M1 of the v1.0
roadmap: the engine's evaluation order. Nothing here is built. The pre-work
that must land first is in §10.

---

## 1. What this document decides

The engine evaluates in five stages, each completing over the whole timeline
before the next begins: state and events, then streams, then distributions,
then results (`cfdl-engine/src/lib.rs`, the orchestrator comment). This order
makes one whole class of models inexpressible: any model in which realised
cash feeds back into the model's own logic. A unit cannot go delinquent
because rent was not received. A balance cannot move by what a waterfall
actually paid without restating the waterfall's input. An option cannot
exercise on realised income.

This document specifies the replacement: the **period walk**. The causal
stages — state, events, streams, distributions — advance through the grid one
period at a time, and within each period settle in a fixed order. The results
stage keeps its current position, after the walk completes, and is named for
what it already is: the **valuation plane**, a computation over the completed
projection.

The design was probed before it was written. Three spellings of "logic reads
cash" run today and are silently inert — including the strictly backward
spelling this document makes legal — and one fails loudly. `docs/13` §7.71
records the probes; this document is the answer to them.

## 2. The two planes

**The causal plane** is streams, entity fields, events, options' elections,
and waterfalls. Everything in it is causal: a value at period `t` is computed
from values at periods before `t` and from same-period values earlier in the
period order of §3. Nothing in the causal plane reads the future. Deferral is
state: cash that waits, waits in a balance or a pot, never in a forward
reference.

**The valuation plane** is the results stage: netting, rollups, discounting,
metrics, statements, and every construct that measures the completed
projection. It runs after the walk and may read any window of any series. It
cannot write back into the causal plane, with the one priced exception in §7.

This split is the architecture the mature tools converged on. Structured
finance engines are a pure causal walk — triggers evaluate on realised
period cash and flip both ways, reserves carry deferred cash as state — with
price and yield analytics layered over the completed projection. Argus
projects with a period walk and rule-driven lease events, and values in a
separate layer. Backward-induction option valuation requires the split: it
sweeps backward over a completed forward simulation, which only exists if the
causal plane is separable. The one tool that mixes the planes in a single
free-read grid is the spreadsheet, and it is the negative example.

## 3. The walk

For each period `t`, in this order:

1. **State settles.** Field recurrences compute the period's candidates;
   events evaluate in declaration order and their writes overwrite; the
   column settles to one value per path. This is the walk that
   `cfdl-engine/src/state.rs` already performs. Guards and recurrences may
   read, in addition to what they read today, any stream's realised values
   **up to and including `t − 1`**.
2. **Streams evaluate at `t`**, against the state settled in step 1, in
   dependency order within the period. A stream's amount, `active when`
   condition, and `active in state` gate see period-`t` state and any
   realised series value at or before `t` permitted by §4.
3. **Waterfalls whose schedule names `t` distribute**, in declaration order,
   from their declared pots. A waterfall not scheduled at `t` does nothing at
   `t` (§5).
4. **The journal appends** every firing, write, activation, election, and
   allocation the period produced (§8).

Then `t + 1`. The valuation plane runs once, after the final period.

The walk is not a new architecture; it is the completion of one half-built.
Stages 1 and 2 already advance period by period and already hand the streams
stage three period-indexed structures: per-period entity state, per-period
active flags per stream, and the ordered transition record (`EventSim`). That
seam is one-directional. The walk makes it two-directional: the streams
report realised per-period amounts back into the walk's record, and the next
period's state may read them. The seam's data shapes already exist; the
change is who writes and who reads.

## 4. What a read may see

The legality of a read depends on who is reading, not on where the value
lives.

| Reader | Same-period state | Series at `≤ t − 1` | Series at `t` | Series at `> t` |
| --- | --- | --- | --- | --- |
| Event guard, field recurrence | as period opened | **legal (new)** | refused | refused |
| Stream amount / `active when` | legal (settled, step 1) | legal | legal, if acyclic within the period | refused |
| Waterfall pot and steps | legal | legal | legal — the pot is the period's cash | refused |
| Valuation plane | n/a — the walk is complete | legal | legal | legal |

Two rules carry the table. First, **logic reads strictly backward.** A guard
or recurrence reading a series at `t` or later is refused at compile with the
read named — a guard acting on cash that has not happened is non-causal, and
the same-period direction is what the waterfall's declared priority already
provides. With this rule the dependency graph over (node, period) cells is
acyclic by construction, and the engine keeps its founding commitment: a
genuine cycle is refused with the path named, never iterated.

Second, **the causal plane never reads forward.** The two shipped constructs
that do — the expense stop reading a future base year, and the forward-income
exit reading NOI beyond the sale — migrate under §7. Streams keep same-period
reads of each other where the intra-period dependency order permits, which is
today's wave discipline applied to one column of cells instead of the whole
grid.

## 5. Waterfalls: schedule sovereignty

A waterfall distributes **only** at the periods its schedule names, from the
pot its author declares — including an accumulated one.
`benchmarks/cre/penzance_highlands` distributes once, at the end of a
thirteen-year hold, from `series_sum("cre.*", 0, time.t)`: every dollar the
deal produced. The walk changes when a waterfall is *computed*, never when it
*distributes*. Computing the 2024-06 distribution at period 2024-06, rather
than in a stage after all time, is what lets a later period read what was
paid — `prev.<waterfall>.<step>`, the symmetric extension of
`prev.<entity>.<field>`, strictly backward and cycle-free by the same
argument.

Two existing separations are preserved unchanged. The pot is built from
streams only, kept distinct from the results-layer fold that attributes
payments to payees, so a waterfall cannot read its own output
(`distributions.rs`). And steps evaluate in declaration order over a single
running pot that cannot go negative (`docs/17`).

### 5.1 The account

Carried cash gets the industry's own object. In a real deal cash does not
sit in a pot; it sits in **named accounts** — collection account, reserve
account, a participant's distribution account — and the waterfall moves
cash between them. The construct:

```
account <name> [owned by <party-ref>] {
  from <inflow expression>          // per-period inflow, may be negative
  currency <code>
}
```

An account is a declared cash location with a balance. **Ownership is
optional**: a general account belongs to the structure (a collection
account, a reserve); a party-owned account holds cash *allocated to that
participant but not yet paid out*, which is what a per-payee distribution
account is. What a participant is **owed** is not an account: owed is a
receivable, and receivables stay entity fields (Highlands' `unreturned`),
exactly as shipped. An account holds cash that exists; a field records
cash that is due. Conflating the two is the classic waterfall modeling
defect, and the type system keeps them apart.

The balance law, applied at each period of the walk:

```
balance(t) = balance(t−1) + inflow(t) + payments_in(t) − draws(t)
```

**A negative inflow lowers the balance, with no floor.** The language
models returns, and an account fed a deal's whole net cash IS the deal's
cumulative position — negative through the J-curve, positive after — so an
account whose inflow is every stream equals `series_sum` of those streams
from inception, an identity a fixture pins. What is floored is the **draw**:
a step takes at most `max(balance, 0)` remaining — cash that is not there
cannot be distributed, and a draw that would need it is refused with the
account named, not overdrafted.

Three uses, all walk-legal:

- **A waterfall draws from an account**: `from <account>` replaces the
  hand-written cumulative window. Residue after the last step stays in the
  account for the next scheduled date, by construction.
- **A step pays to an account**: `pay <step> to account <name> = <expr>` —
  the reserve pattern (fund to target, top up when short, release when
  over) and the per-participant allocation both become one step form. This
  is the mechanism half of every structured-finance trigger structure.
- **Logic reads a balance**: `prev.<account>` is settled state, strictly
  backward under §4 — an OC/IC-style trigger tests a reserve balance the
  same way a delinquency edge tests realised rent.

### 5.2 What does not change

**Carryover is opt-in by declaration.** `available` keeps meaning exactly
this period's netted cash, and `remaining` stays the step-local running
value — both are the indenture's own vocabulary ("Available Funds",
"amounts remaining"), and both keep their shipped semantics. Every
every-period waterfall in the fleet — the REMICs, the auto ABS — is
untouched: a non-exhausting period's residue stays with the entity as it
does today, unless an account is declared to catch it. A declared
cumulative pot expression stays legal and means what it says. The collapse
property of §9 therefore holds by construction, not by care: no existing
keyword changes meaning.

**An account is a location, not a flow — and not a walk output.** Cash in
an account has already been counted once, as stream cash; the account
balance publishes in the valuation plane as a non-cash series, statements
show the distribution flows under their financing categories, and no
number is counted twice. The balance exists *in* the walk only as state
that steps and guards read.

The account gives `docs/13` §7.41's check its object — an account's `from`
names what flows in, and the balance is auditable per period — and the
journal (§8) records each account's movements at that grain: inflow, each
step's draw or payment-in, and the carried residue.

## 6. Events and the state machine

Under the walk, the latch stops being an architectural necessity and becomes
one trigger policy. An event declares whether it fires once (today's latch,
the default, unchanged) or on every period its condition holds. A repeatable
event is what a covenant that breaches and cures, a plant that curtails and
restarts, and a unit that goes delinquent and current again all need; today
they must be bare fields, unchecked and absent from the transition record
(`docs/13` §7.36). The walk makes the checked form buildable, and this
milestone builds it.

Timing is unchanged and is worth stating: an event's write at `t` is visible
to period `t`'s streams (step 1 before step 2), which is the shipped behavior
— occupancy set in July changes July's revenue.

### 6.1 The declared machine

A lifecycle today is a set of state names — declared by a pack in
`types.toml`, checked by `active in state` (a misspelling is `E1332`) — with
no edges: any event may write any declared state at any time, and a regime
that returns cannot use the vocabulary at all because the latch fires once.
The machine completes the declaration. Wherever a lifecycle is declared, its
**transitions** are declared beside its states: which state may move to
which, re-enterable edges included. `leased → delinquent → leased` is two
edges, walked as many times as the deal's history walks them.

The rules, all of which reuse walk machinery rather than adding any:

1. **An undeclared transition is refused.** At compile where the write is
   statable (`set … status = <literal>`), at run with the edge named where
   it is not. Today's unconstrained write becomes the diagnostic §7.36 asks
   for: the states and edges are reviewable in one place, and an absent edge
   is an error rather than a silent overwrite.
2. **Transitions are driven by events**, with the walk's semantics —
   evaluated once per period, in declaration order, guards reading state as
   the period opened and series strictly backward (§4). A cash-driven edge
   is now expressible: *delinquent when last period's rent came in under
   the amount due* is a guard on a settled series, and *current again when
   it resumed* is the return edge.
3. **Every transition is journaled** (§8): period, entity, edge taken, the
   event that drove it, and the values its guard read. The transition
   record that exists today becomes the machine's audit trail; a regime is
   never again a bare field whose flips the record cannot see.
4. **`active in state` is unchanged** and already re-entry-safe: the
   per-period active flags the state walk hands the streams stage
   (`EventSim.stream_active`) are level-checked each period, so a stream
   gated on `leased` turns off during delinquency and back on after cure
   with no new mechanism.
5. **Truly linear time keeps its constructs.** Calendar-fixed eras are
   phases; condition-driven regimes are machine states. The machine does
   not replace phases and phases do not gain edges.

### 6.2 Schedules anchored to a transition

A schedule today anchors to calendar dates or phase boundaries, both fixed
at compile. The machine adds the third anchor: a state entry. `from
state_enter(<entity>, <state>) for <n> periods` resolves its membership
during the walk — the entry period is settled state by the time any stream
reads it, so the anchor is causal and cycle-free by the same argument as
every backward read. This is what "18 months of construction from whenever
construction starts" needs, and with it the delayed-construction option is
fully expressible: the event enters the state whenever its trigger fires,
the schedule hangs off the entry, and the deal's activity window carves
itself out of the grid. A re-entered state re-anchors: each entry starts its
own window, which is what a second delinquency's cure period means.

## 7. The valuation plane, and the priced exception

Forward reads live in the valuation plane. The two causal-plane constructs
that read forward today migrate:

- **The forward-income exit.** The sale is a causal event; the receipt is
  causal cash in `investing.reversion`; only the **amount** is a valuation —
  forward NOI against a cap rate. The priced exception: a valuation-plane
  value may set a causal amount **where the cell graph stays acyclic** — the
  NOI the reversion reads lies beyond the sale and is unaffected by it. This
  is how Argus computes a direct-cap reversion inside a projection, and how
  `one_rosslyn` already behaves. An amount priced this way that does create a
  cycle — sale proceeds feeding state that feeds the NOI being capitalized —
  is refused with the path named, like any other cycle.
- **The expense stop.** A recovery that reads a future base year is a
  modeling convenience for a true-up that in reality settles later. It either
  restates as a causal true-up (read the base year after it happens, adjust
  then), or declares itself in the valuation plane. The MIT Rentleg benchmark
  decides which, as the shipped case that exercises it.

What the valuation plane needs as a construct — declared metrics, subtotals,
statements as readable objects — is `docs/13` §7.25, §7.43 and §7.55, and is
M4. M1 only requires that the plane exist as the stated home of forward
reads, which it already does as the results stage.

## 8. Provenance: the journal

The walk emits its journal as it goes; the journal is the execution trace in
narrative order, which is the property a stage-wise engine cannot have. It
extends `deterministic.transitions` — which today records field writes only —
to every causal act:

- an event firing: name, period, and the values its guard read;
- every action with its **outcome**: applied, declined (an option outside
  its window), or overridden — a stream activation losing to `active when`
  is recorded, not swallowed;
- every waterfall allocation: step, payee, amount, pot before and after;
- every option election and its payoff.

Results series carry back-references into the journal; a golden may assert on
the journal itself. Monte Carlo emits per-trial aggregates over the same
record, which is `docs/13` §7.18. Nothing here waits for the walk — the
journal can ship first against the current engine — but the walk is what
makes it complete.

## 9. The collapse property, and what proves it

For any model with no cash-into-logic edge — every model that compiles today
— the walk's schedule is a reordering of independent work, and the results
must be **byte-identical**. The full golden suite is the proof obligation: it
passes unchanged, with no re-blessing, or the walk has a defect. Models that
do couple cash into logic are new expressiveness, pinned by new fixtures:
the delinquency machine breaching and curing twice on realised rent; the
lagged sweep; a once-at-end waterfall read by a later balance; a reserve
account funded to target and released; trapped cash accumulating in an
account across a failed trigger and releasing on cure — accounts, the
machine, and backward guard reads in one pin; the cumulative-sum identity
(an account fed every stream equals `series_sum` from inception);
Highlands restated with an account in place of its cumulative window,
tied to the same numbers; and a state-anchored construction schedule
delayed by an event and re-anchored on re-entry.

The collapse caveat is owned by §5.2's rule: carryover is opt-in, no
existing keyword changes meaning, and nothing re-interprets a declared
expression — so the shipped fleet collapses by construction.

Performance holds for the same reason. The schedule is static — computed
once from the dependency graph, replayed per scenario and per Monte Carlo
trial — and uncoupled subgraphs still evaluate as columns. A ten-year grid
with a two-year deal window walks past inert cells at the cost of the
schedule lookup, not the cost of evaluation.

## 10. Pre-work, and what M1 is not

Before the walk is touched, two gates from the roadmap: **loud failure for
unbindable series reads** — the three silent spellings of §7.71 become
compile- or load-time refusals, so a reordering defect cannot hide behind a
silent zero — and **mutation testing** over the engine, so the suite provably
notices when evaluation changes.

M1 is the walk, the read rules, the two migrations, the declared state
machine with its transition anchor (§6), the account construct (§5), and
the journal. It is not: the contract runtime behind `activate contract`
(M2, §7.40i); multiple instances of one pack contract type (F.3); typed
pack-declared actions; or optimal exercise, which stays deferred past v1
and which this design exists to make possible later.

## 11. Cross-references

`docs/13` §5.2 (what a recurrence may read), §7.10 (a truncated series
view — subsumed by §4's backward reads), §7.36, §7.38, §7.71. `docs/17`
(the ordered waterfall). `docs/26` for the evaluation-order canon: streams
produce amounts, logic acts on those amounts, financing and distributions
run over aggregated flows — the walk is that sentence made causal, with the
one-period lag that keeps it cycle-free.
