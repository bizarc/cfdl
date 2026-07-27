import GithubSlugger from "github-slugger";
import type { TocEntry } from "@/components/docs/TableOfContents";

/**
 * Extracts h2/h3 headings for the on-this-page nav, using the same slugger
 * rehype-slug uses so anchors and links agree.
 */
export function extractToc(markdown: string): TocEntry[] {
  const slugger = new GithubSlugger();
  const entries: TocEntry[] = [];
  let inFence = false;

  for (const line of markdown.split("\n")) {
    if (line.startsWith("```")) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;

    const match = /^(#{2,3})\s+(.+?)\s*$/.exec(line);
    if (!match) continue;

    const text = match[2]
      .replace(/`([^`]+)`/g, "$1")
      .replace(/\*\*([^*]+)\*\*/g, "$1")
      .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
      .trim();

    entries.push({ id: slugger.slug(text), text, depth: match[1].length });
  }

  return entries;
}
