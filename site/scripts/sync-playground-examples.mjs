#!/usr/bin/env node
/**
 * Generates the playground's example gallery from real models in the repo.
 *
 * The gallery is data, not prose: every entry is the actual `.cfdl` source
 * (and run config) that ships in examples/ or fixtures/, so an example can
 * never drift from a model the test suite exercises. Regenerate with
 * `npm run sync:examples`; CI diff-checks the output.
 */
import fs from "node:fs";
import path from "node:path";

const checkMode = process.argv.includes("--check");
const siteDir = path.resolve(import.meta.dirname, "..");
const repoRoot = path.resolve(siteDir, "..");
const OUTPUT = path.resolve(siteDir, "content", "playground-examples.json");

/** Curated, ordered. `dir` is relative to the repo root. */
const EXAMPLES = [
  {
    id: "minimal",
    title: "Minimal model",
    description: "The smallest valid model: a timeline, an entity, one stream.",
    category: "Tutorial",
    dir: "examples/language_tutorial/minimal_model",
  },
  {
    id: "first-stream",
    title: "Your first stream",
    description: "Schedules and amount expressions on a single cash-flow stream.",
    category: "Tutorial",
    dir: "examples/language_tutorial/first_stream",
  },
  {
    id: "simple-contract",
    title: "A simple contract",
    description: "Declare business terms and let a pack template expand them.",
    category: "Tutorial",
    dir: "examples/language_tutorial/simple_contract",
    pack: "cre",
  },
  {
    id: "with-pack",
    title: "Using an industry pack",
    description: "A larger pack-enabled model with contracts for revenue and opex.",
    category: "Tutorial",
    dir: "examples/language_tutorial/with_pack",
    pack: "opco",
  },
  {
    id: "multi-file",
    title: "Multi-file model",
    description: "Split by concern: time, structure, and contracts in separate files.",
    category: "Tutorial",
    dir: "examples/language_tutorial/multi_file",
  },
  {
    id: "cre-developer",
    title: "CRE: developer lifecycle",
    description: "Construction, lease-up, ops, and an exit cap valuation.",
    category: "Domain",
    dir: "examples/cre_developer",
    config: "run.base.json",
    pack: "cre",
  },
  {
    id: "opco-basic",
    title: "OpCo: operating model",
    description: "Revenue and opex streams with working capital and an exit multiple.",
    category: "Domain",
    dir: "examples/opco_basic",
    config: "run.base.json",
    pack: "opco",
  },
  {
    id: "cre-multi-file",
    title: "CRE: multi-file deal",
    description: "A realistic deal split across four files.",
    category: "Domain",
    dir: "examples/cre_multi_file",
    pack: "cre",
  },
  {
    id: "stochastic-rollover",
    title: "Stochastic lease rollover",
    description:
      "Per-trial renew-or-roll outcomes — the bimodal shape an expected-value blend hides.",
    category: "Stochastic",
    dir: "fixtures/valid/cre_stochastic_rollover",
    pack: "cre",
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
  const candidates = configName ? [configName] : ["run.json"];
  for (const name of candidates) {
    const abs = path.resolve(repoRoot, dir, name);
    if (fs.existsSync(abs)) return JSON.parse(fs.readFileSync(abs, "utf8"));
  }
  return undefined;
}

const generated = EXAMPLES.map((example) => ({
  id: example.id,
  title: example.title,
  description: example.description,
  category: example.category,
  source: example.dir,
  root: "model.cfdl",
  pack: example.pack,
  config: readConfig(example.dir, example.config),
  files: readModelFiles(example.dir),
}));

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
  console.log(`sync:examples: wrote ${generated.length} examples`);
}
