"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { editor } from "monaco-editor";
import {
  AlertCircle,
  Check,
  CheckCircle2,
  Link2,
  Loader2,
  PanelLeftClose,
  PanelLeftOpen,
  Play,
  Square,
} from "lucide-react";
import { Button } from "@/components/ds/Button";
import { Dialog } from "@/components/ds/Dialog";
import { Field, Input } from "@/components/ds/Field";
import { Badge } from "@/components/ds/Badge";
import { EditorPane } from "./EditorPane";
import { ResultsPanel } from "./ResultsPanel";
import { EXAMPLES, Sidebar, type PlaygroundExample } from "./Sidebar";
import { useEngine } from "./useEngine";
import type { Diagnostic, RunConfig } from "@/lib/playground/protocol";
import type { Results } from "@/lib/playground/results";
import { clearDraft, readDraft, readShareFromHash, saveDraft, shareUrl } from "@/lib/playground/share";
import { cn } from "@/lib/cn";

// Open on the simplest model that still produces a meaningful NPV and a
// two-series chart — not the most advanced example in the set.
const DEFAULT_EXAMPLE = EXAMPLES.find((e) => e.id === "first-stream") ?? EXAMPLES[0];

const DEFAULT_CONFIG: RunConfig = {
  deterministic: { annual_discount_rate: 0.08 },
  monte_carlo: { trial_count: 500, seed: 42 },
};

