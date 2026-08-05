# CFDL v0.1 — Makefile
# Provides canonical commands for agents + CI.

SHELL := /bin/bash

.PHONY: help fmt fmt-check lint test build clean gold gold-update ci verify verify-python verify-site verify-site-nofresh verify-site-fresh doc-examples py-develop py-test py-wheel notebooks-render notebooks-check wasm wasm-check cadence-parity ir-schema results-schema pack-validations rule-fragments py-stamp py-check

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
	@echo "  wasm        - rebuild the committed playground wasm bundle"
	@echo "  wasm-check  - verify the committed bundle matches the engine sources"
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
# `wasm-check` is deliberately ABSENT, and its removal cost no coverage: it runs
# `cd site && npm run check:wasm`, which is character-for-character what
# `verify-site-nofresh` already runs. The gate was in both lists, and the copy
# here was the expensive one.
#
# Expensive because the stamp it checks hashes ENGINE SOURCES, so any engine
# edit fails it, and the only way to pass is a full `wasm-pack --release` build
# of the whole engine — a release build in a loop where nothing else needs one,
# to keep a 2 MB artifact in sync that only the website consumes. It was the
# single largest cost in the edit-test cycle and the reason this loop stopped
# feeling fast.
#
# Nothing moves later than it used to: `verify` is the pre-push gate this
# target's own message points at, and it still runs the check. The rebuild now
# happens once before a push instead of once per `make ci`.
ci: fmt-check lint test gold bench analytic cadence-parity ir-schema results-schema pack-validations rule-fragments doc-examples
	@echo
	@echo "make ci: OK — but this is the FAST SUBSET, not the whole suite."
	@echo "  Not run here: py-test, notebooks-check, and the site gates"
	@echo "  (sync:check, check:tokens, check:links, check:examples,"
	@echo "   check:dialogs, check:wasm, check-wasm-fresh)."
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
verify-site-nofresh:
	# The size budget lives in build-wasm.sh and so only fired on a rebuild —
	# `check:wasm` verifies version, stamp and function, not bytes. That made a
	# breach invisible to `make verify`, which is the gap this file exists to
	# close.
	cd site && node scripts/check-wasm-budget.mjs
	cd site && npm run sync:check
	cd site && npm run check:tokens
	cd site && npm run check:links
	cd site && npm run check:examples
	cd site && npm run check:dialogs
	cd site && npm run check:wasm

# Compare committed artefacts against what the sources would produce, relative
# to BASE_REF.
verify-site-fresh:
	cd site && node scripts/check-wasm-fresh.mjs "$(BASE_REF)"
	cd site && node scripts/check-notebooks-fresh.mjs "$(BASE_REF)"

# What the freshness gates diff against. CI overrides it with the PR base or
# the previous commit; locally `main` is the useful default.
BASE_REF ?= main

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
	$(PYGATE) tools/cadence-parity.py

# The published IR schema is a contract; check the emitter still satisfies it.
ir-schema:
	$(PYGATE) tools/check-ir-schema.py

# Same, for the results document. Added after all 67 goldens turned out to
# violate it: the version const said 0.1 while the engine emitted 0.2, and two
# whole sections were undeclared. Documentation drifts; a gate does not.
results-schema:
	$(PYGATE) tools/check-results-schema.py

# A diagnostic code is an identifier. Three codes each named two different
# checks before this gate existed, and a fourth collision was created while
# renumbering them — picking a free number by reading the file is not reliable.
pack-validations:
	$(PYGATE) tools/check-pack-validations.py

rule-fragments:
	$(PYGATE) tools/check-rule-fragments.py

# Closed-form finance the engine must satisfy regardless of implementation.
# The benchmark suite compares against reference implementations, which cannot
# catch a convention both sides share; these identities can.
analytic:
	cargo build -p cfdl-cli
	$(PYGATE) tools/analytic-checks.py

# Documentation examples must compile, run, and exercise what they claim.
doc-examples:
	cargo build -p cfdl-cli
	$(PYGATE) tools/check-doc-examples.py

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