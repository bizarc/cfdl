#!/usr/bin/env node
/**
 * Bundles training/exercises/<chapter>/<name>/ into content/exercises.json,
 * the lookup the <Exercise/> MDX component reads. Same sync-with-check idiom
 * as sync-shared.mjs: `--check` diffs and fails on drift, and runs in CI, so
 * an exercise edit that skips the re-bundle cannot ship.
 *
 * The exercise dirs are the canonical form — they are what the verification
 * gate compiles and runs against the engine — and this bundle is a build
 * artifact kept in-tree so the app needs no reach outside its own folder.
 */
import fs from "node:fs";
import path from "node:path";

const learnDir = path.resolve(import.meta.dirname, "..");
const exercisesRoot = path.resolve(learnDir, "..", "training", "exercises");
const outPath = path.join(learnDir, "content", "exercises.json");

const bundle = {};

if (fs.existsSync(exercisesRoot)) {
  for (const chapter of fs.readdirSync(exercisesRoot).sort()) {
    const chapterDir = path.join(exercisesRoot, chapter);
    if (!fs.statSync(chapterDir).isDirectory()) continue;
    for (const name of fs.readdirSync(chapterDir).sort()) {
      const dir = path.join(chapterDir, name);
      if (!fs.statSync(dir).isDirectory()) continue;

      const read = (f) => {
        const p = path.join(dir, f);
        return fs.existsSync(p) ? fs.readFileSync(p, "utf8") : null;
      };

      const model = read("model.cfdl");
      const solution = read("solution.cfdl");
      if (!model || !solution) {
        console.error(`sync-exercises: ${chapter}/${name} is missing model.cfdl or solution.cfdl`);
        process.exit(1);
      }

      const runRaw = read("run.json");
      const readme = read("README.md") ?? "";
      // First markdown heading is the exercise title; the rest is the prompt.
      const heading = /^#\s+(.+)$/m.exec(readme);
      const packMatch = /^\s*use\s+pack\s+"([^"]+)"/m.exec(model);

      bundle[`${chapter}/${name}`] = {
        title: heading ? heading[1].trim() : name,
        prompt: readme.replace(/^#\s+.+\n/, "").trim(),
        files: { "model.cfdl": model },
        root: "model.cfdl",
        config: runRaw ? JSON.parse(runRaw) : undefined,
        pack: packMatch ? packMatch[1] : "",
        solution,
      };
    }
  }
}

const next = JSON.stringify(bundle, null, 2) + "\n";
const prev = fs.existsSync(outPath) ? fs.readFileSync(outPath, "utf8") : null;

if (process.argv.includes("--check")) {
  if (prev !== next) {
    console.error("sync-exercises: content/exercises.json is stale (run `npm run sync:exercises` in learn/)");
    process.exit(1);
  }
  console.log(`sync-exercises: OK (${Object.keys(bundle).length} exercises in sync)`);
} else if (prev !== next) {
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, next);
  console.log(`sync-exercises: wrote ${Object.keys(bundle).length} exercises`);
} else {
  console.log(`sync-exercises: unchanged (${Object.keys(bundle).length} exercises)`);
}
