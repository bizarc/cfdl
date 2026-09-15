# Notes — penzance_one_rosslyn

Working notes, not published. Reader-facing prose is in `CASE.md`; the
tolerances and their reasons are in `case.toml`.

`model.cfdl` is AUTHORED, not generated. Every figure it uses is an `assume`
labeled with its standing, and everything else — the budget, both
commitments, the draw curve, each tower's lease-up, the value the sale is
priced on, the permanent loan and its payoff — is arithmetic on those figures
inside the model. `inputs/one_rosslyn.toml` is the provenance record the
assumptions cite; it is not read by anything. Edit the model.

## Authored in full (2026-09-14)

Until this date the model was written out by a generator that computed six
period tables in Python — the cost curve and its running total, the blended
occupancy, the condominium proceeds, the lease-up sale and the permanent loan
proceeds — and pasted them in as `curve` blocks, together with the sale
price, the loan principal, the payment and the payoff as literals. The engine
saw the numbers and none of the derivation: changing the growth rate moved
the build-to-core exit (the one figure the model derived, through the tail)
and left the lease-up sale, the permanent loan and every curve where they
were. Everything now derives in the model:

- **The budget and both commitments** are derived assumptions (docs/03 §2.1)
  from the Highlands unit costs, the BLS escalation, the ratios and the
  loan-to-cost: `construction_cost`, `equity_commitment`, `loan_commitment`.
- **The draw curve** is the parabola stated in closed form. An `asset.program`
  entity holds the month's share of the weights, the month's whole cost and
  the month's proceeds, each ONE PERIOD AHEAD, so the cost streams and the
  facility read them as `prev.asset.program.<field>` — a field may read
  another field only at the prior close (`E1127`), and this is the idiom
  that turns that rule into one statement rather than five. The weights sum
  to n(n+1)(n+2)/6 = 13,244 for n = 42.
- **Each tower leases up on its own streams**, on its own entity, from its
  own delivery: `cre.rent.nw` / `cre.rent.south`, and parking and opex
  likewise. Occupancy is `units * clamp((t - delivery + 1) / 18, 0, 1) *
  (1 - vacancy)` and is readable per tower — at the lease-up sale the south
  tower is 11/18 leased, 58% net of the allowance. The blended figure the
  exit reads is the sum, through `series_sum("cre.rent.*", ...)`.
- **The lease-up sale** is `exit_a_value`: stabilized NOI by the County's
  method (every rental unit's rent and parking net of the allowance, less
  guideline expenses on every unit), escalated to the exit month, over the
  cap. It now moves with growth, the cap rate and the rent, as the hold's
  exit always did.
- **The permanent loan** is `perm_principal` (the same stabilized value at
  the refinance month, times loan-to-value) and `perm_payment` (`pmt` over
  360 months). Its balance is `asset.perm.balance`, a recurrence; the payment
  is split into `cre.perm_interest` and `cre.perm_principal` so coverage
  reads both legs, and `cre.perm_payoff` reads the balance after the last
  payment. The single `cre.perm_debt_service` line is gone; the debt-service
  subtotal is unchanged because both legs fold into it.
- **The equity funded to date** stays a field on `asset.facility`, in closed
  form: the cost incurred through month t, capped at the commitment. The
  cumulative parabola is (n+1)m(m+1)/2 - m(m+1)(2m+1)/6 for the m months
  elapsed.

**What moved.** Every input the old generator printed reproduces exactly:
construction 555,953,893.51, equity 249,668,882.60, loan 333,572,336.11,
exit A 567,404,297.94, permanent loan 347,217,832.21, payment 2,183,247.96,
payoff 324,846,432.07, condominium month 5,387,833.29. The only differences
in the results are the fourth-decimal rounding the old occupancy curve
carried (`505.9278` for 505.92777…): the largest per-period movement is
$0.20 on effective gross income, and the deterministic net moved from
223,799,127.44 to 223,799,127.60 — which is the workbook's own figure
(`Returns!C20`, 223,799,127.599258) to six decimals. Scenario A moved from
28,591,605.61 to 28,591,605.50, the workbook's `Returns!B20` to six
decimals, and its worst period against the workbook column fell from 0.16
to 0.005. `expected.csv`, `expected_metrics.json` and
`expected_scenarios.json` are the model's own regression figures and were
regenerated; `expected_merchant_build.csv` is the workbook's column and was
not touched.

**Three engine limits met on the way, and why the facility is still a
recurrence.** The intended shape (docs/42) is a `facility_balance` account
rolled from the loan streams, each stream reading `prev.facility_balance`.
Built that way the model compiled, ran without a warning, and read the
balance as ZERO in every stream: a stream's `prev.<account>` read is a silent
zero in any model that carries a forward-priced amount (the tail-valued sale
is one), and correct in a model that does not. Filed as docs/13 §7.124. A
field cannot take the read instead: `E5035` refuses `prev.<account>` in a
field rule in any priced model, whatever the account's members (§7.123). And
a field may not read a sibling field at the same period (`E1127`, by
design). So the facility is `equity_funded`, `interest`, `draw`, `repay` and
`balance` as before, reading `prev` and the program's schedule, with the
month's draw spelled out where the repayment and the balance need it. The
pack limits are §7.121 (no multifamily absorption on `cre.lease`) and
§7.122 (`cre.construction_loan` takes a curve name, not an expression).

