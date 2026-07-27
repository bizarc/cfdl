# cfdl.dev

The CFDL product site: landing page, documentation, and the in-browser
playground. Next.js (App Router) + Tailwind, deployed on Vercel.

## Develop

```bash
npm install
npm run dev
```

## Checks

```bash
./scripts/check-tokens.sh   # design system: no raw color literals outside tokens
npm run lint
npm run build
```

## Design system

`app/tokens.css` is the single source of color, type, spacing, and motion.
It defines **primitives** (raw ramps and scales) and **semantic tokens**
(`--cfdl-surface-*`, `--cfdl-text-*`, `--cfdl-accent-*`, `--cfdl-chart-*`, …)
for both themes. `app/globals.css` maps Tailwind utilities onto the semantic
layer, so components write `bg-surface-raised` / `text-secondary` and never
name a color. `scripts/check-tokens.sh` enforces this in CI.

## Syntax highlighting

CFDL code is highlighted by Shiki using the **same TextMate grammar the VS
Code extension ships** (`editors/vscode/syntaxes/cfdl.tmLanguage.json`), so
the site and the editor tokenize identically. There is no second grammar to
maintain — see `lib/shiki.ts`.

## Notes

- The hero demo numbers in `components/landing/hero-demo-data.ts` are real
  engine output (documented in the file), not illustrations.
- Vercel builds are Node-only. Anything the site needs from the Rust
  toolchain (the wasm engine bundle) is built by repo scripts, committed,
  and drift-checked in GitHub CI.
