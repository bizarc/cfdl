# A balance is a field a stream updates — design

Status: **proposal.** Nothing here is implemented.

A balance is a field. The language already lets any entity carry any number of
fields, and `docs/18` settles where such a value lives: a quantity that changes
over time is a field of the entity it describes. Nothing about a balance needs a
new construct, a new keyword, or a new kind of object.

What a field cannot do is read a stream. A balance can therefore be driven by
anything a field can see — an input, a curve, another field — and not by the
cash the model computed. Cash does aggregate, in the pack-declared subtotals
the engine folds after streams (§3), but nothing in a model can read those
either. So:

> **A stream cannot update a field, and a subtotal cannot be read.**

The reason is the pass order rather than the expression scope: fields are
computed in one complete pass before any stream is evaluated, so at the moment
a recurrence runs there is no cash to read (§5.1a).

Both halves are the same gap seen from two sides: the model produces cash, and
cannot feed it back into its own state.

This is a stated limit of v0.1, not an oversight. `docs/03` §3.1 — the
authoritative expression environment — says it outright: *"`next` has no series
access in v0.1. It sees `prev`, `prev.<family>.<entity>.<field>`, `time.*`,
`inputs.*`, `cfg`, `obs` and curves. Reading a stream's history from a
recurrence is not expressible."* The engine matches that exactly.

`docs/14` §3.1, the design document, says the opposite — that the environment
contains *"stream series up to and including `t-1`"*. Two specifications
disagree and the engine follows the governing one. Whatever is decided about
the capability, that contradiction is a defect on its own.

---

## 1. What this replaces

An earlier draft of this document argued for a `claim` construct: a
balance-bearing object declared by a contract, with grammar, IR and engine
changes behind it, and a terminology decision to go with it. That was wrong,
and wrong in a way worth recording, because the mistake is easy to make again.

- **A balance does not need a construct.** It is a field, and fields already
  take any expression. The reason `cre.permanent_debt` has no balance is that
  its author did not need one — a mortgage's debt service is a closed form — and
  that is a pack design choice, not a limit of the language.
- **"A waterfall reduces a balance" is not the general shape.** Most steps pay
  an amount computed from terms, and what accumulates is usually a shortfall
  rather than a balance being drawn down. Building the language around the
  narrow case would have been building it around this month's benchmark.
- **A waterfall does know what is owed.** `owed.<step>` is exactly that, and
  `paid.<step>` is what the step got. The earlier draft said nothing knows what
  it is owed, which is contradicted by a probe in the same document.
- **`claim` is already the right word for what the language has.** A claim is an
  asserted right to cash flows. A stream is one, a waterfall step is one. The
  three uses the learn material makes of the word are the same use. Registering
  it as a name for a balance would have narrowed a correct word to fit a
  construct that should not exist.

What survives from that draft is the evidence, not the design.

---

## 2. The gap, stated exactly

Fields are general and streams are general. The edge between them runs one way
only: a stream can read a field, and a field cannot read a stream.

| direction | today |
|---|---|
| a stream reads a field at `t` | works — every waterfall step in the suite does it |
| a field reads its own previous value | works — `prev` |
| a field reads another field's previous value | works — `prev.<path>` |
| a field reads a stream's value at `t-1` | **absent — `docs/03` §3.1 states this; `docs/14` §3.1 contradicts it** |
| a waterfall's `from` reads another waterfall's step | works, under the name `<waterfall>.<step>` |

Measured against `target/debug/cfdl`, **with no pack active** — every result
below is the language's own behavior, not a pack's:

| reader | reads | result |
|---|---|---|
| a field's `next` | its own `prev`, another field's `prev`, `inputs`, curves | works |
| a field's `next` | any series at all | fails — and see below |
| a stream | another stream's series, `0..t-1` | works |
| a stream | a waterfall step's series | fails, warned |
| a stream that reads a series | another stream that also reads a series | refused: *"A cross-stream read can only see streams that read none"* |
| a waterfall step | `paid.<step>`, `owed.<step>`, `remaining` of an earlier step | works |
| a waterfall step | `paid.<step>` of a different waterfall | refused: `E1341_WATERFALL_FORWARD_REF` |
| a waterfall's `from` | another waterfall's step, as `w1.resid` | **works** |
| a waterfall's `from` | the same, as `stream.w1.resid` — the documented name | zero, silently |

**A failed `next` does not lose a term, it loses the balance.** The warning
reads "next evaluation failed ... using 0", and the zero replaces the whole
expression rather than the unreadable part of it. A running balance written as
`prev - series_sum(...)` does not hold at `prev`; it goes to zero in the second
period and stays there:

```
asset.a.running_control        [1000.0, 990.0, 980.0, 970.0, 960.0]
asset.a.running_from_waterfall [1000.0,   0.0,   0.0,   0.0,   0.0]
```

