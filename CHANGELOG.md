# Changelog

All notable changes to this project will be documented in this file.

This project follows Semantic Versioning: https://semver.org/

---

## [Unreleased]

### Changed: MIT Rentleg Plaza reads actual opex instead of restating it

The benchmark's expense reimbursements rebuilt "actual opex per SF" from the
inputs inside each recovery stream — base, trend, fixed share, that year's
occupancy — because reading the opex stream made a recovery a reader, and
`cre.exit_forward` reads the recoveries. Dependency-ordered waves permit that
chain, so both halves of the formula (this period's opex, and the 2004 reset
stop) are now the same `series_sum("cre.opex.line", ...)` read with different
windows. Every reimbursement reproduces to the cent and the net exit price is
unchanged at 3,051,540.54. Closes the language half of backlog 1.2.


### Changed: streams evaluate in dependency-ordered waves

The engine's two-phase stream split — streams that read no series, then
streams that do, against a store sealed between the rounds — banned every
chain of series reads deeper than one to get acyclicity. Waves get exactly
acyclicity: `series_references` extracts each read as written,
`selector_matches` resolves it to the streams it names (the same edges the
old phase guard walked to *reject* depth-two chains), and each stream
evaluates one wave past the deepest stream it reads, against a store in
which everything it names is finished. A genuine circular read is the only
rejection left — `SeriesCycle` names the path (`'a' -> 'b' -> 'a'`) and the
engine never iterates toward a fixed point (docs/14 §5). A stream whose
series names are computed at runtime keeps its old semantics: it evaluates
after every literally-named stream and cannot itself be read. All existing
goldens are byte-identical; `fixtures/valid/series_depth_chain` pins the new
depth with hand-checkable arithmetic. Unblocks backlog 1.2, 1.6 and the
percentage-of-EGI management fee (migrations are their own PRs).

### Added: E1346_STREAM_READS_WATERFALL_STEP

A stream's `series_sum`/`series_avg` naming a waterfall step aggregated to
zero in silence — the step counted as a known producer, so no warning fired,
and the store never holds a step, so the read found nothing. docs/03 §3.2
always said a step's series is visible to a later waterfall's `from` and to
nothing else; the compiler now says so too, beside E1341/E1342.


### Fixed: a name that resolves to nothing is refused, not read as zero

`docs/03` §2 has always said *"Unknown variables are hard errors (EXPR_EVAL),
not nulls."* Every layer honoured it except the engine, which caught the error
and substituted zero: `inputs.typo` or `time.typo` compiled clean, warned once
per period, produced a column of zeros, and the run reported `status: ok`.
Entity fields were the exception — `E1131` has always refused those — so one
of three namespaces was checked.

The fix splits by what each layer can know.

**`time.` is closed**, so the compiler refuses it:
`E1133_UNKNOWN_TIME_READ` names the five bindings the engine actually binds
(`t`, `date`, `days_in_period`, `phase`, `ppy`), at the reader's own source
span, beside the `E1131` check it extends.

**`inputs.` cannot be checked at compile time** — an input may be supplied
entirely by the run configuration, which the compiler never sees, and
`run_dists_full` is exactly that model. The engine refuses it instead, where
every source is known, with a message naming the distinct unresolved names
rather than repeating one per period.

The distinction that makes this safe is DECLARED SOMEWHERE versus BOUND HERE.
An input declared only as a Monte Carlo distribution is unbound in the
deterministic pass and stays a warning; a name nothing declares is fatal.
`cfdl-expr` now separates the two failures at the source, giving an
unresolvable name its own `EXPR_UNKNOWN_NAME` code so arithmetic that merely
failed is still tolerated.

Goldens: 157 pass, zero numeric differences — the only golden movement is the
error code inside one fixture's warning text.

### Fixed: the Python extension's freshness guards no longer hash packs into it

`tools/py-stamp.py` hashed `packs/` into the extension's build stamp and
`test_native_is_fresh` globbed `packs/**/*.toml`, both on the premise that
cfdl-pack `include_str!`s every pack TOML into the binary. It does — but only
under `#[cfg(feature = "embedded-packs")]`, and crates/cfdl-py takes cfdl-pack
without it. The extension says so itself: *"No pack directory was provided and
this build has no embedded packs."* The SDK reads packs from disk at run time,
so a pack edit cannot stale the binary.

The cost was a false alarm on one of the most common actions in this
repository. Editing a lowering rule failed `py-check`, which demanded `make
py-develop`; the rebuild was a no-op because cargo correctly saw no dirty
input, the `.so` kept its mtime, and the mtime test then failed too — the only
way out being to touch a crate source to force a rebuild that changed nothing.

Both guards now stay quiet through a pack edit and speak on an engine one,
verified both ways.

The second defect the first one hid is closed as well. `make py-develop` ran
`pip install -e` and then stamped unconditionally, so a rebuild that did not
happen was certified fresh regardless; it was caught only because a second
guard disagreed. The stamp now records the artefact's identity beside the
source digest, so a stamp written without a build no longer matches the file
it claims to describe — and `--write` with no extension present refuses
instead of writing.

The notebook render stamp keeps its `packs/` entry, correctly: pack data
changes what a notebook prints, because the SDK reads it at run time.

Closes backlog 7.21.


### Added: every pack ships contract templates, and a gate keeps them working

`templates.toml` was populated for CRE's two line-item contracts and nowhere
else — opco, energy and credit had none, and the instruments decomposed this
week (a mortgage with eight terms, a project loan, three exit forms) had none
either, despite the design rules saying the conventional vocabulary ships as
templates.

Now 39 templates across four packs: CRE gains its five instruments beside the
fifteen line items; opco, energy and credit get their line items and
instruments from scratch. Each carries the terms its rule requires, with a
conventional value where a convention exists.

`tools/check-pack-templates.py` renders every template from its declared
defaults — what the editor inserts when no parameter is supplied — and
compiles it. It caught two real defects on its first run: `principal = 0` and
`balance = 0` defaults that the packs' own validations reject (E6051, E8022,
E9001), and declared ranges that outran the model they were placed in. A
template that does not compile teaches a shape the language rejects and leaves
the modeller debugging the pack's own snippet.

### Found: an option's type resolves against nothing (backlog 7.67)

`PackOntology` carries entities, contracts, lifecycles, references and
relations — no options. No pack declares, lowers or validates an option type,
so the three type names shipping across five models are free text the compiler
accepts unexamined. Entities and contracts both have the surface options lack.


### Changed: the teaching and examples adopt expression terms

The adoption pass over learn, training and the examples — pack-using models
only; hand-written core models are untouched:

- The learn quick reference and chapters 6 and 19 teach the capability: a
  signed formula ("CPI plus 50 basis points") is stated in the term, and the
  hand-written boundary moves to STRUCTURE the contract does not have.
- `examples/opco_with_growth` showcases it: the growth plan is a step curve
  read by the term — `growth_rate = curve_value("growth_plan", time.date)` —
  5% through the ramp, 3% mature.
- `mit_rentleg_plaza` states its opex agreement in the term
  (`amount_year = inputs.opex_psf_full * inputs.building_sf`) and restates the
  2004 expense stop FROM THE INPUTS, killing the parameter staleness backlog
  1.2 records. Reconciliation to MIT 11.431J holds at the same tolerance.

### Found: an assumption cannot reference another assumption (backlog 7.65)

`assume b = inputs.a * 2` compiles and then fails at run — assumptions have no
dependency ordering, the assumption is "ignored", and every read degrades to a
warned zero with the run reporting ok. Found because a benchmark rewrite tried
it and the reconciliation caught the zeros.


### Removed (breaking): the curve-selector twin terms

`escalation_curve`/`occupancy_curve` (cre.opex_line), `growth_curve`
(opco.revenue_line/opex_line/capex_line) and `tax_rate_curve`
(opco.cash_taxes) are gone, with the `if("" == "", scalar,
curve_value(...))` splices they required. A varying rate is stated in the
term itself — `escalation = curve_value("cpi", time.date) + 0.005` — which is
what expression terms exist for. The canonical curve semantics are unchanged
(a model-declared `curve`, read flat-forward by `curve_value` at the period's
date), and a mistyped curve name is an evaluation error rather than a silent
scalar fallback, which is stricter than the twin was.

`draw_curve` (construction) and `index_curve` (floating pools) remain: the
curve IS those instruments' primary input, not a selector beside a scalar.

Migration: `x_curve = "name"` becomes `x = curve_value("name", time.date)`.
The two shipped models using twins migrate that way; results are unchanged in
every golden — zero numeric differences across all 98 results files — and all
40 benchmarks pass.


### Changed (breaking): pack streams follow `[domain].[category].[line]`

The pattern is now stated normatively in the pack interface (docs/07 §6.4):
a pack-lowered stream is named `[domain].[category].[line]{.[instance]}`, a
contract's several streams share their `[domain].[category]`, and contract
TYPES may keep underscores — they are authoring surface. Hand-written streams
are the modeller's own names and carry no pattern.

Four streams violated it and rename, values identical everywhere:

- `loan.permanent_debt.*` → `cre.debt.*` (wrong domain, inherited from the
  retired netted stream)
- `opco.capex{.id}` → `opco.capex.line{.id}`
- `opco.taxes{.id}` → `opco.taxes.cash{.id}`
- `cre.pct_rent{.id}` → `cre.pct_rent.overage{.id}`

Metric selectors match the new names by prefix and are unchanged.
damodaran_fcff's two asserted columns rename with their streams; all 40
benchmarks and 22 exercises pass with identical values.


