#!/usr/bin/env python3
"""The REFERENCE for a merchant storage benchmark. Establishes anchors only.

Nothing here models the deal. It does three things:
  1. generates the stated price series (seeded, reproducible);
  2. runs NREL SAM's dispatch optimiser over it, with EVERY assumption set
     explicitly rather than inherited from a config default;
  3. summarises the price series as a duration curve — sorted prices at
     exceedance levels, which is a descriptive statistic of the data and
     carries no dispatch decision.

The CFDL model consumes (1) as physics terms and (3) as a `quantile`, and
computes the dispatch itself. (2) is the anchor it is measured against.
"""
import json, numpy as np
import PySAM.Battery as B, PySAM.BatteryTools as BT

SEED = 20260902

# ── the matched assumption set. Both sides use exactly these. ───────────────
POWER_MW      = 20.0
NOMINAL_MWH   = 80.0
SOC_MIN_PCT   = 15.0
SOC_MAX_PCT   = 95.0
SOC_INIT_PCT  = 50.0
CYCLE_COST    = 0.0       # ZERO on both sides for the base case. SAM's
                          # "$/cycle-kWh" accounting could not be matched to a
                          # $/MWh-throughput hurdle (at 0.02 it dispatched into
                          # a loss), so the objective is pure arbitrage margin
                          # and degradation is modelled separately.
                          # cost model, which neither side could see)
AC_DC_EFF     = 96.0      # %
DC_AC_EFF     = 96.0      # %
DC_DC_EFF     = 99.0      # %
LOOK_AHEAD_HOURS = 24.0   # the optimiser's foresight, STATED: net value rises
                          # monotonically with it, so a reference without a
                          # declared horizon is not a fixed target.


def hourly_prices():
    rng = np.random.default_rng(SEED)
    h = np.arange(8760); doy, hod = h // 24, h % 24
    seasonal = 8.0*np.sin(2*np.pi*(doy-200)/365.0)
    diurnal  = 18.0*np.exp(-(((hod-19)%24))**2/5.0) - 9.0*np.exp(-(((hod-4)%24))**2/9.0)
    price = 32.0 + seasonal + diurnal + rng.normal(0,7,365).repeat(24) + rng.normal(0,3,8760)
    for d in rng.choice(365, 12, replace=False):
        price[d*24 + np.array([17,18,19,20])] += rng.uniform(150, 600)
    return np.maximum(price, 3.0)


def size_to(m, target_mwh):
    """SAM computes bank capacity from cell geometry, so a requested 80 MWh
    lands at 83.33. Solve the request so the COMPUTED bank is the stated
    nameplate — then both sides state one number and neither imports the
    other's. Bank capacity is an assumption a modeller must make; this makes
    the two models make the SAME one."""
    req = target_mwh
    for _ in range(40):
        BT.battery_model_sizing(m, POWER_MW*1000, req*1000, 500.0)
        got = float(m.BatterySystem.batt_computed_bank_capacity)/1000
        if abs(got - target_mwh) < 1e-4:
            return got
        req *= target_mwh/got
    return got


def sam_reference(price):
    m = B.default("StandaloneBatteryMerchantPlant")
    size_to(m, NOMINAL_MWH)
    c, s, d = m.BatteryCell, m.BatterySystem, m.BatteryDispatch
    c.batt_minimum_SOC, c.batt_maximum_SOC, c.batt_initial_SOC = SOC_MIN_PCT, SOC_MAX_PCT, SOC_INIT_PCT
    c.batt_calendar_choice   = 0        # no calendar degradation
    s.batt_replacement_option = 0       # no replacement
    s.batt_ac_dc_efficiency, s.batt_dc_ac_efficiency = AC_DC_EFF, DC_AC_EFF
    s.batt_dc_dc_efficiency = DC_DC_EFF
    ps = m.PriceSignal
    ps.forecast_price_signal_model = 1
    ps.mp_enable_energy_market_revenue = 1
    ps.mp_energy_market_revenue = tuple((1000.0, float(p)) for p in price)
    d.batt_dispatch_choice = 0                  # front-of-meter AutomatedEconomic
    d.batt_dispatch_auto_can_gridcharge = 1
    d.batt_dispatch_auto_can_charge = 1
    d.batt_cycle_cost_choice = 1                # 1 = use the STATED cost below
    d.batt_cycle_cost = (CYCLE_COST,)
    d.batt_look_ahead_hours = LOOK_AHEAD_HOURS
    m.execute(0)
    p = np.array(m.Outputs.batt_power)[:8760]/1000.0
    dis, chg = np.maximum(p,0), np.maximum(-p,0)
    return {
        "bank_mwh": float(m.BatterySystem.batt_computed_bank_capacity)/1000,
        "mwh_out": float(dis.sum()), "mwh_in": float(chg.sum()),
        "revenue": float((dis*price).sum()), "cost": float((chg*price).sum()),
        "margin": float((dis*price).sum() - (chg*price).sum()),
        "realised_rte": float(dis.sum()/chg.sum()),
        "capture_discharge": float((dis*price).sum()/dis.sum()),
        "capture_charge": float((chg*price).sum()/chg.sum()),
        "cycle_cost": float(dis.sum()*CYCLE_COST*1000),
        "net_margin": float((dis*price).sum()-(chg*price).sum()-dis.sum()*CYCLE_COST*1000),
        "active_days": int((dis.reshape(365,24).sum(axis=1) > 0.1).sum()),
    }