**The documented series name is wrong, and the wrong one fails silently.**
`docs/03` §3.2 says steps publish as `stream.<waterfall>.<step>` and that
`series_sum` therefore reaches an earlier waterfall's output from a later
one's `from`. The capability is real and the name is not: `w1.resid` works and
`stream.w1.resid` returns zero without a diagnostic, because a name that does
not exist is not an error. A reader following the specification gets an empty
pot and no indication of why.

Two facts to take from the second table. First, the missing capability is
exactly the one `docs/14` §3.1 describes. Second, waterfall step series are
absent from the series map that streams read, so implementing §3.1 as written
would still leave a waterfall's payments unreadable — both halves are needed,
and the second is smaller than it sounds.

---

## 3. Cash already aggregates — in a layer the model cannot read

A field is not the only thing that can accumulate. The packs declare
**subtotals** and **metrics**, folded by the engine after streams are
evaluated, and the ops available to them include cumulative ones.
`packs/credit/statements.toml` already builds a running pool balance this way,
with no field anywhere near it:

```toml
[[subtotals]]
id = "domain.credit.principal_paid_negated"
op = "negated_cumulative"
subtotals = ["domain.credit.principal_collections"]

[[subtotals]]
id = "domain.credit.balance_outstanding"
op = "sum"
subtotals = ["domain.credit.original_balance",
             "domain.credit.principal_paid_negated"]
```

That is a balance driven by cash — original advanced, less principal collected
to date — and it runs today. It is also how a statement is assembled at all:
`statements.toml` folds stream **categories** rather than stream names, which
is what keeps a statement correct as a pack grows, while `metrics.toml` folds
named streams for the ratios that need particular ones.

So the honest form of this document's thesis is narrower than "a balance cannot
be driven by cash". It is this:

> Cash aggregates in a layer of its own, and that layer is **terminal**.
> Nothing in the model can read what it produces.

(Subtotals are folded at layer 4, before waterfalls run at layer 5 — so they
carry stream cash and never a distribution.)

Measured:

| reader | reads `domain.credit.balance_outstanding` | result |
|---|---|---|
| a field's `next` | | *unknown variable*, warned, zero |
| a stream | | *unknown variable*, warned, zero |
| a waterfall step | | *unknown variable*, warned, zero |

Three consequences follow, and they are the reason the gap survives the
discovery of the subtotal layer.

**A subtotal can be reported and not used.** `cre.permanent_debt` could publish
an outstanding balance tomorrow as a subtotal — original principal less
cumulative amortization — and a case could assert it. No covenant test, no debt
yield, no cash sweep and no waterfall step could read it.

**A subtotal cannot do arithmetic that is not a fold.** The ops are `sum`,
`cumulative`, `negated_cumulative`, `ratio`. A balance floored at zero, a
payment capped at what remains, a preference that compounds only while
outstanding — none of those is a fold over categories, and all of them are
ordinary field expressions.

**A subtotal cannot close a loop, and should not.** An ABS note balance
determines the interest that determines the excess cash that determines the
principal that determines the balance. Nothing in a layer computed *after*
streams can participate in that, which is correct: the loop only opens when the
read is bounded at `t-1`, and that bound belongs to the recurrence.

---

## 4. What it costs today, in three packs

Not a hypothetical. Each of these is in the repository now.

**A mortgage note has no reducing balance.** `cre.permanent_debt` computes debt
service in closed form — interest-only, then `-pmt(...)`, plus an optional
`-fv(...)` balloon — and never states an outstanding balance. Everything a
scheduled mortgage is asked works. Anything that needs the balance itself does
not: a covenant on loan-to-value, a debt yield, a refinancing test, a cash
sweep that pays down principal out of available cash. The obvious
implementation — a field reduced each period by the principal component of the
debt-service stream — cannot be written, so the contract does the only thing it
can and stays closed-form.

**A preferred return accretes and is never paid down.**
`fixtures/valid/waterfall_cre_jv_promote`:

```cfdl
msgw_preference init inputs.msgw_capital next prev * (1.0 + inputs.pref_rate)
...
pay msgw_preference to party.msgw = min(asset.jv.msgw_preference, remaining)
```

The step pays what the pot allows. The field never learns what was paid, so
next period it compounds on the full balance whether or not a distribution
occurred. In a fixture about expressiveness this is harmless. In a deal it is a
preferred return that only ever grows.

**A note class's balance is maintained by copying the waterfall.**
`benchmarks/credit/americredit_2017_1` carries seven balances as fields, and
because a field cannot see what the waterfall paid, each one recomputes the
entire distribution a period lagged. The case therefore states its distribution
twice, and the two copies disagreed: the waterfall paid the pack's servicing
series, which charges a January-cutoff pool for two months in the first
collection period, while the recurrence carried its own copy of the fee and
charged one. Eleven published cells and two published weighted average lives
were wrong. The collateral was right throughout — no allocation feeds back into
it — so the defect was entirely in one layer keeping its state in another.

