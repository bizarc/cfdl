# Notes — penzance_highlands

Working notes, not published. Reader-facing prose is in `CASE.md`; the
tolerances and their reasons are in `case.toml`.

`model.cfdl` is WRITTEN by `reference_gen.py` from `inputs/`: the writer
fills the assumption block, the schedule dates and the one curve (the recorded
closings) from the inputs file and emits the model text, and calculates
nothing. Every figure the model can derive from another it derives itself.
Edit the inputs or the writer's template, then regenerate — a hand edit to the
model survives until the next regeneration and then vanishes. The template is
a `string.Template`, so a literal dollar sign in a comment is written `$$`.

## Authored in full (2026-09-14)

Until this date the generator computed three period tables in Python — the
cost per month, its running total and the proceeds available to the facility
— and pasted them in as `curve` blocks, and printed the equity commitment,
each tower's rent per occupied unit, the retail, parking, expense and tax
lines and the sale costs as literals it had multiplied out. The engine saw
the numbers and none of the derivation. Everything now derives in the model:

- **The budget** is seven stated lines summed to `construction_cost`; the
  obligations are the site plan's stated items summed to what is due at
  permit and at certificate of occupancy; `equity_commitment` is total
  development cost times a stated share, 0.35, which is the calibration the
  old literal 186,245,280.585 encoded (532,129,373.10 × 0.35).
- **The draw curve** is the parabola in closed form on an `asset.program`
  entity, held ONE PERIOD AHEAD so the cost streams and the facility read it
  as `prev.asset.program.<field>` — a field may read another field only at
  the prior close (`E1127`). The weights sum to n(n+1)(n+2)/6 = 10,660 for
  n = 39.
- **The facility's proceeds** are the program's `proceeds_ahead`: each
  closing read from the `pierce_sellout` curve net of its selling cost, gated
  to the closings' own months because a step curve holds its end values
  outside its points, plus the two sales net of the cost of sale in the sale
  month. The `loan_proceeds` table is gone.
