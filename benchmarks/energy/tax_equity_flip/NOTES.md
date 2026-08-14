# Tax-equity partnership flip — what an external model settled

The first case in the repo whose asserted output includes **a date the model
derived**. Every other case checks amounts; this one also checks *when* a
lifecycle transition fired.

## The reference, and what may be committed

The US national laboratory's open-source system-advisor model, leveraged
partnership-flip configuration, driven through its Python bindings. BSD-3
licensed, so the constraint is not the license but the standing rule that the
validation *mechanism* does not persist in the repo. It was installed in a
throwaway virtualenv outside the working tree, run once, and discarded.

    python -m venv /tmp/sam-venv
    /tmp/sam-venv/bin/pip install nrel-pysam        # 7.1.1.post1
    /tmp/sam-venv/bin/python sam_run.py

The run builds `PySAM.Levpartflip.new()` and sets every input explicitly. The
inputs are the terms in `model.cfdl` plus the financial parameters below; the
outputs read are `flip_actual_year`, `cf_tax_investor_aftertax_cash` and
`cf_sponsor_aftertax_cash`.

    analysis_period       25            debt_option           0 (percent of cost)
    system_capacity       100,000 kW    debt_percent          60
    annual energy         250 GWh yr 1  payment_option        0 (level payment)
    degradation           0.5 %/yr      term_int_rate         6 %
    ppa_price_input       4.5 c/kWh     term_tenor            18 yr
    ppa_escalation        2 %/yr        itc_fed_percent       30
    om_capacity           $15/kW-yr     depr_alloc_macrs_5    100 %
    om_capacity_escal     2 %/yr        federal_tax_rate      21 %
    inflation_rate        0             flip_target_percent   8 %
    insurance/prop tax    0             tax_investor_equity   98 %
    preflip cash / tax    98 %          postflip cash / tax   5 %

## The installed cost is larger than the equipment

The reference reports `cost_prefinancing` of $100mm and `cost_installed` of
$103.1mm: it capitalizes $3.1mm of financing into the basis, and both the
credit and depreciation are taken on the larger figure. The model states the
installed cost rather than deriving the financing component, which is the one
input carried across rather than computed. Everything downstream follows:

    itc            30% x 103.1mm      = 30,930,000
    depr basis     103.1mm - 50% itc  = 87,635,000

## What the reconciliation established

The reference solves the flip by testing an internal rate of return each year.
This language has no mid-model IRR, and the case did not need one: at a fixed
hurdle, `IRR through n >= 8%` and `NPV at 8% through n >= 0` are the same
statement, and the second is a discounted running sum — a recurrence.

That the derived transition lands where the reference's flip year says it does
is the check. Both partners' cash then agrees to 1.0e-6 dollars across 25
periods, which it could not do if the date were wrong: the split either side
differs by a factor of nearly twenty.

## The grid finding

Running the identical deal monthly moves the flip ten months earlier, because
the annual grid has no period boundary between month 24 and month 36 at which
the test can pass. Roughly $3.5mm of cash changes hands on the strength of a
calendar line. The monthly model is `fixtures/valid/flip_monthly_grain`; no
external source publishes a monthly answer for it, so it is a fixture with a
golden rather than a benchmark.
