import Link from "next/link";
import { ArrowRight } from "lucide-react";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";
import { Card } from "@/components/ds/Card";
import { firstChapter, getAllChapters, partGroups } from "@/lib/chapters";

/**
 * Editorial blurbs per part. The chapter LISTS are derived from the content —
 * a hand-kept copy drifted once already — but what a part is *for* is prose,
 * so it lives here.
 */
const BLURBS: Record<number, string> = {
  1: "Why a modeling language beats a spreadsheet for auditable deals, the object model — time, entities, streams — and how to read what a run gives you back.",
  2: "Streams, schedules, expressions, assumptions, curves, fields, events, options, waterfalls, and Monte Carlo — the full packless language, one construct at a time, each with runnable exercises.",
  3: "The decisions the syntax can't make for you: choosing grain, choosing constructs, the finance semantics the language encodes, and when a domain pack earns its keep.",
  4: "One development deal carried end to end — skeleton, revenue, costs and financing, exit and returns, then scenarios, Monte Carlo, and an equity waterfall over the whole structure.",
  5: "The language on two pages, every CRE contract type with its terms, and notes for teaching the course.",
};

const ROMAN = ["", "I", "II", "III", "IV", "V"];

export default function Home() {
  const parts = partGroups();
  const start = firstChapter();
  const chapterCount = getAllChapters().length;

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

        <div className="mt-8 flex flex-wrap items-center gap-3">
          {start && (
            <Button asChild size="lg">
              <Link href={start.slug}>
                Start the course
                <ArrowRight className="h-4 w-4" />
              </Link>
            </Button>
          )}
          <Button asChild size="lg" variant="secondary">
            <Link href="#curriculum">Browse the curriculum</Link>
          </Button>
        </div>
        {start && (
          <p className="mt-3 text-sm text-muted">
            Begins with{" "}
            <Link href={start.slug} className="text-accent-text underline decoration-accent-text/40 hover:decoration-accent-text">
              {start.title}
            </Link>{" "}
            — no install required.
          </p>
        )}

        <div className="mt-10 flex flex-wrap items-center gap-3 text-sm text-secondary">
          <span className="inline-flex items-center gap-2">
            <span className="h-2 w-2 rounded-full bg-accent" aria-hidden />
            {chapterCount} chapters across {parts.length} parts
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
          {parts.map((part) => {
            const entry = part.chapters[0];
            return (
              <Card key={part.part} className="flex flex-col p-6">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-xs font-semibold uppercase tracking-wider text-muted">
                    Part {ROMAN[part.part] ?? part.part}
                  </p>
                  <Badge tone={entry ? "accent" : "neutral"}>
                    {entry
                      ? `${part.chapters.length} chapter${part.chapters.length === 1 ? "" : "s"}`
                      : "Planned"}
                  </Badge>
                </div>

                <h3 className="mt-2 text-lg font-semibold">
                  {entry ? (
                    <Link
                      href={entry.slug}
                      className="transition-colors hover:text-accent-text"
                    >
                      {part.title}
                    </Link>
                  ) : (
                    part.title
                  )}
                </h3>

                <p className="mt-2 text-sm leading-relaxed text-secondary">
                  {BLURBS[part.part]}
                </p>

                <ul className="mt-4 space-y-1.5">
                  {part.chapters.map((chapter) => (
                    <li
                      key={chapter.slug}
                      className="flex items-baseline gap-2 text-sm text-secondary"
                    >
                      <span
                        className="h-1.5 w-1.5 shrink-0 translate-y-[-1px] rounded-full bg-strong"
                        aria-hidden
                      />
                      <Link
                        href={chapter.slug}
                        className="transition-colors hover:text-accent-text"
                      >
                        {chapter.title}
                      </Link>
                      {chapter.track === "deep" && (
                        <span className="shrink-0 text-xs text-muted">deep dive</span>
                      )}
                    </li>
                  ))}
                </ul>

                {entry && (
                  <Link
                    href={entry.slug}
                    className="mt-4 inline-flex items-center gap-1.5 self-start text-sm font-medium text-accent-text hover:underline"
                  >
                    Start Part {ROMAN[part.part] ?? part.part}
                    <ArrowRight className="h-3.5 w-3.5" />
                  </Link>
                )}
              </Card>
            );
          })}
        </div>
      </section>

      <section id="course-kit" className="scroll-mt-20 pb-24">
        <h2 className="text-2xl font-semibold tracking-tight">The course kit</h2>
        <p className="mt-2 max-w-2xl text-sm leading-relaxed text-secondary">
          Every exercise in the course as plain files — starters, prompts, run
          configurations, and solutions — for classroom use or offline work with
          the command-line tools. Rebuilt from the course sources on every
          deploy.
        </p>

        <div className="mt-6 flex flex-wrap items-center gap-3">
          <Button asChild>
            <Link href="/course-kit">
              What is in the kit
              <ArrowRight className="h-4 w-4" />
            </Link>
          </Button>
          <a
            href="/cfdl-exercises.zip"
            download
            className="inline-flex h-10 items-center gap-1.5 rounded-md border border-default px-4 text-sm font-medium text-secondary transition-colors hover:border-strong hover:text-primary"
          >
            Exercises only
          </a>
          <a
            href="/cfdl-course-kit.zip"
            download
            className="inline-flex h-10 items-center gap-1.5 rounded-md border border-default px-4 text-sm font-medium text-secondary transition-colors hover:border-strong hover:text-primary"
          >
            Full kit, with solutions
          </a>
        </div>
      </section>

    </div>
  );
}