### Changed (breaking): one CRE revenue line item, `cre.revenue_line`

`cre.ops_revenue` — a singleton with a raw per-period amount and no growth —
is replaced by `cre.revenue_line`, mirroring `cre.opex_line`: instanced per
statement line, the vocabulary in the instance name
(`cre.revenue_line.parking`), a blended figure when unsuffixed, `amount` or
`amount_year` with `escalation`.

It is the first rule designed for expression terms: there is no
`escalation_curve` twin, because `escalation = curve_value("cpi", time.date)`
states the agreed formula directly. Five conventional templates ship —
parking, storage, antenna, laundry/vending, blended.

`E6020`/`E6021` retire with the old contract (E6064 replaces the amount
check; a revenue term legitimately reaches the projection tail so forward NOI
has revenue to read). The forward-NOI windows read the instanced family.
Eighteen models migrate by contract name alone; every golden value is
identical under the stream rename, and all 40 benchmarks and 22 training
exercises pass unchanged.


### Changed: exit contracts state the sale gross, with selling costs as their own line

Four exit rules folded selling costs into the proceeds via `* (1 -
selling_costs)` — `cre.exit`, `cre.exit_forward`, `opco.exit_ebitda`,
`opco.exit_perpetuity`. A pro forma shows the gross sale value and the
transaction costs; a netted figure can show neither. The proceeds streams are
now GROSS and each contract lowers a sibling `*.selling_costs` outflow
(`investing.selling_costs`), so gross less costs is the old net: net cash
flow and NPV are unchanged to the digit on every fixture, verified
numerically. Statements gain a "Less: selling costs" row.

`mit_rentleg_plaza` now asserts both columns — the gross at ten times forward
NOI and the 5% commission are each the source's own stated quantities; the
net_cash_flow column is untouched. All 40 benchmarks pass.


### Changed (breaking): `energy.debt_service` lowers to the whole instrument

Same decomposition as `cre.permanent_debt`, same reason: one netted stream,
no draw. Now `energy.debt.proceeds{.<id>}` (financing.debt_proceeds,
`funded_at_close` default 1), `.interest{.<id>}` and `.principal{.<id>}`.
The legs sum to the level payment exactly and fold into
`domain.energy.debt_service_periodic` unchanged, so DSCR holds to the digit.

The five levered energy benchmarks state `funded_at_close = 0` with the reason
in each model — their references net operations against debt service and never
book the draw. Three expected.csv files asserted the netted stream by name;
the column now asserts the positive-signed periodic subtotal instead — same
magnitudes, per period, against the same sources; only the sign convention
moved. All 40 benchmarks pass.


### Changed (breaking): `cre.permanent_debt` lowers to the whole instrument

The most common CRE financing instrument lowered to ONE netted stream,
`loan.permanent_debt_service` — and emitted no proceeds at all: a $6m mortgage
that funded nothing. The netting foreclosed the interest/principal split — a
tax line, interest coverage, an amortization schedule — and the missing draw
made every levered return wrong even while the net line reconciled.

Per the contract design rules (docs/07 §6.4) it now lowers to three streams:
`loan.permanent_debt.proceeds{.<id>}` (financing.debt_proceeds, at closing),
`.interest{.<id>}` (financing.interest) and `.principal{.<id>}`
(financing.debt_principal, balloon folded in when opted on). All three carry
`dot_suffix`, so a deal may hold more than one mortgage.

The split is exact — ipmt + ppmt equals the level payment identically — and
`domain.cre.debt_service` now folds the two legs where it folded the netted
line, so DSCR is unchanged to the digit (verified: the subtotal series is
byte-identical on the smoke fixture). The operating statement gains Interest
and Principal amortization rows.

`funded_at_close` (default `1`) draws the principal at `term_start`. A
reconciliation whose source's cash flow starts post-financing states
`funded_at_close = 0` and says so — office_two_tenant does, because its
reference nets rents against debt service and never books the draw.

The learn capstones had hand-written a `harbor.perm_proceeds` stream to work
around the missing draw. The workaround is deleted from all five capstone
models and the chapter; the contract funds itself with the same $9.5m, and
every expected exercise metric is unchanged.


### Added: a contract term may hold an expression

A term was a literal or one reference to a declared input, and nothing else —
`escalation = inputs.cpi + 0.01` was a parse error directing the modeller to an
`assume`. But what was agreed is often itself an expression: a lease escalating
at "CPI plus 50 basis points", a coupon of "SOFR + 225bp". Forcing those
through a model-level `assume` moved the agreement out of the contract and left
a reference behind. The grammar always said `literal_or_expr`; the
implementation now conforms to it.

```cfdl
terms {
  base_rent  = 42000
  escalation = curve_value("cpi", time.date) + 0.005
}
```

An expression term is compiled at its own site (`E5025_TERM_EXPR_INVALID` when
it does not parse) and substituted into lowering rules PARENTHESISED — template
expansion is a textual splice, and `a + b` into `{{x}} * {{y}}` would otherwise
silently associate as `a + (b * y)`. Literals and input references substitute
verbatim, byte-for-byte as before: across 98 IR and 98 results goldens, every
pre-existing file is unchanged.

An expression is valid where the rule uses the term in an expression — an
amount, a field's `init`/`next`. A term a rule reads as a name, date,
frequency, or period count must stay literal:
`E5026_TERM_EXPR_IN_LITERAL_SLOT`, and `E5017_PERIOD_TERM_NOT_LITERAL` now
covers expressions and non-numeric literals (the latter previously misreported
as a missing term). A non-integer expansion of a rule's net-days no longer
falls back silently to the contract's payment terms — it is an error.

Pack bounds apply where the value is knowable: literal terms are checked at
compile time, input references against their clip, expression terms not at all
— the same tier the specification already granted input references.

Two latent defects closed on the way: `inputs.cpi * 2` would have classified as
a reference to an input named "cpi * 2" (terms are now kind-classified by the
parser, and an input reference requires exactly one identifier); and
`x = -(a + b)` was silently DROPPED, the pack default applying in its place.

`fixtures/invalid/term_trailing_tokens` — the fixture that pinned the old
restriction — is now `fixtures/valid/term_expression`, and its `1000 + 500`
produces results identical to stating `1500`. docs/01 §8.2.1 is rewritten:
a term records what was agreed, and what was agreed is often a formula.


### Fixed: the published grammar now describes the language the parser accepts

`docs/schemas/CFDL_v0_1_Grammar.ebnf` is normative — `docs/02` says
implementations MUST support it and offers it for download as parser-generator
input. Checked by hand against the parser and against every shipped model, five
productions were wrong:

- `contract_stmt` wanted two qnames, a mandatory `on entity`, and `term` as a
  clause BEFORE the block. It is one qname, an optional `on entity`, and `term`
  is an item inside the block. As written the grammar rejected 519 of the 520
  contract declarations in this repository.
- `entity_stmt` made the block mandatory; it is optional. `entity_block` allowed
  only `IDENT literal_or_expr`, and the real block holds four item forms: a
  field, a rule-bearing field with `init`/`next`, `part of`, and `state`. The
  `entity_field` production was defined and referenced by nothing; it is now the
  field form.
- `option_stmt` had no `on entity`; it is optional and used.
- `map_entry` said a contract term is `literal_or_expr`. A term is a literal or
  one declared input — never a compound expression — which is why a pack
  composes `curve_value` from a term holding a curve name.
- `stream_effect_stmt` and `tags_block` describe features that do not exist.
  Both are now marked NOT IMPLEMENTED, matching §18.2, which lists `owner`,
  `direction` and `tags` as reserved for features not yet built.

Closes backlog 7.49. Filed 7.61 for the standing check that would stop this
recurring.


### Fixed: the reserved-word list is the lexer's list, and says which words do nothing

`docs/01` §18 is the published statement of what a modeller may not name a
thing. The lexer reserved 95 words and §18 listed 57, so 38 identifiers stopped
working with nothing to explain why. Fourteen reserved words are read by no
production at all — they render in error messages and nowhere else.

§18 now splits into words a production reads (81) and words reserved for a
feature not yet built (14), and `tools/check-keyword-register.py` holds the
split to the lexer and the parser. Adding a keyword now fails CI until §18 says
which it is.

Closes backlog 7.47 and 7.48.

### Found: `Mon` through `Sun` describe syntax that has never existed

The weekday keywords were documented as weekly-schedule anchors. No production
reads them, `on` accepts only `day <n>` or `eom`, and `weekly` is not a calendar
frequency. The specification described a feature that was never built, which is
exactly what the gate above exists to prevent. Filed as backlog 7.60.

Backlog 7.49 gains a second instance of the same drift in the grammar itself: a
contract term is documented as `literal_or_expr` and is in fact a literal or one
declared input.


### Added: a statement row may itemise a stream family, and the CRE operating statement does

A `line` row has always accepted `streams` selectors as well as `categories` —
"for what a category cannot express" — but they were undeclarable in practice.
A statement is checked at pack load for completeness: every category a lowering
rule emits must be claimed by some line row. Stream selectors contributed
nothing to that set, so a stream row could not replace the category row it
duplicated, and keeping both double-claims every stream.

The loader now resolves a stream selector to the category of the rule that emits
that family, which it can do because the pack declares both. So a stream row
claims a category exactly as a category row does, and claiming one both ways is
now an error rather than a double count discovered at runtime.

