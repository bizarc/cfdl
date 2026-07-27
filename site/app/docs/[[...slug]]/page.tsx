import type { Metadata } from "next";
import Link from "next/link";
import { SiteHeader } from "@/components/SiteHeader";
import { SiteFooter } from "@/components/SiteFooter";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";

export const metadata: Metadata = {
  title: "Documentation",
  description: "CFDL guides, references, and domain pack cookbooks.",
};

/**
 * Placeholder route. WS-2 migrates the canonical docs into MDX collections
 * rendered here; this catch-all keeps every /docs/* link in the header,
 * footer, and landing page resolvable in the meantime.
 */
export default function DocsPlaceholder() {
  return (
    <>
      <SiteHeader />
      <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center px-4 py-24 text-center sm:px-6">
        <Badge tone="accent" className="mx-auto">
          Migration in progress
        </Badge>
        <h1 className="mt-5 text-3xl font-semibold tracking-tight text-primary">
          Documentation is moving here
        </h1>
        <p className="mt-4 text-base leading-relaxed text-secondary">
          Guides, the language reference, and pack cookbooks are being migrated
          onto this site from the canonical specs in the repository. Until the
          migration lands, they are readable in the repo under{" "}
          <code className="rounded bg-surface-code px-1.5 py-0.5 font-mono text-sm">
            docs/
          </code>
          .
        </p>
        <div className="mt-8 flex justify-center gap-3">
          <Button asChild>
            <Link href="/">Back to home</Link>
          </Button>
          <Button asChild variant="secondary">
            <a href="https://github.com/bizarc/cfdl" target="_blank" rel="noreferrer">
              Browse the repository
            </a>
          </Button>
        </div>
      </main>
      <SiteFooter />
    </>
  );
}
