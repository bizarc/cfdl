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

The first result is the important one, and it sharpens the item: `mwh_cycled_year`
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
`mwh_cycled_year`, a price duration curve is a summary of the hourly price
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

### 7.58 A contract's type is recovered by string prefix, not carried

*Belongs with the packs and the compiler (section 5). Replaces the closed 7.53.*

A pack declares its contract types in `ontology/types.toml` with an identity:

```toml
[[contracts]]
type_id = "CRE.Contract.UnitLease"
contract_name = "cre.lease_unit"
subject_family = "asset"
parties = ["landlord", "tenant"]
```

A model must suffix a contract whenever the deal has more than one of
something — two tenants are `cre.lease_unit.tenant_a` and `.tenant_b`. Nothing
carries the fact that both are a `CRE.Contract.UnitLease`. Every consumer
recovers it by string surgery instead: strip the declared name off the front and
check the next character is a dot.

7.53 closed by deleting `ContractMatch` and sharing ONE predicate between
lowering and validations, so the two can no longer disagree. This item is the
next step, and it is a different claim: the predicate should not need to exist.

**The grammar already answered this and the implementation did not follow.**
`contract_stmt` takes TWO qnames — the contract TYPE and the INSTANCE NAME —
so `contract cre.lease_unit tenant_a` states both and neither has to be
recovered from the other. The implementation accepts one, fusing them into
`cre.lease_unit.tenant_a`, and every consumer then does string surgery to get
the type back. `dot_suffix`, the deleted `ContractMatch` mode, the bare-read
bug class that `tools/check-pack-series.py` now gates, and this item are all
downstream of that one restriction. See 7.63.

**What carrying the type would buy.**

*Resolution becomes decidable.* The predicate takes the first declared name that
fits. Across four packs there are 39 contract types and none is a dotted prefix
of another, so today there is exactly one answer — but that is accidental, not
enforced. A pack adding `cre.lease_unit.retail` as a TYPE makes
`cre.lease_unit.retail.suite_3` match two declarations, and the predicate picks
one silently. Resolving once can require a unique type and say so when it cannot
find one.

*A misspelling gets a diagnostic.* `cre.lease_unitt.tenant_a` names no type.
Today it matches no lowering rule and no validation; whether anything reports
that is the first thing to establish here.

*Matching becomes equality.* Lowering and validations both compare `type_id`,
and the prefix logic exists in one place — resolution — instead of at every
call site.

**Scope.** Resolve at contract declaration, carry type and instance name on the
contract, migrate both match sites, and decide whether the type surfaces in the
IR schema. Larger than 7.53 was; nothing is broken while it waits.

---

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

### 7.63 A contract cannot name its instance separately from its type

*Belongs with the language (section 5). The grammar's design; the implementation
never met it.*

```ebnf
contract_stmt = "contract" qname [ qname ] [ "on" "entity" entity_ref ]
                contract_block ;
```

Two qnames: the contract TYPE a pack declares, and the INSTANCE NAME. The
implementation accepts one and fuses them — `cre.lease_unit.tenant_a` — so the
type is no longer stated anywhere and has to be recovered by stripping a prefix
and checking the next character is a dot.

**Everything downstream of that restriction is a workaround for it:**

- `{{contract.dot_suffix}}` in every instanceable lowering rule
- `ContractMatch::Exact` / `::Instance`, deleted this week after two thirds of
  pack validations turned out to be silently skipped
- the bare-read bug class — `cre.pct_rent` double-counting into forward NOI,
  `cre.property.opex` vanishing from it — now gated by
  `tools/check-pack-series.py`
- 7.58, which asks for the type to be carried rather than recovered

None of these would exist if the instance name were its own token. No reason for
the restriction is recorded anywhere.

**Scope.** Accept the two-qname form; keep the fused spelling working or migrate
the ~520 declarations; carry `type` and `instance` separately on the IR contract;
then retire the string surgery. Large, and it closes more than it costs.

---

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

### 7.67 An option's type resolves against nothing

*Belongs with the packs and the language (section 5). Found surveying what a
pack can declare.*

A model writes an option with a type:

```cfdl
option refi_1 type Option.Refinance exercisable in construction {
  exercise when curve_value("sofr", time.date) < 0.045
  payoff cfg.refi_savings_estimate - 250000
}
```