`cre.opex_line` is the case that needed it: one contract instanced per expense,
every instance carrying the same category, so a category row could only ever
render one number however many lines a model declared. The CRE operating
statement now itemises the nine conventional expense lines that
`templates.toml` ships, with `domain.cre.opex_total` as the total.

A modeller naming their own instance is not lost — it matches no row and the
evaluator emits a `residual` row for it. Verified end to end: a model declaring
`property_tax`, `insurance` and a `moat_maintenance` line of its own renders
-240,000 and -60,000 on their own rows, -15,000 under "Unclassified", and
-315,000 as the total.

Closes backlog 7.59.


### Changed (breaking): one CRE operating expense contract, `cre.opex_line`

`cre.property_opex` and `cre.ops_expense` are removed. They differed only in
escalation — an annual figure compounding against a raw per-period amount — and
in whether they could be instanced, and on that second point their names said
the opposite of the truth: `cre.property_opex` carried `{{contract.dot_suffix}}`
and repeated, while `cre.ops_expense`, the generic-sounding one, was a singleton.
A modeller could not be expected to pick correctly and nothing documented it.

`cre.opex_line` replaces both and spans the range from a single blended figure
to a fully itemised schedule. Grain is how many instances a model declares;
LEVEL is the entity each hangs on, with `part_of` doing the rollup, because the
CRE ontology already says a building may be modeled as one asset, as unit types
or as individual suites — a `level` term would restate that where the two could
disagree.

New terms: `amount` / `amount_year`, `escalation` / `escalation_curve`,
`pct_fixed`, `occupancy` / `occupancy_curve`. Escalation and occupancy are each
a PAIR rather than a toggle, because a contract term is a literal or one
declared input and never an expression — so a time-varying rate cannot arrive
through the term and the pack composes the `curve_value` call from a curve name.
`escalation = 0` is the default and is "no escalation".

The occupancy factor `pct_fixed + (1 - pct_fixed) * occupancy` closes backlog
1.1. `benchmarks/cre/mit_rentleg_plaza` now runs on the pack contract instead of
a hand-written stream and still reconciles to MIT 11.431J at a one-cent period
tolerance: $4.81/SF on 30,000 SF at 81% fixed gives $135,161 of 2001 opex, not
$144,300, and the published answer depends on it.

`templates.toml` — a real loaded structure the LSP reads for snippet
completion, empty in this pack since it was created — now ships nine
conventional expense lines plus a blended one.

### Fixed: opex dropped out of forward NOI when the contract was instanced

`cre.exit_forward` summed `series_sum("cre.property.opex", …)` by BARE name
while `cre.property_opex` was instanceable. A `.*` pattern matches the bare name
and its children; a bare pattern matches only itself. So
`cre.property_opex.taxes` emitted a stream the exit rule did not see: opex fell
out of forward NOI, NOI was overstated, and the exit price struck off it was too
high. Silent, and worth real money. It is the same defect found and fixed for
`cre.pct_rent`, one term over in the same expression.

Both opex terms are now one `series_sum("cre.opex.line.*", …)`.

### Migration

Rename the contract, and `opex_year` to `amount_year`. Note `cre.lease_unit`
also has an `opex_year` term — its expense stop — which is unrelated and
unchanged.

`E6020_CRE_OPS_MISSING_AMOUNT` and `E6021_CRE_OPS_INVALID_SCHEDULE` no longer
apply to the expense side. E6020 demanded a bare `amount`, which `E6061` now
replaces with "either amount term"; E6021 rejected a term reaching into the
projection tail, which an expense legitimately does so that forward NOI has
expenses to read.

New codes: `E6061_CRE_OPEX_LINE_MISSING_AMOUNT`,
`E6062_CRE_OPEX_LINE_PCT_FIXED_RANGE`, `E6063_CRE_OPEX_LINE_OCCUPANCY_RANGE`.

Published stream names change: `cre.property.opex` and `cre.ops.expense` both
become `cre.opex.line{.<id>}`. Scenario overrides keyed
`stream.cre.ops.expense:amount` move with them.

Goldens were re-blessed. Across 98 results files and 88,863 numeric leaves,
every number is identical under the rename — only names moved.


### Changed (breaking for external packs): a validation's `match` field is gone

A pack validation matched a contract by EXACT name unless it declared
`match = "instance"`. A model must suffix a contract whenever a deal has more
than one of something — two tenants are `cre.lease_unit.tenant_a` and
`.tenant_b` — so a validation that omitted the flag was silently skipped on the
form models actually use. It never fired, and nothing said so. Two thirds of
the shipped validations were dead that way: `E7001_OPCO_LINE_MISSING_AMOUNT`
rejected `opco.revenue_line` with no amount and accepted
`opco.revenue_line.core` with no amount, one character apart.

That was fixed by requiring the declaration and gating it. The gate reads
`packs/*/validations.toml` in THIS repository, and packs are a published
extension point — so an author writing a pack elsewhere still got the silent
default, from a field the pack interface documentation never mentioned.

Lowering never offered the choice. `rule_matches_contract` has always matched
instances unconditionally, for the case that decides what cash a contract
produces, and its six lines were a byte-for-byte copy of the validation path's
`Instance` arm. Validations were the outlier and no reason was ever recorded.

So `ContractMatch` is deleted, matching is unconditional, and both callers share
one predicate. The 68 `match = "instance"` declarations are removed, and so is
the gate that required them — there is nothing left to state.

**Migration:** delete `match` from your validations. `PackValidation` sets
`deny_unknown_fields`, so a pack that still carries it fails to load with an
error naming the key rather than loading with it ignored. `match = "exact"` has
no replacement; if you have a case that needs it, it is worth hearing about.

Golden outputs are byte-identical: every shipped validation already declared
`instance`, so nothing in this repository changes behaviour.


### Added: the run configuration schema is a gate

`docs/schemas/run.schema.json` describes what a `run.json` may contain, and
nothing read it. Not the engine, which parses run configs with serde and applies
its own rules; not the CLI; not a test.

It drifted, in the direction silence always allows. `valuation_grain` had been
accepted by the engine and documented in the user guide for as long as it
existed, and the schema never listed it — and because `DeterministicCase` sets
`additionalProperties: false`, a run stating its own grain would have been
rejected by the schema it is supposed to conform to. Nobody noticed, because
nothing checked.

`tools/check-run-schema.py` validates every run config in `benchmarks/`,
`fixtures/`, `training/` and `examples/` — 123 of them — and `make ci` runs it.
It follows `check-ir-schema.py`, including the rule that a gate which can pass
without running is not a gate: `CFDL_REQUIRE_SCHEMA_GATE=1` turns a missing
`jsonschema` into a failure rather than a skip.

It catches what the schema's own description says the design prevents — *"a
misspelled key would otherwise produce a clean run with wrong numbers"*:

```
deterministic: Additional properties are not allowed ('anual_discount_rate' was unexpected)
deterministic/arithmetic: 'float' is not one of ['decimal', 'excel_compat']
```

What it does not yet catch is an override key that resolves to nothing. Two of
the four key shapes could be checked — `inputs.<name>` names an assumption and
`stream.<name>:amount` names a stream, both declared — and two could not, since
`cfg.<path>` and `obs.<path>` name nothing the model declares. Recorded as the
open half of `docs/13` §7.51.

### Added: a run selects its arithmetic, and `act/act` joins the day counts

Two items from `docs/13` §6, both of which had a capability sitting one step out
of reach.

**A run may state `"arithmetic": "excel_compat"`.** `Mode::ExcelCompat` existed
in `cfdl-calc` and was exercised only by that crate's own tests: the engine
always called plain `eval`, and there was no run-config key and no flag.

```json
{ "deterministic": { "arithmetic": "excel_compat" } }
```

```
(0.1 + 0.2) * 1e15    decimal       300000000000000.0
                      excel_compat  300000000000000.06
```

Decimal stays the default and is what every published number uses. The mode is
carried on `ExprEnv` rather than threaded through each evaluation site, so no
signature changed; 152 goldens are byte-identical, which is the check that the
default moved nothing. It is run-wide — scenarios and Monte Carlo trials inherit
it, because a scenario varies the deal's drivers and the rate it is valued at,
not the arithmetic every scenario shares. An unrecognized value is refused.

**`year_frac` accepts `act/act` (ISDA).** The span is split at calendar-year
boundaries and each part measured against its own year's length, returned over
the common denominator 365*366 so the result stays one exact integer ratio like
every other basis:

```
2024-07-01 -> 2025-07-01   act/act  0.998622651   = 184/366 + 181/365
                           act/365  1.000000000   = 365/365
2024-01-01 -> 2025-01-01   act/act  1.000000000   a leap year is still one year
```

The item had recorded the blocker as the expression environment not exposing the
days in a period's year. That holds for the pack's `{{model.accrual_divisor}}`,
which resolves to one number per period and so cannot carry a denominator that
changes with the year — but not for `year_frac`, which receives both endpoints.
Two paths of different difficulty, recorded as one. The divisor remains open.

`docs/schemas/run.schema.json` gained `arithmetic`, and `valuation_grain` with
it: the engine has always accepted that key and the schema had never listed it.
The schema sets `additionalProperties: false`, so a run stating its grain would
have failed validation had anything validated against the schema. Nothing does.

### Fixed: `time.phase` answers with the phase name

The specification requires the expression environment to support `time.phase`
(`docs/01` §16.2), the expression guide lists it in the `time` namespace, and
the course introduces it in chapter 5 as "the current phase's name" and repeats
it in the quick reference. It was never computed. `crates/cfdl-engine/src/env.rs`
inserted `ExprValue::Optional(None)` under `phase` at both env-building sites, so
it was null in every period of every model.

