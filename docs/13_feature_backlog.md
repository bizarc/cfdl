# CFDL — Feature Backlog

Status: informative. Things worth building that are **not** defects.

Bugs do not belong here; they get fixed or they get a failing test. What
belongs here is capability the language or a pack does not yet have, where the
gap was found by trying to model something real and hitting a wall. Each entry
therefore says what could not be expressed, and what forced the discovery —
a backlog item with no provenance is a guess.

Ordered within each section by how much it unblocks, not by effort.

---

**Closed items are removed, not archived.** A capability that ships is
described in the language documentation; reasoning that turned out to be wrong,
and shapes the language already supports, are recorded in
`docs/26_lessons_learned.md`. This file holds work to do.

## 1. CRE pack

## 2. Credit pack

### 2.2 A pool that amortizes on an Actual basis

*Rewritten. The original entry diagnosed this as a pool-factor limit and
proposed a validation gate for pools. Both were wrong, and the measurement
that settled it is below.*

**What was claimed:** `amortization_day_count = "act/360"` holds a constant
payment on a single loan and is only an approximation on a pool, because the
pool factor `S(p)` is a closed form built from one periodic rate.

**What is true:** it does not hold a constant payment on a single loan either.
Measured on one 1,200,000 loan at 6% with `cpr = cdr = 0` — no pool, no
prepayment, no defaults — the payment swung **460.68** over twelve months:
7,349.63 in a 31-day month, 6,888.95 in February. The cause is not pooling. An
Actual basis expands to `(360 / time.days_in_period)`, a period-local value,
and the annuity `pmt(rate / divisor, n - p, 1)` applies it to all `n - p`
remaining periods — January strikes a payment as if every remaining month had
31 days. It is the same failure already measured on the ACCRUAL divisor
(697k-754k, `benchmarks/credit/mbs_pool_conventions`); splitting the two
divisors fixed that spelling and left this one, and the shipped fixture pairs
`act/360` accrual with `30/360` amortization, so the broken combination was
never exercised.

**Shipped:** `E5027_ACTUAL_AMORTIZATION_BASIS` refuses an Actual
`amortization_day_count` outright, for every pack and every instrument rather
than for pools. The pairing a loan document actually states still compiles and
is pinned: strike the payment on `30/360`, accrue interest on `act/360`, and
the payment holds at 7,194.61 while interest moves 6,200.00 to 5,594.43 with
month length.

**What remains, if anything.** An instrument whose payment genuinely does
recompute each period — not a commercial Actual/360 loan, which does not — is
a BALANCE RECURRENCE, not a closed form. Nothing in the language blocks it: a
field's `next` sees `time.days_in_period` and reads no series, and `docs/07`
uses `field.loan_balance` as its own worked example. Open this only when a
real instrument needs it, with the document that says so.

**And the pool-factor concern was misplaced.** A pool that the closed form
cannot hold does not need a gate — it needs its components.
`benchmarks/credit/mbs_pool_by_loan` declares one 100m pool as four loans of
40/30/20/10, each `part of` the pool with its own contract, and ties to the
single-pool model at **0.0** across all 372 periods, through two aggregations
that share no code. Heterogeneity of any kind is already exact that way, which
is the answer `docs/18` gives for the 43-sub-pool auto ABS case as well.

## 3. OpCo pack

## 4. Energy pack

## 5. Language and engine

## 6. Cross-pack

## Where these came from

Almost every item in sections 1 to 6 was found by reconciling a benchmark
against an external reference, and the sections are named for the pack the
reconciliation exercised. Individual item numbers are not cited here: closed
items are removed from this file, so a citation would dangle. The sources are
what matter, and they are recorded in each benchmark's `NOTES.md`.

The CRE items came from `benchmarks/cre/hud_home_multifamily` against HUD's own
populated underwriting Sample — the only source in the programme that may be
redistributed, and so the only one whose reference workbook is committed beside
the model — and from `benchmarks/cre/mit_rentleg_plaza` against MIT
OpenCourseWare 11.431J Problem Set 1, the first CFDL benchmark checked against a
published third-party figure rather than an in-house reference.

The credit items came the same way, from `benchmarks/credit/mbs_pool_conventions`
against the published industry reference for MBS cash flows — which also found
three outright defects, in the prepayment base, the recovery basis and the
payment-striking divisor, all fixed rather than listed here.

The opco items came from `benchmarks/opco/banker_dcf_conventions` against a
disclosed valuation in a public merger filing — the opco pack's first external
check. All nine cells of the banker's answer grid reproduce within $1.2mm on
$19bn. It also found two outright engine defects, fixed rather than listed
here: mid-period discounting had no spelling at all, and `on day <n>` divided
by a literal 30 on every calendar.

Section 4 came from `benchmarks/energy/utility_pv_singleowner` against a
national laboratory's open-source project-finance model — the energy pack's
first external check of any kind. Five rules reproduced it to within 1e-6
dollars on the first attempt; the two items above are what the reconciliation
found *around* the agreement.

**Section 7 is below this one, not above it.** New items are appended to the end
of the file rather than filed into the section they belong to, because the
numbers are positional and inserting one renumbers everything below it. Each
appended item names its home section.

That is the argument for building more of them: an external number finds gaps
that two of your own implementations agreeing never will. See
`research/CFDL_pack_roadmap_and_model_sourcing.md` for the catalogue.

---

## 7. Appended after the section numbering settled

New items go here rather than into the section they belong to. Backlog numbers
are positional, so inserting one renumbers everything below it and silently
breaks every `NOTES.md` reference and commit-message citation pointing past the
insertion — which has already happened once. Appending never renumbers. Each
item says which section it belongs with.

### 7.1 Storage revenue is a reduced form with an unquantified error

*Rewritten. The entry proposed a price-duration curve input and said it "needs
no new engine capability". That was wrong on both counts: a duration curve is
not a date-indexed curve, and the defect is not confined to the energy pack.
Both corrections are below, and the design they led to is `docs/27_quantiles.md`.*

*Belongs with the language and engine (section 5). It was filed against the
energy pack and is not an energy item.*

`energy.storage_arbitrage` is `mwh_cycled_year * spread * (1 - degradation)^y`.
The industry reference models a battery with a **dispatch optimiser** over an
hourly price series, so its revenue emerges from thousands of hourly
charge/discharge decisions. The two do not reduce to one another and no choice
of inputs makes them agree — fitting `spread` until they matched would be
calibration, not validation.

So this rule has **no external validation**, and
`benchmarks/energy/merchant_capacity` says so rather than quietly including it.
Energy is at 9 of 10 rules.

**Attempted, and this is what it showed.** A dispatch run was tried rather than
assumed: SAM's `Battwatts` model, 20 MW / 80 MWh behind a 100 MW PV plant,
diurnal generation and an evening-peak load.

- Behind the meter, it discharged **27.9 MWh across a whole year** from an
  80 MWh battery, and charged nothing. Not a bug — dispatch is driven entirely
  by the load and price context, and with 100 MW of PV against a 55 MW peak
  there was nothing for the battery to do.
- Reconfigured front-of-meter for merchant arbitrage, the native library
  **segfaulted** (exit 139).

