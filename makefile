# CFDL v0.1 — Makefile
# Provides canonical commands for agents + CI.

SHELL := /bin/bash

.PHONY: pack-series pack-templates keyword-register ci-gates invariants glossary glossary-check machine-docs machine-docs-check agent-eval-selftest agent-eval-replay shipped-examples benchmark-cases help fmt fmt-check lint test build clean gold gold-update ci verify site-voice verify-python verify-site verify-site-nofresh verify-site-fresh verify-learn-nofresh doc-examples training-examples py-develop py-test py-wheel notebooks-render notebooks-check wasm cadence-parity ir-schema results-schema run-schema pack-validations rule-fragments py-stamp py-check

help:
	@echo "Targets:"
	@echo "  fmt         - format Rust code (cargo fmt)"
	@echo "  lint        - run clippy (cargo clippy)"
	@echo "  test        - run tests (cargo test)"
	@echo "  build       - build all crates (cargo build)"
	@echo "  clean       - remove build artifacts"
	@echo "  gold        - run golden suite (tools/golden-runner)"
	@echo "  gold-update - update gold outputs (DANGEROUS; requires intent)"
	@echo "  ci          - the FAST SUBSET: Rust workspace + tool gates"
	@echo "  verify      - EVERYTHING CI runs; use this before pushing"
	@echo "  py-develop  - maturin develop the Python SDK (editable, [dev,viz])"
	@echo "  doc-examples - compile and run every example in the pack guides"
	@echo "  wasm        - build the playground wasm bundle locally"
	@echo "  cadence-parity - one deal on every calendar must give the same annual economics"
	@echo "  py-test     - run the Python SDK pytest suite"
	@echo "  notebooks-render - execute example notebooks into site docs pages"
	@echo "  py-wheel    - build a local release wheel (sanity check)"

fmt:
	cargo fmt --all

# What `ci` runs. `fmt` REWRITES and therefore always succeeds, so a `ci` that
# depended on it could never fail on formatting while CI — which checks — could.
# Exactly the local/CI divergence this file exists to prevent, one level down.
fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all --all-features

build:
	cargo build --all --all-features

clean:
	cargo clean
	rm -f *.ir.json *.results.json *.txt

# PEP 597. Text I/O that relies on the platform default encoding is an error
# for the gate scripts. Windows decodes as cp1252, so a `read_text()` with no
# encoding crashes there and nowhere else the moment the file it reads grows a
# curly quote. Scoped to the gates deliberately: pytest and maturin run
# third-party code that is not ours to hold to this.
PYGATE := PYTHONWARNDEFAULTENCODING=1 PYTHONWARNINGS=error::EncodingWarning python3

gold:
	@./tools/golden-runner run

gold-update:
	@CFDL_GOLD_UPDATE=1 ./tools/golden-runner run

bench:
	cargo build -p cfdl-cli
	$(PYGATE) tools/benchmark-runner.py

# THE GATE LIST LIVES HERE AND NOWHERE ELSE.
#
# It used to live in two places — these targets, and the same commands inlined
# into .github/workflows — with neither authoritative. It drifted in both
# directions and both hurt:
#
#   - CI was once the shorter list, and a 23% error in weighted average life
#     survived because `analytic-checks` only ran when someone remembered to.
#     That is recorded at .github/workflows/ci.yml.
#   - `make ci` was later the shorter list, and four gates (sync:check,
#     py-test, check-notebooks-fresh, and the site checks) caught things only
#     after a push, while a full local `make ci` was green.
#
# The workflows now CALL these targets rather than restating them, so there is
# one definition and local-equals-CI holds by construction. Adding a gate to a
# workflow without adding it here is the mistake this arrangement prevents:
# there is nowhere else to add it.