Null fails quietly in both directions, and one of them reads as success: a null
equals no string, so an inclusion test never fired, and a null differs from every
string, so `active when time.phase != "construction"` — the natural way to write
"not during construction" — was true during construction too and the guard did
nothing at all. Neither form warned. The value also reached results as the Rust
debug form of the empty optional.

It now answers with the name of the phase covering the period. The membership
test is the one `state.rs` already applied to an option's `exercisable_in_phase`,
lifted into `phase_at` and called from both sites, so a phase means the same
thing to a guard as it does to an option rather than being written twice. Where
no declared phase covers the date the answer is still null. Where two phases
cover it — overlapping phases compile today — the FIRST DECLARED wins, so the
answer is a stated order rather than map iteration.

Running the course's chapter 9 model with phases added and `time.phase`
published:

```
period  0 (2026-01)  construction
period 11 (2026-12)  operations
period 35 (2028-12)  operations
```

No model in `packs/`, `fixtures/`, `benchmarks/`, `examples/` or `training/`
read `time.phase`, so nothing depended on the old null; 151 goldens are
unchanged.

### Fixed: a waterfall step cannot read its own payments — backlog 7.41 item 3

A step that reads its own waterfall through `series_sum` was answered with a
silent zero. Steps publish when their waterfall finishes, so the read sees
nothing; the arithmetic around it then quietly does nothing.

`fixtures/valid/waterfall_after_contract` was doing exactly that. It capped a
note at its balance by subtracting what it had already paid, the subtraction
took nothing away, and a $500,000 note paid out $1,200,000 across six periods
with a golden that agreed. That is the failure `docs/13` §7.41 predicted as a
preferred return paid in full six times, found in the repository rather than in
a report.

`E1342_WATERFALL_SERIES_NOT_VISIBLE` now refuses the spelling at compile time,
beside `E1341_WATERFALL_FORWARD_REF` — the same failure one spelling over, so
the two answer the same reference the same way. Reading an EARLIER waterfall is
the documented composition and still compiles;
`fixtures/valid/waterfall_nested_split` pins that. The message names the model
that works: `paid.<step>` for this period's payment, and for a running total a
balance the distribution moves — a field the step pays and the balance
subtracts, which works today (`docs/26_lessons_learned.md`).

The fixture now states the per-period cap it was computing all along. Its
`ledger_hash` is unchanged and the other 148 goldens are byte-identical, so
every published number was already this one — only the expression became
honest.

`series_references` moved to `cfdl-expr` so the compiler and the engine read
series names with one scanner rather than two that could drift.

### Added: AmeriCredit 2017-1 — an auto ABS that builds its own enhancement

`benchmarks/credit/americredit_2017_1` reproduces the percent-outstanding grid
a sub-prime auto ABS publishes for six note classes at four ABS speeds, plus a
weighted average life to call and to maturity for each. It is the first case in
the suite whose notes have to build credit enhancement rather than simply
receive collections: excess cash accelerates principal toward 14.75%
overcollateralization net of the reserve, and principal beyond the target is
retained as a Step-Down Amount rather than paid, subject to a floor of 0.50% of
the initial pool.

The reference implementation reproduces **184 of 195 informative cells** inside
the grid's own whole-percent rounding floor — mean error 0.2479 against the
0.25 a correct model predicts, maximum 0.4990 against 0.4973 — and **46 of the
48 published lives** exactly. The CFDL model agrees with it to 4.4 cents on a
$305m class across every class and period.

Four conventions the prospectus does not state had to be recovered, each by
testing candidate readings against all four published speeds: a January-cutoff
pool pays twice before the first distribution; ABS runs from origination, which
retires four seasoned pools outright at 2.00%; the step-down floor is 0.50% of
the initial pool; and weighted average life runs 30E/360 from closing to the
18th with a 25-day stub. Eleven cells remain outside the floor, all Class A-1
or A-2 in the first six months; three candidate explanations were tested and
rejected rather than fitted.

**Found by building it:** the case states its distribution twice, once in the
waterfall and once across seven balance recurrences, and the two can disagree —
which is how a servicing fee came to be right in one and wrong in the other.
The duplication is avoidable: state the amount once as a field, and let the step
pay it while the balance subtracts it. Recorded in `docs/26_lessons_learned.md`,
where the first reading — that a waterfall cannot tell a balance what it paid,
and that the engine needed restructuring — is corrected.

### Added: a writing standard, and the documentation held to it — backlog 7.28–7.35

The documentation estate — cfdl.dev, learn.cfdl.dev, and every source that
feeds them — now has what the numbers have had all along: a standard, a
measurement, a remediation, and a gate. The audit is `docs/21`, the standard is
`docs/22` (CFDL-CE, derived from ASD-STE100 and tiered by content type), the
terminology register is `docs/terminology.toml`, and the accessibility
assessment is `docs/23`.

**Measured first.** 70,438 words of published prose, sentence by sentence. The
findings were concrete: the same words published in two spellings (41
conflicting forms once the generating sources were read, not the 7 the rendered
pages showed), one object under three names, 143 RFC 2119 keywords across the
three specifications with no definition anywhere, no glossary, and not one page
with a meta description.

**Then fixed.** US spelling throughout — 537 replacements, identifiers renamed
with their dependents and four goldens re-blessed label-for-label with every
numeric token verified unchanged. The specs define their normative keywords by
BCP 14. Every page states what it is (generated pages derive their description
from sources that already exist, so there is no second wording to go stale).
`/docs/glossary` publishes 47 terms generated from the register. All 22
exercise prompts are numbered imperative steps — mean sentence length
19.8 → 11.4 words — and the chapters' procedures instruct instead of asking.

**Then gated.** `check-site-voice.py` enforces the mechanical subset — retired
spellings and synonyms load from the register at run time, so the standard, the
glossary, and the enforcement cannot drift apart. Judgment rules (sentence
length, voice) stay in review, deliberately: a gate that flags judgment gets
disabled. The specifications are now read by a prose gate for the first time.

**And assessed for accessibility.** WCAG 2.2 AA, on production builds and the
deployed sites, both themes: the muted-text token failed contrast in both
themes and was split per theme, the playground splitter could not report its
value to a screen reader, and tables, code blocks, and the results panel were
unreachable by keyboard. All fixed; axe reports zero violations on every swept
page. Conformance is **not** claimed until the human assistive-technology pass
runs — that is backlog 7.35, the one item the program leaves open.

### Fixed: streams are line items — backlog 1.3, 1.5, and the reporting half of 7.14

A stream is the atom a statement reports, so a stream that is secretly an
aggregate is a row a statement cannot show. Three of them were.

**A property may now have more than one expense line (1.5).**
`cre.opex_line` takes a suffix and `domain.cre.noi` selects
`cre.opex.line.*`. `benchmarks/cre/hud_home_multifamily` carries its four
published sub-lines as four streams and **asserts all four independently**
against the Sample workbook's Operating Pro Forma rows 18–21, where it
previously asserted only their total. The four states already existed — split
for the rounding reason — so this moved nothing: their sum reproduces the
previously asserted total at every anchor year.

**Free rent is its own deduction (1.3).** `cre.lease_unit` emits
`cre.unit.abatement.<id>` and publishes base rent GROSS; the abatement family
sits in `domain.cre.noi`'s denominator, so the two net to the rent collected.
Previously a model could report the line OR have it counted in NOI, never both.
Verified as an exact decomposition — gross + abatement equals the previous net
to 0.00e+00, and NPV, NOI and DSCR are unchanged.

**HUD's mortgage separates P&I from MIP (7.14).** The pro forma's debt line is
one number and the workbook defines it as P+I+MIP. Both legs are now grounded
in the First Mortgage Sizing tab rather than inferred: MIP is the stated 0.450%
of the stated $150,000 principal (675.00, flat, exact), and debt service is the
residual of the published "Calculated Monthly P+I+MIP Payment" of 1,165.7819 —
which reconstructs the 13,314.3828 that backlog 7.14 had recorded by hand.
`domain.cre.debt_service` carries the MIP because coverage there is measured
against the whole published line, which is what the workbook's own DSCR uses.

**No expectation moved.** An intermediate version of this change used the sizing
tab's unrounded 13,989.3828 and moved the lifetime figure to 195,851.36, on the
reasoning that the pro forma's 13,989 was a rounded display. It is not: that
cell is `=ROUND(...,0)`, so 13,989 is what the workbook COMPUTES, and its
published DSCR is that rounded line divided into a rounded NOI. Using the
unrounded payment would have been more precise and less accurate. The model
applies the workbook's own round — via the `round_to` it already uses for the
expense recurrence — rather than restating 13,989 as a constant, so the
derivation stays visible.

Every native stream in every pack-using model is now classified, so the
completeness gate that Stage 8 turns on starts from zero unclassified streams.

**Invariants hold across all ten changed results goldens**: `model.total`,
`model.npv`, `model.irr`, `model.moic`, `domain.cre.noi`, `domain.cre.dscr` and
every `model.net_cash_flow` period are identical. What changed is that
aggregates became lines.

### Added: provenance, resolved inputs, and a ledger hash — `results_version` 0.3

A published line item can now be traced back to the term that struck it.

