import fs from "node:fs";
import path from "node:path";

export interface DocMeta {
  /** Route path, e.g. `/docs/language-guide`. */
  slug: string;
  title: string;
  /** Path on disk, relative to content/docs. */
  file: string;
}

export interface Doc extends DocMeta {
  body: string;
  /**
   * Which layer of the documentation a page belongs to.
   *
   * `specification` marks the normative pages, which render a banner pointing
   * at their Reference counterpart. Emitted by sync-content.mjs rather than
   * written per page, so it cannot be forgotten on a new one.
   */
  layer?: string;
}

const CONTENT_ROOT = path.join(process.cwd(), "content", "docs");

/** Minimal frontmatter reader — the corpus is generated, so the shape is fixed. */
function parseFrontmatter(raw: string): { data: Record<string, string>; body: string } {
  if (!raw.startsWith("---")) return { data: {}, body: raw };
  const end = raw.indexOf("\n---", 3);
  if (end === -1) return { data: {}, body: raw };

  const data: Record<string, string> = {};
  for (const line of raw.slice(4, end).split("\n")) {
    const idx = line.indexOf(":");
    if (idx === -1) continue;
    const key = line.slice(0, idx).trim();
    let value = line.slice(idx + 1).trim();
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }
    data[key] = value;
  }
  return { data, body: raw.slice(end + 4).replace(/^\n+/, "") };
}

function walk(dir: string, acc: string[] = []): string[] {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, acc);
    else if (entry.name.endsWith(".md") || entry.name.endsWith(".mdx")) acc.push(full);
  }
  return acc;
}

let cache: Doc[] | null = null;

export function getAllDocs(): Doc[] {
  // Cache only in production builds — in dev the cache would pin the first
  // read for the life of the server, so content edits wouldn't show up.
  if (cache && process.env.NODE_ENV === "production") return cache;

  cache = walk(CONTENT_ROOT).map((full) => {
    const rel = path.relative(CONTENT_ROOT, full);
    const raw = fs.readFileSync(full, "utf8");
    const { data, body } = parseFrontmatter(raw);

    // Frontmatter slug is authoritative; fall back to the file path so a page
    // can never become unreachable just because frontmatter is missing.
    const fallback = "/docs/" + rel.replace(/\.mdx?$/, "").replace(/\/index$/, "");
    return {
      slug: (data.slug || fallback).replace(/\/$/, "") || "/docs",
      title: data.title || rel,
      file: rel,
      body,
      layer: data.layer,
    };
  });

  return cache;
}

export function getDocBySlug(slug: string): Doc | undefined {
  const normalized = ("/" + slug.replace(/^\/|\/$/g, "")).replace(/^\/docs$/, "/docs");
  return getAllDocs().find((d) => d.slug === normalized);
}

export function getAllDocSlugs(): string[] {
  return getAllDocs().map((d) => d.slug);
}
