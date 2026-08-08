#!/usr/bin/env node
/**
 * Packages training/exercises into the downloadable course kit, at build time,
 * so the bundles are fresh by construction — the same reasoning as the wasm
 * bundle: nothing generated is committed, nothing committed can go stale.
 *
 * Two bundles:
 *   cfdl-course-kit.zip  — everything: starters, solutions, expected metrics,
 *                          run configs, prompts. The instructor's copy.
 *   cfdl-exercises.zip   — starters, prompts, and run configs only. The set
 *                          to hand a class before solutions are discussed.
 */
import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, rmSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const learnDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const exercises = join(learnDir, "..", "training", "exercises");
const outDir = join(learnDir, "public");
const stage = join(learnDir, ".course-kit-stage");

if (!existsSync(exercises)) {
  console.error("build-course-kit: training/exercises not found");
  process.exit(1);
}

function zip(name, filter) {
  rmSync(stage, { recursive: true, force: true });
  mkdirSync(stage, { recursive: true });
  cpSync(exercises, join(stage, "exercises"), { recursive: true, filter });
  rmSync(join(outDir, name), { force: true });
  execFileSync("zip", ["-qr", join(outDir, name), "exercises"], { cwd: stage });
  rmSync(stage, { recursive: true, force: true });
  console.log(`build-course-kit: wrote public/${name}`);
}

zip("cfdl-course-kit.zip", () => true);
zip("cfdl-exercises.zip", (src) => {
  const base = src.split("/").pop();
  return base !== "solution.cfdl" && base !== "expected.json";
});
