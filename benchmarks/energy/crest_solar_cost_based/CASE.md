## The case

A 2 MW-dc distributed solar project paid a cost-based feed-in tariff. It
generates 3,161,597 kWh in its first year — 2,000 kW at an 18.0456% net capacity
factor over 8,760 hours — degrading 0.5% a year across a 25-year life, and is
paid a flat 23.15 c/kWh.

Five operating expense lines run against it, and they do not share an escalator:
fixed operations and maintenance, insurance and a land lease each inflate at
1.6%; a payment in lieu of property tax **abates 10% a year** on a stated
schedule; and a royalty takes 3% of tariff revenue. $3.15mm of level-pay debt
runs 18 years at 7%, maturing seven years before the asset does.

## The reference

A cost-based renewable energy tariff model published by a national laboratory as
a spreadsheet, and independently ported to Python by a third party. Both were
run; the comparison is three-way.

It publishes a complete annual cash flow, so every line is checkable period by
period rather than at an endpoint.

**Not redistributable.** The spreadsheet states no licence and the port declares
none, which means default copyright. Neither is vendored or wired into the test
suite: the port was cloned outside the repository, run once, and only its output
numbers carried across.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Contract types | `energy.ppa`, `energy.om` (four instances), `energy.debt_service` |
| Language features | contracts with per-instance suffixes, one native stream, term units |
| Conventions | production degradation, three escalation rates including a **negative** one, level-pay amortisation |

The four operating expense contracts are the same type at different escalators,
which is why they are asserted as separate lines rather than as one total.

## The result

**Exact on every individual line.** All seven stream columns agree with the
reference across all 25 periods with zero disagreement — not "within tolerance",
identical at the engine's published precision.

Asserted: seven stream columns across 25 periods, plus `domain.energy.opex` — the
reference's own published expense total — which is what makes the four
decomposed lines evidence rather than assertion. The reference publishes
operating expenses as a single figure, so the decomposition has to sum back to it
in every period.

## The delta

One non-zero figure: **5.0e-7**, on the summed expense column at period 19.

It is not arithmetic. Results carry money to six decimal places, and the engine
rounds a subtotal it computed from *unrounded* components — which is a different
operation from summing five *already-rounded* components, and the two differ by
up to half of the last published place. 5e-7 is exactly that half. It is the
floor any case here can assert to, which is why the tolerance is
set to the engine's precision rather than to anything about this deal.

One thing the case does **not** validate: the reference's actual purpose is to
solve the tariff that clears a target equity return, sweeping the rate until net
present value crosses zero. CFDL has no solve-to-target construct, so the solved
rate — 23.15 c/kWh — is carried across as a constant. Everything downstream of
the tariff is checked period by period; the solve itself is not.