def daily_blocks(price, dis_hours, chg_hours):
    """Daily TBx block prices: the volume-weighted mean of the x dearest hours
    and the x cheapest hours of each day.

    This is the market's own battery product — TB2, TB4, "top-bottom spread" —
    and x is the ASSET'S DURATION, a physical spec. An on-peak/off-peak block
    is the wrong product here: HE 08-23 averages sixteen hours including
    mid-day, while a four-hour battery reaches only the top three, so the block
    understates the spread it lives on by a factor of several.

    Sorting a day's hours by price is the definition of the product, not a
    dispatch decision: it says which hours the product references, not when the
    battery runs. Whether it runs at all is the model's to decide."""
    d = np.sort(price.reshape(365, 24), axis=1)
    def block(hrs, dear):
        col = d[:, ::-1] if dear else d
        n, frac = int(hrs), hrs - int(hrs)
        w = np.zeros(24); w[:n] = 1.0
        if frac > 0: w[n] = frac
        return ((col * w).sum(axis=1) / w.sum()).tolist()
    return block(dis_hours, True), block(chg_hours, False)


# Exceedance levels for the duration curve. NOT uniform: a wholesale price
# series is heavy-tailed at the top, and a uniform grid puts the year's single
# scarcity hour at q=1.00 with nothing between it and q=0.95 — which a linear
# read then interprets as 5% of hours priced near the spike. The tails are
# where a battery operates, so the tails are where the curve needs resolution.
LEVELS = [0.0, 0.0005, 0.001, 0.002, 0.005, 0.01, 0.02, 0.03, 0.05, 0.075,
          0.10, 0.15, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.85, 0.90,
          0.925, 0.95, 0.97, 0.98, 0.99, 0.995, 0.998, 0.999, 0.9995, 1.0]


def price_duration_curve(price, levels=LEVELS):
    """Sorted prices at stated exceedance levels. A statistic of the data — no
    dispatch, no battery, no efficiency. Ascending, as CFDL stores it."""
    return [(float(q), float(np.quantile(price, q))) for q in levels]


if __name__ == "__main__":
    price = hourly_prices()
    ref = sam_reference(price)
    usable = NOMINAL_MWH * (SOC_MAX_PCT - SOC_MIN_PCT) / 100.0
    rte = AC_DC_EFF/100 * DC_AC_EFF/100 * 0.975
    onpk, offpk = daily_blocks(price, usable/POWER_MW, usable/rte/POWER_MW)
    json.dump({"assumptions": {
                  "power_mw": POWER_MW, "nominal_mwh": NOMINAL_MWH,
                  "soc_min_pct": SOC_MIN_PCT, "soc_max_pct": SOC_MAX_PCT,
                  "soc_init_pct": SOC_INIT_PCT, "cycle_cost_per_kwh": CYCLE_COST,
                  "ac_dc_eff": AC_DC_EFF, "dc_ac_eff": DC_AC_EFF, "dc_dc_eff": DC_DC_EFF,
                  "look_ahead_hours": LOOK_AHEAD_HOURS, "seed": SEED},
               "sam": ref, "onpeak": onpk, "offpeak": offpk},
              open("reference.json","w"), indent=1)
    print("── matched assumptions ──")
    print(f"  {POWER_MW} MW / {NOMINAL_MWH} MWh nominal, SOC {SOC_MIN_PCT}-{SOC_MAX_PCT}%, "
          f"cycle cost ${CYCLE_COST}/kWh")
    print("── SAM reference ──")
    for k, v in ref.items(): print(f"  {k:20s} {v:,.4f}")
    import statistics as st
    print("── daily block prices ($/MWh) ──")
    print(f"  on-peak  mean {st.fmean(onpk):6.2f}  min {min(onpk):6.2f}  max {max(onpk):8.2f}")
    print(f"  off-peak mean {st.fmean(offpk):6.2f}  min {min(offpk):6.2f}  max {max(offpk):8.2f}")
