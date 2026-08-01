# Banker DCF conventions — what reproducing a disclosed valuation found

The opco pack had **no external validation at all** before this, and it leans
harder on cross-stream machinery than any other pack. This case does not
validate the pack's lowering rules — see the last section for why — but it
validates the layer underneath all of them, which nothing else did: how CFDL
places cash inside a period and discounts it.

## The source

A sell-side banker's discussion materials filed as an exhibit to a merger
document, public on the securities regulator's filing system. Free to read and
citable; the filer retains copyright in the document, so numbers are asserted
against and the file is not vendored or reproduced.

The exhibit uses code names for the parties, and identifying them is an
inference from circumstantial detail rather than something the document states.
This benchmark therefore describes the *analysis*, not the company — the
conventions are what is being validated and they belong to no one.

What made it worth doing is how complete it is. Most fairness opinions disclose
a value range and little else. This one discloses:

- the unlevered free cash flow build-up, line by line, for six fiscal periods;
- the discount rate range (9.5% / 10.375% / 11.25%);
- the terminal value method — a **forward (NTM)** unlevered-FCF multiple of
  20.0x / 25.0x / 30.0x on terminal-year UFCF of $891mm;
- the discounting convention, stated as mid-period;
- the dilution assumption, ~2% cumulative, ramping 0.5%/yr;
- and a 3x3 grid of implied enterprise values, equity values and per-share
  prices.

That is enough to reproduce the analysis rather than approximate it.

## The result

All nine cells of the disclosed grid, implied enterprise value, $mm:

| WACC | 20.0x | 25.0x | 30.0x |
|---|---|---|---|
| 9.5% | 13,494.20 / **13,494** | 16,331.18 / **16,331** | 19,168.17 / **19,167** |
| 10.375% | 13,032.43 / **13,032** | 15,764.16 / **15,764** | 18,495.89 / **18,495** |
| 11.25% | 12,590.76 / **12,590** | 15,221.93 / **15,221** | 17,853.09 / **17,852** |

CFDL first, published in bold. Worst disagreement +1.17 on $19bn — the filing
rounds to whole $mm, and the build-up's own lines round the same way, so ±1 is
the floor the source can support.

The equity bridge (less debt, plus cash, less the present value of deferred tax
liabilities) reconciles to the published implied equity values on the same
figures. It is not modelled here because it is arithmetic on disclosed
constants, not cash flow.

## Finding 1 — mid-period discounting had no spelling

This is the one that mattered. The convention is standard in project finance and
in every banker DCF: a year's cash arrives throughout the year, so discounting
all of it from 31 December overstates the discount by half a year. The filing
states it explicitly.

CFDL could not say it. The engine has always supported a fractional per-stream
offset — `npv_with_offsets` factorises `(1+r)^-offset` out of each series, and
`discount_offset` returns a float — but nothing produced 0.5. `due` gives 0.0,
the default and `on eom` give 1.0, and a day rule gave `n/30`.

Added `mid` as a schedule modifier: `schedule every year mid from …`, and
`schedule on <date> mid` for a one-shot. It is a **convention, not a date**, so
it is half a period on every calendar — which is the difference between it and
`on day 15`.

Pack lowering rules can set it too (`schedule_mid`), so this is available to
every pack rather than only to hand-written streams.

## Finding 2 — `on day <n>` divided by 30 on every calendar

Found on the way to the above, and a genuine defect. `docs/12_payment_timing.md`
says the offset for `on day <n>` is `n / days_in_period`; the code was
`n / 30`.

On a monthly grid those agree. Nowhere else:

| calendar | `on day 15` was | should be |
|---|---|---|
| monthly | 0.500 | 0.493 |
| quarterly | 0.500 | 0.164 |
| annual | 0.500 | 0.041 |
| daily | 0.500 | 1.0 (the date *is* the period) |

It went unnoticed because the only fixture using a day rule is daily, where the
error is half a day. It is the same class of bug as the 42 pack rules that
divided by a literal 12 — a monthly assumption generalised without being
re-derived — but in the engine rather than in a pack, and `tools/cadence-parity.py`
could not have caught it because packs do not emit day rules.

Fixed to divide by the period's own length. One golden moved:
`schedule_day_rules` NPV 148.868921 -> 148.830207, half a day of discounting on
three payments.

