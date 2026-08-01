# CFDL — Domain Pack Roadmap & Reference Model Sourcing

**Prepared 1 August 2026.** Companion workbook: `CFDL_pack_roadmap_and_model_catalogue.xlsx`
(106 sourced models, 40 roadmap candidates, 221 exploded capability requirements).

---

## What this covers

Two questions, researched separately and then reconciled against each other:

1. **Which asset classes belong on the domain-pack roadmap** beyond `energy`, `cre`, `credit` and `opco`?
   40 candidates, each scored on how it is actually valued, what makes its cash flows structurally
   distinct, what a modelling language must express to model it faithfully, how much reuses existing
   pack primitives, market size, and who would buy it.

2. **Where to source real cash flow models** — for the four existing packs and for the candidates — so
   that example deals can be built from published work rather than invented, the language's
   expressiveness can be pressure-tested against real deal mechanics, and CFDL's numbers can be
   checked against an independent reference.

The requirement statements throughout are written as *requirements* — "requires per-period state to
carry a maintenance-reserve balance" — not as claims about what CFDL lacks. Some are already
satisfied; some are on the engine roadmap; the point of collecting them is to see which engine
capabilities unlock the most packs.

Baseline taken from the repo as it stands: `energy` (PPA, merchant, storage arbitrage, capacity, O&M,
ITC/PTC/MACRS, level-pay project debt), `cre` (lease-by-lease rent, recoveries with expense stops and
gross-up, probability-weighted rollover with downtime, percentage rent, forward-NOI exit),
`credit` (level-pay and IO/bullet pools with CPR/CDR/severity/recovery lag, floating coupon off a
declared curve, servicing strip), `opco` (revenue/opex lines, DSO/DPO/DIO working-capital policy,
capex, term debt, cash taxes, trailing-EBITDA exit). Benchmarks today validate against
`reference_gen.py` scripts written alongside each model — independent implementations, but
in-house ones. Everything in Part 2 is about replacing or supplementing that with external references.

---

# Part 1 — The pack roadmap

## The organising argument: sequence by capability unlock, not by market size

Tallying the 221 requirement statements across all 40 candidates by primitive family gives a clear
shape (full detail on the **Capability Requirements** sheet):

| Primitive family | Requirement statements |
|---|---:|
| Per-period persistent state (balances, accumulators, carryforwards) | 36 |
| Cohort / vintage stock (a vector of units, each with age and rate) | 32 |
| Ordered waterfall / priority of payments | 21 |
| Stochastic drivers & event Monte Carlo | 21 |
| Curves, indices & price paths | 21 |
| Trigger, covenant & regime switch | 20 |
| Calendar, horizon & event timing | 17 |
| Piecewise, threshold & greater-of expressions | 15 |
| Solve-to-target / endogenous circularity | 9 |
| Composition, nesting & multi-entity | 6 |
| Actuarial & tax subsystems | 5 |

Three of these dominate, and they are not independent:

- **Per-period persistent state** is the gate on roughly two thirds of the list. It appears in its
  simplest possible form in hotels (one FF&E reserve scalar), and in its most demanding form in tax
  equity (per-partner capital accounts, outside basis, suspended losses).
- **Cohort / vintage stock** — a vector of units each carrying an age, a rate and a hazard of leaving
  — is the same shape in self-storage tenants, seniors-housing residents, fibre subscribers, container
  fleets, music-catalogue songs and orchard blocks. Building it once unlocks six or seven packs.
- **The ordered waterfall with trigger-gated branching** is shared by CLOs, CMBS, RMBS, aircraft ABS,
  whole-business securitisation, NPL acquisition leverage, receivables facilities and every fund-level
  carry structure — plus, in a per-asset form, film ultimates and litigation finance.

That is the argument for the Tier 1 list below: it is chosen so that each entry either needs nothing
new, or pays for a primitive that several later packs then get cheaply.

## Tier 1 — near term (10)

Ordered by leverage, not by market size.

