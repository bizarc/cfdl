# CFDL v0.1 — Makefile
# Provides canonical commands for agents + CI.

SHELL := /bin/bash

.PHONY: help fmt lint test build clean gold gold-update ci doc-examples py-develop py-test py-wheel notebooks-render notebooks-check wasm wasm-check cadence-parity ir-schema results-schema pack-validations rule-fragments py-stamp py-check

help:
	@echo "Targets:"
	@echo "  fmt         - format Rust code (cargo fmt)"
	@echo "  lint        - run clippy (cargo clippy)"
	@echo "  test        - run tests (cargo test)"
	@echo "  build       - build all crates (cargo build)"
	@echo "  clean       - remove build artifacts"
	@echo "  gold        - run golden suite (tools/golden-runner)"
	@echo "  gold-update - update gold outputs (DANGEROUS; requires intent)"
	@echo "  ci          - run fmt+lint+test+gold (CI parity)"
	@echo "  py-develop  - maturin develop the Python SDK (editable, [dev,viz])"
	@echo "  doc-examples - compile and run every example in the pack guides"
	@echo "  wasm        - rebuild the committed playground wasm bundle"
	@echo "  wasm-check  - verify the committed bundle matches the engine sources"
	@echo "  cadence-parity - one deal on every calendar must give the same annual economics"
	@echo "  py-test     - run the Python SDK pytest suite"
	@echo "  notebooks-render - execute example notebooks into site docs pages"
	@echo "  py-wheel    - build a local release wheel (sanity check)"

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all --all-features

build:
	cargo build --all --all-features

clean:
	cargo clean
	rm -f *.ir.json *.results.json *.txt

gold:
	@./tools/golden-runner run

gold-update:
	@CFDL_GOLD_UPDATE=1 ./tools/golden-runner run

bench:
	cargo build -p cfdl-cli
	python3 tools/benchmark-runner.py

ci: fmt lint test gold bench analytic cadence-parity ir-schema results-schema pack-validations rule-fragments doc-examples wasm-check

# The wasm bundle is committed (Vercel has no Rust toolchain), so it can drift
# from the engine silently. `make ci` never covered it, and a five-day-old
# bundle once shipped a playground that rejected every `schedule every`.
wasm:
	cd site && npm run build:wasm

wasm-check:
	cd site && npm run check:wasm

# A pack must lower the same deal to the same annual economics on every
# calendar. The golden runner compares a fixture to its own blessed output and
# so structurally cannot express this; it takes two fixtures.
cadence-parity:
	cargo build -p cfdl-cli
	python3 tools/cadence-parity.py

# The published IR schema is a contract; check the emitter still satisfies it.
ir-schema:
	python3 tools/check-ir-schema.py

# Same, for the results document. Added after all 67 goldens turned out to
# violate it: the version const said 0.1 while the engine emitted 0.2, and two
# whole sections were undeclared. Documentation drifts; a gate does not.
results-schema:
	python3 tools/check-results-schema.py

# A diagnostic code is an identifier. Three codes each named two different
# checks before this gate existed, and a fourth collision was created while
# renumbering them — picking a free number by reading the file is not reliable.
pack-validations:
	python3 tools/check-pack-validations.py

rule-fragments:
	python3 tools/check-rule-fragments.py

# Closed-form finance the engine must satisfy regardless of implementation.
# The benchmark suite compares against reference implementations, which cannot
# catch a convention both sides share; these identities can.
analytic:
	cargo build -p cfdl-cli
	python3 tools/analytic-checks.py

# Documentation examples must compile, run, and exercise what they claim.
doc-examples:
	cargo build -p cfdl-cli
	python3 tools/check-doc-examples.py

py-develop:
	pip install -e "python/[dev,viz]"
	python3 tools/py-stamp.py --write

# The compiled half of cfdl_sdk is a local build artifact that nothing tracked,
# so it drifted from the working tree silently and surfaced as a bogus pack
# error. Same source-hash gate as the wasm bundle, for the same reason.
py-check:
	python3 tools/py-stamp.py --check

py-test:
	python3 -m pytest -q python/tests

py-wheel:
	cd python && maturin build --release

# Execute the example notebooks and publish them as docs pages. Needed because
# neither the site CI runner nor Vercel has Python or Rust, so the rendered
# output is committed; check-notebooks-fresh.mjs guards it against going stale.
notebooks-render: py-check
	python3 tools/render-notebooks.py

notebooks-check: py-check
	python3 tools/render-notebooks.py --check