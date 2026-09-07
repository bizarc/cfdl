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

### 7.3 Pack contract coverage across the benchmark suite

*Belongs with no single pack — it is about the validation programme.*

**Re-measured 2026-09-06** across all 42 registered cases (2 bespoke, 10 cre,
13 credit, 8 energy, 9 opco — the five Fannie Mae 2019-2 speed cases are
now scenarios of one, §7.23), counting a pack contract type as *exercised*
when at least one case declares it with `contract <pack>.<type>`. When first
measured (six cases, headline "the external cases route around the packs they
should be validating") the counts were energy 9/10, credit 1/4, cre 1/12,
opco 0/10 — for cre and opco the benchmarks bypassed the pack entirely, so
they validated the engine, not the domain logic. That circularity is broken:

| pack | exercised | not exercised |
|---|---|---|
| energy | **10 / 10** (see caveat) | — |
| credit | 3 / 4 | `participation` |
| cre | 11 / 14 | `lease`, `percentage_rent_expected`, `construction_stub` |
| opco | **12 / 12** | — |

The rosters have moved since the previous measure (2026-08-30, 44 cases):
credit's three pool types collapsed into one `loan` and gained `note` and
`participation` (`docs/40`, `docs/42`); cre added `lease_unit`'s companions.
`participation` is the pass-through security; the Ginnie or Fannie
pass-through case `docs/41` owes is what exercises it.

**Elections are a second roster.** Four option types refine
`Contract.Option` and lower nothing, so the contract scan does not see them.
Measured by `option … type <T>`: `CRE.Contract.RenewalOption`
(`office_renewal_option`), `Credit.Contract.CleanUpCall`
(`americredit_2017_1`) and `OpCo.Contract.EquityOption`
(`lbo_option_pool_exit`) are exercised; `CRE.Contract.PurchaseOption` is not
— the purchase option on a finance lease in `docs/41` is its case.

What closed the earlier gaps: `office_two_tenant` exercises the acquisition
spine through the pack (`lease_unit`, `rollover`, `vacancy_loss`,
`opex_line`, `permanent_debt`, `exit_forward`), `retail_strip` adds
`percentage_rent` and `exit`, `one_lincoln_street_contract` proves
`construction_loan` against the native twin, `float_bridge_pool` and
`io_bullet_loan` (now `credit.loan` with the master's `amortization` term)
close the loan, the auto ABS pilot's classes are `note`s, and `lbo_buyout`
plus `damodaran_fcff` (now on `opco.reinvestment`, the derived line) and
`dcf_exit_multiple_nwc` take opco to twelve — the
driver-disclosing sources the first measure asked for.

**Exercised is not the same as validated.** One caveat stands:
`storage_arbitrage` is declared by `solar_ppa_microgrid`, but that case
reconciles the reduced-form arbitrage margin against convention, not against
a dispatch model. The dispatch reference now exists —
`benchmarks/energy/merchant_storage_arbitrage`, a provably optimal linear
program, core-spelled with the state of charge as walked state — and its
notes say plainly that it does not validate the pack rule, whose shape takes
the cycled energy as an input the reference exists to compute. Energy's
*validated* count stays **9 / 10** until `energy.storage_dispatch`
(`docs/27` §9 stage 4; `docs/41` §5) replaces that rule and is checked
against the shipped optimum. Read strictly,
cases whose references are independently recreated conventions
(`office_two_tenant`, `retail_strip`, `solar_ppa_microgrid`) sit a step
below a published third-party model; each CASE.md states which kind it is.

**Two axes, not one.** A concept can be expressible in the core language, in
a pack contract, or both — and a case on native streams is a choice, not a
coverage failure. `one_lincoln_street` exists in both spellings, and the pair
is the assertion. `tax_equity_flip` uses no streams and no contracts at all —
declared state and an event, with the model's own comments arguing why core
is the right spelling. The penzance developments, `banker_dcf_conventions`
and `saas_sbc_convention_fork` model natively for the same reason. A
core-spelled case proves the LANGUAGE expresses the deal with no domain
vocabulary — the stronger claim; the pack contract is the ergonomics layer,
and this entry measures whether that layer is exercised, not whether it is
mandatory.

What remains, and it is narrow:

- **cre:** three types unexercised. `lease` (non-unit grain),
  `percentage_rent_expected` and `construction_stub` may want a new case
  each; `PurchaseOption` waits on the finance-lease demonstration.
- **credit:** `participation`, closed by the pass-through case.
- **energy:** `energy.storage_dispatch`, the pack rule the shipped optimum
  can validate (`docs/27` §9 stage 4).

Recorded because coverage claims must cite this table, and the table must be
re-measured — by scanning `contract <pack>.<type>` and `option … type`
declarations, not `<pack>.` prefixes, which also match namespaced stream
names — whenever cases or rosters change.

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

*Narrowed 6 September 2026.* The payment-day half shipped: a waterfall's
schedule placement is every step's, so `schedule every month on day 25`
puts each class's principal on the 25th and the seven FNMA cases assert
their published lives at the print floor (±0.05). No run-configuration
date was added, and none should be: a settlement date is not a fact the
run asks, it is the day the holder's claim came into being — the term
start of the contract the step pays — and the day count is that contract's
own term, since two instruments in one model may differ.

What remains is the origin. `wal` over a step bound to a contract
(`pay … for contract credit.note.ab line principal`) should measure from
that contract's term start in actual days over 365, the market definition
of a published life, rather than from the model start on period fractions;
the FNMA classes are entity fields today, so the REMIC tranches as notes
(`docs/41` §5, `docs/42` S4) is what lets that land. The residual on FNMA
2019-2 is the two days between its 30 January settlement and the model's
1 February start.

Found asserting the seven published WALs of FNMA 2019-2, where the 400% PSA
column refused the naive floor and the refusal was the convention speaking.

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

*Roadmap: partly M2 (`docs/37`) — the deal mechanics; the analytics ride on
declared metrics (§7.25, shipped). Promoted 2026-09-01 to
`docs/38_intex_parity.md`, which carries the survey the way `docs/34`
carries the events design (`docs/34`): the parity-or-ahead ledger, the itemized gaps, the
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

### 7.76 The account adoption pass: every pack has a reserve it could not model

*Roadmap: M2 (`docs/37`).*

**What forced the discovery:** the account shipped (`docs/28` §5.1) and the
domain survey (`docs/30`) found the same absence recorded independently in
every domain's references. `crest_solar_cost_based/NOTES.md`: the reference
EBITDA "includes interest earned on funded reserve accounts (~$4,606 in year
one), which CFDL does not model." `utility_pv_singleowner/NOTES.md` lists
reserves among what the reference zeroed out to be comparable. `docs/41` §5 carries
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
reserve of `docs/41` §5, the FF&E reserve — each as the `pay <step> to account`
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

Related: `docs/41` §5, §7.41, §7.72 (shipped), §7.74, `docs/25`, `docs/28` §5.1, `docs/30` §1.

### 7.77 A covenant that is published but powerless: the DSCR cash trap

*Roadmap: M2 (`docs/37`). **The mechanism shipped 2026-08-30**; what remains is
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

**The cure period was the part that waited on the arrival actions of `docs/34`**, and it is worth being
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

Related: §7.74, `docs/28` §5.1 and §6, `docs/30` §2,
`docs/20` §5.1.

### 7.80 121 registered diagnostic codes have no minimal failing example

The machine docs work (docs/32 Phase 2) measured the register: docs/08 §7
names 197 codes, and only 71 appear in `fixtures/invalid/` + `gold/diag/`.
The repair catalog (`docs/machine/diagnostics-repairs.md`) lists the
uncovered codes by name, so this entry is a work queue, not a survey: each
item is one minimal failing fixture, its blessed golden, and a
compile-verified repair in `fixtures/repairs/`. Retired codes (§8) are
exempt. The catalog's coverage line is the progress meter.

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
expression), the model-declared statement (`docs/01` §16) (the declaration surface these reads would serve), §7.84
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

