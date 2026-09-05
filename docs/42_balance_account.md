# 42 — The balance is an account: a claim, rolled by the engine from the lines that move it

*Drafted 4 September 2026 from the conversation that followed the level-pay
balance (PR #289); revised the same day after review, and again once the
surface was settled (§3.2, §3.5, §3.6). Status: DESIGN NOTE,
nothing built. It is a language and engine item of its own, separate from
the engine restructure (`docs/13` §7.44), and it should be settled before
the restructure so the restructure is not reopened for it. Where this note
and the shipped level-pay balance disagree, the shipped balance is the
stopgap.*

## 0. The sentence

A balance is what is owed. It changes only because something happened —
cash moved, a claim accrued, a claim was written off — and every one of
those happenings has an amount, a period and a cause. The balance is the
sum of them. Nothing else is allowed to compute it. The state of the
entity that owes it does not change the balance; it changes what happens
to it next.

## 1. What is wrong today

### 1.1 Two calculations of one activity

The credit pack's level-pay pool now carries a balance field
(`credit_level_pay_balance_<instance>`, `docs/40` §3). It is rolled forward
each payment period from the contract's rates: opening, less the level-pay
principal fraction, less defaults and prepayments drawn from the opening
balance. The streams are computed from the same rates and read the balance.
So the balance and the streams agree — but nothing MAKES them agree. The
streams do not feed the balance and the balance is not made of the
streams. It is a second calculation running beside the first.

`docs/13` §7.104 is the proof. Where the accrual and amortization
conventions differ, the principal the pool PAYS and the principal the
balance LOSES drift apart, and the balance keeps reporting a number the
cash never produced. A summary that can disagree with its own detail is not
causal, whatever its recurrence says.

### 1.2 Cash rolls up; balances do not

A stream belongs to an entity; the entity publishes its total; a member
`part of` a container contributes its cash to the container's; categories
and statements fold across entities. That is three roll-ups, all of cash.
A field rolls up nowhere: a trust's balance is not the sum of its loans'
balances, so every trust restates it by hand (`docs/13` §7.98), and the
credit statement reconstructs "balance outstanding" as original balance
less principal paid — a fold over cash standing in for the fold over
balances it cannot do.

### 1.3 The reasons the balance became a field, and why they do not hold

It was made a field rather than an account because a stream cannot read an
account (§7.105), a pack does not lower an account (§7.76, a lean), and the
default write-off is not a flow an account's `from` could carry. It was
rolled forward from rates rather than from the streams because a field
cannot fold "since my last step" on a daily book with monthly payments
(§7.102). Each of those is a limit of the account as it stands, not a fact
about balances. Generalizing the account removes all of them (§3.6).

## 2. What the industry does, and what parity needs

**The amortization schedule.** One row per period: opening balance,
interest, payment, scheduled principal, prepayment, defaults, closing
balance. Opening equals the prior closing. Closing equals opening less the
reductions. Defaults are a column like any other; survival is not a column
at all. That row is the design.

**Intex** (`docs/38`, `docs/13` §7.74). The collateral view is a set of
books with transitions between them — performing, delinquent, defaulted —
recoveries after a lag, and the realized loss as what remains. Every
trigger and every overcollateralization test reads those balances. On the
liability side a bond carries **deferred interest** (an interest shortfall
accrues and is paid later; a PIK bond capitalizes its coupon) and takes
**principal write-downs** (realized losses allocated to junior classes,
reducing their balance without cash).

**Argus** (`docs/33`). A cash projection tool: contractual rent with a
collection-loss percentage, no receivable, no accrual view. What it does
report is balances — the loan schedule's outstanding balance each period,
reserve balances — and those need to be first-class and foldable.

| movement | Argus | Intex | today |
|---|---|---|---|
| cash (draws, principal, prepayments, recoveries) | yes | yes | streams |
| accrual (capitalized construction interest, PIK, interest shortfall) | yes (construction) | yes | none |
| write-off (realized collateral loss, bond write-down) | — | yes | none |
| balances by entity state (performing, delinquent, defaulted) | — | yes | none — a fold by state |

What neither needs: revenue recognition, receivable ageing, bad-debt
expense, a profit-and-loss or a balance sheet. Those are the accounting
layer; they fall out of the account cheaply (§6) but are not a reason to
build anything.

## 3. The design

### 3.1 The account, rolled by the engine

An **account** is a declared balance, rolled by the engine:

```
opening(t)  = closing(t − 1)          (init at the first period)
closing(t)  = opening(t) + Σ lines that move it in t
```

Nothing in a model or a pack writes an account's value. An account moves
only through **streams** that name it, each with an amount, a period, a
cause (the contract and line that produced it) and a direction. The
construct is the account of `docs/01` §10.6, generalized as §3.6 lists;
the word "ledger" is not used, and the word "role" (`docs/40` §3) goes
with it.

### 3.2 One shape: a stream, with a direction and an optional account

A stream is the only line there is. What is added is one clause,
`moves <account>`, and two words in the slot the direction already
occupies:

| direction | cash? | moves an account? | what it is |
|---|---|---|---|
| `inflow` | yes | optionally | money in — a draw to the borrower, a principal receipt to the lender |
| `outflow` | yes | optionally | money out — a principal payment, an advance |
| `accrual` | no | must | a claim raised without cash — capitalized construction interest, a PIK coupon, an interest shortfall |
| `writeoff` | no | must | a claim extinguished without cash — the realized loss, a bond write-down |

Most streams move nothing: revenue, opex, capex, interest, servicing are
cash that lands in the entity's net cash flow and changes nothing owed.
A stream names an account only when it changes a claim. Scheduled
principal, prepayments, draws and recoveries are all CASH — the balance
falls because the borrower paid, not because a schedule said so. The two
non-cash directions are the only movements that are not.

Whether a cash stream RAISES or LOWERS the balance is not written on the
stream. It follows from the account's **side** (§3.6): a draw is an inflow
to the borrower and raises a loan the borrower owes; the same draw is an
outflow to the lender and raises the loan the lender is due. An accrual
always raises a claim and a write-off always lowers it, on either side. A
modeller never writes increase or decrease, and the compiler refuses a
principal payment that would raise a liability.

A default moves nothing. The borrower who stops paying owes exactly what
they owed; the loan's STATE changes, and with it what happens next —
interest stops arriving, principal stops arriving, and after the recovery
window the recovery is an inflow and the loss is a write-off, both against
the one balance. Intex's performing and defaulted buckets are the fold of
`balance` by entity state: a report, not a second account.

### 3.3 What reads an account, and when

- **A stream reads the OPENING balance.** Interest on what was outstanding
  at the open of the period is the ordinary case, and the opening balance
  is the prior close — settled state, safe to read inside the period. This
  is the read the level-pay streams do today through the field, made a
  language rule: a stream never needs, and never gets, the same-period
  closing.
- **A field reads the prior close**, as it reads `prev.<account>` today.
- **A guard, a trigger, a threshold reads the opening balance.** A
  clean-up call fires when the trust's opening balance is below ten percent
  of the initial; an overcollateralization test compares opening balances.
- **An occurrence captures a balance at a date.** A permanent loan's
  principal is the construction account's opening balance on the
  conversion date, written once into a field by the conversion's action
  from `prev.balance` (§4.2, §7). A term never reads an account directly.
- **A statement reads the closing balance** as a fold over the period's
  lines, or the opening as the prior close — the reporting plane sees both.

### 3.4 Roll-up

A container's account of a given name is the relation's fold of its
members' accounts of that name. A trust holding twelve loans `part of` it
has a `balance` that is the sum of the twelve, opening and closing, with no
field written at the trust. This is the second half of `docs/13` §7.98,
built into the construct: cash and balances roll up through the same
relation, by the same rule.

### 3.5 The master declares the account; the refinement provides its lines

`Contract.Debt` today names a field "role" `balance`. That vocabulary goes.
The master declares an **account** by name, as it declares a line:

```toml
[[contracts.accounts]]
name = "balance"
description = "Outstanding principal — what the borrower owes at the open of the period."
```

A refinement's lowering rules say which lines move it and how:

```toml
[[rules]]
id = "credit_loan_sched_principal"
line = "principal"
direction = "inflow"
account = "balance"
```

`direction` accepts the same four words a model stream does; `account`
is the only new key.

Load checks: every account the master declares has at least one line that
moves it; a line that names an account names one the type carries; a pack
field named the same as an account is refused. The machine's
`on enter retired { set balance = 0 }` becomes a **write-off line for the
opening balance**, journaled like any line, so extinguishing a claim is a
happening with an amount and a cause rather than a write into state.

A model with no contract declares an account directly and moves it with its
own streams:

```cfdl
entity asset loan : Asset.Financial {
  account balance owed init 1000000
}

stream loan.draw      on entity asset.loan inflow   { ... moves balance }
stream loan.principal on entity asset.loan outflow  { ... moves balance }
stream loan.pik       on entity asset.loan accrual  { ... moves balance }
stream loan.loss      on entity asset.loan writeoff { ... moves balance }
stream loan.interest  on entity asset.loan outflow  { ... }
```

Five streams, one shape, one new clause, two new words in an existing
slot, one word on the account. Interest moves nothing.

### 3.6 The account, generalized — the whole change

The account of `docs/01` §10.6 is a cash location: fed by settled cash
through `from` or by waterfall steps, owned by the structure or by one
party, read by fields as `prev.<account>`. Four generalizations make it the
balance construct as well, and they are the entire language change:

1. **An entity may own an account.** The subject of an agreement carries
   the agreement's balance. `owner` is unchanged for party accounts.
2. **Streams may move it, in any of the four directions.** `from` remains
   the expression-fed spelling for a cash pot; `moves <account>` on a
   stream is the general one, and `pay … to account` is already it.
   An account has a **side**, `owed` or `due` from its owner's view, and
   the effect of a cash stream follows from its direction and that side.
   For a pack contract the side comes from the MASTER — `Contract.Debt`
   says the balance is owed by the borrower to the lender, and the pack
   contract already states which role its subject plays: the CRE
   permanent loan on the owner is owed, the credit pool on the trust is
   due — so no pack and no modeller using one ever writes it. The
   one-word declaration exists for a model with no contract.
3. **A stream may read its opening balance.** Today refused (§7.105); the
   opening is settled, so the refusal was of the wrong value, not of the
   read.
4. **It folds through the relation** (§3.4).

A note holder's distribution account (party-owned, fed by steps) and the
note's claim account (entity-owned, moved by those same steps and by
write-downs) are then two accounts with different owners and different
lines. A step pays `min(remaining, opening claim)`.

