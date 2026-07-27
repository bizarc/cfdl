# CLAUDE.md — CFDL Operating Rules

You are an implementation agent for **CFDL** (Cash Flow Domain Language), being launched
at CFDL.dev as a standalone, source-available product.

## Start here

1. **The internal launch plan** (kept outside version control — ask the maintainer) —
   locked decisions, workstreams, file ownership,
   coordination rules. Your work belongs to exactly one workstream.
2. `docs/0*.md` + `docs/schemas/` — authoritative language/IR/results specs.
3. `CONTRIBUTING.md` — conventions, golden workflow, change checklist.

## Hard rules

- **Determinism**: same inputs + same pack + same compiler version → identical output.
- **Golden-first**: `make ci` must pass; never hand-edit `gold/`; re-bless only
  intentional changes via `CFDL_GOLD_UPDATE=1 ./tools/golden-runner run`, explained in
  the commit message. Only one workstream re-blesses per merge (see the launch plan coordination rules).
- **Diagnostic codes are stable** — add, never rename/reuse.
- **Branching**: one workstream = one branch (`ws/<letter>-<slug>`); merge to `main`
  only with `make ci` green.
- **Licensing language**: CFDL is "source available" (BSL 1.1), never "open source".
- **No publishing without human approval**: no public repo flip, crates.io, PyPI,
  Marketplace, Pages deploys, or GitHub Releases.
- Strict Rust: `cargo fmt`, `cargo clippy -D warnings`. Conventional Commits.

## Build & test

```bash
make ci      # fmt + clippy + tests + golden suite (59 fixtures) + benchmarks
make gold    # golden suite only
```

## The website (`site/`)

cfdl.dev is a Next.js app in `site/`, deployed on Vercel. Rules:

- **Docs content under `site/content/docs/` is generated** from the canonical
  sources (`docs/0*.md`, `packs/*/README.md`, `examples/`, `benchmarks/`,
  `distribution/install-configure.md`) by `site/scripts/sync-content.mjs`.
  Never hand-edit a generated page — fix the canonical source and re-run
  `npm run sync:content`. CI diff-checks it.
  Hand-written pages (getting-started, concepts, install/*, guides/*, faq,
  troubleshooting, packs/index, reference/*) live in the same tree and are
  edited directly.
- **Design system**: `site/app/tokens.css` is the only place a raw color may
  appear. Components use semantic tokens (`bg-surface-raised`,
  `text-secondary`, …); `npm run check:tokens` enforces it.
- **Syntax highlighting** uses the VS Code extension's TextMate grammar
  directly — one grammar for editor and site.
- Before merging site changes: `npm run sync:check && npm run check:tokens
  && npm run check:links && npm run lint && npm run build`.
