import fs from "node:fs";
import path from "node:path";

export interface ChapterMeta {
  /** Route path, e.g. `/chapters/why-a-language`. */
  slug: string;
  title: string;
  description: string;
  /** 1-based part number; parts group chapters in the sidebar. */
  part: number;
  /** Global reading order across the whole course. */
  order: number;
  /** "core" is the main track; "deep" chapters are the optional technical dives. */
  track: "core" | "deep";
}

export interface Chapter extends ChapterMeta {
  body: string;
}

/** Part titles, indexed by part number. Also names parts with no chapters yet. */
export const PARTS: Record<number, string> = {
  1: "Thinking in cash flows",
  2: "The core language",
  3: "Modeling judgment",
  4: "The CRE capstone",
  5: "Reference",
};

const CONTENT_ROOT = path.join(process.cwd(), "content", "chapters");

/** Same minimal frontmatter shape the site's docs corpus uses. */
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

let cache: Chapter[] | null = null;

export function getAllChapters(): Chapter[] {
  // Cache only in production builds, so dev edits show up on reload.
  if (cache && process.env.NODE_ENV === "production") return cache;

  if (!fs.existsSync(CONTENT_ROOT)) return (cache = []);

  cache = fs
    .readdirSync(CONTENT_ROOT)
    .filter((name) => name.endsWith(".mdx"))
    .map((name) => {
      const raw = fs.readFileSync(path.join(CONTENT_ROOT, name), "utf8");
      const { data, body } = parseFrontmatter(raw);
      // `01-why-a-language.mdx` → slug `why-a-language`, default order 1.
      const match = /^(\d+)-(.+)\.mdx$/.exec(name);
      return {
        slug: "/chapters/" + (match ? match[2] : name.replace(/\.mdx$/, "")),
        title: data.title || name,
        description: data.description || "",
        part: Number(data.part) || 1,
        order: Number(data.order) || (match ? Number(match[1]) : 0),
        track: data.track === "deep" ? ("deep" as const) : ("core" as const),
        body,
      };
    })
    .sort((a, b) => a.order - b.order);

  return cache;
}

export function getChapterBySlug(slug: string): Chapter | undefined {
  const normalized = "/chapters/" + slug.replace(/^\/|\/$/g, "").replace(/^chapters\//, "");
  return getAllChapters().find((c) => c.slug === normalized);
}

export function chapterNeighbours(slug: string): {
  prev?: ChapterMeta;
  next?: ChapterMeta;
} {
  const all = getAllChapters();
  const idx = all.findIndex((c) => c.slug === slug);
  if (idx === -1) return {};
  return { prev: all[idx - 1], next: all[idx + 1] };
}
