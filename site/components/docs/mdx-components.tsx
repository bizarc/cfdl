import Link from "next/link";
import type { ComponentPropsWithoutRef } from "react";
import { CodeActions } from "@/components/docs/CodeActions";
import { cn } from "@/lib/cn";

/**
 * Element overrides for MDX-rendered docs. Everything reads design-system
 * tokens so docs match the rest of the site in both themes.
 */
export const mdxComponents = {
  h1: (p: ComponentPropsWithoutRef<"h1">) => (
    <h1
      {...p}
      className={cn(
        "mt-0 mb-6 scroll-mt-24 text-3xl font-semibold tracking-tight text-primary",
        p.className,
      )}
    />
  ),
  h2: (p: ComponentPropsWithoutRef<"h2">) => (
    <h2
      {...p}
      className={cn(
        "mt-12 mb-4 scroll-mt-24 border-b border-subtle pb-2 text-xl font-semibold tracking-tight text-primary",
        p.className,
      )}
    />
  ),
  h3: (p: ComponentPropsWithoutRef<"h3">) => (
    <h3
      {...p}
      className={cn("mt-8 mb-3 scroll-mt-24 text-base font-semibold text-primary", p.className)}
    />
  ),
  h4: (p: ComponentPropsWithoutRef<"h4">) => (
    <h4 {...p} className={cn("mt-6 mb-2 scroll-mt-24 font-semibold text-primary", p.className)} />
  ),
  p: (p: ComponentPropsWithoutRef<"p">) => (
    <p {...p} className={cn("my-4 leading-relaxed text-secondary", p.className)} />
  ),
  a: ({ href = "", ...p }: ComponentPropsWithoutRef<"a">) => {
    const external = /^https?:\/\//.test(href);
    const className = cn(
      "font-medium text-accent-text underline decoration-accent-text/30 underline-offset-2 transition-colors hover:decoration-accent-text",
      p.className,
    );
    if (external) {
      return <a href={href} target="_blank" rel="noreferrer" {...p} className={className} />;
    }
    return <Link href={href} {...p} className={className} />;
  },
  ul: (p: ComponentPropsWithoutRef<"ul">) => (
    <ul {...p} className={cn("my-4 list-disc space-y-2 pl-6 text-secondary", p.className)} />
  ),
  ol: (p: ComponentPropsWithoutRef<"ol">) => (
    <ol {...p} className={cn("my-4 list-decimal space-y-2 pl-6 text-secondary", p.className)} />
  ),
  li: (p: ComponentPropsWithoutRef<"li">) => (
    <li {...p} className={cn("leading-relaxed [&>ul]:my-2 [&>ol]:my-2", p.className)} />
  ),
  strong: (p: ComponentPropsWithoutRef<"strong">) => (
    <strong {...p} className={cn("font-semibold text-primary", p.className)} />
  ),
  blockquote: (p: ComponentPropsWithoutRef<"blockquote">) => (
    <blockquote
      {...p}
      className={cn(
        "my-6 rounded-r-md border-l-2 border-accent bg-surface-sunken px-4 py-3 text-sm text-secondary [&>p]:my-1",
        p.className,
      )}
    />
  ),
  hr: (p: ComponentPropsWithoutRef<"hr">) => (
    <hr {...p} className={cn("my-10 border-subtle", p.className)} />
  ),
  table: (p: ComponentPropsWithoutRef<"table">) => (
    <div className="my-6 overflow-x-auto rounded-lg border border-default">
      <table {...p} className={cn("w-full border-collapse text-sm", p.className)} />
    </div>
  ),
  th: (p: ComponentPropsWithoutRef<"th">) => (
    <th
      {...p}
      className={cn(
        "border-b border-default bg-surface-sunken px-4 py-2.5 text-left font-semibold text-primary",
        p.className,
      )}
    />
  ),
  td: (p: ComponentPropsWithoutRef<"td">) => (
    <td
      {...p}
      className={cn("border-b border-subtle px-4 py-2.5 align-top text-secondary", p.className)}
    />
  ),
  // Inline code only — fenced blocks arrive pre-highlighted from Shiki and
  // render through `pre`.
  code: (p: ComponentPropsWithoutRef<"code">) => (
    <code
      {...p}
      className={cn(
        "rounded border border-subtle bg-surface-code px-1.5 py-0.5 font-mono text-[0.85em] text-primary",
        "[pre_&]:border-0 [pre_&]:bg-transparent [pre_&]:p-0 [pre_&]:text-inherit",
        p.className,
      )}
    />
  ),
  // The Shiki transformer stashes the original source and language on the
  // element, so the actions work on the real text rather than reconstructing
  // it from highlighted spans.
  pre: ({
    "data-code": code,
    "data-lang": lang,
    ...p
  }: ComponentPropsWithoutRef<"pre"> & {
    "data-code"?: string;
    "data-lang"?: string;
  }) => (
    <div className="group relative">
      <pre
        {...p}
        className={cn(
          "my-6 overflow-x-auto rounded-lg border border-default p-4 pr-24 font-mono text-[13px] leading-relaxed",
          p.className,
        )}
      />
      {code ? <CodeActions code={code} lang={lang} className="top-8" /> : null}
    </div>
  ),
};
