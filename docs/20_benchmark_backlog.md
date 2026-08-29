# CFDL — Benchmark Backlog

Status: informative. Coverage the benchmark suite does not yet have.

`docs/13_feature_backlog.md` holds capability the language or a pack cannot
express. This holds something different: things the language **can** express and
the suite has not yet checked — deals read but not modeled, axes of validation
nobody has opened, and assertion shapes that turned out to be weaker than they
looked.

The two overlap at the edges. Where a coverage gap is blocked by a capability
gap, the item says which one and cites it; where a coverage gap produced a new
capability gap, that gap was appended to `13` and is cited here.

Same rule as `13`: each item says what is missing and **what forced the
discovery**. An entry with no provenance is a guess about what would be useful,
and the suite has enough real gaps that guesses are not needed.

Ordered within each section by how much it would prove, not by effort.

---

## 1. Deals read and not modeled

### 1.1 Ginnie Mae REMIC Trust 2026-100 — fifteen groups reconciled, one shipped and withdrawn

A $1.89bn agency REMIC in sixteen security groups. A reference model of it
reproduces **21,570 decrement cells** — every class, at every published PSA
speed, across all 58 decrement tables — with **zero** outside the issuer's own
whole-percent rounding floor, and all **709 published weighted average lives**
exactly.

Only Group 16 was ever written in CFDL, and it was withdrawn before the rest
followed. What the other fourteen groups would exercise, none of which any case
covers today:

| | |
|---|---|
| Groups 5, 7, 9, 11, 12, 13, 15, 16 | seven accrual (Z) classes, whose balances compound on themselves |
| Groups 5, 7, 9, 12, 13, 15, 16 | accretion-directed classes repaid out of a Z coupon, on a schedule prepayments cannot touch |
| Groups 11, 13, 15 | PAC classes against an **published** aggregate schedule (Schedule II), with a support class absorbing the rest |
| Groups 7, 9 | TAC classes against a published targeted balance |
| Group 1 | a pro-rata pair |
| Groups 4, 5 | one security group drawing on two trust asset subgroups with different collateral |
| Schedule I | 63 exchangeable (MX) combinations, including three that span security groups |
| Terms Sheet | 30 notional strips, on classes, on class aggregates, and on trust asset subgroups |

Group 14 is the exception and belongs to §2.1: its only class is notional and
backed by 21 certificates of other deals.

Provenance: modeled end to end in August 2026, validated against the Offering
Circular Supplement dated 24 June 2026, then withdrawn at the author's request
in favor of a different deal. The reference model is not in the repository.

### 1.2 Fannie Mae REMIC Trust 2019-2, Groups 1 and 2 — structured collateral

Group 3 of this deal ships as `benchmarks/credit/fnma_remic_2019_2_g3`. Groups 1
and 2 do not, and the reason is worth recording precisely because it is **not** a
gap in the document.

The instrument is fully specified for both — one line each, the same shape as
Group 3. What is missing is the collateral: their assets are seventeen REMIC and
RCR certificates of Fannie Mae deals issued 2002–2006, so the cash arriving is
another instrument's output. Exhibit A supplies each certificate's January 2019
factor, its balance in the trust, and its pool's WAC, WAM and WALA — the values —
but not those seventeen deals' priority sequences.

Treating each underlying certificate as simply amortizing its own stated balance
gets **all 14 published weighted average lives exact**, and most decrement
columns inside the 0.5pp floor — the underlying PACs are 94–98% retired, so their
supports are long gone and the schedules no longer bind. But three of fourteen
columns land at 0.507–0.532 against a 0.5 floor, so it is an approximation that
nearly reproduces the tables rather than a model of the deal, and it is not
shippable on that basis.

Closing it needs either the seventeen underlying disclosure documents, or the
per-CUSIP factor series of §2.2.

Provenance: found building the Group 3 case, August 2026.

---

## 2. Axes the suite does not cover

