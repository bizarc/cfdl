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
import fs from "node:fs";
import path from "node:path";

const base = process.argv[2] || "origin/main";

/**
 * Inputs whose change can alter what a rendered page shows.
 *
 * `benchmarks/` is not here as a whole directory. The notebooks read four
 * specific benchmark models, and a case in another pack cannot change what
 * they print — but a directory-wide input made every new benchmark demand a
 * full notebook re-render, which is a false alarm on the action this
 * repository takes most often. The models actually read are discovered from
 * the notebooks below and appended, so the list cannot go stale.
 *
 * Kept in step with STAMP_INPUTS in tools/render-notebooks.py.
 */
const SOURCE_PATHS = [
  "examples/notebooks/",
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

// The benchmark models the notebooks read, taken from the notebooks themselves.
// This script runs from site/, but SOURCE_PATHS are repo-relative because they
// are matched against git output.
const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..", "..");
const notebookDir = path.join(repoRoot, "examples", "notebooks");
for (const notebook of fs.readdirSync(notebookDir).sort()) {
  if (!notebook.endsWith(".ipynb")) continue;
  const text = fs.readFileSync(path.join(notebookDir, notebook), "utf8");
  for (const [, pack, name] of text.matchAll(/benchmarks\/([a-z_]+)\/([a-z_0-9]+)/g)) {
    const dir = `benchmarks/${pack}/${name}/`;
    if (fs.existsSync(path.join(repoRoot, dir)) && !SOURCE_PATHS.includes(dir)) {
      SOURCE_PATHS.push(dir);
    }
  }
}

let changed;
try {
  changed = execSync(`git diff --name-only ${base}...HEAD`, { encoding: "utf8" })
    .split("\n")
    .filter(Boolean);
} catch (error) {
  // Fatal, not skip — see the same change in check-wasm-fresh.mjs. A shallow
  // checkout makes this throw on every CI run, so exiting 0 here disabled the
  // gate entirely without ever saying so.
  console.error(`check-notebooks-fresh: cannot diff against ${base}.\n`);
  console.error(`  ${error instanceof Error ? error.message.split("\n")[0] : String(error)}\n`);
  console.error("The base ref must be present locally. In CI, set:");
  console.error("  - uses: actions/checkout@v4");
  console.error("    with:");
  console.error("      fetch-depth: 0");
  console.error("\nLocally, fetch it:\n  git fetch origin main");
  process.exit(1);
}

const sourceChanges = changed.filter((f) => SOURCE_PATHS.some((p) => f.startsWith(p)));
const rendersChanged = changed.some((f) => RENDER_PATHS.some((p) => f.startsWith(p)));

// A render is fresh when it ran against the current inputs, which the stamp
// records. Requiring a *diff* in the rendered pages was wrong: a change that
// does not alter notebook output — a new diagnostic, say — left the gate
// unsatisfiable, because re-rendering produced nothing to commit.
const stampPath = "site/content/docs/notebooks/.render-stamp";
const stampChanged = changed.includes(stampPath);

if (sourceChanges.length > 0 && !rendersChanged && !stampChanged) {
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
    : "check-notebooks-fresh: OK (inputs changed and the render ran against them)",
);
