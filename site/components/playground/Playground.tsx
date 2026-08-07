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

const SPLIT_KEY = "cfdl.playground.split";

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
  const [newFileOpen, setNewFileOpen] = useState(false);
  const [newFileName, setNewFileName] = useState("contracts.cfdl");

  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const mainRef = useRef<HTMLElement | null>(null);
  const autoRan = useRef(false);

  // Editor/results split, as a percentage given to the editor. A statement or
  // a JSON document wants far more than half the window; a reader who is
  // writing wants the opposite. Persisted, because it is a workspace
  // preference and not a per-run one.
  const wide = useMediaQuery("(min-width: 1024px)");

  // The drawer starts closed and the column starts open, so crossing the
  // breakpoint lands on the right default for that layout. Adjusted during
  // render rather than in an effect: an effect would paint one frame of the
  // wrong state, and the sidebar is exactly the thing that would be seen
  // sliding in and out on every resize.
  const [sidebarOpen, setSidebarOpen] = useState(wide);
  const [lastWide, setLastWide] = useState(wide);
  if (lastWide !== wide) {
    setLastWide(wide);
    setSidebarOpen(wide);
  }
  const [split, setSplit] = useState<number>(() => {
    if (typeof window === "undefined") return 50;
    const stored = Number(window.localStorage.getItem(SPLIT_KEY));
    return Number.isFinite(stored) && stored >= 20 && stored <= 80 ? stored : 50;
  });
  useEffect(() => {
    window.localStorage.setItem(SPLIT_KEY, String(Math.round(split)));
  }, [split]);

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
    // Height-locked rather than `flex-1`. The body is `min-h-full`, so a flex
    // child sizes to its content and the whole document scrolls once the panes
    // are taller than the viewport — which took the tab bar and the Run button
    // off screen at 1280x720. An IDE's chrome does not scroll away; the panes
    // inside it scroll instead. 3.5rem is the sticky SiteHeader, and the 1px is
    // its bottom border — without it the page is exactly one pixel too tall and
    // grows a scrollbar that scrolls nothing.
    <div className="flex h-[calc(100dvh-3.5rem-1px)] min-h-0 flex-col overflow-hidden">
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

      <div className="relative flex min-h-0 flex-1">
        {/* Below lg the sidebar is a drawer OVER the panes, not a column
            beside them. As a 16rem column it left a 375px viewport with
            119px for the editor and the results panel entirely off-screen —
            reachable only by scrolling the document sideways, which the
            height-locked shell no longer allows. */}
        <aside
          className={cn(
            "shrink-0 border-r border-subtle bg-surface-page",
            wide
              ? cn(
                  // translate-x-0 explicitly rather than relying on the drawer's
                  // utility being absent: this branch is what the sidebar
                  // returns to when the window grows past lg, and it should
                  // state its own offset instead of inheriting whatever the
                  // other branch left behind.
                  "translate-x-0 transition-all",
                  sidebarOpen ? "w-64" : "w-0 overflow-hidden",
                )
              : cn(
                  "absolute inset-y-0 left-0 z-40 w-64 shadow-lg transition-transform",
                  sidebarOpen ? "translate-x-0" : "-translate-x-full",
                ),
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

        {/* Tapping outside a drawer closes it; a column has no backdrop. */}
        {!wide && sidebarOpen ? (
          <button
            type="button"
            aria-label="Close sidebar"
            onClick={() => setSidebarOpen(false)}
            className="absolute inset-0 z-30 bg-surface-inverse/40"
          />
        ) : null}

        <main
          ref={mainRef}
          className={cn(
            // min-w-0: a flex child's automatic minimum is its content, and
            // Monaco plus a results table made that 644px inside a 119px slot.
            //
            // grid-cols-1 is minmax(0, 1fr) and is load-bearing in the stacked
            // layout: without a declared column the implicit one is `auto`, so
            // the rows sized to their content and overflowed the pane. That is
            // what clipped the right half of the Monte Carlo histogram — a
            // bimodal run drew its second cluster off-screen.
            "grid min-h-0 w-full min-w-0 flex-1 grid-cols-1 grid-rows-2 lg:grid-rows-1",
          )}
          // minmax(0, …) rather than a bare `fr`: an `fr` track has an `auto`
          // minimum, so a wide statement table pushes the results column past
          // its share and squeezes the editor to a sliver. Zero minimum makes
          // the split mean what it says and the table scroll instead.
          style={
            wide
              ? {
                  gridTemplateColumns: `minmax(0, ${split}fr) 5px minmax(0, ${100 - split}fr)`,
                }
              : undefined
          }
        >
          <section className="min-h-0 min-w-0 border-b border-subtle lg:border-b-0">
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

          {wide ? <Splitter onDrag={setSplit} onReset={() => setSplit(50)} mainRef={mainRef} /> : null}

          <section className="min-h-0 min-w-0">
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

/** Matches a CSS media query, without assuming one on the server. */
function useMediaQuery(query: string): boolean {
  // Read synchronously on the first render — this tree is client-only
  // (`ssr: false`), so there is no server value to match, and starting at
  // `false` would paint one frame of the mobile layout on a desktop.
  const [matches, setMatches] = useState(
    () => typeof window !== "undefined" && window.matchMedia(query).matches,
  );
  useEffect(() => {
    const mql = window.matchMedia(query);
    const update = () => setMatches(mql.matches);
    update();
    // Both signals, deliberately. `change` is the right event and is enough in
    // a real browser; a viewport resized by devtools or an automation harness
    // can update `mql.matches` without emitting it, and the layout then keeps
    // rendering a splitter for a two-column grid that no longer exists.
    mql.addEventListener("change", update);
    window.addEventListener("resize", update);
    return () => {
      mql.removeEventListener("change", update);
      window.removeEventListener("resize", update);
    };
  }, [query]);
  return matches;
}

/**
 * Drag handle between the editor and the results.
 *
 * Pointer capture rather than window listeners: the pointer routinely leaves
 * the 5px handle mid-drag, and over Monaco — which would otherwise swallow the
 * move events and drop the drag.
 */
function Splitter({
  onDrag,
  onReset,
  mainRef,
}: {
  onDrag: (pct: number) => void;
  onReset: () => void;
  mainRef: React.RefObject<HTMLElement | null>;
}) {
  const [dragging, setDragging] = useState(false);

  const move = (clientX: number) => {
    const rect = mainRef.current?.getBoundingClientRect();
    if (!rect || rect.width === 0) return;
    const pct = ((clientX - rect.left) / rect.width) * 100;
    onDrag(Math.min(80, Math.max(20, pct)));
  };

  return (
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize editor and results"
      tabIndex={0}
      onPointerDown={(e) => {
        // Capture is an optimisation, not the mechanism — a pointer id the
        // browser doesn't recognise throws, and the drag should still start.
        try {
          e.currentTarget.setPointerCapture(e.pointerId);
        } catch {
          /* no capture; the drag still tracks while the button is down */
        }
        setDragging(true);
      }}
      // Gated on the button being held, not on the `dragging` state: a fast
      // drag can deliver pointermove before React has re-rendered with
      // `dragging === true`, and those first moves were being dropped.
      // `dragging` is for the highlight only.
      onPointerMove={(e) => {
        if (e.buttons & 1) move(e.clientX);
      }}
      onPointerUp={(e) => {
        try {
          e.currentTarget.releasePointerCapture(e.pointerId);
        } catch {
          /* never captured */
        }
        setDragging(false);
      }}
      onDoubleClick={onReset}
      onKeyDown={(e) => {
        const rect = mainRef.current?.getBoundingClientRect();
        if (!rect) return;
        const step = (rect.width * 0.02);
        if (e.key === "ArrowLeft") move(rect.left + rect.width * (splitFrom(mainRef) / 100) - step);
        else if (e.key === "ArrowRight")
          move(rect.left + rect.width * (splitFrom(mainRef) / 100) + step);
        else return;
        e.preventDefault();
      }}
      className={cn(
        "group relative hidden cursor-col-resize touch-none border-x border-subtle bg-surface-sunken lg:block",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-ring",
        dragging && "bg-accent",
      )}
    >
      <span className="absolute inset-y-0 -left-1 -right-1 group-hover:bg-accent/30" />
    </div>
  );
}

/** Current split read back off the DOM, so arrow keys nudge from where it is. */
function splitFrom(mainRef: React.RefObject<HTMLElement | null>): number {
  const first = mainRef.current?.firstElementChild as HTMLElement | undefined;
  const rect = mainRef.current?.getBoundingClientRect();
  if (!first || !rect || rect.width === 0) return 50;
  return (first.getBoundingClientRect().width / rect.width) * 100;
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
