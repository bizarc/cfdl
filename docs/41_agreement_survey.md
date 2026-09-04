# 41 — The agreements a financial model contains: a survey for the roster

*Written 4 September 2026 as the reference the master roster (`docs/40`
§4) is argued against. Top-down: the families here are the industry's,
read from the documents that govern each kind of agreement and calibrated
against the published taxonomies. No family is admitted because a
benchmark models it, and none is left out because no benchmark does yet.
A benchmark DEMONSTRATES a master; it never gates one.*

## 0. What a contract is, for us

Two layers, and the survey is about the first.

A **master** is an agreement as the industry defines it — what passes
between the parties, what every instrument of the kind states, what
cash it produces by its own terms. The families in §2 are masters.

A **pack contract** is what a modeller reaches for: a bundled solution
to a cash-flow scenario. It refines a master and carries, together, the
terms a modeller states, the parties in the pack's own words, the
lowering rules that turn the terms into streams and fields, the template
that starts a model, the validations that bound the terms, the metrics
and statement rows that read what it produces, and the lifecycle its
subject moves through. A CRE unit lease is one such bundle; a credit
pool with prepayment, default, severity and recovery is another. The
master gives the bundle its identity and its obligations; the pack gives
it everything a modeller needs to solve the scenario without assembling
the pieces.

That is why the roster is small and the packs are not: a master names a
kind of agreement, and a pack ships the several forms of it a domain
actually meets, each complete. The survey asks which kinds of agreement
exist; the packs answer which scenarios a modeller needs solved.

## 1. Method and sources

Three sources, used for three different things.

**The governing documents** say what an agreement of each kind STATES.
An indenture, a credit agreement, a lease, a power purchase agreement, a
limited partnership agreement, a licence, an EPC contract, a grant
agreement: each has a standard shape, and the sections every instrument
of the kind carries are the master's core. What only some carry is a
refinement's, and what a lawyer could add is nobody's.

**ACTUS** (Algorithmic Contract Types Unified Standards, 32 contract
types) classifies instruments by CASH-FLOW LOGIC rather than by legal
label: a bullet loan and a bullet bond are one type (`PAM`), an annuity
mortgage and an annuity note another (`ANN`). It is the calibration for
mechanics — what a repayment pattern is, what a swap exchanges, what a
guarantee creates — and its families (basic fixed income, ownership,
symmetric and asymmetric combined, securitization, credit enhancement)
are the check that a mechanic we name exists and is bounded the way the
industry bounds it.

**FIBO** (the Financial Industry Business Ontology, an OMG standard of
some 2,400 classes) classifies by legal and business meaning: loans,
debt securities, equities, derivatives, funds, rights. It is the
calibration for NAMES and for the boundary between kinds — that a
security and a loan are both debt but not the same thing, that an equity
interest is ownership rather than a claim.

Neither is adopted. ACTUS has no lease, no offtake, no tax and no
construction contract, because it models the balance sheet of a bank;
FIBO has hundreds of classes a modeller would never write. The roster
takes their boundaries and their words and stops there.

## 2. The families

Arranged by what the agreement IS — what passes between the parties —
rather than by the domain that happens to use it. A CRE developer, a
securitisation trust and a wind farm each sign several of these.

### A. Money owed — financing instruments
| family | governing document | the core every instrument states | ACTUS | FIBO |
|---|---|---|---|---|
| Loan / credit facility (term, revolving, construction, bridge, mezzanine) | credit agreement, loan agreement | principal or commitment; rate (fixed, or index + margin); term; repayment pattern; day count; payment frequency | PAM, ANN, LAM, NAM, CLM, UMP | Loan |
| Debt security (bond, note, commercial paper, ABS/MBS class) | indenture, trust agreement, offering memorandum | face; coupon (fixed, or index + margin); payment dates; day count; redemption provisions; ranking within the issuance | PAM; SCRCR / SCRMR for tranches | Debt instrument, Security |
| Hybrid (convertible, warrant, perpetual) | indenture plus option terms | a security with an embedded election | BNDCP, BNDWR, PBN | Convertible |
| Finance lease / sale-leaseback | lease with purchase option | a lease whose payments amortize a price | — | Lease |

