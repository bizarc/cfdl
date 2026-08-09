import type { Metadata } from "next";
import Link from "next/link";
import { ArrowRight, Download } from "lucide-react";
import { Badge } from "@/components/ds/Badge";
import { Card } from "@/components/ds/Card";
import { getAllChapters } from "@/lib/chapters";
import { getExerciseCount } from "@/lib/exercises";

export const metadata: Metadata = {
  title: "The course kit",
  description:
    "Every exercise in the course as plain files — starters, prompts, run configurations, and solutions — for classroom use or offline work with the command-line tools.",
};

export default function CourseKitPage() {
  const exercises = getExerciseCount();
  const chapters = getAllChapters().length;

  return (
    <div className="mx-auto max-w-4xl px-4 py-12 sm:px-6">
      <Badge tone="accent">For teaching and offline work</Badge>
      <h1 className="mt-4 text-4xl font-semibold tracking-tight">The course kit</h1>
      <p className="mt-5 text-lg leading-relaxed text-secondary">
        Every exercise in the course as plain files. The same models you can run
        in the browser, packaged to hand to a class, mark up in an editor, or
        run from the command line — rebuilt from the course sources on every
        deploy, so a bundle is never out of step with the chapter it belongs to.
      </p>

      <h2 className="mt-12 text-2xl font-semibold tracking-tight">
        Two bundles
      </h2>
      <p className="mt-2 leading-relaxed text-secondary">
        Same {exercises} exercises, drawn from {chapters} chapters. They differ
        in one respect: whether the answers are in the box.
      </p>

      <div className="mt-6 grid gap-6 sm:grid-cols-2">
        <Card className="flex flex-col p-6">
          <h3 className="text-base font-semibold">Exercises only</h3>
          <p className="mt-2 flex-1 text-sm leading-relaxed text-secondary">
            Starters, prompts, and run configurations — no solutions, no
            expected metrics. The set to hand a class before the answers are
            discussed.
          </p>
          <a
            href="/cfdl-exercises.zip"
            download
            className="mt-4 inline-flex items-center gap-1.5 self-start rounded-md bg-accent px-3 py-1.5 text-sm font-medium text-accent-fg shadow-sm transition-colors hover:bg-accent-hover"
          >
            <Download className="h-4 w-4" />
            cfdl-exercises.zip
          </a>
        </Card>

        <Card className="flex flex-col p-6">
          <h3 className="text-base font-semibold">Full kit</h3>
          <p className="mt-2 flex-1 text-sm leading-relaxed text-secondary">
            Everything above plus <code className="font-mono text-[0.85em]">solution.cfdl</code>{" "}
            and the expected metrics for each exercise — the instructor&apos;s
            copy, and the answer key for self-study.
          </p>
          <a
            href="/cfdl-course-kit.zip"
            download
            className="mt-4 inline-flex items-center gap-1.5 self-start rounded-md border border-default px-3 py-1.5 text-sm font-medium text-secondary transition-colors hover:border-strong hover:text-primary"
          >
            <Download className="h-4 w-4" />
            cfdl-course-kit.zip
          </a>
        </Card>
      </div>

      <h2 className="mt-12 text-2xl font-semibold tracking-tight">
        What is inside
      </h2>
      <p className="mt-2 leading-relaxed text-secondary">
        One directory per exercise, named for the chapter it belongs to:
      </p>
      <pre className="mt-4 overflow-x-auto rounded-lg border border-default bg-surface-code p-4 font-mono text-sm text-secondary">
        {`exercises/
  04-streams-and-schedules/
    billing-terms/
      README.md        the prompt, and what to check before running
      model.cfdl       the starter — the exercise is to finish it
      run.json         the run configuration the chapter uses
      solution.cfdl    one solution        (full kit only)
      expected.json    the metrics it produces  (full kit only)`}
      </pre>

      <h2 className="mt-12 text-2xl font-semibold tracking-tight">
        Running one
      </h2>
      <p className="mt-2 leading-relaxed text-secondary">
        Every exercise is an ordinary model directory, so the command-line tools
        take it as-is — compile it, then run the result against the exercise&apos;s
        own configuration:
      </p>
      <pre className="mt-4 overflow-x-auto rounded-lg border border-default bg-surface-code p-4 font-mono text-sm text-secondary">
        {`cfdl compile exercises/04-streams-and-schedules/billing-terms --out /tmp/ir.json
cfdl run /tmp/ir.json --out /tmp/results.json \\
  --config exercises/04-streams-and-schedules/billing-terms/run.json`}
      </pre>
      <p className="mt-4 leading-relaxed text-secondary">
        Nothing is required beyond the CLI —{" "}
        <a
          href="https://cfdl.dev/docs/install"
          target="_blank"
          rel="noreferrer"
          className="text-accent-text hover:underline"
        >
          installing it
        </a>{" "}
        takes a minute. An exercise that uses a domain pack says so in its model
        header, and needs <code className="font-mono text-[0.85em]">--packs</code>{" "}
        pointed at a pack directory. If you would rather not install anything,
        every exercise also runs in the page it belongs to.
      </p>

      <h2 className="mt-12 text-2xl font-semibold tracking-tight">Teaching it</h2>
      <p className="mt-2 leading-relaxed text-secondary">
        The full kit&apos;s expected metrics make marking mechanical: an exercise
        is right when its solution compiles, runs, and reproduces them. What that
        cannot tell you is whether a model <em>says what the deal means</em>,
        which is the course&apos;s real subject — so the instructor notes carry
        rubrics for that, alongside session plans, per-chapter discussion prompts
        and two ways to run the capstone.
      </p>
      <Link
        href="/chapters/instructor-notes"
        className="mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-accent-text hover:underline"
      >
        Read the instructor notes
        <ArrowRight className="h-3.5 w-3.5" />
      </Link>

      <div className="mt-12 border-t border-subtle pt-6">
        <Link
          href="/"
          className="inline-flex items-center gap-1.5 text-sm text-secondary transition-colors hover:text-primary"
        >
          Back to the course
        </Link>
      </div>
    </div>
  );
}
