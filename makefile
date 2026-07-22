# CFDL v0.1 — Makefile
# Provides canonical commands for agents + CI.

SHELL := /bin/bash

.PHONY: help fmt lint test build clean gold gold-update ci py-develop py-test py-wheel

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
	@echo "  py-test     - run the Python SDK pytest suite"
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

ci: fmt lint test gold bench

py-develop:
	pip install -e "python/[dev,viz]"

py-test:
	python3 -m pytest -q python/tests

py-wheel:
	cd python && maturin build --release