# 30 — What the walk enables, domain by domain

Status: survey. Nothing here is a design; each item points at the design that
owns it (`docs/28` for the mechanism, `docs/13` for the gap, `docs/20` for the
missing case). This document exists because the period walk, the account, and
the declared machine were argued from structured credit — and the survey that
produced it found the same four mechanisms at the top of every other pack's
gap list. The walk is a cross-domain dividend, and this is the map of where it
pays.

Provenance: a read of all four packs, all 40 registered benchmark cases
(`site/content/docs/benchmarks.md`), `docs/13`, `docs/17`, `docs/20`,
`docs/25`–`docs/29`, and the pack roadmap in `research/`. Items already
recorded elsewhere are cited, not restated.

## 1. Four mechanisms, every domain

The walk's new capability decomposes into four mechanisms, and each domain's
gap list wants all four:

1. **Logic reads settled cash** (`docs/28` §4, shipped). Kills the
   restate-the-build idiom: `tax_equity_flip` recomputes the project's entire
   cash build inside `return_position`'s recurrence; `americredit_2017_1`
   duplicates seven step-down expressions; the LBO cases rebuild the free cash
   flow stack inside the balance recurrence (`docs/13` §5.2). All can now read
   the published streams at `t − 1` instead.
2. **The account** (`docs/28` §5.1, shipped). Every domain has a reserve
   wearing a different name: the DSRA and major-maintenance reserve in energy
   (both PV references zeroed reserves out as inexpressible —
   `benchmarks/energy/utility_pv_singleowner/NOTES.md`,
   `crest_solar_cost_based/NOTES.md`, which also records ~$4,606 of year-one
   interest earned on funded reserves CFDL could not model), the replacement
   reserve in CRE (`docs/13` §7.5), the FF&E reserve in hospitality
   (`research/` Tier 1), the minimum-cash and facility balances of an opco
   revolver, the recoverable-advances balance of a mortgage servicer.