**`inputs.streams`** records, per stream, the contract terms a pack rule
actually consumed. Not the contract's whole term map: a contract lowers to
several streams and each reads a different subset, so "the contract's terms" is
not an answer to "what struck this line". One `cre.lease_unit` contract produces
three streams with three different term sets:

    cre.unit.base_rent.tenant_a   <- rent_year, escalation
    cre.unit.recoveries.tenant_a  <- opex_year, opex_escalation, expense_stop_year,
                                     pro_rata_share, gross_up_factor (pack default)
    cre.unit.ti_lc.tenant_a       <- ti_total, lc_total

`defaults_applied` separates the values the model stated from the ones the pack
assumed, because "the model said 0" and "the pack assumed 0" are different facts
and a reader tracing a number needs to tell them apart.

Note what this was NOT: `crates/cfdl-compile/src/lib.rs` emits `terms: {}` on
every contract and always has, so nothing was being un-dropped. The terms are
read from the rule's own templates *before* expansion — afterwards the keys are
gone and only their values remain, indistinguishable from literals.

**`inputs.resolved`** publishes evaluated `assume` values. Worth having on the
page rather than only in the model source: in a deterministic run a random
assumption resolves to its clipped CENTRAL value, not to a draw, and publishing
it is what stops that being invisible.

**`ledger_hash`** is a SHA-256 over the deterministic ledger — the series and
the annual rollup. Together with `model_hash` and `engine` it closes the chain:
identical inputs on an identical engine must reproduce an identical ledger. A
golden diff can say "this document changed"; it cannot say whether that was a
real behavioural difference or a run-to-run wobble, and a wobble would surface
as a flapping test rather than as the defect it is.

It deliberately covers the ledger and not the metrics. NPV and IRR are folds OF
the ledger, so including them would make the hash move for a reason the ledger
did not — and it means `ledger_hash` is **invariant to the discount rate**,
which is correct: the ledger is cash before discounting. There is a test
asserting exactly that, alongside reproducibility and the fact that changing a
model's cash does move it.

The engine passes `stream_inputs` through as opaque JSON. `IrStream` is not
widened and the per-period evaluation path is untouched.

**No numbers move.** 116 goldens change: 1,384 IR `stream_inputs` leaves, the
same republished under `inputs.streams`, 72 `ledger_hash` values, 7 resolved
assumptions, plus the `results_version` bump and the 44 `model_hash` values that
follow the IR change. Zero numeric leaves differ.

### Added: stream categories

Every stream may now declare what it IS, economically, and aggregation reads
that rather than pattern-matching its name:

    stream cre.abatement.suite_200 on entity asset.rentleg outflow currency USD {
      schedule every year from 2001-01 to 2006-01
      category operating.deduction.abatement
      amount = ...
    }

A name is an address; a category is a meaning. Deciding that `cre.vacancy.loss`
is a deduction by reading its spelling means every metric, fold and statement
re-derives the same judgement independently — and they drift, which is exactly
how two `.*` selector dialects came to disagree.

**Why direction is not enough.** CRE emits seven outflow rules; three sit above
the NOI line (`ops.expense`, `vacancy.loss`, `property.opex`) and four below it
(`unit.ti_lc`, `rollover.ti_lc`, `construction.draws`, `permanent_debt_service`).
`direction` says "outflow" to all seven. The split already existed — as nine
hand-listed stream names in `domain.cre.noi`, restated in
`cre_exit_forward_noi_derived` and again in a benchmark's reference generator.
Categories do not add a concept; they move one to where it cannot drift.

**Categories are hierarchical paths, rooted in the cash flow statement.**
`operating.revenue.base_rent`, `investing.capital.leasing`,
`financing.debt_service`. Every system that solves this converged on the same
shape — IAS 7's three sections, a chart of accounts' five root types,
beancount's `Expenses:Rent:Office`, XBRL's calculation linkbase: a small
universal root, then an arbitrary domain tree, with the rollup defined by the
tree. So a subtotal is a prefix query over the selector streams already use —
NOI is `operating.*` — and a generic statement works against a pack it has
never seen.

CFDL enforces the root vocabulary and nothing below it. WHICH root a category
takes is the pack's call, because that genuinely varies: interest paid is
operating under IFRS and financing under US GAAP, and a lender's interest
*received* is operating revenue rather than financing at all.

All 58 lowering rules across the four packs are classified, and
`benchmarks/cre/mit_rentleg_plaza` now classifies its ten native streams —
including the abatement line that backlog 1.3 is about, which the pack has no
contract for and which a name-based selector could never have reached.

New diagnostic `E5022_UNKNOWN_STREAM_CATEGORY`. A pack whose vocabulary is not
rooted in a known section fails to load.

**No numbers move.** 81 goldens change: 169 added `category` fields, the 40
`model_hash` values that follow, and one parser message now advertising the new
item. Checked leaf by leaf — zero numeric values differ, and all 21 benchmarks
still reconcile.

The wasm budget moved 600 → 640 KB gzipped. It had been sitting at exactly
600/600, so the next addition of any kind was going to trip it; categories cost
~9 KB raw / 3 KB gzipped. Recorded in `build-wasm.sh`, along with the thing that
did *not* work: the pack TOMLs are `include_str!`-embedded so their comments do
ship, but cutting 2 KB of comment prose recovered 0 KB gzipped.

### Fixed: one selector dialect, and two metrics that were quietly wrong

There were two implementations of the `.*` stream selector and they disagreed
about whether it reaches the BARE name. That matters because a lowering rule
writing `energy.ppa.revenue{{contract.dot_suffix}}` emits the bare name for an
unsuffixed contract and `energy.ppa.revenue.plant_a` for a suffixed one, so a
selector reaching only one form silently drops the other — an absent stream
contributes 0 rather than raising.

Neither defect was caught, for the same reason: none of the affected fixtures
runs with `--pack`, so `domain_metrics` is absent from every golden that would
have shown them.

- **`domain.credit.wal_years` omitted unsuffixed pools.** `wal_years` matched
  `stream.<prefix>.` against series keys, which carry no `.total`, so a bare
  `credit.pool.prepay` failed the prefix test. It selects sched_principal,
  prepay, bullet and recoveries this way and goldens ship all four bare, so an
  unsuffixed pool reported a weighted average life computed over a subset of its
  own principal. (`sum` reached the bare name too — but only because its keys
  end in `.total`, which supplied the separating dot by coincidence rather than
  by decision.)
- **Every energy metric omitted suffixed contracts.** All fourteen selectors in
  `packs/energy/metrics.toml` named their stream exactly, while all ten energy
  lowering rules template the name. A suffixed PPA therefore contributed nothing
  to revenue, EBITDA, DSCR or tax benefits. Three goldens ship
  `energy.ppa.revenue.plant_a`, one carrying $29.9m.
- **`cre.exit_forward` double-counted an unsuffixed percentage rent.** Its
  forward-NOI expression summed both `cre.pct_rent` and `cre.pct_rent.*`, and
  the glob already includes the bare name — so the stream entered twice and
  inflated the exit price it strikes. Latent: every shipped model suffixes that
  contract.

Matching now lives in one place, `cfdl_expr::selector_matches`, and matches
NAMES rather than storage keys, so the key format cannot be load-bearing again.
`.*` reaches the bare name and its children both; the path-segment boundary is
unchanged, so `cre.pct_rent.*` still does not reach `cre.pct_rent_extra`.

**No shipped model's numbers move.** Ten goldens change — four IR expression
texts, the four `model_hash` values that follow, and 28 lineage selector
strings. Checked leaf by leaf: zero numeric values differ.

`check-pack-validations.py` gains a fourth check, because "pick the right
selector by reading the file" is exactly what failed here: a metric that names a
templated stream exactly is now rejected. Verified both ways — it reports all
fourteen energy selectors against the pre-fix file.

### Added: schema `--write`, warning codes in the gates, and a determinism lint

`check-results-schema.py` and `check-ir-schema.py` gained `--write`, which
regenerates the site mirror and the embedded docs block from the source schema.
Both gates could previously say the three copies disagreed but not make them
agree, so keeping them in step was a three-way paste — and `docs/06` is the copy
that fell four releases behind. The canonical serialisation now lives in one
place, `tools/schema_sync.py`, rather than being re-derived by hand.

`check-pack-validations.py`'s code-uniqueness checks matched an `E` followed by
digits, so warning codes were invisible to both of them: a `W3500` could be
added twice, or added without ever being documented, and nothing looked. Widened
to `[EWI]`, keyed on letter-plus-number so `E3500` and `W3500` stay distinct.

`clippy.toml` disallows `HashMap`/`HashSet`, making determinism in the numeric
path a property of the type rather than of anyone remembering. Float sums
reassociate, so a map with unspecified iteration order there would produce
results that differ between runs of the same model — and the golden suite would
report it as a flapping test rather than as the nondeterminism it is. `cfdl-lsp`
and one never-iterated map in `cfdl-calc` are exempt at the declaration, with
reasons.

### Added: `cre.permanent_debt`

A commercial mortgage on a stabilized property — the CRE pack previously had no
debt contract at all, so every model hand-wrote its mortgage and
`domain.cre.dscr` worked only because the metric matched a stream *name* by
convention.

    contract cre.permanent_debt on entity asset.tower {
      term 2026-01..2035-12
      terms { principal = 6000000  rate = 0.055  amort_months = 300 }
    }

`amort_months` strikes the payment and is normally longer than the term — the
30-year-amortization-on-a-10-year-loan structure is what a commercial mortgage
is. Optional interest-only period; the balloon is opt-in via
`balloon_at_maturity` and defaults OFF, because coverage is measured on periodic
debt service and the standard pro forma repays the balance from the sale.

