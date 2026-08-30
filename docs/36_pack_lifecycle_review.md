# The pack lifecycles, redrawn against their standards

Status: **recommendations, 2026-08-30**. Not published; repository-only, like
the backlog. Closes the reviewable half of `docs/13` §7.84.

The pack lifecycles were drawn before a machine could carry behavior. Nothing
gated on them, so their shape was never pressed: **every pack transition in all
four packs is guard-less**, and **no benchmark model binds a state** — no
`active in state`, no `initial_state`, no `state_enter`, no `.status` read.
They are, today, decorative.

That is why they can be corrected cheaply now, and why they should be corrected
before §7.79's arrival actions land on top of them — and before the vocabulary
reaches the site, learn.cfdl.dev, or `docs/machine/`, which is generated from
these sources and is what an agent reads.

---

## 1. The rules applied

**A state is a condition, not an event.** States read uniformly as adjectives
or participles. A noun naming an occurrence — `default`, `foreclosure` —
describes something that *happens*, which is an event or an edge, not somewhere
an entity sits.

**A counter or a magnitude is a field.** Days past due and available-capacity
fraction vary continuously and coexist with the regime. States are for regimes
a model gates behavior on.

**A terminal state absorbs.** No outbound edge, and the entity does not
continue past it.

**Each machine names its source, and its source's class.** A *standard* is
maintained by a standards body; a *convention* is market practice with no
authority behind it. The two make different claims and an ontology description
should not blur them.

---

## 2. Credit

The best-standardised domain in the estate: a prudential definition of default,
an accounting stage model, and loan-level datasets recording what happened.

### 2.1 `credit.loan` — rename and extend

*Standard: Basel BCBS d403 and the EBA guidelines on the definition of default;
IFRS 9 three-stage impairment; Fannie Mae, Freddie Mac and Ginnie Mae
loan-level performance data; SIFMA Uniform Practices Manual Ch. SF.*

| | states |
|---|---|
| today | `current`, `delinquent`, `default`, `foreclosure`, `liquidated`, `prepaid`, `matured` |
| proposed | `current`, `delinquent`, **`defaulted`**, **`in_foreclosure`**, `liquidated`, `prepaid`, `matured`, **`repurchased`** |

- `default` → `defaulted`. Basel and the EBA describe a *defaulted exposure* —
  a condition, entered at 90 days past due **or** on unlikeliness-to-pay
  independently of any day count.
- `foreclosure` → `in_foreclosure`. The MBA's own phrasing is "in the process
  of foreclosure": the loan is in a condition while a process runs.
- **Add `defaulted -> current`.** The EBA requires a **minimum probation
  period** before an exposure returns to performing — a cure with a dwell
  requirement, not an instantaneous flip. This is what §7.79 makes
  expressible: `on enter defaulted { set months_in_state = 0 }` and a guard on
  the counter.
- **Add `in_foreclosure -> current`** — reinstatement before sale, ordinary in
  US practice and currently unreachable.
- **Add `delinquent -> prepaid` and `delinquent -> matured`.** A late borrower
  can pay off or reach stated maturity; today only a `current` loan can.
- **Add `repurchased`**, terminal — Fannie Mae zero-balance code 06, a real
  termination the pack cannot record.
- **Add a `days_past_due` field; do not add bucket states.** The industry
  reports delinquency in buckets (30-59, 60-89, 90+), which is a counter
  reading rather than a regime. One `delinquent` state plus a counter carries
  it, and the 90-day threshold becomes a guard rather than a state boundary.
- `liquidated`, `prepaid`, `matured` stay terminal and are correct.
  Liquidation is how a loan *ends*; REO is what happens to the property
  afterwards and is not a loan state — which is why the MBA survey stops
  tracking at that point.
- **Not recommended:** `modified` as a state. A modification is an event that
  resets terms; the loan stays in whatever condition it was in.

### 2.2 `credit.pool` — restructure

*Standard: SIFMA UPM Ch. SF, with ABS structural convention for the revolving
and early-amortization periods.*

| | states |
|---|---|
| today | `warehouse`, `amortizing`, `called`, `retired` |
| proposed | `warehouse`, **`revolving`**, `amortizing`, **`rapid_amortization`**, `retired` |

- **Add `revolving`** — the reinvestment period is a named, universal ABS/CLO
  regime the pack cannot presently say.
- **Add `rapid_amortization`** — early amortization on a trigger breach is a
  distinct regime with different cash mechanics, not a variant of normal
  amortization.
- **Retire `called` as a state.** A clean-up call is an occurrence, not a
  condition a pool sits in. It becomes an event driving `amortizing ->
  retired` with a no-return topology, which is how `docs/34` D1 says
  once-ness is declared.

---

## 3. Energy

The only domain with a formal state-machine standard.

### 3.1 `energy.facility` — restructure to IEEE 762