3. **The declared machine** (`docs/28` §6, phase 5, unbuilt). The
   breach-and-cure regime is the same construct everywhere: OC/IC tests in
   credit (`docs/13` §7.74), delinquency and cure on realised rent in CRE
   (`docs/28` §6.1's own worked example), curtail-and-restart in energy — the
   `energy.facility` lifecycle already declares `operating ⇄ curtailed` as
   edges, inert today (`packs/energy/ontology/types.toml`) — covenant breach
   and PIK toggles in opco (`docs/13` §7.36), a DSCR cash trap in project
   finance.
4. **Schedules anchored to state entry** (`docs/28` §6.2, phase 5, unbuilt).
   Makes milestones causal: construction from whenever construction starts
   (CRE), COD-anchored escalation vintages and the merchant tail after PPA
   expiry (energy; `docs/26` names the tail as the latching-event case),
   energization-gated revenue commencement (data centers, `research/` Tier 1).

## 2. Energy

What the pack proves today: ten lowering rules, nine externally validated
(`docs/13` §7.3), six benchmark cases including a derived flip date
(`tax_equity_flip`) and a three-way reconciliation with a negative escalator
(`crest_solar_cost_based`).

Walk-enabled, in value order:

- **State of charge as walked state.** `energy.storage_arbitrage` is the
  pack's one unvalidated rule, and `docs/13` §7.1's third way forward — "needs
  per-period persistent state" — is now buildable: a SOC balance stepped per
  period turns `quantity` (the storage rule's MWh cycled) from an assumed input into an output, which
  is the circularity §7.1 says blocks validation. Recorded as `docs/13` §7.75.
- **Reserve accounts.** The references model them; the pack could not. DSRA
  and major-maintenance funding to target, with `dscr_periodic`
  (`packs/energy/statements.toml`) already publishing the covenant test that
  would gate a release. Recorded as `docs/13` §7.76.
- **A DSCR cash trap.** The statement publishes the per-period ratio; nothing
  acts on it. Breach → trap distributions in an account → release on cure is
  the energy twin of the credit trigger, and needs phase 5 plus the account.
  Recorded as `docs/13` §7.77.
- **The curtailment regime, checked.** Today a bare field flipped by a price
  test (`docs/26`'s workaround); under phase 5 the declared
  `operating ⇄ curtailed` edges become enforced, journaled, and re-entry-safe
  through the existing `active in state` machinery.
- **The flip case, restated.** Its pot is a hand-carried entity field
  (`docs/25`, the one case where revenue is computed a second time inside the
  distribution) and its `return_position` restates the build; the account and
  backward reads retire both. The model's own header defers this until the
  case is rebuilt — that rebuild is now unblocked.

Pack-shape work the walk does not touch: additional opex categories,
revenue-linked expense, additive real-plus-inflation escalation, derived ITC
basis, bonus depreciation and multi-class basis (`crest_solar_cost_based`
gaps 1–3; `utility_pv_singleowner` findings).

## 3. CRE

What the pack proves today: the lease-by-lease engine end to end — expense
stops and gross-ups (`retail_strip`, `office_two_tenant`), probability-blended
rollover with TI/LC (`office_two_tenant`), percentage rent including the
partial-expectation form, construction draw with capitalized interest and a
contract twin proving the pack against the primitive original
(`one_lincoln_street` / `_contract`), restricted rents reverting to market
(`hud_home_multifamily`), and a JV distributing once at the end of a
thirteen-year hold (`penzance_highlands`).

Walk-enabled:

- **Delinquency and cure on realised rent** — the phase 5 gate fixture is the
  CRE case (`docs/29` phase 5): `leased → delinquent → leased` on a settled
  series, with the rent stream turning off and back on through
  `active in state`.
- **Construction anchored to state entry** — `cre.property` already declares
  `predevelopment → construction → lease_up → operating`; `state_enter` makes
  the delayed-construction option fully expressible and re-anchors on
  re-entry (`docs/28` §6.2).
- **The JV through accounts** — Highlands restated through an account is a
  named phase 4 gate; party-owned accounts are the input `docs/13` §7.72's
  participant-level IRR waits on.
- **The replacement reserve** (`docs/13` §7.5) becomes the account's
  fund-to-target step form rather than a bespoke contract.
- **Phase 6 owns the two forward reads** — `cre.exit_forward` under the
  priced exception, the expense stop's plane decided by `mit_rentleg_plaza`
  (`docs/29`, decisions still open).

## 4. OpCo

What the pack proves today: the valuation conventions fleet — banker DCF,
Damodaran FCFF, Gordon growth against a published nine-point grid, three LBO
cases including the affine circular-interest schedule and the option-pool exit
waterfall, and the SBC convention fork.

The walk reverses a recorded design gate: `docs/13` §7.5 parked
`opco.revolver`, `opco.cash_sweep` and `opco.nol_carryforward` with "all
three need per-period state (5.2) and should be designed with it rather than
before it." That condition is now met:

- **The NOL carryforward** is a plain accumulator over realised taxable
  income — the canonical §7.10 shape — and is why `opco.cash_taxes` floors at
  zero today.
- **The revolver and the sweep** stop being model-specific: an account gives
  the minimum-cash balance and the facility balance a location with a balance
  law, and a later period reads what a step was allocated strictly backward
  (`docs/28` §5) instead of restating the free-cash-flow build.
- **PIK toggles and covenant breach-and-cure** are §7.36's repeatable regime,
  waiting on phase 5 for the checked form.

The cheapest coverage wins need no walk feature at all: `opco.depreciation`,
`opco.equity_bridge`, `opco.share_count`, `opco.exit_forward_multiple`
(`docs/13` §7.5) are pure contract-shape work, and are what moves opco off
0/10 externally validated contract types (`docs/13` §7.3).

## 5. Credit

Owned entirely by `docs/13` §7.74; nothing is restated here. The one addition
from this survey: the phase 5 and walk gate fixtures — the delinquency
machine, trapped cash across a failed trigger releasing on cure, the
once-at-end waterfall read by a later balance — are benchmark seeds, not just
fixtures, and belong in `docs/20` §5 once each has a published reference.

## 6. Candidate domains, re-ranked

The pack roadmap (`research/CFDL_pack_roadmap_and_model_sourcing.md`) gates
roughly two thirds of its list on per-period persistent state. That gate is
lifted, which reorders Tier 1:

**Unblocked by the walk and accounts (phase 5 completes them):**

- *Hospitality* — one accumulating FF&E reserve, the roadmap's own "cleanest
  first proof" of per-period state; the incentive fee needs nothing new.
- *Mortgage servicing rights* — an advance balance that accrues on
  delinquency and recovers on cure is exactly the delinquency machine plus an
  account.
- *Securitization waterfalls* — the §7.74 umbrella; the remaining engine asks
  are listed there.
- *Data centers* — `state_enter` gives milestone-gated revenue commencement;
  the new primitive is a capacity dimension, which is pack-shape work.
- *Toll roads / availability P3* — fixed-term first; `ppiaf_toll_highway`
  already proves three tranches, capitalizing interest, staggered grace and a
  DSCR-1.30x-sized subsidy as a bespoke case. The pack is a promotion, not an
  invention. LPVR's endogenous horizon stays deferred.

**Needed nothing from the engine:**

- *Telecom towers* — the roadmap's own "needs nothing new" entry; the cre
  lease-by-lease engine with a ground-lease contra-revenue line.
- *Industrial, student housing, manufactured housing* — "ship them as `cre`
  pack templates, not packs."

**Still gated, and by what:**

- *Project finance / district energy* — DSCR-sculpted debt and circularly
  capitalized IDC are solves (`docs/14` §5); also blocked on a source
  (`docs/13` §7.13).
- *ILS / cat bonds* — event-set Monte Carlo; the quantile primitive already
  speaks exceedance and layers (`docs/27`), the sampler does not.
- *RMBS OAS* — per-period stochastic draws (`docs/13` §7.74) and a
  valuation-plane solver.
- *Life settlements / pension risk transfer* — an actuarial data subsystem;
  "build them together or not at all."
- *Mining, timberland, LPVR* — the endogenous horizon.
  `bespoke/buenavista_del_cobre` already derives a mine plan from a reserve
  statement; the horizon itself is still an input.

## 7. What this asks of the benchmark library

`docs/20` was, before its §5, entirely structured credit — while `docs/13`
§7.3 records cre at 1/12 and opco at 0/10 contract types externally
validated. The domain coverage ask now lives in `docs/20` §5. The discipline
that travels with it, from the credit sections: assert the legs rather than
the residual (§3.2), mutation-list every case (§3.3), count informative cells
(§4), and build the contract twin beside every primitive-built case
(`docs/13` §7.5 — `one_lincoln_street_contract` is the proof of the form, 48
cells at zero difference).