- **Each tower's rent** is `units leased × derived rent × (1 + other income)
  × (1 − vacancy)`, with the lease-up `min(units, max(0, (t − (delivery − 1))
  × pace))` from the stated pace; retail is area × rent × occupancy; parking
  is units × ratio × rate; expenses are units × the Guidebook figure; tax is
  the assessed value × the effective rate. The old literals (3369.8496,
  35236.8, 39720, 207178.4167, 215699.0142 and their Evo counterparts) were
  these products.
- **The equity funded to date** stays a field on `asset.facility`, in closed
  form: the cost incurred through month t, capped at the commitment. The
  cumulative parabola is (n+1)m(m+1)/2 − m(m+1)(2m+1)/6 for the m months
  elapsed.
- **Month numbers** are `months_between` over the stated dates; the dates
  appear once in the inputs file and the writer copies them.

**What moved: nothing beyond the fourth decimal.** The old tables were
printed to four decimals; the closed forms are exact. The largest per-period
movement in `expected.csv` is 0.005 on a waterfall tier at the distribution,
the deal's net moved by 0.0015 and `model.total` by 0.0035, every return and
multiple is unchanged to six decimals, and the run carries no warnings. The
expectation files are untouched: every column is within its tolerance.

The One Rosslyn conversion (its NOTES.md, 2026-09-14) records the four
engine and pack limits met on the way and why the facility is a recurrence
rather than the docs/42 account; the same reasons hold here, without the
priced-amount complication, since this model prices nothing from a tail.

## The accounts conversion (2026-08-29)

Converting the JV split onto `account` and `metric` — `docs/31` W4 phase 3,
the migration `docs/29` phase 4 names as "Highlands' cumulative window".

**The cumulative window became an account, exactly.** The pot was
`series_sum("cre.*", 0, time.t)`, a running sum written out at the distribution
because `available` is one month. `account deal_cash { from series_sum("cre.*",
time.t, time.t) }` accumulates the same thing, and on the first run every
shipped figure was byte-identical: `model.total` 196,361,512.478008,
`model.irr` 0.110161, `model.moic` 2.04664, and both payee totals. That is the
identity `docs/28` §5.1 claims, measured on a 160-period case.

**Then the returns came out wrong, and the reason was in CASE.md all along.**
The first run published `moic(party.baupost)` = 0.959618 — Baupost losing money
on a deal returning 2.05x. Short by exactly 1.0, and the cause was the pot: the
old text recorded that "there is no return-of-capital tier ... contributions are
outflows inside those streams, so the running sum has already recovered the
capital". A profit-only pot cannot return capital, so a party account carrying
the contribution as a negative inflow had no offsetting leg.

That shortcut is sound for splitting profit and incompatible with measuring a
position. The fix grosses the pot up — net stream cash **plus** the equity that
funded it — and adds two return-of-capital tiers off the top. It changes no
split: every tier below sees the same `remaining`, and both MoICs moved by
exactly +1.0 (0.959618 → 1.959618, 1.906607 → 2.906607). The payee totals grew
by exactly each partner's capital, 167,620,752.53 and 18,624,528.06 = the 90/10
of 186,245,280.59. `model.total`, `model.irr` and `model.moic` never moved,
because an account is not cash.

**A check worth keeping:** each party account's FINAL BALANCE equals that
party's old profit-only payee total — 160,851,859.44 and 35,509,653.04 — because
the capital leg nets out. The old assertion survives as the new balance.

## Two things a reviewer will ask about

**An account's `from` carries no entity state.** `account_inflow_at`
(`crates/cfdl-engine/src/lib.rs`) builds its environment with
`build_expr_env(ir, None, …)` and adds only series; entity fields are never
applied. The first attempt fed the accounts from a field on `asset.jv` and the
run refused it — the engine is loud about this, not silent: a probe reading
`asset.deal.contribution` from an account's `from` fails with
`unresolved name: 'asset.deal.contribution' is not declared — each read as
zero`, naming the field.

It is a RUN-time refusal, not a compile-time one — the probe compiles clean.
The account-inflow environment's shape is known to the compiler, so this is a
candidate for the same treatment `E1355`/`E1356` gave the participant-return
reads: refuse at compile time, where the modeller is. Recorded as an
observation, not filed — the refusal exists and is clear.

The contributions are therefore spelled from `dev_cost_cum` and
`inputs.equity_commitment`, the same arithmetic `asset.facility.equity_funded`
performs. That is forced rather than preferred: equity is not a stream in this
model (making it one would put 186m of contributions into `model.total` and
break the workbook tie), and an account's inflow can read series but not entity
state, so neither route to the model's own equity figure is open.

**Where the accounts ARE the record.** The two return-of-capital steps read
`prev.baupost_capital` / `prev.penzance_capital` directly: a party's unreturned
capital IS its account balance while that balance is negative. What accounts
cannot do is accrue, so `asset.jv.unreturned` stays a field recurrence for the
compounding preference. Accounts hold the capital record; the recurrence holds
the preference.

**Two contribution timelines, one period apart.** The accounts use the true
funding dates. `asset.jv.capital` recognizes the same dollars one period later
— its increment is `prev.asset.facility.equity_funded - prev.asset.jv.funded_prev`,
which is period *t−1*'s funding, because a field's `next` can only read `prev`.
The totals are identical and both have long converged by the 2024-06
distribution, so the preference is unaffected; the difference would matter only
to a waterfall that distributed during the funding period.

**`deal_cash` dips to −1,366,128.27** mid-build, because equity funds against
cumulative cost while the loan draws against period cost. Legal (`docs/28` §5.1:
a negative balance has no floor; what is floored is what a step may take) and
immaterial — the only allocation is at period 153, against a balance of
382,606,793.06.

## Capital-weighted MoIC does not equal `model.moic`

382,606,793.06 / 186,245,280.59 = 2.05430, against `model.moic` 2.04664. Not a
discrepancy: `model.moic` is a fold over the deal's STREAMS, and equity is not a
stream here — it enters as the facility's `equity_funded` field, reducing the
draw. The party figure is a fold over accounts. Two different denominators,
both correct for what they measure.

## Why equity is not a stream, measured

The obvious tidy-up is to declare the equity contribution as a stream, which
would let `deal_cash` read it through `series_sum` and drop the gross-up. It was
tried in scratch (category `financing.equity.contribution` — `financing.equity_contribution`
is refused by `E5022`; the pack's category list is the authority) and it is
wrong, decisively:

| metric | equity not a stream | equity as a stream |
|---|---|---|
| `model.total` | 196,361,512.48 | 382,606,793.06 |
| `model.npv` | 9,058,409.10 | 135,999,657.99 |
| `model.irr` | 0.110161 | 809,072,402.74 |
| `model.moic` | 2.04664 | 281.07 |

A `financing` category keeps it out of the pack's operating folds (`domain.cre.*`
is unaffected), but NOT out of `model.total`, `model.npv` or `model.irr` — those
read `valued_streams` with no category filter
(`crates/cfdl-engine/src/lib.rs`, model metric construction).

The IRR is the substance, not the NPV. **`model.irr` is the equity IRR precisely
BECAUSE equity is not a stream**: the construction-phase deficits in the stream
vector ARE the equity investment. Declaring equity as an inflow cancels that
investment phase, so the solver is left with a near-zero early vector and a large
positive late one and returns garbage.

Hence the division the model actually rests on:

- the STREAM VECTOR is the equity INVESTMENT vector — what `model.irr` and
  `model.moic` measure, and equity must stay out of it;
- the ACCOUNT is the venture's CASH POSITION — what the waterfall allocates,
  and equity must be in it.

The gross-up in `deal_cash` is that second object being given the cash the first
one deliberately does not carry. It is not a workaround for a missing feature.

## Contributions as streams (2026-09-06, D13)

The equity is cash again. Two streams per partner in
`financing.equity.contribution` — the land at period 0 as a one-shot, then
each month's draw as the difference of the facility's `equity_funded` field —
carry the partners' money into the project on the dates the facility draws
it, and each `moves` its partner's capital account, declared `due`, so the
account goes negative as capital is paid in and positive as the split
allocates. The three restated copies of the draw arithmetic are gone;
`deal_cash` is the project's net stream cash and nothing more, because the
contributions are now in it.

What moved and what did not. Every partner figure is byte-identical:
328,472,611.96 and 54,134,181.10 distributed, 1.959618 and 2.906607, 8.1203%
and 12.7552%. `model.total` gained the 186,245,280.59 contributed, and
`model.irr` and `model.moic` stopped meaning anything — a project whose
equity is an inflow has no investment vector at the model level — so the
workbook's levered net cash, return and multiple tie to `slice.deal`, the
project's cash with the contributions excepted, asserted per period as the
slice's column and as `slice.deal.total`, `.irr` and `.moic`. The "division
the model rests on" above is therefore superseded: the equity IS in the
stream vector, and the slice is where the investment vector lives.

Two engine changes were needed and both are general. A party's return
(`irr(party)`, `moic(party)`) folds the journal of the party's account and
counted `inflow` and allocation lines only; a stream that `moves` the
account is journaled as `move`, and the fold now counts it. A slice
publishes `moic` beside `total`, `npv` and `irr`. The observation that an
account's inflow cannot read an entity field still holds and no longer
matters here; the facility's own field is read by the contribution streams.
