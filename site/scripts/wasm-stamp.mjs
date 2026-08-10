#!/usr/bin/env node
/**
 * Source stamp for the committed wasm bundle.
 *
 * The third of three layered gates, and the one that catches what the other
 * two cannot:
 *
 *   check-wasm-version  — bundle's engine version vs the workspace version.
 *                         Exact and unskippable, but blind to any change that
 *                         ships without a version bump.
 *   check-wasm-fresh    — did a commit range touch engine sources without
 *                         touching the bundle. Broad, but needs git history
 *                         and needs its workflow to have been triggered.
 *   wasm-stamp (this)   — a SHA-256 over the exact source bytes the bundle was
 *                         built from. Needs neither git nor a version bump.
 *
 * Hashing sources rather than the built artifact is deliberate. wasm-pack,
 * binaryen's wasm-opt and the `/rustc/<hash>/` path prefixes baked into the
 * module all vary by machine, so byte-comparing two honest builds fails on
 * machines that are perfectly in sync. Source bytes do not vary.
 *
 * `packs/` is in the input set because crates/cfdl-pack `include_str!`s every
 * pack TOML at compile time — editing a lowering rule changes the bundle with
 * no Rust source change at all.
 *
 * Usage:
 *   node scripts/wasm-stamp.mjs --write    (called by build-wasm.sh)
 *   node scripts/wasm-stamp.mjs --check    (called by CI)
 */
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, relative, sep } from "node:path";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
// WASM_STAMP_DIR lets a sibling app's bundle (learn/public/wasm) carry its
// own stamp; unset, the site's location is unchanged.
const STAMP = join(
  process.env.WASM_STAMP_DIR ?? join(repoRoot, "site", "public", "wasm"),
  ".build-stamp",
);

/** Everything whose bytes end up inside the bundle. Keep in sync with
 *  ENGINE_PATHS in check-wasm-fresh.mjs. */
const INPUTS = [
  "crates/cfdl-wasm",
  "crates/cfdl-compile",
  "crates/cfdl-engine",
  "crates/cfdl-metrics",
  "crates/cfdl-pack",
  "crates/cfdl-calc",
  "crates/cfdl-expr",
  "crates/cfdl-parser",
  "crates/cfdl-lexer",
  "crates/cfdl-resolver",
  "crates/cfdl-validate",
  "packs",
  "Cargo.toml",
];

const SKIP_DIRS = new Set(["target", "node_modules", ".git"]);
const SKIP_FILES = new Set([".DS_Store"]);

function walk(abs, out) {
  let st;
  try {
    st = statSync(abs);
  } catch {
    return; // a declared input that does not exist is simply not hashed
  }
  if (st.isFile()) {
    if (!SKIP_FILES.has(abs.split(sep).pop())) out.push(abs);
    return;
  }
  if (!st.isDirectory()) return;
  for (const entry of readdirSync(abs).sort()) {
    if (SKIP_DIRS.has(entry)) continue;
    walk(join(abs, entry), out);
  }
}

function digest() {
  const files = [];
  for (const rel of [...INPUTS].sort()) walk(join(repoRoot, rel), files);
  files.sort();
  const hash = createHash("sha256");
  for (const f of files) {
    hash.update(relative(repoRoot, f).split(sep).join("/"));
    hash.update(readFileSync(f));
  }
  return hash.digest("hex");
}

const mode = process.argv[2] ?? "--check";
const actual = digest();

if (mode === "--write") {
  writeFileSync(STAMP, actual + "\n");
  // The path, not a hardcoded one: WASM_STAMP_DIR points this at learn/ too,
  // and a message naming site/ while writing to learn/ sent one debugging
  // session looking in the wrong directory.
  console.log(
    `wasm-stamp: wrote ${relative(repoRoot, STAMP).split(sep).join("/")} (${actual.slice(0, 12)}…)`,
  );
  process.exit(0);
}

let recorded;
try {
  recorded = readFileSync(STAMP, "utf8").trim();
} catch {
  console.error("wasm-stamp: site/public/wasm/.build-stamp is missing.");
  console.error("\nRebuild and commit the bundle:\n  cd site && npm run build:wasm");
  process.exit(1);
}

if (recorded === actual) {
  console.log(`wasm-stamp: OK (bundle built from the current sources, ${actual.slice(0, 12)}…)`);
  process.exit(0);
}

console.error("wasm-stamp: engine or pack sources changed since the wasm bundle was built.\n");
console.error(`  bundle built from : ${recorded.slice(0, 12)}…`);
console.error(`  sources now hash  : ${actual.slice(0, 12)}…`);
console.error("\nRebuild and commit the bundle:\n  cd site && npm run build:wasm");
process.exit(1);
