#!/usr/bin/env node
/**
 * Version guard for the committed wasm bundle.
 *
 * `crates/cfdl-engine` stamps its own version into every build via
 * `env!("CARGO_PKG_VERSION")`, and it inherits `version.workspace = true`. So a
 * correctly-built bundle contains the literal `cfdl-engine<workspace version>`,
 * and a bundle built before the last release bump does not.
 *
 * This is the companion to check-wasm-fresh.mjs, and it deliberately works a
 * different way. That one compares a *commit range*, so it needs git history
 * and it needs its workflow to have been triggered — both of which failed
 * silently and let a five-day-old engine ship. This one reads two files and
 * needs neither, so it cannot be skipped into passing.
 *
 * What it does NOT catch: a source change that ships without a version bump.
 * That is what the .build-stamp covers. The three gates are layered on purpose.
 *
 * Usage: node scripts/check-wasm-version.mjs
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const manifest = join(repoRoot, "Cargo.toml");
const bundle = join(repoRoot, "site", "public", "wasm", "cfdl_wasm_bg.wasm");

// `[workspace.package]` … `version = "x.y.z"` — the first version key after the
// section header. Matching the section first avoids picking up a dependency
// pin from elsewhere in the file.
const cargo = readFileSync(manifest, "utf8");
const section = cargo.split(/^\[workspace\.package\]$/m)[1];
const expected = section?.match(/^\s*version\s*=\s*"([^"]+)"/m)?.[1];

if (!expected) {
  console.error("check-wasm-version: no version found under [workspace.package] in Cargo.toml.");
  process.exit(1);
}

let bytes;
try {
  bytes = readFileSync(bundle);
} catch {
  console.error(`check-wasm-version: ${bundle} is missing.`);
  console.error("\nBuild and commit the bundle:\n  cd site && npm run build:wasm");
  process.exit(1);
}

// wasm-bindgen keeps Rust string literals in the data section, so the crate
// name and its version land adjacent and searchable.
const haystack = bytes.toString("latin1");
const needle = `cfdl-engine${expected}`;

if (haystack.includes(needle)) {
  console.log(`check-wasm-version: OK (bundle reports cfdl-engine ${expected})`);
  process.exit(0);
}

const found = [...new Set(haystack.match(/cfdl-engine\d+\.\d+\.\d+/g) ?? [])];
console.error(
  `check-wasm-version: the committed wasm bundle was built from a different engine version.\n`,
);
console.error(`  workspace version : ${expected}`);
console.error(`  bundle reports    : ${found.length ? found.join(", ") : "no version literal found"}`);
console.error("\nRebuild and commit the bundle:\n  cd site && npm run build:wasm");
process.exit(1);
