# The v1.0 roadmap — five milestones

Status: **adopted 2026-08-20; committed to the repository 2026-08-31.** Not
published; repository-only, like `docs/33` and `docs/36`.

**v1.0 is the standalone professional best-practice modeling tool.**
Milestones are sequenced by capability unlock, with **no fixed date**. The
backlog (`docs/13`) is the working surface: closed entries are deleted from
it, so roadmap-era IDs go stale — re-derive from current headers, and never
cite an old ID without checking. The commercial plan (`docs/31`) and the
agent substrate plan (`docs/32`) run beside this roadmap and may pull items
forward (W4 pulled §7.25); they do not reorder it.

---

## M1 — pro-forma evaluation order. **Shipped.**

The period walk: state → streams → distributions per period, results as a
valuation plane after; backward reads; the account; the declared lifecycle
machine; `state_enter`; the priced exception; the journal. `docs/28` is the
specification (restated proposed → shipped), `docs/29` the seven-phase
implementation record. The collapse property is asserted over the whole
blessed corpus in `golden_corpus.rs`.

## M2 — what the walk unlocked. **In progress; the umbrella is `docs/13` §7.78.**

The walk, the machine and the account are the substrate; every M2 item
stands on them. §7.79 (events re-founded, `docs/34`) was the milestone's
settled first priority and closed 2026-08-30, paying out §7.77's cure
period and §7.76's reserve interest the same day. What remains, current as
of 2026-08-31:

| item | remains |
|---|---|
| §7.74 | the deal mechanics: coupled interest/principal waterfalls on a trigger, a step's shortfall as a published series, deferred/PIK interest, servicer advances. The clean-up call — a case, not a construct — closed 2026-09-01 on `americredit_2017_1` (`docs/38` item 4), and cost three new entries, §7.96 to §7.98 |
| §7.76 | part two only, and now only for the packs other than credit — a reserve contract shape where a document demands one (DSRA, replacement reserve, FF&E). Credit's shipped 2026-08-31 on `americredit_2017_1` (#247), clause 19's reserve as `account reserve` funded at closing with the top-up as its step. CREST is NOT the cheapest opening, contrary to what this row said: its ~$4,606 is one rounded aggregate against three unknowns, the port's reserve schedule was never carried into the repo, and the case has no close period to fund from — see `docs/13` §7.76 |
| §7.77 | the external-reference benchmark — a published credit agreement with a cash-trap schedule; a sourcing problem, not a language gap |
| §7.75 | the dispatch model that makes `mwh_cycled_year` an output — gated on a dispatch reference that runs (SAM segfaulted front-of-meter), not on the engine; also M3's last energy item |
| §7.41 | the freeform `from <expr>` pot, the one unchecked selection left |

## M3 — validation coverage. **The living table is `docs/13` §7.3.**

Coverage claims must cite that table, and it must be re-measured whenever
cases or rosters change. As of the 2026-08-31 re-measure: energy 10/10
exercised (validated 9/10), credit 4/4, cre 14/14, opco 11/11 — counting
examples as well as benchmarks, which the 08-30 figure of cre 11/14 did not.
Only `percentage_rent_expected` is fixture-only; `cre.lease` is in two
benchmarks and two examples and `construction_stub` in four examples, so what
§7.3 wants of those two is a different GRAIN, not first exercise. Note the
asymmetry the re-measure also found: credit and energy have zero example
coverage — all 14 of their types are demonstrated only in benchmarks, never in
a teachable example. What remains: the dispatch
comparison that moves `storage_arbitrage` from exercised to validated
(§7.75's reference, worn as validation). This milestone is the license to
compete — every "displaces Argus/Intex" claim is hostage to it, and the
public line restates rather than overclaims until the table says so.

## M4 — surface polish. **Effectively complete.**

Declared metrics (§7.25) and participant-level returns (§7.72) shipped —
pulled forward by `docs/31` W4, whose phases 1–2 *were* this milestone's
work. `excel_compat` arithmetic and act/act (ISDA) shipped earlier despite
being listed deferred. Stream ownership in results (§7.43) shipped as the
graph the valuation plane publishes (`results_version` 0.7, #242); what survives of
that entry is a default-presentation request, which is a statement's job
(§7.55) and not a gate on release.

## M5 — release mechanics. **Open, and human-gated.**

1. **Cut the overdue `[Unreleased]` release.** The changelog's unreleased
   section has carried all of M1 since it shipped, and now carries M2's
   ships (#235–#243) besides. This is the mechanical half.
2. **The BUSL-1.1 marketplace decision.** "Source available, not open
   source" must be crisp in every downstream surface before distribution
   widens.
3. **Distribution: a Homebrew tap and Open VSX** for the CLI and the
   editor extension.
4. **The WCAG human pass (§7.35).** axe is clean everywhere; what remains
   is what a rule cannot check — the screen-reader session, 2.2's judgment
   criteria, content order at 200/400% zoom, and a skip link. Until it
   runs, the public statement stays "built to WCAG 2.2 AA; formal
   conformance assessment in progress", never a claim of conformance.

Items 2–4 are decisions and sessions a maintainer must drive; no code in
this repository closes them.

---

## The EVS boundary

"EVS should not occur in CFDL, but it should not be precluded": no EVS
features land here; the IR stays deterministic and versioned, and the pack
seam stays additive. Deferred past v1 deliberately: new packs, model
linking (one model consuming another's output), and backward-induction /
optimal exercise — which M1's design exists to make possible later, not
now.
