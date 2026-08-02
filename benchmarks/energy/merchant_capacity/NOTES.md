# Merchant + capacity + production credit — wiring, and one real staircase

## Be clear about what this case does and does not claim

`utility_pv_singleowner` validated the energy pack's arithmetic against the
national laboratory's model. This case is its companion, and its claim is
narrower. Compare the rules:

```
ppa_revenue      (mwh_year/ppy) * availability * pow(1-degradation, y) * ppa_price      * pow(1+escalation,       y)
merchant_revenue (mwh_year/ppy) * availability * pow(1-degradation, y) * price          * pow(1+price_escalation, y)
ptc_credit       (mwh_year/ppy) * availability * pow(1-degradation, y) * credit_per_mwh * pow(1+escalation,       y)
```

Identical but for term names, and `capacity_payment` is `payment_year / ppy`.
So reconciling merchant revenue and capacity payments confirms the terms reach
the right places and the contracts compose. It does **not** re-prove arithmetic,
and reporting it as "the energy pack is now fully validated" would be
overstating it. This was worth doing; it was not worth overselling, and an
earlier draft of the plan did exactly that.

## What was genuinely new: the production credit is a staircase

The credit rate is inflation-adjusted and then **published rounded to the
nearest 0.1 cent per kWh**, so it steps once a year and holds. The pack carried
it as a continuous ramp.

`utility_pv_singleowner` found this and could not fix it — the rounding needed a
builtin the language did not have. That builtin (`round_to`) now exists, so the
credit is asserted here at a **non-zero escalation**, which is precisely the
configuration that used to be wrong:

| year | CFDL | reference | difference |
|---|---|---|---|
| 1 | 7,000,000.00 | 7,000,000.00 | +8e-7 |
| 2 | 6,965,000.00 | 6,965,000.00 | +2e-7 |
| 3 | 7,177,681.25 | 7,177,681.25 | +2e-7 |
| … | | | |
| 10 | 8,125,061.42 | 8,125,061.42 | 0 |

All ten years of the credit window, worst 8e-7. Previously up to **1.8% in a
single year**, alternating sign — which is why it read as noise rather than as a
defect.

Choosing a grid-aligned rate and zero escalation would also have made this case
pass, while testing nothing about the convention that was actually wrong. That
would have been the easy version and a worthless one.

### The unit slip, and what caught it

First attempt set the tick to `$0.10/MWh`. Wrong by a factor of ten:

```
0.1 cent = $0.001 per kWh,  x 1000 kWh = $1.00 per MWh
```

The published credit moves in **whole dollars per MWh** — 28, 29, 30 — not
tenths. Rounding to $0.10 is rounding to a hundredth of a cent, which is
indistinguishable from not rounding at all, so every unit test still passed and
the builtin looked correct.

The external reference is what caught it: CFDL produced rates of 27.5, 28.2,
28.9 against the reference's 28, 28, 29. No in-house check would have found
this, because an in-house reference would have carried the same conversion.
That is the third time this programme has caught an error of exactly that shape.

## The rest

Everything else agrees to floating-point noise, as expected — the same rules the
PV case already established:

| stream | worst over 26 periods |
|---|---|
| merchant revenue | 9e-7 |
| capacity payment | **0** |
| O&M expense | 5e-7 |
| debt service | 3e-7 |
| MACRS shield | **0** |

Note MACRS runs on the **full $100m basis** here. Investment and production
credits are mutually exclusive, so unlike the PV case nothing reduces the
depreciable basis — which is itself a small confirmation that the basis rule in
that case was applied for the right reason rather than by coincidence.

## Storage is deliberately absent, and stays unvalidated

`energy.storage_arbitrage` is `mwh_cycled_year * spread * (1-degradation)^y` — a
reduced form. The reference models a battery with a **dispatch optimiser** over
an hourly price series, so its annual revenue emerges from thousands of hourly
decisions. The two do not reduce to one another, and no choice of inputs makes
them agree; fitting `spread` until they matched would be calibration, not
validation.

So storage is not in this model and is not asserted. The pack's storage revenue
has **no external validation** and that is recorded in
`docs/13_feature_backlog.md` rather than papered over. Energy is at **9 of 10
rules**, not 10.

The reduced form is not wrong — practitioners use exactly this shape at the
financing stage. It is coarse, and the useful work is quantifying its error
against a dispatch model, which needs either a price-curve input (expressible
today via `curve` declarations) or an explicit statement of what the reduced
form assumes.

## Reproducing the reference

Same posture as `utility_pv_singleowner`: a throwaway virtualenv outside the
tree, run once, nothing vendored.

    python -m venv /tmp/sam-venv
    /tmp/sam-venv/bin/pip install nrel-pysam        # 7.1.1.post1
    /tmp/sam-venv/bin/python sam_merchant.py 0.0    # inflation_rate = 0

Inputs as `utility_pv_singleowner`, with the investment credit off, the
production credit on (`ptc_fed_amount = 0.0275`, `ptc_fed_escal = 2.5`,
`ptc_fed_term = 10`), and capacity payments on
(`cp_capacity_payment_type = 1`, `cp_capacity_payment_amount = 4,000,000`,
no escalation). Outputs read: `cf_total_revenue`, `cf_capacity_payment`,
`cf_ptc_fed`, `cf_om_capacity_expense`, `cf_debt_payment_total`,
`cf_feddepr_macrs_5`.

Merchant energy revenue is `cf_total_revenue − cf_capacity_payment`; the
reference reports them together.
