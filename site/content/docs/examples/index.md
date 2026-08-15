---
id: examples
title: "Examples"
slug: "/docs/examples"
description: "Complete CFDL models that run: eight short lessons, a few longer domain models, and the benchmark models checked against published references."
---

Every example on this page is a complete model that runs. They come in three
kinds: eight short lessons that build the language one construct at a time,
a few longer domain models, and twenty-five benchmark models checked against
published references.

## Lessons

Read in order. Each adds one construct to the model before it.

- [Minimal model](/docs/examples/minimal_model)
- [Your first stream](/docs/examples/first_stream)
- [A simple contract](/docs/examples/simple_contract)
- [Using an industry pack](/docs/examples/with_pack)
- [Multi-file model](/docs/examples/multi_file)
- [Curves](/docs/examples/curves)
- [Uncertainty and Monte Carlo](/docs/examples/uncertainty)
- [Events and options](/docs/examples/options_events)

## Domain models

Longer models that put the constructs together.

- [CRE examples](/docs/examples/cre-examples) — lease-up, developer lifecycle, phased development, multi-file, development with financing.
- [Operating business examples](/docs/examples/operating-business-examples) — revenue, opex, working capital, exit multiple, growth, multi-file.


## Benchmark models

Complete models for every pack, each checked period by period against an independent reference implementation. These detailed examples have been verified. How that is done is on the [validation](/docs/benchmarks) page.

### Energy

- [Energy: cost-based solar feed-in tariff](/docs/examples/energy-crest-solar-cost-based) — A distributed solar project paid a cost-based feed-in tariff, with an abating payment in lieu of property tax and a revenue-linked royalty.
- [Energy: merchant generator with capacity revenue](/docs/examples/energy-merchant-capacity) — A merchant generator earning both energy and capacity revenue, exposed to price rather than to a contracted offtake.
- [Energy: solar PPA microgrid](/docs/examples/energy-solar-ppa-microgrid) — A solar microgrid selling under a long-term power purchase agreement, with production degradation and a fixed escalator on the contracted price.
- [Energy: a tax-equity flip, with the date derived](/docs/examples/energy-tax-equity-flip) — A tax-equity partnership whose flip date is derived from the investor's return rather than stated, reconciled against an external model.
- [Energy: utility-scale PV, single owner](/docs/examples/energy-utility-pv-singleowner) — A utility-scale photovoltaic project in a single-owner structure, carrying its own tax position rather than allocating to an investor.
- [Energy: wind with PTC and MACRS](/docs/examples/energy-wind-ptc-macrs) — A wind project claiming the production tax credit over ten years and depreciating on the MACRS five-year schedule.

### Commercial real estate

- [CRE: HOME-funded affordable multifamily](/docs/examples/cre-hud-home-multifamily) — A 29-year affordable multifamily underwriting from HUD's HOME Multifamily template, with restricted rents reverting to market at year 15 and a first mortgage that matures before the hold ends.
- [CRE: rent-regulated plaza](/docs/examples/cre-mit-rentleg-plaza) — A five-year office acquisition and disposition from MIT's real estate finance course, valued on a levered before-tax cash flow with an exit at a stated cap rate.
- [CRE: two-tenant office](/docs/examples/cre-office-two-tenant) — An institutional two-tenant office DCF: free rent, anniversary escalations, recoveries above expense stops, tenant improvements and leasing commissions, probability-blended rollover, and a forward-NOI exit over ten years.
- [CRE: office development joint venture](/docs/examples/cre-one-lincoln-street) — A ground-up office development drawing on a construction facility, capitalizing interest through the build, then stabilizing and refinancing.
- [CRE: office development, through the pack contract](/docs/examples/cre-one-lincoln-street-contract) — The same published construction schedule as the native case, declared as one cre.construction_loan contract — equity first, the facility behind it, interest on the drawn balance.
- [CRE: retail strip with expense stops](/docs/examples/cre-retail-strip) — A retail strip center with base-year expense gross-ups, percentage rent over a breakpoint, and staggered tenant rollover across a ten-year hold.

### Credit

