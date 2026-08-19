# A balance is a field a stream updates — design

Status: **proposal.** Nothing here is implemented.

A balance is a field. The language already lets any entity carry any number of
fields, and `docs/18` settles where such a value lives: a quantity that changes
over time is a field of the entity it describes. Nothing about a balance needs a
new construct, a new keyword, or a new kind of object.

What a field cannot do is read a stream. So a balance can be driven by time —
a rate, a schedule, a declared curve — and cannot be driven by cash. That is
the whole gap, and it is one sentence long:

> **A stream cannot update a field.**

`docs/14` §3.1 already specifies the fix, in the environment it gives a
recurrence: `prev`, `time.*`, `inputs.*`, curves, and *"stream series up to and
including `t-1`"*. The engine builds that environment from `prev_states` and
`prev_self` and no series map at all. The specification is right and the
implementation is missing.

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
| a field reads a stream's value at `t-1` | **specified in `docs/14` §3.1, absent from the engine** |

Measured, against `target/debug/cfdl`:

| reader | reads | result |
|---|---|---|
| a field's `next` | any series at all | zero, with a warning |
| a stream | another stream's series, `0..t-1` | works |
| a stream | a waterfall step's series | zero, and silently |
| a stream that reads a series | another stream that also reads a series | refused: *"A cross-stream read can only see streams that read none"* |
| a waterfall step | `paid.<step>`, `owed.<step>`, `remaining` of an earlier step | works |
| a waterfall step | `paid.<step>` of a different waterfall | refused: `E1341_WATERFALL_FORWARD_REF` |

Two facts to take from the second table. First, the missing capability is
exactly the one `docs/14` §3.1 describes. Second, waterfall step series are
absent from the series map that streams read, so implementing §3.1 as written
would still leave a waterfall's payments unreadable — both halves are needed,
and the second is smaller than it sounds.

---

## 3. What it costs today, in three packs

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

## 4. What changes

Two changes, both small, neither of them syntax.

**4.1 Implement `docs/14` §3.1.** Put completed stream series into the
environment a field's `next` is evaluated in, bounded at `t-1`. The bound is
what keeps the guarantee: every edge still points backward, so no cycle can
close, and the property stays enforced by absence rather than by a detector.
`compute_states` already receives what it needs to do this; it builds the
environment without a series map.

Then a balance is what it should have been all along:

```cfdl
entity asset note_a1 : Credit.Asset.Tranche {
  balance init 182000000
          next max(prev - series_sum("notes.distribution.a1_principal",
                                     time.t - 1.0, time.t - 1.0), 0.0)
}
```

and the same shape serves a mortgage (`- series_sum("loan.principal", ...)`), a
preferred return (`prev * (1 + rate) - series_sum("w.msgw_preference", ...)`),
and a construction facility that draws and repays.

**4.2 Publish waterfall step series into the series map.** A waterfall step is a
stream and appears in results as one; it is not in the map streams and fields
read. Without this, §3.1 reaches a mortgage and not a note class.

Neither change adds a keyword. No existing model changes meaning, because
nothing today can read what these two changes expose.

### 4.3 Worth doing at the same time

`series_sum` on an unknown series returns zero in a stream, silently — no
diagnostic, no warning. The same read in a field's `next` warns. Once field
recurrences start naming stream series, a typo becomes a balance that silently
never amortizes. Recorded as `docs/13` §7.38, and it should land with this
rather than after it.

---

## 5. What packs do with it

Nothing is required of a pack. The changes above are additive and every
existing contract keeps working.

What becomes *available* is the choice each pack author already had, with the
awkward option removed:

- `cre.permanent_debt` could carry an outstanding balance, which is what a
  covenant test, a refinancing and a cash sweep all need.
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

## 6. What this does not solve

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
