#!/usr/bin/env node
/**
 * Refuses to build a deployable learn/ without the engine bundle beside it.
 *
 * learn/public/wasm is gitignored and built by exactly one thing: the deploy
 * job in .github/workflows/learn.yml, which runs site/scripts/build-wasm.sh
 * with OUT_DIR pointed here before `vercel build`. Any other route to a
 * deployment — most obviously Vercel's own Git integration, whose build image
 * has no Rust toolchain — produces a site with no /wasm/* at all.
 *
 * That failure was invisible until someone clicked Run. next.config.ts reads
 * the build stamp for cache-busting and falls back to the literal "dev" when
 * it is missing, so a bundle-less build looked completely normal, served 404
 * for the glue, and every in-page exercise died on "Failed to fetch
 * dynamically imported module" — in all 22 chapters at once, while the
 * "Open in playground" links kept working because those are just links to
 * cfdl.dev. A broken engine was indistinguishable from a working one until a
 * learner hit it.
 *
 * So: on Vercel, no bundle is a hard failure. A failed build does not replace
 * the running deployment, which means a toolchain-less build can no longer
 * clobber a good one — it just fails and leaves the working site up.
 *
 * Off Vercel this only warns. `npm run build` locally and the `build` gate job
 * in CI both legitimately run without a bundle: they are checking that the app
 * compiles, not producing something anyone will visit.
 */
import { readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const learnRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const wasmDir = join(learnRoot, "public", "wasm");

// Every Vercel build, preview included — a preview with a dead engine is just
// as misleading as a production one, and the CI deploy job builds the bundle
// for both.
const onVercel = Boolean(process.env.VERCEL || process.env.VERCEL_ENV);
const required = process.env.REQUIRE_WASM === "1" || onVercel;

if (process.env.SKIP_WASM_CHECK === "1") {
  console.log("check-wasm-bundle: SKIP_WASM_CHECK=1 — skipped");
  process.exit(0);
}

const problems = [];

for (const name of ["cfdl_wasm.js", "cfdl_wasm_bg.wasm"]) {
  try {
    const size = statSync(join(wasmDir, name)).size;
    // wasm-pack has been seen to leave a truncated artifact when a build is
    // interrupted; a file that exists but is empty would pass a bare
    // existsSync and still 404-equivalent at runtime.
    if (size === 0) problems.push(`public/wasm/${name} is empty`);
  } catch {
    problems.push(`public/wasm/${name} is missing`);
  }
}

let stamp = null;
try {
  stamp = readFileSync(join(wasmDir, ".build-stamp"), "utf8").trim();
  if (!stamp) problems.push("public/wasm/.build-stamp is empty");
} catch {
  // The stamp is what next.config.ts turns into NEXT_PUBLIC_WASM_BUILD. Without
  // it the app still builds, but every engine URL is cache-busted by the string
  // "dev" and a returning visitor can be pinned to a stale bundle forever.
  problems.push("public/wasm/.build-stamp is missing (engine URLs would be cache-busted as ?v=dev)");
}

if (problems.length === 0) {
  console.log(`check-wasm-bundle: OK (engine bundle present, stamp ${stamp.slice(0, 12)}…)`);
  process.exit(0);
}

const report = required ? console.error : console.warn;
report(`check-wasm-bundle: ${required ? "the engine bundle is not here." : "no engine bundle (in-page exercises will not run)."}`);
for (const problem of problems) report(`  - ${problem}`);

if (!required) {
  report("\n  Fine for a local build or a compile check. To build it:");
  report("    OUT_DIR=\"$PWD/public/wasm\" ../site/scripts/build-wasm.sh");
  process.exit(0);
}

report("\n  This is a Vercel build, so the result would be deployed with a dead");
report("  engine: /wasm/cfdl_wasm.js would 404 and every in-page exercise would");
report("  fail on \"Failed to fetch dynamically imported module\".");
report("");
report("  The bundle is built by the deploy job in .github/workflows/learn.yml,");
report("  which needs a Rust toolchain. If this build is coming from Vercel's Git");
report("  integration, that is the bug — deploys must go through that workflow.");
report("");
report("  Override (only if you know the bundle is served some other way):");
report("    SKIP_WASM_CHECK=1");
process.exit(1);