**1. Securitization tranche waterfalls (CLO / ABS / CDO).** Highest downstream unlock. The `credit`
pack's CPR/CDR/severity/lag pool engine already *is* the collateral side; what is added is the
liability side — an ordered payment waterfall whose steps consume from a running available-funds
accumulator, with OC/IC coverage tests evaluated from current-period state that divert interest to
senior principal until they cure. Reuse ~55%. CLO market ~$1.4tn within a ~$13.3tn structured credit
universe (Guggenheim). Every subsequent structured pack reuses this machinery.

**2. Private fund cash flows: capital calls, distributions & carry waterfalls.** Widest buyer base on
the list, and the underlying assets are already expressible — `opco` is exactly the portfolio-company
layer. Needs the same solve-to-target tier as (1), plus a preferred return that compounds on
unreturned capital and a cumulative IRR that is a live state variable rather than a post-hoc report.
Enables GP stakes and continuation vehicles as thin follow-ons. 2025 secondary volume $240bn at 87% of
NAV (Jefferies) is the visible clearing price for exactly these cash flows.

**3. Tax equity partnership flips & transferable tax credits.** The natural upsell into the existing
`energy` installed base — the pack already produces the project cash flow, ITC/PTC/MACRS and project
debt, roughly half the model. What is missing is the layer that actually gets US renewables financed:
dual tax and cash ledgers with different allocation ratios, per-partner capital accounts with a
deficit restoration obligation cap, outside-basis-limited loss suspension, and a flip date that is
endogenous (the period in which the investor's cumulative after-tax IRR first reaches target). Shares
the sizing solver and cumulative-IRR primitive with (2). Transferable credit transactions ran
$20–25bn in 2024 (Crux).

**4. Telecom towers & cell-site ground leases.** The single highest reuse-to-value ratio in real
assets (~60%) and the only Tier 1 entry that needs *nothing new from the engine*. A tower is a rent
roll: stacked independent contracts each with its own commencement, term, fixed or CPI escalator and
renewal option — which is what the `cre` lease-by-lease engine already does. The new parts are all
closed-form over dated events: a lease-up and amendment arrival rate, the ground lease as a
contra-revenue stream with its own escalator and expiry (and a buyout modelled as a capital event),
and dated MNO-consolidation churn rather than a percentage. Shippable against today's engine.

**5. Mortgage servicing rights.** The best narrow-fast candidate: a compact model (survival-weighted
fee strip on a declining balance, per-loan rather than per-dollar cost to service, escrow float on a
short-rate curve, an advance balance that accrues on delinquency and is recovered on cure) against a
small, wealthy, spreadsheet-bound buyer base. Needs state and rate-driven prepayment but **no
waterfall**, so it ships in parallel with (1) rather than behind it — and it pays for the rate-path
machinery RMBS later needs. ~$245bn UPB traded in 2025 at 5.0–5.5x multiples.

**6. Data centers.** Highest-demand real asset class right now (North American supply 8,155 MW in
H1 2025, 74.3% of construction preleased, 1.6% vacancy — CBRE) and ~50% reuse across two packs at
once: `cre` for the contract stack, escalators and TI amortisation, `energy` for power curves,
energisation milestones and MACRS/ITC on on-site generation. The main new primitive is a capacity
dimension — contracts denominated in kW with a contracted-vs-utilised split and a PUE multiplier on
the power pass-through — which is an extension of the existing recovery/expense-stop concept. Also
needs milestone-gated revenue commencement (a phase starts at `max(construction_complete,
energization_date)`) and a capitalised construction-interest balance.

**7. District energy & waste-to-energy.** The cheapest pack to build, ~65% reuse, and best treated as
an **`energy` pack extension rather than a greenfield pack**. Merchant power, contracted offtake,
capacity payments, fuel curves, O&M and project debt already exist; a district plant is a
capacity-plus-consumption contract stack sold to buildings instead of to a utility. The two-part
thermal tariff is a capacity payment with a different unit; put-or-pay waste tonnage is a deficiency
test on a volume. Both closed-form. Low-cost way to widen the energy buyer base into infrastructure
funds. 700+ US systems (IDEA/EESI).

**8. Insurance-linked securities (cat bonds, sidecars, collateralised re).** The only Tier 1 pick
orthogonal to everything else, and therefore the least commoditisable. Needs event-set Monte Carlo —
draw a frequency, draw a severity per event, apply a piecewise layer payoff (zero below attachment,
pro-rata to exhaustion, 100% above) per contract — rather than a continuous shock on one variable.
Seeded Monte Carlo already exists, so the extension is to event sets and layer payoffs. Cat bonds
$61.3bn outstanding on record $25.6bn 2025 issuance; ~$124bn total alternative reinsurance capital
(Artemis).

**9. Hospitality / hotels.** Earns its place on buyer density rather than reuse (~45%) — an unusually
large population of analysts model hotels weekly, and 2025 US transaction volume was $24bn (JLL). The
new state requirement is minimal and well bounded: **one accumulating FF&E reserve** that accrues as a
percentage of revenue and is drawn by dated renovation capex, floored at zero. If per-period state is
on the roadmap, hotels are the cleanest first proof of it, because the state is a single scalar rather
than a vector of cohorts. The incentive management fee is just `pct * max(0, GOP − owner_priority)`
and needs nothing new. The rest is a USALI departmental build off occupancy × ADR with seasonality.

**10. Toll roads & availability-payment P3 concessions.** The strategic entry into infrastructure
proper, and the reuse from `energy` is better than it looks: an availability payment is a contracted
revenue stream with an escalator and a performance deduction — structurally a PPA with a curtailment
adjustment — and P3 senior debt is the same sculpted amortisation. USDOT publishes standardised model
contract guides for both availability-payment and toll concessions, so the deduction regime
(hourly × segment × time factors, with peak-hour weights up to 0.35) is documented rather than
proprietary. **Sequencing note:** ship fixed-term availability-payment concessions first. The
handback reserve is ordinary per-period state, but LPVR-style least-present-value-of-revenue deals
need an *endogenous horizon* — the model ends when its own cumulative discounted revenue reaches a
target — which is the most demanding requirement in this tier. Defer it.

## Tier 2 — gated on one shared primitive each (10)

**Gated on cohort/vintage state:** self-storage, seniors housing (RIDEA), single-family rental /
build-to-rent, fibre networks, transport equipment lease pools (containers, railcars, chassis),
aircraft leasing. All six are the same shape — a stock of units, residents, subscribers or containers,
each with a vintage, a rate and a hazard of leaving — which is why building the cohort primitive well
is a stronger argument than any single pack makes on its own. Two notes: the `credit` pack's pool-decay
machinery is a surprisingly close analogue for length-of-stay decay (self-storage) and should be
looked at before building new; and aircraft leasing additionally needs the sequential-pay waterfall
and DSCR sweep from Tier 1 item (1), so it is shared work rather than incremental work.

**Following the waterfall engine:** RMBS (adds rate-path Monte Carlo, an endogenous prepayment S-curve
with burnout, PAC schedules and an OAS root-find) and CMBS (adds per-loan status state, servicer
advances, appraisal reductions and a balloon refinance test). Each is mostly the waterfall engine plus
one new capability.

**The longevity pair:** life settlements and pension risk transfer. Their maths — survival-probability-
weighted cash flows — actually suits a closed-form, loop-free engine well. What they need is an
actuarial table subsystem (mortality tables by age/sex/smoker, mortality multipliers, generational
improvement scales), which is a *data and domain* investment rather than an engine investment, and
shares almost nothing with the rest of the list. Build them together or not at all.

## Tier 3 — highest new-primitive cost (19)

Two distinct groups.

**Genuinely new physical/biological state (8):** timberland, farmland & permanent crops, mining,
carbon credits, shipping, airports & ports, EV charging, parking. Each needs a stock that migrates
between value tiers over time — diameter classes, tree age, ore grade and reserve depletion,
sequestered tonnes — or a regulatory asset base carried forward with a periodic true-up. Mining and
timberland additionally need an endogenous horizon (the mine ends when the orebody is exhausted).
These are the most *differentiated* packs — nobody models them well — but the most expensive.
**Treat timberland, farmland and nature-based carbon as one programme**, not three: the biological
inventory state machine, once built, substantially de-risks the other two. Sequence after the cohort
primitive proves out in self-storage.

**Recombinations of Tier 1 primitives (11):** whole-business securitisation, music catalogues, film
libraries, pharma royalties, litigation finance, NPL portfolios, trade receivables, venture debt,
equipment lease portfolios, GP stakes, continuation vehicles. Each is real, and several are
commercially attractive — music and pharma royalties in particular need only cohort/decay/lag
machinery plus probability trees. But each is largely a recombination of primitives that the Tier 1
packs create, which is the argument for sequencing by capability unlock rather than by market size.

## Deliberately excluded — and why

**Industrial / logistics, student housing, manufactured housing** are not packs. Industrial is a CRE
model with a longer WALT, flatter escalators and lower TI. Student housing needs by-the-bed leases on
a hard August-to-August calendar with a pre-leasing velocity curve; manufactured housing needs lot
rent plus a home-sale line. Both are configurations of the existing rent-roll engine.
**Ship them as `cre` pack templates, not packs** — shipping them separately would dilute what a domain
pack means.

**Water rights** is split. The concession half (desalination, bulk supply) is near-term `energy`
reuse — a take-or-pay two-part tariff with an energy-indexed variable leg is almost one-for-one with
the existing PPA and capacity-payment machinery, and should ship alongside district energy. The rights
half needs a priority-ordered allocation waterfall applied to a *physical quantity* rather than to
cash (seniors served in full before juniors receive anything), which is a genuinely new primitive.

---

# Part 2 — Reference models

## What was collected

106 sources, all publicly accessible, all URLs fetched and confirmed:

| | Count |
|---|---:|
| **By access** | |
| Direct download (no gate) | 65 |
| Free registration (email) | 20 |
| Free to view, no download | 21 |
| **By validation fidelity** | |
| Full cash flow table available — checkable period by period | 62 |
| Assumptions and outputs only — endpoint checkable | 27 |
| Methodology only — specification, no numbers | 17 |
| **By domain group** | |
| CRE | 20 |
| Energy & infrastructure | 19 |
| Credit & structured finance | 22 |
| OpCo & corporate finance | 21 |
| Transport, resources & royalty/IP | 24 |

The sourcing bias was deliberate: a published cash flow schedule you can check period by period is
worth far more than a methodology description, so **62 of 106 entries carry full numeric output**.

## The highest-value validation targets, by existing pack

These are the ones worth building first, because each yields an unambiguous published number that
CFDL either reproduces or does not.

**CRE.** MIT OpenCourseWare 11.431J Problem Set 1 is the sharpest precision target in the entire
catalogue: a 30,000 SF office building, two suites at different expense stops ($4.00 and $5.00/SF),
$14.00/SF market rent, differentiated new-vs-renewal TI ($10 vs $3/SF), 5 months free rent, opex
growing 4% — and a **published exact answer of $2,292,810**. It is CC BY-NC-SA, so it can ship in the
repo. Case Assignment 3 (One Lincoln Street, Boston — a real 36-storey, $330.5m development with
$285.1m of quarterly construction draws, $16.3m of accrued construction interest, an 11% cumulative
compounding preferred and a three-party JV) is the most credible *named* example deal available under
a redistributable licence. Beyond that: A.CRE's commercial mortgage model for day-count conventions
(30/360, Actual/360, Actual/365) and yield maintenance — exact, unambiguous computations where any
discrepancy is a bug, not a convention difference — and A.CRE's waterfall model with catch-up and
clawback, where clawback specifically tests whether the language can express a distribution that is
provisional until a terminal test resolves.

**Energy.** NREL's System Advisor Model is the strongest end-to-end reference: BSD-3 licensed, so
any disagreement can be traced to a specific formula rather than argued about. **PySAM (`pip install
nrel-pysam`) turns it into a headless oracle** — a harness can sweep hundreds of parameter
combinations and diff every annual cash flow array against CFDL, which is far stronger than
reconciling one hand-built spreadsheet. There is also a three-way check available: NREL CREST exists
as Excel *and* as an independent Python port, so CFDL as a third implementation gives three-way
agreement, and the port materialises the MACRS half-year convention as data (`macrs_halfyear.csv`)
rather than burying it in a formula. For infrastructure, the PPIAF highway PPP numerical model is the
richest single set of verifiable numbers found anywhere: a 125 km 2×2 toll highway, 10,000 vehicles/day
at opening growing 3%, tolls at $0.13 and $0.25/vehicle/km, three debt tranches with distinct
maturities, rates and grace periods, and an **ADSCR-targeted subsidy solve** — a genuine circular
constraint across all periods.

**Credit.** SIFMA's *Standard Formulas for the Analysis of Mortgage-Backed Securities* is the
definitional source for CPR/SMM/PSA/SDA and ships **two complete 176-month cash flow schedules**;
CFDL can assert period-by-period equality against the industry's own definition. (Caution: freely
downloadable but explicitly not redistributable.) For waterfalls, the AmeriCredit 2017-1 424B5 is a
canonical fully-specified real structure — $930m of notes against a $1.011bn subprime pool, a
**22-step priority of payments**, OC building from 5.75% to a 14.75% target — in a free public filing.
And there is a rare complete numeric grid: an auto ABS Exhibit 99.4 tabulating percent-outstanding at
every monthly distribution date for six classes across seven ABS prepayment speeds, published by the
issuer. On the open-source side, **AbsBox (Apache-2.0) and Hastructure (BSD-3)** are runnable second
opinions on exactly the semantic surface CFDL targets — same deal, two engines, assert equality.

