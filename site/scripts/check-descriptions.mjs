#!/usr/bin/env node
/**
 * Every published page states what it is.
 *
 * WHY THIS EXISTS. A page's `description` is the sentence a search result and a
 * pasted link show under the title. Until this gate, not one of the 111 pages
 * had one — so every link into the docs rendered with whatever text the crawler
 * guessed, and the guess is usually the first line of a code fence.
 *
 * It is a gate rather than a required field on the type in lib/docs.ts because
 * the two failures are different. A missing description should fail the docs
 * build with the page name; making it non-optional in TypeScript would instead
 * fail an unrelated file with a type error, and the fix would be to write `?`.
 *
 * WHERE A FIX GOES depends on who owns the page, which is what `generated:`
 * records:
 *
 *   generated: none | regions   the page itself
 *   generated: full             the `frontmatter` block in sync-content.mjs,
 *                               or tools/render-notebooks.py for a notebook
 *   source: benchmarks/...      the case's own `summary` in case.toml
 *   no marker                   the example builders in sync-content.mjs
 *
 * Usage: node scripts/check-descriptions.mjs
 */

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const CONTENT = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..", "content", "docs");

// Long enough to say something, short enough to survive a search result. The
// upper bound is deliberately generous: the benchmark pages reuse the case's
// own one-sentence summary, and truncating that to fit would create a second
// wording of a sentence that already exists.
const MIN = 30;
const MAX = 240;

function walk(dir, acc = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.resolve(dir, e.name);
    if (e.isDirectory()) walk(full, acc);
    else if (e.name.endsWith(".md")) acc.push(full);
  }
  return acc;
}

const problems = [];
let checked = 0;

for (const file of walk(CONTENT)) {
  const rel = path.relative(CONTENT, file);
  const raw = fs.readFileSync(file, "utf8");
  if (!raw.startsWith("---")) {
    problems.push(`${rel}: no frontmatter`);
    continue;
  }
  const end = raw.indexOf("\n---", 3);
  const front = raw.slice(4, end === -1 ? undefined : end);
  const line = front.split("\n").find((l) => l.startsWith("description:"));
  checked += 1;

  if (!line) {
    problems.push(`${rel}: no description`);
    continue;
  }
  const value = line.slice("description:".length).trim().replace(/^"|"$/g, "");
  if (value.length < MIN) problems.push(`${rel}: description is ${value.length} chars, minimum ${MIN}`);
  else if (value.length > MAX) problems.push(`${rel}: description is ${value.length} chars, maximum ${MAX}`);
  // A description is prose shown as plain text. Markdown in it renders as
  // literal asterisks and backticks in a search result.
  else if (/[*`]|\[.*\]\(/.test(value)) problems.push(`${rel}: description contains markdown`);
}

if (problems.length > 0) {
  console.error("check-descriptions: pages that do not say what they are.\n");
  for (const p of problems) console.error(`  ${p}`);
  console.error(
    "\nA description is one sentence, shown under the title in a search result\n" +
      "and a pasted link. Where the fix goes depends on who owns the page — see\n" +
      "the header of this script.",
  );
  process.exit(1);
}

console.log(`check-descriptions: OK (${checked} pages state what they are)`);