### 2.1 Collateral that is another instrument's output

Every case in the suite models collateral: a pool, a property, a project. None
models an instrument whose collateral is **another instrument's output**, where
the pot is supplied rather than derived.

This is not exotic. Fannie Mae's own class-type taxonomy defines `SC`
(Structured Collateral) for exactly it — "designed to receive principal payments
based on the actual distributions on series trust assets representing 'regular
interests' in a separate series trust" — and both REMIC deals read for this suite
contain it: Ginnie Mae 2026-100's Group 14 (21 underlying certificates) and
Fannie Mae 2019-2's Groups 1 and 2 (17).

The language needs nothing new. A waterfall takes `from <expr>`, and a `curve`
can carry a declared principal series, so the shape is:

```cfdl
curve group1_principal { 2019-02: ... }

waterfall g1.principal on entity asset.group1 {
  schedule every month from 2019-02 to 2019-05
  from curve_value("group1_principal", time.date)
  pay cd_principal to party.cd_holders = remaining
}
```

Nothing demonstrates it, and until something does, the suite implicitly claims
that a model must own its collateral. It does not — the separation is the point
of a waterfall — and a case saying so out loud is worth more than the deal it
happens to use.

Provenance: raised while assessing Fannie Mae 2019-2, after two consecutive
write-ups wrongly called this a blocker rather than a compositional boundary.

### 2.2 Realized performance, rather than a published projection

Every case in the suite reproduces a **projection**: a table the issuer computed
at pricing, under assumptions the issuer stated. That is a real and checkable
claim — the model does the arithmetic the issuer did — but it is not the claim a
reader is most likely to assume, which is that the model describes what happened.

Nothing checks a model against realized experience. Doing so needs a factor
series per pool or per CUSIP, which agency issuers publish monthly: Fannie Mae
through PoolTalk and Data Dynamics, Ginnie Mae through its monthly REMIC factor
files. `fnma_remic_2019_2_g3` is a natural first subject — it settled in January
2019 and has six years of history against a model already reconciled to its
projection.

The two axes answer different questions and the suite should be able to say which
it is doing. A case of this kind also needs a tolerance story that is not
rounding: divergence from realized experience is *information*, not error, and a
case that fails when prepayments differ from 198% PSA would be asserting the
wrong thing.

Provenance: raised in discussion while scoping Fannie Mae 2019-2, August 2026.

### 2.3 A published grid at several prepayment speeds

Decrement tables publish five to seven PSA speeds per class. A case runs at one.

The existing precedent is one case directory per speed — `auto_abs_speed_050`
and `auto_abs_speed_150` — which is fine for two and does not scale: Ginnie Mae
2026-100 publishes 58 tables at five to seven speeds each, and covering that grid
the same way would mean roughly 75 near-duplicate directories differing in one
term.

Scenarios already run a full deterministic pass under different parameters, but
`expected_scenarios.json` asserts **metrics only**, so the per-period column that
is the whole published artefact cannot be checked per scenario. Recorded as a
capability gap at `13` §7.23.

Provenance: Ginnie Mae 2026-100 (58 tables) and Fannie Mae 2019-2 (7 speeds on
one table), August 2026. Confirmed again by AmeriCredit 2017-1, which publishes
four speeds across six classes and ships as a single case at 1.50% ABS —
`benchmarks/credit/americredit_2017_1`, whose reference reproduces all four
speeds while the model asserts one.

### 2.4 Class types nothing exercises

Both REMIC deals modeled use PAC, TAC, SEQ, AD, Z, PT and NTL. Fannie Mae's
Exhibit A defines a good deal more, and the harder ones are untouched:

