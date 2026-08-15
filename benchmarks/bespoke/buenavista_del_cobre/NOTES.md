# Maintainer's notes

Not published. Working notes on how this case was built and what it cost to
establish.

## Reading the table out of the filing

Table 19.1 sits on report page 19-3, **PDF page 296**, a rotated landscape
page. `pdftotext -layout` returns that page as an empty string and
`pdfimages -list` reports only a 37x98 logo, so every text-based check says the
page is blank. It is not — the table is vector text. Render it:

```
pdftoppm -f 296 -l 296 -r 300 -png scco-20241231xex96d6.pdf out
```

then rotate -90 and slice into bands with the label column pasted alongside
each, so no cell is read by row position alone. The document's producer is
"Microsoft: Print To PDF", which is where the broken text layer comes from.
La Caridad (Exhibit 96.7) has the identical defect on its PDF page 336.

**This cost a wrong conclusion.** The case was first declared unusable on the
strength of the empty extraction. Never conclude a page is blank from
`pdftotext` alone.

## Transcription checks

`published_grid.csv` was validated two ways before anything was built on
it: every row summed against its own printed Total/Avg column (worst deviation
7, on a whole-Mt tonnage row; exact on Total Material, EBITDA, Total Taxes,
Total Capex and Pre Tax Cash Flow), and the four internal identities — revenue,
operating cost, EBITDA and pre-tax cash flow — confirmed in all 21 annual
columns.

## How the fiscal structure was found, and three wrong turns

The bridge between EBITDA and net income took four attempts.

1. **Fitted the tax rule to the printed line.** 30% x (gross income − the whole
   duty) reproduced the tax 11 times out of 11, and was adopted on that basis
   while the prose seemed to say 30% of the duty. That was curve-fitting, and
   it happened to be right for the wrong reason — see 4.
2. **Missed the accretion add-back.** The bridge went negative in 2036-2043,
   which made no sense as a charge. ARO accretion sits in operating cost and is
   non-cash; restoring it made all six loss years fit to within 0.6.
3. **Chased an 11.1% residual.** With accretion restored, a residual of a
   constant 11.1% of (duty + PTU) remained. Numerology on it — grossing up by
   1/0.9, a second 10% layer — fit but named nothing, so it was not
   implemented.
4. **Read a different mine.** La Caridad/Pilares prints every intermediate
   line. Its rows give the structure directly, and it resolves all three
   earlier problems at once: the residual was PTU (gross income is 0.9 x
   (ebitda − dep − royalty), so PTU is gross income over 9), and the filing's
   "Minimum tax" row is the 30%-of-royalty credit, which makes the prose in
   1 literally correct as written.

The lesson worth keeping: when a filing omits an intermediate, look for a
sibling filing by the same author under the same regime before inferring it.
The whole of Section 19 is boilerplate shared across Southern Copper's mines.

## The 10-K

The parent FY2024 Form 10-K (same accession) confirms the duty base in the
issuer's own words — "a mining royalty of 7.5% on taxable earnings before
taxes, depreciation, and interest" — and discloses two things the technical
report does not: a 0.5% additional royalty on gold, silver and platinum
receipts, and that the Ley Federal de Derechos was amended on 19 December 2024,
effective 1 January 2025, raising the duty to 8.5% and the additional royalty
to 1%. The report applies 7.5% across a forecast starting on that date.

Neither is modeled, deliberately: the case reproduces what the filing
computed. Empirically the printed tax line wants 7.5% (max error 0.31) and the
cash bridge wants something nearer 8.5%, which is consistent with the report
deducting the old rate while the cash charges carry more — but no single rate
closes both lines, so nothing was fitted.

## The architecture revision, and the three shapes tested

The case first shipped with the fiscal charges recomputing EBITDA inline from
the curves, held in a pseudo-entity of computed fields. Review found that
wrong on the language's own terms, and three architectures were then run
against the same expectations:

1. **Fields-as-computation** (as shipped): works, but misuses fields — only
   the carryforward is memory — and repeats the EBITDA expression eight
   times.
2. **A fiscal waterfall** (`from series_sum(...)`): reproduces every charge
   exactly, and changes what net cash flow means — waterfall steps allocate
   cash inside the model rather than paying it out, so `model.net_cash_flow`
   became pre-tax and the after-tax NPV and all 72 scenario assertions would
   be lost. Taxes are expenses leaving the model, not an allocation among
   claimants, so the construct is wrong here even though the numbers land.
3. **Second-tier streams reading the period's series** (adopted): each charge
   is a stream whose amount reads EBITDA as the realized result of the base
   streams. Cross-stream reads are one hop deep (docs/10: phase-2 streams
   cannot reference each other), so each charge derives from EBITDA in
   closed form rather than chaining.

Two constraints shaped the remainder. A field rule has no series access
(docs/03 section 3.1), so the shelter restates gross income from the curves —
the one duplication no architecture removes. And `phase_enter` evaluates in
schedule position but not in an event condition, so the lifecycle events
state their boundaries by period index; `docs/13` section 7.36 records both,
with the state-machine enhancement they point at.

## Tolerance

`period_tolerance` is 1e-5, not a rounding allowance. `expected.csv` holds the
reference's output rather than the filing's printed cells, so CFDL and the
reference compute the same rules from the same drivers and any daylight is a
defect. 1e-5 is the float noise of carrying payable metal as revenue over
price and multiplying it back. The filing's whole-million rounding is absorbed
one level up, inside `reference_gen.py`, which fails outside 2.5.

## Not done

- Mutation testing. The case was checked by hand for discrimination while it
  was built — perturbing each fiscal rate, the schedule placement, the PTU
  floor and one scenario expectation, each of which the suite caught. None of
  that is recorded here, because `docs/20` §3.3 is a backlog item rather than
  an adopted practice and no other case records it. Adopting it belongs in its
  own change, across the suite.
- The 78 sensitivity points are asserted against the reference, not against the
  filing's published grid. `reference_gen.py` diffs the two and prints the
  result (worst 53.6, 1.57% of base), but nothing fails if that drifts. Making
  the published grid a first-class assertion needs a per-scenario tolerance
  wide enough for the bucket effect, which would be a different kind of claim.
- The intra-bucket profile for 2046-2065 is unrecoverable from this document.
  If Southern Copper files an updated TRS with those years annualized, both NPV
  residuals and most of the sensitivity error should close.
