# CFDL v0.1 — Makefile
# Provides canonical commands for agents + CI.

SHELL := /bin/bash

.PHONY: help fmt lint test build clean gold gold-update ci doc-examples py-develop py-test py-wheel notebooks-render notebooks-check

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

ci: fmt lint test gold bench analytic ir-schema doc-examples

# The published IR schema is a contract; check the emitter still satisfies it.
ir-schema:
	python3 tools/check-ir-schema.py

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

py-test:
	python3 -m pytest -q python/tests

py-wheel:
	cd python && maturin build --release

# Execute the example notebooks and publish them as docs pages. Needed because
# neither the site CI runner nor Vercel has Python or Rust, so the rendered
# output is committed; check-notebooks-fresh.mjs guards it against going stale.
notebooks-render:
	python3 tools/render-notebooks.py

notebooks-check:
	python3 tools/render-notebooks.py --check