`Option.Refinance` resolves against nothing. `PackOntology` carries
`entities`, `contracts`, `lifecycles`, `references` and `relations` — there is
no options member — and no pack declares an option type, lowers one, or
validates one. Three type names ship across five models (`Option.Call`,
`Option.Equity`, `Option.Refinance`) and the compiler accepts any string in
that position, so a typo is silent.

**Entities and contracts both have the surface options lack.** An entity's type
is checked (a misspelled field on a typed entity is `E1131`); a contract's type
selects the lowering rules and the domain validations. An option gets neither,
which makes it the one core construct a pack cannot describe.

**What a pack option type would carry**, by analogy with `[[contracts]]`:
a `type_id`, the subject family it attaches to, and the shape of its exercise
and payoff — enough for the compiler to reject an unknown type and for a pack
to state, say, that a refinance option's payoff is a function of a debt
contract's balance.

**Decide first** whether an option is a pack concept at all. The alternative
reading is that options are pure core language — a payoff expression and a
trigger, with no domain vocabulary — in which case the fix is smaller: a
closed set of type names in the specification, checked, rather than a pack
registry. Either way the current state (an unchecked free-text type) is not
one of the two.

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
declared metrics (§7.25, shipped).*

**What this item is.** An umbrella over the gaps that separate CFDL from the
full scope of a structured-finance cash flow engine (the Intex/Trepp
category: collateral pools feeding tranche waterfalls with triggers and
reserve accounts, plus bond analytics over the result). It exists so the
parity question has one place to stand; each constituent either references an
existing item or is named here for the first time.

**What is already covered, and is the larger half.** The collateral side runs
to published-schedule parity (the credit pack; the FNMA REMIC family at six
PSA speeds, the auto-ABS cases at speed variants). Sequential-pay tranching
runs as an ordered waterfall (`benchmarks/credit/auto_abs_tranches`; the
AmeriCredit 22-step priority compiles). The walk with accounts covers reserve
mechanics — fund to target, top up, release, trapped cash across a failed
test — and logic reads settled cash strictly backward (`docs/28` §4–§5,
shipped). Deterministic scenario grids — the dominant workflow of the
category — are scenarios plus curves plus options, today.

**Deal mechanics still open:**

- **Repeatable triggers as a checked construct.** An OC/IC test that fails
  and cures is a bare field flipping both ways until the declared machine
  ships — `docs/28` §6, docs/29 phase 5. Referenced, not duplicated.
- **Coupled interest/principal waterfalls.** Interest diverted into principal
  redemption on a trigger failure crosses two waterfalls; one pot does not
  express it. `docs/17` §5 question 2, still unresolved — the account and the
  walk are the machinery an answer would use, but the answer is not designed.
- **A step's shortfall as a published series** (`docs/17` §5 question 3) and
  **deferred/PIK interest on an unpaid step** (`docs/17` §5 question 1 —
  "probably a second form, not a default"). Both are the write-up-from-the-
  bottom mechanics CMBS and CLO documents assume.
- **Servicer advances.** P&I advancing and stop-advance appear nowhere in the
  docs. Under the machine they are a state (`advancing`, `stopped`) with
  streams gated on it and a recoverable-advances account; the item is naming
  that shape, not new machinery.
- **The clean-up call.** Exists only as the `called` lifecycle state in the
  credit pack's ontology; the election itself is an option whose guard reads
  pool factor — expressible now, but no shipped case exercises it. A
  benchmark deal with a call is the ask, not a construct.

**Analytics still open:**

- **Valuation-plane solvers: yield from price, price from yield, discount
  margin.** `model.irr` is the shipped precedent — a bracketed bisection over
  the completed projection, deterministic and replayable. These are the same
  computation with a different objective, and they belong in the valuation
  plane as declared metrics (§7.25, shipped — the construct they ride on), bracketed
  bisection or Brent per `docs/17` §12 — never in the causal core, where a
  solver would cost provenance and replay.
- **The make-whole.** A causal cash amount whose size is a discounting
  computation — the priced exception of `docs/28` §7 is the sanctioned
  mechanism, as with the direct-cap reversion. Currently on the credit pack's
  parity worklist as "needs an engine primitive"; the primitive is the priced
  exception plus a PV expression, not a new solver.
- **Per-period stochastic draws.** `assume ~ Dist` draws one scalar per
  trial; a rate path is a field recurrence whose innovation must differ per
  period. The extension is a per-period draw stream, seeded per
  (assumption, period, trial) the way per-assumption streams are seeded
  today — additive, journaled, replayable. Correlation stays excluded
  (`docs/01` §1.1.10) until a document forces it; a rate-dependent CPR is a
  recurrence reading the rate path, and needs no correlation construct.