| | |
|---|---|
| `NAS` / `AS` | non-accelerated security — limited or no principal before a designated date, then an increasing specified share, with an accelerated class mirroring it |
| `JMP`, `SJ`, `NSJ` | priority that changes on a trigger event: permanently (sticky), temporarily (non-sticky), or on multiple/compound triggers |
| `CPT` | component classes — one class of two or more components with different payment characteristics |
| `SEG` | segment groups, allocating inside an overriding structure |
| `AFC` | available funds, where interest may be short and the shortfall carries over and itself accrues |
| `SP` / `SPS` | specified payment and its support |

`JMP`/`SJ`/`NSJ` are the interesting ones for the language: a trigger that
reorders a waterfall is the shape `docs/17` §5 left open, and a deal exercising
it would settle whether declaration order plus `when` is enough or whether
priority needs to be first-class.

Provenance: read from the Fannie Mae Single-Family REMIC Prospectus, Exhibit A,
while assessing candidate deals, August 2026.

---

## 3. Assertion shapes that were weaker than they looked

### 3.1 A published weighted average life cannot be asserted

Both REMIC deals publish a weighted average life per class per speed — 709 of
them for Ginnie Mae 2026-100, seven for Fannie Mae 2019-2 — and none of them can
be asserted. They went into `CASE.md` as prose.

This is the pattern `13` §7.12 was written about, one level up: a published
figure with no series or metric to check it against, so it is reconciled in
words. Recorded as a capability gap at `13` §7.22.

Provenance: both REMIC cases, August 2026.

### 3.2 A clamped waterfall step can hide an over-payment

`fnma_remic_2019_2_g3` asserts a coupon-strip identity: class AB's 3.25% plus
class IO's 1.75% should exhaust the 5.00% the pool passes through, leaving the
residual step nothing. The first draft asserted only that the residual is zero,
which reads as a complete statement of the identity and is not one.

A step pays `min(max(0, expr), remaining)`. If a coupon is set too **high**, the
over-payment clamps against `remaining`, the residual is still zero, and the
error is invisible. Mutation testing found it: raising AB's coupon from 3.25% to
3.50% survived the case unchanged. Asserting the interest legs themselves — a
published balance times a stated coupon, so still external — closed it, and all
eight mutations tried are now caught.

The general shape: **a residual assertion is a one-sided test.** It catches
under-payment and is blind to over-payment. Any case whose evidence is "the
remainder is nothing" has this hole, and the suite has no way to say "this step
was not clamped" — `owed` and `paid` differ exactly then, but only a later step
in the same waterfall can read them, and neither is published as a series.

Provenance: found by mutation testing the Fannie Mae Group 3 case before it
shipped, August 2026.

### 3.3 Mutation testing is not part of the workflow

Both findings above came from deliberately breaking the model and checking the
case failed. It caught a real hole in §3.2 and confirmed discrimination
elsewhere, and it is not something the repository does, asks for, or records.

A case is evidence in proportion to what it would reject. `make bench` proves a
case passes; nothing proves it could fail. The cheapest version is a documented
habit — perturb each stated input, confirm the case breaks, record the list in
`CASE.md` — and the expensive version is a harness target that does it from a
declared list of mutations.

Provenance: applied by hand to `gnma_remic_group16` (5 mutations) and
`fnma_remic_2019_2_g3` (8 mutations, one of which found §3.2), August 2026.
`americredit_2017_1` shipped without it and has the shape §3.2 warns about: the
certificateholder's step-down release absorbs whatever the notes are not paid,
so a residual assertion there would be one-sided by construction.

---

## 4. Method notes worth keeping

Not backlog items. Findings that cost real effort to establish and would cost it
again.

**Weighted average life conventions differ by issuer, and the difference is
visible — three issuers now, three conventions, none of them stated.** Ginnie Mae 2026-100's published lives reproduce exactly under
**30E/360 from the closing date** to the 20th of each month — 709 of 709. A
plain `(t+1)/12` gets 76% of them and is wrong by up to 0.078 years. Fannie Mae
2019-2 uses 30/360 from its 30 January settlement to the 25th. AmeriCredit
2017-1 uses 30E/360 from its 23 February closing to the 18th, which makes the
first period a 25-day stub — measuring from period zero instead overstates every
life by 0.014 years and misses 20 of its 48 published figures. No document
states the day count for this purpose; all three were recovered by trying
conventions against the published figures.