export function Playground() {
  const { status, readyMs, run, cancel } = useEngine();

  // Client-only component, so the share hash and local draft can be read
  // during the first render instead of patched in afterwards.
  const [initial] = useState(() => {
    const restored = readShareFromHash() ?? readDraft();
    return restored
      ? {
          files: restored.files,
          root: restored.root,
          config: restored.config ?? DEFAULT_CONFIG,
          pack: restored.pack ?? "",
          exampleId: null as string | null,
        }
      : {
          files: DEFAULT_EXAMPLE.files,
          root: DEFAULT_EXAMPLE.root,
          config: DEFAULT_EXAMPLE.config ?? DEFAULT_CONFIG,
          pack: DEFAULT_EXAMPLE.pack ?? "",
          exampleId: DEFAULT_EXAMPLE.id as string | null,
        };
  });

  const [files, setFiles] = useState<Record<string, string>>(initial.files);
  const [activeFile, setActiveFile] = useState(initial.root);
  const [config, setConfig] = useState<RunConfig>(initial.config);
  const [pack, setPack] = useState(initial.pack);
  const [exampleId, setExampleId] = useState<string | null>(initial.exampleId);

  const [results, setResults] = useState<Results | null>(null);
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([]);
  const [engineError, setEngineError] = useState<string | null>(null);
  const [elapsed, setElapsed] = useState<number | null>(null);
  const [copied, setCopied] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [newFileOpen, setNewFileOpen] = useState(false);
  const [newFileName, setNewFileName] = useState("contracts.cfdl");

  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const autoRan = useRef(false);

  const execute = useCallback(
    async (override?: {
      files?: Record<string, string>;
      root?: string;
      config?: RunConfig;
      pack?: string;
    }) => {
      const started = performance.now();
      const outcome = await run({
        files: override?.files ?? files,
        // model.cfdl is always the entry point; the active tab is just what
        // is being edited.
        root: override?.root ?? "model.cfdl",
        config: override?.config ?? config,
        // Asking for domain metrics from a pack the model doesn't use yields
        // a column of zeros that reads as a broken calculation.
        pack: (override?.pack ?? pack) || undefined,
      });
      setElapsed(Math.round(performance.now() - started));

      if (outcome.status === "ok") {
        setResults(outcome.results as Results);
        setDiagnostics([]);
        setEngineError(null);
      } else if (outcome.status === "diagnostics") {
        setDiagnostics(outcome.diagnostics);
        setResults(null);
        setEngineError(null);
      } else {
        setEngineError(outcome.message);
        setResults(null);
        setDiagnostics([]);
      }
    },
    [run, files, config, pack],
  );

  // Land on results, not an empty pane: run once the engine is warm.
  useEffect(() => {
    if (status !== "ready" || autoRan.current) return;
    autoRan.current = true;
    void execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status]);

  // Autosave, but never persist an untouched example — a draft should mean
  // "work in progress", not "the last thing you clicked".
  useEffect(() => {
    if (exampleId) return;
    const timer = setTimeout(
      () => saveDraft({ files, root: "model.cfdl", config, pack: pack || undefined }),
      1000,
    );
    return () => clearTimeout(timer);
  }, [files, config, pack, exampleId]);

  const pickExample = useCallback(
    (example: PlaygroundExample) => {
      setFiles(example.files);
      setActiveFile(example.root);
      setConfig(example.config ?? DEFAULT_CONFIG);
      setPack(example.pack ?? "");
      setExampleId(example.id);
      setResults(null);
      setDiagnostics([]);
      setEngineError(null);
      clearDraft();
      if (typeof window !== "undefined") {
        window.history.replaceState(null, "", window.location.pathname);
      }
      void execute({
        files: example.files,
        root: example.root,
        config: example.config ?? DEFAULT_CONFIG,
        pack: example.pack ?? "",
      });
    },
    [execute],
  );

  const onShare = useCallback(async () => {
    const url = shareUrl({ files, root: "model.cfdl", config, pack: pack || undefined });

    // Put the link in the address bar first: clipboard access can be denied
    // (permissions, insecure context, no user gesture), and the shareable URL
    // must exist either way — copying is the convenience, not the feature.
    window.history.replaceState(null, "", url.slice(url.indexOf("/playground")));

    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard unavailable — the URL is already shareable from the bar.
    }
  }, [files, config, pack]);

  const jumpTo = useCallback((file: string | undefined, line: number) => {
    if (file && file in files) setActiveFile(file);
    const ed = editorRef.current;
    if (!ed) return;
    ed.revealLineInCenter(line);
    ed.setPosition({ lineNumber: line, column: 1 });
    ed.focus();
  }, [files]);

  // The pack the model itself declares — recomputed as the source is edited,
  // so the selector and the domain-metric explanation stay truthful.
  const declaredPack = useMemo(() => {
    for (const source of Object.values(files)) {
      const match = /^\s*use\s+pack\s+"([^"]+)"/m.exec(source);
      if (match) return match[1];
    }
    return undefined;
  }, [files]);

  const busy = status === "running";

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <header className="flex shrink-0 flex-wrap items-center gap-2 border-b border-subtle px-3 py-2">
        <button
          type="button"
          onClick={() => setSidebarOpen((v) => !v)}
          aria-label={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
          className="rounded-md p-1.5 text-muted transition-colors hover:bg-surface-sunken hover:text-primary"
        >
          {sidebarOpen ? (
            <PanelLeftClose className="h-4 w-4" />
          ) : (
            <PanelLeftOpen className="h-4 w-4" />
          )}
        </button>

        <EngineBadge status={status} readyMs={readyMs} />

        {elapsed !== null && !busy ? (
          <span className="font-mono text-xs text-muted">{elapsed} ms</span>
        ) : null}

        <div className="ml-auto flex items-center gap-2">
          <Button variant="ghost" size="sm" onClick={onShare}>
            {copied ? <Check className="h-3.5 w-3.5" /> : <Link2 className="h-3.5 w-3.5" />}
            {copied ? "Copied" : "Share"}
          </Button>
          {busy ? (
            <Button variant="secondary" size="sm" onClick={cancel}>
              <Square className="h-3.5 w-3.5" />
              Stop
            </Button>
          ) : null}
          <Button size="sm" onClick={() => execute()} disabled={status === "starting" || busy}>
            {busy ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Play className="h-3.5 w-3.5" />
            )}
            Run
            <kbd className="ml-1 hidden font-mono text-[10px] opacity-70 sm:inline">⌘↵</kbd>
          </Button>
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <aside
          className={cn(
            "shrink-0 border-r border-subtle transition-all",
            sidebarOpen ? "w-64" : "w-0 overflow-hidden",
          )}
        >
          <Sidebar
            activeId={exampleId}
            onPick={pickExample}
            config={config}
            onConfigChange={setConfig}
            pack={pack}
            onPackChange={setPack}
            modelDeclaredPack={declaredPack}
          />
        </aside>

        <main className="grid min-h-0 flex-1 grid-rows-2 lg:grid-cols-2 lg:grid-rows-1">
          <section className="min-h-0 border-b border-subtle lg:border-b-0 lg:border-r">
            <EditorPane
              files={files}
              activeFile={activeFile}
              diagnostics={diagnostics}
              editorRef={editorRef}
              onChange={(file, value) => {
                setFiles((prev) => ({ ...prev, [file]: value }));
                setExampleId(null);
              }}
              onSelectFile={setActiveFile}
              onAddFile={() => {
                setNewFileName("contracts.cfdl");
                setNewFileOpen(true);
              }}
              onDeleteFile={(name) => {
                setFiles((prev) => {
                  const next = { ...prev };
                  delete next[name];
                  return next;
                });
                setActiveFile("model.cfdl");
                setExampleId(null);
              }}
              onRun={() => void execute()}
            />
          </section>

          <section className="min-h-0">
            <ResultsPanel
              results={results}
              diagnostics={diagnostics}
              engineError={engineError}
              onJumpTo={jumpTo}
              selectedPack={pack}
              modelDeclaredPack={declaredPack}
            />
          </section>
        </main>
      </div>

      <NewFileDialog
        open={newFileOpen}
        onOpenChange={setNewFileOpen}
        name={newFileName}
        onNameChange={setNewFileName}
        existing={Object.keys(files)}
        onCreate={(name) => {
          setFiles((prev) => ({ ...prev, [name]: "" }));
          setActiveFile(name);
          setExampleId(null);
          setNewFileOpen(false);
        }}
      />
    </div>
  );
}

