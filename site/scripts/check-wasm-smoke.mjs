#!/usr/bin/env node
/**
 * Functional smoke test of the COMMITTED wasm bundle.
 *
 * The version and stamp gates prove the bundle was built from the current
 * sources. This proves the thing we actually ship can still run a model — the
 * failure that prompted all of this was a committed bundle that rejected every
 * `schedule every <interval> from …`, which is most non-trivial models and the
 * first thing anyone types into the playground.
 *
 * Loads the same two files the playground worker loads
 * (site/lib/playground/engine.worker.ts), so a break here is a break there.
 *
 * Usage: node scripts/check-wasm-smoke.mjs
 */
import { readFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, join } from "node:path";

const siteDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const glue = join(siteDir, "public", "wasm", "cfdl_wasm.js");
const binary = join(siteDir, "public", "wasm", "cfdl_wasm_bg.wasm");

const model = (calendar, periods, interval, last) => `version 0.1
model "wasm-smoke"
time calendar ${calendar} from 2026-01 for ${periods}
entity asset a
stream x.y on entity asset.a inflow currency USD {
  schedule every ${interval} from 2026-01 to ${last}
  amount = 100
}`;

// Every calendar x its own interval, each schedule ending on the timeline's
// own last period. The stale bundle failed all of these at the parser, before
// bounds were ever considered.
const CASES = [
  ["monthly", 12, "month", "2026-12"],
  ["monthly", 12, "quarter", "2026-10"],
  ["quarterly", 4, "quarter", "2026-10"],
  ["annual", 3, "year", "2028-01"],
  ["daily", 31, "day", "2026-01-31"],
];

const engine = await import(pathToFileURL(glue).href);
await engine.default({ module_or_path: readFileSync(binary) });

let failed = 0;
for (const [calendar, periods, interval, last] of CASES) {
  const label = `${calendar} calendar / every ${interval}`;
  let envelope;
  try {
    envelope = JSON.parse(
      engine.compile_and_run(
        JSON.stringify({ "model.cfdl": model(calendar, periods, interval, last) }),
        "model.cfdl",
      ),
    );
  } catch (error) {
    console.error(`  FAIL  ${label} — engine threw: ${error}`);
    failed++;
    continue;
  }
  if (envelope.ok) {
    console.log(`  ok    ${label}`);
    continue;
  }
  const first = envelope.diagnostics?.[0];
  console.error(
    `  FAIL  ${label} — ${first ? `${first.code}: ${first.message}` : (envelope.error ?? "unknown")}`,
  );
  failed++;
}

// Every example the playground actually offers, run against the shipped
// engine and its shipped run config. These are the first thing a visitor
// clicks, and nothing else executes them: the golden suite compiles fixtures
// without their run configs, and check-doc-examples covers pack READMEs. A
// run-config field rename once left all five tutorial examples erroring in the
// playground with a red JSON error and no gate noticed.
const examplesPath = join(siteDir, "content", "playground-examples.json");
let examples = [];
try {
  examples = JSON.parse(readFileSync(examplesPath, "utf8"));
} catch (error) {
  console.error(`  FAIL  cannot read playground-examples.json — ${error}`);
  failed++;
}

for (const example of examples) {
  const label = `example: ${example.id ?? example.title}${example.pack ? ` [pack ${example.pack}]` : ""}`;
  let envelope;
  try {
    envelope = JSON.parse(
      engine.compile_and_run(
        JSON.stringify(example.files),
        example.root,
        example.config ? JSON.stringify(example.config) : undefined,
        example.pack || undefined,
      ),
    );
  } catch (error) {
    console.error(`  FAIL  ${label} — engine threw: ${error}`);
    failed++;
    continue;
  }
  if (envelope.ok) {
    console.log(`  ok    ${label}`);
    continue;
  }
  const first = envelope.diagnostics?.[0];
  console.error(
    `  FAIL  ${label} — ${first ? `${first.code}: ${first.message}` : (envelope.error ?? "unknown")}`,
  );
  failed++;
}

const total = CASES.length + examples.length;

if (failed > 0) {
  console.error(`\ncheck-wasm-smoke: ${failed} of ${total} cases failed.`);
  console.error("If the engine is fine, the committed bundle is stale — rebuild it:");
  console.error("  cd site && npm run build:wasm");
  process.exit(1);
}

console.log(`check-wasm-smoke: OK (${CASES.length} schedule cases, ${examples.length} playground examples)`);
