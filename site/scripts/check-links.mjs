#!/usr/bin/env node
/**
 * Internal-link guard for the docs corpus.
 *
 * Next.js does not fail a build on a dead <Link href>, so this stands in for
 * the broken-link check the old site had. Every site-absolute link in
 * content/docs must resolve to a real page slug.
 */
import fs from "node:fs";
import path from "node:path";

const CONTENT = path.resolve(import.meta.dirname, "..", "content", "docs");
const PUBLIC = path.resolve(import.meta.dirname, "..", "public");
/** Routes owned by the app rather than the docs corpus. */
const APP_ROUTES = new Set(["/", "/playground", "/docs"]);
const IGNORED_PREFIXES = ["/schemas"];

/**
 * A link may also point at a static file — the notebook charts, for instance.
 * Those are checked against `public/` rather than the slug set, so a page that
 * references a missing image still fails rather than being waved through.
 */
function isStaticAsset(target) {
  if (!path.extname(target)) return false;
  return fs.existsSync(path.join(PUBLIC, target));
}

function walk(dir, acc = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, acc);
    else if (entry.name.endsWith(".md")) acc.push(full);
  }
  return acc;
}

const files = walk(CONTENT);
const slugs = new Set(APP_ROUTES);

for (const file of files) {
  const raw = fs.readFileSync(file, "utf8");
  const match = /^slug: *"?([^"\n]+)"?/m.exec(raw);
  const rel = path.relative(CONTENT, file).replace(/\.md$/, "").replace(/\/index$/, "");
  const slug = match ? match[1] : `/docs/${rel}`;
  slugs.add(slug.replace(/\/$/, "") || "/docs");
}

let broken = 0;
for (const file of files) {
  // Links inside fenced code are illustrative, not navigation.
  const body = fs.readFileSync(file, "utf8").replace(/```[\s\S]*?```/g, "");
  for (const [, href] of body.matchAll(/\]\((\/[^)#\s]*)/g)) {
    const target = href.replace(/\/$/, "") || "/";
    if (IGNORED_PREFIXES.some((p) => target.startsWith(p))) continue;
    if (isStaticAsset(target)) continue;
    if (!slugs.has(target)) {
      console.error(`broken link  ${path.relative(CONTENT, file)}  ->  ${href}`);
      broken += 1;
    }
  }
}

if (broken > 0) {
  console.error(`\n${broken} broken internal link(s).`);
  process.exit(1);
}
console.log(`check-links: OK (${files.length} pages, ${slugs.size} slugs)`);