Worth noting the accident this created: on an annual calendar `on day 15` gave
exactly 0.5, so the mid-period convention was reachable — by a bug, for the
wrong reason, and only on one calendar. Fixing the defect removed it, which is
why `mid` had to be added in the same change rather than after it.

## Finding 3 — `mid` and payment terms do not compose, and that is now rejected

Payment terms (`net <n>`) and the discount offset are resolved by two different
mechanisms. `net` works on the **calendar**: take the billing date, add the lag,
roll for business days, then find which period the result lands in and move the
cash into that bucket — whole-period granularity, with the sub-period residual
dropped. `mid` is a **discounting convention** applied to whichever period the
cash ends up in.

Combining them silently produces something nobody runs: the cash is placed as
though billed at the period's end, then discounted as though it had arrived
halfway through that period. Composing them properly means billing from the
midpoint *and* carrying the lag's sub-period residual into the offset, which is
a real design question — and picking one answer quietly is how conventions go
wrong.

So `mid` with `net`, with `due`, or with a day rule is
`E2109_SCHEDULE_CONFLICTING_PLACEMENT`. A schedule states one position in its
period, not two.

The general question — that a settlement lag's sub-period part is dropped from
discounting for *every* schedule, not just this one — is older than this change
and is now recorded in the backlog.

## Finding 4 — there is no stub period, and valuation dates do not land on year ends

The valuation date is 30 September and the fiscal year ends 30 June, so the
first forecast period is a nine-month stub and the full years that follow sit at
1.25, 2.25, 3.25 and 4.25 years out. This is not exotic; it is what every live
deal looks like, because valuation dates are set by negotiation and fiscal years
are not.

CFDL calendars are uniform — `for <n>` periods of one length — so a stub cannot
be expressed. The model works around it by dropping to a **monthly** grid and
placing each fiscal year's cash as a one-occurrence flow on the date that
carries its convention:

    stub (9 months)   mid of month 4    4.5/12 = 0.375
    FY+1              end of month 14  15.0/12 = 1.25
    FY+2              end of month 26  27.0/12 = 2.25
    FY+3              end of month 38  39.0/12 = 3.25
    FY+4              end of month 50  51.0/12 = 4.25
    terminal value    end of month 56  57.0/12 = 4.75

Every exponent lands exactly, which is luck: the offsets happen to be month
boundaries except the stub's, and the stub's happens to be a month midpoint. A
valuation dated mid-month would not have this out. Backlogged.

## Finding 5 — a one-shot cannot be placed at its period's end from surface syntax

`schedule on <date>` discounts from the period's **open**. A pack lowering rule
can move it to the close with `schedule_at_period_end` — added when the CRE
reversion turned out to be discounted a period short — but there is no surface
spelling, so a hand-written model cannot say it.

Hence the flows above are written `schedule every month from 2025-12 to
2025-12`: a single-occurrence ordinary annuity, which falls at its period's end.
It produces the right answer and reads like a workaround, because it is one.
Backlogged alongside finding 4.

## Finding 6 — mid-period applies to flows, not to the terminal value

Worth recording because it is easy to get wrong in either direction. The filing
discounts the projected cash flows mid-period and the terminal value **whole**,
from the point in time at which the multiple is struck. That is correct — a
terminal value is a price, not a flow, and a price does not arrive evenly
through a year — and the published figures confirm it: applying the mid-period
convention to the terminal value as well overstates every cell by about 5%.

The model reflects the asymmetry, and `opco.exit.value` carries no `mid`.

## What this case does not validate

Not the opco pack's lowering rules. The filing discloses the UFCF build-up as
**stated figures per fiscal year** — bespoke, not a geometric series — so
`opco.revenue_line`, `opco.opex_line`, `opco.capex_line`,
`opco.working_capital` and `opco.cash_taxes` cannot generate them: those rules
grow a base at a rate, and these numbers do not grow at a rate. The build-up
does reconcile arithmetically to the disclosed UFCF within ±1 per year
(rounding of the individual lines), which confirms the composition and the sign
conventions the pack uses, but it is checked by hand rather than asserted here.

The streams are named into the opco taxonomy so `--pack opco` metrics aggregate
them, which is the same posture `benchmarks/cre/mit_rentleg_plaza` takes for its
native operating lines.

Validating the pack's own rules needs a source that discloses *drivers* rather
than *outputs* — a sponsor model with a stated growth rate, margin path and
working-capital policy. The LBO sources in the catalogue are the candidates.
