# CREST cost-based solar — a three-way check, and three pack gaps it found

Every other benchmark in this repo agrees with **one** reference. This one
agrees with a model that exists **twice**, so CFDL is a third implementation.

That distinction is the entire reason the case exists alongside
`utility_pv_singleowner` rather than replacing it. Two implementations agreeing
is consistent with both being wrong the same way — they can share a
misreading of a convention, and a spreadsheet and its own port share an author's
intent by construction. Three implementations, in three languages, by unrelated
authors, agreeing to the engine's published precision is a much harder thing to
arrange by accident.

## The source

A US national laboratory publishes a **cost-based renewable energy tariff
model** as an Excel workbook, in five technology variants. Its purpose is
regulatory: it computes the tariff a project must be paid for its equity to
clear a target return, which is what a feed-in tariff or a cost-based incentive
has to be set to.

An independent contributor has **ported the solar variant to Python**. The port
is a separate work by a separate author, and it materialises things the
spreadsheet buries in formulas — most visibly, it carries the MACRS half-year
convention as its own data file rather than as a branch inside a cell.

**Neither is vendored and neither is in CI.** The workbook carries no stated
licence, and the port declares none at all, which means default copyright. The
port was cloned outside this repo, run once, and only its **output numbers**
were carried across. That is the same handling `utility_pv_singleowner` gives
the laboratory's own tool, and the reason there is no `reference_gen.py` here:
writing our own second implementation is exactly what an external reference
replaces.

### Running it

The port targets **numpy < 1.20**, where `ppmt`, `ipmt`, `irr` and `npv` still
lived in numpy itself. They were moved to `numpy-financial` and the port fails
on import-era numpy with `AttributeError: module 'numpy' has no attribute
'ppmt'`.

The fix is the smallest one available: install `numpy-financial` and rebind the
four names onto the numpy module before importing the port. `numpy-financial`
**is** the same implementation, relocated — so this restores the port's original
behaviour rather than substituting for it. The port's own files stay
byte-identical to upstream.

- port commit `9c915ed57bea7cbedd70fa15a044d467f0042ddb` (2 August 2019)
- run under numpy 2.2.4 / pandas 2.3.2 / numpy-financial 1.0.0, Python 3.13
- inputs are the port's own defaults: 2,000 kW-dc, California capacity factor,
  25-year life, 0.5% degradation, $7.0m hard cost, 45% debt at 7% over 18
  years, 30% ITC, 35% federal / 8.5% state tax

## The deal

2 MW-dc distributed solar. 3,161,597.15 kWh in year one — 2,000 kW at an
18.0456% net capacity factor over 8,760 hours — degrading 0.5% a year over a
25-year life, paid a flat cost-based tariff. Five operating expense lines, four
of which escalate and one of which abates. $3.15m of level-pay debt, 18 years
at 7%, maturing seven years before the asset does.

## The result

**Exact.** All seven individual stream columns agree with the reference across
all 25 periods with **zero** disagreement — not "within tolerance", identical
at the engine's published precision.

The single non-zero figure in the case is **5.0e-7** on `domain.energy.opex` at
period 19, and it is not arithmetic. Results JSON publishes money to six
decimal places. The engine rounds a subtotal it computed from *unrounded*
components; summing five *already-rounded* components is a different operation,
and the two differ by up to half of the last published place. 5e-7 is exactly
that half. It is the floor any case in this repo can assert to, which is why
`period_tolerance` is 1e-6 rather than something chosen for this deal.

### What the expense lines actually prove

The reference publishes operating expenses as **one total**, so the four
decomposed lines here are restated from its stated inputs. `domain.energy.opex`
is what makes that restatement evidence rather than assertion: it is the
reference's own published total, and the four lines plus the royalty have to
sum to it in every period.

They escalate at three different rates, which is the point:

| line | year 1 | convention |
|---|---:|---|
| fixed O&M | 13,000 | +1.6%/yr |
| insurance | 28,000 | +1.6%/yr |
| land lease | 5,000 | +1.6%/yr |
| payment in lieu of tax | 50,000 | **−10%/yr** |
| royalty | 21,957 | 3% of tariff revenue |

A single blended escalator reproduces the year-one total and drifts from every
year after it. The **negative** escalator is worth pinning on its own: the pack
carries it through `pow(1 + escalation, t)` with no special case, so an
escalator implemented as a growth-only ratchet would silently hold this line
flat and be caught here rather than in production.

## What this case does NOT validate

**The solve.** The reference model's actual purpose is to find the tariff that
clears a 12% after-tax equity return — it sweeps the rate until net present
value crosses zero, and reports the crossing. Reproducing that sweep here gave
**23.15 c/kWh**, and that rate is carried into the model as a constant.

So the tariff is an *input* to this case and an *output* of the reference.
CFDL has no solve-to-target construct, and this is the cleanest available
statement of what that costs: everything downstream of the tariff is validated
period by period, and the one step that makes the reference a policy tool
rather than a cash flow model cannot be expressed at all. That gap is already
on the roadmap; this case is a concrete measurement of it rather than another
argument for it.

**The tax layer.** Not asserted, for two reasons that are themselves findings —
see gaps 2 and 3 below.

**EBITDA.** The reference's EBITDA includes interest earned on funded reserve
accounts (~$4,606 in year one), which CFDL does not model. Revenue and opex are
asserted separately instead; asserting an EBITDA that differs by a line neither
model disputes would weaken the case rather than strengthen it.

## Three pack gaps this found

**1. No revenue-linked expense.** The royalty is 3% of tariff revenue. Every
energy pack expense rule takes a *fixed annual amount* and escalates it; none
takes a percentage of another stream. So the royalty is hand-written, and it has
to restate the production and price the PPA contract already carries — the
duplication in `model.cfdl` is the evidence. Revenue-linked charges are not
exotic: royalties, revenue-share ground leases and percentage-based management
fees are all this shape, and the CRE pack's percentage rent is the same
mechanic solved once already in a different pack.

**2. One operating expense category.** The pack defines exactly one:
`operating.expense.om`. Insurance, a land lease, a property tax payment and a
royalty are none of them operations and maintenance, but there is nowhere else
to put them. It changes no number here — `domain.energy.opex` globs
`operating.expense.*` — but it means the pack cannot tell these lines apart in
a statement, and a lender's operating statement distinguishes them. The
compiler diagnostic that surfaced this (`E5022_UNKNOWN_STREAM_CATEGORY`) named
every known category, which is why it took one attempt to find rather than a
search.

**3. No bonus depreciation, and no multi-class allocation.** The reference
allocates installed cost across 5-year, 15-year, 20-year, straight-line and
non-depreciable classes via a published table, then applies 50% bonus
depreciation in year one on top of the half-year convention. The pack's
`energy.macrs_shield` takes a single `life` and applies `macrs_rate()`, so the
allocation could be approximated with several contracts — but bonus
depreciation cannot be expressed at all, and it moves roughly half the
depreciable basis into the first year. This is the largest of the three gaps
and the reason the tax layer is out of scope rather than partially asserted.
