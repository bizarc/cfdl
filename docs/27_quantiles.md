# Quantiles — a value indexed by cumulative share

Status: **stages 1 to 3 shipped**. The declaration, the three functions, the
provenance wiring and the first pack contract to consume them are built and
specified in `docs/01` §12.6. Stage 4 in §9 — `energy.storage_dispatch` — has
not been built, and is gated on whether a dispatch reference can be made to run
at all.

Some quantities are not one number, and not one number per date either. They
are a *spread of values within a period*, and the thing the model needs from
them is a slice: the top 2% of hours, the sales above a breakpoint, the losses
between two attachment points.

The language has no way to say that. This document proposes one construct,
three functions, and the guardrails that keep them inside the language's
existing commitments.

## 1. What forced it

`energy.storage_arbitrage` prices a battery as
`mwh_cycled_year * spread * (1 - degradation)^y`. Backlog 7.1 records that the
rule has no external validation and that the reason is circular: `mwh_cycled_year`
is an input to our rule and the primary *output* of the dispatch model it would
be validated against, so the two cannot be compared without first deciding the
answer.

The deeper problem is that storage revenue is a **dispersion functional**. It
depends on the spread of the price distribution, not its level — a
mean-preserving spread strictly increases it, because a battery discharges only
into the upper tail and charges only from the lower one. `spread` is a
hand-supplied stand-in for that dispersion, which is why fitting it is
calibration rather than validation.

**The same defect is already shipped in another pack.** `cre.percentage_rent`
lowers to:

```
max(0, sales_year * pow(1 + sales_growth, elapsed_years) - breakpoint_year) * overage_pct
```

That is a call option on tenant sales evaluated at a point estimate of sales.
At any point estimate below the breakpoint it returns exactly zero, when the
true expectation is positive. `benchmarks/cre/retail_strip` exercises it.

Two packs, one shape: a nonlinear payoff evaluated at a summary statistic
instead of over the distribution the payoff is nonlinear in.

## 2. The construct already has an ontology home

`reference` is the fourth entity family — a market observable
(`crates/cfdl-pack/src/lib.rs`, `ENTITY_FAMILIES` and `OntologyReference`). All
four packs declare references today:

```toml
[[references]]
reference_id = "energy.power_price"
kind = "price_curve"
unit = "USD/MWh"
```

`docs/18` §5 settles the governing rule — *every quantity belongs to something,
and a reference is one of the things it can belong to* — then names exactly two
mechanisms for the family:

- **known values** become a `curve`, which §5 describes as "already a declared,
  date-indexed reference";
- **compounding values** become a reference entity with a recurrence.

A price duration curve is a third mechanism in that same family: known values,
indexed by dispersion rather than by date. This is not a new concept grafted
onto the language. It is a missing member of a set `docs/18` already enumerated.

Two channels are reserved for it and currently empty. `ref.<name>` is reserved
and "not in the v0.1 dialect" (`docs/01` §16.2). `required_refs` is required by
`docs/01` §17.3 and hard-coded to `vec![]` in `crates/cfdl-compile/src/lib.rs`.

## 3. Why the name is `quantile`

The name was measured, not chosen. Counts are occurrences in published prose
(`site/content`, `learn/content`, `docs/*.md`) and in sources
(`crates/*/src`, `packs`), taken the way `docs/terminology.toml` takes them.

| Candidate | Prose | Source | Verdict |
|---|---|---|---|
| `quantile` | 0 | 0 | Adopted |
| `exceedance` | 0 | 0 | Energy and insurance native; commits to an orientation §5 normalizes away |
| `stratification`, `strata` | 0 | 0 | Structured-credit native; reads as borrowed elsewhere |
| `dispersion` | 0 | 0 | Names a scalar property, not a function |
| `spectrum`, `ogive`, `population`, `census` | 0 | 0 | No domain uses them; would read as invented |
| `profile` | 10 | 2 | **Rejected** |
| `distribution` | 279 | 82 | **Rejected** |
| `deck` | 37 | 0 | **Rejected** |
| `shape`, `tier`, `spread`, `layer`, `bucket`, `cumulative` | 164 / 66 / 70 / 45 / 36 / 60 | | Heavy general use |

Three rejections are load-bearing and are recorded here so the question is not
reopened from taste:

**Not `profile`.** Every one of its ten occurrences in this corpus means a
*time-ordered path* — "the collection profile shortens", "a development's
funding profile", "generation profile", "decline profile". A chronology-free
object cannot take the corpus's existing word for a chronological one without
breaking one word, one meaning, which is the rule `docs/terminology.toml` exists
to enforce.

