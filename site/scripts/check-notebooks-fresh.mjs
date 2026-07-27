#!/usr/bin/env node
/**
 * Staleness guard for the committed notebook pages.
 *
 * Rendering a notebook means executing it, which needs Python and the compiled
 * Rust extension. The site CI runner has neither, and nor does Vercel, so
 * site/content/docs/notebooks is committed. The risk is a change to a notebook
 * — or to the packs and engine whose numbers the notebooks print — landing
 * without a re-render, leaving the site publishing outputs that no longer match
 * what the code produces.
 *
 * Like check-wasm-fresh.mjs, this compares *what changed* rather than
 * re-rendering: reproducing the output requires the toolchain this runner
 * doesn't have. Execution correctness is separately covered by the notebook
 * step in ci.yml, which does have Python and Rust.
 *
 * Usage: node scripts/check-notebooks-fresh.mjs <base-ref>   (default: origin/main)
 */
import { execSync } from "node:child_process";

const base = process.argv[2] || "origin/main";

/** Inputs whose change can alter what a rendered page shows. */
const SOURCE_PATHS = [
  "examples/notebooks/",
  "benchmarks/",
  "packs/",
  "python/cfdl_sdk/",
  "crates/cfdl-py/",
  "crates/cfdl-compile/",
  "crates/cfdl-engine/",
  "crates/cfdl-metrics/",
  "crates/cfdl-pack/",
  "crates/cfdl-calc/",
  "crates/cfdl-parser/",
  "crates/cfdl-lexer/",
  "crates/cfdl-resolver/",
  "crates/cfdl-validate/",
  "tools/render-notebooks.py",
];
const RENDER_PATHS = ["site/content/docs/notebooks/", "site/public/notebooks/"];

let changed;
try {
  changed = execSync(`git diff --name-only ${base}...HEAD`, { encoding: "utf8" })
    .split("\n")
    .filter(Boolean);
} catch {
  console.log(`check-notebooks-fresh: cannot diff against ${base} — skipping.`);
  process.exit(0);
}

const sourceChanges = changed.filter((f) => SOURCE_PATHS.some((p) => f.startsWith(p)));
const rendersChanged = changed.some((f) => RENDER_PATHS.some((p) => f.startsWith(p)));

if (sourceChanges.length > 0 && !rendersChanged) {
  console.error(
    "check-notebooks-fresh: notebook inputs changed but the rendered pages were not regenerated.\n",
  );
  for (const f of sourceChanges.slice(0, 10)) console.error(`  ${f}`);
  if (sourceChanges.length > 10) console.error(`  … and ${sourceChanges.length - 10} more`);
  console.error("\nRe-render and commit the pages:\n  make notebooks-render");
  process.exit(1);
}

console.log(
  sourceChanges.length === 0
    ? "check-notebooks-fresh: OK (no notebook inputs changed in this range)"
    : "check-notebooks-fresh: OK (inputs changed and the pages were re-rendered)",
);
