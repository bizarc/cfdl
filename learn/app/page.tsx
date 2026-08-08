import { Badge } from "@/components/ds/Badge";
import { Card } from "@/components/ds/Card";

import Link from "next/link";

type Part = {
  number: string;
  title: string;
  blurb: string;
  chapters: { label: string; href?: string }[];
  status: "in-progress" | "planned";
};

const PARTS: Part[] = [
  {
    number: "Part I",
    title: "Thinking in cash flows",
    blurb:
      "Why a modeling language beats a spreadsheet for auditable deals, the object model — time, entities, streams — and how to read what a run gives you back.",
    chapters: [
      { label: "Why a language?", href: "/chapters/why-a-language" },
      { label: "The object model", href: "/chapters/the-object-model" },
      { label: "Reading results", href: "/chapters/reading-results" },
    ],
    status: "in-progress",
  },
  {
    number: "Part II",
    title: "The core language",
    blurb:
      "Streams, schedules, expressions, assumptions, curves, fields, events, options, waterfalls, and Monte Carlo — the full packless language, one construct at a time, each with runnable exercises.",
    chapters: [
      { label: "Streams and schedules", href: "/chapters/streams-and-schedules" },
      { label: "Expressions", href: "/chapters/expressions" },
      { label: "Assumptions and inputs", href: "/chapters/assumptions-and-inputs" },
      { label: "Curves", href: "/chapters/curves" },
      { label: "Fields and recurrence", href: "/chapters/fields-and-recurrence" },
      { label: "Growth, ramps, and guards", href: "/chapters/growth-ramps-and-guards" },
      { label: "Events and options", href: "/chapters/events-and-options" },
      { label: "Waterfalls", href: "/chapters/waterfalls" },
      { label: "Uncertainty and Monte Carlo", href: "/chapters/uncertainty-and-monte-carlo" },
      { label: "Multi-file models and style", href: "/chapters/multi-file-models-and-style" },
      { label: "Under the hood (deep dive)", href: "/chapters/under-the-hood" },
      { label: "Diagnostics as a discipline (deep dive)", href: "/chapters/diagnostics-as-a-discipline" },
    ],
    status: "in-progress",
  },
  {
    number: "Part III",
    title: "Modeling judgment",
    blurb:
      "The decisions the syntax can't make for you: choosing grain, choosing constructs, the finance semantics the language encodes, and when a domain pack earns its keep.",
    chapters: [
      { label: "Choosing grain", href: "/chapters/choosing-grain" },
      { label: "Stream, contract, field, event, option, or waterfall?", href: "/chapters/choosing-constructs" },
      { label: "Finance semantics", href: "/chapters/finance-semantics" },
      { label: "Packs: when and why", href: "/chapters/packs-when-and-why" },
    ],
    status: "in-progress",
  },
  {
    number: "Part IV",
    title: "The CRE capstone",
    blurb:
      "One development deal carried end to end — skeleton, revenue, costs and financing, exit and returns, then scenarios, Monte Carlo, and an equity waterfall over the whole structure.",
    chapters: [
      { label: "The capstone: Harbor Point", href: "/chapters/the-deal-and-the-skeleton" },
      { label: "Revenue", href: "/chapters/capstone-revenue" },
      { label: "Costs and financing", href: "/chapters/capstone-costs-and-financing" },
      { label: "Exit and returns", href: "/chapters/capstone-exit-and-returns" },
      { label: "Risk and the split", href: "/chapters/capstone-risk" },
    ],
    status: "in-progress",
  },
  {
    number: "Part V",
    title: "Reference",
    blurb:
      "The language on two pages, and every CRE contract type with its required and defaulted terms.",
    chapters: [
      { label: "Appendix A: language quick reference", href: "/chapters/language-quick-reference" },
      { label: "Appendix B: CRE pack term tables", href: "/chapters/cre-pack-term-tables" },
      { label: "Appendix C: instructor notes", href: "/chapters/instructor-notes" },
    ],
    status: "in-progress",
  },
];

