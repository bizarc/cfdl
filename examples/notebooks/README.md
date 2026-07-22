# CFDL example notebooks

Industry walkthroughs of the CFDL Python SDK, one per pack. Each notebook loads
the corresponding **benchmark** model (validated to the penny against an
independent reference in `benchmarks/`), compiles and runs it, and explores the
results with the pandas accessors.

| Notebook | Pack | Model |
|---|---|---|
| `01_energy_solar_microgrid.ipynb` | energy | solar-plus-storage PPA microgrid |
| `02_cre_office_acquisition.ipynb` | cre | two-tenant office, Argus-style |
| `03_credit_loan_pool.ipynb` | credit | level-pay loan pool |
| `04_opco_lbo.ipynb` | opco | five-year services LBO |

## Run them

From the repository root:

```bash
pip install -e "python/[notebooks]"
jupyter lab examples/notebooks
```

Each notebook resolves the repo root itself, so it also runs headless:

```bash
jupyter nbconvert --to notebook --execute examples/notebooks/01_energy_solar_microgrid.ipynb
```

CI executes all four on every push (Linux) to keep them honest as the SDK and
packs evolve.

## Committing

Notebooks are committed **output-stripped** (no execution outputs, no counts).
Before committing changes, clear outputs — e.g.:

```bash
jupyter nbconvert --clear-output --inplace examples/notebooks/*.ipynb
```