/**
 * Replaces a former `window.prompt`: validation is visible and specific
 * instead of the prompt path's silent rejection of bad names.
 */
function NewFileDialog({
  open,
  onOpenChange,
  name,
  onNameChange,
  existing,
  onCreate,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  name: string;
  onNameChange: (name: string) => void;
  existing: string[];
  onCreate: (name: string) => void;
}) {
  const trimmed = name.trim();
  const error = !trimmed
    ? "Enter a file name."
    : !trimmed.endsWith(".cfdl")
      ? "File names must end with .cfdl"
      : existing.includes(trimmed)
        ? `${trimmed} already exists.`
        : undefined;

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title="New file"
      description="Imported from model.cfdl with an import statement."
      footer={
        <>
          <Button variant="secondary" size="sm" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button size="sm" disabled={Boolean(error)} onClick={() => onCreate(trimmed)}>
            Create file
          </Button>
        </>
      }
    >
      <Field label="File name" error={name ? error : undefined}>
        <Input
          autoFocus
          value={name}
          invalid={Boolean(name && error)}
          onChange={(e) => onNameChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !error) onCreate(trimmed);
          }}
        />
      </Field>
    </Dialog>
  );
}

function EngineBadge({ status, readyMs }: { status: string; readyMs: number | null }) {
  if (status === "starting") {
    return (
      <Badge>
        <Loader2 className="h-3 w-3 animate-spin" />
        Starting engine
      </Badge>
    );
  }
  if (status === "error") {
    return (
      <Badge tone="err">
        <AlertCircle className="h-3 w-3" />
        Engine failed to load
      </Badge>
    );
  }
  if (status === "running") {
    return (
      <Badge tone="accent">
        <Loader2 className="h-3 w-3 animate-spin" />
        Running
      </Badge>
    );
  }
  return (
    <Badge tone="ok">
      <CheckCircle2 className="h-3 w-3" />
      Engine ready{readyMs !== null ? ` · ${readyMs} ms` : ""}
    </Badge>
  );
}
