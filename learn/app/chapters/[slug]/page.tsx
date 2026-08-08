import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { compileMDX } from "next-mdx-remote/rsc";
import rehypeSlug from "rehype-slug";
import remarkGfm from "remark-gfm";
import rehypeShikiFromHighlighter from "@shikijs/rehype/core";
import type { ShikiTransformer } from "shiki";

import { ChaptersSidebar } from "@/components/ChaptersSidebar";
import { TableOfContents } from "@/components/docs/TableOfContents";
import { mdxComponents } from "@/components/docs/mdx-components";
import { Exercise } from "@/components/exercise/Exercise";
import {
  PARTS,
  chapterNeighbours,
  getAllChapters,
  getChapterBySlug,
} from "@/lib/chapters";
import { getHighlighter } from "@/lib/shiki";
import { extractToc } from "@/lib/toc";

type Params = { slug: string };

/**
 * Same transformer the site's docs pages use: the original source and
 * language ride on the rendered element so a code block can offer copy and
 * open-in-playground without reconstructing text from highlighted spans.
 */
const codeMetadataTransformer: ShikiTransformer = {
  pre(node) {
    node.properties["data-lang"] = this.options.lang;
    node.properties["data-code"] = this.source;
    const meta = this.options.meta?.__raw ?? "";
    const run = meta.match(/(?:^|\s)run=(\{.*\})\s*$/);
    if (run) node.properties["data-run"] = run[1];
  },
};

export function generateStaticParams(): Params[] {
  return getAllChapters().map((c) => ({
    slug: c.slug.replace(/^\/chapters\//, ""),
  }));
}

export async function generateMetadata({
  params,
}: {
  params: Promise<Params>;
}): Promise<Metadata> {
  const chapter = getChapterBySlug((await params).slug);
  if (!chapter) return {};
  return { title: chapter.title, description: chapter.description };
}

export default async function ChapterPage({
  params,
}: {
  params: Promise<Params>;
}) {
  const chapter = getChapterBySlug((await params).slug);
  if (!chapter) notFound();

  const highlighter = await getHighlighter();

  const { content } = await compileMDX({
    source: chapter.body,
    components: { ...mdxComponents, Exercise },
    options: {
      mdxOptions: {
        // Full MDX, unlike the site's markdown corpus: chapters embed the
        // <Exercise/> component. The cost is that prose must stay clear of
        // bare braces and angle brackets outside code spans.
        format: "mdx",
        remarkPlugins: [remarkGfm],
        rehypePlugins: [
          rehypeSlug,
          [
            rehypeShikiFromHighlighter,
            highlighter,
            {
              themes: { light: "github-light", dark: "github-dark-default" },
              defaultColor: false,
              cssVariablePrefix: "--shiki-",
              fallbackLanguage: "text",
              transformers: [codeMetadataTransformer],
            },
          ],
        ],
      },
    },
  });

  const toc = extractToc(chapter.body);
  const { prev, next } = chapterNeighbours(chapter.slug);
  const chapters = getAllChapters().map((c) => ({
    slug: c.slug,
    title: c.title,
    description: c.description,
    part: c.part,
    order: c.order,
    track: c.track,
  }));

  return (
    <div className="mx-auto flex w-full max-w-7xl flex-1 gap-8 px-4 sm:px-6">
      <aside className="hidden w-64 shrink-0 lg:block">
        <div className="sticky top-14 max-h-[calc(100vh-3.5rem)] overflow-y-auto py-8 pr-2">
          <ChaptersSidebar chapters={chapters} parts={PARTS} />
        </div>
      </aside>

      <main className="min-w-0 flex-1 py-8">
        <p className="text-xs font-semibold uppercase tracking-wider text-muted">
          Part {chapter.part} · {PARTS[chapter.part]}
        </p>
        <article className="mt-2">
          <h1 className="text-3xl font-semibold tracking-tight">{chapter.title}</h1>
          {content}
        </article>

        {(prev || next) && (
          <nav
            aria-label="Pagination"
            className="mt-16 grid gap-4 border-t border-subtle pt-6 sm:grid-cols-2"
          >
            {prev ? (
              <Link
                href={prev.slug}
                className="group rounded-lg border border-default p-4 transition-colors hover:border-strong"
              >
                <span className="flex items-center gap-1.5 text-xs text-muted">
                  <ArrowLeft className="h-3 w-3" />
                  Previous
                </span>
                <span className="mt-1 block font-medium text-primary group-hover:text-accent-text">
                  {prev.title}
                </span>
              </Link>
            ) : (
              <span />
            )}
            {next ? (
              <Link
                href={next.slug}
                className="group rounded-lg border border-default p-4 text-right transition-colors hover:border-strong"
              >
                <span className="flex items-center justify-end gap-1.5 text-xs text-muted">
                  Next
                  <ArrowRight className="h-3 w-3" />
                </span>
                <span className="mt-1 block font-medium text-primary group-hover:text-accent-text">
                  {next.title}
                </span>
              </Link>
            ) : null}
          </nav>
        )}
      </main>

      <aside className="hidden w-56 shrink-0 xl:block">
        <div className="sticky top-14 max-h-[calc(100vh-3.5rem)] overflow-y-auto py-8">
          <TableOfContents entries={toc} />
        </div>
      </aside>
    </div>
  );
}