*Belongs with the language and engine (section 5). Related: the model-declared statement (`docs/01` §16), §7.43.*

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
- **A DECLARED per-period line**, which is the model-declared statement (`docs/01` §16) — a model cannot declare a
  subtotal, and a field's `next` reads no stream series (`docs/14` §3.1), so
  there is no legal place to compute an indicator. If the model-declared statement (`docs/01` §16) shipped, an
  indicator line declared once and `series_count` over it answers the covenant
  question WITH NO NEW SYNTAX AT ALL.

That last is the reason this entry exists rather than a `series_count_if`: the
missing thing is not a reduction, it is the line to reduce. **Do not build the
predicate argument before the model-declared statement (`docs/01` §16) is decided.**

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
halves of this entry are not independent: settle the model-declared statement (`docs/01` §16), and both get simpler.

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
shipped and these did not. The `period -> date` gap and the the model-declared statement (`docs/01` §16) dependency
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
inferred. Related: `docs/27` §4.4 (what `ref` buys), and EVS question 26.

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

*Narrowed 5 September 2026: a BALANCE no longer needs this. The account
rolls every model period and a stream reads the prior close as
`prev.<account>` on any grid (`docs/42` §7). What remains is the field.*

**The ask:** a fold bounded by the reader's own cadence — the stream's
values since the field last stepped — so a stream-driven field is
expressible at every cadence. It is what a loan-level pool will want, where
the reductions are actual payments and there is no rate to roll forward
from. Related: §7.98, `docs/28` §4.

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

