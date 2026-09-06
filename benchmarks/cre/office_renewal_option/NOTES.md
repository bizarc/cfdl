# Office renewal option — maintainer's notes

The published write-up is `CASE.md`. This file carries what a maintainer
needs and a reader of the site does not.

## Why this case exists

It is the demonstration owed by stage 7 of the master contracts programme
(`docs/40` §10): an election written on a contract, with its own terms, a
schedule, and actions on exercise. It is `office_two_tenant` with the
`cre.rollover.tenant_a` contract replaced by the option and its two
outcomes, so everything the option does not touch is anchored to the same
recreation, and the difference between the two cases is exactly what the
option changes.

## How the model is put together

- The option is `on contract cre.lease_unit.tenant_a`, so its owner is the
  tower and `contract.<term>` reads its own terms first. `renewal_rent_year`
  is an `inputs.` reference so the renewal lease states the same number; a
  contract cannot read an option's terms.
- `schedule on 2031-01` tests the election once, the month after expiry — the
  original's rollover window starts there too. Without the schedule the
  election's rising edge would fire at period 0, since the market rent is a
  constant above the option rent from the start.
- Both outcomes are `cre.lease_unit` contracts, declared in full. A contract
  takes no activation guard (`docs/01` §13.4) and an exercise cannot bring a
  contract into being (`docs/13` §7.108), so the two leases are both live
  and the election switches one off: the option's actions deactivate the
  market lease's four lowered streams, and `event tenant_a.lapse`, scheduled
  on the same date with the opposite test, deactivates the renewal lease's.
  A write is read by streams as the period closed, so the deactivated
  lease's commencement TI/LC in that same month is suppressed too. When
  §7.108 lands, the lapse event and eight `deactivate` lines go away.
- `payoff 0`: the exercise's cash is the renewal lease's leasing cost, which
  the lease lowers as its dated one-shot and the pack's subtotals see. An
  option's payoff carries no category (`docs/13` §7.109), so putting the
  $100k there would have kept it out of `domain.cre.leasing_costs`.
- The market lease escalates on its own anniversaries (from 2031-04); the
  original's blended rollover escalated from the window start (2031-01).
  That is the lease convention and the reference follows it.
- The exit is `cre.exit_forward`, whose forward NOI is derived from the
  pack's stream families. That is why the outcomes had to be leases: a first
  build wrote them as model streams gated on the renewal field, and the
  exit rule could not see them, so the sale valued an empty building.
- The reference is `reference_gen.py`: the original case's generator with
  `rollover_rent` replaced by the two lease branches and the scenario loop.
  Regenerate with `python3 reference_gen.py`.

## Anchors

Every figure is from the independent recreation, not an external
publication. The debt service, 4,421,429.94, is the original case's figure
unchanged. Scenario figures are asserted in `expected_scenarios.json`.
