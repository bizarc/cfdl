# Source

**Numerical Model for Financial Simulation of Highway PPP Projects**
(`Numerical_model_Operis_v3.xls`), with its user guide, from the World Bank /
PPIAF *Toolkit for Public-Private Partnerships in Roads and Highways*, Module 6
(Tools), updated March 2009.

- Toolkit landing page:
  https://ppp.worldbank.org/library/toolkit-public-private-partnerships-roads-and-highways
- Workbook (as archived; the live PPIAF paths have since been reorganised):
  `www.ppiaf.org/sites/ppiaf.org/files/documents/toolkits/highwaystoolkit/6/financial_models/Numerical_model_Operis_v3.xls`
- User guide:
  `www.ppiaf.org/sites/ppiaf.org/files/documents/toolkits/highwaystoolkit/6/pdf-version/numerical_model.pdf`

## What was used

The workbook ships a case study as its default values, restorable in-tool with
a "Case study" button. It was opened once, outside this repository, at those
defaults and unmodified. The cached values on the `Funding construction`,
`Debt repayment`, `Operating revenues`, `Cash Flows waterfall`,
`Income Statement`, `Depreciation and VAT`, `Ratios` and `Results` sheets are
the reference; they are carried into `expected.csv` and `expected_metrics.json`
as numbers.

The user guide supplies the assumptions and the stated formulas — the ADSCR
subsidy rule, the CAFDS definition, the mid-year capitalisation of construction
interest, the regressive variable-cost scale and the tax-in-arrears rule. The
model here is built from those, not by transcribing spreadsheet formulas.

## Licence

**Do not vendor.** Both files are freely downloadable and freely citable, and
the toolkit is a World Bank / PPIAF publication, but neither the workbook nor
the guide carries an explicit reuse or redistribution grant. Neither is
committed here. Only the numbers they output are, which is the same treatment
the credit and LBO cases give their sources.

## Deltas between the guide and the workbook

The printed user guide and the workbook's own saved case study disagree in two
places. The workbook is authoritative here, since it is what produced the
cached values:

- **3rd tranche interest rate.** The guide's summary table says 6%; the
  workbook's assumptions sheet holds 5%. This model uses 5%.
- **Construction spend profile.** The guide describes the cost as spread over
  the construction period; the workbook uses an explicit 10 / 30 / 50 / 10
  profile across 2009-2012 and reports its flow chart of works as "Special".
  This model uses 10 / 30 / 50 / 10.

The guide also quotes traffic as "light vehicles (70%) — heavy vehicles (30%)",
while the case study it ships splits the 10,000 vehicles a day evenly between
the two toll categories. The even split is what the workbook computes.
