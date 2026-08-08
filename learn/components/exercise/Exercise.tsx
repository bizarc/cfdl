import fs from "node:fs";
import path from "node:path";
import { compileMDX } from "next-mdx-remote/rsc";
import { Badge } from "@/components/ds/Badge";
import { highlight } from "@/lib/shiki";
import type { RunConfig } from "@/lib/playground/protocol";
import { ExerciseRunnerClient } from "./ExerciseLoader";

interface ExerciseEntry {
  title: string;
  prompt: string;
  files: Record<string, string>;
  root: string;
  config?: RunConfig;
  pack: string;
  solution: string;
}

function loadExercises(): Record<string, ExerciseEntry> {
  const p = path.join(process.cwd(), "content", "exercises.json");
  return JSON.parse(fs.readFileSync(p, "utf8"));
}

/**
 * Server half of an embedded exercise: looks the exercise up in the synced
 * bundle, renders its prompt, pre-highlights the solution, and hands the
 * editable parts to the client runner.
 *
 * Referencing an exercise that does not exist fails the build — a chapter
 * cannot ship pointing at a missing exercise.
 */
export async function Exercise({ id }: { id: string }) {
  const entry = loadExercises()[id];
  if (!entry) {
    throw new Error(`Exercise "${id}" is not in content/exercises.json — check the id or re-run sync:exercises`);
  }

  const [solutionHtml, prompt] = await Promise.all([
    highlight(entry.solution, "cfdl"),
    compileMDX({ source: entry.prompt, options: { mdxOptions: { format: "md" } } }),
  ]);

  return (
    <section className="not-prose my-8">
      <div className="mb-3 flex items-center gap-2">
        <Badge tone="accent">Exercise</Badge>
        <h3 className="text-base font-semibold">{entry.title}</h3>
      </div>
      <div className="mb-4 text-sm leading-relaxed text-secondary [&_code]:font-mono [&_code]:text-[0.85em] [&_p]:mt-2 first:[&_p]:mt-0">
        {prompt.content}
      </div>
      <ExerciseRunnerClient
        files={entry.files}
        root={entry.root}
        config={entry.config}
        pack={entry.pack}
        solutionHtml={solutionHtml}
      />
    </section>
  );
}