- **The output surface an analyst reads:** per-class WAL assertions (§7.22),
  the published settlement axis (§7.26), participant-level return (§7.72, shipped),
  model-declared metrics (§7.25, shipped) and statements (§7.55). Referenced, not
  duplicated.

**Infrastructure still open:**

- **Multi-currency.** No mechanism has landed; the account was shaped so the
  currency clause is additive (`docs/28` §5.1). Blocked on a document that
  needs it, not on design room.
- **Loan-level scale is undemonstrated, not disproven.** Four loans tie to
  the single-pool model at 0.0 over 372 periods (§2.2); 43 sub-pool entities
  run in the auto-ABS cases. Nothing has measured thousands of entities, and
  the per-(stream, period) environment rebuild (`docs/29` §2.3) is the known
  hot spot to profile first. The ask is a measurement, then the fix if the
  measurement demands one.

**What stays out, on purpose.** Same-period circular conventions (a fee on an
ending balance that includes the fee) are spreadsheet artifacts, not
indenture mechanics; priorities are ordered, and the causal plane's refusal
to iterate is the product's guarantee, not its gap. A tool of this category
built on CFDL is those guarantees applied to the one domain that most needs
an auditable engine — the constituent items above are what remain between
here and that claim.

### 7.75 Storage state of charge is now buildable, and it is what validates the last energy rule

*Roadmap: M2 (§7.78); the case it unblocks is M3 (§7.3).*

**What forced the discovery:** the domain survey behind `docs/30`.
`energy.storage_arbitrage` is the energy pack's only externally-unvalidated
rule (§7.3: energy 9/10), and §7.1 recorded three ways forward, the third
being "needs per-period persistent state (5.2) and would let cycling be
modeled rather than assumed." The walk's phases 3 and 4 are that state: a
state-of-charge balance — a field or an account — stepped per period, charged
and discharged by streams the balance reads strictly backward.

**What it changes:** `mwh_cycled_year` stops being an assumed input and
becomes an output of dispatch against a price shape, which is the circularity
§7.1 says blocks validation against a dispatch reference. It is also the
state the `energy.storage_dispatch` quantile rule (`docs/27` §9 stage 4)
prices around: the quantile closes the Jensen gap, the SOC balance closes the
chronology gap — a 4-hour battery reaching only contiguous hours is a
constraint on a walked balance, not on a distribution.

**What still gates it:** a dispatch reference that runs (§7.1's SAM attempt
segfaulted front-of-meter). The construct no longer waits on the engine; the
case waits on a source. Related: §7.1, §7.3, `docs/27` §9, `docs/30` §2.

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

**The ask, in three parts.** First, the migrations the shipped fleet already
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
no split at all. The flip case is the remaining migration. Second, a
reserve contract shape per pack where a document demands one — the DSRA
funded to target with `dscr_periodic` gating the release, the replacement
reserve of §7.5, the FF&E reserve — each as the `pay <step> to account`
pattern rather than a bespoke contract. Third, interest ON a reserve balance:
a stream whose amount reads `prev.<account>`, legal under §4's backward rule,
and the first case that models it closes the CREST reconciliation line.

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

**What M2 is**, all of it standing on the walk, the machine and the account
(`docs/28` §4–§6):

| item | what it unlocks |
|---|---|
| §7.41 | the freeform `from <expr>` pot, the one unchecked selection left after the account |
| §7.76 | the account adoption pass: the reserve every pack's references assume and no pack could model |
| §7.77 | the DSCR cash trap — the first covenant whose breach has consequences, and can end |
| §7.75 | storage state of charge, which turns `mwh_cycled_year` from an assumption into an output |
| §7.74 | the deal mechanics still open after the machine: coupled interest/principal waterfalls, a step's shortfall, PIK on an unpaid step, servicer advances, the clean-up call |

**What M2 is not.** Declared metrics (§7.25, since shipped) and
participant-level returns (§7.72) are M4 — both since shipped — and `docs/31` W4 pulled the first forward on the
commercial path rather than the roadmap's. Pack coverage (§7.3) is M3.

Re-derived 2026-08-28. Related: `docs/28` §10, `docs/29`.

### 7.79 An event is restricted to firing once, and a transition cannot act

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
