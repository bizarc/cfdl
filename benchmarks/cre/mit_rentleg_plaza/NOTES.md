# Rentleg Plaza — what building this against CFDL surfaced

Internal validation notes. The model reproduces MIT OCW 11.431J PS1 exactly
(every pro forma line, and PV = $2,292,810.18 against a published $2,292,810).
These are the places the language or the CRE pack pushed back on the way there.

## 1. The CRE pack cannot be used at all on a non-monthly calendar

Every rule in `packs/cre/lowering/rules.toml` divides by 12 and anchors time
with `months_between(term_start, time.date)`. On an `annual` calendar
`months_between` steps by 12 per period while the `/ 12` still divides a year's
figure into a month's, so a `cre.lease_unit` on an annual grid pays 1/12 of the
rent once a year.

`time calendar annual` is documented as supported (`docs/10_implementation_status.md`)
and the language handles it correctly — this is a pack-authoring assumption, not
an engine limit. Nothing rejects the combination; it silently produces numbers
that are wrong by 12x. That is the behaviour `10_implementation_status.md`
explicitly says the project does not want ("nothing is accepted and silently
discarded").

**No model in the repo uses a non-monthly calendar**, so this has never been
exercised. This benchmark is the first.

Options: gate the pack to monthly with a diagnostic, or parameterise the
rules on periods-per-year.

## 2. `E2103_SCHEDULE_OUT_OF_BOUNDS` blocks native streams from the projection tail

`cfdl-validate/src/lib.rs:331` computes the timeline end as
`end_of_timeline(start, cadence, time.periods)` — `time.projection` is not
included. The check then walks *source AST* stream statements, so:

- a **pack-lowered** stream may schedule into the tail (`office_two_tenant`
  runs `cre.property.opex` to 2036-12 under `for 120 project 12`);
- a **hand-written** stream with the same reach is rejected.

Net effect: the projection tail is reachable only through a pack contract. The
documented `cre.exit_forward` recipe — derive forward NOI from the modelled
streams via `series_sum` over the tail — cannot be written natively.

To be clear, this is **not** evidence that the pack benchmarks are wrong.
`office_two_tenant` was re-run during this work and reproduces its blessed
values exactly (`model.npv` 1,433,678.078 vs 1,433,678.08; all four domain
metrics to the cent), and its `cre.exit.proceeds` of 3,237,142.70 implies a
forward NOI of ~214,708, which is only possible if `series_sum` genuinely read
non-zero data past period 120. The tail works. The bounds check is just applied
asymmetrically.

Consequence here: this model computes 2006 NOI inline from the same closed-form
expressions instead of reading it back from a tail.

## 3. No way to reference another period's phase-1 value

MIT fn 5 resets Suite 100's expense stop, on re-lease, to *actual 2004 opex per
SF*. Expressing that needs the value of the opex stream at t=3 from inside the
recoveries stream.

`series_sum` would do it, but it makes the caller a phase-2 stream, and phase-2
streams cannot reference each other — so the recoveries stream and the exit
stream could not both use it. The opex formula is therefore duplicated into an
`assume` (`opex_psf_2004`). It is correct but it is a second copy of a formula
that must stay in sync with the stream.

Base-year and base-year-reset stops are standard in full-service office leases,
so this is not an exotic requirement.

## 4. Pack gaps that a monthly rewrite would not fix

Independent of calendar, three things in this deal cannot be expressed through
CRE pack contracts:

- **Occupancy-varying opex.** `cre.property_opex` takes `opex_year` and
  `escalation` only. MIT splits opex 81% fixed / 19% variable-with-occupancy —
  that is what produces $135,161 rather than $144,300 in 2001. `terms` accept
  one literal or one `inputs.*` reference each, and inputs are static scalars,
  so no term can carry a time-varying occupancy factor.
- **A one-time market rent step.** `cre.rollover` has a single scalar
  `market_escalation`. The 0%/0%/0%/+20%/0% path is not expressible.
- **A stop reset to a computed later-year value.** `expense_stop_year` is a
  literal.

Each currently forces a drop to native streams, which then costs the pack's
`domain.cre.*` metrics unless the native streams are hand-named into the pack's
taxonomy (which is what this model does).

## 5. `domain.cre.noi` has no slot for abatements

The metric's denominator is `cre.ops.expense`, `cre.vacancy.loss`,
`cre.property.opex`. Free rent has no line. In the pack's own `cre.lease_unit`
rule free rent is folded into base rent, so it never surfaces separately — but
institutional pro formas (and this one) report Abatements as its own deduction
from potential gross revenue. Reporting it as a line and having it counted in
NOI are currently mutually exclusive.

## 6. Playground: the shipped wasm is stale

`site/public/wasm/` was built 2026-07-27; HEAD is 2026-08-01. The stale build
rejects **every** `schedule every <interval> from ...` form — `every month`,
`every quarter`, `every year`, on every calendar — with
`E0004_EXPECTED_TOKEN: Expected token 'from', found <identifier>`. The same
source compiles clean on a current CLI build.

So anything a user writes in the playground that uses an interval schedule
fails, which is most non-trivial models. `npm run build:wasm` regenerates it;
worth wiring into CI so the asset cannot drift from the grammar again.

## Why this stays annual

The published $2,292,810 is annual by construction — MIT fn 12, "Assumes first
cash flow occurs 1 year from present." A monthly rebuild of the same deal is a
legitimate model but a different number: spreading the same cash across months
and discounting at `(1.12)^(1/12)-1` gives ~$2,323,050, about +1.3%. Rebuilding
it monthly would trade an externally-verified figure for a self-graded one.
