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
 * Reproducing the output needs a toolchain this runner does not have, so the
 * gate checks the STAMP instead: `make notebooks-render` records a digest of
 * every input the pages were rendered against, and hashing those same files is
 * something Node can do unaided. A stamp that matches the working tree means
 * the render ran against exactly this content.
 *
 * It replaces a diff-based heuristic — "inputs changed, so the stamp must have
 * changed too" — which had a hole. A stamp committed from a tree carrying
 * uncommitted changes already covers those changes, so once it reaches main a
 * branch containing them can never produce a differing stamp, and the gate
 * fails on a render that is in fact current. Comparing digests asks the
 * question directly and has no such state.
 *
 * Kept in step with `write_render_stamp` in tools/render-notebooks.py; the
 * digests must agree byte for byte, and a mismatch fails this gate loudly.
 *
 * Usage: node scripts/check-notebooks-fresh.mjs [base-ref]   (base-ref unused,
 * accepted so the makefile and workflow calls do not have to change)
 */
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

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

/** Every file under a stamp input, in the order render-notebooks.py walks them. */
function stampFiles(rel) {
  const target = path.join(repoRoot, rel);
  if (!fs.existsSync(target)) return [];
  if (!fs.statSync(target).isDirectory()) return [rel];
  const found = [];
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir)) {
      const full = path.join(dir, entry);
      if (fs.statSync(full).isDirectory()) walk(full);
      else found.push(path.relative(repoRoot, full));
    }
  };
  walk(target);
  // Python sorts pathlib objects, which compares their full path strings.
  return found.sort();
}

const digest = crypto.createHash("sha256");
for (const rel of [...SOURCE_PATHS].sort()) {
  for (const file of stampFiles(rel)) {
    if (path.basename(file) === ".DS_Store") continue;
    digest.update(file);
    digest.update(fs.readFileSync(path.join(repoRoot, file)));
  }
}
const expected = digest.digest("hex");

const stampPath = path.join(repoRoot, "site", "content", "docs", "notebooks", ".render-stamp");
if (!fs.existsSync(stampPath)) {
  console.error("check-notebooks-fresh: no render stamp.\n");
  console.error("Render and commit the pages:\n  make notebooks-render");
  process.exit(1);
}
const committed = fs.readFileSync(stampPath, "utf8").trim();

if (committed !== expected) {
  console.error(
    "check-notebooks-fresh: the rendered pages were not produced from these inputs.\n",
  );
  console.error(`  stamp    ${committed}`);
  console.error(`  inputs   ${expected}\n`);
  console.error("Re-render and commit the pages:\n  make notebooks-render");
  process.exit(1);
}

console.log("check-notebooks-fresh: OK (the render ran against these inputs)");
