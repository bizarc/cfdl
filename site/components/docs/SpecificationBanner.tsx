import Link from "next/link";
import { FileText } from "lucide-react";

/**
 * Marks a page as normative rather than instructional.
 *
 * These pages are precise and unwelcoming by design — they exist so a second
 * implementation could be written from them. Someone who arrived from a search
 * result while trying to build a model needs to be told, once, that there is a
 * page pitched at their question, and where it is.
 *
 * Driven by `layer: specification` in frontmatter, which `sync-content.mjs`
 * emits for every page it puts under `specification/`. That matters: a new
 * specification page cannot be added without being labelled as one, because the
 * label is not a per-page decision anybody can forget.
 */
export function SpecificationBanner() {
  return (
    <div className="mb-8 flex items-start gap-3 rounded-lg border border-default bg-surface-sunken p-4">
      <FileText className="mt-0.5 h-4 w-4 shrink-0 text-muted" aria-hidden />
      <div className="min-w-0 text-sm leading-relaxed text-secondary">
        <p className="font-medium text-primary">This is a specification page.</p>
        <p className="mt-1">
          It defines CFDL normatively — complete, exact, and written for people
          implementing against it. If you are building a model,{" "}
          <Link href="/docs/reference" className="text-accent-text underline underline-offset-2">
            Reference
          </Link>{" "}
          covers the same ground at the altitude of the work.
        </p>
      </div>
    </div>
  );
}
