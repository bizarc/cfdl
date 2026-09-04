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
| `fields` | the terms the agreement states: name, type, required, unit | inherited; strengthen-only redeclaration; a refinement may add |
| `lines` | the economically distinct cash the agreement produces, by role name | a refinement's rules MUST emit every master line; may add |
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

Seventeen masters in the language base: the eleven counterparty masters,
`Contract.Line`, and its five specializations for the general lines
(§4.12, decision R1); and four more drafted on 4 September 2026 for review
(§4.13–4.16), which complete the roster against what the benchmarks model
by hand. For each: the roles, the fields (required unless marked opt,
with the default where one is a fact of the instrument rather than of a
deal), the lines, and the side. The argument is given where the choice
was not obvious.

### 4.1 `Contract.Debt`
Roles `lender`, `borrower`. Fields: the amount borrowed, stated one way
(`principal`, or a facility's `commitment`, or a `draw_curve` the facility
funds against — one required); the rate, stated one way (`interest_rate`
fixed, or `index_curve` with `margin` floating — one required); `day_count` (opt; the model's convention when absent);
`payment_frequency` (opt; the calendar's when absent); `amortization` (opt:
`level_pay`, `interest_only`, `bullet`, `custom`; a refinement fixes it — a
pool type is `level_pay` by definition); `amortization_months` (opt; the
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
`term`, not a second notional.

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

### 4.5 `Contract.Offtake`
Roles `seller`, `offtaker`. Fields: `price` (per unit, or per year where
the payment is for availability); `quantity` (opt — per year, in the pack's
unit; absent for a capacity payment, which pays for availability rather
than volume); `escalation` (opt, 0); `degradation` (opt, 0); `availability`
(opt, 1). Line: `revenue`. Side: open. A
merchant sale names only the seller — its offtaker is the market — and
the master allows a role to be unbound where the refinement says so.
Energy's `ppa_price`, `price` and `payment_year` are one field; its
`mwh_year` is `quantity`.

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

### 4.9 `Contract.Construction`
Roles `owner`, `contractor`. Fields: `budget`; `draw_curve` (a declared
curve, per `cre.construction_loan`'s argument that a draw schedule is data
and not a term); `retainage` (opt, 0). Line: `draw`. Side: owner pays. No refinement yet:
`cre.construction_stub` was expected to be the first, and the load check
settled what it is — a flat draw of an `amount` over a term, emitting no
interest and repaying nothing, which is a capital-expenditure line. It
refines `Contract.CapitalExpenditure`, and its `lender` role, which nothing
read, is gone: a refinement's roles are its master's roles, specialized,
and a party the agreement never pays or reads is not a role.

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

### 4.13 `Contract.Security` — drafted for review
Roles `issuer`, `holder`. Fields: `face` (the original principal — what
the holder is owed at issuance); the coupon, stated one way (`coupon`
fixed, or `index_curve` with `margin` floating — one required);
`day_count` (opt; the model's convention when absent);
`payment_frequency` (opt; the calendar's when absent). Line: `interest`.
Side: open — an ABS model is written from the issuer's seat, a bond
portfolio from the holder's; each refinement fixes it.

**Why it is not a debt.** A debt's cash follows from its own terms: a
rate, a balance and a schedule produce interest and principal. A
security's interest follows from its coupon on its outstanding claim,
but its PRINCIPAL follows from collateral through a priority of
payments: it is what the structure allocates to the holder, and the
holder's claim is `face` less what the holder's account has received —
the D7 shape the auto ABS pilot already uses. So the master's only line
is `interest`, which is the only cash a security produces by its own
terms. Its principal is a waterfall step paying the holder's account, and
the load check asks no rule to emit it. The same split as `Contract.Equity`
below, and the same the language already draws between a contract's
terms and a waterfall's priority.

**Why seniority is not a term.** An indenture agrees the priority, and a
model states it once, as the ORDER of the waterfall's steps. A `seniority`
number on the contract would say the same thing a second time, and could
not say what a waterfall can — sequential here, pro rata there, a
step-down after a trigger. What was agreed lives on the contract (face,
coupon); how it is paid lives on the waterfall (order, claims). A slice
by `type Contract.Security` reaches every class; a step's claim reads the
class's `face` and the holder's account.

**Balance.** Derived, never lowered: the claim over the holder's account.
Stage 6's balance role belongs to `Contract.Debt`, whose balance IS a
lowered field; a security needs no role, and retires when the account
reaches its face.

**First refinement.** The credit pack's note class, and the auto ABS
pilot declares its seven classes as securities whose steps read the
contract's `face` and `coupon` instead of `assume` values. A residual
certificate is not a security in this sense — it has no face and no
coupon, and takes what remains — and refines `Contract.Equity`.

### 4.14 `Contract.Equity` — drafted for review
Roles `issuer`, `holder`. Fields: `commitment` (the capital the holder
agreed to contribute); `share` (the holder's share of what is
distributed, as a ratio); `preferred_return` (opt; the annual rate the
holder's contributed capital accrues before any promote). Line:
`contribution`. Side: holder pays the contribution; distributions run the
other way by allocation.

**What the contract states and what the waterfall states — decided
4 September 2026.** The contract states what was agreed: the commitment,
the share, the preference rate. The waterfall states the priority:
return of capital, the preferred return, the promote, the split — steps
that read the contract's terms and the holder's ACCOUNT (D13:
contributions are streams into the deal's cash, each partner's account
holds what has been allocated to them, the accrued preference is a field
compounding on `prev.<account>`). Nothing on the master is a payment
rule, for the same reason nothing on `Contract.Security` is: an equity
interest's cash follows from what remains after a priority, and a
priority is a waterfall.

**Why `contribution` is the master's line.** It is the one cash an
equity agreement produces by its own terms — a commitment funded on a
schedule — and every interest has one. What the holder gets back is
allocated, so `distribution` is a step paying the holder's account, not
a line a rule emits.

**Balance.** The holder's account: contributed less distributed, the
position D13 already keeps. No lowered field, so no stage 6 role.

**Refinements expected.** A JV or LP interest (the Penzance cases, One
Lincoln's placeholder tiers) adds nothing to the core — the promote and
the tiers are waterfall steps. A tax-equity interest adds what a flip
needs: a pre-flip and a post-flip `share`, and the target yield the flip
tests, which the refinement's machine reads. An ABS residual certificate
adds nothing and has no `preferred_return`. A management or option pool
stays an `Option`: an election, not an interest.

### 4.15 `Contract.Royalty` — drafted for review
Roles `licensor`, `licensee`. Fields: `rate` (the share of the basis
paid, as a ratio); `basis` (the revenue the rate applies to — a series
the model publishes, named as a selector); `minimum` (opt, 0; a floor per
period). Line: `royalty`. Side: licensee pays.

**Why it is its own master.** A royalty is a claim on ANOTHER agreement's
revenue: nothing is sold (not an offtake), nothing is done (not a
service), and the amount is a rate on a basis the licensee's own
contracts produce. That is why its basis is a reference rather than a
quantity, and why the CREST solar case (`crest_solar_cost_based`) restates
its royalty as a hand stream reading the PPA's revenue: the pack had no
place to put a payment computed on a selector. Music and IP catalogues
are the same shape with a different basis.

### 4.16 `Contract.Grant` — drafted for review
Roles `grantor`, `recipient`. Fields: the support, stated one way
(`amount` per period, or `target` with `basis` — the level the grantor
tops the basis up to — one required). Line: `support`. Side: recipient
receives.

**Why it is neither a tax nor an offtake.** A tax attribute (`Contract.Tax`)
is a position against the recipient's own liability — a credit, a
shield — and lands there. An availability or capacity payment
(`Contract.Offtake`) buys something: the asset's availability. A grant
buys nothing and offsets no liability; it is support paid because a
public party agreed to pay it, either as a fixed amount or as a top-up to
a target. The PPIAF toll road's coverage subsidy — the authority pays the
shortfall to a target ADSCR — is the standing case, hand-rolled today
with the formula stated three times (backlog P4, P9).

**The four together.** Security and Equity are the financing side of
every structured deal in the corpus, and until now both were invisible
to the language — note classes as `assume` values and party accounts,
partnership interests as hand-rolled preference fields. Royalty and Grant
are the two agreements the bespoke and energy cases restate as streams
because no master gave a pack a place for them. All four are added
before their first refinement on the argument §4.9–4.11 already made: a
master that exists before its refinement costs nothing, and its absence
is what forces the hand-rolled stream.

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
role UNBOUND in a model (the merchant sale's offtaker) by saying so:
`[[contracts.roles]] name = "offtaker" unbound = true`.

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
across its rules covers its effective lines. The CATEGORY stays the
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
5b. **Roster completion** — the survey (`docs/41`) first, then the four
   cores reworked from their governing documents, then a fifth the survey
   found missing (`Contract.Guarantee`). **Drafted, for review** (§4.13–4.16):
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