# The fast inner loop: the Rust workspace and the gates that need only it.
# Deliberately NOT everything — see `verify`.
#
# NO WASM GATE HERE, and none anywhere local. The bundle is not committed: CI
# builds it from the current sources immediately before deploying and checks it
# there. A release build of the whole engine used to be required just to satisfy
# a freshness stamp on a 2 MB artifact only the website consumes, which was the
# single largest cost in this loop.
# WHAT CI RUNS BEYOND THE PLATFORM MATRIX. Named so `.github/workflows/ci.yml`
# calls one target instead of restating tool invocations — which is how seven
# gates came to run locally and never in CI, `check-run-schema` among them, and
# how a 23% weighted-average-life error once survived because `analytic-checks`
# was in this file and not in the workflow.
ci-gates: analytic invariants cadence-parity ir-schema results-schema run-schema \
          pack-validations pack-series pack-templates keyword-register site-voice \
          glossary-check machine-docs-check agent-eval-selftest \
          rule-fragments \
          doc-examples training-examples shipped-examples benchmark-cases
	@echo
	@echo "make ci-gates: OK — every gate that is not the platform matrix."

ci: fmt-check lint test gold bench ci-gates
	@echo
	@echo "make ci: OK — but this is the FAST SUBSET, not the whole suite."
	@echo "  Not run here: py-test, notebooks-check, and the site gates"
	@echo "  (sync:check, check:tokens, check:links, check:examples,"
	@echo "   check:dialogs)."
	@echo "  They need a Python venv and node_modules, which the Rust loop"
	@echo "  should not have to install. Before pushing:  make verify"

# Everything CI runs. What to run before pushing.
verify: ci verify-python verify-site
	@echo
	@echo "make verify: OK — every gate CI runs has passed locally."

# Needs `make py-develop` first (editable install + native extension).
verify-python: py-check py-test notebooks-check

# Needs `cd site && npm ci` first.
verify-site: verify-site-nofresh verify-site-fresh

# The site gates that need no git history. Split out because CI runs these on
# every event, while the freshness pair below needs a base ref that differs
# between a pull request and a push.
# The wasm gates are NOT here. The bundle is built in CI immediately before it
# is deployed and is not committed, so there is nothing on a developer's machine
# or in this job for them to check — and a bundle built from the current sources
# is fresh by construction rather than by inspection. They run in the deploy
# job, against the bundle it just produced.
verify-site-nofresh:
	cd site && npm run sync:check
	cd site && npm run check:tokens
	cd site && npm run check:links
	cd site && npm run check:examples
	cd site && npm run check:dialogs
	cd site && npm run check:middleware
	cd site && npm run check:descriptions
	# The learn app mirrors the design system from site/; a site-side edit to a
	# shared file fails here until learn/ is re-synced. Runs plain node, so it
	# needs no npm ci in learn/.
	cd learn && npm run check:shared

# The learn-app gates that need no git history. Same split as the site's for
# the same reason. Needs `cd learn && npm ci` first (check:shared excepted).
verify-learn-nofresh:
	cd learn && npm run check:shared
	cd learn && npm run check:exercises
	cd learn && npm run check:tokens
	cd learn && npm run check:dialogs

# Compare committed artefacts against what the sources would produce, relative
# to BASE_REF.
verify-site-fresh:
	cd site && node scripts/check-notebooks-fresh.mjs "$(BASE_REF)"

# What the freshness gates diff against. CI overrides it with the PR base or
# the previous commit; locally `main` is the useful default.
BASE_REF ?= main

# Build the bundle locally, for `npm run dev` and the playground. Not committed:
# CI builds it from the current sources immediately before deploying.
wasm:
	cd site && npm run build:wasm

# Internal engineering narrative must not reach the documentation site. Lives
# in the fast loop rather than the site gates: it reads repository files and
# needs no node_modules, no venv and no toolchain.
# The glossary page is generated from the terminology register, so a term is
# defined in exactly one place. Lives in the fast loop: it reads two repository
# files and needs no node_modules and no toolchain.
glossary:
	$(PYGATE) tools/gen-glossary.py

glossary-check:
	$(PYGATE) tools/gen-glossary.py --check

# The documentation surface for machines (docs/32 Phase 2): llms.txt, the
# machine docs bundle, llms-full.txt, and the diagnostics -> repair catalog,
# generated from the same sources the site renders. Outputs are committed;
# --check byte-compares, so drift fails the fast loop. Builds cfdl-cli because
# every recorded repair in fixtures/repairs/ must compile.
machine-docs:
	cargo build -p cfdl-cli
	$(PYGATE) tools/gen-machine-docs.py

