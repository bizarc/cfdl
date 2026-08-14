# Provenance

**FCFF Simple Ginzu — all-in-one free cash flow to firm valuation model**

| | |
|---|---|
| Author | Aswath Damodaran, NYU Stern School of Business |
| Retrieved | 1 August 2026 |
| URL | https://pages.stern.nyu.edu/~adamodar/pc/fcffsimpleginzu.xlsx |
| Index page | https://pages.stern.nyu.edu/~adamodar/New_Home_Page/spreadsh.htm |
| Size | 281 KB |
| SHA-256 | `d6ffb67d965dc22463e4d013636befab3da431e7fc608936a82612fa73198e72` |

## License

Stated on the author's own index page:

> These spreadsheet programs are in Excel and are not copy protected. Download
> them and feel free to modify them to your own specifications.

No registration, no paywall, no formal open-source license. That permission is
explicit enough to redistribute, so the workbook is committed — the second
source in this repo where a reader can open the reference and mark us, rather
than take the reconciliation on trust. (The first is
`benchmarks/cre/hud_home_multifamily`, a US federal work in the public domain.)

## What was read

Sheet **Input sheet** for the drivers: base revenue, operating margin, effective
and marginal tax rates, sales-to-capital ratio, riskfree rate, and the
convergence years.

Sheet **Valuation output** for the ten-year forecast: revenue growth, revenues,
EBIT, tax rate, EBIT(1-t), reinvestment and FCFF.

The converging growth and tax paths were **derived** from the stated inputs
(5% to the riskfree 4.58%, and 17.5% to the marginal 25%, linearly across years
6-10) and verified to reproduce the published rows exactly, rather than being
read back off the output. Nothing in `expected.csv` is fitted.

The workbook is committed as data. No test opens it.