### B. Ownership — equity interests
| family | governing document | core | ACTUS | FIBO |
|---|---|---|---|---|
| Shares / common equity | articles, shareholders' agreement | shares held; distributions by share | STK | Equity instrument |
| Partnership / LLC / JV interest | LPA, LLC agreement, JV agreement | commitment; capital account; allocation percentages; preferred return; distribution ordering | — | Partnership interest |
| Fund interest | LPA, subscription agreement | commitment; draws; distributions; carried interest | — | Fund unit |
| Tax-equity partnership | LLC agreement with flip terms | pre- and post-flip allocations; target yield; the flip test | — | — |
| Preferred equity | articles, subscription | a preference rate ahead of common | — | Preferred share |
| Residual / equity certificate | trust agreement | what remains after the notes | — | — |

### C. Use of an asset
| family | governing document | core | ACTUS | FIBO |
|---|---|---|---|---|
| Lease (space, ground, equipment) | lease | rent; term; escalation; free rent; renewal rights | — | Lease |
| Licence / royalty | licence agreement | rate on a defined basis; minimum; term | — | Licence |
| Concession / franchise (PPP, toll) | concession agreement | the right to operate and collect for a term; a concession fee or revenue share; hand-back | — | — |
| Easement / right of way | easement | a fee for a right, usually one-shot or fixed | — | — |

### D. Sale of output and services
| family | governing document | core | ACTUS | FIBO |
|---|---|---|---|---|
| Offtake (PPA, tolling, capacity, availability, merchant) | PPA, tolling agreement, capacity contract | quantity; price; term; escalation; availability | — | Commodity contract |
| Supply / procurement | supply agreement | the same agreement seen from the buyer | — | — |
| Service (O&M, management, servicing, administration) | services agreement | fee; term; escalation | — | Service agreement |
| Construction (EPC, GC) | construction contract | contract sum; draw schedule; retainage; liquidated damages | — | — |

### E. Transfer of the asset
| family | governing document | core | ACTUS | FIBO |
|---|---|---|---|---|
| Purchase / acquisition | PSA, SPA | price; closing; adjustments | — | Purchase agreement |
| Sale / disposition | PSA | proceeds or a valuation basis; selling costs | — | — |
| Option to buy or sell | option agreement | strike; exercise window | OPTNS | Option |

### F. Risk transfer
| family | governing document | core | ACTUS | FIBO |
|---|---|---|---|---|
| Derivative (rate swap, cap/floor, FX, future) | ISDA master + confirmation | notional; reference; fixed leg or strike; settlement | SWPPV, SWAPS, CAPFL, FXOUT, FUTUR | Derivative |
| Insurance | policy | premium; coverage; deductible; term | — | Insurance policy |
| Guarantee / credit enhancement (parent guarantee, LC, completion guarantee) | guarantee, LC | guarantor, beneficiary, the obligation covered; cap; fee | CEG | Guarantee |
| Collateral / security interest | security agreement, pledge | the asset pledged for the obligation | CEC | Collateral |
| Credit default swap, repo, margining | ISDA, GMRA | trading-book instruments | CDSWP, REP, MAR | — |

### G. Public and fiscal
| family | governing document | core | ACTUS | FIBO |
|---|---|---|---|---|
| Tax liability and attributes (cash taxes, credits, depreciation) | statute | rate or amount; basis | — | — |
| Grant / subsidy / support payment | grant agreement, support agreement | amount, or a top-up to a target on a basis | — | — |

### H. Not agreements
Reserves, escrows and sinking funds are ACCOUNTS: cash locations with a
balance, fed and drawn by agreements. Provisions and obligations (asset
retirement, decommissioning) are EXPENSE LINES with a timing. Cash and
commodity positions (ACTUS `CSH`, `COM`) are entity fields. Stock-based
compensation is an expense line. None needs a master.

## 3. The roster against the families