**OpCo.** SEC merger-proxy fairness opinions are the underused source here: bankers disclose the
actual projected free cash flows *and* the resulting value range, which is a real third-party accuracy
benchmark rather than a self-graded exercise. Four are catalogued with full numeric detail —
Qatalyst on Aspen Technology (9.5–11.25% WACC, 20–30x terminal NTM UFCF on FY30E UFCF of $891m,
mid-period discounting, implied $197.37–$219.55), Centerview on Squarespace (UFCF disclosed both pre-
and post-SBC — $331m vs $198m in year one, a convention fork that moves value materially), Goldman on
Genentech (a 16-year forecast driven by patent cliffs), and Deutsche Bank on Wanda Sports (explicit
contract-renewal scenario branching). For debt mechanics, Multiple Expansion's 7-step LBO build is the
densest free test available: revolver with commitment fee and minimum cash, TLB with mandatory
amortisation plus an excess-cash-flow sweep, across three leverage cases. Damodaran's library supplies
the textbook anchors (FCFF Ginzu, the LBO model, and ~24 named-company valuations covering banks,
cyclicals, distressed and emerging-market cases). And CalPERS publishes real net IRRs and multiples on
real, irregularly-timed LP cash flows across hundreds of funds — the largest free accuracy benchmark
for IRR and multiple computation anywhere in the catalogue.

