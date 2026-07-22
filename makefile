# CFDL v0.1 — Makefile
# Provides canonical commands for agents + CI.

SHELL := /bin/bash

.PHONY: help fmt lint test build clean gold gold-update ci

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