| family | master | status |
|---|---|---|
| Loan / credit facility | `Contract.Debt` | in the base; `amortization` covers bullet (PAM), level_pay (ANN), interest_only, custom — LINEAR (LAM) and NEGATIVE (NAM) are recognised patterns it does not yet name |
| Debt security | `Contract.Security` | drafted (`docs/40` §4.13) |
| Hybrid | `Contract.Security` + `Contract.Option` | a security carrying an election; no master of its own |
| Finance lease | `Contract.Lease` + `Contract.Option` | a lease with a purchase option; the refinement decides whether its payments amortize |
| Shares, partnership, fund, tax-equity, preferred, residual | `Contract.Equity` | drafted (§4.14); each is a refinement, the flip and the preference being refinement terms |
| Lease | `Contract.Lease` | in the base |
| Licence / royalty | `Contract.Royalty` | drafted (§4.15) |
| Concession | `Contract.Lease` on the concessionaire's side, with `Contract.Grant` for any availability support | a right to use and collect for a fee; no master of its own |
| Easement | `Contract.Lease` | a one-shot or fixed-fee lease |
| Offtake | `Contract.Offtake` | in the base |
| Supply | `Contract.Offtake`, buyer's side | the side is open on the master for exactly this |
| Service | `Contract.Service` | in the base |
| Construction | `Contract.Construction` | in the base; liquidated damages are a refinement's |
| Purchase | `Contract.Purchase` | in the base |
| Sale | `Contract.Sale` | in the base |
| Option | `Contract.Option` | in the base |
| Derivative | `Contract.Derivative` | in the base |
| Insurance | `Contract.Insurance` | in the base |
| Guarantee / credit enhancement | — | **missing**; see §4 |
| Collateral | — | not a contract: a RELATION between an obligation and an asset (`docs/13` §7.89) |
| CDS, repo, margining | — | out of scope: trading-book instruments outside a cash-flow model of a deal |
| Tax | `Contract.Tax` | in the base |
| Grant | `Contract.Grant` | drafted (§4.16) |
| General lines | `Contract.Line` kinds | in the base |

## 4. Findings

**One family is missing: the guarantee.** Every project financing carries
one (a completion guarantee, a parent guarantee, a debt service reserve
letter of credit), every ABS has a form of credit enhancement, and ACTUS
gives it a class of its own because it creates a three-party
relationship: guarantor, beneficiary, and the obligation covered. Its
core is small — the obligation it stands behind, a cap, a fee — and its
cash is a fee paid and a claim drawn on the guarantor's shortfall. It is
the one recognised family the roster cannot express, and `Contract.
Guarantee` is proposed for the rework of the drafted cores.

**Two patterns are missing from a master's vocabulary.** `Contract.Debt`'s
`amortization` names bullet, level payment, interest-only and custom;
the industry also names LINEAR (equal principal, ACTUS `LAM`) and
NEGATIVE (payment held, term shifts, `NAM`). Both are one word on the
existing term and belong there.

**Several families are refinements, not masters, and the survey says
why.** A hybrid is a security with an election; a finance lease is a
lease with a purchase option; a concession is a lease on the
concessionaire's side; a supply agreement is an offtake seen from the
buyer; a fund interest, a preferred share and a residual certificate are
equity interests with different terms. Naming a master for any of them
would put the mechanism where the agreement belongs. The masters are the
families in §2; the refinements are the industry's forms of them.

**Three things are not contracts** and keep their homes: collateral is a
relation, a reserve is an account, a provision is a line.

## 5. Demonstration map

What each master is demonstrated by today, and where a demonstration is
owed. A gap here is an item for the benchmark programme, and never a
reason to hold a master back.

| master | demonstrated by | owed |
|---|---|---|
| Debt | every CRE, credit, energy and opco debt case | a linear amortizer; a negative amortizer |
| Security | — (note classes are hand-carried in the auto ABS and REMIC cases) | the auto ABS pilot's classes as declared securities |
| Equity | — (Penzance, One Lincoln, the flip cases hand-roll the interest) | a JV under D13; the flip partnership |
| Lease | office, retail, multifamily cases | a ground lease; a finance lease |
| Royalty | — (CREST restates it) | CREST's royalty as a contract |
| Concession | — (the toll road models it as streams and phases) | the toll road on a lease refinement |
| Offtake | PPA, merchant, capacity, storage | a supply agreement on the buyer's side |
| Service | O&M, servicing | a management agreement on a CRE case |
| Construction | — | a construction contract with retainage |
| Purchase, Sale | acquisitions and exits in every domain | — |
| Option | management options, calls, renewals | a purchase option on a finance lease |
| Derivative | — | a rate swap on a floating construction loan |
| Insurance | — (HUD's MIP is a hand stream) | the HUD case's MIP |
| Guarantee | — | a completion guarantee on a construction financing |
| Tax | cash taxes, ITC, PTC, depreciation | — |
| Grant | — (the toll road's subsidy is a hand stream) | the toll road's coverage subsidy |