machine-docs-check:
	cargo build -p cfdl-cli
	$(PYGATE) tools/gen-machine-docs.py --check

# The agent-eval harness self-test (docs/32 Phase 3): the scripted replay
# agent must score 100% on a sampled task set — the check that separates
# harness bugs from model failures. The full replay run is
# `make agent-eval-replay`, before any public claim about agent authoring.
agent-eval-selftest:
	cargo build -p cfdl-cli
	$(PYGATE) tools/agent-eval/runner.py --self-test

agent-eval-replay:
	cargo build -p cfdl-cli
	$(PYGATE) tools/agent-eval/runner.py --tier all --agent replay

site-voice:
	$(PYGATE) tools/check-site-voice.py

# A pack must lower the same deal to the same annual economics on every
# calendar. The golden runner compares a fixture to its own blessed output and
# so structurally cannot express this; it takes two fixtures.
cadence-parity:
	cargo build -p cfdl-cli
	$(PYGATE) tools/cadence-parity.py

# The published IR schema is a contract; check the emitter still satisfies it.
ir-schema:
	$(PYGATE) tools/check-ir-schema.py

# Same, for the results document. Added after all 67 goldens turned out to
# violate it: the version const said 0.1 while the engine emitted 0.2, and two
# whole sections were undeclared. Documentation drifts; a gate does not.
results-schema:
	$(PYGATE) tools/check-results-schema.py

run-schema:
	$(PYGATE) tools/check-run-schema.py

# A diagnostic code is an identifier. Three codes each named two different
# checks before this gate existed, and a fourth collision was created while
# renumbering them — picking a free number by reading the file is not reliable.
pack-validations:
	$(PYGATE) tools/check-pack-validations.py

rule-fragments:
	$(PYGATE) tools/check-rule-fragments.py

# A `.*` selector matches a stream family AND its bare name; a bare selector
# matches only the bare name. Reading an instanceable family without the glob
# therefore skips every suffixed instance — and nothing warns, because the
# pattern did match something. It reached main twice in the same expression,
# both times in forward NOI, both times moving the exit price.
pack-series:
	$(PYGATE) tools/check-pack-series.py

# docs/01 §18 is the published list of what a modeller may not name a thing.
# The lexer reserved 95 words and §18 listed 57, so 38 identifiers stopped
# working with nothing to explain why — and §18 documented weekday anchors for a
# weekly schedule that no production has ever read.
keyword-register:
	$(PYGATE) tools/check-keyword-register.py

# A template is what the editor inserts when a modeller reaches for a contract.
# One that does not compile teaches a shape the language rejects, and the
# modeller ends up debugging the pack's own snippet.
pack-templates:
	cargo build -p cfdl-cli
	$(PYGATE) tools/check-pack-templates.py

# Closed-form finance the engine must satisfy regardless of implementation.
# The benchmark suite compares against reference implementations, which cannot
# catch a convention both sides share; these identities can.
analytic:
	cargo build -p cfdl-cli
	$(PYGATE) tools/analytic-checks.py

# Properties the engine must hold whatever a model says: streams are the only
# cash, and a contract accounts for every clause a stream has. Each exists
# because its violation was found by hand first (docs/13 7.41).
invariants:
	cargo build -p cfdl-cli
	$(PYGATE) tools/invariant-checks.py

# Documentation examples must compile, run, and exercise what they claim.
benchmark-cases:
	$(PYGATE) tools/check-benchmark-cases.py

doc-examples:
	cargo build -p cfdl-cli
	$(PYGATE) tools/check-doc-examples.py

training-examples:
	cargo build -p cfdl-cli
	$(PYGATE) tools/check-training-examples.py

# A shipped example must run the way a reader will run it: with its own
# run.json. The Monte Carlo lesson shipped a config the engine rejected.
shipped-examples:
	cargo build -p cfdl-cli
	$(PYGATE) tools/check-shipped-examples.py

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