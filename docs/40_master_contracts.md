# Master contract types — the construct, stated top-down

Status: **stage 1 of seven built** — principles and the roster settled in
discussion 2 September 2026; R1–R3 decided the same day (§4.12, §7, §8).
Stage 1 (the ontology model in `cfdl-pack`: fields, roles with
specialization, lines, side, the effective walks, the load checks, the
roster below in the language base, and every pack's roles specialized)
is in. Stages 2–7 are listed at the end and each lands as its own change.
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
   a master is mined from pack usage. Where a pack's word differs, the
   pack conforms.
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

Seventeen masters: the eleven counterparty masters, `Contract.Line`, and
its five specializations for the general lines (§4.12, decision R1). For each: the roles, the fields
(required unless marked opt, with the default where one is a fact of the
instrument rather than of a deal), the lines, and the side. The argument
is given where the choice was not obvious.

### 4.1 `Contract.Debt`
Roles `lender`, `borrower`. Fields: `principal`; `interest_rate` (annual
nominal); `day_count` (opt; the model's convention when absent);
`payment_frequency` (opt; the calendar's when absent); `amortization` (opt:
`level_pay`, `interest_only`, `bullet`, `custom`; a refinement fixes it — a
pool type is `level_pay` by definition); `amortization_months` (opt; the
horizon the payment is struck on, which may exceed the term);
`interest_only_months` (opt, 0); `funded_at_close` (opt, 1);
`balloon_at_maturity` (opt, 0). Master names are full words: a master is
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
`free_rent_months` (opt, 0). Lines: `rent`, `abatement`. Side: lessee
pays. Recoveries, expense stops, TI/LC, rollover probability and
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
the income a cap rate is applied to, and the master is not CRE's.) Lines: `proceeds`, `selling_costs`. One-shot. Side: seller
receives. The "required as a group" form (`any_of`) is new to the field
model and is needed here and on the rent of §4.2.

### 4.5 `Contract.Offtake`
Roles `seller`, `offtaker`. Fields: `quantity` (per year, in the pack's
unit); `price` (per unit); `escalation` (opt, 0); `degradation` (opt, 0);
`availability` (opt, 1). Line: `revenue`. Side: seller receives. A
merchant sale names only the seller — its offtaker is the market — and
the master allows a role to be unbound where the refinement says so.
Energy's `ppa_price`, `price` and `payment_year` are one field; its
`mwh_year` is `quantity`.

### 4.6 `Contract.Service`
Roles `provider`, `recipient`. Fields: `fee` or `fee_year` (one required);
`escalation` (opt, 0). Line: `expense`. Side: recipient pays. O&M is
the first refinement; a management agreement, a servicing agreement and
an administration agreement are the next, and the auto ABS trust's
servicing and administration fees (`benchmarks/credit/auto_abs_tranches`)
are the case that will want them.

### 4.7 `Contract.Tax`
Roles `taxpayer`, `authority`. Fields: `tax_rate` or `amount` (one
required); `basis` (opt). Lines: `paid` or `benefit`, fixed by the
refinement. Side: taxpayer pays, or receives a benefit. The weakest
master to state a core for — a cash tax, an investment credit, a
production credit and a depreciation shield share only "an obligation or
attribute against a revenue authority, with a period" — and that is what
the core says. Refinements carry the rest.

### 4.8 `Contract.Option`
Roles `grantor`, `holder`. Fields: `strike` (opt); the election and the
payoff come from the `option` grammar (`exercise when`, `payoff`) and are
declared there, not in `terms`. Line: `payoff`. An election binds no
lowering rule; stage 7 aligns the grammar with the master and retires the
base names (`Option.Call`, `Option.Refinance`) that resolve against
nothing today (`docs/13` §7.67).

### 4.9 `Contract.Construction`
Roles `owner`, `contractor`. Fields: `budget`; `draw_curve` (a declared
curve, per `cre.construction_loan`'s argument that a draw schedule is data
and not a term); `retainage` (opt, 0). Line: `draw`. Side: owner pays. No refinement yet:
`cre.construction_stub` was expected to be the first, and is not — its
roles are `owner` and `lender`, it lowers a draw a lender funds, and that
is a debt facility, so it refines `Contract.Debt` (`owner` refines
`borrower`).

### 4.10 `Contract.Derivative`
Roles `party`, `counterparty`. Fields: `notional`; `reference` (a
declared curve or quantile); one of `fixed_rate` or `strike`. Line:
`settlement`. Side: open. No refinement yet; declared because a rate swap
on a floating loan is the first thing a project-finance case will ask for.

### 4.11 `Contract.Insurance`
Roles `insurer`, `insured`. Fields: `premium`; `coverage` (opt);
`deductible` (opt). Lines: `premium`, `claim` (opt). Side: insured pays.
HUD's mortgage insurance premium is the standing case
(`benchmarks/cre/hud_home_multifamily`), which today is a hand stream
because the debt contract rightly refused to carry it.

### 4.12 `Contract.Line` and its kinds — decision R1
Role `owner` (the model's own party; no counterparty). Fields: `amount`
or `amount_year` (one required); `growth_rate` (opt, 0). `Contract.Line`
is the pure master and is itself abstract; it is specialized, still
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
selector can then ask for a line by role across packs — `interest` of
`type Contract.Debt` — without knowing any pack's category spelling
(stage 5).

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
| `tax_rate` (on tax) | `tax_rate` | already the master's word |
| `ppa_price`, `price`, `payment_year` | `price` | energy |
| `mwh_year` | `quantity` | energy |
| `price_escalation` | `escalation` | energy |
| `om_year` | `fee_year` | energy |
| `base_rent_year`, `rent_year` | `rent_year` | cre |
| `base_rent` | `rent` | cre |
| `noi_value` | `income` | cre |

*How — decided 2 September 2026:* a HARD rename, in one change per pack
or one for all. The language is pre-release with no downstream consumers,
so there is no deprecated alias and no warning code; every benchmark,
fixture, template, learn chapter and site example is migrated in the same
change, and an old name is simply an unknown term. The master's names were
reviewed for correctness first (full words, no pack abbreviations, no
domain jargon — §4.1, §4.4, §4.7), because a hard rename is done once.

Bounds that every refinement of a master shares (`interest_rate ≥ 0`,
`amortization_months > 0`, `principal > 0`) move to the master field and
are checked once; a pack's `validations.toml` keeps the bounds that are
its own.

## 8. What the compiler checks — decision R3

A contract's type is resolved ONCE, where its lowering rule is matched,
and carried: `type_id`, `instance`, and the master chain travel on the
contract through the IR and into the results graph. The prefix match at
every consumer (`docs/13` §7.58) goes. Against the effective roster the
compiler refuses an unknown term with a near-miss hint
(`E1371_UNKNOWN_CONTRACT_TERM`), a missing required field
(`E1372_MISSING_CONTRACT_TERM`), an unknown or abstract type named on an
`option` (`E1373_UNKNOWN_CONTRACT_TYPE`,
`E1374_ABSTRACT_TYPE_INSTANTIATED`), and a role outside the effective
roles (`E1322`, now naming the master role too). A model's `parties`
block is serialized, closing the "parsed and discarded" status.

*Scope — decided 2 September 2026:* the two-token declaration the grammar
already allows — `contract cre.lease_unit tenant_a` (`docs/13` §7.63) —
lands in the same stage as the type carry rather than as a follow-on:
leaving it open would carry the string-surgery it exists to remove into
every consumer the stage touches. The fused `cre.lease_unit.tenant_a`
spelling keeps working; models migrate to the two-token form as they are
next touched.

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
2. **Pack conformance**: role specializations, `line` on every rule,
   term renames with aliases (§7), line types refine §4.12, bounds moved
   to masters; every benchmark byte-identical.
3. **Compiler**: resolve-once type carry, the four diagnostics of §8,
   effective-roster term and role checks, parties in the IR.
4. **Results and tools**: contracts as graph nodes with master and
   roles; MCP `lookup`/`skeleton` see masters.
5. **Cross-pack reading**: `type <Master>` on metrics and statement rows;
   line-by-role selectors; pack metrics migrate where equivalent.
6. **State owned by the agreement**: `balance_field` on `Contract.Debt`
   so a pack machine's `on enter retired` extinguishes it for every
   refinement.
7. **Elections**: `Contract.Option`'s core in the `option` grammar; base
   option names retired.

`docs/13` §7.92 is closed when stage 3 lands; §7.58, §7.67 and §7.96–7.98
close with the stages that answer them.
