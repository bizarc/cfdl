#!/usr/bin/env node
/**
 * Generates the playground's example gallery from real models in the repo.
 *
 * The gallery is data, not prose: every entry is the actual `.cfdl` source
 * (and run config) that ships in examples/ or fixtures/, so an example can
 * never drift from a model the test suite exercises. Regenerate with
 * `npm run sync:examples`; CI diff-checks the output.
 *
 * The pack is DERIVED from the model's `use pack "…"` declaration, never
 * hand-specified — hardcoding it once meant the playground asked for CRE
 * domain metrics on a model that doesn't use the CRE pack, which rendered a
 * misleading `domain.cre.noi = 0.00`.
 */
import fs from "node:fs";
import path from "node:path";

const checkMode = process.argv.includes("--check");
const siteDir = path.resolve(import.meta.dirname, "..");
const repoRoot = path.resolve(siteDir, "..");
const OUTPUT = path.resolve(siteDir, "content", "playground-examples.json");

/**
 * Curated and ordered. `dir` is relative to the repo root.
 *
 * `summary` says what the model demonstrates; `tryThis` gives the reader a
 * concrete first edit. Tutorial entries are a numbered progression.
 */
const EXAMPLES = [
  {
    id: "minimal",
    title: "Minimal model",
    category: "Tutorial",
    dir: "examples/language_tutorial/minimal_model",
    summary: "The smallest model that compiles: a timeline, an entity, one stream.",
    tryThis: "Change the amount to 2500 and re-run — watch NPV move.",
    docsHref: "/docs/examples/minimal_model",
  },
  {
    id: "first-stream",
    title: "Your first stream",
    category: "Tutorial",
    dir: "examples/language_tutorial/first_stream",
    summary: "Money in and money out, on their own schedules.",
    tryThis: "Add `on day 1` to the expense schedule and compare the cash-flow chart.",
    docsHref: "/docs/examples/first_stream",
  },
  {
    id: "simple-contract",
    title: "A simple contract",
    category: "Tutorial",
    dir: "examples/language_tutorial/simple_contract",
    summary: "Declare lease terms; the CRE pack expands them into streams for you.",
    tryThis: "Raise base_rent to 30000 — no schedule maths required.",
    docsHref: "/docs/examples/simple_contract",
  },
  {
    id: "with-pack",
    title: "Using an industry pack",
    category: "Tutorial",
    dir: "examples/language_tutorial/with_pack",
    summary: "A fuller pack-driven model with revenue, opex, and domain metrics.",
    tryThis: "Switch the pack selector off and on to see domain metrics appear.",
    docsHref: "/docs/examples/with_pack",
  },
  {
    id: "multi-file",
    title: "Multi-file model",
    category: "Tutorial",
    dir: "examples/language_tutorial/multi_file",
    summary: "Split a growing model into time, structure, and contract files.",
    tryThis: "Open the structure.cfdl tab and add a second entity.",
    docsHref: "/docs/examples/multi_file",
  },
  // The tutorial stopped at five here while the docs taught eight, so curves,
  // Monte Carlo and events were the three lessons a reader could not open in
  // the one place they can run something without installing anything.
  {
    id: "curves",
    title: "Curves",
    category: "Tutorial",
    dir: "examples/language_tutorial/curves",
    summary: "A rate or price path declared once and read by date.",
    tryThis: "Add a point to the curve and watch the amounts between it move.",
    docsHref: "/docs/examples/curves",
  },
  {
    id: "uncertainty",
    title: "Uncertainty and Monte Carlo",
    category: "Tutorial",
    dir: "examples/language_tutorial/uncertainty",
    summary: "Swap a constant for a distribution and get bands around every metric.",
    tryThis: "Open the Monte Carlo tab for the distribution around NPV.",
    docsHref: "/docs/examples/uncertainty",
  },
  {
    id: "options-events",
    title: "Events and options",
    category: "Tutorial",
    dir: "examples/language_tutorial/options_events",
    summary: "A condition that changes an asset's state, and a contract with an election.",
    tryThis: "Move the event's trigger period and watch the streams switch with it.",
    docsHref: "/docs/examples/options_events",
  },
  {
    id: "cre-developer",
    title: "CRE: developer lifecycle",
    category: "Real deals",
    dir: "examples/cre_developer",
    config: "run.base.json",
    summary: "Construction, lease-up, operations, and an exit cap valuation.",
    tryThis: "Adjust the discount rate and watch NPV and IRR respond.",
    docsHref: "/docs/examples/cre_developer",
  },
  {
    id: "opco-basic",
    title: "OpCo: operating model",
    category: "Real deals",
    dir: "examples/opco_basic",
    config: "run.base.json",
    summary: "Revenue and opex with working capital and an exit multiple.",
    tryThis: "Change the exit multiple in the contract terms.",
    docsHref: "/docs/examples/opco_basic",
  },
  {
    id: "cre-multi-file",
    title: "CRE: multi-file deal",
    category: "Real deals",
    dir: "examples/cre_multi_file",
    summary: "A realistic deal organised across four files.",
    tryThis: "Follow a lease from contracts.cfdl through to the cash-flow table.",
    docsHref: "/docs/examples/cre_multi_file",
  },
  {
    id: "stochastic-rollover",
    title: "Stochastic lease rollover",
    category: "Stochastic",
    dir: "fixtures/valid/cre_stochastic_rollover",
    summary:
      "Each trial either renews or re-lets — the two-humped outcome an average hides.",
    tryThis: "Open the Monte Carlo tab: two clusters, not one blended number.",
    docsHref: "/docs/stochastic-modeling",
  },
];