## The JV split (2026-09-12)

The companion case's construction carried over: contributions as streams that
move each partner's due account, the venture's cash as an account fed by every
`cre.*` stream, a seven-tier waterfall, and `irr`/`moic` per party. Three
things differ here, and each was forced by the case rather than chosen.

**Two exits, two waterfalls.** A waterfall's schedule is a date, and one model
carries two strategies selected by `inputs.scenario_b`. So there are two
waterfalls, `jv.distribution_a` on 2032-05 and `jv.distribution_b` on 2037-06,
and every tier is weighted by the scenario switch. The inactive waterfall pays
zero on every tier and leaves the pot untouched; `expected.csv` asserts those
zeros at period 102 under scenario B, and `expected_merchant_build.csv` asserts
them at period 163 under scenario A.

Scenario A distributes at the **last condominium closing**, not at the tower
sale. The sale is 2031-10 (t=95) and the sellout runs to 2032-05 (t=102); the
closings after the sale are the venture's cash too, and a distribution at the
sale would leave them in the pot. The companion case distributes at its last
closing for the same reason.

**Capital and the preference come back pro rata.** The companion pays the
investor's capital first, then the sponsor's, then the investor's preference,
then the sponsor's. Its pot is never short, so the order never binds. Here it
binds in five of the seven runs:

- At the guideline basis the deal returns 6.57% (B) or 2.02% (A), below the 8%
  preference, so the pot is short of the preference in both strategies.
- At the 2026 discount under scenario A the pot (212,441,589.65) is short of
  the capital itself (249,668,882.60).

Paid investor-first, the 2026-discount run returned Baupost 191,197,430.68 and
Penzance nothing, and the engine refused `irr(party.penzance)` — a party that
never received anything has no return to solve. Paid pro rata, both partners
recover 85.09% of their capital and the metric is defined. The first tier of
each pair is therefore capped at the investor's share of the pot:

```
pay capital_inv   = min(0.0 - prev.baupost_capital, remaining * (1.0 - inputs.sponsor_share))
pay preferred_inv = min((asset.jv.unreturned - asset.jv.capital) * (1.0 - inputs.sponsor_share),
                        remaining * (1.0 - inputs.sponsor_share))
```

The second tier of each pair reads `remaining` as before: when the pot is not
short, `remaining` after the investor's tier still covers the sponsor's full
amount, and when it is short, `remaining` is exactly the sponsor's share.
Below the promote the partners are pari passu, which is what the two capped
tiers say. On the companion case the same spelling would change nothing.

**The preference accrues from construction start** (2026-12, t=37), the
companion's convention, stated in the model. Nothing in the record fixes it.

## Cross-check: the tiers, recomputed

Every tier was recomputed outside the engine from three numbers the run
publishes — the pot at the distribution (`allocate_out` in the journal), each
partner's capital (its account balance the period before), and the accrued
preference (`asset.jv.unreturned - asset.jv.capital` at the distribution) —
by walking the seven tiers in order. The largest difference from the engine's
journaled payments, across all six named runs, is 0.000001.

| run | pot | preference owed | promote |
|---|---:|---:|---:|
| build_to_core | 473,468,010.04 | 297,775,808.20 | 0 |
| merchant_build | 278,260,488.21 | 115,348,263.97 | 0 |
| merchant_at_2026_discount | 212,441,589.65 | 115,348,263.97 | 0 |
| merchant_at_2022_premium | 460,397,267.85 | 115,348,263.97 | 19,076,024.26 |
| core_at_2026_discount | 393,297,229.92 | 297,775,808.20 | 0 |
| core_at_2022_premium | 695,319,910.21 | 297,775,808.20 | 29,575,043.88 |

Capital is 224,701,994.34 (Baupost) and 24,966,888.26 (Penzance) in every
run; it depends on the cost curve alone.

**A check worth keeping:** in every run where the pot is short of the
preference, the two partners' `moic` and `irr` are identical to six decimals,
because pro rata below the promote means each receives the same fraction of
what it put in. Where the promote is paid they diverge, and the whole
difference is the promote.

## What the accounts conversion did not move

`slice.deal` is byte-identical to the old `net_cash_flow` column in
`expected.csv` (maximum difference 0.0 over 164 periods), and `slice.deal.irr`
and `slice.deal.moic` equal the old `model.irr` and `model.moic`. An account is
not cash. `model.total` now carries the 249,668,882.60 of contributions as
well, which is why the case asserts the slice.

## The workbook's scenario A column

`expected_merchant_build.csv` is the `Equity CF` column of the workbook's
ScenarioA sheet, 103 periods (0 to 102), checked against a run with
`inputs.scenario_b = 0`. The largest difference from the engine's `slice.deal`
is 0.16, so the slice tolerance in `case.toml` is 1.0 rather than the case's
0.01. The deterministic column still ties to 0.0; the tolerance is loosened
for the workbook's rounding, not the engine's.

## An observation on the engine, not filed

A declared party metric that is undefined in **any** named scenario fails the
whole run with `E5031`, even when the deterministic run evaluates it. A
partner wiped out in a downside scenario is a real outcome, and a downside
scenario exists to show it; `moic` of 0.0 is a defined answer there and the
engine could report it, with only `irr` undefined. This case avoids the
question by paying pro rata, which is the better convention anyway. Recorded
so the next case that meets it knows it is the engine, not the model.