- [Credit: auto ABS at 0.5x prepayment speed](/docs/examples/credit-auto-abs-speed-050) — An auto loan pool prepaying at 0.5 ABS, amortizing to schedule with prepayments taken as a constant share of the original balance.
- [Credit: auto ABS at 1.5x prepayment speed](/docs/examples/credit-auto-abs-speed-150) — The same auto loan pool at 1.5 ABS, three times the prepayment speed, showing how the collection profile shortens.
- [Credit: auto ABS note classes](/docs/examples/credit-auto-abs-tranches) — The note classes of an auto ABS: one ordered waterfall paying six classes by seniority, reconciled against the issuer's published percent-outstanding grid at every distribution date.
- [Credit: auto ABS weighted average life](/docs/examples/credit-auto-abs-wal) — An auto loan pool measured for weighted average life, the standard summary of when principal actually comes back.
- [Credit: floating-rate bridge pool](/docs/examples/credit-float-bridge-pool) — A floating-rate bridge loan pool priced off a forward curve, where the coupon resets each period rather than being fixed at origination.
- [Credit: Fannie Mae REMIC with a stripped coupon](/docs/examples/credit-fnma-remic-2019-2-g3) — Security Group 3 of a Fannie Mae REMIC: a seasoned mortgage pool passing through to a single class, with the coupon stripped between it and an interest-only class that carries no principal.
- [Credit: Fannie Mae REMIC at 0% PSA](/docs/examples/credit-fnma-remic-2019-2-g3-psa000) — Group 3 of a Fannie Mae REMIC with the mortgage loans never prepaying — the supplement's own alternative collateral of new 7.50% thirty-year loans, amortizing on schedule for thirty years.
- [Credit: Fannie Mae REMIC at 100% PSA](/docs/examples/credit-fnma-remic-2019-2-g3-psa100) — Group 3 of a Fannie Mae REMIC at 100% of the standard prepayment curve — the slow column of the issuer's decrement table.
- [Credit: Fannie Mae REMIC at 1000% PSA](/docs/examples/credit-fnma-remic-2019-2-g3-psa1000) — Group 3 of a Fannie Mae REMIC at 1000% PSA — the table's fastest column, past half the pool prepaying every year.
- [Credit: Fannie Mae REMIC at 300% PSA](/docs/examples/credit-fnma-remic-2019-2-g3-psa300) — Group 3 of a Fannie Mae REMIC at 300% PSA — one and a half times its pricing speed, retiring the pass-through class in fifteen years.
- [Credit: Fannie Mae REMIC at 400% PSA](/docs/examples/credit-fnma-remic-2019-2-g3-psa400) — Group 3 of a Fannie Mae REMIC at 400% PSA — a fast pool, and the column whose weighted average life pins the timing convention hardest.
- [Credit: Fannie Mae REMIC at 700% PSA](/docs/examples/credit-fnma-remic-2019-2-g3-psa700) — Group 3 of a Fannie Mae REMIC at 700% PSA — a refinancing wave, with the class under two years of average life.
- [Credit: IO/bullet bridge loan](/docs/examples/credit-io-bullet-loan) — An interest-only loan repaying its entire principal in a single balloon at maturity.
- [Credit: level-pay auto pool](/docs/examples/credit-level-pay-pool) — A level-payment amortizing loan pool — the constant instalment that splits into shrinking interest and growing principal.
- [Credit: a mortgage pool modeled loan by loan](/docs/examples/credit-mbs-pool-by-loan) — The same mortgage pool declared loan by loan, with the published pool schedule asserted against the aggregate the engine rolls up from its children.
- [Credit: mortgage pool conventions](/docs/examples/credit-mbs-pool-conventions) — A mortgage pool priced under standard market conventions, reconciling published factors, CPR and SMM against a fixed prepayment vector.
- [Credit: mortgage pool on a prepayment ramp](/docs/examples/credit-mbs-pool-ramped) — A mortgage pool on a ramping prepayment curve, where speeds build over the first thirty months before levelling off.

### Operating businesses

- [OpCo: banker DCF conventions](/docs/examples/opco-banker-dcf-conventions) — An operating company discounted cash flow built to standard banking conventions, from revenue through unlevered free cash flow to enterprise value.
- [OpCo: free cash flow to firm](/docs/examples/opco-damodaran-fcff) — A free cash flow to firm valuation following Damodaran's published method, with reinvestment driven by growth and return on capital.
- [OpCo: stable-growth dividend discount](/docs/examples/opco-gordon-growth-coned) — A Gordon growth valuation of a regulated utility, where a perpetual dividend growing at a constant rate collapses to a closed form.
- [OpCo: leveraged buyout](/docs/examples/opco-lbo-buyout) — A leveraged buyout: entry at a stated multiple, debt paid down out of operating cash flow, and an exit that returns the sponsor's equity.
- [OpCo: LBO debt schedule with average-balance interest](/docs/examples/opco-lbo-circular-interest) — A leveraged buyout's debt schedule, where interest accrues on the average balance and every dollar of free cash flow sweeps against the term loan.
- [OpCo: one buyout at three capital structures](/docs/examples/opco-lbo-financing-cases) — One sponsor buyout run at three capital structures, with the published five-year multiple and return reproduced for each.
- [OpCo: LBO exit waterfall with an option pool](/docs/examples/opco-lbo-option-pool-exit) — A leveraged buyout's exit waterfall, splitting proceeds between an accruing preferred, rolled-over management equity and a laddered management option pool.
- [OpCo: SaaS DCF and the stock-compensation fork](/docs/examples/opco-saas-sbc-convention-fork) — A subscription software business valued on discounted cash flow, with stock-based compensation carried as its own line so the same model states value before and after it.

### Without a pack

- [Bespoke: open-pit copper mine](/docs/examples/bespoke-buenavista-del-cobre) — A 41-year open-pit copper mine from Southern Copper's SEC technical report, carrying three payable metals, six cost lines and a four-layer Mexican fiscal stack that resolves in one pass without a solver.
- [Bespoke: tolled highway PPP concession](/docs/examples/bespoke-ppiaf-toll-highway) — A 125 km toll highway concession from the World Bank's highway PPP toolkit, financed with three debt tranches and topped up each year by an availability subsidy sized to hold debt service cover at 1.30x.