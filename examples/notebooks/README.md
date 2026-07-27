# CFDL example notebooks

Industry walkthroughs of the CFDL Python SDK, one per pack. Each notebook loads
the corresponding **benchmark** model (validated to the penny against an
independent reference in `benchmarks/`), compiles and runs it, and explores the
results with the pandas accessors.

| Notebook | Pack | Model | Published page |
|---|---|---|---|
| `01_energy_solar_microgrid.ipynb` | energy | solar-plus-storage PPA microgrid | https://cfdl.dev/docs/notebooks/energy-solar-microgrid |
| `02_cre_office_acquisition.ipynb` | cre | two-tenant office, lease-by-lease DCF | https://cfdl.dev/docs/notebooks/cre-office-acquisition |
| `03_credit_loan_pool.ipynb` | credit | level-pay loan pool | https://cfdl.dev/docs/notebooks/credit-loan-pool |
| `04_opco_lbo.ipynb` | opco | five-year services LBO | https://cfdl.dev/docs/notebooks/opco-lbo |

## Run them

From the repository root:

```bash
pip install -e "python/[notebooks]"
jupyter lab examples/notebooks
```

Each notebook locates the repo root itself, so it also runs headless:

```bash
jupyter nbconvert --to notebook --execute examples/notebooks/01_energy_solar_microgrid.ipynb
```

They read their model from `benchmarks/` and their pack definitions from
`packs/`, so they need a checkout — run one from elsewhere and it says so
rather than failing obscurely.

CI executes all four on every push (Linux) to keep them honest as the SDK and
packs evolve.

## Committing

Notebooks are committed **output-stripped** (no execution outputs, no counts).
Before committing changes, clear outputs — e.g.:

```bash
jupyter nbconvert --clear-output --inplace examples/notebooks/*.ipynb
```

## Publishing

The pages on cfdl.dev are rendered by executing these notebooks, because
neither the site CI runner nor Vercel has Python or Rust. After changing a
notebook — or anything whose numbers it prints, such as a pack or the engine —
regenerate and commit the rendered output:

```bash
make notebooks-render
```

`site/scripts/check-notebooks-fresh.mjs` fails CI if you forget. The rendered
pages under `site/content/docs/notebooks/` and the charts under
`site/public/notebooks/` are derived artifacts: never edit them by hand.
