#!/usr/bin/env node
/**
 * Native browser dialogs (alert/confirm/prompt) can't be styled, block the
 * main thread, and are suppressed in some embedding contexts. Use the design
 * system's <Dialog> instead. This guard exists because a window.prompt
 * reached production once.
 *
 * A `dialogs-allow: <reason>` comment on the match or the two lines above it
 * exempts a line — for prose that merely names the APIs.
 */
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const DIRS = ["app", "components", "lib"];
const CALL = /\b(?:window\.)?(?:alert|confirm|prompt)\s*\(/;

function walk(dir, acc = []) {
  if (!fs.existsSync(dir)) return acc;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, acc);
    else if (/\.tsx?$/.test(entry.name)) acc.push(full);
  }
  return acc;
}

let violations = 0;
for (const dir of DIRS) {
  for (const file of walk(path.join(root, dir))) {
    const lines = fs.readFileSync(file, "utf8").split("\n");
    lines.forEach((line, i) => {
      if (!CALL.test(line)) return;
      const window = lines.slice(Math.max(0, i - 2), i + 1).join("\n");
      if (window.includes("dialogs-allow")) return;
      console.error(`${path.relative(root, file)}:${i + 1}  ${line.trim()}`);
      violations += 1;
    });
  }
}

if (violations > 0) {
  console.error(`\n${violations} native browser dialog(s).`);
  console.error("Use <Dialog> from components/ds/Dialog.tsx, or add a");
  console.error("`dialogs-allow: <reason>` comment if this is only prose.");
  process.exit(1);
}
console.log("check-no-native-dialogs: OK");