*Standard: IEEE Std 762, "Definitions for Use in Reporting Electric Generating
Unit Reliability, Availability, and Productivity"; NERC GADS Data Reporting
Instructions; PJM Manual 22.*

| | states |
|---|---|
| today | `development`, `construction`, `commissioning`, `operating`, `curtailed`, `decommissioned` |
| proposed | `development`, `construction`, `commissioning`, **`in_service`**, **`reserve_shutdown`**, **`forced_outage`**, **`planned_outage`**, **`maintenance_outage`**, **`mothballed`**, **`retired`** |

- **The standard separates availability from dispatch; the pack conflates
  them.** A unit available but not synchronised (*reserve shutdown*) is a
  different condition from one forced out. Both collapse today into
  `curtailed`, and they have opposite implications for an availability
  payment.
- `operating` → `in_service`, the GADS term for a synchronised, generating
  unit.
- **Retire `curtailed`.** Economic curtailment is a reserve shutdown —
  available, not dispatched. A physical limit is a *derate*, which is the next
  point.
- **Add an `available_capacity_fraction` field; do not add a `derated`
  state.** IEEE reports derates with a magnitude, and a derate coexists with
  being in service. It is partial, so it is a field.
- **Distinguish forced / planned / maintenance outages.** That distinction is
  the basis of the availability metrics the energy pack already publishes;
  collapsing them makes EAF and EFOR uncomputable.
- `decommissioned` → `retired`, and **add `mothballed`** — GADS's inactive
  states. A mothballed plant can return; a retired one cannot.
- **Add abandonment edges from `development` and `construction`**, where
  projects most often die and neither can presently reach a terminal state.

### 3.2 `energy.flip_structure` — keep

*Convention: US tax-equity partnership-flip practice; IRC §704(b) capital
account maintenance.*

Two states and one edge looks like a boolean, but the flip date is
**endogenous** — struck on a target after-tax yield, not a calendar date — and
it gates allocation. That earns a machine: a field cannot anchor
`state_enter`, and the flip is exactly the date a schedule wants to hang off.

Consider renaming to `energy.tax_equity_flip`: `flip_structure` names the deal,
not the regime.

---

## 4. Commercial real estate

No standards body defines either machine. The unit-level vocabulary is market
convention that Argus Enterprise is the widest carrier of — worth adopting
because practitioners already speak it, but it needs filtering. Several Argus
items are *expiration-handling choices* (Market, Reabsorb, Renew, Vacate,
Option) which select a transition rather than name a condition.

### 4.1 `cre.unit` — extend and merge

*Convention: commercial leasing practice; lease-status vocabulary as carried by
Argus Enterprise, filtered to conditions.*

| | states |
|---|---|
| today | `vacant`, `leased`, `holdover`, `downtime` |
| proposed | `vacant`, `leased`, `holdover`, **`month_to_month`** |

- **Add `month_to_month`.** Practice distinguishes it from holdover and the
  pack does not: month-to-month is a contractual continuation at agreed terms;
  holdover is occupancy *without* a contract, usually at a penalty rent. They
  pay differently, so they cannot be one state.
- **Merge `downtime` into `vacant`.** They are the same condition — no tenant,
  no rent. What differed was the *path*: initial lease-up versus re-let after
  an expiry. With §7.79, edge actions carry that difference, so provenance no
  longer needs a state to encode it. This is the new construct removing a
  state rather than adding one.
- Edges: `vacant -> leased`, `leased -> holdover`, `leased -> month_to_month`,
  `leased -> vacant`, `holdover -> leased`, `holdover -> vacant`,
  `month_to_month -> leased`, `month_to_month -> vacant`.

### 4.2 `cre.property` — rename and close the cycle

*Convention: institutional CRE investment practice. No standards body, and the
ontology description should say so.*

| | states |
|---|---|
| today | `predevelopment`, `construction`, `lease_up`, `operating`, `repositioning`, `disposed` |
| proposed | `predevelopment`, `construction`, `lease_up`, **`stabilized`**, `repositioning`, `disposed` |

- `operating` → `stabilized`. The industry term, and it carries information
  `operating` does not: it contrasts with `lease_up`, where a property is also
  operating.
- **The returning cycle is what justifies this machine**, and it should be
  declared plainly: `stabilized -> repositioning -> lease_up -> stabilized`,
  walked as many times as the hold walks it. This is the part a phase cannot
  express.
- **Add `repositioning -> disposed` and `lease_up -> disposed`.** Selling out
  of a repositioning or a lease-up is ordinary; today only a stabilised
  property can be sold.
- **Keep `initial = stabilized`.** An earlier reading (§7.84 point 2) called
  the development states unreachable. That was wrong: an entity declares its
  own opening state, so a development declares `initial_state predevelopment`
  while an acquisition takes the default. The default should be the common
  case, and it is. §7.84 is amended accordingly.

