#!/usr/bin/env node
/**
 * Staleness guard for the committed wasm bundle.
 *
 * Vercel has no Rust toolchain, so site/public/wasm is committed. The risk is
 * an engine change landing without a rebuild, leaving the site running an old
 * compiler. Byte-comparing two wasm builds is not a reliable check (output
 * varies with toolchain version), so this compares *what changed* instead: if
 * a commit range touches engine sources but not the bundle, fail.
 *
 * Usage: node scripts/check-wasm-fresh.mjs <base-ref>   (default: origin/main)
 */
import { execSync } from "node:child_process";

const base = process.argv[2] || "origin/main";

/** Crates whose code ends up inside the wasm bundle. */
const ENGINE_PATHS = [
  "crates/cfdl-wasm/",
  "crates/cfdl-compile/",
  "crates/cfdl-engine/",
  "crates/cfdl-metrics/",
  "crates/cfdl-pack/",
  "crates/cfdl-calc/",
  "crates/cfdl-parser/",
  "crates/cfdl-lexer/",
  "crates/cfdl-resolver/",
  "crates/cfdl-validate/",
  "packs/",
];
const BUNDLE_PATH = "site/public/wasm/";

let changed;
try {
  changed = execSync(`git diff --name-only ${base}...HEAD`, { encoding: "utf8" })
    .split("\n")
    .filter(Boolean);
} catch {
  console.log(`check-wasm-fresh: cannot diff against ${base} — skipping.`);
  process.exit(0);
}

const engineChanges = changed.filter((f) => ENGINE_PATHS.some((p) => f.startsWith(p)));
const bundleChanged = changed.some((f) => f.startsWith(BUNDLE_PATH));

if (engineChanges.length > 0 && !bundleChanged) {
  console.error("check-wasm-fresh: engine sources changed but the wasm bundle was not rebuilt.\n");
  for (const f of engineChanges.slice(0, 10)) console.error(`  ${f}`);
  if (engineChanges.length > 10) console.error(`  … and ${engineChanges.length - 10} more`);
  console.error("\nRebuild and commit the bundle:\n  cd site && npm run build:wasm");
  process.exit(1);
}

console.log(
  engineChanges.length === 0
    ? "check-wasm-fresh: OK (no engine changes in this range)"
    : "check-wasm-fresh: OK (engine changed and the bundle was rebuilt)",
);
