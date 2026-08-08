"use client";

import { useCallback, useState } from "react";
import { ChevronDown, Play, RotateCcw } from "lucide-react";
import { EditorPane } from "@/components/playground/EditorPane";
import { ResultsPanel } from "@/components/playground/ResultsPanel";
import { useEngine } from "@/components/playground/useEngine";
import { Button } from "@/components/ds/Button";
import { cn } from "@/lib/cn";
import type { Diagnostic, RunConfig } from "@/lib/playground/protocol";
import type { Results } from "@/lib/playground/results";

/**
 * One exercise, runnable in place: the chapter's editor on the left of the
 * reader's attention, the engine's answer right under it. Composes the same
 * EditorPane / engine / ResultsPanel the site's playground is built from, so
 * behavior (diagnostics, charts, statement views) is identical — only the
 * chrome differs.
 */
export default function ExerciseRunner({
  files: initialFiles,
  root,
  config,
  pack,
  solutionHtml,
}: {
  files: Record<string, string>;
  root: string;
  config?: RunConfig;
  pack?: string;
  solutionHtml: string;
}) {
  const { status, run } = useEngine();
  const [files, setFiles] = useState(initialFiles);
  const [results, setResults] = useState<Results | null>(null);
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([]);
  const [engineError, setEngineError] = useState<string | null>(null);
  const [ran, setRan] = useState(false);
  const [solutionOpen, setSolutionOpen] = useState(false);

  const onRun = useCallback(async () => {
    setRan(true);
    const outcome = await run({ files, root, config, pack: pack || undefined });
    if (outcome.status === "ok") {
      setResults(outcome.results as Results);
      setDiagnostics([]);
      setEngineError(null);
    } else if (outcome.status === "diagnostics") {
      setResults(null);
      setDiagnostics(outcome.diagnostics);
      setEngineError(null);
    } else {
      setResults(null);
      setDiagnostics([]);
      setEngineError(outcome.message);
    }
  }, [run, files, root, config, pack]);

  const onReset = useCallback(() => {
    setFiles(initialFiles);
    setResults(null);
    setDiagnostics([]);
    setEngineError(null);
    setRan(false);
  }, [initialFiles]);

  return (
    <div className="not-prose overflow-hidden rounded-lg border border-default">
      <div className="flex items-center gap-2 border-b border-subtle bg-surface-sunken px-3 py-2">
        <Button size="sm" onClick={onRun} disabled={status === "starting" || status === "running"}>
          <Play className="h-3.5 w-3.5" />
          {status === "starting" ? "Loading engine…" : status === "running" ? "Running…" : "Run"}
        </Button>
        <Button size="sm" variant="ghost" onClick={onReset}>
          <RotateCcw className="h-3.5 w-3.5" />
          Reset
        </Button>
        <button
          type="button"
          onClick={() => setSolutionOpen((open) => !open)}
          className="ml-auto inline-flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium text-secondary transition-colors hover:bg-surface-raised hover:text-primary"
        >
          <ChevronDown
            className={cn("h-3.5 w-3.5 transition-transform", solutionOpen && "rotate-180")}
          />
          {solutionOpen ? "Hide solution" : "Show solution"}
        </button>
      </div>

      <div className="h-72">
        <EditorPane
          files={files}
          activeFile={root}
          diagnostics={diagnostics}
          onChange={(file, value) => setFiles((f) => ({ ...f, [file]: value }))}
          onSelectFile={() => {}}
          onAddFile={() => {}}
          onDeleteFile={() => {}}
          onRun={onRun}
        />
      </div>

      {solutionOpen && (
        <div className="border-t border-subtle">
          <p className="px-3 pt-2 text-xs font-semibold uppercase tracking-wider text-muted">
            One solution — compare after your own attempt
          </p>
          <div
            className="overflow-x-auto p-3 text-sm [&_pre]:m-0 [&_pre]:rounded-md [&_pre]:p-3"
            dangerouslySetInnerHTML={{ __html: solutionHtml }}
          />
        </div>
      )}

      {ran && (
        <div className="h-80 border-t border-subtle">
          <ResultsPanel
            results={results}
            diagnostics={diagnostics}
            engineError={engineError}
            selectedPack={pack || undefined}
          />
        </div>
      )}
    </div>
  );
}
