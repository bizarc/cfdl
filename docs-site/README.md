# CFDL Docs Site

This folder hosts the Docusaurus documentation site for CFDL onboarding and references.

## Framework decision (2026-07-22): keep Docusaurus

The launch plan floated migrating to Astro Starlight. **Decision: stay on
Docusaurus 3.9.** The site already works, has a content-sync pipeline
(`scripts/sync-content.mjs` regenerates pages from the canonical `docs/`,
`packs/`, and `benchmarks/` sources), and deploys via CI. A Starlight
migration would mean rewriting the sync pipeline and re-theming two months
before launch, for no capability we need — the playground (Workstream F3)
embeds fine as a Docusaurus custom page. Revisit post-1.0 only if the site's
needs outgrow Docusaurus.

## Content sync

Pages under `docs/` are **generated** — do not hand-edit them. Edit the
canonical source (root `docs/0X_*.md`, `packs/*/README.md`) and run:

```bash
npm run sync:content     # regenerate; sync:check --check verifies freshness in CI
```

The sync also stages the JSON schemas at `static/schemas/` so they serve at
their `$id` paths (`/schemas/CFDL_v0_1_{IR,Results}.schema.json`).

## Local development

```bash
cd docs-site
npm install
npm run start
```

## Production build

```bash
npm run build
```