## Reference models for the roadmap candidates

Mining is the standout: **NI 43-101 and S-K 1300 technical reports publish complete annual cash flow
tables alongside a stated NPV and IRR**, which makes them directly checkable. Seven are catalogued,
each exercising a different mechanic — Buenavista del Cobre (a royalty levied on *earnings before
tax* rather than revenue, which is genuinely circular with cost and depreciation; NPV10 $3.4bn),
Cripple Creek & Victor (heap-leach revenue continuing ~14 years after the last tonne is mined; NPV5
$824m), Copper World (a $230m Wheaton stream deposit), Josemaria (a six-layer Argentine fiscal stack
including a debits-and-credits tax charged on the model's own cash movements), Granite Creek (a gross
NSR package coexisting with a 10% net profits interest, testing royalty-waterfall ordering), San
Gabriel (Peru's sliding-scale IEM and mining royalty, both progressive in operating margin).

Elsewhere: UC Davis's 2024 almond cost study is probably the cleanest free permanent-crop benchmark in
existence (105-acre farm, cumulative establishment cost $17,292/acre through year 3, amortised over 22
productive years at 8.25%). The University of Georgia loblolly pine rotation study demonstrates that
the *discount rate determines the optimal rotation* — a structural result, so reproducing the SEV
ranking flip between 4% and 6% proves more than matching one number. The Appraisal Journal LEV paper
publishes two exact intermediate values (NFV $916.76, LEV $408.65/acre) from fully disclosed inputs.
`dcapy` (MIT) is a runnable decline-curve and well-economics reference. Mills Music Trust discloses
decades of realised music-royalty receipts, so a catalogue model can be *back-tested* rather than only
built. Royalty Pharma's 10-K is the richest single disclosure of royalty mechanics — tiers with
annual resets, per-asset expiries, perpetual and finite royalties in one portfolio, milestones.

## Licensing reality — what can ship in-repo versus what is validation-only

This distinction matters more than it first appears, because CFDL is source-available and benchmarks
get committed.

**Can ship as example deals (explicit permissive licence):** MIT OpenCourseWare (CC BY-NC-SA —
attribution, non-commercial, share-alike) is the only *content* in the catalogue with an unambiguous
reuse grant; MIT 15.414 and 11.431J together cover CRE DCF, development JVs, after-tax analysis,
divisional WACC and covenant mechanics. On the code side: SAM and PySAM (BSD-3), Hastructure (BSD-3),
`pyforma` (BSD-3), `dcapy` (MIT), AbsBox (Apache-2.0). The World Bank guarantee scenario-analysis
paper is CC BY 3.0 IGO. LBNL's utility-scale solar dataset is CC-BY.

**Validation targets only — do not vendor:** SIFMA's Standard Formulas ("reproduction in any form is
strictly forbidden"). Fannie Mae and Freddie Mac loan-level datasets — free with registration, but the
terms bar distributing data to third parties and, for Fannie, "using it in support of external
commercial purposes"; read them before wiring either into CI. A.CRE and Finamodel models are free
(pay-what-you-can / email signup) but state no redistribution rights — fine to model against, not to
commit. Ed Bodmer's collection is ungated but unlicensed.

**Public filings** (SEC EDGAR prospectuses, technical reports, fairness opinions, 10-Ks) are freely
accessible and citable; the underlying documents may still carry filer copyright, so cite and
reproduce numbers rather than republishing files.

## Suggested next steps

1. **Replace one in-house reference with an external one, end to end**, to prove the harness works
   before scaling it. The cleanest candidate is MIT 11.431J PS1 → a new `benchmarks/cre/mit_ocw_ps1/`
   asserting $2,292,810, since the licence permits shipping the source alongside it.
2. **Stand up the PySAM oracle** for `benchmarks/energy/`. `pip install nrel-pysam`, express the
   existing solar and wind benchmarks as Singleowner configurations, and diff annual arrays. This
   converts energy validation from "we wrote a second implementation" to "we agree with NREL",
   which is a materially stronger claim for cfdl.dev.
3. **Add the SIFMA 176-month schedules as a credit golden test.** It is the definitional source for
   conventions the `credit` pack already implements, so it is a high-confidence, low-effort win —
   and it would settle the CPR/SMM conversion and recovery-lag semantics against the industry text.
4. **Build one Tier 1 pack that needs nothing new** — telecom towers — as the proof that the domain
   pack model scales without engine work, while the state/waterfall work proceeds in parallel.
5. **Decide the state primitive before committing to Tier 2.** Hotels (one scalar) and self-storage
   (a cohort vector) are the two natural first proofs, in that order; six Tier 2 packs are downstream
   of the second one.
6. **Convert industrial, student housing and manufactured housing into `cre` templates** — cheap
   coverage breadth without diluting what a pack means.

## Caveats

- Tier assignments are a sequencing proposal, not a decision. They weigh reuse, shared gating
  capability and buyer density; a commercial reason to jump the queue would override them.
- Market-size figures are as reported by the cited source on the date fetched. They are for relative
  sizing, not a verified market study.
- All 106 URLs were re-fetched on 1 August 2026. 105 resolved and matched their description. One
  exception: the Ginnie Mae bulk-data layout page returns a generic shell rather than the layout
  content — navigate from the Ginnie Mae disclosure-data index instead. Flagged in the workbook's
  Link check column.
- One minor discrepancy: the "A Simple Model" distribution waterfall entry is described as five worked
  structures; the current page text describes four.
- Licence statements are as found on the publisher's page. "Not stated" means no terms were located —
  it is not a grant of rights. Confirm before committing any file.
- The **Capability Requirements** sheet's primitive-family column is auto-tagged from the requirement
  text by keyword and is intended for counting and sorting, not as an authoritative taxonomy. 18 of
  221 statements fell through to "Other / uncategorised".

## Principal sources

Roadmap evidence: [Guggenheim CLO primer](https://www.guggenheiminvestments.com/perspectives/portfolio-strategy/understanding-collateralized-loan-obligations-clo/) ·
[Jefferies 2025 secondary market review](https://www.jefferies.com/insights/the-big-picture/2025-global-secondary-market-review-another-record-breaking-year/) ·
[Norton Rose Fulbright — partnership flips](https://www.projectfinance.law/publications/2021/february/partnership-flips) ·
[Grant Thornton — tower valuation drivers](https://www.grantthornton.co.uk/insights/understanding-value-drivers-in-telco-tower-valuations/) ·
[CBRE — North America data center trends H1 2025](https://www.cbre.com/insights/reports/north-america-data-center-trends-h1-2025) ·
[USDOT — availability payment concession model contract guide](https://www.transportation.gov/sites/dot.gov/files/docs/ap_concession_model_p3_contract_guide_0117.pdf) ·
[Artemis — 2025 cat bond issuance](https://www.artemis.bm/news/catastrophe-bond-issuance-breaks-q4-and-full-year-records-market-grows-24-report/) ·
[JLL — 2025 US hotel investment trends](https://www.jll.com/en-us/newsroom/jll-releases-2025-us-hotel-investment-trends-report) ·
[HousingWire — MSR trades 2025](https://www.housingwire.com/articles/msr-trades-strategic-shift-2025/) ·
[KKR — asset-based finance](https://www.kkr.com/insights/asset-based-finance-fast-growing-frontier-private-credit)

Reference models: [MIT OCW 11.431J Problem Set 1](https://ocw.mit.edu/courses/11-431j-real-estate-finance-and-investment-fall-2006/3cdd8d84d7dd0fef202fcd0ba8d7a7a7_ps1.pdf) ·
[MIT OCW — One Lincoln Street case](https://ocw.mit.edu/courses/11-431j-real-estate-finance-and-investment-fall-2006/2c49789596ca71e885d2747c0e59090c_case3.pdf) ·
[NREL SAM (BSD-3)](https://github.com/NREL/SAM) ·
[PySAM Singleowner](https://nrel-pysam.readthedocs.io/en/master/modules/Singleowner.html) ·
[PPIAF highway PPP numerical model](https://www.ppiaf.org/sites/ppiaf.org/files/documents/toolkits/highwaystoolkit-russian/6/pdf-version/numerical_model.pdf) ·
[SIFMA Standard Formulas, Chapter SF](https://www.sifma.org/wp-content/uploads/2017/08/chsf.pdf) ·
[AmeriCredit 2017-1 prospectus](https://www.sec.gov/Archives/edgar/data/1694010/000119312517045288/d269131d424b5.htm) ·
[Hastructure (BSD-3)](https://github.com/absbox/Hastructure) ·
[Qatalyst / Aspen Technology discussion materials](https://www.sec.gov/Archives/edgar/data/1897982/000114036125003656/ny20042057x7_exc-4.htm) ·
[Centerview / Squarespace materials](https://www.sec.gov/Archives/edgar/data/1496963/000114036124030374/ny20030653x1_ex16cvi.htm) ·
[CalPERS PEP fund performance](https://www.calpers.ca.gov/investments/about-investment-office/investment-organization/pep-fund-performance) ·
[Southern Copper — Buenavista S-K 1300](https://www.sec.gov/Archives/edgar/data/1001838/000155837025002017/scco-20241231xex96d6.pdf) ·
[UC Davis 2024 almond cost study](https://coststudyfiles.ucdavis.edu/2024/09/30/AlmondsSJVNorth%20Final%20draft4.pdf) ·
[Appraisal Journal — Land Expectation Value](https://www.timbertax.org/getstarted/appraisal/papers/pdf/ajoct96.PDF)

Full source list with links, licences and per-entry detail: **`CFDL_pack_roadmap_and_model_catalogue.xlsx`**, *Model Catalogue* sheet.