export default function Home() {
  return (
    <div className="mx-auto max-w-7xl px-4 sm:px-6">
      <section className="py-20 sm:py-28">
        <Badge tone="accent">A structured course</Badge>
        <h1 className="mt-4 max-w-3xl text-4xl font-semibold tracking-tight sm:text-5xl">
          Learn to author cash-flow models in CFDL
        </h1>
        <p className="mt-5 max-w-2xl text-lg leading-relaxed text-secondary">
          Built for finance professionals and technical modelers alike: a core
          track that teaches the language through the deals it describes, and
          deep-dive chapters for readers who want the machinery underneath.
          Every example runs in your browser against the real engine.
        </p>
        <div className="mt-8 flex flex-wrap items-center gap-3 text-sm text-secondary">
          <span className="inline-flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-accent" aria-hidden />
            24 chapters across 5 parts
          </span>
          <span className="inline-flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-accent" aria-hidden />
            Runnable exercises with solutions
          </span>
          <span className="inline-flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-accent" aria-hidden />
            A real-estate deal carried end to end
          </span>
        </div>
      </section>

      <section id="curriculum" className="scroll-mt-20 pb-12">
        <h2 className="text-2xl font-semibold tracking-tight">Curriculum</h2>
        <p className="mt-2 max-w-2xl text-sm leading-relaxed text-secondary">
          Read the parts in order — each chapter builds only on material already
          introduced. Chapters marked as deep dives can be skipped on the core
          track.
        </p>

        <div className="mt-8 grid gap-6 md:grid-cols-2">
          {PARTS.map((part) => (
            <Card key={part.number} className="p-6">
              <div className="flex items-center justify-between gap-3">
                <p className="text-xs font-semibold uppercase tracking-wider text-muted">
                  {part.number}
                </p>
                <Badge tone={part.status === "in-progress" ? "accent" : "neutral"}>
                  {part.status === "in-progress" ? "In progress" : "Planned"}
                </Badge>
              </div>
              <h3 className="mt-2 text-lg font-semibold">{part.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-secondary">
                {part.blurb}
              </p>
              <ul className="mt-4 space-y-1.5">
                {part.chapters.map((chapter) => (
                  <li
                    key={chapter.label}
                    className="flex items-baseline gap-2 text-sm text-secondary"
                  >
                    <span
                      className="h-1.5 w-1.5 shrink-0 translate-y-[-1px] rounded-full bg-strong"
                      aria-hidden
                    />
                    {chapter.href ? (
                      <Link
                        href={chapter.href}
                        className="transition-colors hover:text-accent-text"
                      >
                        {chapter.label}
                      </Link>
                    ) : (
                      chapter.label
                    )}
                  </li>
                ))}
              </ul>
            </Card>
          ))}
        </div>
      </section>

      <section className="pb-24">
        <h2 className="text-2xl font-semibold tracking-tight">The course kit</h2>
        <p className="mt-2 max-w-2xl text-sm leading-relaxed text-secondary">
          Every exercise in the course as plain files — models, run
          configurations, and prompts — for classroom use or offline work with
          the command-line tools. Rebuilt from the course sources on every
          deploy.
        </p>
        <div className="mt-6 grid gap-6 sm:grid-cols-2">
          <Card className="p-6">
            <h3 className="text-base font-semibold">Exercises only</h3>
            <p className="mt-2 text-sm leading-relaxed text-secondary">
              Starters, prompts, and run configurations — no solutions. The set
              to hand a class.
            </p>
            <a
              href="/cfdl-exercises.zip"
              download
              className="mt-4 inline-flex items-center gap-1.5 rounded-md border border-default px-3 py-1.5 text-sm font-medium text-secondary transition-colors hover:border-strong hover:text-primary"
            >
              Download cfdl-exercises.zip
            </a>
          </Card>
          <Card className="p-6">
            <h3 className="text-base font-semibold">Full kit</h3>
            <p className="mt-2 text-sm leading-relaxed text-secondary">
              Everything, including solutions and their expected metrics — the
              instructor&apos;s copy, and the self-study answer key.
            </p>
            <a
              href="/cfdl-course-kit.zip"
              download
              className="mt-4 inline-flex items-center gap-1.5 rounded-md border border-default px-3 py-1.5 text-sm font-medium text-secondary transition-colors hover:border-strong hover:text-primary"
            >
              Download cfdl-course-kit.zip
            </a>
          </Card>
        </div>
      </section>
    </div>
  );
}
