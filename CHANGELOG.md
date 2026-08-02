# Changelog

All notable changes to this project will be documented in this file.

This project follows Semantic Versioning: https://semver.org/

---

## [Unreleased]

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
understatement. Short amortising deals are hit hardest, because one period is a
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
behaviour. `model.moic` is deliberately unchanged — it is a ratio of cash in to
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
USD-labelled streams — a PPA in Rajasthan is not a USD contract.

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

Release-pipeline fixes. No behaviour change: the compiler, engine and packs
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

Release-pipeline fixes. No behaviour change: the compiler, engine and packs
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
  ignore unrecognised keys, so a misspelling produced a clean run with wrong
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
ordinary annuity, and a fully-amortising loan is worth its principal. Part of
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
