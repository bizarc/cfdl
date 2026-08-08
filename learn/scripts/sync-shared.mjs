#!/usr/bin/env node
/**
 * The design system has one source of truth: site/. This script mirrors the
 * shared surface (tokens, logo, ds components, theme plumbing, shiki setup)
 * into learn/ so both apps compile the same bytes.
 *
 * `--check` diffs instead of copying and exits non-zero on drift. It runs in
 * both the site and learn CI workflows, so an edit to a shared file on either
 * side fails until re-synced from site/. Never hand-edit a mirrored file in
 * learn/ — change it in site/ and run `npm run sync:shared`.
 */
import fs from "node:fs";
import path from "node:path";

const learnDir = path.resolve(import.meta.dirname, "..");
const siteDir = path.resolve(learnDir, "..", "site");

/** site-relative → learn-relative (identical unless stated). */
const MANIFEST = [
  "app/tokens.css",
  "app/globals.css",
  "components/Logo.tsx",
  "components/ThemeToggle.tsx",
  "components/Providers.tsx",
  "components/ds/Badge.tsx",
  "components/ds/Button.tsx",
  "components/ds/Card.tsx",
  "components/ds/CodeBlock.tsx",
  "components/ds/Dialog.tsx",
  "components/ds/Field.tsx",
  "components/ds/Tabs.tsx",
  "lib/cn.ts",
  "lib/shiki.ts",
  "lib/toc.ts",
  "lib/playground/protocol.ts",
  "lib/playground/share.ts",
  "components/docs/mdx-components.tsx",
  "components/docs/CodeActions.tsx",
  "components/docs/TableOfContents.tsx",
  "public/favicon.svg",
];

const check = process.argv.includes("--check");
let drift = 0;

for (const rel of MANIFEST) {
  const src = path.join(siteDir, rel);
  const dst = path.join(learnDir, rel);

  if (!fs.existsSync(src)) {
    console.error(`sync-shared: missing source ${rel} in site/`);
    drift++;
    continue;
  }

  const want = fs.readFileSync(src);
  const have = fs.existsSync(dst) ? fs.readFileSync(dst) : null;
  const same = have !== null && want.equals(have);

  if (check) {
    if (!same) {
      console.error(`sync-shared: drift in ${rel} (run \`npm run sync:shared\` in learn/)`);
      drift++;
    }
  } else if (!same) {
    fs.mkdirSync(path.dirname(dst), { recursive: true });
    fs.writeFileSync(dst, want);
    console.log(`sync-shared: updated ${rel}`);
  }
}

if (check && drift === 0) console.log("sync-shared: OK (learn/ matches site/)");
if (drift > 0) process.exit(1);
