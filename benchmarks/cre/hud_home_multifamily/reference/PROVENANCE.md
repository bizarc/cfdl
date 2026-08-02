# Provenance

**HOME Multifamily Underwriting Template — populated Sample workbook**

| | |
|---|---|
| Publisher | U.S. Department of Housing and Urban Development (HUD Exchange) |
| Retrieved | 1 August 2026 |
| URL | https://files.hudexchange.info/resources/documents/HOME-Multifamily-Underwriting-Template-Sample.xlsm |
| Blank template | https://files.hudexchange.info/resources/documents/HOME-Multifamily-Underwriting-Template.xlsm |
| Size | 530 KB |
| SHA-256 | `95c74094962567f915429875c6d5df8a82b107a467a908b7f8e6b17987cc41af` |

## Why this file is committed and the other references are not

Works of the U.S. federal government are not subject to domestic copyright, and
the HUD Exchange resource page carries a public-domain dedication. This is the
only source in the validation programme that may be redistributed, so it is the
only one committed.

Every other external case in `benchmarks/` reconciles against a document that
is free to read and not free to republish — an industry standards text, an
issuer's filing, a bank's discussion materials. Those cases assert against
published *numbers*, which are facts, and record the reconciliation in NOTES.md
without vendoring anything.

Committing this one means a reader can open the workbook, find the Operating
Pro Forma tab, and check every figure in `expected.csv` themselves. That is a
materially stronger claim than any other case in this repo can make.

## What was read

Sheet **Operating Pro Forma**, 29 annual columns. Rows: Gross Potential Rent,
rent loss, other revenue, total expenses, replacement reserve deposit, net
operating income, debt service, and the two rent tracks the affordability
switch selects between. Sheet **Pro Forma Assumptions** supplies the trends and
the published debt-service coverage ratios.

The workbook contains macros (`.xlsm`). It is committed as data and is never
opened or executed by any test; the figures were read once with a spreadsheet
reader and transcribed into `expected.csv`.
