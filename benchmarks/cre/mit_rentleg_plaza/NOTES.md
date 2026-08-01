# Rentleg Plaza — what building this against CFDL surfaced

Internal validation notes. The model reproduces MIT OCW 11.431J PS1 exactly
(every pro forma line, and PV = $2,292,810.18 against a published $2,292,810).
These are the places the language or the CRE pack pushed back on the way there.

## 1. RESOLVED — the CRE pack now lowers on any calendar

Originally: every rule divided annual figures by a literal 12 and anchored on
`months_between`, so a `cre.lease_unit` on an annual grid paid one twelfth of
its rent once a year, silently. Nothing rejected it.

Fixed. All four packs are cadence-neutral; `tools/cadence-parity.py` asserts
that one deal produces the same annual figures on every calendar. This
benchmark's own five-months-free term was the decisive case for the pro-rating
policy — `_months` means calendar months on every grid, so five months is
0.417 of an annual period and year one is 480,000 x 7/12.

## 2. RESOLVED — the projection tail is reachable from a native model

Originally: `E2103_SCHEDULE_OUT_OF_BOUNDS` measured a native stream's schedule
against the cash horizon and excluded the `project` tail, while a pack-lowered
stream was not bounds-checked at all. So this model could not read a forward
year and carried its 2006 NOI as an inline closed form, duplicating the opex
formula.

Fixed. The bound now spans the evaluation window and is mirrored onto lowered
streams. The reversion derives 2006 NOI from the modelled streams over
`project 1`, and the duplicated formula is gone.

## 3. RESOLVED — disposals settle at the end of their period

Originally: `cre.exit_forward` lowered to an `on_date` schedule, which
discounts from the START of the period the date falls in. A sale placed in
year 5 was discounted four years, not five. Using the pack contract here gave
$2,500,593.30 against the published $2,292,810 — a gap of exactly
`reversion / 1.12^4 - reversion / 1.12^5`.

Fixed. A lowering rule may now declare `schedule_at_period_end`, and the
disposal rules in cre and opco do. Acquisitions, funding draws, dated TI/LC and
tax credits keep the original placement, because for those "on its stated date"
is right — the defect was treating every one-shot flow alike.

The in-house reference generators had the same conflation, having been written
alongside the engine. That is the failure mode `tools/analytic-checks.py` was
created for: two implementations agreeing is not evidence when both came from
one assumption. The external published figure was the tiebreaker, which is the
argument for this benchmark existing at all.

Correcting it moved eight CRE and opco models by about a period of discounting
— office_two_tenant's NPV fell 0.93%, the right direction for a reversion that
now waits one more month.

## 4. Pack gaps that remain — tracked in the backlog

Two CRE pack features this deal needs and does not have, which is why its opex
and recoveries are still native streams while everything else runs through pack
contracts:

- occupancy-varying opex (MIT fn 7 splits it 81% fixed / 19% variable, which is
  what makes 2001 opex $135,161 rather than $144,300);
- an expense stop that resets to a computed later-year value (fn 5, which makes
  the 2004 reimbursement exactly zero).

Both are recorded in `docs/13_feature_backlog.md` §1.1 and §1.2, along with the
abatement-line gap that was §5 here. They are additive features, not defects.

## 5. RESOLVED — the playground shipped a stale engine

`site/public/wasm/` was built five days and four breaking grammar changes
before HEAD, and rejected every `schedule every <interval> from ...`. A
freshness gate existed and had never fired: `site.yml`'s `paths:` filter
excluded `crates/**`, and every checkout is shallow so the gate's `git diff`
threw and it exited 0.

Fixed and gated four ways — engine version literal, a source hash, a
functional smoke test over the shipped bundle, and a wasm32 build in CI.
Rebuilding it immediately exposed a second bug it had been masking: seven run
configs still used `discount_rate`, renamed to `annual_discount_rate` some
releases earlier, so every language-tutorial example in the playground was
erroring.

## Why this stays annual

The published $2,292,810 is annual by construction — MIT fn 12, "Assumes first
cash flow occurs 1 year from present." A monthly rebuild of the same deal is a
legitimate model but a different number: spreading the same cash across months
and discounting at `(1.12)^(1/12)-1` gives ~$2,323,050, about +1.3%. Rebuilding
it monthly would trade an externally-verified figure for a self-graded one.
