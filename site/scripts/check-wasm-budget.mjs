// The committed bundle's gzipped size, checked against the same budget
// build-wasm.sh enforces.
//
// Split out because the budget previously lived only in the build script, so it
// fired when someone rebuilt and was silent everywhere else — a breach could sit
// in the repo unnoticed until the next rebuild happened to surface it.
import { gzipSync } from "node:zlib";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const wasm = path.join(here, "..", "public", "wasm", "cfdl_wasm_bg.wasm");
const script = readFileSync(path.join(here, "build-wasm.sh"), "utf8");

const budget = Number(/^BUDGET_KB=(\d+)/m.exec(script)?.[1]);
if (!Number.isFinite(budget)) {
  console.error("check-wasm-budget: could not read BUDGET_KB from build-wasm.sh");
  process.exit(1);
}

const kb = Math.round(gzipSync(readFileSync(wasm)).length / 1024);
if (kb > budget) {
  console.error(`check-wasm-budget: ${kb} KB gzipped exceeds the ${budget} KB budget.`);
  console.error("  Shrink the module, or raise BUDGET_KB deliberately with a note.");
  process.exit(1);
}
console.log(`check-wasm-budget: OK (${kb} KB gzipped, budget ${budget} KB)`);