function readModelFiles(dir) {
  const abs = path.resolve(repoRoot, dir);
  const files = {};
  for (const name of fs.readdirSync(abs).sort()) {
    if (name.endsWith(".cfdl")) files[name] = fs.readFileSync(path.join(abs, name), "utf8");
  }
  if (!files["model.cfdl"]) {
    throw new Error(`${dir}: expected a model.cfdl entry point`);
  }
  return files;
}

function readConfig(dir, configName) {
  for (const name of configName ? [configName] : ["run.json"]) {
    const abs = path.resolve(repoRoot, dir, name);
    if (fs.existsSync(abs)) return JSON.parse(fs.readFileSync(abs, "utf8"));
  }
  return undefined;
}

/** The pack the model itself declares — the only source of truth for this. */
export function derivePack(files) {
  for (const source of Object.values(files)) {
    const match = /^\s*use\s+pack\s+"([^"]+)"/m.exec(source);
    if (match) return match[1];
  }
  return undefined;
}

let order = 0;
const generated = EXAMPLES.map((example) => {
  const files = readModelFiles(example.dir);
  return {
    id: example.id,
    title: example.title,
    category: example.category,
    order: (order += 1),
    summary: example.summary,
    tryThis: example.tryThis,
    docsHref: example.docsHref,
    source: example.dir,
    root: "model.cfdl",
    pack: derivePack(files),
    config: readConfig(example.dir, example.config),
    files,
  };
});

const payload = JSON.stringify(generated, null, 2) + "\n";

if (checkMode) {
  const current = fs.existsSync(OUTPUT) ? fs.readFileSync(OUTPUT, "utf8") : "";
  if (current !== payload) {
    console.error(
      "playground examples are stale — run `npm run sync:examples` and commit the result.",
    );
    process.exit(1);
  }
  console.log(`sync:examples --check: OK (${generated.length} examples)`);
} else {
  fs.mkdirSync(path.dirname(OUTPUT), { recursive: true });
  fs.writeFileSync(OUTPUT, payload, "utf8");
  const withPack = generated.filter((e) => e.pack).length;
  console.log(`sync:examples: wrote ${generated.length} examples (${withPack} declare a pack)`);
}