The first result is the important one, and it sharpens the item: `quantity` (the storage rule's MWh cycled)
is an **input** to our rule and the primary **output** of a dispatch model. The
quantity we ask the modeller to state is the thing the reference exists to
compute. So the two cannot be compared without first deciding the answer — which
is why "fit `spread` until they agree" is calibration and not validation.

A real comparison needs the full `Battery` module with a price-signal dispatch
choice and a generation chain, not `Battwatts`. Worth doing, but it is a
scoping exercise of its own rather than a benchmark.

**What the error actually is.** Storage revenue is a DISPERSION FUNCTIONAL: it
depends on the spread of the price distribution, not its level, because a
battery discharges only into the upper tail and charges only from the lower one.
A mean-preserving spread strictly increases it. `spread` is a hand-supplied
stand-in for that dispersion, which is the precise reason fitting it is
calibration — you are fitting the answer.

That decomposes the unquantified error into two parts, and only one of them
needs a dispatch model:

- a **Jensen gap**, from evaluating a dispersion functional at a point
  statistic; and
- a **chronology error**, because a four-hour battery cannot reach eight
  non-contiguous peak hours without recharging.

**The same defect is already shipped in the CRE pack.** `cre.percentage_rent`
lowers to `max(0, sales - breakpoint) * pct` — a call option on tenant sales
evaluated at a point estimate of sales. Below the breakpoint it returns exactly
zero when the true expectation is positive. `benchmarks/cre/retail_strip`
exercises it. Two packs, one shape, so the fix is a language primitive rather
than an energy contract.

**Correction: a duration curve is not expressible today.** The previous entry
said CFDL's `curve` declarations already cover this. They do not. A `curve` is
indexed by DATE at every layer — the grammar's `curve_point = DATE ":" NUMBER`,
the IR schema's required `points[].date`, and `CurveDef`'s `Vec<(Date, f64)>` —
and is consumed by point lookup. A duration curve is indexed by cumulative
share and is consumed by INTEGRATION, and no expression function integrates
anything. What is expressible today is a price curve varying by year, which is
a different object on a different axis and would not close this item.

**And it breaks the circularity the paragraph above identifies.** Unlike
`quantity` (the storage rule's MWh cycled), a price duration curve is a summary of the hourly price
series the dispatch model ALSO takes as input — so it is an input on both sides.
Cycled energy then becomes an output of our rule, derived from power rating,
duration and efficiency, and comparable to the reference's output. That is
validation rather than calibration. It is the reason to build this, and it does
not depend on the reduced form being replaced.

The reduced form is not wrong; practitioners use exactly this shape at the
financing stage. Ways forward, in order of cost:

- **A `quantile` declaration and three functions.** Designed in
  `docs/27_quantiles.md`: a value indexed by cumulative share, with
  `quantile_at`, `quantile_mean` and `quantile_of`. Language surface — spec,
  grammar, IR schema, two IR structs and a `CurveDef` sibling — so roughly a
  week, not free. It closes the Jensen gap in both packs. Cheapest first proof
  is `cre.percentage_rent` against `benchmarks/cre/retail_strip`, which needs no
  new reference model.
- **`energy.storage_dispatch`** (7.5), the contract that consumes it.
- **State of charge**, which needs per-period persistent state (5.2) and would
  let cycling be modeled rather than assumed.

Note what none of these close. The chronology error still needs the dispatch
comparison, so this item stays open after the primitive lands and energy stays
at 9 of 10 rules in 7.3 until that measurement exists.

True hourly dispatch optimization is out of scope and should stay there — that
is an optimizer, not a declarative cash-flow model.

### 7.3 Pack contract coverage across the benchmark suite

*Belongs with no single pack — it is about the validation programme.*

**Re-measured 2026-08-30** across all 44 registered cases (2 bespoke, 9 cre,
18 credit, 6 energy, 9 opco), counting a pack contract type as *exercised*
when at least one case declares it. When first measured (six cases, headline
"the external cases route around the packs they should be validating") the
counts were energy 9/10, credit 1/4, cre 1/12, opco 0/10 — for cre and opco
the benchmarks bypassed the pack entirely, so they validated the engine, not
the domain logic. That circularity is now broken:

| pack | exercised | not exercised |
|---|---|---|
| energy | **10 / 10** (see caveat) | — |
| credit | **4 / 4** | — |
| cre | 11 / 14 | `lease`, `percentage_rent_expected`, `construction_stub` |
| opco | **11 / 11** | — |

(The cre and opco rosters have grown since the first measure — 12→14 and
10→11 — so the denominators moved too.)

What closed the gaps: `office_two_tenant` exercises the acquisition spine
through the pack (`lease_unit`, `rollover`, `vacancy_loss`, `opex_line`,
`permanent_debt`, `exit_forward`), `retail_strip` adds `percentage_rent` and
`exit`, `one_lincoln_street_contract` proves `construction_loan` against the
native twin, `float_bridge_pool` and `io_bullet_loan` close credit, and
`lbo_buyout` plus `damodaran_fcff` take opco from zero to nine — the
driver-disclosing sources the first measure asked for. `dcf_exit_multiple_nwc`
closes the remaining two, against a template that states an increase in net
working capital line by line and strikes its terminal value on an LTM EBITDA
multiple — the two figures Damodaran's engine cannot supply, because it folds
working capital into reinvestment through a sales-to-capital ratio and takes
its terminal value by Gordon growth.

**Exercised is not the same as validated.** One caveat stands: `storage_arbitrage`
is declared by `solar_ppa_microgrid`, but that case reconciles the reduced-form
arbitrage margin against convention, not against a dispatch model — the
chronology comparison the storage entry (§7.5's duration-curve discussion)
requires does not exist, so energy's *validated* count stays **9 / 10** until it
does. Read strictly, cases whose references are independently recreated
conventions (`office_two_tenant`, `retail_strip`, `solar_ppa_microgrid`) sit a
step below a published third-party model; each CASE.md states which kind it is.

**Two axes, not one.** A concept can be expressible in the core language, in a
pack contract, or both — and a case on native streams is a choice, not a
coverage failure. `one_lincoln_street` exists in both spellings, and the pair
is the assertion. `tax_equity_flip` uses no streams and no contracts at all —
declared state and an event, with the model's own comments arguing why core is
the right spelling. The penzance developments, `banker_dcf_conventions` and
`saas_sbc_convention_fork` model natively for the same reason. A core-spelled
case proves the LANGUAGE expresses the deal with no domain vocabulary — the
stronger claim; the pack contract is the ergonomics layer, and this entry
measures whether that layer is exercised, not whether it is mandatory.

What remains, and it is now narrow:

- **cre:** three types unexercised. `basic_acquisition_exit_cap` closed
  `revenue_line` and `exit_cap` together, off a stabilized property whose
  income is stated at the property level and whose disposition is a stated NOI
  over a stated cap rate. `lease` (non-unit grain), `percentage_rent_expected`
  and `construction_stub` may want a new case each.
- **energy:** the dispatch comparison that would move `storage_arbitrage` from
  exercised to validated.

Recorded because coverage claims must cite this table, and the table must be
re-measured — by scanning `contract <pack>.<type>` declarations, not `<pack>.`
prefixes, which also match namespaced stream names — whenever cases or rosters
change.

### 7.4 A discount rate cannot vary over time

*Belongs with the language and engine (section 5).*

`RunConfig.discount_rate` is a single `f64`, turned into one `per_period_rate`
and handed to `npv_with_offsets`. Every discounted figure in a model uses it.

Intrinsic valuation converges the cost of capital as a firm matures —
Damodaran's model runs 7.055% for five years and 8.81% thereafter. Project
finance uses one rate through construction and another in operation. Neither is
exotic and neither is expressible.

The consequence is not cosmetic: `benchmarks/opco/damodaran_fcff` asserts the
entire cash-flow build and **no discounted figure at all** — not NPV, not
enterprise value, not the per-share price the source exists to produce.
Discounting at a flat rate would have produced a number and not a check.

Shape: a discount *curve* alongside the scalar, read per period. Note the offset
machinery in `npv_with_offsets` already handles per-STREAM variation; this is
per-PERIOD variation, a different axis, and it touches IRR too — `irr_with_offsets`
solves for a single rate by construction.

### 7.5 Candidate contracts, and the packs that need them

*Belongs with the CRE and OpCo packs (sections 1 and 3).*

Every entry below was forced by a source, not proposed from taste. Listed
together because the shape of the gap is the same in both packs: the contracts
that exist model an operating business well and stop at the point where a deal
gets financed or valued.

**CRE — the pack cannot borrow money.**

| candidate | forced by |
|---|---|
| ~~`cre.permanent_debt`~~ | **SHIPPED**, then decomposed per docs/07 §6.4: proceeds, interest and principal as their own streams, balloon opt-in, `funded_at_close` for post-financing reconciliations. DSCR-based sizing is a solve and stays out. |
| ~~`cre.construction_loan`~~ | **SHIPPED.** Equity-first funding behind a commitment, the facility taking the balance once it depletes, interest on the drawn balance. The draw schedule stays a model `curve` and the contract names it, because a funding profile is per-deal data rather than a term. `benchmarks/cre/one_lincoln_street_contract` reproduces the primitive-built case in all 48 cells with zero difference — the pair is the assertion, and if they disagree the contract is wrong. Capitalised interest is a follow-on: affine in the closing balance, so it collects rather than needing a solver. |
| `cre.restricted_rent` | HUD — rent capped for an affordability period and reverting to a market track. The defining mechanic of affordable housing, currently a hand-written conditional. |
| `cre.abatement` | MIT — free rent as its own deduction from potential gross revenue. Today it can be reported as a line or counted in NOI, not both (1.3). |
| `cre.replacement_reserve` | HUD — a capital reserve, separately published and semantically distinct from operating expense. Also One Lincoln Street, whose operating pro forma carries a Capital Reserve line. |

With 1.5, 1.6 and 1.7, these are what would let a real CRE deal be expressed in
pack contracts instead of native streams — which is the actual fix for 7.3 on
the CRE side, and needs no new source.

**A correction to how 7.3 originally framed this** (absorbed into its
2026-08-27 re-measure, kept here for the argument). That entry treated a benchmark running
on native streams as a coverage failure. It is not, or not only. A case built
from primitives proves the LANGUAGE expresses the deal with no domain vocabulary
— which is the stronger claim, and the one a reader evaluating CFDL as a
language can check. A pack contract is an ergonomics layer for a practitioner
who should not have to derive an equity-first waterfall from scratch.

So the fix is not to CONVERT those cases. It is to add a contract twin beside
each, asserted against the primitive-built original rather than only against the
source: `one_lincoln_street` and `one_lincoln_street_contract` are the first
pair. A contract validated solely against its own source is the pack marking its
own homework.

**OpCo — no terminal value a valuation practitioner would recognise.**

| candidate | forced by |
|---|---|
| ~~`opco.exit_perpetuity`~~ | **SHIPPED**, and validated against a published nine-point growth sensitivity grid (`benchmarks/opco/gordon_growth_coned`). `discount_rate` is a contract term, which is faithful to the sources rather than a workaround: a terminal cost of capital is not the near-term one. A stream-derived variant is the follow-on. |
| `opco.exit_forward_multiple` | The banker DCF — a forward (NTM) multiple struck at a point before model end. |
| `opco.depreciation` | No D&A contract exists, yet `opco_cash_taxes` consumes `da_monthly` as a bare term with no rule producing it. |
| `opco.equity_bridge` | Both opco sources — debt, cash, minority interests and non-operating assets between enterprise and equity value. Done outside the model today. |
| `opco.share_count` | Both — a share count that dilutes over time, so per-share value is expressible at all. |
| `opco.revolver`, `opco.cash_sweep`, `opco.nol_carryforward` | Every LBO source. All three need per-period state (5.2) and should be designed with it rather than before it. |

**Elsewhere.** `energy.storage_dispatch`, a storage rule priced against a
declared price distribution rather than a scalar spread (7.1). It consumes the
`quantile` primitive designed in `docs/27_quantiles.md` and cannot be built
before it — a `curve` is indexed by date and cannot express the integral.
Credit's three uncovered contract types need a source, not a new contract.

### 7.9 `opco.capex_line` cannot express a derived line

Found closing 5.1 against `benchmarks/opco/damodaran_fcff`, and worth separating
because the old drift table made it look like the same defect it is not.

Reinvestment is **derived** from another line: `revenue(t) * g(t+1) /
sales_to_capital`. It funds *next* year's growth, so its own growth factor is
`(1 + g_t) * g_{t+1} / g_t`, which leads the revenue growth path by one year:

| | yr 5 | yr 6 | yr 7 | yr 8 | yr 9 | yr 10 |
|---|---|---|---|---|---|---|
| reinvestment grows | 3.24% | 3.12% | 3.01% | 2.89% | 2.78% | 4.58% |
| revenue grows | 5.00% | 4.92% | 4.83% | 4.75% | 4.66% | 4.58% |

`opco.capex_line` is a self-growing line — a base times a rate path — so it
cannot express a quantity defined by another line's growth. No recurrence fixes
this; it is a contract shape gap. The benchmark therefore asserts reinvestment
for years 1–4 only, which is honest rather than fitted: deriving a
reinvestment-ratio curve by hand would pass and would hide the gap, exactly as a
cumulative-index curve would have hidden 5.1.

Shape: a reinvestment contract taking a revenue reference, a growth curve and a
sales-to-capital ratio, reading revenue through `series_sum` as
`opco.working_capital_policy` already does. That also makes the FCFF identity
(`EBIT(1-t) − reinvestment`) expressible from drivers rather than from a
hand-computed base.

### 7.13 District energy has no usable reference model

Scoped, not built. `research/CFDL_pack_roadmap_and_model_catalogue.xlsx` ranks
District Energy / Waste-to-Energy as a Tier 1 pack candidate with the gate
"None new — Energy pack extension (~65% reuse)", and names the Ed Bodmer project
finance collection as the first reference to build against.

That collection does not contain one. Measured across its thermal and
biomass/biogas pages:

| term | thermal page | biomass page |
|---|---|---|
| "district" | 0 | 0 |
| "cogeneration" | 0 | 0 |
| "combined heat" | 0 | 0 |
| "waste" | 0 | 3 (prose) |

The four downloadable thermal models are gas-fired IPPs (`IPP-Model.xlsm`,
`Gas-Plant-Example`, `Indonesia-Gas-Plant`, `NGCC-with-Merchant`). Their
structure — PPA and merchant revenue, O&M, senior debt, tax depreciation — is
what `benchmarks/energy/utility_pv_singleowner` and
`benchmarks/energy/merchant_capacity` already reconcile against a national
laboratory model, so they would add a second source for mechanics already
covered rather than the new ones the candidate needs (thermal load, fuel cost,
heat offtake).

They also lean on two things that are not expressible: debt sized to a target
coverage ratio ("sculpt" appears 41 times on the thermal page) and capitalised
construction interest resolved circularly ("circular", 28 times). Both are
solves. `docs/14_state_and_recurrence.md` §5 covers why an iterative construct
would need to be explicit, bounded and convergence-checked rather than implied.

What a district energy case actually needs is a source publishing a thermal
plant's drivers and the lines they produce — heat and power sold separately,
fuel cost as a driver, and a heat offtake contract. The catalogue's remaining
Tier 1 entries with no new gate are Telecom Towers (#9, A.CRE single-tenant NNN)
and Hospitality (#3/#20, A.CRE or Finamodel), both of which require an email
registration to download.

### 7.19 The lexer reserves words the canonical grammar admits

*Belongs with language and packs (section 5).*

The formal grammar allows any identifier where a name is expected. The lexer
does not, and the difference is 95 words.

```ebnf
entity_stmt     = "entity" IDENT IDENT ":" qname entity_block ;
entity_block    = "{" { kv_stmt } "}" ;
kv_stmt         = IDENT literal_or_expr ;
entity_field    = IDENT ( "=" literal | "init" expr [ "next" expr ] ) ;
```

`IDENT` is `[A-Za-z_][A-Za-z0-9_]*` (`docs/02` §1). No production excludes a
reserved word, and the grammar file states the case outright a few lines below
`entity_field`: **"`use = "office"` states a fact and holds."** That is the
canonical example of a field, and it does not compile.

```
entity asset tower : Asset.Real { use = "office" }
  ERROR[E0004_EXPECTED_TOKEN] Expected a field, 'part of', 'state' or '}' in entity block.

entity asset tower               { use = "office" }        (untyped)
  ERROR[E0004_EXPECTED_TOKEN] Expected token 'pack', found <punctuation>.
```

The second message is the mechanism showing through: `use` becomes a keyword
token, the parser abandons the block and starts looking for a `use pack`
statement, and reports a construct the author never wrote.

**Where it happens.** `crates/cfdl-lexer/src/lib.rs` converts a bare word to
`TokenKind::Keyword` before any position is known, so the exclusion applies
everywhere an identifier is expected. A word whose only grammatical role is
inside one clause blocks a name at the top of a file: `net` exists only for
`payment net 30`, and it is refused as an entity name, a field name and an
assumption name.

The one survival is accidental. `is_qname` is tested BEFORE the keyword table,
so a dotted name lexes as a single token and never meets it. `stream a.net`
compiles and `net = 1` does not — the same word, legal after a dot, illegal
alone. That is not a rule anyone chose; it falls out of the order of two
branches.

**Three measurements.**

*The set is larger than the specification says.* §18 documents 57 reserved
words; the lexer reserves 95. The 38 undocumented ones include `year`, `month`,
`net`, `state`, `active`, `in`, `none`, `mid`, `due` and `clip` — ordinary
words a financial model wants for a field.

*Fourteen are consumed by no production at all* — the seven weekday names, the
four stub policies, `direction`, `owner` and `tags`. They appear in the parser
only in `keyword_text`, which renders a keyword back to text for error
messages. Eleven belong to features `docs/10` records as REJECTED. `owner` and
`direction` are vestigial: direction is written as the bare words `inflow` and
`outflow`, and `owner` appears in no rule.

*Nothing enforces it where the names originate.* A pack's type registry
declares CFDL identifiers through a TOML door that never lexes them, so
`Credit.Asset.Loan.term` and `CRE.Asset.Unit.use` load clean and no model can
spell either. Neither is referenced by any rule, which is why the packs work.

**A second route exists and is worse than none.** An event may write such a
field — `set entity asset.tower.use = "retail"` compiles and publishes the
transition — but reading it fails `E1131_UNKNOWN_FIELD_READ`, because a read
requires a declaration and declaring is what the lexer blocks. The field is
write-only: it reaches the transition log, produces no series, and no
expression can reach it. `check_field_paths` builds its set of known names from
declared fields plus `status` and `state`, and never inspects an event's `set`
targets.

**Shape.** Accept a keyword token where the grammar admits `IDENT`, in the
naming positions: `kv_stmt`/`entity_field`, `assume_stmt`, `entity_stmt`,
`phase_stmt`, `curve_stmt`, `set_entity_stmt`. One token of lookahead
disambiguates the two clauses that share the entity block, because a field is
always followed by `=` or `init` while `state <name>` and `part of <ref>` are
not. Implemented that way the change is ADDITIVE: every spelling it accepts is
a compile error today, so no model that compiles now can change meaning, and
the goldens should be byte-identical. Implemented without the lookahead it
breaks `state` and `part of`, which is the one way to get it wrong.

Out of scope here: bare keywords in EXPRESSION position, which the expression
grammar governs separately; and the two cleanups this audit turned up — the 14
words reserved for nothing, and §18's 38 missing entries.

**The canonical grammar is behind the implementation in the other direction
too.** `entity_block = "{" { kv_stmt } "}"` omits `part of` and the `state`
clause, both of which the parser supports and the grammar's own comment
describes. Whichever way this is settled, the EBNF needs to state what an
entity block actually contains.

Found building `benchmarks/credit/mbs_pool_by_loan`, the first case to declare
typed fields on loan-level assets, and rewritten August 2026 after reading the
canonical grammar. The original item read this as a pack problem with two
possible repairs — widen the grammar, or refuse the collision at pack load. It
is neither: the grammar already admits the name, and refusing it at pack load
would constrain a pack's vocabulary by the core's keyword set, which is the
opposite of what `docs/07` §6.2 says a pack is for.

### 7.22 A published weighted average life cannot be asserted

Belongs with section 4 (credit pack).

`domain.credit.wal_years` folds the pool's own streams —
`credit.pool.sched_principal.*`, `prepay.*`, `bullet.*`, `recoveries.*` — so it
answers "when does the collateral come back". A structured deal publishes the
question one level up: when does *each class* come back. There is no metric for
that, and a waterfall step's stream cannot be reached by one, because
`metrics.toml` names streams by pattern and a WAL needs the class's original
balance as well as its payments.

So a published weighted average life cannot be checked. Ginnie Mae REMIC Trust
2026-100 publishes 709 of them, one per class per prepayment speed; Fannie Mae
REMIC Trust 2019-2 publishes seven. In both cases the model reproduces them —
709 of 709 exactly for the first, all seven for the second — and in both cases
the only place to say so was `CASE.md`, in prose.

This is the pool-factor problem one level up. That defect was a pool's amortisation state not
being exposed, which left `auto_abs_speed_050` reconciling its percent-outstanding
column in words; the fix was a cumulative subtotal, and the case now asserts it.
The same argument applies here: a figure the issuer publishes, that the model
gets right, that no gate would notice going wrong.

Shape: a per-class WAL wants two inputs the pack does not currently pair — a
payment stream and the original balance of the thing being paid. The class
already carries the second as `original_balance` on a `Credit.Asset.Tranche`, so
the metric is plausibly a fold over a stream *keyed to an entity*, rather than
over a stream pattern alone. That is a wider change than a new metric row, which
is why this is a backlog item and not a patch.

Found modelling Ginnie Mae 2026-100 and Fannie Mae 2019-2, where between them
716 published figures could be reproduced and none could be asserted.

### 7.23 A scenario asserts metrics, but not the per-period column that is the published artefact

Belongs with section 5 (harness and tooling).

`expected_scenarios.json` checks a scenario's **metrics**. `expected.csv` checks
per-period series, but only for the deterministic run. So a case can vary an
input across scenarios and assert what that does to a summary number, and cannot
assert what it does to a schedule.

For structured credit that is backwards. The published artefact *is* the
per-period column, tabulated at several prepayment speeds — five to seven of
them per class — and the summary number is the derived thing. Ginnie Mae
2026-100 publishes 58 such tables; the model reconciles every one, and a case can
assert one speed.

The existing route is one case directory per speed, as `auto_abs_speed_050` and
`auto_abs_speed_150` do. That works for two. It does not work for roughly 75
directories differing in a single term, and the duplication is not free: each
carries its own `CASE.md`, `SOURCE.md` and tolerances to keep in step, and the
site publishes each as a separate page.

Shape: let `expected.csv` carry a scenario column, or let a scenario name a CSV
of its own. Either makes the speed grid one case with N columns instead of N
cases with one, and neither changes what the engine computes — the scenario runs
already happen and their series are already produced, they are simply not
reachable from the harness.

Found modelling Ginnie Mae 2026-100, whose decrement tables publish 21,570 cells
across five to seven speeds per class, of which a single case can assert the
0%-, 100%- or 259%-PSA column but not all three.

### 7.26 Time-weighted metrics measure from the model start, on period fractions; a published WAL measures from settlement to stated payment days

Belongs with section 5 (language and engine).

`docs/12_payment_timing.md` names both limits itself: the axis origin is the
model start, not a settlement date, and precision is period fractions, not
calendar days. A prospectus weighted average life is defined on neither. The
FNMA REMIC prospectus (1 November 2018) computes it as principal reductions
weighted by "the number of years from the settlement date ... to the second
such distribution date", and the 2019-2 supplement's Pricing Assumptions fix
both anchors for its tables: settlement 30 January 2019, "each Distribution
Date occurs on the 25th day of a month".

The distinction is falsifiable, and was falsified. Recomputing the deal's
seven published WALs under four axes, only settlement-to-the-25th (actual/365)
reproduces all seven to the printed tenth. The discriminating column is 400%
PSA: printed 2.9; settlement-to-25th 2.9474; the engine's month-end axis
2.9608, and both month-end-from-settlement variants 2.956-2.962 — every
month-end reading rounds to 3.0. Amounts are unaffected throughout: P&I per
period is scheduled activity and carries no day. Only the time-weighting
moves.

Today the seven REMIC cases carry the gap as tolerance: ±0.07, decomposed in
each `case.toml` as 0.05 print floor plus ~0.015 axis. That is stated rather
than hidden, but it is the wrong long-term shape, because the axis differs by
program — Ginnie I pays the 15th, Ginnie II the 20th, FNMA REMICs the 25th —
while a single widened band is deaf to which one a deal used. A band derived
from one deal's axis can hide a same-sized convention error in the next
deal's, and nothing in the suite would notice: 2019-2 happened to publish a
column (400%) that discriminates, and the next deal may not.

Shape, in two independent pieces:

- a run-config `settlement_date` that becomes the origin for `wal_years` and
  `payback_years`, measured in actual days;
- a declarable payment-day placement — a day rule the pack lowering carries
  into its emitted streams' offsets, the way model-level schedules already
  carry `on day <n>` — so a deal states "distributions on the 25th" once and
  every time-weighted metric lands on the deal's own axis.

With both, the WAL tolerances return to the print floor and the axis stops
being a tolerance line item anywhere.

Found asserting the seven published WALs of FNMA 2019-2, where the 400% PSA
column refused the naive floor and the refusal was the convention speaking.

### 7.35 Accessibility: the automated findings are fixed; the human pass has not run

The WCAG 2.2 AA assessment (`docs/23`) fixed everything automation found — the
muted-text token failed 4.5:1 in both themes, the playground splitter could not
report its value to a screen reader, and scrollable tables, code blocks and the
results panel were unreachable by keyboard. axe now reports zero violations on
every swept page of both apps, and the deployed sites matched local builds.

What remains is what a rule cannot check: a screen-reader session over the docs
reading flow, the playground round trip and a learn exercise; 2.2's judgment
criteria (focus appearance, dragging alternatives, consistent help); content
order at 200/400% zoom; and a skip link, which landmarks currently stand in
for. Until that pass runs, the public statement is "built to WCAG 2.2 AA;
formal conformance assessment in progress" — not a claim of conformance.

Method note for whoever runs it: test themes through the stored-preference-plus-
reload path. Stamping `data-theme` on a live page produces mixed-token states
that are unreachable in production and read as catastrophic contrast bugs; two
false findings died that way during the assessment.


### 7.38 A misspelled series reads as zero, in silence

`series_sum("no.such.series", 0, time.t)` in a stream returns 0.0 for every
period and emits nothing — no diagnostic at compile time, no warning at run
time. The same read inside a field's `next` does warn: *"series `w.step_a` is
not available in this context; using 0"*. One of the two is wrong, and it is
not the field.

A stream that reads a series which does not exist has almost certainly been
mistyped, or names something the model no longer produces. Reading it as zero
is the worst available answer: a benchmark case can go green while asserting a
line it never computed, and `expected.csv` will agree with it, because zero is
a number.

Cheapest fix: resolve series names at compile time against what the model
lowers to, and refuse the unknown ones. If a late-bound name is genuinely
needed, the run-time path should at minimum warn as the recurrence already
does.

Provenance: found probing 7.37, August 2026, when a waterfall step's series
read as zero and the only reason that was visible at all was a field warning
about a different read.

**A third failure mode is now closed, and it was not the one this entry
describes.** A pack expression could read an INSTANCEABLE stream family by its
bare name. `.*` matches the bare name and its children; a bare pattern matches
only the bare name, so every suffixed instance is skipped — and nothing warns,
because the pattern did match something. That is worse than the case below: the
warning that would fire on a name matching nothing never fires at all.

It reached main twice, in the same expression, both times in forward NOI:
`cre.pct_rent` double-counted an unsuffixed contract, and `cre.property.opex`
made an instanced expense line invisible, overstating NOI and the exit price
struck off it. Both were found by hand, months apart. `tools/check-pack-series.py`
is now a gate over three surfaces — lowering expressions, metric selectors, and
statement row selectors — because the same mistake was made independently on
two of them.

**Of what this entry actually describes, half is closed, and it is worth being
exact about which half.**
A name the model does not produce ANYWHERE warns — `W5022_UNKNOWN_SERIES_REFERENCE`
— rather than failing, because a literal naming nothing is a pack idiom as well
as a typo (`cre.exit` names nine NOI components and a given property declares
some of them), and refusing it outright broke four goldens. A name the model
DOES produce but which cannot be seen from where it is read now fails:
`E1342_WATERFALL_SERIES_NOT_VISIBLE`, §7.41 item 3. What remains here is the
first case, and it stays open until the convention is settled — the question is
whether a pack's own expression should be able to name a component the model
lacks, not whether a typo should be caught.

*(Update: the unsettled convention has a measurable cost, probed while writing
`E1346`. Both step-visibility checks skip any reference ending `.*`, on the
reading that a selector states matching nothing is intended. So a stream
reading `series_sum("fund.distribution.*", ...)` — a glob over a WATERFALL's
steps — compiles, runs, and pays 0.00 every period with no diagnostic, while
the same read spelled exactly is `E1346`. The distinction the allowance rests
on does not hold: a selector matching nothing is a pack idiom, but a selector
whose matches are all step names is naming things that DO exist and are simply
unreadable from a stream at any time. Settling the convention should
distinguish those two, rather than treating every `.*` as an intent to match
nothing. A few lines in the existing checks — the step set and
`selector_matches` are both already there.)*

---

### 7.41 A freeform pot expression is still unchecked

*Roadmap: M2 (§7.78). Narrowed by M1's account (`docs/28` §5.1).* The checked forms now exist:
`from available` is the engine's own quantity, and `from <account>` draws a
balance whose inflow is declared and whose movements are journaled per
period — what flows in is named, checked, and auditable. What remains open
is the freeform `from <expr>`: a hand-written pot names whatever its
windows happen to say, and nothing checks the economics of the selection.
The residue is the freeform form only, and the account is the recommended
spelling wherever the pot is "what has accumulated."
### 7.43 Results do not say which entity owns a stream

This is a request rather than a defect, and the part of it that is a defect is
smaller and different from the part first written down.

**The axis exists and is correct.** Without a pack, a model's structured view of
its own cash is `entity.<symbol>.net_cash_flow`, one series per entity,
aggregated through `part of` — twelve pools rolled into a trust to within 2e-06
in the pack-free AmeriCredit model, identical to the packed one.

**A model without a pack publishes no `domain.*` series, and that is by
design.** `docs/01` §15.2 states that CFDL models do not declare output metrics
and points at the pack interface for how a pack defines output categories,
aggregations and metrics. A statement is a pack's job. The request below is for
a DEFAULT presentation, not for a model-declared one, and it should not be read
as a gap in the language.

**The original claim about a parent's own cash was wrong.** A trust with pools
and a fee of its own does not show one number: the fee is published in its own
right, and the arithmetic closes.

```
entity.asset.trust.net_cash_flow   550        rollup
entity.asset.pool1.net_cash_flow   300
entity.asset.pool2.net_cash_flow   200
stream.trust.fee                    50        the parent's own cash
                                              550 - 300 - 200 = 50
```

**What is actually missing is the ownership.** A series entry in results carries
`index`, `offset` and `values`, and nothing else; `docs/06` never names an
owner. So the derivation above needs the parent-child tree, which lives in the
IR rather than in results, and a consumer holding `results.json` alone cannot
attribute a stream to an entity at all. Name inspection is not a substitute: a
pack-lowered `cre.unit.base_rent.anchor` does not contain the symbol of the
entity that owns it.

Publishing stream ownership is the smaller change and the more useful one. It is
a structural fact the engine already holds, it lets any consumer build a
hierarchy view without the engine shipping one, and it makes an entity's own
cash derivable from results alone. Publishing `entity_own` beside the rollup is
the narrower version of the same idea and would serve the default statement
directly.

**The presentation request stands on its own merits.** A default statement
organized by the entity tree — each node's cash with its children beneath it,
no declarations and no pack — would give every model a readable cash flow rather
than a flat list of series keyed by symbol. Nothing in the language prevents it
and nothing forces it; it is a product decision about what a pack-free run
should look like. A declarable statement structure, the pack's fold available to
a model that wants to name its own lines, is the larger job and a separate one.

Provenance: found sectioning `benchmarks/credit/americredit_2017_1` into a
pack-free model, August 2026. Rewritten August 2026 after probing the own-cash
claim, which does not hold — the parent's streams are published individually and
the rollup arithmetic closes — and finding the ownership gap underneath it.

**Status, 31 August 2026 — shipped, and wider than the entry asked.**
results_version 0.7 publishes a `graph` section — every entity's symbol,
family, type, `part of` parent, and stable id — and attributes each stream
series to its owning entity AND its category. A consumer holding
results.json alone can now build the hierarchy view, attribute any stream's
cash to the thing that owns it, and select by kind. The schema carries the
descriptions; `docs/06` regenerated.
---

### 7.44 The engine's stages are modules, not crates

*Narrowed. The file split shipped: `crates/cfdl-engine/src/` is ten modules and
`lib.rs` is about 2,200 lines, not the 5,341 this item was filed against.
`run_deterministic` reads as the stage list it runs — config, timeline, state,
streams in waves, subtotals, waterfalls, results.*

What remains is the second step the original entry proposed: making the stages
CRATES rather than modules, so the compiler enforces the layering that the
module boundaries currently only suggest. A module can reach across a boundary
and nothing objects; a crate cannot.

Weigh it against the cost before doing it. The stages share the IR types and
the expression environment, so crate boundaries mean either a shared types
crate everything depends on, or a lot of re-export. The benefit is enforcement
of an order that is already documented and already tested by
`fixtures/valid/evaluation_order`, which is a real but modest gain.

### 7.46 A run with no discount rate still publishes an NPV

A run that states no rate discounts at zero and reports the result as
`model.npv`:

```
cfdl run <ir> --out results.json          (no --config, no --rate)

  model.npv                = 3750.0
  run.annual_discount_rate = 0.0
```

3,750 is the undiscounted total. The rate is published beside it, so the run is
not lying, but a metric named `model.npv` is being reported for a valuation
whose rate nobody stated, and a reader scanning results for a present value sees
a figure that reads as valued and is not.

**No rate, no NPV.** A discounted metric with no discount rate is a missing
term, not a shortcut, and the repository already applies that standard where it
matters most: `cre.permanent_debt` deliberately defaults neither `principal` nor
`rate`, because "a mortgage with an unstated balance or an unstated rate is not
a modeling shortcut, it is a missing term, and E5006 should say so rather than
the pack inventing a zero." A valuation with an unstated discount rate is the
same shape. Omit `model.npv` and the metrics derived from it when no rate is
supplied, and say why.

The zero default is what makes the current behavior defensible-looking: it is a
real arithmetic answer to a question nobody asked. Removing it costs nothing a
model wanted, because a model that means zero can state zero.

**Scope to settle when implementing.** Which metrics travel with the rate —
`model.npv` certainly; whether `model.irr`, `model.payback_years` and
`model.wal_years` do is a separate question, since a time-weighted life needs an
axis rather than a rate. And whether omission or an explicit null is the better
shape in `results.json`, which `docs/06` should state either way.

**Not the same as letting a model set the rate.** Discounting belongs to the
run: one set of cash flows is valued at several rates by different readers, and
neither rate is a fact about the asset (§7.42). A model default with a run
override, or a run that falls back to `inputs`, would put the resolution order
out of sight and let one model value differently depending on which channel won.
The fix here is to require the rate, not to relocate it.

Provenance: found August 2026 while correcting §7.42, when a probe run without a
config produced an NPV equal to its own total.

---

### 7.51 A parameter override is never checked against the model

*Narrowed. The schema half shipped — `tools/check-run-schema.py`, wired at
`run-schema` in the makefile, validates every committed run configuration
against `run.schema.json`.*

What remains is resolution, which the schema cannot do. `parameter_overrides`
are applied by key with no check that the key names anything the model
declares, so `inputs.captial_cost` for `inputs.capital_cost` overrides nothing
and the run reports ok. The schema knows the SHAPE of an override key; only the
IR knows whether it resolves.

This is the same family as the unresolved-name work: a name that resolves to
nothing must not read as silence. The engine already refuses an unresolved
`inputs.` read inside an expression; an override naming a non-existent input is
the same mistake one layer out, and should be the same kind of error.

`run.schema.json` also contradicts itself on the point — its header says
"Unknown properties are rejected", and the `parameter_overrides` description
says "Four key shapes are recognized and anything else is ignored". Both cannot
be true, and the second is what the engine does.

### 7.54 The HUD case cannot move onto `cre.permanent_debt`

*Belongs with the CRE pack (section 1). Split from the closed 7.14.*

The reporting half is done: `benchmarks/cre/hud_home_multifamily` states P+I+MIP
the way its source publishes it, and mortgage insurance is no longer counted as
debt service.

The case still hand-writes its mortgage rather than using `cre.permanent_debt`,
because HUD's instrument carries mortgage insurance the contract does not model.
A `cre.mortgage_insurance` contract is the shape that would close it, and it is
not added on one case's evidence — the pack candidate list (§7.5) is where it
belongs if a second source wants it.

This is the coverage question §7.3 and §7.15 measure, in one instance: a case
that reconciles externally while routing around the pack it should validate.

---

### 7.55 A model cannot declare a subtotal or a statement

*Belongs with the language and engine (section 5). Split from the closed 7.17.*

Reporting is a language capability in its design — the category roots are the
language's, a stream states its own `category`, and a pack-less model classifies
its streams correctly. The DECLARATIONS still live only in pack TOML, so a model
with no pack cannot declare a subtotal or a statement of its own.

That needs a surface in the language and the syntax is undecided; `docs/16`
records the question. It is the half this item's original title was about, and
the larger of the two — the pack-side fold, the classification, the grain
folding and the display sign all shipped.

Related: §7.43, where the same absence shows up as results carrying no statement
for a pack-less model. §7.25, where a model could not declare a metric either, is closed.
The three are one surface question asked from three directions.

**Rewritten 2026-09-01, after the design discussion this entry was blocking.**
Both of the original nouns are wrong, and the entry had been asking for the
wrong construct.

- A **slice** is a FILTER — focus on one product line, region or period. It
  narrows what is included.
- A **statement** is the ORGANISING STRUCTURE — whether a presentation is an
  entity hierarchy, a category hierarchy, or something else. Independent of any
  filter. This is the actual gap.
- A **subtotal** is not a declaration at all. It is what an interior node of a
  hierarchy looks like at a chosen level of aggregation: show an entity
  hierarchy two levels deep and the interior nodes ARE the subtotals, and
  aggregating coarser changes them. Nobody enumerates them, which is how a
  statement carries dozens of rows without dozens of declarations. A model
  needs no subtotal construct.

What makes this buildable now is what shipped since: the entity hierarchy is
published in `graph` (§7.43, §7.91) and rolled up as
`entity.<symbol>.net_cash_flow`; a category is a dotted path, so its levels are
structural; and a metric can be declared (§7.25) and now fold the published
surface (§7.85, §7.86), so figures can sit beside a statement.

**Shipped 2026-09-01, part one: the slice window.** A slice selected streams and
all of their periods, so "the 2027 to 2028 result for this asset" was not
expressible. `window from <date> to <date>` bounds the periods; a period outside
it contributes nothing, so `total`, `npv` and `irr` are folds over the window,
and the window publishes in the slice's own selection because it is the one part
of a selection that removes cash a reader can still see in the series beside it.

Not a phase, deliberately: a phase is a lifecycle anchor that drives schedules
(`phase_start()`, `phase_end()`), a window is a reporting bound on a finished
projection, and one construct with both jobs would mean neither could change
without the other. Dates rather than period indices, because an index is a fact
about one grid. `window` is a CONTEXTUAL word, like `category` beside it, so the
reserved-word list is unchanged at 100.

`ledger_hash` is unmoved by adding slices, verified rather than assumed — a
presentation is not a change to the underlying values.

**Shipped 2026-09-01, part two: the statement.** `statement <name> { structure
entity | category, depth N, grain, slice, metrics }`. It enumerates no rows.

The rows come from the tree — `part of` for an entity structure, the dotted
path for a category one — and `depth` decides which are shown. **A node whose
children are shown is a `subtotal`; a node whose children are cut off is a
`line`, carrying all of its descendants' cash.** That one rule is what keeps
the bottom line reconciling at every depth, because the lines always partition
the cash whichever level the tree is cut at. Measured on the fixture: the same
model reconciles at 480 as a two-level entity tree, as a one-line summary, and
as a three-level category tree.

An entity row FOLDS ITS SUBTREE rather than reading the published
`entity.<symbol>.net_cash_flow` rollup. The rollup is the same number and
cheaper, and it is computed over all of the entity's cash — so a statement
scoped to a slice would have silently ignored the filter.

**A filtered statement reconciles against its SLICE**, not against the model.
Reconciling it against the model reported the filter as a shortfall and raised
`W3502` on a correct model, which is the noise standard this codebase already
holds ratios to.

`ledger_hash` is unmoved by adding statements, verified: the same model with
none and with four hashes identically.

Packs converge on the evaluator rather than the surface: `cfdl-run::enrich`
renders model statements beside the pack's, `StatementsSection.pack` is now
optional because a model-declared statement has no pack, and all 45 benchmark
cases render byte-identically — including the HUD pack statement.

Fixtures: `valid/statement_by_entity` (four statements over one model: two
depths of the entity tree, the category tree, and a sliced one with a metrics
block) and `invalid/statement_unknown_structure` (`E1367`). New codes: `E1366`
(duplicate), `E1367` (unknown structure, or a category structure over
uncategorized streams), `E1368` (unknown slice or metric).

**Shipped 2026-09-01, part three: authored rows.** A statement may state its
own rows instead of generating them. A generated statement is right when the
tree IS the presentation; a pro forma is not that, because its rows carry
curated labels, its expenses show positive under "Less:", and it ends in a
coverage ratio that is a node of no hierarchy.

```cfdl
line     "Less: operating costs" { category "operating.expense.*" display positive }
subtotal "Net operating income"  { category "operating.*" }
ratio    "DSCR"                  { of noi to debt_service display positive }
```

This closes three gaps at once, which is why it was the first thing to build:
labels, the display sign, and the per-period ratio — which needed no entry of
its own after all, because `ratio` is a row kind the packs already use.

**A ratio divides two declared SLICES.** A slice is already a named selection
with a per-period net, so a ratio needs no row identifiers. A zero denominator
publishes `null` rather than zero, the rule a pack ratio already follows.

**Authored or generated, never both, and never neither** (`E1369`). A generated
statement partitions the cash by construction, because a hierarchy covers its
own tree; an authored one partitions it by the author's care. Mixed, neither
holds — an authored row claims streams the generated rows already claimed, and
the bottom line double-counts. The published IR schema states the rule as a
`oneOf` rather than leaving it to the compiler alone.

**The display sign never changes what is summed.** Debt service is stored
negative because an outflow is negative cash, so NOI over it is arithmetically
-1.75; `display positive` renders the conventional 1.75 and leaves `values`
signed, so a consumer that ignores the sign still adds up.

Found by the gate rather than by review: an authored statement first emitted
`structure: ""`, an empty string meaning "no value", which `check-ir-schema`
refused. It is omitted now.

Fixtures: `valid/statement_authored_rows` (curated labels, a flipped expense, a
claiming-nothing subtotal, a spacer, and the ratio) and
`invalid/statement_authored_and_generated` (`E1369`). No existing golden moved.

**Shipped 2026-09-01, part four: a generated statement reads as one.** Three
presentation defects, two of which a single-root single-category fixture could
not show.

**Depth first.** Rows were sorted by (depth, symbol), which is BREADTH first:
two funds holding two properties each came out as both funds followed by all
the properties in one flat block, with nothing saying which belonged to which.
A parent is now followed by its own subtree. Siblings sort by symbol —
declaration order would read better and is not available, because the IR sorts
entities by their stable key so its bytes do not depend on where a declaration
sits in a file. (The earlier note here claiming the IR preserves declaration
order was wrong.)

**The category roots have a canonical order.** `cfdl_pack::CATEGORY_ROOTS` is
operating, investing, financing — the order a cash flow statement is read in —
and generation iterated a `BTreeSet`, putting financing first. Below a root
there is no canonical order, so siblings sort alphabetically: arbitrary, but
stated in the spec rather than emergent.

**Labels are derived.** The last path segment, underscores opened out, first
letter capitalized, so `operating.revenue.base_rent` reads "Base rent" and
`asset.north` reads "North". A generated statement is meant to need no
declarations, and a row reading its own selector is a presentation that has not
been presented. An authored row states its own label.

Fixture: `valid/statement_generated_order` — two funds and three properties
across three category roots, which is the smallest model that shows either
ordering defect. Of the existing goldens only labels moved: no value, no
ordering, no hash.

**Shipped 2026-09-01, part five: the default statement.** A model that declares
no statement, with no pack providing one, is rendered as its entity hierarchy
and marked `default`. §7.43 asked for exactly this — "each node's cash with its
children beneath it, no declarations and no pack" — and called it a product
decision, which it was until the generator existed.

**A fallback, not a declaration.** It is assembled when results are rendered and
never enters the compiled document, so it moves neither `model_hash` nor
`ledger_hash` — verified: no IR golden changed and no hash moved. It yields to
any declared statement, a pack's included, because a declaration means the
presentation question is already answered.

The measurement that settled the size objection: a median of twelve values
added per document, and the largest addition is about 1,200 cells on a file
already 1.7MB. 130 result goldens gained a section; 36,694 insertions and zero
deletions, so nothing existing moved.

**Shipped 2026-09-01, part six: one evaluator. §7.55 is closed.** A pack's
`statements.toml` lowers into the same shape a model's statements use, and the
second renderer — 407 lines — is deleted. One evaluator, two producers.

The convergence was worth doing because the divergence was already costing:
while there were two renderers they drifted, and the model path was the one
that had drifted. Reading them side by side found three defects in shipped
code, none of which any golden caught:

- rows were never bucketed to the statement's grain, so an annual statement
  published two labels against twenty-four monthly values (fixed in part five's
  follow-up, `#264`);
- a ratio would have been re-bucketed rather than recomputed, giving -3.6 where
  the answer is -2.0;
- an authored statement that omitted cash emitted a silent residual row, where
  a pack statement named the streams with `W3500`.

**What the byte-identical test found.** The acceptance test — 45 benchmark
cases and every pack golden rendering unchanged — caught four more differences
that a reading would not have:

- a pack row publishes the BARE stream name (`cre.unit.base_rent.anchor`), as a
  slice does; the model path published the prefixed results key. Three
  publishers, one spelling, and mine was the newcomer.
- an UNCLASSIFIED stream is never claimed by name, because a stream row refines
  within a category. Dropping that condition put an "Operating expenses" line
  of -240,000 above a net operating income that excluded it. `dscr_smoke` is
  the model whose comment predicted exactly this.
- a named series must be PRESENT, not merely declared: `Grain::sum` of an
  absent series is one zero per bucket, not an empty vector, so a row must
  publish no values rather than a column of manufactured zeros.
- a ratio whose inputs were never published still emits its row, falling back
  to its own series. Dropping the row silently shortened a statement the pack
  declared.

`W3501_STATEMENT_STREAM_DOUBLE_COUNTED` was lost in the deletion and restored:
the converged path tracked claims in a set rather than a count, so a stream
claimed by two rows — "wrong in a direction that looks plausible" — became
invisible. Found by auditing the codes the deleted renderer emitted against
the codes the new one does, which is the check worth running whenever four
hundred lines go.

---

### 7.56 A term deferred to `inputs.` is never bounds-checked

*Belongs with the language and engine (section 5). Split from the closed 7.24.*

Two questions survived the correction to 7.24, and they are separable.

**A term referencing `inputs.` escapes its pack's bounds.** `docs/01` §8.2.1
accepts this deliberately — the value is not known at compile time — but it
means a scenario may drive a term past a bound the pack states, at compile time
and at run start alike. Either the bound is checked when the value arrives, or
the pack's bound means less than it appears to.

**Should `cfg.*` work in a term as well?** It is the run configuration's other
half. A reader who reaches for it today gets a diagnostic saying the value is
invalid rather than that the channel is wrong: `E9016` naming a bound is
actively misleading when the term is `cfg.psa`. If the answer is no, the
diagnostic should say so.

Both are about a term's relationship to the run rather than about bounds as
such, which is why they belong together.

---

### 7.57 A pack rule cannot accrue on act/act, because a divisor is not a fraction

*Belongs with the packs (section 5). Split from the closed 6.1.*

`year_frac` accepts `act/act` (ISDA), so a hand-written model can accrue on it.
A pack rule cannot: `{{model.accrual_divisor}}` expands to `<ppy>` or
`(360 / time.days_in_period)` — one number per period — and act/act needs a
denominator that changes with the year the period falls in.

The shape is the one the expansion table already implies. A divisor is the
reciprocal of a year fraction:

```
30/360   rate / 12                  ==  rate * year_frac(s, e, "30/360")
act/360  rate * days/360            ==  rate * year_frac(s, e, "act/360")
act/365  rate * days/365            ==  rate * year_frac(s, e, "act/365")
```

So the placeholder could expand to a `year_frac` call over the period's bounds
rather than to a number. act/act then falls out with no special case, and the
pack placeholder becomes sugar over a capability a model already has natively —
which is the property worth having whether or not act/act is the reason.

Note that two of the three expansions are already run-time text, not compile-time
constants: `(360 / time.days_in_period)` reads the environment. So the argument
that the divisor must resolve at compile time holds only for the fixed case.

**What it needs first.** `year_frac` takes two dates, and an expression can read
`time.date` and `time.days_in_period`. Whether those reconstruct the period's
start and end — and which end `time.date` denotes — is the fact to establish
before scoping this.

---

### 7.60 A weekly schedule cannot be anchored to a weekday

*Belongs with the language (section 5). Found building the keyword register.*

`docs/01` §18 documented `Mon` through `Sun` as reserved words for eight
versions. They are reserved, they render in error messages, and no production
reads them. The syntax they imply does not exist:

```
schedule every week on Mon from 2026-01 to 2026-02
  -> Expected 'day <n>' or 'eom' after 'on'
```

`weekly` is not a calendar frequency either — the frequencies are `daily`,
`monthly`, `quarterly` and `annual` — so a weekly TIMELINE is unavailable as
well, though `every week` is a valid schedule interval.

Provenance: found by the gate that now holds §18 to the lexer. The words were
documented as though the feature shipped, which is how it went unnoticed.

**What it needs.** `on <weekday>` in the schedule anchor, and a decision on
whether a weekly calendar frequency is wanted or whether weekly schedules on a
daily timeline are the answer. No case needs it yet; a rent roll on weekly
billing or a daily-book instrument settling on Fridays would.

---

### 7.61 Nothing checks the grammar against the parser

*Belongs with the language (section 5). Replaces the closed 7.49.*

`docs/schemas/CFDL_v0_1_Grammar.ebnf` is NORMATIVE and published. `docs/02`
says implementations MUST support the lexical rules, calls the grammar
"suitable as the basis for a hand-written parser or parser-generator input
after minor adaptation", and the site offers it for download for use with
"railroad diagram generators, parser generators, etc."

Nobody has ever performed that adaptation, so nobody discovered the grammar did
not survive it. Five productions were wrong when checked by hand — `contract`
alone was wrong four ways, and would have rejected 519 of the 520 contract
declarations in this repository. They are fixed. Nothing stops it recurring.

**It recurred on 2026-08-27, as predicted.** `account_stmt` was added to the
parser and then to the EBNF BY HAND, in separate edits, with nothing checking
that the two describe the same language. Both copies of the grammar —
`docs/schemas/` and the site mirror — had to be edited by hand as well. The
production may be right; nothing establishes that it is, which is the whole
complaint. Every keyword this project adds from here repeats the exposure, and
`account` will not be the last: the state machine of `docs/28` §6.1 and the
schedule anchor of §6.2 are both new surface.

**The parser is hand-written recursive descent, so the grammar is source for
nothing.** That is the right call — the diagnostics are a feature and generated
parsers do not produce them — but it means the two artefacts agree only by
attention.

**The long-term answer is to make the grammar executable in CI, in both
directions**, without generating the product parser:

- Build a RECOGNISER from the EBNF and require it to accept every shipped
  `.cfdl` — around 500 files CI already proves parse. Catches a grammar that is
  too narrow, which is what `contract_stmt` and `entity_block` were.
- Generate sentences from the EBNF and require `cfdl parse` to accept them.
  Catches a grammar that is too broad, which is what `map_entry` was.

That is how a published grammar is normally held to an implementation — spec
tests, in the manner of WebAssembly or test262 — and it keeps the hand-written
diagnostics. Cost is a real project: an EBNF adaptation layer, a generator
dependency, and a gate.

**The cheap interim** is a terminal cross-check: extract the keywords each
production mentions and require the parser to read them, and the reverse. Same
shape as `check-keyword-register.py`, no dependency. It would have caught
`owner` and `direction`; it would NOT have caught `term` moving inside the
contract block, so it is a stopgap and should be labelled one.

---

### 7.62 An option accepts `on entity`, which nothing documents

*Belongs with the language (section 5). Found reconciling the grammar.*

`option call_at_120 on entity asset.plant type Option.Call { … }` parses and
ships in models. §14.1 of the language specification does not show the clause,
and the EBNF does not have it — the parser grew it and neither document
followed.

Resolve in one direction: document it in §14.1 and the grammar, or remove it and
migrate the models that use it. It is small either way, and it should not
survive to v1 as surface nobody wrote down.

---

### 7.66 Two published pages disagree about the arithmetic, and nothing checks

*Belongs with the documentation (section 7). Found reading the live site.*

`/docs/reference/expressions` says:

> All arithmetic is floating point.

`/docs/specification/expression-environment`, which is NORMATIVE, says:

> All arithmetic is exact 128-bit decimal (`rust_decimal`, 28 significant
> digits). `0.1 + 0.2 == 0.3` is `true`.

The specification is right — `cfdl-calc`'s header states decimal-first with
float64 as a documented escape for transcendental work. For a financial
modelling language this is close to the most consequential sentence either
page carries, and the wrong one is on the page a modeller reads first.

**The same page pair is stale in the other direction.** The specification's
`excel_compat` paragraph says the mode "is reachable **only from Rust**...
There is no CLI flag and no run-config key, so a *model* cannot be run in it",
and "Nothing in the repo calls `eval_with_mode` today". Both have been false
since the `arithmetic` run-config key shipped: it is declared in
`run.schema.json`, the engine rejects any other value, and `eval` routes
through `eval_with_mode`. `docs/09`'s user-guide entry is correct, so the
three pages describe two different languages.

**The general defect is that nothing compares them.** Every gate checks a page
against the code or against itself — `check-doc-examples` compiles snippets,
`gen-glossary` matches the register, `check-site-voice` reads tone. Nothing
asks whether two pages making the same claim agree, which is why a
one-sentence contradiction survived on the site.

Fixing the two sentences is minutes. What is worth deciding is whether a
claims gate is possible at all, or whether the reference layer should stop
restating what the specification states and link to it instead — the same
single-source-of-truth question the gate list and the keyword register both
answered by making one place authoritative.

---

### 7.68 An assumption that fails to evaluate is reported as "not declared"

*Belongs with the language and engine (section 5). Found while giving
assumptions dependency ordering.*

When an `assume` fails to evaluate, the engine warns and skips it. Every later
read of that name then hits the unresolved-name gate, which says:

```
`inputs.net_sf` is not declared — each read as zero. Declare it, supply it in
the run configuration, or correct the name.
```

All three remedies are wrong, because the name **is** declared. The model says
`assume net_sf = ...` in plain sight; the assumption simply did not produce a
number. A modeller reading that message goes looking for a missing declaration
or a typo and finds neither.

Dependency ordering removed the common cause (an assume reading another
assume), so the message is now reachable only when an assumption fails for its
own reasons — a non-numeric result, a division by zero, a call the empty
assumption environment cannot serve. Rarer, and the diagnosis is still wrong
when it happens.

Shape: the gate knows the declared names, so it can distinguish "no such
assumption" from "declared but unresolved" and say which. The second case
should also name the ORIGINAL failure — the warning that explains why is
already in the warnings array, one entry above.

---

### 7.69 The annual grain is deliberate; what follows from it is not all settled

*Belongs with the engine (section 5). Found reconciling
`benchmarks/cre/penzance_highlands`. REWRITTEN after `Grain` became a type —
the first version of this item called the central behavior a defect, and the
tests say otherwise.*

Filed first as "the annual grain discards intra-year timing", evidenced by a
monthly model covering 2026 with one $100 inflow at 10%:

| | period-grain NPV | annual-grain NPV |
|---|---|---|
| $100 in January | 100.0000 | 100.0000 |
| $100 in September | **93.8436** | **100.0000** |

That measurement still reproduces. It is not a defect. Two tests in
`crates/cfdl-engine/src/lib.rs` assert exactly this, in both directions:
`npv_at_grain` on a monthly model must EQUAL the same cash as a single annual
payment — *"valued at the same annual convention these must agree"* — and it
must DIFFER from discounting that monthly model per period — *"discounting
twelve times differs from discounting one bucket once"*. Valuing at an annual
convention regardless of the grid the model is written on is the whole point,
and it is what lets a monthly model reconcile against an annually stated
source. Time-differentiating cash inside the bucket would defeat it.

`Grain` (`crates/cfdl-engine/src/results.rs`) has since made that mechanism a
type, with three consumers: the valuation (`lib.rs:1130`), the annual rollup
(`results.rs:155`), and statements, which reach it through
`Grain::from_index(ix, spec.grain)`. Anything below therefore touches three
surfaces rather than one.

Three questions the tests do NOT settle.

**Does the short first bucket belong in a VALUATION?** `Grain::calendar_year`
documents it — *"a mid-year start therefore produces a short first bucket,
which is what the annual rollup has always done and what a fiscal reader
expects"* — and for reporting that is plainly right. Discounting is a different
use of the same partition: a bucket's exponent is its integer index, so a flow
four months after a September start is discounted a full year. Identical cash
flows, 24 monthly inflows of 100 at 10%:

| model start | period-grain | annual-grain |
|---|---|---|
| 2026-01 | 2,193.81 | 2,290.91 |
| 2026-09 | 2,193.81 | 2,152.07 |

139 apart on the start month alone. A reader who accepts that January and
September are not distinguished *within* a bucket may still not expect the
model's start month to move the answer. Whether it should is a convention
question — a fiscal-year source would want exactly this, a project-life
valuation would not — and it is currently unstated either way.

**A stream's placement changes units across the two paths.** At period grain
the exponent is `i + offset`, so `end` waits one PERIOD. At annual grain it is
`bucket + offset`, so the same declaration waits one YEAR. Twelvefold on a
monthly model, from an unchanged line of source, and no test covers it.

**`model.irr` never follows the grain.** It always solves `irr_with_offsets`,
the per-period form (`lib.rs:1147`), while the NPV branches above it. To be
precise about what is and is not wrong: both read the same `valued_streams`, so
the IRR does solve NPV = 0 over exactly the cash flows the NPV values — the
difference is the convention, not the inputs. But under an annual valuation a
reader who checks by discounting at the reported IRR gets a non-zero NPV.

Nothing published is affected: **0 of 41 benchmark run configs set
`valuation_grain`**, so all three live in a path no case exercises.

Shape: state the convention rather than change it. The bucketing is settled and
should stay — calendar years are what make external reconciliation possible,
and `Grain` now expresses that in one place. What is missing is a written
answer to "what does a bucket's exponent mean", and a decision on whether
`model.irr` follows the grain or is documented in `06_results_schema.md` as
always model-grain. If the exponent ever does become electable, it belongs on
`Grain` as a property of the partition, not as a second code path.

### 7.70 A quantile's audit record is empty for the contracts that will use it

*Belongs with the language and engine (section 5). Found closing stage 3 of
`docs/27_quantiles.md`, against the contract that stage shipped.*

`InputsSection.quantiles` publishes each quantile call site with the slice it
asked for and what that resolved to. For a hand-written model it does what it
was built for:

    quantile_mean  prices  [0.98, 1.0]  ->  426.0

For `cre.percentage_rent_expected`, the first pack contract to consume a
quantile, it publishes this:

    quantile_mean  store_sales  []  ->  ABSENT
    quantile_of    store_sales  []  ->  ABSENT

**Not a defect in the resolver.** The record is computed at compile time, and
the pack rule deflates the breakpoint by
`pow(1 + growth, {{time.elapsed_years}})`, which expands to an expression over
`time.date`. The slice bounds are therefore genuinely different in every
period, and no single compile-time value exists to publish. Declining to invent
one is correct.

Constant folding does not rescue it. Even at `sales_growth = 0` the expanded
text still reads `time.date`, so the expression is not constant however
degenerate the arithmetic.

**What that costs.** The audit chain's stated purpose is that a reviewer can
check a nonlinear input without redoing the integral. That holds for a
hand-written model and does not hold for a pack-lowered one — which is the case
most models will be, and is precisely the case the primitive was built to
serve. `docs/27` §6 claims the property in general; it is true in one half.

**And the shape misreads.** `args: []` renders as a call taking no arguments
rather than one whose arguments vary by period. The results schema says
"empty when they were not literals", so the document is accurate and the
rendering is still misleading to anyone who has not read it.

**The fix is a stage 2 revision, not a patch here.** Recording slices during
EVALUATION would capture a value per period, which is the true answer. It was
considered and rejected when stage 2 was built, for reasons that have not
changed: the `Env` hooks take `&self`, so recording needs interior mutability;
it moves work into the per-period path that the compile-time design keeps out
of it; and the same call recurs every period, so it needs a dedup rule and a
canonical order or the results document stops being reproducible.

**The shape is already in the language, and it is not a scalar.** The slice
bound and the resolved mean are a NUMBER PER PERIOD — geometrically a curve,
but emitted rather than declared, which makes it a SERIES. The results document
already publishes non-cash per-period numbers that way: an entity field appears
as `{index: {calendar, start, periods}, values: [...]}` under its own key, bare
numbers with no currency wrapper, and 58 such series exist across the goldens.

Framing it as a series dissolves two of the three objections that stopped this
being built at evaluation time. Dedup and canonical order are moot, because a
series is one value per period in period order. Reproducibility is moot, for
the same reason it is moot for any stream. Only interior mutability survives,
and it may not survive either: the engine already evaluates these expressions
every period and already emits a per-period number for a field, so this is the
existing machinery rather than new machinery.

**And it is the argument that settles the design.** A scalar in
`InputsSection` is inert — a reviewer reads it and takes it on trust. A series
in `deterministic.series` is checkable BY MACHINE, every period, against a
reference: it inherits the CSV export, the per-period tolerance in the
benchmark harness, and the statement layer. `docs/26` makes exactly this point
about covenants — a benchmark asserts COLUMNS, and testing every period is
strictly stronger than testing one number. For a nonlinear input that is the
difference between publishing a figure and proving it.

So the design is: emit the resolved slice as a series under its own key, the
way a field is published, and let the audit run through machinery that already
exists. What remains open is the key's name, whether both the slice bound and
the resolved value are published or only the second, and whether the
compile-time scalar record stays for the literal case or is replaced.

Open this before any further pack contract consumes a quantile. Shipping a
second one against an audit record that does not work would make the gap
structural rather than a known debt.

### 7.74 Structured-finance engine parity — the Intex scope

*Roadmap: partly M2 (§7.78) — the deal mechanics; the analytics ride on
declared metrics (§7.25, shipped). Promoted 2026-09-01 to
`docs/38_intex_parity.md`, which carries the survey the way `docs/34`
carries §7.79's design: the parity-or-ahead ledger, the itemized gaps, the
non-items and the licensing position all live there, and this entry stays as
the anchor other entries reference.*

**What this item is.** An umbrella over the gaps that separate CFDL from the
full scope of a structured-finance cash flow engine (the Intex/Trepp
category: collateral pools feeding tranche waterfalls with triggers and
reserve accounts, plus bond analytics over the result). The collateral side
and the reserve mechanics are the larger half and are done; what remains, per
`docs/38`: the coupled-waterfall trio of `docs/17` §5 (cross-linked pots, the
shortfall series, deferred/PIK), the externally-referenced trigger case
(§7.77's remainder), servicer advances, a clean-up call case, valuation
solvers and the make-whole, per-period stochastic draws, the analyst output
surface (§7.22, §7.23, §7.26), the unexercised class types and structured
collateral (`docs/20` §2), multi-currency, and a loan-level scale
measurement. Same-period circular conventions stay out on purpose — the
causal plane's refusal to iterate is the product's guarantee, not its gap.

### 7.75 Storage state of charge is now buildable, and it is what validates the last energy rule

*Roadmap: M2 (§7.78); the case it unblocks is M3 (§7.3).*

**What forced the discovery:** the domain survey behind `docs/30`.
`energy.storage_arbitrage` is the energy pack's only externally-unvalidated
rule (§7.3: energy 9/10), and §7.1 recorded three ways forward, the third
being "needs per-period persistent state (5.2) and would let cycling be
modeled rather than assumed." The walk's phases 3 and 4 are that state: a
state-of-charge balance — a field or an account — stepped per period, charged
and discharged by streams the balance reads strictly backward.

**What it changes:** `quantity` (the storage rule's MWh cycled) stops being an assumed input and
becomes an output of dispatch against a price shape, which is the circularity
§7.1 says blocks validation against a dispatch reference. It is also the
state the `energy.storage_dispatch` quantile rule (`docs/27` §9 stage 4)
prices around: the quantile closes the Jensen gap, the SOC balance closes the
chronology gap — a 4-hour battery reaching only contiguous hours is a
constraint on a walked balance, not on a distribution.

**The gate is open, and the answer was not a tool.**
`benchmarks/energy/merchant_storage_arbitrage` ships against a provably optimal
linear program, because SAM's dispatch is documented as "automated but
SUBOPTIMAL" with "no optimization around the cost of energy and power"
(NREL/TP-6A20-68614) and reaches 27% of the optimum on that case's price year.
The full `Battery` module does run front-of-meter — §7.1's segfault was
`Battwatts` — but running is not the same as being a target.

The case is core-spelled and makes the state-of-charge argument concrete without
building the contract: cycling is an OUTPUT, the run/idle decision is a guarded
edge on a machine in IEEE Std 762's vocabulary, and the chronology cost is
measured at 4.8% — what carrying charge across midnight is worth, and therefore
what a daily grain gives up. What remains is `energy.storage_dispatch` itself
(`docs/27` §9 stage 4), which is no longer gated on a reference.
Related: §7.1, §7.3, `docs/27` §9, `docs/30` §2.

### 7.76 The account adoption pass: every pack has a reserve it could not model

*Roadmap: M2 (§7.78).*

**What forced the discovery:** the account shipped (`docs/28` §5.1) and the
domain survey (`docs/30`) found the same absence recorded independently in
every domain's references. `crest_solar_cost_based/NOTES.md`: the reference
EBITDA "includes interest earned on funded reserve accounts (~$4,606 in year
one), which CFDL does not model." `utility_pv_singleowner/NOTES.md` lists
reserves among what the reference zeroed out to be comparable. §7.5 carries
`cre.replacement_reserve` from two sources. The roadmap's hospitality entry
is one accumulating FF&E reserve. Servicer advancing (§7.74) is a
recoverable-advances balance.

**The ask, in three parts** — the first and third are done, and the second
is open. First, the migrations the shipped fleet already
owes: the flip case's hand-carried pot (`docs/25` — the one case where
revenue is computed a second time inside the distribution) and Highlands'
cumulative window, both named gate shapes in `docs/29` phase 4. **Highlands is
done** (2026-08-29): `series_sum("cre.*", 0, time.t)` became
`account deal_cash`, and the identity held to the byte — `model.total`,
`model.irr`, `model.moic` and both payee totals unmoved on the first run. The
conversion also found what a cumulative pot costs: the pot was net of
contributions, so it could not return capital, and a party account carrying the
contribution had no offsetting leg —
`moic(party.baupost)` published 0.96 on a deal returning 2.05x. Grossing the pot
up and adding two return-of-capital tiers moved both MoICs by exactly +1.0 and
no split at all. **The flip case is done as a twin** (2026-08-30,
`benchmarks/energy/tax_equity_flip_account`): the original states what it was
waiting for, so the rebuild is carried alongside it rather than replacing it,
and both reconcile against the same external anchors at the same one-cent
tolerance — against the original's own output the twin is within tolerance on
50 of 50 cells, largest difference 0.0047 dollars on figures of about four
million. The residual is reassociation, not modeling: the original reconciles
to 1.0e-6 against the reference and the twin to 4.7e-3, the same quantities
summed through the ledger rather than inside one field expression. Whether the
original retires is left open deliberately, since it is the suite's tightest
external reconciliation. Second, a
reserve contract shape per pack where a document demands one — the DSRA
funded to target with `dscr_periodic` gating the release, the replacement
reserve of §7.5, the FF&E reserve — each as the `pay <step> to account`
pattern rather than a bespoke contract. **The credit pack's is done**
(2026-08-31, `benchmarks/credit/americredit_2017_1`): clause 19's reserve, 2.0%
of the initial pool funded at closing, was a literal written out twenty-eight
times and a step `pay reserve_topup to party.certificate = 0.0` — the right
amount to the wrong payee. It is now `account reserve` funded by its own `from`
inflow at closing, with clause 19 as the top-up
`max(0.0, inputs.reserve_required - prev.reserve)`, and the
overcollateralization target reading the balance the prospectus states it
against. All 177 series unmoved at every period, zero difference.

**The energy pack's is not, and this entry was wrong about why.** It reads
above as though CREST is the near-done one — "the case that reconciles against
CREST's own ~$4,606 still wants the reference". What CREST wants is not a
reference for the interest; it is the reserve SCHEDULE the interest is earned
on, and that is the one thing not in the repo: the port is unlicensed, was run
once outside it, and only its output numbers were carried across. The ~$4,606
is a single rounded year-one aggregate against three unknowns — balance,
funding rule, rate — and the conventional structures do not fit it (6mo debt
service + 6mo opex implies 2.1368%, 6mo debt service alone 2.9417%, 12mo
1.4709%). Fitting one is numerology, and CREST is the suite's tightest external
reconciliation. There is a second, independent blocker: CREST funds reserves at
close, and the case deliberately has no close period — `funded_at_close = 0`,
and `model.cfdl` records that a period 0 would shift every escalation exponent
by one. Energy's reserve wants the port re-run, which is a sourcing step; the
choice to take it is open.

**What doing it in the credit pack found**, both of them prerequisites rather
than by-products. `account` was missing from the parser's `is_statement_start`,
so an account declared after an `assume` was silently swallowed — the same bug
that list's own comment already records once, a `metric` declared after a
contract vanishing the same way; `lifecycle` was missing too, making three
instances of one omission. And `window_bound_is_backward` did not recognise `if`, so the
AmeriCredit pot's `if(time.t == 1.0, 0.0, time.t)` lower bound — the first
distribution draws two collection periods — was read as forward, keeping the
model on the column order, where account balances are not computed at all. `if`
now joins `max` and `min`. That change also produced the first evidence that
the corpus's most intricate waterfall agrees period-for-period under the walk
and under the column order. Third, interest ON a reserve balance — **done 2026-08-30**,
`fixtures/valid/reserve_interest_on_balance`. The entry was wrong about the
spelling, and the first attempt was withdrawn for a reason that turned out to
be wrong too; both are recorded because the second one is the interesting one.

**The spelling.** The entry called this "a stream whose amount reads
`prev.<account>`, legal under §4's backward rule". That is refused —
`E1123_PREV_OUTSIDE_NEXT`, because `prev` outside a `next` means nothing — and
`docs/03` is precise that a balance is readable in rules, guards and step
expressions. A field's `next` is a rule, so the field carries the balance
forward and the stream reads the field.

**The withdrawal, and what it actually found.** The fixture failed
`walk_matches_the_column_order`: column 0 against walk 5. That reads as the
mechanism being unsound, and it is not. The test already excludes models whose
logic reads settled cash, on the stated ground that "the column order settles
all state before any stream has a value, so the read binds nothing there and
the model means something different — which is exactly the expressiveness the
walk adds". An account balance is that same category, and §5.1 above says so in
terms: `prev.<account>` is settled state read "the same way a delinquency edge
tests realised rent".

The predicate simply did not know about accounts. It detects `series_` and
predates `docs/28` §5.1, and no blessed model read a balance in logic until
this one, so the gap had never been exposed. Extending it to account reads is
completing an existing principle, not waiving a failure — and the property
still holds where it applies: 124 models compare with walk == column, four are
walk-only.

**The pin:** a reserve funds toward 3,000 out of 1,000/month, and interest
accrues at 0.5% on the PRIOR balance — 5.00 on the first 1,000, 10.03 on 2,005,
then 15.00 a month once the target holds. Reading strictly backward is what
keeps the reserve and the interest it earns from being mutually circular. The
CREST reconciliation line is closed as a mechanism; the case that reconciles
against CREST's own ~$4,606 still wants the reference.

Related: §7.5, §7.41, §7.72 (shipped), §7.74, `docs/25`, `docs/28` §5.1, `docs/30` §1.

### 7.77 A covenant that is published but powerless: the DSCR cash trap

*Roadmap: M2 (§7.78). **The mechanism shipped 2026-08-30**; what remains is
the benchmark against an external reference, which is `docs/20` §5.1's ask.*

**What could not be expressed:** consequences. The energy pack publishes
`dscr_periodic` per period (`packs/energy/statements.toml`, with its own
argument that "a project finance covenant is tested EVERY PERIOD"), and
`ppiaf_toll_highway` sizes a subsidy to hold 1.30x — but no model could say
what a real credit agreement says: below the trigger, distributions stop and
cash traps in an account; at or above it **for the cure period**, the trap
releases.

**What shipped.** `fixtures/valid/dscr_cash_trap_cure_period` runs the whole
covenant: NOI of 12,000 against 15,000 of debt service puts DSCR at 0.80
against a 1.20 trigger, the machine reads settled cash strictly backward and
traps at t=5, cash accumulates once NOI recovers (5,000 at t=7, 10,000 at
t=8), and two consecutive good periods at t=9 release the trap in full.

**The cure period was the part that waited on §7.79**, and it is worth being
precise about why. `trapped_cash_cure` has existed since the walk, and it
cures on the *next* good period — which no credit agreement says. A cure
period is a duration measured from the last breach, and a field recurrence
counts consecutive good periods without any way to start over at each new
one. `on enter trapped { set good_periods = 0 }` is the whole difference,
and it is the same shape as the EBA probation the credit pack's machine
carries (`docs/36` §2.1).

**What remains: the external reference.** A fixture asserted against its own
engine is the suite marking its own homework (`docs/20` §5.1). The mechanism
is pinned; the covenant case wants a published credit agreement with a
cash-trap schedule and figures to reconcile against, and none is vendored.
That is a case-authoring ask with a sourcing problem, not a language gap.

Related: §7.36, §7.74, §7.79, `docs/28` §5.1 and §6, `docs/30` §2,
`docs/20` §5.1.

### 7.78 M2: what the walk unlocked, and what it retired

*An umbrella, in the shape of §7.74 — it owns no work of its own; each
constituent is an entry below or above it.* Recorded because the v1.0
roadmap's M2 was written before M1 shipped, and two of the four items it
named no longer describe work.

**What M2 no longer is.** Sequential-pay note classes (the closed §2.4) run
today as an ordered waterfall — `benchmarks/credit/auto_abs_tranches`
compiles AmeriCredit's 22-step priority — so what remains of that item is
§7.74's deal mechanics, not a liability-stack construct. And contract gating
(the closed §7.40i) was not a runtime to build: §7.73 (also closed) concluded
the grain was wrong and the action should be retired, which made M2's gating
work §7.50 plus state-gating through the declared machine — both now done. Per-period persistent state
(the closed §5.2) shipped with M1 itself.

**Closed since.** §7.72 (a participant's realized return had no construct) is
fixed: `irr(party.<p>)` and `moic(party.<p>)` fold the party's OWN ACCOUNT —
contributions are negative inflows, receipts are allocations in, so the sign
change an IRR needs is recorded rather than inferred from payee streams, which
is the §7.43 attribution trap the entry warned against. The party is a
REFERENCE, so the compiler resolves it (`E1301`), checks it is a party that
owns an account (`E1356`), and refuses the fold outside a `metric` (`E1355`);
only flows that never change sign wait for the run, and that refuses naming the
party. `docs/31` W4 phase 2 is done, which leaves the calculator a benchmark
case and a surface.

**Closed since.** §7.25 (a model could not declare a metric) is fixed:
`metric <name> = <expr>` is evaluated once at the horizon in the valuation
plane and published as `metric.<name>`, a third namespace beside the engine's
`model.*` and a pack's `domain.*`. Metrics compose in declaration order — the
waterfall rule — with a forward or circular reference refused (`E1354`) and a
duplicate name refused (`E1008`). Every declared metric reaches every scenario
summary, because scenarios and the deterministic block publish the same map.
That unblocked §7.72 (participant-level returns), since shipped, and
`docs/31` W4 phase 1 is therefore done.

**Closed since.** §7.73 (the wrong grain) is fixed: `activate`/`deactivate
contract` is out of the grammar, and `E1303` — which resolved only that
action's target — is deleted with it. No new code marks the absence: the
parser's existing "Expected 'stream' after activate/deactivate" says enough,
and a language with no installed base retires a spelling by removing it, not by
commemorating it. The `ignored` journal outcome survives, since the engine
still needs it for an action kind hand-written IR carries and no compiler emits.
What remains of §7.40i is the contract-surface `active when` / `active in state`
that would let a pack's streams be gated as a group — worth a case before it is
worth a construct, since the per-stream spelling now covers the three documents
that forced the item.

**Closed since.** §7.50 (a model could not name the streams its own contracts
produced) is fixed: event stream targets resolve after lowering, where a
contract's streams exist, so `deactivate stream cre.debt.principal` compiles and
the loan's cash stops — `fixtures/valid/event_stops_lowered_stream` runs debt
service to zero at the period the event fires. `docs/04` §1.1 now records that
lowering is the one GENERATIVE stage, which is why a check over lowered names
cannot sit at name resolution. What remains of §7.40i's additivity argument is a
contract-surface `active when` / `active in state`, recorded under §7.73's
closure below.

**Closed since.** §7.45 (a waterfall with no schedule distributed once, at the
model start) is fixed: `E1348_WATERFALL_NO_SCHEDULE` refuses the omission, which
is what `docs/01` §10.1 had required in normative text since the waterfall
entered the spec — the compiler had been inventing a first-period default
against its own specification, and the engine's every-period branch, which no
compiler output could reach, is gone.

**Closed since.** §7.79 (an event fired once, and a transition could not
act) is fixed — the milestone's settled first priority, and the mechanism
three other entries were waiting to spell. #235 landed `docs/34` phases 1–4
(rising-edge occurrence, no latch, arrival actions, augmentation, the redrawn
pack machines of §7.84) and #236 phase 5. It paid out the same day: #238
built §7.77's cash trap whose cure is a period (`on enter trapped
{ set good_periods = 0 }` is the whole difference) and §7.76's interest on a
reserve balance, and shipped the flip case's pot as an account twin — so of
the table below, §7.77 remains only as an external-reference benchmark and
§7.76 only as part two, the reserve contract shape per pack where a document
demands one — and part two is now down to the packs other than credit, whose
reserve shipped 2026-08-31 on `americredit_2017_1`. Energy's is blocked on
sourcing rather than on language: see the entry.

**What M2 is**, all of it standing on the walk, the machine and the account
(`docs/28` §4–§6):

| item | what it unlocks |
|---|---|
| §7.41 | the freeform `from <expr>` pot, the one unchecked selection left after the account |
| §7.76 | the account adoption pass: the reserve every pack's references assume and no pack could model |
| §7.77 | the DSCR cash trap — the first covenant whose breach has consequences, and can end |
| §7.75 | storage state of charge, which turns `quantity` (the storage rule's MWh cycled) from an assumption into an output |
| §7.74 | the deal mechanics still open after the machine: coupled interest/principal waterfalls, a step's shortfall, PIK on an unpaid step, servicer advances, the clean-up call |

**What M2 is not.** Declared metrics (§7.25, since shipped) and
participant-level returns (§7.72) are M4 — both since shipped — and `docs/31` W4 pulled the first forward on the
commercial path rather than the roadmap's. Pack coverage (§7.3) is M3.

Re-derived 2026-08-28; §7.79's closure and its consequences recorded
2026-08-31. The full five-milestone roadmap this entry re-derives M2 from —
M3 validation coverage, M4 polish, M5 release mechanics included — is now
committed as `docs/37_v1_roadmap.md`. Related: `docs/28` §10, `docs/29`.

### 7.79 An event is restricted to firing once, and a transition cannot act

*Closed 2026-08-30. Phases 1–4 of `docs/34` landed as #235 — rising-edge
firing, no latch, `on enter` and edge actions, model-side augmentation of
pack machines, `results_version` 0.6 — and phase 5 as #236; phase 6's
surfaces (`docs/10` rows, `terminology.toml`) followed. The migration audit
answered itself: no event's condition re-rises anywhere in the corpus, and
across the 123 pre-existing results goldens the only changed line was the
version stamp. What consumed it shipped next — §7.77's cure counter and
§7.76's reserve interest (#238). The residue is not this entry's: the
chained-rollover re-strike showcase is `docs/33` Item 1's case, and it is
what will force `cre.unit`'s declared actions (`docs/34` phase 5 note).*

*Belongs with the language and engine (section 5). Roadmap: M2 (§7.78).
Scoped in `docs/34_events_and_the_machine.md`; found by the Argus parity
survey (`docs/33`, Item 1).*

**What could not be expressed:** an occurrence that recurs, and behavior
performed on arrival. An event is something that happens — time, a default,
a cure, a payment — and nothing about happening is once-only. The shipped
constructs each hold half of this: a guarded edge (the anonymous event,
described by the entity it impacts and the conditions that must be true)
fires every occurrence but arrives empty-handed — no action rides on the
transition; the named `event` carries `set` but latches — the engine skips
a fired event forever (`event_fired`, `crates/cfdl-engine/src/state.rs`).
The construct that repeats cannot act, and the construct that acts cannot
repeat.

**What forced the discovery:** chained rollover, probed pack-free
(`docs/33`). The cycle itself runs — edges re-arm, `state_enter` windows
re-anchor, per-cycle costs re-fire — but a duration-in-state counter cannot
reset on re-entry (the conditional recurrence dies at run:
`prev.<entity>.status is not declared`), and market rent cannot be struck
into a field at an endogenous transition. §7.77's cure window and §7.74's
shortfall bookkeeping are the same absence wearing credit vocabulary.

**The shape** (`docs/34` D1–D8): events fire on each rising edge of their
conditions, with no `once` keyword — a one-shot expresses its once-ness in
a singular schedule or a no-return topology, never a latch; states carry
`on enter` action blocks and edges carry path-specific ones,
entity-relative, run on every arrival whatever took it, under the existing
event-`set` write law and the guard's own environment — no new cycle
risk. Migration is measured, not assumed: a corpus audit counts
re-rising conditions per event before the goldens are re-blessed.

### 7.80 121 registered diagnostic codes have no minimal failing example

The machine docs work (docs/32 Phase 2) measured the register: docs/08 §7
names 197 codes, and only 71 appear in `fixtures/invalid/` + `gold/diag/`.
The repair catalog (`docs/machine/diagnostics-repairs.md`) lists the
uncovered codes by name, so this entry is a work queue, not a survey: each
item is one minimal failing fixture, its blessed golden, and a
compile-verified repair in `fixtures/repairs/`. Retired codes (§8) are
exempt. The catalog's coverage line is the progress meter.

### 7.81 Runtime expression codes are unregistered and load-bearing

`EXPR_EVAL` and `EXPR_UNKNOWN_NAME` are runtime warning codes emitted by
`cfdl-expr`, documented in docs/03 §5, present in results goldens — and
absent from the docs/08 register, whose `E3002`–`E3004` are registered but
never emitted. The engine string-matches `EXPR_UNKNOWN_NAME` in warnings
(`crates/cfdl-engine/src/lib.rs`), so renaming is not a find-replace: it
needs a deliberate pass that reconciles the register with the emitters,
re-blesses the results goldens, and decides whether run-time warning codes
belong in docs/08 at all or in docs/06 beside the results contract.
(`EXPR_PARSE` was the compile-time member of this family and is fixed:
it now emits its registered name `E3001_EXPR_PARSE_ERROR`.)

### 7.82 CFDL-CE tiers are prose; nothing asserts the estate maps to them

docs/22 §2 assigns every published surface to a tier (A–D) with path
globs written in a markdown table. No tool parses that table, so a new
published file lands in no tier and no rule applies to it — the estate's
coverage is whatever `check-site-voice.py` happens to glob. Promoting the
tier table to a machine-readable form (or parsing it as written) and
asserting every published path matches exactly one tier would close the
loop the authoring contract needs. (`ste-allow:` rule ids are now
validated against §3's rule tables; the tier mapping is the remaining
unenforced half.)

### 7.84 The pack machines were drawn before the machine could act

*Closed 2026-08-30 by `docs/36_pack_lifecycle_review.md`, which carries the
survey, the standards each machine was redrawn against, and what changed.*

Found surveying all four packs while implementing arrival actions (§7.79) —
the first work that made a pack's states load-bearing rather than decorative.
Seven machines; three families of defect.

**What shipped.** `credit.loan` and `credit.pool` onto Basel/EBA, IFRS 9, the
GSE loan-level datasets and SIFMA UPM Ch. SF — conditions rather than events
(`defaulted`, `in_foreclosure`), the cure edges the standards require including
the EBA's probation-gated return to performing, and `days_past_due` as a field
because a delinquency bucket is a counter reading and not a regime.
`energy.facility` onto IEEE Std 762 as NERC GADS operationalises it, which
restores the standard's distinction between AVAILABILITY and DISPATCH that
`curtailed` had collapsed, and makes a derate a magnitude rather than a state.
`cre.unit` gains `month_to_month` and loses `downtime` — the same condition as
`vacant`, differing only in the path reached, which an edge's actions now
carry. `cre.property` renames `operating` to `stabilized` and closes the
returning cycle that earns it a machine at all. `opco.enterprise` is unbound:
it encoded a transaction process and a capital structure at once, so the normal
condition of every LBO could not be said.

**One of this entry's three findings was wrong and is withdrawn.** "Declared
states nothing can enter" misread the language: an entity declares its own
opening state, so `predevelopment` on `cre.property` and `warehouse` on
`credit.pool` are reachable as opening states rather than through an edge, and
the declared `initial` should be — and was — the common case.

**What it cost.** Nothing computed. Every pack transition was guard-less, so no
machine fired on its own, and no benchmark gated on a state; the results
goldens moved only on `model_hash` and on state names inside transition
records, and all 43 benchmark cases passed unchanged.

### 7.83 An action kind the engine does not know is journaled, not refused

*Recorded 2026-08-29, while retiring `activate contract` (§7.73).*

The engine's action dispatch ends in a catch-all: a kind it does not recognise
is journaled with outcome `ignored`, noted "unknown action kind", warned into
`deterministic.warnings`, and the run continues with `status: ok`. Only
hand-written IR can carry one, since every kind a model can write is known to
the compiler that wrote it.

**Why it was left alone.** Retiring the contract action was expected to retire
`ignored` with it. It does not: this arm still produces it. Removing the
outcome would have meant a `results_version` bump and 119 results goldens
moving for a change in nothing anyone can observe, so the outcome stayed and
the question was separated from the retirement.

**The question.** `docs/13` §7.71 settled that a defect must not hide behind a
substituted value and a warning nobody reads, and M1's pre-work turned three
such spellings into compile-time refusals. An unrecognised action kind is the
same shape one layer down: the IR asked for something the engine cannot do, and
the run reports success. The alternative is to refuse the IR outright, which
deletes `ignored` from the results schema (`results_version` bumps; every
results golden re-blesses for the version string alone).

What would settle it: whether IR is a surface a third party writes. If it is
only ever compiler output, an unknown kind is a bug and refusing it is right.
If hand-written IR is a supported entry point — the engine's own unit tests use
it, and `docs/32`'s agents may — then tolerating an unknown kind loudly may be
the better contract. `docs/05` does not say which.

Related: §7.71, §7.73 (closed), `docs/28` §8, `docs/06`.

### 7.84 `model.moic` does not compute what its own comment says

*Belongs with the language and engine (section 5).*

The comment above it describes a ratio of cash in to cash out over the life.
The code sums the model's net-POSITIVE periods over its net-NEGATIVE ones.
Those are the same quantity only while no period holds both.

`benchmarks/cre/basic_acquisition_exit_cap` is the minimal case. Its purchase
settles at the open of period 0 and its first year of operations at the close
of the same period, so they net inside it:

```
-1,417,958.33 + 83,077.50 = -1,334,880.83
2,542,954.53 / 1,334,880.83 = 1.905005    model.moic
2,626,032.03 / 1,417,958.33 = 1.851981    published, and what the case asserts
```

Two consequences. The figure moves with the CALENDAR: on a monthly grain the
purchase would sit alone in month 0 and the same deal would read differently,
which is not a property a return should have. And it is not a multiple on
invested capital in any published sense — MOIC partitions by KIND, capital
contributed in the denominator and value returned in the numerator, which is
how A.CRE states it for real estate and how GIPS 2020 defines the fund-level
TVPI it resembles. GIPS is emphatic in the other direction: a distribution that
is recalled *increases* paid-in capital, so the same dollar out and back raises
both sides of the ratio where this fold reduces the denominator.

**This is not a gap in the language, and the remedy is already in it.** A
multiple belongs on the valuation plane, where the model says what it counts as
invested capital rather than the engine guessing from a sign. `metric` folds
once at the horizon and reads series and `model.*`, so the case declares the
multiple in three lines and asserts the published figure exactly. `moic(party.X)`
already does the same job per party in `penzance_highlands`. Nothing needed
adding.

So the question this raises is what `model.moic` should BE, not how to fix it:
whether a whole-model multiple has a defensible meaning at all — for a levered
deal it puts debt proceeds in the numerator and repayment in the denominator —
or whether it should be narrowed, renamed to what it actually computes, or
withdrawn in favour of the declared form.

**`moic(party.X)` folds the same way and is not a counter-example.**
`penzance_highlands` reproduces exactly (baupost 1.959618, penzance 2.906607)
only because its contributions land in periods 0-91 and its single distribution
at period 153, so nothing ever shares a period — verified from its journal, the
overlap is empty. A fund distributing while it is still calling capital would
trip it.

**One existing assertion depends on the current behaviour.**
`penzance_highlands` asserts `model.moic` = 2.04664, and it is the only entry in
that case's `expected_metrics.json` with no `source` line — the party metrics
beside it both cite contributed and distributed. 43 of its 160 periods hold both
a positive and a negative stream flow. Whatever is decided above, that figure
has to be re-derived rather than carried over.

Provenance: found building `basic_acquisition_exit_cap`, 30 August 2026,
against a published equity multiple. Two earlier drafts of this entry
overreached — the first called the party metric structurally different, the
second proposed a taxonomy node and a `cre.acquisition` contract as though the
language could not express the multiple. It can, and does.

---

### 7.85 The valuation plane cannot read what it itself publishes

*Belongs with the language and engine (section 5). Found with §7.86 and §7.87
in one investigation; the three are separable and this is the widest.*

`docs/01` §15.3 is normative: a metric's expression MAY read series, **entity
fields**, `inputs`, `cfg` and the engine's `model.*`. Entity fields it cannot
read. `metric x = asset.proj.drawn` is `EXPR_UNKNOWN_NAME`;
`series_sum("asset.proj.drawn", 11, 11)` returns 0 while the published series
holds 10,000. The metric environment is built from `stream_series` plus
`waterfall_series` and nothing else (`crates/cfdl-engine/src/lib.rs`, the
declared-metrics block), and `bind_states` — called for streams, distributions
and state evaluation alike — is never called there. That is a missing binding,
not a design: the restrictions the block does argue (horizon pinning,
declaration order, folds never counting as cash) are documented at length, and
this one appears nowhere.

The same absence hides every computed aggregate. `entity.<symbol>.net_cash_flow`
is computed, published, and unreachable: a two-loan pool probe reads loan A as
3,600 by stream prefix and 0 by entity aggregate, silently. `domain.*`
subtotals, `entity.*.total`, `run.*` scalars — all dropped, the last because the
scalar binding filters on the `model.` prefix alone.

**Every failure above is a silent zero.** `check_series_names` walks stream
amounts, guards, waterfall sources and field rules — not metric expressions —
so `series_sum("total.nonsense.xyz", 0, 11)` publishes 0 with no warning. The
engine's own stance is that a metric that fails to evaluate is fatal, because
"a missing key reads as 'not run' rather than 'not defined'"; a metric reading
a name nothing binds deserves the same severity, and today gets none.

**This is not a plane boundary, which the entry's first title implied by
calling the published document a "results plane".** `docs/28` §2 names two
planes and only two: the causal plane, and the VALUATION plane — the results
stage, netting, rollups, discounting, metrics and statements alike. Every
name listed above is computed in the valuation plane, published by the
valuation plane, and unreachable from a metric evaluated in that same plane.
A missing binding inside one plane is a worse finding than a boundary
between two, because no rule was being upheld.

**The fix must not recreate the August 2026 naming ambiguity.** Expression
names and published results keys are different dialects (`ops.rev` vs
`stream.ops.rev`), and `docs/03` records what happened when documentation
conflated them: "a model that followed it got an empty pot rather than a
diagnostic." Merging results keys into `env.series` reopens that. A distinct
accessor for the published keys keeps the dialect explicit at the call site
and leaves every existing metric meaning what it meant.

Related: §7.43 (ownership is the other half of reaching results from an
expression), §7.55 (the declaration surface these reads would serve), §7.84
(another figure the valuation plane computes that the engine got wrong first).

Provenance: found probing the metric environment against `docs/01` §15.3,
30 August 2026. Six probe models; every number above reproduced from a run
rather than read from the source.

**Status, 31 August 2026 — shipped, in three pieces.**

1. **Entity fields bind.** `bind_states` is called for the metric environment
   at the horizon, which is the normative §15.3 promise that was simply
   absent. `asset.proj.drawn` in a metric was `EXPR_UNKNOWN_NAME`; it now
   reads the field's value at the horizon, in both spellings.

2. **The published keys bind too, and the two dialects agree.** Every series
   the valuation plane publishes is visible to a metric under the key the
   results document uses: `stream.<name>`, `entity.<symbol>.net_cash_flow`,
   `account.<name>`, a field's own series, a money subtotal, and
   `model.net_cash_flow` — beside the bare expression names, which keep their
   meaning exactly. The entry feared this "reopens the August 2026 naming
   ambiguity" and the measurement says the opposite: the ambiguity IS that
   `ops.rev` read 300 while `stream.ops.rev` read 0, and binding both
   dissolves it. The binding is added to the METRIC environment only — a
   pot's window and a guard's read are untouched, because a metric reads the
   finished projection and they read the walk.

3. **A name nothing publishes is refused** — `E1365_METRIC_UNKNOWN_SERIES`,
   at compile time, walking the ASSEMBLED IR because the vocabulary is the
   whole document (lowered streams, waterfall steps, entity rollups,
   accounts, fields, pack subtotals) and half of it does not exist where
   metrics are read. `series_sum("total.nonsense.xyz", 0, 11)` published 0
   with no diagnostic; it is now a compile error. A `.*` selector may still
   match nothing, because matching nothing is what a selector states at its
   call site.

**One thing deliberately left unbound: a RATIO subtotal.** Its undefined
periods publish as `null` rather than zero — a coverage ratio in a period with
no debt service — so a fold over it must decide what `null` means, and that
decision belongs with the reductions of §7.86. Naming one is refused with its
own hint rather than folded as though `null` were nothing. That is not the old
behaviour: before, it read zero and said nothing.

Fixtures: `valid/metric_reads_published_results` (both field spellings, the
field's own series, the two stream dialects proved equal by a third metric
that subtracts them, the entity rollup and the model aggregate) and
`invalid/metric_unknown_series` (E1365). No golden value moved and all 45
benchmark cases hold — nothing in the corpus was relying on a silent zero.

The vocabulary now exists in two places, the compiler's check and the engine's
binding, and they must agree. Both derive from the same published-series
rules and the two fixtures pin the pairing from both ends; a third place would
be the point to extract it.

---

### 7.86 Sum and mean are the only reductions over a series

*Belongs with the language and engine (section 5). Split from §7.85.*

`series_sum` and `series_avg` are the whole reduction vocabulary. Peak
outstanding debt, maximum drawdown, the period a balance peaked, the first
period DSCR crosses a threshold, a count of breach periods — none is
expressible over a series.

**The trap is that the miss looks like a hit.** `min`/`max`/`sum`/`avg` are
variadic scalar folds, so `max(series_sum("dbt.*", 0, 11))` compiles, runs,
and returns the net lifetime figure — a one-element fold — silently labelled
as a peak. Probed: draws of 6,000 and 4,000 with a 7,000 repayment publish
`peak_naive` = 3,000 against a true peak of 10,000, no diagnostic. The
hand-unrolled alternative (`max` over one cumulative window per period) is
correct and O(horizon) of source text that silently under-measures if the
horizon grows.

**The two workarounds each poison something.** A helper stream carrying a
running balance can read other series in a later wave, but a stream must be
`inflow` or `outflow`, so the helper is cash: the probe corrupted
`model.total` from 3,000 to 78,000 and `model.moic` to 20.5. A field
recurrence is non-cash but its `next` reads no stream series at all
(`docs/14` §3.1), so the schedule must be restated by hand and the two
statements drift.

Shape: `series_max` / `series_min` beside the existing pair — same signature,
same selector dialect, same window semantics, projection-tail rules
unchanged. Position-returning forms (argmax, first crossing) need one design
decision — a period index is trivially comparable, a date is what a covenant
clause names — and belong in the same pass.

Provenance: found asking what a metric could do with a running balance,
30 August 2026; every workaround above was run, not reasoned about.

**Status, 31 August 2026 — shipped, and wider than the entry proposed.** Four
reductions, not two: `series_max`, `series_min`, `series_prod` and
`series_count`, beside the existing pair. Same signature, same selector
dialect, same window semantics, same contexts, projection-tail rules
unchanged.

**The decision the entry did not anticipate: EVERY FOLD READS THE PER-PERIOD
AGGREGATE.** When a selector matches several streams they are added together
within each period first, and the fold runs over that one series. Addition is
associative, so for `series_sum` the order was invisible and the shipped code
flattened stream-by-stream. A maximum is not associative that way: the peak of
the combined position and the largest single cell are different numbers, and
only the first is what "peak outstanding" means. Pinned by a unit test whose
data makes the two answers differ (a cell of 7 in a period whose aggregate is
4). No golden moved, which is the evidence that `series_sum` and `series_avg`
still compute what they computed.

**A selection matching nothing** sums to 0, multiplies to 1 and counts 0.
`series_max`/`series_min` publish NULL — nothing has no maximum, and a zero
there would state a peak no period reached, which is this entry's own lesson
applied to its own fix.

Null rather than an evaluation error, decided after the first shipping and
changed: null is already the language's word for absent (an entity state no
event has set is one; a ratio's undefined period publishes as one), it carries
the guard rails — `null == null` compares while ordering and arithmetic on it
are errors, so an absence cannot quietly become a number — and unlike an error
it leaves a model able to SAY a selector may legitimately be empty:
`if(series_count("x.*", 0, t) == 0, 0, series_max("x.*", 0, t))`. The results
schema has always permitted a null scalar, so this needed no version bump; what
it needed was a `Scalar::Null`, because the catch-all arm was stringifying the
absence as `"null"` and making it look like a value of type text. There is no
`null` LITERAL in the dialect, so emptiness is tested through `series_count`.

**Three outcomes, not two — and collapsing two of them was measured.** A
selection that matched nothing and a window the walk has not reached both used
to arrive at the caller as `None`, so the first attempt at the null change
turned every REFUSED read into a null: a cash-trap guard that had said "series
`ops.noi` is not available in this context" started saying "cannot apply Sub to
number and null". Four goldens caught it inside one run. `SeriesFold` now
distinguishes `NoAnswer` — a fact about the DATA — from `Unavailable`, a fact
about the CONTEXT, which is `docs/28` §4's refusal to clamp a forward read and
must stay an error. A unit test pins both.

**And the entry's headline example needed correcting.** "Peak outstanding
debt" is NOT `series_max` over the debt streams: that is the largest per-period
NET FLOW, a different and also useful question. A peak balance is a fold over
the series that CARRIES the balance — an entity field — which a metric could
not read until §7.85 bound it. The two entries close this together, and the
fixture shows both readings side by side: `series_max("dbt.*")` = 6,000, the
largest flow; `series_max("asset.tlb.balance")` = 10,000, the peak the entry
asked for.

`series_prod` retires a documented workaround rather than duplicating one:
`exp(series_sum(helper, 0, t))` with a helper stream carrying `ln(1 + r_t)`
needs the helper to be `inflow` or `outflow`, so it IS cash, and both `ln` and
`exp` escape to f64. `series_prod` needs no helper and stays decimal.

Fixture: `valid/series_reductions`, which also pins the TRAP — `peak_wrong =
max(series_sum("dbt.*", 0, 3))` and `lifetime` publish the same number, so the
one-element fold cannot come back silently.

**What this does NOT close** is §7.94: a reduction over a TRANSFORMED series
(a count of breach periods, a maximum drawdown), and the position-returning
forms. Both were part of this entry's "same pass" and neither is a reduction —
see the entry for why they separated.

---

### 7.87 A Monte Carlo trial discards every metric but model.npv

*Belongs with the language and engine (section 5).*

Each trial executes a complete deterministic run — journal, streams, every
declared and domain metric. What survives into `trial_summaries` is a map
built fresh with one entry, `model.npv`. The scenario path, one function up,
does it right: `scenario_metrics = scenario_run.metrics` carries the whole
map, which is why §15.3 can promise a declared metric in every scenario
column. The trial loop has `trial_run.metrics` in scope and does not use it.

Consequences, in order of cost. A declared metric gets no distribution — the
figure a case exists to assert exists in no trial. `moic`, `irr`, every
`domain.*` KPI: no distribution. The section-level `MetricSummary` schema
defines p01 through p99; the engine fills mean, stdev, min, max, p50 and
hard-codes the rest `None`. And because per-trial series are (reasonably) not
retained, the metric map is the only window into a trial — whatever was not
declared before the run is unrecoverable after it.

The narrow fix is nearly free: carry `trial_run.metrics` into the summary and
extend the aggregation to every key present, percentiles included. The volume
question that makes per-trial *series* expensive does not arise for scalars.

Related: §7.23 — the scenario plane has the mirror-image gap (metrics but no
per-period series), and a decision about stochastic exports should cover both.

Provenance: found checking the claim "the deterministic results are exported
for each MC trial" against the trial loop, 30 August 2026. The claim is
false today and one line from true.

**Status, 31 August 2026 — shipped, `results_version` 0.9.** The trial loop
carries `trial_run.metrics` into the trial summary, so a trial's record is now
the same metric map the deterministic block publishes: `model.irr`,
`model.moic`, every `stream.*.total` and `entity.*.total`, each `domain.*` KPI
and every metric the model declared. `monte_carlo.metrics` summarises each name
present rather than the one that was hard-coded, and fills p01 through p99 —
the section whose whole subject is dispersion had been declining to state its
tails. Percentiles interpolate linearly between order statistics (R type 7,
Excel's `PERCENTILE`), which at q = 0.5 is exactly the median already
published: every blessed NPV figure is unchanged, and the goldens show the
change as purely additive. `period_distribution` keeps nearest-rank, because a
period is an observation rather than a continuous amount.

Two things the entry did not anticipate, both found by building it. Not every
trial publishes every name — `model.irr` exists only where the flows solve for
a rate — so a summary states `trials`, the count it was taken over, or a mean
over three trials and a mean over five hundred would read identically. And a
name a distribution cannot be taken over (a string, or a kind that changed
between trials) is carried per trial and omitted from the summary rather than
guessed at.

The reach is wider than metrics, because of what shipped beside it: a trial row
keys `entity.<symbol>.total`, and the published entity graph (§7.43, §7.91,
`results_version` 0.7) keys `graph.entities[].symbol` — so a per-entity
distribution is now readable from results alone, on the ownership axis rather
than by inspecting names. Fixtures: `valid/monte_carlo_metric_distribution`
(the declared metric, the IRR, the MoIC and the rolled-up container total, all
distributed) and `valid/monte_carlo_partial_metric` (`model.irr` in 20 trials
of 24, which is what `trials` exists to say). §7.23's mirror-image gap — the
scenario plane publishes metrics but no per-period series — is untouched and
still open.

---

### 7.88 A container is not a kind of asset

*Belongs with the language and engine (section 5).*

`ENTITY_FAMILIES` is closed to `asset` and `party`, and the closure is right —
"the language, not the pack, decides what kinds of thing a model contains."
The roster is one family short. A fund, a portfolio, an SPV, a transaction is
a grouping that *scopes* cash, not a thing that produces or consumes it.
Modelling one as an `asset` with `part_of` children types it falsely, and the
falsehood is load-bearing: `Asset.Financial` claims "a claim on cash," which a
portfolio is not, and every validation built on families inherits the lie.

Shape: a `container` family in the language base, with core types the platform
layer above already specifies (Transaction, Portfolio, Fund, SPV) as pack- or
base-supplied subtypes. `contains` already exists as the inverse of `part_of`;
a container adds `container -> asset` and plausibly `container -> contract`
edges. The rollup machinery is indifferent —
`entity.<symbol>.net_cash_flow` follows `part_of` today and would follow a
container edge identically — so the engine change is small; the change is to
what a model may *say*.

This is the standalone fraction of "model linking" (deferred past v1): one
model, many assets, fund-level cash and fund-level metrics, no cross-model
plumbing. What it does not cover — one model consuming another's published
results — stays deferred.

**Sequencing note: families are a closed vocabulary and results keys embed the
family** (`<family>.<entity>.<field>`). Adding a family after 1.0 is a
breaking change to every consumer that switches on it; adding it before is
additive. This belongs in the release candidate, not after it.

Provenance: raised comparing the language base against the platform ontology
specification above it, 30 August 2026.

---

**Addendum, 30 August 2026 — this is a restoration, not an addition.** The
comment above `ENTITY_FAMILIES` declares "FOUR FAMILIES, fixed here": asset,
party, contract, reference. The constant beneath it implements two. Contract
and reference are already first-class rosters in the ontology
(`OntologyContract`, `OntologyReference`) with their own declaration keywords
— what was never finished is treating them as NODE families: identity-bearing,
valid endpoints for relations. So the entry's real shape is: restore the
roster to its own comment, add `container` as the fifth, and unify the GRAPH
while leaving the syntax per-kind (`entity` declares asset/party/container;
`contract` declares contracts; `curve`/`quantile` declare references).

**Status, 30 August 2026 — shipped in cfdl-pack.** `ENTITY_FAMILIES` is
asset/party/container; `NODE_FAMILIES` (asset, party, container, contract,
reference) is the new superset relations validate against — the graph
unified, the syntax per-kind, exactly as the addendum below specifies. Four
container base types ship (`Container.Fund`/`Portfolio`/`SPV`/`Transaction`),
`part_of` and `owns` endpoints widened to include containers (endpoints now
accept one family or a list; every pre-widening pack file still parses). The
engine needed nothing: `entity container fund` already compiled — the model
namespace was never family-gated — and the rollup already follows `parent`
regardless of family, verified with a probe whose container aggregated its
child's cash. The Portfolio migration landed
31 August 2026: `CRE.Container.Portfolio` and `Energy.Container.Portfolio`
(renamed — "Asset" in a container's type_id would be incoherent), both
penzance models re-declared (`entity container project`, every
`asset.project` reference moved with it), economics identical — 45/45
benchmarks. The migration settled a design point the models forced: a
container MAY carry directly-attached cash (penzance hangs land and
development costs on the project), so "does not produce" softened to
"deal-level cash is real cash" in docs/01 and docs/07. What remains: deciding whether a model-level `entity` namespace should be
validated against `ENTITY_FAMILIES` at all — today `entity carpark x` is
legal and silently untyped, which is a finding of this work, not a change
it made.

**`part_of` is untouched, and containment reuses it.** Unit-in-building and
loan-in-pool are asset→asset hierarchy and stay exactly as they are. A
container's containment is the same relation with widened endpoint families
(`container → asset`, plausibly `container → contract`), not a parallel edge:
one hierarchy concept, and `contains` is already its registered inverse. The
rollup machinery follows the relation and is indifferent to the family.

---

### 7.89 Two relations are not a relation vocabulary

*Belongs with the language and engine (section 5). Pairs with §7.88.*

The language base declares `part_of` and `owns`. The machinery around them is
complete — cardinality, inverse names, per-pack extension, the CRE pack adds
`occupies` and `manages` — but the base vocabulary stops before the relations
deal models actually turn on:

- `secured_by` (contract -> asset): collateral. Loans and the assets securing
  them are both modelled today with no way to bind one to the other, so LTV
  is a hand-paired input, a release provision has no structure to read, and
  nothing can validate that a mortgage names its property.
- `guarantees` (party -> contract): the guarantee obligation recourse
  analysis needs.
- `is_counterparty_to` (party -> contract): who is on the other side —
  today recoverable only by reading a contract's terms.

First increment: declarative only. The relations exist, are validated
(endpoint families, cardinality), and are published; no engine semantics
change. That alone unlocks the "search-around" selection pattern the
ontology's inspiration (Palantir's object sets) treats as primary: start at a
party, traverse `guarantees` to contracts, `secured_by` to assets, and name
the resulting cash — which is what "isolate one artist's royalties" or "one
guarantor's exposure" actually is. Whether any relation later acquires engine
semantics (does `secured_by` feed a recovery calculation?) is a separate
decision per relation.

Related: §7.43 — relational selection over results requires results to carry
the graph; publishing ownership is the first edge of that.

Provenance: raised comparing the language base against the platform ontology
specification above it, 30 August 2026.

---

**Status, 30 August 2026 — shipped with §7.88.** `secured_by`
(contract→asset), `guarantees` (party→contract) and `is_counterparty_to`
(party→contract) are in the language base, declarative as specified —
validated, published with the ontology, no engine semantics. They typecheck
because relation endpoints now range over `NODE_FAMILIES`, which is the
contract-as-node dependency the addendum below records.

**Addendum, 30 August 2026 — depends on §7.88's restoration.** `guarantees`
and `is_counterparty_to` are party→CONTRACT edges. They can only typecheck
once a contract is a node family, which is §7.88's graph unification. The two
entries are one change wearing two numbers, and should land together.

---

### 7.90 A slice: selection with a name, and no pretence of completeness

*Belongs with the language and engine (section 5). Related: §7.55, §7.43.*

A statement's defining property is completeness — every category in exactly
one line row, a reconciliation block, a `residual` row for cash nothing
claimed, `E5029` for cash outside every fold. The complementary thing has no
name and no surface: a *deliberately partial* selection — one loan out of a
pool, one artist's royalties, the portfolio with a product line removed — with
metrics computed over the selection.

Two design commitments, both load-bearing:

**A slice must not inherit the reconciliation machinery.** A filtered total
that publishes a residual invites reading a partial number as a complete one.
The absence of the reconciliation block is what the declaration *means*; it is
the difference between a slice and a statement, and the reason "a statement
with a filter" is the wrong construction.

**A slice is a selection, not a copy.** It names entities, categories,
relations (once §7.89 lands) or stream patterns; everything computed over it
carries the selection in its lineage the way a metric carries its formula.
The precedent is Palantir's object set — a saved, composable, named selection
that functions and views consume — which is the concept the platform layer's
"ontology slice" already borrows for packages; this brings the same idea to
what the valuation plane publishes. The EVS spelling ("slice: a subset of the graph relevant
to a specific valuation... portable and self-contained") is the right one and
the word should be registered in `docs/terminology.toml` beside `statement`
and `metric` when this lands — noting that `subtotal` and `category`, both
load-bearing, were never registered at all.

Depends on: §7.43 (results must attribute streams to entities before a slice
can select on them), §7.88/§7.89 (the vocabulary worth selecting with).
Category- and pattern-scoped slices are expressible with nothing else landing
first.

Provenance: raised working out why "remove certain products and recompute" has
no home in the language, 30 August 2026. The naming (`slice`, not `view`) was
settled against the platform vocabulary the same day.

**Status, 31 August 2026 — shipped** (docs/01 §15.4, normative). `slice` is
the 87th reserved word; clause kinds intersect, values within a kind union,
excepts subtract; entities are references selecting their `part of`
descendants (a container's slice is its members'); `type` matches
transitively through the recorded refinement, expanded at compile because
the engine is pack-free; category and stream selectors are quoted — one
dialect. Results publish selection lineage, matched streams (empty
published, not omitted), net series, and total/npv/irr — and no
reconciliation block, exactly as this entry demanded. Fixtures:
`valid/slices` (intersection, container scope, except — 420/300/510 pinned),
`valid/slice_by_type` (`Contract.Debt` expanding through
CRE.Contract.PermanentDebt to the three lowered debt streams), four invalid
fixtures for E1361–E1364. results_version 0.8.

---

### 7.91 An entity may carry a stable identity, and results repeat it

*Belongs with the language and engine (section 5). Related: §7.43, §7.88–§7.90.*

An entity is a symbol scoped to one model. The governance layer above the
language assigns canonical identifiers to real-world things — the same
building, borrower or fund referenced across many deals — and its identity
contract reads "CFDL references those IDs, never invents ambiguous entities."
That contract has no CFDL half: an entity declaration has nowhere to carry an
external identifier, and results publish none, so a consumer joining two
packages on "the same asset" is joining on symbol names and hope.

Shape: an optional `id "<opaque string>"` on an entity declaration. The engine
ignores it entirely — no semantics, no resolution, no network anything.
Validation is uniqueness within the model and nothing else, because the
language cannot know what the string means and must not pretend to. It rides
the IR with the entity's provenance and is published in results wherever the
entity's symbol appears, which today means beside the rollup keys and — once
§7.43 lands — beside each stream's owner.

The standalone cost is near zero; that is the point. A model that carries no
ids loses nothing. A layer above that assigns them gets the one hook it needs
to make a package's numbers attributable to canonical things, without the
language growing an opinion about identity.

**Sequencing: pre-1.0, for the same reason as §7.88.** Adding an optional
field is additive; retrofitting identity into published results after
consumers exist forces a version switch on all of them. Reserving the
declaration surface now costs one optional token.

Provenance: raised from the platform layer's identity-contract gap analysis
(H.3), 30 August 2026, which found the binding missing on both sides —
and scoped here to the half the language can supply alone.

**Status, 31 August 2026 — shipped.** The literal field `id` is the
carrier: engine-opaque, unique within the model (`E1360`, with the
join-would-merge reasoning in the hint), republished per entity in the
results graph. `fixtures/valid/stable_identity` pins the round trip;
`invalid/duplicate_entity_id` pins the refusal.

---

### 7.93 Every engine run failure reports as an IR schema violation

*Belongs with the CLI and diagnostics (section 5). Found shipping §7.85.*

`EngineError` has variants for genuinely different failures — an unresolved
name, a metric that does not compile, a metric folding a series nothing
publishes — and `crates/cfdl-cli/src/main.rs` maps every one of them to
`E5002_IR_SCHEMA_VALIDATION_FAILED` at three call sites. So a run that failed
because a metric named a series wrongly told the author its IR violated the
published schema, which it did not: the IR was valid, the compiler wrote it,
and `check-ir-schema` would have passed it.

Measured while building §7.85's refusal:

```
ERROR[E5002_IR_SCHEMA_VALIDATION_FAILED] Run failed while reading IR
'ir.json': unresolved name: Metric 'nonsense' names series
'total.nonsense.xyz', which this run does not publish.
```

The message underneath is precise and the code above it is false, which is the
worst arrangement: a reader who trusts codes over prose goes looking at the
schema, and a tool that routes on the code routes wrongly. §7.85's own check
moved to compile time and got a real code (`E1365`), so the example above no
longer reproduces from that path — the mis-mapping is untouched and every
other engine failure still goes through it.

Shape: `EngineError` variants map to distinct runtime codes, registered in
`docs/08` the way the compiler's are, with `E5002` kept for what it names —
an IR that genuinely fails the schema. The register already carries runtime
codes (§7.81 is the sibling entry: `EXPR_EVAL` and `EXPR_UNKNOWN_NAME` are
emitted and unregistered), so the two should be settled in one pass.

Provenance: found reading the CLI's error mapping while checking that
§7.85's new refusal surfaced legibly, 31 August 2026.

---

### 7.94 A reduction reads a series, never a transformed one — and cannot say WHERE

*Belongs with the language and engine (section 5). Split from §7.86 when its
four reductions shipped and these two did not.*

`series_max` answers "what was the peak". Two neighbouring questions it does
not answer, and neither is a reduction:

**1. A reduction over a TRANSFORMED series.** "How many periods was DSCR below
1.20", "what was the maximum drawdown", "the sum of the absolute movements" —
each folds a series that does not exist. `series_count(name, from, to)` counts
periods whose aggregate is non-zero, which is a real question and not this one;
the covenant question needs a per-period predicate, and there is nowhere to put
it. Every reduction takes a series NAME — a text selector — so no expression
can sit between the series and the fold.

Three shapes, in rising order of language cost:

- **A predicate argument**: `series_count_if(name, from, to, "<", 1.20)`,
  passing a comparison operator as a STRING. Cheap and ugly, and unlike
  anything else in the language.
- **A predicate expression**, which means lambdas or first-class expressions.
  Out of scope; the language has no construct that takes one.
- **A DECLARED per-period line**, which is §7.55 — a model cannot declare a
  subtotal, and a field's `next` reads no stream series (`docs/14` §3.1), so
  there is no legal place to compute an indicator. If §7.55 shipped, an
  indicator line declared once and `series_count` over it answers the covenant
  question WITH NO NEW SYNTAX AT ALL.

That last is the reason this entry exists rather than a `series_count_if`: the
missing thing is not a reduction, it is the line to reduce. **Do not build the
predicate argument before §7.55 is decided.**

**2. WHERE, not what.** `series_argmax`, and "the first period DSCR crossed
1.20". Three decisions, which is why it did not ride along with §7.86:

- **The return type.** A period index composes with the windows every
  reduction already takes; a DATE is what a covenant clause names. And the
  registry has no `period -> date` function, so an index cannot be turned into
  the date the clause wants — whichever is returned, the other needs a second
  function built beside it.
- **Ties.** First or last occurrence of the maximum. First is conventional and
  should be stated rather than emergent.
- **Nothing to point at.** An empty selection has no position, the same
  argument that makes `series_max` refuse it — so these inherit that refusal.

A first-crossing form additionally needs the predicate of part 1, so the two
halves of this entry are not independent: settle §7.55, and both get simpler.

**Why the position-returning forms are not urgent.** Results publish the full
per-period series in `deterministic.series`, so an analyst holding results has
everything argmax and first-crossing need and can take them in pandas. That is
a reason to sequence them late, not a reason they are unnecessary: a figure
computed outside the model is not asserted by the model, carries no lineage,
and cannot appear in a scenario column or a Monte Carlo distribution. Recorded
so a later reader knows this was decided rather than overlooked (31 August
2026).

**Also unbuilt, and related: a cumulative scan.** A peak balance is a fold over
a series that CARRIES the balance, and §7.86's fixture shows that working
because the model declares the balance as a field. A model that has only flows
cannot synthesise the running total to fold — that is a scan (a series in, a
series out), not a reduction, and it is the same missing capability as part 1
seen from another side.

Provenance: split out of §7.86 on 31 August 2026, when its four reductions
shipped and these did not. The `period -> date` gap and the §7.55 dependency
were both found while scoping that work, not before it.

---

### 7.95 Undefined is not zero, and a series cannot say so

*Belongs with the language and engine (section 5). The design is SETTLED below
and not built; §7.85 deferred it and §7.86 sharpened it.*

A ratio subtotal publishes `null` for the periods where it is genuinely
undefined — a coverage ratio in a period with no debt service — and no
reduction can fold it. `E1365` refuses the name with a hint saying why, which
is honest and not an answer: `series_max("domain.dscr", 0, 11)` is the covenant
question, and the covenant question is the reason ratios exist.

**The cause is the representation.** A metric's visible series are
`BTreeMap<String, Vec<f64>>`, in which "undefined" has no spelling. Binding a
ratio there would have to write SOMETHING in the undefined periods, and every
candidate is a lie: 0 is a value the ratio never had, and it is the exact
failure §7.86 exists to end.

**Two things look like "missing" and are not the same thing.** Conflating them
is the trap this entry exists to avoid, and §7.86 already paid once for
conflating a neighbouring pair:

- **Past the end of the data** — the window runs into the projection tail or
  past a short series. The CELL DOES NOT EXIST. `series_avg` pads here: the
  numerator sums the cells that exist and the divisor is the REQUESTED window,
  so a window past the data averages over the full window. Shipped,
  deliberate, documented, and staying.
- **Genuinely undefined** — the period exists and the quantity does not. This
  is what a ratio's `null` says, and nothing handles it.

**The settled design:**

1. **Bind ratio series in the METRIC environment only**, as an optional-valued
   series. The causal plane reads `env.series` too — streams, guards, field
   rules — and has no ratio subtotals to read, so widening it would take the
   blast radius for no gain. Narrow first; widen if a document forces it.
2. **Every fold SKIPS the undefined periods.** They are not observations.
   `series_max`/`series_min` over the defined ones, `series_sum` adds them,
   `series_prod` multiplies them, `series_count` counts the defined non-zero.
3. **`series_avg`'s divisor counts the periods it actually folded.** This
   sounds like a change to the shipped rule and is not one: a CASH series has
   no undefined periods, so its divisor is the requested window exactly as
   today. The rule follows the SERIES, not the function, and that sentence has
   to be in the spec or it will be rediscovered as a bug.
4. **An all-undefined window** gives null for max, min and avg, and 0 for sum
   and count. A mean of nothing is not zero.

Note what this inherits: §7.86 already made `series_max` publish null for an
empty selection, so the value shape and the `Scalar::Null` publication exist
and the results schema already permits them. What remains is the
representation and the skip rule.

Related: §7.86 (the reductions), §7.85 (which bound everything else a metric
can read), §7.94 (the transformed-series reductions, which need this decided
first — a breach indicator over a ratio is exactly a series with undefined
periods).

Provenance: deferred out of §7.85 on 31 August 2026, sharpened while building
§7.86's four reductions, and settled the same day rather than left as an open
question — the decision is cheap to record now and expensive to re-derive.

### 7.96 A party owns at most one account

*Belongs with the language and engine (section 5). Found converting
`benchmarks/credit/auto_abs_tranches` onto accounts, 2 September 2026.*

**What could not be expressed:** a noteholder's two positions. A class of
notes has a principal position — what has been repaid, which is what its
remaining claim is stated against — and an interest position, what it has
earned. Both are cash allocated to the same party, and `docs/01` §10.6 says a
party owns at most one account, so `pay a2_interest to party.a2_holders` and
`pay a2_principal to party.a2_holders` land in one balance and the class's
claim, `face − principal repaid`, cannot be read from it.

**What forced the discovery:** the case's principal steps read
`prev.<class>_principal` as the class's cumulative repayment. The interest
steps had to go somewhere else, and the only spelling the rule allows is a
STRUCTURE-owned account per class (`account a2_interest { from 0.0 }`, paid
by `to account`), which records the cash correctly and attributes it to
nobody: the holder's `entity.party.*.net_cash_flow` carries principal only,
and the interest a class earned is visible in an account that no party owns.
Seven such accounts in one model, each a workaround for one sentence.

**Why the rule exists, and why it is the wrong rule.** §10.6 keeps "their
account" resolvable: `pay <step> to <party>` lands in the party's account
without naming it, and with two the destination is ambiguous. That is a
reason to require the explicit form when a party owns more than one, not to
forbid the second account. A party with several positions is the ordinary
case in every structured deal — principal and interest on a note, capital and
preferred return on a partnership interest, a lender's advances and its
recoveries — and each is a claim the waterfall pays separately.

**The shape.** Lift the limit: a party MAY own several accounts. `pay <step>
to <party>` keeps its meaning while the party owns exactly one; when it owns
more, the bare form is refused at compile with the accounts named, and the
step says `to account <name>`. Party-level returns (`irr(party.x)`,
`moic(party.x)`, `entity.party.x.net_cash_flow`) fold across every account
the party owns, which is what they mean today with one. Nothing else moves:
the balance law, `prev.<account>`, and the journal are per account already.

Related: §7.76 (the account adoption pass, whose reserve was the first
account), `docs/28` §5.1 (where the one-account rule is stated as a
resolution convenience), `docs/17` §13.

### 7.97 A field that reads a waterfall step reads zero, in silence

*Belongs with the language and engine (section 5). Found by a probe during
the benchmark review, 2 September 2026, and the reason the review's
"balance a waterfall reduces by paying it" framing was withdrawn.*

**What happens.** A field recurrence reading a waterfall step's series at the
previous period —

```cfdl
entity asset trust : Asset.Financial {
  bal init 1000.0 next prev - series_sum("dist.principal", time.t - 1, time.t - 1)
}
```

— compiles clean, runs with no warning, and reads zero every period. In the
probe the balance never moved and a step capped at `min(remaining,
asset.trust.bal)` paid 1,800 of collections against a 1,000 balance. The
walk's read table (`docs/28` §4) makes a field's read of series at `t − 1`
legal, and `fixtures/valid/recurrence_reads_settled_cash` pins it for a
STREAM's series; a waterfall step's series is not available to the causal
plane at all, and nothing says so.

**Why this is a capability entry and not only a defect.** The silence is the
same class as §7.38 and §7.95, and the refusal that closes it exists for the
neighbouring reader: `E1346_STREAM_READS_WATERFALL_STEP` refuses a STREAM
that names a step, on the stated ground that every waterfall runs after every
stream. A field's rule has the same relationship to a waterfall and no such
check. What the probe settled beyond the diagnostic is the modeling rule the
benchmark programme now carries: **a waterfall never influences or updates a
balance in the causal plane.** What a waterfall does is allocate cash to
parties, whose ACCOUNTS hold their claims; a class's remaining claim is
`face − principal in its holder's account`, read as `prev.<account>`, and
every structural test — an overcollateralization target, a step-down, a
turbo — is an expression over accounts and pool state. Under that rule there
is no reason for a field to read a step, and the read should be refused the
way `E1346` refuses it for a stream.

**The shape.** Extend `E1346` (or a sibling) to field rules, event guards and
account inflows: a `series_sum`/`series_avg` naming a waterfall step in any
causal-plane reader is refused at compile with the step named. Then retire
the framing this entry replaces: `docs/17` §5's "a balance a waterfall
reduces by paying it", `docs/26` "A liability stack" (the paragraph that
says a diverging distribution forces a balance field), and
`benchmarks/credit/americredit_2017_1/NOTES.md` "Why the waterfall reads a
field" — each of which asks for a balance the waterfall writes, and each of
which is answered by the holder's account.

Related: §7.38, §7.95, §7.74, `docs/28` §4 and §5.1, `docs/17` §5 and §13.
`benchmarks/credit/auto_abs_tranches` is the first case written under the
rule: no class carries a balance, and the published grid is asserted as the
holders' account balances.

### 7.98 The remaining pool balance is not readable in the causal plane

*Belongs with the language and the credit pack (sections 2 and 5). Found
converting `benchmarks/credit/auto_abs_tranches`, 2 September 2026; the same
absence made `americredit_2017_1` restate its pool as a closed form.*

**What could not be expressed:** "what the trust still owns". A container's
`part of` relation folds its members' CASH — `entity.container.trust.
net_cash_flow` — and folds nothing else. The credit pack lowers each pool's
interest, principal, prepayments and servicing as streams and carries one
piece of state, the surviving fraction, but publishes no balance series per
pool; the balance is inside every rule's closed form. The reporting plane has
it — `domain.credit.balance_outstanding` and `domain.credit.pool_factor` are
statement subtotals a reader can see — and a guard, a field or a step cannot
read `domain.*`, because a subtotal is a fold over the settled ledger and
inside the walk the ledger has not settled (`docs/01` §13.1).

**What forced the discovery.** `auto_abs_tranches` does not need the balance:
a no-loss sequential deal tests nothing against the pool. The next deal shape
does, and the one case that has it shows the cost: AmeriCredit's
overcollateralization target and clean-up call both read the pool balance the
trust carried in, and the trust restates it as `pool_bal`/`pool_prior` — a
closed form summing twelve pools' amortization arithmetic, written a second
time beside the contracts that already carry it. Every trust with a target,
a trigger or a call will need the same field, and every one will restate the
pack.

**The shape**, in two halves that are separately useful — **the first is
built for `credit.pool_level_pay`** (4 September 2026: `credit_level_pay_
balance_<instance>` fills the `balance` role and every stream reads it; the
interest-only families follow), the second is open:

1. **The pack publishes the balance as a field.** `credit.pool_level_pay` and
   its siblings declare a rule field `credit_<family>_balance{{contract.
   suffix_ident}}` beside the survival fraction — the opening balance the
   rules already compute, carried as state so a guard can read
   `asset.p01.credit_level_pay_balance_p01` by declared name (the discipline
   `docs/28` §4 and the withdrawn selector-validation entry both land on: a
   guard reads a name, never a pattern).
2. **A container can fold its members' fields.** `part of` today gives the
   parent its members' cash; a parent should be able to declare a field
   that is the relation's fold of a member field — `pool_balance = sum of
   parts' credit_level_pay_balance` — rather than twelve `prev.asset.pNN.…`
   reads written by hand. That is the same request as reading the relation
   the run already publishes as `graph.entities`, stated for the one query
   every SPV asks: what do I hold, and how much of it is left.

Until both land, a trust that needs its balance in a guard materializes it
as a field summing named member fields, which is what AmeriCredit does and
what `auto_abs_tranches`' NOTES.md records as the gap the clean-up call will
meet.

Related: §7.88 (a container is not a kind of asset), §7.89 (the relation
vocabulary), §7.74 (Intex scope — every trigger there reads a pool balance),
§7.97 (why the balance must come from the collateral and never from the
waterfall).

### 7.99 A `reference` names an external series and cannot reach one

*Belongs with the language and engine (section 5), and with the ontology.*

The `reference` family is the fourth entity family and every pack declares
members of it — `energy.power_price` (kind `price_curve`, USD/MWh),
`credit.base_rate`, `energy.inflation`. A model may cite one from a `quantile`
through its `ref` clause, and `required_refs` records the citation. That is the
whole of what a reference does today: it is an identifier, and citing it buys
provenance.

**The sister repo specifies more, and the missing half is the half that
matters.** `evs-platform/docs/03_registries_specification.md` splits the concept
in two:

- an **Observable Registry**, which "defines the external data series a domain
  model may reference" — `Rates.SOFR.1M`, `Index.CPI`, `FX.USD.EUR`, each a time
  series;
- a **Binding Registry**, which "maps observable IDs to their data sources or
  snapshot columns", declaring for each one a "target: snapshot column, run
  config key, or connector endpoint" and a "fallback behavior when the
  observable is unavailable".

CFDL shipped the first and none of the second. A reference declares that a
series EXISTS and can be cited; nothing binds it to a source, so nothing
resolves it to values.

**Three consequences, each visible in a shipped artefact.**

`obs` is a scalar. `fixtures/valid/obs_smoke` reads `obs.rate` and the run
configuration supplies `"obs.rate": 1000.0` — one number. EVS resolved its v0.2
syntax as `obs.rate("SOFR")`, a FUNCTION returning the series, and records the
runtime question as open and High: "Observable binding hydration at runtime (who
provides values: run config? snapshot? connector?)" (`09_inconsistencies_and_gaps.md`,
item 26). A scalar cannot carry a rate curve, let alone a price year.

`curve` cannot cite a reference at all. Compare the two grammar productions:

```
curve_stmt    = "curve" IDENT [ curve_interp ] "{" curve_point … "}" ;
quantile_stmt = "quantile" IDENT [ quantile_interp ] [ quantile_order ]
                [ "ref" qname ] "{" quantile_point … "}" ;
```

The two constructs are deliberate counterparts — level by date, dispersion by
probability (§7.1) — and only one can name where its numbers came from. A
`curve` is exactly the construct an external series would arrive in, and it is
the one with no provenance.

So a model carries market data as literals. `benchmarks/energy/merchant_storage_arbitrage`
declares 730 curve points across two curves, generated by its own
`reference_gen.py` and imported from `prices.cfdl` to keep the model legible.
The import is the right shape for what exists; it is not a binding, and nothing
in the results says which observable those numbers are.

**What this is not.** Not a request for a connector, and not for CFDL to fetch
anything. The narrow version is that a declared reference should be resolvable
to a series through a stated binding — at minimum a file or a run-configuration
key — and that `curve` should be able to name the reference it carries, the way
`quantile` already does.

Provenance: found writing `merchant_storage_arbitrage`, whose market input is
730 literal points that no results document can attribute. The desired shape was
then read from `evs-platform/docs/03_registries_specification.md` rather than
inferred. Related: §7.1, `docs/27` §4.4 (what `ref` buys), and EVS question 26.

### 7.100 A curve extrapolates flat past both ends, in silence

*To investigate. The behavior is defensible for the construct's original use and
wrong for another it has acquired; what follows is the evidence, not a proposed
fix.*

Belongs with §5, language and engine.

`curve_value` outside a curve's declared range returns the nearest endpoint.
Probed directly against a three-point table:

```
curve tbl { 2026-01: 10.0, 2027-01: 20.0, 2028-01: 30.0 }
```

| read over | values returned |
|---|---|
| 2026–2031 | 10, 20, 30, **30, 30, 30** |
| 2024–2029 | **10, 10,** 10, 20, 30, **30** |

Flat both directions, with no diagnostic at compile time or run time.

**The deal outruns the curve; the curve never outruns the deal.** Nothing is
evaluated outside the model timeline — `E2103_SCHEDULE_OUT_OF_BOUNDS` refuses a
stream whose schedule extends past it — and a curve with more points than the
horizon simply leaves them unread (a six-point table on a three-period deal
returns `[10, 20, 30]`). So every read above is *inside* the deal timeline, and
the only way to reach the flat tail is to declare a curve shorter than the
horizon that reads it. That is the whole of the exposure, and it is worth
stating because it names the check that is missing: the engine already
bounds-checks a schedule against the timeline, and does not bounds-check a
curve read against the curve.

**Why the behavior is right, and why it is also wrong.** A curve is the construct market
data arrives in, and flat-forward extrapolation is the standard convention for a
price curve — nobody wants a forward rate to fall off a cliff at the last quoted
tenor. But a curve is also the natural home for a *schedule*: a depreciation
table, a step-down fee, any finite series of stated allowances. A schedule that
silently repeats its final entry forever is not a convention, it is a wrong
answer, and the two uses want opposite behavior from the same call.

**How it was found, which is the part worth keeping.** Writing a closed-form
after-tax solve, the MACRS five-year table was declared as a curve and read by a
tax stream running the full 25-year horizon. The allowance continued at 5.76%
for nineteen years past the schedule's end, and the run reported a confident
`model.npv` of 260,805.84. Nothing flagged it. It was caught only because the
closed form independently predicted what net present value should be, and the
discrepancy was exactly the extra depreciation — without that second computation
the number would have looked entirely reasonable.

The fix used in that model was to let the stream's `schedule` end the deduction,
which works and is arguably the better spelling anyway. That is a workaround
available to a modeler who already knows the behavior.

**What this is not.** Not a request to make extrapolation an error — that would
break every price curve in the suite. The narrow questions are whether a curve
should be able to *declare* that it ends (so a read past it is a diagnostic
rather than a repeat), and whether the two uses are actually one construct.

Provenance: found 2026-09-03 building a closed-form tariff solve against
`benchmarks/energy/crest_solar_cost_based`. Related: §7.99 (a curve cannot cite
its source), §7.95 (undefined is not zero) — the same shape of silence.

### 7.101 A stream that folds a field reads zero, in silence

*To investigate, and the block may well be correct. The silence is the part that
is not.*

Belongs with §5, language and engine.

A field is CFDL's non-cash computed series: its values publish as a series and
stay out of `model.net_cash_flow`. Confirmed on a two-field probe, where
`asset.p.disc_cost` published `[12000.0, 11111.1, 10288.1, 9526.0, 8820.4]`
while net cash flow carried only the model's actual outflow.

The same fold, from two readers:

| reader | `series_sum("asset.p.disc_cost", 0, 4)` |
|---|---|
| a metric | **51745.52** |
| a stream | **0.0** |

`series_sum` from a stream selects streams. A field pattern matches nothing, and
nothing is reported — the stream evaluated, produced zero, and the model ran to
completion.

**Why the block is probably right.** A stream folding other streams over a whole
horizon is already supported and correct: dependency-ordered waves (#144) settle
an acyclic fold target first, and a stream is not state. A *field* is state, and
`docs/28` §4 makes state reads strictly backward. Folding a field forward from
period 0 would read state that has not settled. So this is not the causal plane
failing to reach something it should — it is very likely the backward-only rule
holding exactly as designed, and any entry framed as "let a stream see a field's
future" would be wrong at the premise in the way §7.100 (closed) was.

**What is left after conceding that.** The diagnosis costs nothing and the
silence costs a wrong number. A selector that matches no series is the same
defect as §7.38: a pattern that resolves to nothing should say so, particularly
when the identical text in a neighboring construct resolves to a real value.
A modeler who writes this has made a plane error and gets a plausible zero.

**What this is not.** Not a request for a new construct, and not urgent. Where
the target is linear in the unknown, the whole need disappears — a rate solve
can be written in closed form with no helper series at all, which is how the
CREST tariff solve was ultimately expressed. See the note on that below.

Provenance: found 2026-09-03 probing whether a model can carry a computed series
that is not cash, chasing a discounted helper for a tariff solve. The
investigation also refuted an earlier claim of mine that horizon-wide folds are
unreachable from streams; they are reachable over streams, and the probe pair
above is what distinguishes the two cases. Related: §7.38, §7.94, §7.95.

### 7.102 A field cannot fold a stream "since my last step"

Belongs with §5, language and engine. Found 4 September 2026 building the
level-pay pool's balance.

The natural row for a balance is `closing = opening − what the streams paid`:
a field whose `next` reads the prior period's principal streams, which a
field may do (`fixtures/valid/recurrence_reads_settled_cash`). On a monthly
book it works. On a daily book with monthly payments it does not: the field
steps on payment dates, the streams strike on the same dates, and
`series_sum(…, time.t − 1, time.t − 1)` reads the prior DAY, which is zero.
A window in model periods cannot be written in the rule, because the rule
knows its payment frequency and not the model's calendar, and a month is
28 to 31 days. So the balance is rolled forward from the RATES instead —
opening × (1 − principal fraction) × (1 − mdr − smm), the same row written
in terms of the schedule rather than the cells — which reproduces the
closed form exactly and restates the hazard fragments the fragment gate
already polices.

**The ask:** a fold bounded by the reader's own cadence — the stream's
values since the field last stepped — so a stream-driven balance is
expressible at every cadence. It is what a loan-level pool will want, where
the reductions are actual payments and there is no rate to roll forward
from. Related: §7.98, §7.101 (a stream cannot fold a field), `docs/28` §4.

### 7.103 A division by zero inside a field's recurrence aborts the run

Belongs with §5, engine. Found 4 September 2026.

The balance recurrence calls `pmt(r, n − p, 1)` for the payments left; when
the contract's `term` runs past its `term_months` the field keeps stepping,
`n − p` reaches zero, and the engine panicked — a `rust_decimal` division by
zero out of `annuity`, with no diagnostic, no period, no field named. The
rule now guards at maturity, but a modeler's own recurrence can do the same
thing and gets a stack trace. A runtime arithmetic failure inside the walk
should surface as a diagnostic naming the field, the period and the
expression, as `E5020` does for a recurrence that fails to parse.

### 7.104 The pool's amortization schedule and its accrued interest can disagree

Belongs with §2, credit pack. Found 4 September 2026 reading the level-pay
rules against `pack_amortization_day_count`.

The balance amortizes on the annuity factor at the AMORTIZATION divisor
(`{{model.amortization_divisor}}`, 30/360 by default) while the scheduled
principal stream is the level payment less interest at the ACCRUAL divisor.
When the two differ (an `act/360` accrual on a 30/360 amortization) the
principal the pool pays is not the principal the balance loses, and over a
term the two drift. A real level-pay loan fixes the PAYMENT and lets
principal be the remainder after actual interest; the balance then falls by
that remainder. Not changed here — the balance reproduces the former closed
form exactly, which is what the rollout needs — but it should be decided:
either the balance rolls forward from the stream (which §7.102 enables), or
the day-count term is documented as an interest convention that leaves the
amortization schedule untouched.

### 7.105 A stream cannot read an account

Belongs with §5, language. Confirmed 3–4 September 2026 (probe: a field may
read `prev.<account>`; a stream's read is refused as `E1123`, and
`series_sum("account.<name>", …)` from a stream reads zero in silence).

An account is the language's non-cash balance, and the ledger settles it at
period close, so a stream reading the CURRENT balance would be reading
unsettled state and the refusal is right. The prior balance is settled, and
`prev.<account>` from a stream is the interest-on-a-reserve, fee-on-a-balance
and coupon-on-what-is-outstanding case: today each is a field that copies
the account first. The silent zero from the `series_sum` spelling is the
§7.101 defect again, at an account. Related: §7.76, §7.101, the party-owns-
several-accounts entry above.