One combined stream, `loan.permanent_debt_service`, matching the exact name the
metric selects. Diagnostics `E6050`–`E6056`.

### Added: `opco.exit_perpetuity`

Terminal value as a growing perpetuity — the Gordon form. The pack could
previously express only a *multiple* of something, so the largest single
component of value in a DCF had no contract.

    TV = base_value * (1 + growth_rate) / (discount_rate - growth_rate)

`base_value` is the terminal-period flow **before** the `(1 + g)` step; the
contract applies it. `discount_rate` is a contract term, not the run's NPV rate:
a terminal cost of capital is the rate for a business in steady state, and the
published models that state these terminals build it from their own CAPM inputs.
The run's rate discounts the result; this one capitalizes it.

Diagnostics `E7025`–`E7029`. `E7025` guards `r > g`, below which the perpetuity
has no finite value.

### Added: two externally reconciled benchmark cases

- **`benchmarks/cre/one_lincoln_street`** — a real named Boston development.
  Reconciles the construction period funding and interest schedule across
  sixteen quarters: equity and loan draws exact to the dollar, interest within
  the source's own thousand-rounding. The equity commitment depletes mid-quarter,
  and that split falls out of a declared state rather than being stated.
- **`benchmarks/opco/gordon_growth_coned`** — nine published values across nine
  growth rates, spanning a sign change. **The first case in this repo to assert
  a value** rather than a cash flow or a ratio.

Both sources are redistributable and are committed under `reference/`, bringing
to four the number of sources a reader can open and check directly.

### Note: adding a CRE contract and a CRE source did not improve CRE coverage

Externally reconciled pack-contract coverage moved opco 4/10 → 5/11 and left CRE
at 1/13 — worse as a ratio, since the denominator grew. `cre.permanent_debt`'s
only user is an in-house case, and One Lincoln Street's funding waterfall needs
a construction-loan contract that does not exist. Recorded as backlog 7.15
rather than left to be inferred from the numbers.


### Added: a state has its own schedule

A `state` now takes the same `schedule` clause a stream does:

    state pool_survival {
      schedule every quarter from 2026-01 to 2031-01
      init 1.0
      next prev * (1 - hazard)
    }

The recurrence STEPS on that cadence and HOLDS between ticks and outside its
window. It does not fall to zero — that is what separates a schedule from
`active when`, which a state deliberately does not have.

This corrects the original design. `docs/14_state_and_recurrence.md` said "a
state has no schedule", conflating cadence with activity and dropping both, so
every state advanced once per MODEL period. Since a lowering rule's
`{{time.elapsed_periods}}` counts its own PAYMENT periods, a pool on a daily
book paying monthly would have compounded 365 times a year instead of 12. §8 of
that document records the correction.

Absent, a state steps every model period over the whole timeline, so nothing
already written changes. Pack rules gain `state_every` / `state_from` /
`state_to`.

### Added: PSA, SDA and the ABS prepayment model in the credit pack

`psa_speed`, `sda_speed` and `abs_speed`, each a MULTIPLE of the published
curve, plus `age_months` for a pool's seasoning at closing. All default to `0`,
selecting the existing flat `cpr`/`cdr` path.

The pool factor is now a per-period state rather than `pow(k, p)` — the closed
form of the running product only while the hazard is constant. Three externally
reconciled cases were blocked on this and now land:

  - `benchmarks/credit/auto_abs_speed_050`   0.0048 percentage points
  - `benchmarks/credit/auto_abs_speed_150`   0.0036 percentage points
  - `benchmarks/credit/mbs_pool_ramped`      within the source's rounding floor

New diagnostics `E9016`–`E9019`. Closes backlog 2.1.

Two convention defects were found by those external references after every
in-house identity already passed: all three ramps index from loan ORIGINATION
rather than the deal's closing (20 percentage points on a seasoned pool at
1.50% ABS), and the lagged pool factor the recoveries rules read was consuming
the hazard one lag too late (7.6% on recoveries by month 60). Both are recorded
in the cases' NOTES.

### Added: `make rule-fragments`

`tools/check-rule-fragments.py` asserts that repeated expression fragments in a
pack's lowering rules are byte-identical, normalising the age argument. Every
committed golden runs at a constant hazard, so nothing in the suite evaluates a
ramp branch; a typo in one of eighteen copies is invisible to it. Measured: a
10x typo in a shared `state_next` is caught by `E5021`, but the same typo in one
rule's `amount_expr` passes gold, benchmarks and analytic checks.

### Changed: pool factors are no longer decimal-exact

`pow(k, p)` was one decimal exponentiation; a state is `p` sequential
multiplications stored as `f64`. Measured at 4.6e-16 relative over 360 periods,
which publication rounding at six decimals absorbs — no committed golden moved.
Recorded because it is a real, if tiny, loss of exactness.


### Added: declared state variables

A `state` is a named number per period defined by a recurrence — the one shape
`pow(1 + r, t)` cannot express, since that applies a single period's rate as
though it had held from the start:

    state revenue_index {
      init  1.0
      next  prev * (1 + curve_value("growth", time.date))
    }

    stream firm.revenue on entity legal.firm inflow currency USD {
      schedule every year from 2026-01 to 2035-01
      amount = 21765.4 * state.revenue_index
    }

Language-level, not pack-level: a state has no entity, direction, currency or
schedule, and any model may declare one regardless of which pack it uses (or
none). Inside `next`, bare `prev` is this state's previous value and
`prev.<name>` is another's.

`init` is mandatory. An unstated base case would evaluate as a silent zero for
every period, since an unmatched lookup returns 0.

The safety property is preserved by ABSENCE rather than by a check: a `next`
environment carries no `state` map and a stream environment carries no `prev`
map, so a same-period read is not there to be found. Everything a state can see
is already finished, so no reference can close a cycle — "cycles are impossible
by construction" is intact, and states may reference each other mutually with
declaration order carrying no meaning.

Six diagnostics, `E1120`–`E1125`, each probed against a fixture that violates it.

States are published in `results.deterministic.series` as `state.<name>`, as
bare numbers with no currency and no offset. They are **not cash**: excluded
from `model_series`, `model.total`, `model.npv`, the annual rollup and every
domain metric, with an analytic identity asserting it.

Pack lowering rules may declare a state too (`state_name`, `state_init`,
`state_next`, plus a `{{contract.suffix_ident}}` placeholder), with
`E5020_LOWERED_STATE_INVALID` and `E5021_DUPLICATE_LOWERED_STATE`. The three
opco growth rules now compound through a running product; no model needed
editing.

Verified against two independent published sources:

  - the FCFF forecast: revenue drifted -2.4% by year 10 and years 6-10 were
    unasserted; now all ten years agree to floating-point noise
  - the HUD multifamily pro forma: a 12.26 residual under `period_tolerance = 13`
    is now exact, with the tolerance at 0.5 — the whole-dollar rounding floor

Across all 110 goldens the only movement from the pack change is
`7365967.000481 -> 7365967.00048` (1.4e-13 relative) and a `-0.0 -> 0.0`.

Closes backlog 5.1 and 7.2; supersedes most of 7.8. See
`docs/14_state_and_recurrence.md`.

### Changed: `Series.values` may hold a bare number

`MoneySeries` is renamed `Series`, and its `values` becomes
`Money | number` — cash carries a currency, a state does not. The results
schema always permitted a number here, so no published shape changed.

Consumers that weight or sum cash take `SeriesValue::money_amount()`, which
returns `None` for a non-money series.

### Added: `ln` and `exp`

Two builtins that turn a cumulative **product** into a cumulative **sum**:

    PROD(1 + r_i)  ==  exp(series_sum("ln_one_plus_r", 0, t))

A survival factor or growth path under a *varying* rate has no closed form, and
`pow(1 + r, t)` is not it — that applies one period's rate as though it had held
throughout. Verified end to end: the identity reproduces all ten years of a
published forecast with a decaying growth rate exactly, where `pow` drifts to
-2.4% by year 10.

Both escape to float64, as `pow` already does for fractional exponents, so they
are **not decimal-exact**. Prefer a closed form where one exists.

Note the technique is not yet usable from a pack rule: the helper stream
carrying `ln(1 + r_t)` is counted as cash. See backlog 7.8.


### Breaking: three diagnostic codes renumbered

A diagnostic code is an identifier — what a user greps for and what a tool
matches on. Three named two different checks each:

| was | is now | check |
|---|---|---|
| `E7010` | **`E7013`** | `OPCO_WC_MISSING_AMOUNT_OR_RULE` |
| `E7011` | **`E7014`** | `OPCO_WC_INVALID_SCHEDULE` |
| `E6030` | **`E6033`** | `CRE_UNIT_INVALID_ESCALATION` |

The ambiguity checks keep `E7010`, `E7011` and `E6030`; they form a family and
are documented as such. Anyone matching on the three old codes for the
working-capital or unit-escalation meanings needs to update.

### Two thirds of pack validations were never running

A validation matches a contract by exact name unless it declares
`match = "instance"`, and contracts are routinely written suffixed
(`opco.revenue_line.core`). 33 of 48 shipped validations lacked the flag and
were silently skipped on the form models actually use — `E7001` rejected
`opco.revenue_line` with no amount and accepted `opco.revenue_line.core` with no
amount.

All 48 now declare it. No golden moved: eight previously-dormant checks are live
and every shipped model already satisfied them.