### 3.7 What does not change

Cash aggregation folds `inflow` and `outflow` streams only. `net_cash_flow`, `model.total`,
IRR, MOIC, NPV, WAL and every statement that exists today read exactly what
they read now. The account adds the balance plane beside the cash plane; it
takes nothing from the cash plane.

### 3.8 There is no pool construct

The account is declared on an entity and moved by the agreements the
entity holds, so the design is neutral about how a deal is arranged, and
none of the following needs a construct of its own:

- **The security alone.** A note has a face, a coupon and a claim account,
  moved by what it is paid. Nothing beneath it.
- **The loans beneath it.** Each loan has its balance account; the trust
  folds them; the note's claim is moved by the waterfall that allocates
  what the fold produced.
- **Scheduled beside actual.** Two containers of the same loans, or one
  container carrying a scheduled account and an actual account side by
  side, the pass-through reading whichever the security's terms say.
- **A representative loan.** One `Contract.Debt` refinement whose
  payments, prepayments and defaults are fractional because it stands for
  many. This is what the credit pack's rate-driven pools already are, named
  honestly. It is a loan, and the trust folds it as it would fold a tape.
- **The lowest level, rolled up.** A tape of loans, each a contract, each
  with its account, folded by the container.

Where a modeller wants a Pool, it is a container they named Pool. The
design owes nothing to it.

