import Link from "next/link";
import {
  ArrowRight,
  Braces,
  GitCompare,
  Layers,
  ShieldCheck,
  Sigma,
  Terminal,
} from "lucide-react";
import { SiteHeader } from "@/components/SiteHeader";
import { SiteFooter } from "@/components/SiteFooter";
import { Button } from "@/components/ds/Button";
import { Badge } from "@/components/ds/Badge";
import { Card, CardBody, CardTitle } from "@/components/ds/Card";
import { HeroDemo } from "@/components/landing/HeroDemo";
import { heroModel } from "@/components/landing/hero-demo-data";
import { highlight } from "@/lib/shiki";

const FEATURES = [
  {
    icon: Sigma,
    title: "One model, two answers",
    body: "Swap a constant for a distribution and the same file returns the point estimate and percentile bands around every metric — seeded and reproducible.",
  },
  {
    icon: ShieldCheck,
    title: "Parity you can check",
    body: "Every domain pack is gated by benchmark suites diffed against independent reference models, held decimal-exact on schedule math.",
  },
  {
    icon: Layers,
    title: "Domain packs",
    body: "Energy, real estate, credit, and operating businesses ship as contract templates with validated terms and industry metrics.",
  },
  {
    icon: GitCompare,
    title: "Diffable and deterministic",
    body: "Models compile to canonical IR. The same inputs always produce the same output, so results belong in code review and CI.",
  },
  {
    icon: Braces,
    title: "Diagnostics, not #REF!",
    body: "A real compiler with spans, error codes, and hover docs. Mistakes fail at compile time, naming the term that caused them.",
  },
  {
    icon: Terminal,
    title: "Runs everywhere",
    body: "The same engine in your terminal, your notebook, your browser, and your service — byte-identical results across all of them.",
  },
];

const SURFACES = [
  {
    title: "Playground",
    body: "Compile and run in the browser. Nothing to install.",
    href: "/playground",
  },
  {
    title: "CLI",
    body: "cfdl compile / run / validate for files, git, and CI.",
    href: "/docs/install/cli",
  },
  {
    title: "Python SDK",
    body: "pandas accessors over results; notebook-ready.",
    href: "/docs/install/python",
  },
  {
    title: "VS Code",
    body: "Diagnostics, hover, and completion via the CFDL LSP.",
    href: "/docs/install/vscode",
  },
];