`tools/check-pack-validations.py` joins `make ci`, enforcing that codes are
unique **and** that every validation states its match mode explicitly. `exact`
remains available; it just has to be written, because defaulting was the trap.


### Breaking: WAL and payback are measured on the discounting time axis

`model.wal_years`, `domain.credit.wal_years` and `model.payback_years` weighted
a period-0 cash flow at **t = 0**. The market convention — the one a prospectus
states as "the number of years from the closing date to the related
distribution date" — puts an ordinary annuity's first monthly collection at
1/12 of a year. Credit models put their first collection in period 0, so every
WAL this engine has ever reported was one period short.

Reconstructed from an issuer-published auto-ABS schedule, the effect is not
academic: a class with a published WAL of 0.37 years came out at 0.286, a 23%
understatement. Short amortizing deals are hit hardest, because one period is a
larger share of a shorter life.

A flow's time is now `(period + offset) / ppy`, where `offset` is the same
placement `npv_with_offsets` discounts on (`docs/12_payment_timing.md`). So NPV,
IRR, WAL and payback now agree about when a dollar arrived. Consequences:

- a bullet's WAL is exactly its term (it reported term − 1 period);
- an annuity due's WAL is exactly one period shorter than the equivalent
  ordinary annuity's (they were identical);
- `mid` sits exactly halfway between the two (it was indistinguishable);
- the same deal has the same WAL on any calendar (an annual grid was a full
  year out).

All four, plus a payback identity, are now asserted in
`tools/analytic-checks.py` — they fail on the previous engine and pass on this
one. Nothing else could have caught this: the three credit benchmarks asserted
WAL against reference generators that restated the same off-by-one, so both
sides agreed for as long as they existed. The generators are fixed here
independently of the engine, and their agreement afterwards is the check.

Time-weighted metrics now net **within** an offset rather than across one: two
flows in one period at different points in it are not the same cash at the same
moment, so a purchase settling on its date no longer cancels that period's
collections. Where every stream shares a placement this is exactly the previous
behavior. `model.moic` is deliberately unchanged — it is a ratio of cash in to
cash out and does not depend on when the cash moved.

Numbers that move:

| benchmark | `model.wal_years` | `domain.credit.wal_years` |
|---|---|---|
| `credit/level_pay_pool` | 3.817027 → 3.843940 | 3.973633 → 4.056967 |
| `credit/io_bullet_loan` | 3.812188 → 3.864922 | 4.244941 → 4.328274 |
| `credit/float_bridge_pool` | 2.313942 → 2.367044 | 2.456847 → 2.540180 |

The domain metric moves by exactly 1/12; `model.wal_years` moves by less,
because period 0's collections were being annihilated by the purchase and now
re-enter the denominator at 1/12 year. 56 goldens move `model.wal_years` and 16
move `model.payback_years`; no golden gains or loses a metric key.

### The published results schema is a gate, and was wrong

Every one of the 67 committed results goldens violated
`docs/schemas/results.schema.json`, and had since 0.3.0 — four releases:

- `results_version` declared `const "0.1"` while the engine has emitted `"0.2"`
  since 0.3.0. The one field whose entire job is to say which shape a document
  has was itself wrong in every document;
- `deterministic.annual_rollup` was emitted by 62 goldens and undeclared;
- the root-level `domain_metrics` was emitted by 8 and undeclared.

Fixed, and gated: `tools/check-results-schema.py` joins `make ci`, the sibling
`check-ir-schema.py` has had since the IR schema drifted the same way.

`docs/06_results_schema.md` was an independently maintained copy of the same
JSON and had drifted further. It is now generated from the schema, and the gate
checks all three copies agree — the site mirror, the doc page, and the source of
truth. Three copies of one contract, only ever one of them read, is how this
happened in the first place.

### CI ran five fewer gates than `make ci`

`bench`, `analytic`, `cadence-parity`, `ir-schema` and `results-schema` existed
only locally, so they fired when someone remembered. That is how the weighted
average life defect above survived — the identity that catches it lives in
`analytic-checks`, which the workflow never executed. All five now run in CI.

### The compiled Python extension has a freshness gate

`cfdl_sdk` is half editable Python and half a compiled Rust extension. The
Python half tracked the working tree; the compiled half was rebuilt only on
`make py-develop` and nothing said when it had gone stale. It went stale, and
`make notebooks-render` failed with

    E4004_MISSING_PACK: unknown variant `terms_mutually_exclusive`

which reads like a broken pack and was nothing of the sort — the extension
predated the commit that added that validation kind. `tools/py-stamp.py` hashes
the sources the extension is built from, `make py-develop` stamps it, and
`notebooks-render` / `notebooks-check` check it first and name the remedy.

A source hash rather than a version check, for the same reason the wasm bundle
uses one: the commit that broke this shipped no version bump, and it changed a
pack TOML rather than any Rust source.

### Added

- `MoneySeries.offset` in the results document — a series' placement in its
  period, published so a consumer holding `results.json` can recompute the
  time-weighted metrics the engine reported. Optional and additive; absent on
  aggregates, which sum streams whose placements differ.

---

## [0.7.0] - 2026-07-28

Schedules, contract terms and the published surface. Breaking: see below.

### Schedules honour what they declare

A stream's recurrence interval was discarded at parse time, so every stream
paid in every period. This release completes the fix end to end.

- An interval finer than the model's calendar is rejected
  (`E2108_SCHEDULE_FINER_THAN_CALENDAR`) rather than collapsed. A weekly
  schedule on a monthly grid paid twelve times a year instead of about
  fifty-two: several occurrences fall in one period, and a period holds one
  payment. This is section 10.3's own rule, finally implemented.
- A lowering rule may declare `schedule_every`, so a pack can express a
  quarterly coupon or an annual true-up rather than being pinned to the
  calendar cadence. Unset means the cadence, which is every shipped rule.
- `stub` is rejected instead of accepted and discarded — a model could ask
  for a short front stub and silently receive a full period.
- The doc-examples gate counts payments: a stream may not pay in more periods
  than its schedule declares. That is the check that would have caught the
  original defect.

### Contract terms

A term is a literal or a reference to one declared input. Trailing tokens are
rejected: `rent_year = 12 * 8500` compiled as `12`, silently, in any pack.

A term naming an input defers to it, so scenarios and Monte Carlo drive it
through the one channel they already write to. Terms were previously baked
into lowered expressions as literals, so a Monte Carlo run sampled a variable
the expression did not contain and returned a degenerate distribution with no
warning.

### Currencies

`model "x" currency INR` parses, and every metric reports in it. Pack
lowering rules no longer hardcode USD — a PPA in Rajasthan is not a USD
contract — and a stream whose currency differs from the model's is rejected
rather than summed as though the units matched.

### The published surface describes the language that exists

The EBNF splits `cadence` from `interval`, documents `due`, and drops the
`stub` and weekday productions nothing implements.

The IR schema was public at cfdl.dev/schemas and checked against nothing. It
listed `metrics` as required though no compiler emits it, declared `stub` and
weekday rules that are never produced, and used `oneOf` for a union whose
members overlap, which could never be satisfied. `tools/check-ir-schema.py`
validates every IR golden against it and is part of `make ci`; it immediately
caught an `on eom` rule emitting `day: 0` against its own 1..31 bound.

The pack manifest documentation described a format the loader never read — a
pack written to it would have loaded with no entrypoints at all.

### Tooling

- The four standard packs are built into the CLI, so `cfdl compile my-model`
  resolves `use pack` with no flag and no download. A packs directory that
  holds packs stays authoritative.
- A pack present at a different version says so, naming both versions,
  instead of reporting "not found".
- `cfdl validate` applies the same `./packs` default as compile and run.
- Object ids no longer depend on the compiler version. Every release rewrote
  every id, churning goldens and making a downstream store treat the same
  entity as new after an upgrade. Ids move once here and should not again.
- `run.json` gained a JSON Schema, all five distributions, `clip`, and
  rejection of unknown keys — which found twelve example configs running
  undiscounted while claiming 0.1.

### Breaking

- `stub`, schedules finer than the calendar, mixed-currency models, and terms
  with trailing tokens no longer compile.
- Schedule intervals are singular nouns: `every month`, not `every monthly`.
- Object ids change once, as described above.
- A pack rule pinning a currency the model does not declare is rejected.

---

## [0.6.0] - 2026-07-28

Packs work outside the United States. Breaking: see below.

### Lowering rules inherit the model's currency

All 58 lowering rules across the four packs hardcoded `currency = "USD"`, and
the compiler fell back to the model's currency only when a rule left the field
empty. An INR model using the energy pack therefore reported INR metrics over
USD-labeled streams — a PPA in Rajasthan is not a USD contract.

Nothing caught it. `E2107_STREAM_CURRENCY_MISMATCH` lives in `cfdl-validate`,
which runs on the AST and so sees only hand-written streams; pack-lowered
streams are generated afterwards and bypassed it. The guarantee 0.5.0 made —
that currencies cannot be silently mixed — held for hand-written models and
not for pack-based ones, which is every serious model. The check now also runs
where lowered streams are built.

Rules omit `currency` rather than defaulting it to USD, because the default
already exists one level up: an unset rule currency takes the model's, and a
model that declares none takes USD. Two defaults would shadow each other and
reinstate the bug. An empty value is a deferral, not a missing value — the same
shape as a term deferring to a declared input. Pin a currency only when the
instrument is genuinely fixed to one, and the model must then agree.

