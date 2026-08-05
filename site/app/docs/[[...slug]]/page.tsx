import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { compileMDX } from "next-mdx-remote/rsc";
import rehypeSlug from "rehype-slug";
import remarkGfm from "remark-gfm";
import rehypeShikiFromHighlighter from "@shikijs/rehype/core";
import type { ShikiTransformer } from "shiki";

import { SiteHeader } from "@/components/SiteHeader";
import { EnginePrefetch } from "@/components/playground/EnginePrefetch";
import { SiteFooter } from "@/components/SiteFooter";
import { DocsSidebar } from "@/components/docs/DocsSidebar";
import { TableOfContents } from "@/components/docs/TableOfContents";
import { SpecificationBanner } from "@/components/docs/SpecificationBanner";
import { mdxComponents } from "@/components/docs/mdx-components";
import { getAllDocs, getDocBySlug } from "@/lib/docs";
import { getHighlighter } from "@/lib/shiki";
import { extractToc } from "@/lib/toc";
import { sectionNeighbours } from "@/content/nav";

type Params = { slug?: string[] };

/**
 * Carries the original source and language onto the rendered element so a
 * code block can offer copy and open-in-playground without reconstructing
 * text from highlighted spans.
 */
const codeMetadataTransformer: ShikiTransformer = {
  pre(node) {
    node.properties["data-lang"] = this.options.lang;
    node.properties["data-code"] = this.source;
  },
};

export function generateStaticParams(): Params[] {
  return getAllDocs().map((doc) => {
    const rest = doc.slug.replace(/^\/docs\/?/, "");
    return { slug: rest ? rest.split("/") : [] };
  });
}

function slugFromParams(params: Params): string {
  return "/docs" + (params.slug?.length ? "/" + params.slug.join("/") : "");
}

export async function generateMetadata({
  params,
}: {
  params: Promise<Params>;
}): Promise<Metadata> {
  const doc = getDocBySlug(slugFromParams(await params));
  if (!doc) return {};
  return { title: doc.title };
}

export default async function DocPage({ params }: { params: Promise<Params> }) {
  const slug = slugFromParams(await params);
  const doc = getDocBySlug(slug);
  if (!doc) notFound();

  const highlighter = await getHighlighter();

  const { content } = await compileMDX({
    source: doc.body,
    components: mdxComponents,
    options: {
      mdxOptions: {
        // The corpus is plain Markdown; treating it as such avoids MDX
        // choking on stray braces and JSX-looking text in code samples.
        format: "md",
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

  const toc = extractToc(doc.body);
  const { prev, next } = sectionNeighbours(doc.slug);

  return (
    <>
      <EnginePrefetch />
      <SiteHeader />

      <div className="mx-auto flex w-full max-w-7xl flex-1 gap-8 px-4 sm:px-6">
        <aside className="hidden w-60 shrink-0 lg:block">
          <div className="sticky top-14 max-h-[calc(100vh-3.5rem)] overflow-y-auto py-8 pr-2">
            <DocsSidebar />
          </div>
        </aside>

        <main className="min-w-0 flex-1 py-8">
          {doc.layer === "specification" && <SpecificationBanner />}
          <article>{content}</article>

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

      <SiteFooter />
    </>
  );
}