**A whole-percent decrement table has a 0.5pp floor, and the error distribution
is the real test.** If a model is exactly right, its errors against a
whole-percent grid are uniform on [0, 0.5], so the mean should sit near 0.25 and
the maximum over *n* informative cells near `0.5n/(n+1)`. Ginnie Mae Group 16
came in at 0.2664 mean and 0.4941 max over 74 informative cells against 0.25 and
0.4933 predicted. A model that is subtly wrong shows a *biased* distribution —
clustered high, drifting with time, or concentrated in one class — even when
every individual cell passes. Checking the shape catches what checking the bound
does not.

**Count informative cells, not cells.** In these tables a large share of the
published values are exactly 0 or 100, which assert only "retired by then" and
"not started yet". Ginnie Mae Group 16 publishes 180 cells of which 74 carry
information; Fannie Mae Group 3 publishes 30 of which 14 do. Quoting the gross
count overstates the evidence.

## 5. Domains the suite does not cover

Sections 1–4 are structured credit. The suite is not: 40 registered cases
across five directories, and `docs/13` §7.3 records the imbalance the credit
sections cannot see — cre at 1/12 and opco at 0/10 contract types externally
validated, against credit's 10/10 and energy's 9/10. The items here are
domain cases the language can express (or will, at a named phase) that the
suite has not checked. Provenance: the domain survey in `docs/30`.

**5.1 The gate fixtures should graduate.** The walk and phase 5 name their
proof fixtures (`docs/29` phases 4–5): the delinquency machine breaching and
curing twice on realised rent; trapped cash accumulating across a failed
trigger and releasing on cure; a reserve funded to target and released;
Highlands restated through an account, tied to the same numbers. Each is a
fixture pinning a mechanism. Each is also a deal shape with published
references — a servicer's delinquency roll, a credit agreement's cash-trap
covenant (`docs/13` §7.77), a DSRA funding schedule — and a fixture asserted
against its own engine is the suite marking its own homework, the same
argument as `docs/13` §7.5's contract twin. The ask: one benchmark case per
mechanism, each against an external reference, promoted as the phases land.

**5.2 The contract-twin debt.** `one_lincoln_street_contract` proves the
form: the pack contract asserted against the primitive-built original, zero
difference in all 48 cells. No other primitive-built case has its twin. The
cheapest coverage moves in the suite are twins for the cases that already
exist, and the opco candidates of `docs/13` §7.5 (depreciation, equity
bridge, share count, forward multiple) each arrive with a case or they are
not validated.

**5.3 The flip case rebuilt on the walk.** `energy/tax_equity_flip` restates
the project's cash build inside its `return_position` recurrence and carries
a hand-carried pot (`docs/25`); its own header defers the rehoming because
the asserted figures would move. Rebuilding it — backward reads for the
return test, an account for the pot — re-asserts the same published figures
through the machinery the case motivated, and is the walk's first end-to-end
benchmark rather than fixture.

**5.4 A storage dispatch case.** Blocked on a reference that runs, not on
the language any longer (`docs/13` §7.75, §7.1's SAM attempt). Whichever
reference first produces a defensible dispatch schedule — even behind the
meter — gets the case, and with it the energy pack's last uncovered rule.

**5.5 A promotion, not an invention: the availability-payment concession.**
`bespoke/ppiaf_toll_highway` already models three tranches, capitalizing
interest, staggered grace and a subsidy sized to hold 1.30x cover — as a
bespoke case with no pack. The toll-road entry of the roadmap's Tier 1 is
this case with a pack behind it and a deduction-regime availability
adjustment; the benchmark exists before the pack does, which is the right
order and worth recording as such.
