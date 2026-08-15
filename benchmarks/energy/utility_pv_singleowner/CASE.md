## The case

A 100 MW-AC utility-scale photovoltaic project in a single-owner structure,
generating 250 GWh in its first year. It sells under a 25-year power purchase
agreement at $45/MWh escalating 2% a year, against 0.5% annual module
degradation. $60m of debt amortizes over 18 years at 6%. A 30% investment tax
credit lands in the first operating year, and the project depreciates on the
five-year MACRS schedule, on a basis reduced by half the credit.

Single owner means the project carries its own tax position rather than
allocating it to an investor.

## The reference

A national laboratory's open-source project-finance model, the standard tool
for this structure. Being open source, a disagreement can be traced to a
specific formula.

**Not vendored.** The tool was run once outside the repository and only its
output numbers were carried across, so nothing about it is a build dependency.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.ppa`, `energy.om`, `energy.debt_service`, `energy.itc`, `energy.macrs_shield`, `energy.capex` |
| Language features | pack contracts across a full capital structure; term units |
| Conventions | production degradation, price escalation, level-pay debt, an investment tax credit, MACRS with a basis reduction |

More of the energy pack's contract surface than any other case.

## The result

Every asserted line agrees, worst **9.1e-7 dollars** across all 26 periods and
all four escalating streams.

Asserted: six stream columns at anchor periods — the MACRS table through its
final year and the zero after it, the debt tenor and its cliff at periods 18 and
19, and the compounding at the end of the hold.

## The delta

The residual is float noise, not convention. Anchors rather than every period
because escalation and degradation compound: a convention error shows up in
every period after the first and grows, so the anchors bracket where it would
appear.

The reference states its operations and maintenance escalation as a *real* rate
carried on top of an inflation assumption, while the pack's escalation term is
nominal. The case runs at zero inflation, where the two coincide exactly.