---

## 5. Operating companies

No standards body defines a corporate state machine, and the honest reading is
that the enterprise does not have one. The states that matter attach to the
**financing**, and their vocabulary comes from credit agreements — the public
corpus being SEC filings, where forbearance, waiver and standstill language
appears verbatim.

### 5.1 `opco.enterprise` — unbind

*No source. The current state set was not derived from one.*

| | states |
|---|---|
| today | `operating`, `under_offer`, `acquired`, `levered`, `deleveraged`, `exited` |
| proposed | *no lifecycle* |

- **It encodes two orthogonal concerns.** `under_offer -> acquired` is a
  transaction process; `levered -> deleveraged` is a capital structure. A
  bought-out business is levered *and* operating, but an entity is in exactly
  one state — so the normal condition of every LBO cannot be said. And the
  declared path means an LBO leaves `operating` at close, so a stream gated
  `active in state "operating"` would stop paying the moment the deal funds.
- **Remove the lifecycle binding from `OpCo.Asset.Enterprise`.** The entity
  type stays exactly as it is — entities matter for typing, fields and
  contracts regardless of state. It simply carries no machine, which is the
  common case: 26 of 33 declared types already carry none.
- The transaction process is a one-time occurrence — an event with a no-return
  topology. Capital structure is a **field**, a leverage ratio, continuous and
  gating nothing categorically.
- **When a case forces one, add `opco.covenant` on the financing** rather than
  the enterprise: `compliant`, `in_breach`, `forborne`, `accelerated`, with
  `in_breach -> compliant` (cured) and `in_breach -> forborne` (waiver or
  forbearance). That is a regime that returns, gates cash, and needs
  per-arrival bookkeeping — what a machine is for. It also gives §7.77's DSCR
  cash trap somewhere to live.

---

## 6. What this adds up to

| machine | verdict | source class | states |
|---|---|---|---|
| `credit.loan` | rename + extend | standard | 7 → 8 |
| `credit.pool` | restructure | standard | 4 → 5 |
| `energy.facility` | restructure to IEEE 762 | standard | 6 → 10 |
| `energy.flip_structure` | keep | convention | 2 → 2 |
| `cre.unit` | extend + merge | convention | 4 → 4 |
| `cre.property` | rename + close cycle | convention | 6 → 6 |
| `opco.enterprise` | unbind | — | 6 → 0 |

Seven machines become six, and three fields appear where a magnitude was being
forced into a state: `days_past_due`, `available_capacity_fraction`, and a
leverage ratio on the enterprise.

**What it costs.** Less than at any later point. Every pack transition is
guard-less, so no machine fires on its own; no benchmark binds a state; and
only six fixtures reference pack states, all `cre.unit`, whose names do not
change. The renames reach about 33 files — mostly IR goldens, three published
site pages and one learn chapter — and **no asserted number in any benchmark
moves**, because the two benchmark references are prose and a generator script
rather than `expected.csv`.

**Licensing.** SIFMA Ch. SF prohibits reproduction and the GSE loan-level
datasets prohibit redistribution. Both are usable as specifications to
reconcile against and to cite; neither can be vendored — the position `docs/33`
already takes on Argus output.

## Sources

- NERC, *Generating Availability Data System — Data Reporting Instructions*,
  implementing IEEE Std 762. <https://www.nerc.com/globalassets/programs/rapa/gads/conventional/gads_dri_2024.pdf>
- PJM, *Manual 22: Generator Resource Performance Indices*.
  <https://www.pjm.com/-/media/DotCom/documents/manuals/m22.pdf>
- BCBS d403, *Prudential treatment of problem assets — definitions of
  non-performing exposures and forbearance*. <https://www.bis.org/bcbs/publ/d403.pdf>
- EBA, *Guidelines on the application of the definition of default*.
  <https://www.eba.europa.eu/publications-and-media/press-releases/eba-amends-guidelines-definition-default>
- Fannie Mae, *Single-Family Loan Performance Data*.
  <https://capitalmarkets.fanniemae.com/credit-risk-transfer/single-family-credit-risk-transfer/fannie-mae-single-family-loan-performance-data>
- Freddie Mac, *Single-Family Loan-Level Dataset FAQ*.
  <https://www.freddiemac.com/fmac-resources/research/pdf/faq.pdf>
- MBA, *National Delinquency Survey*.
  <https://www.mba.org/news-and-research/research-and-economics/single-family-research/national-delinquency-survey>
- SIFMA, *Standard Formulas for the Analysis of Mortgage-Backed Securities*
  (UPM Ch. SF). <https://www.sifma.org/wp-content/uploads/2017/08/chsf.pdf>
- Norton Rose Fulbright, *Partnership Flips: Structures and Issues*.
  <https://www.projectfinance.law/publications/2021/february/partnership-flips>