**Not `distribution`.** In CFDL that word means cash paid down a waterfall. It
names an engine stage (`crates/cfdl-engine/src/distributions.rs`), a registered
Technical Name, and a lesson (`docs/26`, "Distributions never reach a cash flow
statement, by design"). Reusing the most load-bearing word in the waterfall
layer for a probability object is the worst available collision.

**Not `deck`.** "Price deck" is already an informal synonym for a date-indexed
`curve` in `docs/01` §12.5 and `docs/09`.

**And not a domain term.** The precedent is `curve` itself: the language took
the neutral noun and left "price deck" and "rate curve" in prose. A construct
serving four domains must not adopt one domain's word, or the other three read
as borrowing. The industry terms stay in documentation, where they belong:

| Domain | Terms of art |
|---|---|
| Power | price duration curve, load duration curve, merit-order stack, P50/P90 exceedance |
| Structured credit | stratification tables, loss distribution, severity distribution, attachment and detachment points |
| Insurance | exceedance probability curve, loss exceedance curve, layer |
| CRE | unit mix, rent band, sales distribution |
| OpCo | revenue mix, customer concentration, price ladder |
| Hydrology | flow duration curve |
| Statistics | quantile function, inverse CDF, percent-point function |

`quantile` gives the parallel the language wants: **a curve is indexed by when,
a quantile is indexed by how much.**

```cfdl
curve    sofr        { 2026-01: 0.048 }   // value at a date
quantile ercot_north { 0.98: 340.0 }      // value at a share
```

## 4. The declaration

```cfdl
quantile ercot_north linear by exceedance ref energy.power_price {
  1.00: 512.0        // scarcity hours
  0.98: 340.0
  0.90:  61.0
  0.50:  28.0
  0.00:  11.0
}
```

Grammar:

```ebnf
quantile_stmt   = "quantile" IDENT [ quantile_interp ] [ quantile_order ]
                  [ "ref" qname ]
                  "{" quantile_point { [ "," ] quantile_point } "}" ;
quantile_interp = "step" | "linear" ;
quantile_order  = "by" ( "quantile" | "exceedance" ) ;
quantile_point  = NUMBER ":" [ "-" ] NUMBER ;
```

Four decisions, each with its reason.

### 4.1 x is normalized to [0, 1]

The physical measure — 8760 hours, $100m of pool balance, a store's square feet
— stays in the contract term. A quantile is a *shape*; scale is not its
business. One ERCOT price stack then serves a 20 MW battery and a 200 MW battery
with no edit, which is the property that makes the construct reusable at all.

### 4.2 Orientation is surface only

The IR stores exactly one canonical form: ascending quantile, x rising from 0.
`by exceedance` reverses at parse time and does not survive into the IR.

Energy and insurance authors write duration curves descending and should keep
doing so. Credit strats read ascending. Both spellings normalize to one stored
form, so every downstream consumer — engine, results, statements, tooling —
sees one orientation and no consumer carries a sign convention.

This is `docs/26`'s "One axis is one field, not three booleans" applied before
the mistake rather than after it. The contradictory state is unwritable rather
than rejected at runtime.

### 4.3 Quadrature is derived, never declared

`step` and `linear` keep the meanings they have on a `curve`. The integral is
defined as the **exact integral of the interpolated function** — rectangles
under `step`, trapezoids under `linear`.

So there is no second flag for quadrature, no approximation error, and neither
word quietly acquires a new meaning. A reader who knows what `linear` does to a
curve already knows what it does here.

### 4.4 `ref` is optional and is what buys the provenance

Naming a pack reference links the quantile to its `[[references]]` entry, which
already carries `kind` and `unit`. That gives unit checking against a rule's
`[rules.units]`, and it gives `required_refs` its first real content since v0.1.

## 5. The three functions

The expression vocabulary is engine-owned and fixed; packs compose primitives
and do not define functions (`docs/01` §3.4, `docs/07` §6.7). These three are
engine-level.

| Call | Returns |
|---|---|
| `quantile_at(name, x)` | the value at cumulative share `x` |
| `quantile_mean(name, from, to)` | the mean value over the slice `[from, to]` |
| `quantile_of(name, value)` | the share at or below `value` |

`quantile_of` is the inverse of `quantile_at`, and it is what makes the
construct cross-domain rather than energy-specific: it turns a *stated value
threshold* — a lease breakpoint, a tranche attachment point — into an x
coordinate. Without it, only quantities already expressed as percentiles could
be sliced.

All three payoffs then fall out of the same three calls:

```
storage  = (quantile_mean(p, 1-h, 1) - quantile_mean(p, 0, h) / efficiency) * mwh

overage  = (1 - x) * (quantile_mean(s, x, 1) - breakpoint) * pct
           where x = quantile_of(s, breakpoint)

tranche  = a partial expectation between quantile_of(l, attach)
           and quantile_of(l, detach)
```

`quantile_at(name, x)` deliberately parallels `curve_value(name, date)` in
argument order and in shape, so the two constructs are learned together.

## 6. How this holds the language's commitments

**Deterministic compilation** (`docs/01` §1.1.2). Nothing samples. Points are
literals. `quantile_mean` is a closed-form integral of a piecewise function:
no RNG, no iteration, no convergence tolerance. Same inputs, same IR, same
`ledger_hash`.

**Replayable — and here is the temptation to refuse.** Points are inlined into
the IR and therefore inside `model_hash`. An 8760-point stack in a `.cfdl` file
is ugly, and the fix will look like pointing at a CSV. Refuse it: an external
path puts the audit chain outside the hash, and a results document whose
`model_hash` does not cover its own price assumption is not reproducible in any
sense worth the word. A duration curve is already a compression of 8760 hours —
10 to 30 points is the normal size — and `import` organizes files without
leaving the hash.

**Provenance** (`docs/01` §17.2, §17.3). A `ref` clause populates
`required_refs`, which has been declared and empty since v0.1.

**Auditable.** The results document's `InputsSection` calls itself "what went
in, above the line items — the top of the audit chain", and publishes resolved
`assume` values so a deterministic run's central values are not invisible. The
resolved *slices* need exactly the same treatment. A reviewer must be able to
read "the top 2% of hours averaged 340.00 USD/MWh, and that is the number that
struck the revenue", not merely "a quantile was declared". `resolved` is keyed
by `inputs.<name>`, so this wants a sibling `quantiles` key — a small additive
change to `docs/schemas/results.schema.json`.

**Built as described**, with one refinement the design did not anticipate: the
record is per CALL SITE rather than per quantile. A model asks several
questions of one declaration — the top 2%, the bottom half, the share below a
breakpoint — and it is the questions that explain the numbers, not the
declaration they were asked of.

Publishing the resolved slice is not a nicety. A nonlinear input whose
evaluation is not published is a number no reviewer can check.

## 7. Non-goals, and why each is hard

These belong in the specification, not in a reader's inference.

1. **A quantile is univariate. Full stop.** A joint quantile over price and load
   is correlation through the back door, and `docs/01` §1.1.10 and §17.4 forbid
   it: "The IR MUST NOT contain any correlation field/slot." This is the first
   extension anyone will ask for and the answer is no.

2. **A quantile is never sampled.** It is data, not an uncertainty axis. An
   `assume ~ Dist` draws one scalar per trial
   (`crates/cfdl-engine/src/lib.rs`, the trial loop) and collapses to a central
   value outside Monte Carlo; a quantile is consumed whole, every period, in
   every trial. Uncertainty *about* a quantile is an `assume` scaling it. The
   two compose and stay separate — fusing them would reintroduce the very error
   the construct exists to remove.

3. **The slice is stated, not solved.** Choosing `h` to maximize revenue is
   optimization, excluded by `docs/01` §1.2.

4. **Chronology is discarded by construction.** A quantile cannot see that a
   four-hour battery is unable to reach eight non-contiguous peak hours. That
   residual error must be *stated* in the pack documentation, not hidden. It is
   precisely the quantity backlog 7.1 asks to be bounded.

## 8. What this does and does not close

It closes the **Jensen gap** — evaluating a dispersion functional at a point
statistic. It does not close the **chronology error**. Backlog 7.1 asks for a
bound on the total, so the construct is necessary and not sufficient there.

What it does change is that the comparison becomes possible at all. A price
duration curve is a summary of the hourly price series that a dispatch model
*also takes as input*, so it is an input on both sides. Cycled energy becomes an
output of our rule, derived from power rating, duration and efficiency, and
comparable to the reference's output. That is validation rather than
calibration, which is the objection 7.1 raises and this answers.

## 9. Staging

Each stage is independently landable and keeps the suite green.

1. ~~**Language core.**~~ **SHIPPED.** Declaration, three functions, IR node,
   `docs/01` §12.6, the grammar, and fixtures. No pack changes. The one
   departure from this document as written: the functions are `quantile_at`,
   `quantile_mean` and `quantile_of` rather than the `curve_area` this
   originally proposed — an area is not what the payoff needs, a partial
   expectation is, and `quantile_of` was added because without an inverse only
   quantities already stated as percentiles could be sliced.
2. ~~**Provenance wiring.**~~ **SHIPPED.** `required_refs` is populated from
   `ref` clauses — its first content since v0.1, having been declared by §17.3
   and hard-coded empty. `InputsSection.quantiles` publishes every call site
   with the slice it asked for and what that resolved to.

   Two decisions worth recording. The record is built at COMPILE time and
   passed through verbatim, the way `stream_inputs` already is, so the engine
   and the per-period evaluation path are untouched; and it walks the
   assembled IR rather than the statement list, so an expression is reached
   wherever it sits — a stream amount, a field's `next`, an event guard, a
   waterfall step — and a construct added later is covered without changing
   this code.

   A call whose arguments are not literals cannot be resolved at compile time.
   It is published WITHOUT a value rather than dropped, because a missing call
   site would read as a model that never made one. Resolved values are rounded
   to the engine's published-number policy, so the figure agrees exactly with
   the ledger figure it explains and no last-bit float noise reaches
   `model_hash`.
3. ~~**`cre.percentage_rent` takes an optional sales quantile.**~~ **SHIPPED**,
   as a SIBLING contract rather than an optional term. This document said
   "optional sales quantile" before it was known that a lowering rule has
   exactly one `amount_expr` and no conditional. The pack's own answer to a
   variant is a separate contract type — opco carries three exit forms — so
   `cre.percentage_rent_expected` sits beside `cre.percentage_rent`, which is
   left byte-untouched and keeps `benchmarks/cre/retail_strip` reconciling.

   The distribution is stated in currency and carries the level, so no
   `sales_year` term sits beside it to disagree with. Escalation rides outside
   on `E[max(0, kS - B)] = k * E[max(0, S - B/k)]`.

   **The gap is measured.** A 1,200,000 breakpoint against sales expected at
   1,000,000 pays 0.00 at the point estimate and 4,937.50 a year over a
   distribution with that same mean, so the two differ in the shape of the
   payoff and in nothing else. The whole payment is the Jensen gap, and a
   breakpoint above expected sales is the ordinary case.
4. **`energy.storage_dispatch`** (backlog 7.5). Gated on whether a dispatch
   reference can be made to run at all — `Battwatts` discharged 27.9 MWh across
   a year behind the meter and segfaulted reconfigured front-of-meter, so the
   reference is a scoping exercise of its own.

Stage 3 is deliberately ahead of stage 4. It proves the primitive is not an
energy special case, and it lands a measured result while the energy reference
is still unresolved.

## 10. What has to be updated when this lands

- **Specification**: `docs/01` §12.6 and the reserved-keyword register in §18,
  which is checked against the lexer.
- **Grammar**: `docs/02` and `docs/schemas/CFDL_v0_1_Grammar.ebnf`.
- **Expression reference**: `docs/03` §4, the function catalog.
- **IR**: a new node beside `Curve` in `docs/schemas/ir.schema.json`, `docs/05`,
  `crates/cfdl-compile/src/lib.rs` and `crates/cfdl-engine/src/ir.rs`.
- **Engine**: `CurveDef`'s sibling in `crates/cfdl-expr/src/lib.rs`, the
  functions in `crates/cfdl-calc/src/funcs.rs`, and the `Env` hook in
  `crates/cfdl-calc/src/eval.rs`.
- **Lexer and parser**: one keyword, one statement production.
- **Diagnostics**: `docs/08` — non-monotone points, x outside [0, 1], a duplicate
  x, a curve passed to a quantile function or the reverse, and an unknown `ref`.
- **Results**: `docs/06` and `docs/schemas/results.schema.json` for
  `InputsSection.quantiles`.
- **Terminology**: `docs/terminology.toml`, and `docs/glossary.md` regenerates
  from it.
- **Tooling**: `crates/cfdl-lsp` completion and hover.
- **Site and training**: the curves chapter gains a sibling; `docs/19` §7.

Every one of those is generated from or checked against the repository, so the
existing gates catch a miss: `sync:check`, `check-doc-examples`, the snippet
parser, `glossary-check`, `ir-schema`, `results-schema`, and the golden and
benchmark suites.