Three packs, three domains, one missing edge.

---

## 5. What changes

Two changes, both small, neither of them syntax.

**5.1 One scalar, not a series.** A recurrence does not need a window over a
stream's history. Every case in §4 wants the same thing:

```
balance(t) = balance(t-1) - paid(t-1)
```

which is one number — the previous period's value — and never a range. So the
capability to add is a *value*, `prev.<stream>` and `prev.<waterfall>.<step>`,
alongside the `prev.<family>.<entity>.<field>` a recurrence already reads. The
same keyword, the same meaning: the completed previous column.

```cfdl
balance init 182000000  next max(prev - prev.notes.distribution.a1_principal, 0.0)
```

A scalar is not merely sufficient, it is *better* than the series access
`docs/14` §3.1 describes. A window has to be clamped at `t-1`, and a clamp is a
check that can be wrong; a scalar cannot reach period `t` because there is
nothing to address it with. `docs/14`'s own argument for the design was that
the guarantee is enforced by absence rather than by analysis, and a window
weakens exactly that. Nor does a window buy anything: a cumulative-to-date
quantity is itself a recurrence, so anything a window could sum can be
accumulated one period at a time by a second field.

**5.1a What actually blocks it: the layer order.** Neither form works today,
and the reason is not the environment's contents. The engine evaluates in
complete layers — fields, then events, then streams, then subtotals, then
waterfalls — each finishing before the next begins. `compute_states` runs as one
pass over every period and returns a whole series per field, before any stream
is evaluated. A field at period `t` cannot read a stream at `t-1` because at
that moment no stream has been evaluated at any period at all.

`fixtures/valid/evaluation_order` pins the boundaries. Two of them are worth
stating on their own: an event guard can fire on a **field** crossing a
threshold but never on **cash**, since events are simulated before any stream
exists; and subtotals are folded *before* waterfalls run, so a waterfall's
payments never appear in a subtotal or a statement — the collections statement
describes what the assets produced and says nothing about who was paid.

`docs/14` §3.2 specifies the interleaving that would fix it:

```
for t in 0..n:
    for each state:  value[t] = (t == 0) ? init : next(prev = value[t-1], ...)
    for each stream: evaluate at t, with state values at t available
```

The engine runs field-major, then stream-major, then waterfalls. So the work is
evaluation order, not expression scope — and it is the same restructuring
whether the exposed quantity is a scalar or a series, which is another reason
to take the scalar.

**5.2 Widen where waterfall steps are visible.** They already are visible —
to another waterfall's `from`, under the bare name. They are not visible to a
stream or to a field. Without widening it, 5.1 reaches a mortgage's principal
component and not a note class's principal allocation.

Correct `docs/03` §3.2's name in the same change, and decide which of `docs/03`
§3.1 and `docs/14` §3.1 is the specification.

Neither change adds a keyword. No existing model changes meaning, because
nothing today can read what these two changes expose.

### 5.3 Worth doing at the same time

`series_sum` on an unknown series returns zero in a stream, silently — no
diagnostic, no warning. The same read in a field's `next` warns. Once field
recurrences start naming stream series, a typo becomes a balance that silently
never amortizes. Recorded as `docs/13` §7.38, and it should land with this
rather than after it.

---

## 6. What packs do with it

Nothing is required of a pack. The changes above are additive and every
existing contract keeps working.

What becomes *available* is the choice each pack author already had, with the
awkward option removed:

- `cre.permanent_debt` could carry an outstanding balance a model can *read* —
  which a subtotal cannot give it — and that is what a covenant test, a
  refinancing and a cash sweep all need.
- The credit pack could offer a note stack whose classes carry balances,
  instead of every case hand-rolling one — three cases have now done so three
  different ways. That is `docs/13` §2.4, and it stays a pack question rather
  than becoming a language one.
- A JV preference could be paid down.

The existing convention — closed form wherever possible, a field only where a
quantity cannot be recomputed from the period — remains right. It is why these
contracts are small and survive a change of calendar. What the convention
cannot express is a quantity that depends on cash, and after this it can.

---

## 7. What this does not solve

- **Termination.** A clean-up call redeems every class and ends the deal; a
  contract still runs its declared term (`docs/13` §7.39).
- **Triggers that reorder a waterfall** (`docs/17` §5, `docs/20` §2.4).
- **Same-period feedback.** A balance updated by a stream is the balance the
  entity carried *into* the period, which is what a distribution date means and
  what every published decrement table states. A model needing a payment to
  affect a balance within the same period still cannot have one, and should not:
  that is where cycles live.

---

Provenance: `benchmarks/credit/americredit_2017_1`, August 2026, which needed a
balance driven by cash, could not have one, and paid for it with a defect its
own assertions could not see. The earlier version of this document proposed a
construct to fix that; the construct was unnecessary and the gap it was built
around was one missing edge in an environment the specification already
describes.
