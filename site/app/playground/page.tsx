import type { Metadata } from "next";
import Link from "next/link";
import { SiteHeader } from "@/components/SiteHeader";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";

export const metadata: Metadata = {
  title: "Playground",
  description:
    "Compile and run CFDL models in your browser — the real compiler and engine, no install.",
};

/**
 * Placeholder shell. The IDE (worker-backed execution, multi-file editor,
 * results workbench) lands in WS-3/WS-4; this keeps the route and its
 * chrome real so navigation and layout can be reviewed now.
 */
export default function PlaygroundPage() {
  return (
    <>
      <SiteHeader />
      <main className="flex flex-1 items-center justify-center px-4 py-24">
        <div className="max-w-lg text-center">
          <Badge tone="accent">Coming in the next increment</Badge>
          <h1 className="mt-5 text-3xl font-semibold tracking-tight text-primary">
            The playground is being rebuilt
          </h1>
          <p className="mt-4 text-base leading-relaxed text-secondary">
            A three-region IDE — examples, a multi-file editor, and a results
            workbench with cash-flow charts and Monte Carlo distributions —
            running the real engine in a web worker.
          </p>
          <Button asChild variant="secondary" className="mt-8">
            <Link href="/docs/getting-started">Read the getting-started guide</Link>
          </Button>
        </div>
      </main>
    </>
  );
}