export default async function HomePage() {
  const codeHtml = await highlight(heroModel, "cfdl");

  return (
    <>
      <SiteHeader />

      <main className="flex-1">
        <section className="relative overflow-hidden border-b border-subtle">
          <div
            aria-hidden="true"
            className="pointer-events-none absolute inset-0"
            style={{ background: "var(--cfdl-hero-glow)" }}
          />
          <div className="relative mx-auto grid max-w-7xl gap-12 px-4 py-16 sm:px-6 lg:grid-cols-[1.05fr_1fr] lg:py-24">
            <div className="flex flex-col justify-center">
              <Badge tone="accent" className="w-fit">
                Source available · pre-1.0
              </Badge>
              <h1 className="mt-5 text-4xl font-semibold leading-[1.1] tracking-tight text-primary sm:text-5xl">
                Cash-flow models that give you the number{" "}
                <span className="text-accent-text">and the distribution</span>{" "}
                around it.
              </h1>
              <p className="mt-5 max-w-xl text-lg leading-relaxed text-secondary">
                CFDL is a deterministic language for modeling cash flows across
                energy, real estate, credit, and operating businesses. Declare
                the deal in text; the engine derives the schedule, the metrics,
                and the uncertainty.
              </p>
              <div className="mt-8 flex flex-wrap items-center gap-3">
                <Button asChild size="lg">
                  <Link href="/playground">
                    Open the playground
                    <ArrowRight className="h-4 w-4" />
                  </Link>
                </Button>
                <Button asChild size="lg" variant="secondary">
                  <Link href="/docs/getting-started">Read the guide</Link>
                </Button>
              </div>
              <p className="mt-5 text-sm text-muted">
                No signup. The compiler and engine run in your browser.
              </p>
            </div>

            <HeroDemo codeHtml={codeHtml} />
          </div>
        </section>

        <section className="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:py-20">
          <h2 className="text-2xl font-semibold tracking-tight text-primary sm:text-3xl">
            Built like a compiler, not a spreadsheet
          </h2>
          <p className="mt-3 max-w-2xl text-base leading-relaxed text-secondary">
            Underwriting models decide real money. They deserve the tooling
            software engineering has had for decades.
          </p>
          <div className="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {FEATURES.map(({ icon: Icon, title, body }) => (
              <Card key={title}>
                <Icon className="h-5 w-5 text-accent-text" aria-hidden="true" />
                <CardTitle className="mt-4">{title}</CardTitle>
                <CardBody>{body}</CardBody>
              </Card>
            ))}
          </div>
        </section>

        <section className="border-y border-subtle bg-surface-sunken">
          <div className="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:py-20">
            <h2 className="text-2xl font-semibold tracking-tight text-primary sm:text-3xl">
              Use it where you already work
            </h2>
            <p className="mt-3 max-w-2xl text-base leading-relaxed text-secondary">
              One engine, four front doors. A model written in the browser runs
              unchanged in CI.
            </p>
            <div className="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              {SURFACES.map((s) => (
                <Link key={s.href} href={s.href} className="group">
                  <Card className="h-full group-hover:border-strong">
                    <CardTitle className="flex items-center justify-between gap-2">
                      {s.title}
                      <ArrowRight className="h-4 w-4 text-muted transition-transform group-hover:translate-x-0.5 group-hover:text-accent-text" />
                    </CardTitle>
                    <CardBody>{s.body}</CardBody>
                  </Card>
                </Link>
              ))}
            </div>
          </div>
        </section>

        <section className="mx-auto max-w-7xl px-4 py-16 sm:px-6 lg:py-20">
          <div className="grid gap-10 lg:grid-cols-[1fr_1.1fr] lg:items-center">
            <div>
              <h2 className="text-2xl font-semibold tracking-tight text-primary sm:text-3xl">
                Parity is proven, not claimed
              </h2>
              <p className="mt-4 text-base leading-relaxed text-secondary">
                Every pack ships with a benchmark suite: the CFDL model is
                diffed period-by-period against an independent reference
                implementation, inside a tolerance each case declares. Schedule
                math is held decimal-exact; only IRR-class iteratives get a bps
                tolerance.
              </p>
              <Button asChild variant="secondary" className="mt-6">
                <Link href="/docs/benchmarks">
                  How benchmarks work
                  <ArrowRight className="h-4 w-4" />
                </Link>
              </Button>
            </div>
            <dl className="grid grid-cols-2 gap-4 sm:grid-cols-4 lg:grid-cols-2">
              {[
                ["4", "domain packs"],
                ["8", "benchmark cases"],
                ["59", "golden fixtures"],
                ["3", "operating systems in CI"],
              ].map(([n, label]) => (
                <div
                  key={label}
                  className="rounded-lg border border-default bg-surface-raised p-5"
                >
                  <dt className="font-mono text-3xl font-semibold tabular-nums text-accent-text">
                    {n}
                  </dt>
                  <dd className="mt-1 text-sm text-secondary">{label}</dd>
                </div>
              ))}
            </dl>
          </div>
        </section>

        <section className="border-t border-subtle bg-surface-sunken">
          <div className="mx-auto max-w-3xl px-4 py-16 text-center sm:px-6 lg:py-20">
            <h2 className="text-2xl font-semibold tracking-tight text-primary sm:text-3xl">
              Write your first model in ten minutes
            </h2>
            <p className="mx-auto mt-3 max-w-xl text-base leading-relaxed text-secondary">
              Start in the browser, then install the CLI when you want the model
              in version control.
            </p>
            <div className="mt-8 flex flex-wrap justify-center gap-3">
              <Button asChild size="lg">
                <Link href="/playground">Open the playground</Link>
              </Button>
              <Button asChild size="lg" variant="secondary">
                <Link href="/docs/install">Install CFDL</Link>
              </Button>
            </div>
          </div>
        </section>
      </main>

      <SiteFooter />
    </>
  );
}