## 4. The agreements under the account

### 4.1 The credit pack

`credit.pool_level_pay`, `pool_io_bullet` and `pool_float_io_bullet` become
representative loans, each carrying ONE account, `balance` (due — the
subject is the lender's side): init `principal`; lowered by scheduled
principal, prepayments and recoveries (inflows); lowered by the loss
(write-off) after `recovery_lag_months`; the interest-only families add
the bullet. Interest and servicing move nothing.

The defaulted fraction is not an account. In a loan-level model it is the
loan's state, and the pool's "defaulted balance" is the fold of `balance`
over loans in that state. In the representative loan the pack keeps
whatever private field it needs to report the fraction in default, as it
keeps a lagged twin today. The survival field and the stopgap balance
field disappear; the pool factor is `balance` over `principal`, read from
the account rather than reconstructed in the statement.

The trust folds its loans' `balance`. The clean-up call is one event at
the trust reading its opening `balance`; retirement is a write-off on
every loan, journaled. The overcollateralization target and every
Intex-style trigger read the same number and, where they need the
buckets, the fold by state.

`credit.note` carries a `claim` account (owed — the subject is the issuer's
side): init `face`; lowered by principal allocated to the holder (the
step's cash line, attributed to the contract and line it pays); raised by
an interest-shortfall accrual; lowered by a write-down (write-off) when
the waterfall allocates a loss. That is the deferred-interest and
write-down behaviour `docs/38` lists and no case can express today.

### 4.2 Construction lending: capitalized interest and conversion

Construction interest is the case that tests whether the balance is really
the claim, and the pack should be explicit about three things.

**How interest is paid during construction.** A term selects among the
arrangements a modeller actually meets, and each is a different kind of
line on the same account:

- *Capitalized into the loan*, funded from an interest reserve in the
  budget. The balance grows by the interest each period, no cash moves, and
  the claim is settled at payoff or conversion. An `accrual` stream moving
  `balance`. This is the common development-model case.
- *Paid currently* from equity, or from operations once in service. An
  `outflow`; the balance does not grow.
- *Drawn from a funded reserve.* The reserve is a party account holding
  cash; the interest is a cash line paid out of it; no accrual.

**Takeout by a separate permanent loan.** Two contracts. At conversion the
permanent loan's proceeds are a cash line in and the construction loan's
payoff is a cash line out; the construction `balance` closes to zero
through that payoff. The permanent loan's `principal` is the construction
account's **opening balance on the conversion date**, capitalized interest
included, plus any fees rolled in — a field the conversion occurrence
captures from `prev.balance` (§7), which the permanent contract's term
then reads. The borrower's net cash at conversion is the
fees; the construction lender is repaid and the permanent lender funds.

**Conversion of the same note.** One contract, one account, no line at
conversion. The loan moves from a construction phase to a permanent one:
draws stop, accruals stop, the rate may switch from floating to fixed, and
amortization begins on the account's opening balance at the conversion
date — that balance is the amount financed for the level-pay schedule, so
the payment is computed from it, not from the commitment. A lifecycle on
the loan with the lines active by state, and an entry action capturing the
amount financed from `prev.balance` for the amortization rule to read.

In both, the capitalized interest is repaid in cash through the permanent
amortization, which is what makes it a claim and not a loss. Conversion
never writes the balance: it either pays it off with a line or carries it
forward unchanged.

### 4.3 The other debt refinements

`cre.permanent_debt`, `opco.term_debt` and `energy.project_debt` provide
`balance` with their existing lines: draws increase it, scheduled principal
decreases it, a balloon decreases it to zero. PIK on the OpCo notes is an
`accrual` stream. No closed form is restated anywhere.

## 5. Sequencing

Separate from the engine restructure, and before it.

1. **Language and engine.** The four generalizations of §3.6, the two
   non-cash directions, the read rules (§3.3), the results series
   (`account.<name>` opening and closing), the journal lines, the category
   roots the non-cash directions need (`docs/35`). One PR for the
   construct, one for the fold and results. **The construct is built**
   (4 September 2026): an entity owns an account with a side and an init,
   streams `moves` it in all four directions, a stream reads the opening
   as `prev.<account>`, every movement is a `move` journal line, non-cash
   streams enter no cash total, and `E1378`–`E1382` refuse the malformed
   cases. **The relation fold is built** (same day): every ancestor of an
   entity that declares a claim carries the sum of its members' claims of
   that name, opening and closing, declared nowhere and moved only through
   a member (`E1383`, `E1384`). Not yet: the master-declared account and
   the pack rule's `account` key (§3.5), the `set balance = 0` retirement
   as a write-off, the non-cash category roots.
2. **The level-pay loan onto accounts**, replacing the stopgap field.
   Numbers unchanged: the lines are the same, only who sums them changes.
3. **The clean-up call** on the auto ABS cases and AmeriCredit, reading the
   trust's fold. The first demonstration.
4. **The note's `claim`** with shortfall and write-down; REMIC tranches as
   notes.
5. **Construction lending** per §4.2, with a conversion case.
6. **The other families and packs**, as their cases need them; then the
   loader requires every `Contract.Debt` refinement to provide `balance`.

## 6. What falls out later, and is not a goal

A profit-and-loss is a statement over `accrual` and `writeoff` streams; a
balance sheet is a statement over account closings; a receivable is an
account raised by rent due (accrual) and lowered by rent received (inflow)
and by write-offs. All of it is expressible once the generalized
account exists, and none of it is asked for by any parity target.
`docs/35` §6 holds the open question of a model-declared statement; this
note adds nothing to it.

## 7. Open questions

- **Settled (4 September 2026).** Streams name the account they move
  (`moves <account>`); `from` stays for expression-fed cash pots. One
  shape: a stream, whose direction is one of `inflow`, `outflow`,
  `accrual`, `writeoff`. No `kind`, no `effect`, no `transfer`, no second
  account for defaults. An account's side comes from the master.
- **Settled (4 September 2026): the opening is `prev.balance`.** Inside a
  period the only balance anyone can read is the prior close, and it is
  always spelled `prev` — the spelling fields already use; streams gain
  it, and nothing gains a same-period close. Across entities,
  `prev.asset.loan.balance`. A lowering rule reads `prev.balance` against
  its subject, since the master names the account.
- **Settled (4 September 2026): `init` is the balance at the timeline's
  first period, defaults to zero, and `prev.balance` is never absent.**
  A balance outstanding when the model opens states it (the pool at
  cut-off, init `principal`); a balance created during the run does not,
  and the cash that creates it — the permanent loan's proceeds, the
  construction draws — is an inflow that raises it from zero. No
  `prev.balance.init`: the initial amount is the declaration or the term.
  This also removes the period-zero guard the ABS pilot wrote before
  every step: a party account without `init` opens at zero.
- **Settled (4 September 2026): a balance at a date is a field an
  occurrence captures.** No new capability. The conversion is an event or
  a lifecycle transition, and its action writes the field once from the
  prior close: `on enter permanent { set amount_financed = prev.balance }`.
  The same-note conversion amortizes from that field; the takeout's
  permanent loan reads it as its `principal`. A term is an INPUT to the
  lowering, not a stream: a term that is an expression is spliced into
  the rules' per-period formulas, so `principal = prev.asset.tower.balance`
  would re-read a balance the loan itself pays down — a pack term slot
  that must be constant refuses an expression that reads an account, the
  way `E5026` refuses an expression in a literal slot. The account is
  never written by either path. The conversion date is therefore an
  occurrence the model declares, journaled and movable by a scenario,
  not a term on the loan.
- **Cadence.** An account on a daily book with monthly lines rolls every
  model period, most with no lines; the opening the monthly interest stream
  reads is the close of the prior day, which is the prior payment's close.
  Confirm this dissolves §7.102 for the balance.
- **A claim moving between entities.** A sale of a loan moves its balance
  from one owner to another. Out of scope here; note it.
- **Rounding.** A `round_step` on an account, or on the lines only.

## 8. Related

`docs/13` §7.74 (Intex scope), §7.76 (accounts), §7.96–§7.98 (the pool
balance, the clean-up call), §7.101 (a stream cannot fold a field),
§7.102–§7.106 (filed from the level-pay work); `docs/15` §7.8 (a non-cash
quantity is a flow kind excluded from cash aggregation); `docs/28` §4
(state reads are backward); `docs/35` (the taxonomy); `docs/38` (Intex);
`docs/40` §3 (state owned by the agreement).