### 7.106 MOIC of a zero cash flow is a ratio of sign noise

Belongs with §5, metrics. Found 4 September 2026 when the decimal-to-float
conversion became deterministic.

`fixtures/valid/credit_participation` passes every dollar of the pool
through to the holder, so `model.net_cash_flow` is zero in every period.
`model.moic` published 0.818182 before the conversion change and 0.9 after
it: both are counts of which periods' zeros carried a negative sign at the
28th digit. A multiple on money that never moved should be null, as the
pool factor is before the pool is bought, not a number that changes when a
float flips its last bit. The same applies to `model.irr` on such a series.

### 7.107 The construction loan compounds interest within the period

Belongs with §1, CRE pack. Found 5 September 2026 moving the construction
loan onto the balance account.

`cre.construction_loan`'s interest row accrues on `funded − draw − equity`,
and `funded` at that point already includes the period's own capitalized
interest (the funded field's `next` adds it before the streams read the
field). So interest is charged on interest accrued in the same period — a
compounding no loan agreement writes. The account reproduces it exactly
(the `capitalized_interest` accrual row copies the interest row's base) so
that no number moves; the right base is the opening balance plus the
period's draw at its accrual fraction, which is what `prev.balance` would
give. Decide with the pack, and re-bless `one_lincoln_street_contract`
when it changes. Related: §7.104.

### 7.108 An exercise cannot bring a contract into being

Belongs with §5, language and engine. Found 5 September 2026 building
`benchmarks/cre/office_renewal_option`.

A renewal option, exercised, produces a lease: five more years at the stated
rent, with the pack's own lowering — escalation, recoveries, the leasing cost
as a dated one-shot. The language cannot say that. A contract takes no
activation guard (`docs/01` §13.4), `activate`/`deactivate contract` was
removed (§7.73), and an option's actions reach entity fields and streams,
never a contract. So the case declares BOTH outcomes as leases and the
election switches one off: the option's actions deactivate the market
lease's four lowered streams, and a scheduled event with the opposite test
deactivates the renewal lease's — two declarations of one decision, and
eight `deactivate` lines for what is one sentence in the lease.

The shape to decide: an option's action that activates a declared contract
(`activate contract cre.lease_unit.tenant_a_renewal`, the contract's streams
inactive until then), or a contract whose `term` starts at an occurrence
(`term from exercise(renewal) for 60 months`). The second reads as the
agreement it is — the renewal term begins when the right is exercised — and
keeps the contract free of guards. Either way the renewal case's lapse
event and its `deactivate` lines go away. Related: §7.73, `docs/40` §10.

### 7.109 An option's payoff carries no category

Belongs with §5, language and engine. Found the same day, in the same case.

A stream states a category and the pack's subtotals fold it (`docs/35`). An
option's payoff publishes as `option.<name>` with an owner and no category,
so a renewal's leasing cost, paid as the exercise's payoff, would be in
`model.total` and absent from `domain.cre.leasing_costs`. The option should
take the stream's `category` clause, or the election type should fix one
the way a lowering rule does for its line — `Contract.Option` declares the
line `payoff`, and a pack refinement knows what its payoff IS. Until then a
case that needs the subtotal writes the exercise cash as a gated stream and
leaves the payoff at zero, which puts the exercise's cash outside the option
— what `benchmarks/cre/office_renewal_option` does, with the renewal
lease's own leasing cost as the exercise's cash.

### 7.110 A stream whose expression fails pays zero, under a warning

Belongs with §5, language and engine. Found 5 September 2026 routing the
curve-outside-its-dates refusal through the evaluation sites.

§7.103 made a FIELD whose rule fails fatal: a value that was never computed
is not a number, and the run refuses naming the field and the period. The
same failure everywhere else is still softened. A stream's amount that fails
to evaluate — a division by zero, a function on an argument out of range, a
name that resolves to nothing — is paid as 0; a guard that fails is read as
false; an account inflow that fails contributes 0; an option payoff that
fails pays 0. Each leaves a warning in `results.warnings` (`… evaluation
failed [EXPR_EVAL]: …; using 0.`), and the run reports ok with a total that
is the true total minus whatever the failed expression would have paid. A
compile failure of a stream's or field's expression is softened the same
way (`… expression compile failed …; using 0.`).

The benchmark harness fails a case on any warning, so no shipped case hides
one; a modeller's run does not have that gate. The rule §7.103 settled
applies without change: the causal plane refuses, naming the reader, the
clause and the period, and the fold does it once over the walk's markers
(the shape `E5032` and `E5040` already use). Decide whether the guard's
`false` is the one exception — a guard that cannot be evaluated is closer
to "the condition did not hold" than an amount is to "nothing was paid" —
before building. Related: §7.103 (the field half, closed), §7.95 (undefined
is not zero).

### 7.111 A security named by one waterfall and not another is a valid model

Belongs with §5, language and engine. Found 6 September 2026 building the
securitization front door in the UI prototypes.

A note class in a structured deal is five declarations: its holders, the
account that IS its position, an account for its interest, the note itself,
and a step in EACH of two waterfalls — interest at its coupon, principal by
seniority. `benchmarks/credit/auto_abs_tranches` writes seven of those, which
is thirty-five declarations and fourteen steps kept in agreement by hand.

Omit one step and nothing says so. A class present in `notes.interest` and
absent from `notes.principal` compiles, runs clean, and is never repaid
principal; the trust simply keeps the money, the ledger balances, and every
metric is a real number. The failure surfaces only as a class whose holder
account ends at zero, which is indistinguishable from a class that was
genuinely never reached.

BOTH MODELS ARE VALID, which is why this is a warning rather than an error. A
security paid interest and no principal is a legitimate thing to model — an
interest-only strip is exactly that — so the check cannot refuse. What it can
do is say that the deal declared a line the priority of payments never
mentions.

The ontology already knows enough. A `credit.note` is a `Contract.Security`
whose master declares its lines; a waterfall step binds `for contract <c> line
<l>`. So for each contract on a subject that some waterfall on that subject
pays, the checker can ask which of its declared lines no step names, and warn
once per unnamed line, naming the contract and the line. It should stay quiet
when no waterfall pays that subject at all — a security in a deal with no
priority of payments is a different and deliberate shape, not an omission.

The same evidence supports the mirror check, which is cheaper and stricter: a
step naming `line principal` on a contract whose master declares no such line
is already an error, and this is the other half of that pair.

Related: `docs/40` (Contract.Security and its lines), §7.96–7.98 (an account
per party, from the same benchmark), `benchmarks/credit/auto_abs_tranches`.

### 7.112 A statement row cannot repeat over the instances of a type

Belongs with §5, language and engine. Found 6 September 2026 building the
securitization front door in the UI prototypes.

`packs/credit/statements.toml` ships the artifact this domain publishes — a
servicer remittance report, folding categories rather than stream names, so it
stays correct as the pack grows. It applies to a deal that declares no
statement of its own, which is the right default and works.

It reports `Paid to holders: interest` and `Paid to holders: principal` as ONE
line each, across every class. An investor report is per class: what Class A
was paid, then B, then C. The pack cannot write those rows, because the class
names belong to a deal it has never seen.

A model can declare its own `statement` with a row per class naming that
class's stream — `streams = [...]` exists on a row for what a category cannot
express. That is hand-written and has to be kept in agreement with the classes,
which is §7.111's failure mode wearing different clothes: add a class, forget
the row, and the statement quietly reports less than the deal paid.

The shape to decide: a row that REPEATS over the instances of a contract type —
one row per `credit.note`, its label built from the instance, its figure the
line that instance was paid. Order is the declaration order of those contracts,
which is safe here in a way it would not be in a waterfall: a statement is a
VIEW, no cash depends on the order, and there is no seniority to get wrong.
Related: §7.111 (the waterfall half), §7.113, `docs/40` (a master's lines).

### 7.113 A pack template cannot extend a declaration it did not create

Belongs with §6, cross-pack. Found the same day, in the same work.

`packs/credit/templates.toml` already answers most of "add a note class". Its
`credit.note` body emits FOUR declarations — the trust, the holders, the
account that is the class's position, and the contract — which is more than a
template is usually credited with doing.

It cannot emit the other three quarters of the answer. A class is also a step
in the interest waterfall and a step in the principal waterfall, and a template
creates declarations; it cannot insert into one that already exists. So the
template covers four of the seven parts, and the three it cannot reach are
exactly the three whose omission nothing detects (§7.111).

The shape to decide: a template clause that appends a step to a NAMED
waterfall, with the position stated rather than implied — after a named step,
or at the end. Position must be explicit precisely because order is the
meaning: a template that silently appended would make the newest class the
most junior, which is right about half the time and wrong silently the other
half.

What this buys is that "add a class" becomes a pack capability rather than an
application's. Someone writing CFDL in an editor gets the same seven parts the
prototype's tranche table writes, from the pack that knows what a class is.
Related: §7.111, §7.112, `packs/credit/templates.toml`.

### 7.115 Conventions checks: the model is legal and almost certainly not meant

Belongs with §5, language and engine. Found the same day; §7.111 is the first
member and the reason to name the family.

Some models compile, run clean, and are wrong in a way the language cannot
call an error, because the same shape is legitimate elsewhere. A note paid
interest and never principal is an interest-only strip or a forgotten
waterfall step, and nothing in the text distinguishes them.

A diagnostic is the right home for these, and not a document, because it is
the ONE channel every consumer already reads: the CLI, the MCP `compile`, the
wasm engine, and any application built on them, each getting the span with it.
A convention recorded in prose reaches whoever read the prose.

The family, as warnings — never fatal, allowlistable, and each naming what it
saw and what it expected:

- a security whose declared line no waterfall step on its subject names (§7.111)
- an `assume` nothing reads
- a note whose `principal_account` is not the account its holder owns
- a waterfall step naming a contract that is not written on that waterfall's
  subject
- a party that owns no account in a deal whose waterfalls pay parties

To decide before building: these are new W-codes, so they land against the
W-code parity gate, and a warned run is already a suspect run to the benchmark
harness — which means each member has to be quiet on every shipped case before
it can ship. That is a feature: a member that fires on a benchmark is either a
finding or a badly drawn rule, and both are worth knowing before release.

Related: §7.111, W-code parity gate, `docs/22` (how a diagnostic should read).

### 7.117 `lookup` cannot reach the entity types, the published fields, or the categories

Belongs with §5, language and engine (the tooling half). Found the same day,
and over the preceding week building against the MCP tools.

`lookup` resolves a pack's contract types through their master chains and
returns effective roles, fields, lines, side and templates — the form-builder
payload, and it is good. `PackInfo` carries `contracts`, `masters`,
`templates`, `metrics`, `validations`. Three things a model needs are not
reachable through it:

ENTITY TYPES. A pack's entities are absent, so what a `contract ... on entity
...` may be written on is unknown. Guessing produced `Energy.Asset.Project`
and `OpCo.Asset.Business`, neither of which exists; the real names are
`Energy.Asset.GenerationFacility` and `OpCo.Asset.Enterprise`. The fix is a
field beside `contracts`:

    entities: [ { type_id, family, class, refines, lifecycle,
                  fields: [ { name, field_type, required, unit, description } ] } ]

WHAT A LOWERING PUBLISHES. A `credit.note` publishes
`credit_note_claim_<instance>` and `credit_note_interest_due_<instance>` on its
subject, and a waterfall step is written in terms of them. Those names appear
in no tool output; they are learned from a benchmark. Beside `lines` on
`ContractInfo`:

    publishes: [ { name: "credit_note_claim_<instance>", of: "money",
                   description: "face less what the holder's account received" } ]

THE CATEGORIES A RULE EMITS. An account, a statement row and a subtotal all
fold categories, and which categories a type's lowering produces is not
returned. Beside `lines`:

    categories: [ "operating.collection.interest", ... ]

Two smaller ones: a role whose name collides with a reserved word cannot be
bound and the failure reads as a parse error (`owner`), so `RoleInfo` should
say so; and the metric key shapes a run publishes (`entity.party.<x>.total`,
`model.npv`) are discoverable only by running, which is a poor way to find out
what a scenario grid can have columns of.

The honest counterweight, recorded because it shapes the priority: `RoleInfo`
ALREADY returns `unbound`, and a role that may not be bound was bound anyway,
because nothing consumed the field. Exposure is necessary and not sufficient.
This entry is worth doing and it is worth less than §7.115.

Related: §7.115, §7.116, `crates/cfdl-mcp/src/tools/lookup.rs`.

### 7.118 `skeleton` starts a valid model, not a valid structure

Belongs with §5, language and engine (the tooling half). Found the same day.

`skeleton` sets the right bar: it compiles AND runs its output before
returning, so a starter that warns is not a starter. What it returns is a valid
MODEL — the smallest thing that runs.

Every mistake worth guarding here is STRUCTURAL. A securitization is a trust, a
pool, two collection accounts, and per class a holder, two accounts, a note and
a step in each of two waterfalls, in an order that IS the seniority. Assembling
that from a menu is where a class ends up in one waterfall and not the other
(§7.111); starting from a shape that already has three classes wired
consistently is where that cannot happen.

The shape to decide: named shapes per pack — `securitization`, `acquisition` —
answered from a few values and returned already compiled and run, which is the
bar `skeleton` already holds itself to. The shapes belong in the pack rather
than in the tool, because what a deal of a kind consists of is domain knowledge
and the pack is where domain knowledge lives; §7.113's template extension is
most of the mechanism.

Related: §7.111, §7.113, §7.117.
