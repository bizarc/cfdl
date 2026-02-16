# CRE Development with Financing (Time-Bounded Streams)

This example models a development lifecycle with **construction-phase financing** (interest-only) and **permanent financing** (debt service). The transition is **hardcoded by stream dates**: no events are used.

- **Construction interest:** `loan.construction` — stream runs from 2026-01 to 2027-06 (18 months).
- **Permanent debt service:** `loan.permanent` — stream runs from 2027-07 to 2031-12 (after conversion).

Property-side cash flows use the CRE pack (construction stub, lease, ops, exit). Loan-side streams are core streams with fixed schedule ranges.

## Compile

```bash
./target/debug/cfdl compile examples/cre_development_with_financing --out /tmp/cre_fin.ir.json --packs packs
```

## Run

```bash
./target/debug/cfdl run /tmp/cre_fin.ir.json --out /tmp/cre_fin.results.json --config examples/cre_development_with_financing/run.json --packs packs
```
