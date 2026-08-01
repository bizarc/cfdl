# Utility PV single-owner — what checking against an external model found

The energy pack had **no external validation at all** before this. Ten rules,
two benchmarks, both checked against reference generators written alongside the
engine — the failure mode `tools/analytic-checks.py` exists to catch. This is
the first energy figure in the repo that came from somewhere else.

## The reference, and what may be committed

The US national laboratory's open-source system-advisor model, single-owner
project-finance configuration, driven through its Python bindings. BSD-3
licensed, so unlike the credit reference there is no reproduction constraint —
the reason it is still not vendored is the standing rule that the validation
*mechanism* does not persist in the repo. It was installed in a throwaway
virtualenv outside the working tree, run once, and discarded.

    python -m venv /tmp/sam-venv
    /tmp/sam-venv/bin/pip install nrel-pysam        # 7.1.1.post1
    /tmp/sam-venv/bin/python sam_run.py 0.0         # inflation_rate = 0

`sam_run.py` builds `PySAM.Singleowner.new()` and sets every input explicitly
rather than loading a bundled default, so there is no hidden assumption behind
the numbers. The inputs are exactly the terms in `model.cfdl` plus the
financial parameters below; the outputs read are `cf_total_revenue`,
`cf_om_capacity_expense`, `cf_debt_payment_total`, `cf_feddepr_macrs_5` and
`itc_total`.

    analysis_period      25            debt_option          0 (percent of cost)
    system_capacity      100,000 kW    debt_percent         60
    annual energy        250 GWh yr 1  payment_option       0 (level payment)
    degradation          0.5 %/yr      term_int_rate        6 %
    ppa_price_input      4.5 c/kWh     term_tenor           18 yr
    ppa_escalation       2 %/yr        itc_fed_percent      30
    om_capacity          $15/kW-yr     depr_alloc_macrs_5   100 %
    om_capacity_escal    2 %/yr        federal_tax_rate     21 %
    inflation_rate       0             insurance/prop tax   0

Everything the case does not use — reserves, fees, construction financing,
salvage, state tax, capacity payments, dispatch factors — is set to zero
explicitly.

## The result

Five of the pack's ten rules reproduce the external model **to within 9.1e-7
dollars over 26 periods**: PPA revenue with escalation and degradation, O&M with
escalation, level-pay project debt, the 5-year MACRS shield, and the ITC. The
worst disagreement on any period of any stream:

    energy.ppa.revenue      9.1e-7   (period 23)
    energy.om.expense       4.7e-7   (period 21)
    energy.debt.service     2.5e-7   (period 1)
    energy.macrs.shield     0        (exact, every period)

That is decimal arithmetic against IEEE-754 float64 over 25 years of
compounding, so the residual is the reference's representation error, not ours.

It matched on the first attempt, which is worth stating plainly — nothing was
adjusted to make it agree. The findings below are all *convention* differences
that a modeller has to know about, plus one real gap.

## Why the model is annual, and starts a year early

The reference produces annual arrays indexed from 1, with index 0 as the
construction year. The model uses a 26-period annual calendar so periods 1..25
line up index-for-index with operating years 1..25 and period 0 carries only
capex. No cadence translation sits between the two models, so any divergence is
a convention difference and nothing else. A monthly rebuild would be a
legitimate model and a different number.

## Finding 1 — escalation is nominal here and real there

The pack's `escalation` is a single **nominal** rate: `pow(1 + escalation,
elapsed_years)`, full stop. The reference states O&M escalation as a **real**
rate carried on top of a separate inflation assumption, and — this is the part
worth writing down — it combines them **additively**, not multiplicatively.

At `inflation_rate = 2.5` and `om_capacity_escal = 2.0`, year 2 O&M is
1,500,000 x 1.045, not 1,500,000 x 1.025 x 1.02. The two differ by 750 in year
2 and compound from there.

So a modeller moving a deal across needs `escalation = inflation + real`, and
this case is run at zero inflation where the two coincide exactly. Not a defect
in either model — but a silent 0.05%/yr drift for anyone who assumes the rates
compose the way rates usually do.

Note the asymmetry *inside* the reference: its PPA escalation is nominal —
`cf_ppa_price` is byte-identical at 0% and 2.5% inflation — while its O&M
escalation is real. Two escalation terms, two conventions, in one model. The
pack has one convention for both, which is the simpler thing to reason about.

## Finding 2 — the ITC reduces the depreciable basis, and the pack will not do it for you

Taking a 30% ITC on a $100m project gives an $85m depreciable basis, not $100m:
half the credit comes off the basis. The reference derives this
(`depr_fedbas_after_itc_macrs_5 = 85,000,000`, a reduction of 15,000,000).

`energy.macrs_shield` takes `basis` as an input and does not derive it, so the
reduction is stated in `model.cfdl` with a comment. That is a defensible design
— basis adjustments are jurisdictional and there are several — but it is a
sharp edge: entering the installed cost as the basis overstates the shield by
17.6% for the life of the schedule, and nothing in the pack would object.

Documented in `packs/energy/README.md` rather than fixed. A derived-basis term
is a backlog candidate, not a defect.

## Finding 3 — the production credit is statutorily rounded, and the pack does not round

This one is a real gap. `energy_ptc_credit` computes
`credit_per_mwh * pow(1 + escalation, elapsed_years)` as a continuous quantity.
The published inflation-adjusted credit is **rounded to the nearest 0.1 cent per
kWh** each year, and the reference does exactly that.

Reconciled on a separate run (ITC off, `ptc_fed_amount = 0.0275`,
`ptc_fed_escal = 2.5`, `ptc_fed_term = 10`), the implied rate is 0.028, 0.028,
0.029, 0.030, 0.030, 0.031, 0.032, 0.033, 0.034, 0.034 — every one of the ten
reproduced by rounding `0.0275 x 1.025^k` half-up to three decimals.

The effect is not a drift but a sawtooth, because rounding alternates direction:

    year  1   -1.79%      year  6   +0.37%
    year  2   +0.67%      year  7   -0.34%
    year  3   -0.37%      year  8   -0.94%
    year  4   -1.29%      year  9   -1.45%
    year  5   +1.18%      year 10   +1.01%

Ten-year total 75,236,326 against 75,459,520, **-0.30%**. Small in aggregate and
up to 1.8% in any single year, which matters for a debt sizing struck off a
particular year's coverage.

`benchmarks/energy/wind_ptc_macrs` asserts the unrounded figure against an
in-house generator, so both sides carry the same omission and have always
agreed. Exactly the pattern the credit pack's prepayment base turned out to
have. Tracked in `docs/13_feature_backlog.md`; not fixed here, because the
rounding step needs a `round_to` builtin the expression language does not have.

## What this case does not cover

Five of the pack's ten rules are exercised. `energy.merchant`,
`energy.storage_arbitrage` and `energy.capacity` have no counterpart in a
single-owner PV configuration; `energy.ptc` is reconciled above but not asserted
here, because a model claiming both the investment and the production credit
would be a bad example to ship. A merchant-plus-storage configuration of the
same reference would cover the first two and is the obvious next pass.