No golden moved, which is the check that the fallback is wired correctly:
every model in the repository is USD, so the inherited value is identical.

### The packs archive ships what the docs promise

`package_packs.sh` archived only `cre` and `opco` while the install page
promised all four, so the flagship energy pack was undownloadable for anyone
without a checkout. It now discovers packs by their manifests rather than
listing them, so a new pack ships automatically.

`verify_release_assets.py` previously checked only that the archive's filename
existed. It now looks inside — a tarball missing half its packs passed three
releases undetected.

### Breaking

- A pack lowering rule that pins a currency the model does not declare is
  rejected with `E2107_STREAM_CURRENCY_MISMATCH`. No shipped rule pins one, so
  this affects third-party packs only.
- `LoweringRule.currency` is optional. Packs that omit it now inherit the
  model's currency instead of failing to parse.

---

## [0.5.2] - 2026-07-28

Release-pipeline fixes. No behavior change: the compiler, engine and packs
are identical to 0.5.0.

- `distribution/scripts/package_docs.sh` named three documents that were
  renamed in the docs restructure, so `tar` exited non-zero and the docs
  archive failed to build on every tagged release. It now archives the docs
  tree wholesale, which cannot drift as files are renamed.
- The server image failed at `cargo build -p cfdl-server`:
  `utoipa-swagger-ui`'s build script downloads the Swagger UI bundle at
  compile time and shells out to `curl` when its reqwest feature is off, and
  `rust:1-slim` ships neither `curl` nor CA certificates. Both are now
  installed in the builder stage.
- Adds a `.dockerignore`. The image builds from the repository root with
  `COPY . .` and had no ignore file, so `target/`, `node_modules/` and `.git`
  were all being sent as build context.

Together with 0.5.1 this makes the full release pipeline green for the first
time — the VS Code extension lockfile, the docs archive and the server image
had each been failing independently since v0.3.0 or earlier.

---

## [0.5.1] - 2026-07-28

Release-pipeline fixes. No behavior change: the compiler, engine and packs
are identical to 0.5.0.

- The VS Code extension's `package-lock.json` still declared `0.0.1` while
  `package.json` tracks the project version, so `npm ci` refused to install
  and the Extension lint step failed on every tagged release from v0.3.0
  onward. The lockfile now carries the real version and is bumped with it.
- Playground examples were stale against the repo's models: the schedule
  syntax migration in 0.4.0 changed the `.cfdl` sources without regenerating
  them, and the site workflow had been failing on `main` as a result.
- Monte Carlo dispersion is asserted as a property in
  `tools/analytic-checks.py` rather than as a golden. A long run over a
  pack-lowered expression containing `pow()` is not bit-identical across
  platforms, so it passed locally and failed on Windows CI. The golden keeps
  its deterministic scenario sweep.

---

## [0.5.0] - 2026-07-28

Contract terms, stochastic layering, and currencies. Breaking: see below.

### Contract terms are a literal or one declared input

A term kept only the first token after `=` and silently discarded the rest, so
`rent_year = 12 * 8500` compiled as `12` — no diagnostic, and no validation
caught it because `12` parses cleanly. That is now an error
(`E0004_EXPECTED_TOKEN`), and a term is defined as either a literal or a
reference to one declared input:

```cfdl
assume annual_yield ~ Normal(mean=5000, stdev=350, clip=[4000, 6000])

terms {
  ppa_price = 3000                 // contractual fact
  mwh_year  = inputs.annual_yield  // driver, supplied per run
}
```

Contracts stay declarative records of what was signed; anything that varies is
named and supplied from outside. Because `inputs.*` is the single channel that
scenarios and Monte Carlo already write to, one declaration serves a fixed
case, a scenario sweep and a stochastic run alike.

This also fixes Monte Carlo through pack contracts. Terms were baked into
lowered expressions as literals, so a trial sampled a variable the expression
did not contain and returned a degenerate distribution with no warning.

- `E5010_TERM_UNKNOWN_INPUT` — a term naming an input that was never declared.
- `E5011_TERM_CLIP_OUT_OF_BOUNDS` — a deferred term's value cannot be checked
  at compile time, but its distribution's `clip` states the range it can reach,
  so where a pack declares bounds the clip is checked against them.
- `E5009_LOWERED_EXPR_INVALID` — pack-lowered amount expressions are now
  compile-checked. The engine evaluates a failed expression as zero with only a
  warning, so a malformed expansion became a silently empty stream.

### Model currency

`model "x" currency INR` now parses; every metric is denominated in it, and it
defaults to USD when omitted. Streams must agree with it: cash flows are summed
period by period, so a 500 USD outflow in an INR model was being subtracted as
500 INR, producing a total in no currency at all
(`E2107_STREAM_CURRENCY_MISMATCH`). Cross-currency models require an explicit
conversion in the amount expression — the language applies no implicit FX.

### Run configuration

- All five distributions (`fixed`, `normal`, `uniform`, `log_normal`,
  `triangular`) and `clip` now work from `run.json`, matching what
  `assume x ~ Dist(...)` offers. `stdev` is accepted alongside `stddev`.
- Unknown keys are rejected. Parsing was lenient and the override consumers
  ignore unrecognized keys, so a misspelling produced a clean run with wrong
  numbers and no warning.
- `docs/schemas/run.schema.json` — the format had no schema at all.
- An in-source `run monte_carlo trials N seed S` is honoured. It was parsed and
  lowered, then dropped by the engine, so a model asked for trials and got a
  single deterministic pass. An explicit run config still wins.

### Breaking

- Terms with trailing tokens, mixed-currency models, and unknown run-config
  keys now fail to compile or run.
- Twelve example run configs set `discount_rate`, which is not the wire name
  (`annual_discount_rate`) and was therefore ignored — those examples ran
  undiscounted while claiming 0.1. Migrated rather than aliased, so the
  correction is visible; their numbers change.

---

## [0.4.0] - 2026-07-28

Payment timing. Breaking: discounted metrics change for every model.

### Schedules honour the declared interval

A stream's recurrence interval was discarded — the parser dropped the token and
the compiler substituted the model's calendar frequency — so every stream paid
in every period. A model written `every quarterly` on a monthly grid paid twelve
times a year, silently. Intervals are now parsed, required, and honoured.

Interval and cadence became separate words because they are separate concepts:
a calendar is adjectival and describes the grid (`time calendar monthly`); an
interval is a noun and describes how far apart one stream's payments fall
(`every month`). Only intervals have a weekly member.

`on day <n>` and `on eom` work for the first time. The compiler had always
emitted the rule; the engine had no field for it and dropped it on
deserialization.

### Payment timing is specified and discounted correctly

A payment belongs to the period that earned it. What separates the two annuity
conventions is where in that period the cash falls, and therefore how far it is
discounted — one mechanism rather than three special cases:

| Schedule | Position | Discounted from |
|---|---|---|
| `due` | start | period start |
| default, `on eom` | end | period end |
| `on day <n>` | day n | that point in the period |

This is Excel's convention, matching `pmt(rate, nper, pv, [fv], [due])` in the
expression library. Mid-period discounting follows from the same rule.

Written honestly, a five-year par bond now returns an NPV of exactly zero — the
identity that exposed the defect, since the first coupon previously landed
undiscounted and the final year fell off the end of the range.

See `docs/12_payment_timing.md`.

### Verification against closed-form finance

`tools/analytic-checks.py` asserts identities drawn from the definition of
present value, so they hold for any correct implementation and cannot be
satisfied by making two implementations agree: a par bond is worth par, a level
annuity matches `(1-(1+i)^-n)/i`, an annuity due is worth `(1+i)` times the
ordinary annuity, and a fully-amortizing loan is worth its principal. Part of
`make ci`.

The benchmark suite compares each model against a reference implementation,
which cannot detect a convention both sides share — that is how the original
defect survived eight passing benchmarks. Every reference was corrected to
separate one-shot flows from recurring ones.

### Breaking

- Discounted metrics (NPV, IRR, and anything derived) change for every model.
  Undiscounted cash flows are unchanged for models scheduling at their calendar
  frequency, which was every model in the repository.
- Schedule intervals are spelled as singular nouns: `every month`, not
  `every monthly`. The interval is now required after `every`.

---

## [0.3.0] - 2026-07-27

First public release. CFDL is pre-1.0: the language and IR spec is v0.1, and
interfaces may change until 1.0 freezes the IR and Results schemas.

### Language and engine

- Deterministic compilation: the same sources, pack version and compiler
  version emit byte-identical IR, enforced by a golden suite.
- Native `cfdl-calc` expression engine with decimal-exact money arithmetic and
  an Excel-compatible function library (annuities, day counts, business-day
  calendars, MACRS, prepayment conversions).
- Deterministic DCF, scenarios, and seeded Monte Carlo, emitting
  schema-governed Results JSON.

### Domain packs

- `energy`, `cre`, `credit` and `opco`, each supplying contract types,
  template-driven lowering rules, domain metrics, and declarative validations.
- Every pack is gated by a parity suite: each model is diffed period-by-period
  against an independent reference implementation.

### Surfaces

- CLI (`cfdl compile`, `cfdl run`, `cfdl validate`).
- Python SDK (`cfdl_sdk`) with pandas result accessors.
- WebAssembly build powering the in-browser playground at cfdl.dev.
- HTTP API server, and a VS Code extension with LSP diagnostics.

### Licensing

- Business Source License 1.1 (source available, not open source). Each
  released version converts to Apache-2.0 four years after its release.
