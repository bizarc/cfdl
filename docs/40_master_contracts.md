# Master contract types — the construct, stated top-down

Status: **stages 1 and 2 of seven built** — principles and the roster settled in
discussion 2 September 2026; R1–R3 decided the same day (§4.12, §7, §8).
Stage 1 (the ontology model in `cfdl-pack`: fields, roles with
specialization, lines, side, the effective walks, the load checks, the
roster below in the language base, and every pack's roles specialized)
is in. Stage 2 (the packs conform: hard renames to the masters' names,
`line` on every rule, template coverage as a load check) is in. Stages 3–7
are listed at the end and each lands as its own change.
Repository-only; the site carries the result, not the argument.

## 1. Why this document exists

The packs were written first. Each grew a contract vocabulary from the
sources it reconciled against, and the masters were added afterwards as a
classification: `CRE.Contract.PermanentDebt refines Contract.Debt` records
that a mortgage is a debt, and nothing else. A master today declares a
role pair and a description. It promises no field, names no line of cash,
and no check exists that a refinement keeps a promise it never made. The
consequence is visible in the packs' own words: the master says `lessor`
and every lease says `landlord`; the master will say `principal` and the
credit pack says `balance`; one pack says `escalation` and its neighbour
`price_escalation` for the same thing. The compiler cannot see any of it,
because a contract's type is recovered by string prefix at each consumer
and recorded in the IR as the literal `core.Contract`.

That was the right order to learn the domains in, and it is the wrong
order to leave the language in. A contract is meant to be the construct
that records what was agreed and lets everything else refer to the
agreement rather than to the cash. This document states what a master
contract type IS, top-down, and what follows for packs, the compiler, the
results, and the tools.

## 2. Principles

1. **A master is defined from what the agreement is.** A debt has a
   principal, a rate, a term, an amortization shape, a day count, a
   payment frequency, a lender and a borrower, because that is what debt
   is — not because four packs happen to consume those words. Nothing on
   a master is mined from pack usage, and nothing on a master is gated on
   a benchmark: the roster is read from the industry's agreements and
   their governing documents (`docs/41`), and a benchmark is then chosen
   or built to DEMONSTRATE a master, never to admit one. Where a pack's
   word differs, the pack conforms. A pack contract is the other layer —
   a bundled solution to a cash-flow scenario that refines a master and
   carries its terms, parties, rules, template, validations, metrics and
   lifecycle together (`docs/41` §0).
2. **Packs inherit and specialize; they never redefine.** A refinement
   carries every master field, role and line without restating it. It
   may strengthen a field (optional to required, a tighter bound), add
   fields of its own, specialize a role, and add lines. It may not retype,
   re-unit, weaken or drop anything the master declared. A reader who
   learned the master must not be lied to by the refinement — the rule
   the entity side already enforces (`docs/07` §6.1).
3. **Roles are generic on the master and specialized by the pack.** The
   master says `lessor` and `lessee`. `landlord` is what CRE calls a
   lessor; it is declared as a specialization of the master role, and
   every check, result and selector can name the party by either. A
   domain word never appears on a master.
4. **A master carries classification information and fixes no
   category.** It names the economically distinct LINES its refinements
   must produce (a debt produces proceeds, interest and principal) and
   which SIDE of the agreement the subject entity is on. The pack
   classifies each line into the cash flow statement (`docs/35`): a
   borrower's interest is `financing.debt.interest_paid`; a lender's is
   `operating.collection.interest` when lending is its business. The two
   axes stay orthogonal.
5. **General revenue, expense and capital-expenditure lines are
   contracts with the model, not with a counterparty.** The eleven pack
   types that refine nothing today are statement-line generators. They
   get masters that say exactly that (§4.12) rather than being forced
   under a counterparty master they do not fit.
6. **A contract records; it does not decide.** Whether a right is
   exercised, whether a covenant is breached, when a pool is called —
   these are events and states (`docs/26`, `docs/34`). A master never
   carries a guard.

## 3. What a master declares

Every master carries the same members, and every refinement inherits
them. The shape is the entity side's, extended with the three things a
contract has that an entity does not: roles, lines, and a side.

| member | meaning | inheritance |
|---|---|---|
| `roles` | the parties to the agreement, by generic role name | a refinement MUST cover every master role, inherited as is or specialized (`landlord refines lessor`) |
| `subject_family` | which family the contract sits on (`asset` today) | inherited; a refinement may narrow to a type |
| `term` | the effective dates; a one-shot uses `term_start` as its instant | always present (`docs/01` §8.1) |
| `fields` | the terms the agreement states: name, type (`decimal`, `integer`, `string`, `date`, or `contract` — a reference to a declared contract by name, as a guarantee's `covered`), required, unit | inherited; strengthen-only redeclaration; a refinement may add |
| `lines` | the economically distinct cash the agreement produces, by role name; a line is LOWERED by a rule, ALLOCATED by a waterfall step (a security's principal), or OPTIONAL — named so every pack spells it alike, required of none | a refinement's rules MUST emit every lowered master line; may add |
| `side` | whether the subject pays or receives on each line | inherited; a refinement may fix it where the master leaves it open |
| `balance_field` (stage 6) | which field is the running balance a state may extinguish | inherited |

**Terms are fields.** There is no second member kind for contracts. The
"term schema" `docs/07` §6.3 once described as a JSON side-file, and its
`E4003`, never existed in code; today's effective schema is the union of
what rules consume, templates render and validations bound, and the
three already disagree. After this design the master's `fields` are the
schema, the rule consumes them by name, the template renders the required
ones, and pack load checks both.

**Lowering is untouched.** A rule keeps reading `{{contract.principal}}`;
a template keeps rendering defaults; `validations.toml` keeps bounding
values; `inputs.` deferral keeps its rules (`docs/13` §7.56). What changes
is what a type DECLARES, never how it LOWERS — with one addition: each
rule names the line it emits (`line = "interest"`), so the master's
promise is checkable.

## 4. The roster

Twenty-one masters in the language base: the fifteen counterparty
masters, `Contract.Line`, and its five specializations for the general
lines (§4.12, decision R1). The last five (§4.13–4.17) were reworked from
their governing documents on 4 September 2026 against the survey in
`docs/41`. For each: the roles, the fields (required unless marked opt,
with the default where one is a fact of the instrument rather than of a
deal), the lines, and the side. The argument is given where the choice
was not obvious.

### 4.1 `Contract.Debt`
Roles `lender`, `borrower`. Fields: the amount borrowed, stated one way
(`principal`, or a facility's `commitment`, or a `draw_curve` the facility
funds against — one required); the rate, stated one way (`interest_rate`
fixed, or `index_curve` with `margin` floating — one required); `day_count` (opt; the model's convention when absent);
`payment_frequency` (opt; the calendar's when absent); `amortization` (opt;
the repayment pattern — `level_pay`, `interest_only`, `bullet`, `custom`;
a refinement fixes it); `amortization_months` (opt; the
horizon the payment is struck on, which may exceed the term);
`interest_only_months` (opt, 0); `funded_at_close` (opt, 1);
`balloon_at_maturity` (opt, 0). Line: `interest` — what EVERY debt produces;
`proceeds` and `principal` are lines a refinement adds, because a purchased
pool has no proceeds and a construction facility repays nothing inside the
model. Side: open. Master names are full words: a master is
read by people who do not know the pack's abbreviations, so `rate`,
`amort_months` and `io_months` are the packs' spellings and not the
master's (decision R2's naming review). Lines:
`proceeds`, `interest`, `principal`. Side: open on the master — the
subject is the borrower for a mortgage and the lender for a held pool —
and each refinement fixes it. The credit pack's `balance` is this
master's `principal` (§7); the distinction the pack draws between a
pool's outstanding and a loan's original is `amort_months` versus
`term`, not a second notional. A debt has a repayment pattern unless it
does not amortize, and `amortization` states it; the master does NOT
enumerate every pattern the industry knows (decided 4 September 2026). A
level-payment pool, an interest-only bullet, a linear or a negative
amortizer are different pack contracts — different rules, different cash
— so the pattern IS the refinement, and a pattern the master's four words
do not name (linear, negative) is the refinement's word. ACTUS supplies
the shared names (`ANN`, `PAM`, `LAM`, `NAM`, `CLM`) the refinements use
consistently across packs (`docs/41` §1).

### 4.2 `Contract.Lease`
Roles `lessor`, `lessee`. Fields: a rent stated one way — `rent` (per
period) or `rent_year` (annual), one required; `escalation` (opt, 0);
`free_rent_months` (opt, 0). Line: `rent` (`abatement` is a refinement's
addition). Side: open — the subject is the lessor in a property model and
the lessee in a tenant's. Recoveries, expense stops, TI/LC, rollover probability and
percentage rent are CRE's extensions, which is why `CRE.Contract.Lease`
is itself a pack-level master and the only two-level chain in the packs
today. That chain is the pattern: a master states what every lease has, a
pack master states what every lease in its market has, the leaf states
what this lease type adds.

### 4.3 `Contract.Purchase`
Roles `buyer`, `seller`. Field: `price`. Line: `price`. One-shot at
`term_start`. Side: buyer pays. The two existing refinements already
have exactly this shape.

### 4.4 `Contract.Sale`
Roles `seller`, `buyer`. Fields: `selling_costs` (opt, 0), and ONE
valuation basis, required as a group: `value`; or `cap_rate` with
`income`; or `multiple` with `base`; or `discount_rate` with `growth_rate`
and `base`. (`income`, not `noi`: net operating income is CRE's word for
the income a cap rate is applied to, and the master is not CRE's.) Line: `proceeds` (`selling_costs` where a refinement charges them).
One-shot. Side: seller receives. The "required as a group" form (`any_of`) is new to the field
model and is needed here and on the rent of §4.2.

### 4.5 `Contract.Supply`
Roles `supplier`, `buyer`. Fields: `price` (per unit, or per year where
the payment is for availability); `quantity` (opt — per year, in the pack's
unit; absent for a capacity payment, which pays for availability rather
than volume); `escalation` (opt, 0); `degradation` (opt, 0); `availability`
(opt, 1). Line: `revenue`. Side: open. Named `Supply` rather than
`Offtake` (decided 4 September 2026): offtake is the project-finance word
for the long-term purchase of a plant's output, and the general
commercial family is the supply agreement — goods or output delivered
over a term for a price — seen from either side. The open side is what
lets one master serve a PPA from the seller's seat and a fuel supply
agreement from the buyer's. Energy specializes the roles: `seller`
refines `supplier`, `offtaker` refines `buyer`. A merchant sale names
only the seller — its buyer is the market — and the master allows a
role to be unbound where the refinement says so. Energy's `ppa_price`,
`price` and `payment_year` are one field; its `mwh_year` is `quantity`.

### 4.6 `Contract.Service`
Roles `provider`, `recipient`. Fields: `fee` or `fee_year` (one required);
`escalation` (opt, 0). Line: `expense`. Side: open. O&M is
the first refinement; a management agreement, a servicing agreement and
an administration agreement are the next, and the auto ABS trust's
servicing and administration fees (`benchmarks/credit/auto_abs_tranches`)
are the case that will want them.

### 4.7 `Contract.Tax`
Roles `taxpayer`, `authority`. Fields: `tax_rate` or `amount` (one
required); `basis` (opt). Lines: none on the master — a refinement adds
`paid` or `benefit`, because no single line is common to a cash tax, a
credit and a depreciation shield. Side: open. The weakest
master to state a core for — a cash tax, an investment credit, a
production credit and a depreciation shield share only "an obligation or
attribute against a revenue authority, with a period" — and that is what
the core says. Refinements carry the rest.

### 4.8 `Contract.Option`
Roles `grantor`, `holder`. Fields: `strike` (opt); the election and the
payoff come from the `option` grammar (`exercise when`, `payoff`) and are
declared there, not in `terms`. Line: `payoff`. An election binds no
lowering rule. The base carries four generic elections as concrete
refinements — `Option.Call`, `Option.Put`, `Option.Renewal`,
`Option.Refinance` — so a model with no pack active can write one, and an
option's type is checked against them and the pack's own (`E1373`,
`E1374`; stage 3). Stage 7 aligns the grammar with the master and decides
whether the generic names stay.

### 4.9 Construction — removed from the roster, 4 September 2026
There is no `Contract.Construction`. A build is capital expenditure on a
draw curve inside a construction phase, which is what the language
already does well, and the construction loan that funds it is a
`Contract.Debt` reading the same curve from the other side. What a
construction contract adds over the spend is a counterparty — the
contractor — and a holdback: retainage withheld from each draw and
released at completion, plus in some forms liquidated damages and change
orders. A cash-flow model rarely needs the contractor as a party: it is
not paid through a waterfall, holds no account the model reads, and
takes a share of nothing. Retainage is a fact about the spend's timing
and belongs as an optional term on a pack's capital-expenditure
refinement; a model that needs the contractor names it as the party the
draws are paid to. The survey's test (`docs/41` §4) asks what every
instrument of the kind states that no other master can express, and for
a construction contract the answer is only the counterparty. Insurance
and Derivative pass the same test because their cash is contingent on
something outside the model; a construction contract's cash is a
schedule. `cre.construction_stub` refines `Contract.CapitalExpenditure`,
as it did.

### 4.10 `Contract.Derivative`
Roles `party`, `counterparty`. Fields: `notional`; `reference` (a
declared curve or quantile); one of `fixed_rate` or `strike`. Line:
`settlement`. Side: open. No refinement yet; declared because a rate swap
on a floating loan is the first thing a project-finance case will ask for.

### 4.11 `Contract.Insurance`
Roles `insurer`, `insured`. Fields: `premium`; `coverage` (opt);
`deductible` (opt). Line: `premium` (`claim` where a refinement models
recoveries). Side: insured pays.
HUD's mortgage insurance premium is the standing case
(`benchmarks/cre/hud_home_multifamily`), which today is a hand stream
because the debt contract rightly refused to carry it.

### 4.12 `Contract.Line` and its kinds — decision R1
Role `owner` (the model's own party; no counterparty). Fields: `amount`,
`amount_year`, `growth_rate` (opt, 0) — all optional on `Contract.Line`
itself, because a line may be derived (a vacancy allowance is a rate on a
base; a working-capital policy is days); the plain kinds Revenue, Expense
and CapitalExpenditure strengthen `amount`/`amount_year` into a required
group. `Contract.Line` is the pure master and is itself abstract; it is specialized, still
abstractly, into the kinds a statement distinguishes, and a pack refines
those:

| kind | line | side |
|---|---|---|
| `Contract.Revenue` | `revenue` | receives |
| `Contract.Deduction` | `deduction` | pays — a contra-revenue line: vacancy, credit loss, an abatement stated as a line |
| `Contract.Expense` | `expense` | pays |
| `Contract.CapitalExpenditure` | `capex` | pays |
| `Contract.WorkingCapital` | `working_capital` | open — a balance movement, so it varies by period |

*Decided 2 September 2026 (R1):* one pure master, specialized by kind, so
"all revenue" is a type and the packs specialize further
(`cre.opex_line` refines `Contract.Expense`; `cre.vacancy_loss` refines
`Contract.Deduction`; `opco.working_capital` and its policy refine
`Contract.WorkingCapital`). Deduction and WorkingCapital are the two
kinds the packs already distinguish that the first three did not cover;
a further kind is added when a pack needs one, never speculatively.

### 4.13 `Contract.Security`
*Reworked from the governing document, 4 September 2026.* The source is
a trust indenture, read through the articles every one carries: the
definitions and the form of the notes; PAYMENTS, which fix the principal
amount, the interest rate or the index and margin, the payment dates and
record dates, the day count and the final maturity; REDEMPTION, optional
and mandatory, and any clean-up call; the APPLICATION OF COLLECTIONS or
priority of payments, which in a securitisation indenture says how each
payment date's cash is distributed among the classes; covenants; events
of default and remedies. The Trust Indenture Act shapes the trustee and
default articles; the payment and priority articles are where the cash is.

Roles `issuer`, `holder`. Fields: `face` (the initial principal amount);
the coupon, stated one way (`coupon` fixed, or `index_curve` with `margin`
floating — the same words a debt's rate uses, because a holder reads them
the same way; optional on the master, because a pass-through's interest
follows the collateral and states no coupon of its own, and a structured
note's refinement requires it); `payment_frequency` (opt; the calendar's
when absent); `day_count` (opt; the model's when absent). The final
maturity is the contract's `term` end, as on every master. Lines:
`interest` and `principal`. Whether each is LOWERED by a rule or
ALLOCATED by a priority of payments is the REFINEMENT's statement, not
the master's (corrected 4 September 2026): a pass-through — a Fannie Mae
or Ginnie Mae certificate, a participation — pays its holder its share of
what the collateral produced each period, scheduled and unscheduled
principal alike, and nothing is ranked or chosen, so both lines are
lowered from the collateral; a structured note — a REMIC tranche, an ABS
class — is paid what the priority allocates it, so the refinement marks
both allocated, the steps pay the holder's account, and the claim is
`face` less what the account has received (D7). The load check asks no
rule for an allocated line. Named optional lines a
refinement may add, so two packs never spell the same cash differently:
`proceeds` at issuance, `premium` for a make-whole, `redemption` for a
call at a stated price. Interest shortfall carried forward is a field.
Side: open — an ABS model sits with the issuer, a portfolio model with
the holder.

**Refinements, not the core.** Ranking within the issuance, which is the
waterfall's step order — a `seniority` term would say it a second time
and could not say what a waterfall can (sequential here, pro rata there,
a step-down after a trigger). Redemption provisions, which are options:
the clean-up call already refines `Contract.Option` in the credit pack.
Credit enhancement — the reserve, the overcollateralisation target —
which are accounts and structural tests. A sinking fund, a step-up
coupon, a PIK toggle, original issue discount. Events of default, which
are events. A residual certificate is not a security: it has no face and
no coupon, and refines `Contract.Equity`.

**Balance.** For a structured note, derived, never lowered: the claim
over the holder's account, and the note retires when the account reaches
its face. For a participation there is no claim to keep: the holder's
position is its share of the pool's. Stage 6's balance role belongs to
`Contract.Debt`. **First refinement:** the credit pack's
`credit.participation` — a `share` of a pool's cash, both lines lowered,
carrying the pool's suffix; `fixtures/valid/credit_participation` shows
a share of one reconciling to the pool's net collections. **Second:** the
structured note, with the auto ABS pilot's seven classes declared as notes
whose steps read `face` and `coupon` from the contract — the change that
also lands the step naming the contract and line it pays.

### 4.14 `Contract.Equity`
*Reworked from the governing document, 4 September 2026.* The source is a
limited partnership agreement or an LLC operating agreement, through the
articles every one carries: the parties and their PERCENTAGE INTERESTS;
CAPITAL CONTRIBUTIONS — the commitment, how it is called, what happens on
a shortfall; CAPITAL ACCOUNTS — contributed capital less distributions
and allocated profit and loss; ALLOCATIONS of income and loss for tax;
DISTRIBUTIONS — the order cash is paid out: return of capital, the
preferred return, the catch-up, the promote or carried interest;
transfer, withdrawal and dissolution. A joint venture agreement carries
the same articles with a developer's control rights; a fund LPA adds
management fees and a term.

Roles `issuer`, `holder` — the venture or the fund issues the interest,
the partner, member or limited partner holds it, and a pack specializes
the holder's word. Fields: `commitment` (the capital the holder agreed to
contribute); `share` (the holder's percentage interest — its share of
distributions before any promote); `preferred_return` (opt; the annual
rate the holder's contributed capital accrues ahead of the promote —
absent on common equity). Lines: `contribution`, lowered — the commitment
funded on its call schedule, the one cash an equity agreement produces by
its own terms; `distribution` — a pro rata share lowered by rule, or,
where a priority of distributions decides it, marked ALLOCATED by the
refinement and paid as steps into the holder's account, as a structured
note's principal is (§4.13). The holder's position is the account,
contributed less distributed, and the accrued preference is a field on
that account (D13). Side: open — a fund model and an investor's model sit
on opposite sides.

**Refinements, not the core.** The distribution order itself — return of
capital, preferred return, catch-up, promote tiers, the split — which are
waterfall steps reading `share`, `preferred_return` and the holder's
account. Tax allocations, which are not cash. The flip: a tax-equity
interest adds a pre-flip and a post-flip `share` and the target yield the
flip tests. A fund's management fee, which is a `Contract.Service` between
the fund and the manager; its carried interest, which is the promote. A
preferred share is this master with a `preferred_return` and no promote. A
residual certificate is this master with a `share` of the whole remainder.
Capital calls on a shortfall, defaulting-partner dilution and transfer
rights are refinement terms or events.

**Balance.** The holder's account. No lowered field, so no stage 6 role.

### 4.15 `Contract.Royalty`
*Reworked from the governing document, 4 September 2026.* The source is a
licence or royalty agreement, through the articles every one carries: the
GRANT — what right is licensed, for what territory and term; the ROYALTY —
the rate and the base it is computed on, with the definitions of net sales
or gross revenue that decide what counts; MINIMUMS — a floor paid whether
or not the base reaches it; ADVANCES — a payment up front recouped against
future royalties; reporting and audit, which fix when the royalty is
calculated and paid; term and termination. A mineral or oil-and-gas
royalty, a music or publishing royalty, a franchise fee and a land royalty
on a solar or wind lease carry the same articles with different bases.

Roles `licensor`, `licensee` — the licensor owns the right and receives,
the licensee uses it and pays; a pack specializes the words (lessor and
operator on a mineral lease, landowner and project on a solar site).
Fields: `rate` (the share of the base paid, a ratio); `basis` (what the
rate applies to — a selector over the series the licensee's own
agreements produce, because a royalty is a claim on ANOTHER agreement's
revenue and the base is defined by reference to it); `minimum` (opt, 0;
the floor per period); `advance` (opt, 0; paid at term start, recouped as
royalties accrue). Lines: `royalty`, lowered — the greater of rate on
basis and the minimum, less any unrecouped advance; `advance`, a named
optional line, at term start where one is stated. Nothing is allocated: a
royalty is paid by its own terms. Side: licensee pays.

**Refinements, not the core.** Tiered rates stepping with cumulative
sales, a curve on the cumulative base. Deductions from the base, the
net-sales definition, in the selector or a `deductions` term. A
most-favoured-licensee clause or an audit true-up, which are events. An
annual minimum on a monthly model, a cadence question handled as a lease's
annual rent is. **Balance:** the unrecouped advance, where one exists, a
field the line reads. **Demonstration owed:** CREST's royalty as a contract
reading the PPA's revenue, the case that could only restate it as a stream.

### 4.16 `Contract.Grant`
*Reworked from the governing document, 4 September 2026.* The source is a
grant or support agreement between a public body and a project, through
the articles every one carries: PURPOSE AND ELIGIBILITY — what the support
is for and what the recipient must be or do; the AMOUNT — a fixed sum, a
schedule, or a formula on a measured base; PAYMENT CONDITIONS — when each
payment is made and what must be certified first; the term; CLAWBACK — when
support is repaid. The same articles appear in a tax increment financing
agreement (the base is the increment, the target is the TIF bonds' debt
service), a minimum revenue guarantee on a toll concession (the base is
toll revenue, the target a stated coverage), an availability-based support
payment, and a capital grant paid on completion.

Roles `grantor`, `recipient` — a pack specializes the words (authority and
concessionaire, agency and developer). Fields: the support, stated one way
(`amount` per period or `amount_year`, a fixed sum spread by the
calendar; or `target` with `basis`, a top-up of a measured series to a
stated level — one required); `cap` (opt; the most the grantor pays over
the term — a fixed-sum grant is its own cap, an uncapped top-up is real).
Line: `support`, lowered — the fixed amount, or the shortfall of the basis
below the target, bounded by the cap. Nothing is allocated. Side:
recipient receives.

**Refinements, not the core.** Eligibility — an affordability share, an
income band, a unit count, a completion certificate — which is a set of
facts about the recipient tested by an event. Clawback, an event that
reverses support paid. A TIF's base, the incremental assessed value times
a rate, a field the refinement lowers so `basis` can name it. Availability
tests, the same eligibility shape on a per-period fact.

**Neither a tax attribute nor a supply.** `Contract.Tax` is a position
against the recipient's own liability and lands there — a credit, a
shield, an abatement. `Contract.Supply` buys something for a price. A grant
buys nothing and offsets no liability; it is support paid because a public
party agreed to pay it. A TIF sits across the boundary and the survey's
mapping keeps it straight: the abatement of the project's own taxes is
Tax, the increment paid to the project or its bondholders is Grant, the
TIF bonds are Security. **Demonstration owed:** the toll road's coverage
subsidy as a contract with a target and a basis, replacing a formula
stated three times by hand.

### 4.17 `Contract.Guarantee`
*Added from the survey (`docs/41` §4), 4 September 2026.* The source is a
guarantee or credit support agreement — a three-party instrument, and its
articles say so: the GUARANTEED OBLIGATION, naming the agreement and the
obligor whose performance is covered; the BENEFICIARY, who may call on it;
the guarantor's UNDERTAKING, on demand or on default, with any conditions;
the LIMIT, a cap on the guarantor's liability, sometimes reducing; the
FEE, where the guarantor is paid; the term, usually ending when the
underlying obligation is discharged or a milestone is reached;
SUBROGATION, the guarantor's claim against the obligor after paying. A
parent guarantee on a construction loan, a completion guarantee, a letter
of credit standing in for a debt service reserve, a monoline's bond
guarantee and a government revenue guarantee carry those articles. ACTUS
gives the family its own class (`CEG`) for the same reason.

Roles `guarantor`, `beneficiary`, `obligor` — three, the one thing that
makes this master unlike every other; the obligor is a role because the
instrument covers that party's performance, and a refinement may leave it
unbound where the obligor is the model's own subject. Fields: `covered`
(the agreement whose performance is guaranteed — a declared contract, by
name: the reference field type of §3, because a guarantee without the
thing it guarantees is not a guarantee and a reference is what the
compiler can check); `limit` (the most the guarantor pays — an unlimited
guarantee is a limit that cannot bind); `fee` (opt, 0; per period, on the
limit or the covered balance — a parent guarantee is usually free, a
letter of credit is not). Lines: `fee`, lowered; `claim`, ALLOCATED — what
the guarantor pays the beneficiary on a shortfall of the covered
agreement, a step drawn from the guarantor rather than from collections,
bounded by the limit less claims paid, and not lowered because the covered
agreement's shortfall sizes it; `recovery`, a named optional line — the
guarantor's recovery from the obligor by subrogation. Side: open.

**Refinements, not the core.** The trigger — on demand, or on a defined
default — an event on the covered agreement's state. A reducing limit, a
curve. Release conditions — completion, a coverage test met for
consecutive periods — events that retire the guarantee. A letter of
credit's commitment fee and draw mechanics are this master with a bank as
guarantor. **Balance:** the limit less claims paid, derived from the claims
allocated, as a security's claim is derived from its holder's account.
**Demonstration owed:** a completion guarantee on a construction financing.

**The five together.** Security and Equity are the financing side of
every structured deal, and until this stage both were invisible to the
language — note classes as `assume` values and party accounts, partnership
interests as hand-rolled preference fields. Royalty and Grant are the two
agreements the bespoke and energy cases restate as streams because no
master gave a pack a place for them; Guarantee is the one family the
survey found the roster could not express. All five are added before
their first refinement on the argument §4.10–4.11 already made: a master
that exists before its refinement costs nothing, and its absence is what
forces the hand-rolled stream. Two things landed with them for every
master: a line may be ALLOCATED (§6), and a field may be a reference to a
declared contract (§3).

## 5. Roles

A role on a master is generic. A refinement covers each master role in
one of two ways: it inherits the name, or it declares a specialization —
`landlord` refines `lessor`. In `types.toml`:

```toml
[[contracts]]
type_id = "CRE.Contract.UnitLease"
refines = "CRE.Contract.Lease"
contract_name = "cre.lease_unit"

[[contracts.roles]]
name = "landlord"
refines = "lessor"

[[contracts.roles]]
name = "tenant"
refines = "lessee"
```

`parties = ["landlord", "tenant"]` remains the shorthand when no
specialization is stated — for a master, or for a refinement whose roles
are the master's own. A role a master declares and a refinement neither
inherits nor specializes is a load error. A refinement may leave a master
role UNBOUND in a model (the merchant sale's buyer) by saying so:
`[[contracts.roles]] name = "offtaker" refines = "buyer" unbound = true`.

A model's `parties { landlord = party.acme }` is validated against the
effective roles, and the binding is recorded under the master role as
well as the pack's word, so `party.acme` is a lessor to any reader that
does not know CRE. This is what makes "all lenders" answerable across
packs, and it is what joins a contract to the account model: the lender a
Debt names is the party whose account the waterfall pays
(`docs/13` §7.96–7.98 and the auto ABS conversion).

## 6. Lines and side

A master names its lines by role. Each `[[rules]]` entry in a pack's
lowering names the line it emits:

```toml
[[rules]]
id = "cre_permanent_debt_interest"
contract_name = "cre.permanent_debt"
line = "interest"
category = "financing.debt.interest_paid"
```

Pack load checks that, for every concrete type, the set of `line` values
across its rules covers its effective LOWERED lines. An ALLOCATED line is
what a structure pays through a priority of payments: no rule may emit
it, and a waterfall step pays it into the holder's account (the step
naming the contract and line it pays is the change that follows). WHO
MARKS IT: a master marks a line allocated only where every form of the
agreement is paid by a structure — a guarantee's `claim`, sized by the
covered agreement's shortfall; a security's `principal` and an equity
interest's `distribution` are lowered on the master, and a REFINEMENT
marks them allocated where its form is a structured one (a REMIC tranche,
a fund's tiers) and leaves them lowered where it is not (a pass-through,
a pro rata share). A refinement may mark a line allocated, never the
reverse. An OPTIONAL line is a name
the master reserves — `proceeds`, `premium`, `redemption`, `recovery` — so
a refinement that adds it spells it as every other pack would. The CATEGORY stays the
pack's, per `docs/35`: the master says a debt produces interest, the pack
says where a borrower's interest sits in the statement. A reporting
selector asks for a line by role across packs — `line interest` beside
`type Contract.Debt` on a slice or a statement row — without knowing any
pack's category spelling; a metric folds the slice's net as
`slice.<name>`; and every lowered stream series carries its `line` beside
its `contract`, so a consumer holding results alone can do the same
(stage 5, built).

`side` says which way cash runs for the subject on each line. It is
what lets the same master serve a mortgage on a property (the subject
pays) and a pool held by a trust (the subject receives) without two
masters or a flag in the model. A refinement fixes the side; a master
may leave it open.

## 7. Conformance and migration — decision R2

The packs conform to the masters. Where a pack's term name differs from
the master's field, the pack's name is renamed:

| pack term today | master field | packs |
|---|---|---|
| `balance` | `principal` | credit (three pool types) |
| `rate` (on debt) | `interest_rate` | cre, credit, energy, opco |
| `amort_months` | `amortization_months` | cre, opco |
| `io_months` | `interest_only_months` | cre, opco |
| `credit` (ITC), `credit_per_mwh` (PTC) | `amount` | energy |
| `exit_cap` | `cap_rate` | cre |
| `exit_multiple`, `base_value` | `multiple`, `base` | opco |
| `escalation` on opex/revenue lines | `growth_rate` | cre |
| `mwh_cycled_year`, `spread` | `quantity`, `price` | energy (storage) |
| `payment_year` (capacity) | `price` | energy |
| `ppa_price`, `price`, `payment_year` | `price` | energy |
| `mwh_year` | `quantity` | energy |
| `price_escalation` | `escalation` | energy |
| `om_year` | `fee_year` | energy |
| `base_rent_year`, `rent_year` | `rent_year` | cre |
| `base_rent` | `rent` | cre |
| `noi_value`, `noi_forward_year` | `income` | cre |

*How — decided 2 September 2026, done in stage 2:* a HARD rename, one
change for all four packs. The language is pre-release with no downstream consumers,
so there is no deprecated alias and no warning code; every benchmark,
fixture, template, learn chapter and site example is migrated in the same
change, and an old name is simply an unknown term. The master's names were
reviewed for correctness first (full words, no pack abbreviations, no
domain jargon — §4.1, §4.4, §4.7), because a hard rename is done once.

*Bounds — decided 3 September 2026, stage 3:* bounds stay in a pack's
`validations.toml`. A bound carries a code and a message in the pack's
own words (`E6053_CRE_DEBT_INVALID_RATE`), and the packs' READMEs and the
repair catalog cite them; the roster carries the SHAPE of a term (type,
unit, required, group), which is what every refinement must agree on. A
master stating `interest_rate ≥ 0` would say the same thing as four pack
validations with a fifth code, so it does not.

*The clauses and derived lines, decided in stage 3:* a refinement may put
a field of its own into a master's group. A rollover's
`renewal_rent_year` and `market_rent_year` join the lease's `rent` group,
because the rent of a successor lease IS its renewal or market rent; a
percentage-rent clause's `overage_pct` joins it, because the clause's
rent is stated as a rate on sales; an OpCo capex line's `pct_of_revenue`
joins `amount`. The master's obligation stands — a lease states its rent —
and the refinement states how this form of the agreement spells it.
`cre.vacancy_loss` refines `Contract.Deduction` and
`opco.working_capital_policy` refines `Contract.WorkingCapital`, whose
masters carry no amount group: a deduction and a working-capital movement
are derived by construction.

*Terms are fields, made real in stage 3:* no pack type declared a field of
its own before this stage — the effective roster was the masters' fields
alone, and a pack's terms lived in three places that agreed by care. Every
shipped type now declares every term its rules read, its validations bound
and its templates render (`[[contracts.fields]]`), strengthening a master's
field where it requires one (`principal`, `interest_rate`, `rent_year`),
and the loader refuses a pack where a rule, validation or template names a
term outside the roster. The credit pack's three pool shapes share an
abstract `Credit.Contract.Pool` for the terms every pool states.

## 8. What the compiler checks — decision R3, built in stage 3

A contract's type is resolved ONCE, from its declaration, and carried. The
two-token form states it (`contract cre.lease_unit tenant_a`); the fused
form (`cre.lease_unit.tenant_a`) is matched by rule-name prefix on the
same boundary lowering uses. Both spell the same qualified name, so
lowered streams and references are unchanged whichever form wrote them.
The binding — pack type, ontology type, master, instance — is what the
party check, the term check, lowering and the IR read; the prefix match at
each consumer is gone.

Against the effective roster the compiler refuses an unknown term with a
near-miss hint (`E1371_UNKNOWN_CONTRACT_TERM`) and a missing required
term or an empty group of alternatives (`E1372_MISSING_CONTRACT_TERM`),
before any rule is expanded. A type named on a declaration resolves or is
refused: an unknown type on a contract or an option, a fused contract name
no rule lowers, an election written as a `contract` or a lowered type
written as an `option` (`E1373_UNKNOWN_CONTRACT_TYPE`, with the near miss
or the declarable types); a master named where a concrete type belongs
(`E1374_ABSTRACT_TYPE_INSTANTIATED`, with its concrete refinements).
Roles are the type's effective roles resolved through the master chain:
a pack's specialization is bound by the pack's word, an unbound role is
refused, and `E1322` names the master's word beside each (`landlord (the
master's lessor)`).

The IR contract carries `type` (the ontology type, `core.Contract` only
with no pack), `contract_name`, `master`, `instance`, the `parties` with
`role` and `master_role`, and the `terms` as typed values — a number, a
string, or CFDL source for an input reference or an expression. The
`parties` block is no longer parsed and discarded.

*Scope — decided 2 September 2026:* the two-token declaration the grammar
already allowed lands with the type carry rather than as a follow-on:
leaving it open would have carried the string-surgery it exists to remove
into every consumer the stage touched. Models migrate to the two-token
form as they are next touched.

## 9. What a contract is not

Not a decision: exercise, breach and call are events or states. Not a
guard: a contract takes no `active when` (`docs/01` §13.4). Not a balance
the waterfall writes: a Debt's balance is its own field, driven by what
the contract lowers; the waterfall allocates cash to the parties the
contract names, and their accounts hold what they received (`docs/13`
§7.97). Not a category: the pack classifies.

## 10. Stages

1. **Ontology model** (`crates/cfdl-pack`) — **built.** `OntologyContract`
   carries `fields`, `lines`, `side` and `roles` with specialization and
   unbound markers; `effective_fields` covers contracts, with
   `effective_roles`, `effective_lines`, `effective_side` and `master_of`
   beside it; load validation mirrors the entity side (subject family,
   strengthen-only fields, every master role covered, specialization
   targets exist) and adds line coverage where a type's rules name their
   lines; the roster of §4 is in `language_base()`; every pack's roles
   are specialized and every pack type now refines a master. Template
   coverage of required fields waits for stage 2, where the packs' terms
   take the masters' names.
2. **Pack conformance** — **built.** Every rule names its `line` and the
   check is live for all four packs; the terms are renamed to the masters'
   names across packs, templates, validations, every model, every learn
   chapter and every pack README (§7); a contract template must render
   every required effective field and one member of each group, checked at
   load; credit's template no longer renders `smm`/`mdr`. Every benchmark
   `expected.csv` is byte-identical.
3. **Compiler** — **built.** Resolve-once type carry, the four diagnostics
   of §8, effective-roster term and role checks, parties in the IR, the
   two-token declaration; every pack type declares its terms as fields and
   the loader checks rules, validations and templates against the roster.
4. **Results and tools** — **built.** `results_version` 0.13 publishes the
   model's contracts in `graph.contracts` — name, type, master, the pack's
   name, instance, subject, parties with the master's role, and the
   streams lowered from each — and attributes each lowered stream series
   to its contract (`contract` beside `entity` and `category`), the IR
   naming the contract on every lowered stream's provenance. MCP `lookup`
   describes each pack type against its master chain (refines, master,
   effective fields, roles with the master's word, lines, side) and lists
   the masters the pack refines with the roster every refinement inherits;
   `skeleton` resolves an ontology type id to the pack's template and,
   asked for a master, names the pack's refinements of it.
5. **Cross-pack reading** — **built.** `type <Master>` and `line <role>`
   on slices and authored statement rows, expanded by the compiler to the
   exact streams they select (one expansion for both, so a row and a slice
   can never disagree); a metric folds a slice's net as `slice.<name>`;
   each lowered stream series carries its `line`; three benchmarks carry
   slices by master. Pack metrics were NOT migrated, deliberately: a
   pack's own metric reads its own streams, and the cross-pack reading is
   the model's and the consumer's — which the results now support without
   the pack.
5b. **Roster completion** — **built.** The survey (`docs/41`); Construction
   out and `Contract.Supply` for Offtake; then the four cores reworked from
   their governing documents and `Contract.Guarantee` added, all five in the
   base with allocated and optional lines and the `contract` field type;
   MCP `lookup` reads a master by name. Demonstrations follow as their own
   change (§4.13). Originally drafted as:
   `Contract.Security`, `Contract.Equity`, `Contract.Royalty` and
   `Contract.Grant` in the language base; the credit pack refines
   Security for its note classes and the auto ABS pilot declares its
   classes as securities whose steps read the contract's terms; a pack
   refines Equity where a case carries a partnership interest. Built
   before stage 6, whose balance role is defined against the finished
   roster.
6. **State owned by the agreement**: `balance_field` on `Contract.Debt`
   so a pack machine's `on enter retired` extinguishes it for every
   refinement. A security's and an equity interest's balance is the
   holder's account (§4.13, §4.14), derived rather than lowered, so
   neither needs the role.
7. **Elections**: `Contract.Option`'s core in the `option` grammar; base
   option names retired.

`docs/13` §7.58, §7.63, §7.67 and §7.92 closed with stage 3; §7.96–7.98
close with the stages that answer them.
