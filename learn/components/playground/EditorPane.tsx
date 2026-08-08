"use client";

import dynamic from "next/dynamic";
import { useEffect, useRef } from "react";
import type { editor } from "monaco-editor";
import { X } from "lucide-react";
import { cn } from "@/lib/cn";
import { CFDL_MONARCH, CFDL_LANGUAGE_ID } from "@/lib/playground/cfdl-monarch";
import type { Diagnostic } from "@/lib/playground/protocol";

const MonacoEditor = dynamic(() => import("@monaco-editor/react"), {
  ssr: false,
  loading: () => (
    <div className="flex h-full items-center justify-center text-sm text-muted">
      Loading editor…
    </div>
  ),
});

export interface EditorPaneProps {
  files: Record<string, string>;
  activeFile: string;
  diagnostics: Diagnostic[];
  onChange: (file: string, value: string) => void;
  onSelectFile: (file: string) => void;
  onAddFile: () => void;
  onDeleteFile: (file: string) => void;
  onRun: () => void;
  editorRef?: React.RefObject<editor.IStandaloneCodeEditor | null>;
}

export function EditorPane({
  files,
  activeFile,
  diagnostics,
  onChange,
  onSelectFile,
  onAddFile,
  onDeleteFile,
  onRun,
  editorRef,
}: EditorPaneProps) {
  const monacoRef = useRef<typeof import("monaco-editor") | null>(null);
  const localEditorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  // Monaco captures the action callback once at mount, so route it through a
  // ref that an effect keeps current — otherwise ⌘↵ would run a stale model.
  const runRef = useRef(onRun);
  useEffect(() => {
    runRef.current = onRun;
  }, [onRun]);

  const names = Object.keys(files);

  // Surface compile diagnostics as editor markers on the file they belong to.
  useEffect(() => {
    const monaco = monacoRef.current;
    const ed = localEditorRef.current;
    if (!monaco || !ed) return;
    const model = ed.getModel();
    if (!model) return;

    const markers = diagnostics
      .filter((d) => !d.file || d.file === activeFile)
      .filter((d) => d.span)
      .map((d) => ({
        startLineNumber: d.span!.start_line,
        startColumn: d.span!.start_col,
        endLineNumber: d.span!.end_line,
        endColumn: d.span!.end_col + 1,
        message: d.hint ? `${d.message}\n\nHint: ${d.hint}` : d.message,
        code: d.code,
        severity:
          d.severity === "warning"
            ? monaco.MarkerSeverity.Warning
            : d.severity === "info"
              ? monaco.MarkerSeverity.Info
              : monaco.MarkerSeverity.Error,
      }));

    monaco.editor.setModelMarkers(model, "cfdl", markers);
  }, [diagnostics, activeFile]);

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center gap-px overflow-x-auto border-b border-subtle bg-surface-sunken px-2">
        {names.map((name) => (
          <div
            key={name}
            className={cn(
              "group flex items-center gap-1 whitespace-nowrap rounded-t px-2.5 py-1.5 font-mono text-xs transition-colors",
              name === activeFile
                ? "bg-surface-code text-primary"
                : "text-muted hover:text-secondary",
            )}
          >
            <button type="button" onClick={() => onSelectFile(name)}>
              {name}
            </button>
            {names.length > 1 && name !== "model.cfdl" ? (
              <button
                type="button"
                aria-label={`Delete ${name}`}
                onClick={() => onDeleteFile(name)}
                className="opacity-0 transition-opacity group-hover:opacity-100"
              >
                <X className="h-3 w-3" />
              </button>
            ) : null}
          </div>
        ))}
        <button
          type="button"
          onClick={onAddFile}
          className="ml-1 rounded px-2 py-1 text-xs text-muted transition-colors hover:text-secondary"
        >
          + file
        </button>
      </div>

      <div className="min-h-0 flex-1">
        <MonacoEditor
          language={CFDL_LANGUAGE_ID}
          theme="cfdl-dark"
          path={activeFile}
          value={files[activeFile] ?? ""}
          onChange={(value) => onChange(activeFile, value ?? "")}
          beforeMount={(monaco) => {
            monacoRef.current = monaco;
            if (!monaco.languages.getLanguages().some((l: { id: string }) => l.id === CFDL_LANGUAGE_ID)) {
              monaco.languages.register({ id: CFDL_LANGUAGE_ID });
              monaco.languages.setMonarchTokensProvider(CFDL_LANGUAGE_ID, CFDL_MONARCH);
              monaco.languages.setLanguageConfiguration(CFDL_LANGUAGE_ID, {
                comments: { lineComment: "//", blockComment: ["/*", "*/"] },
                brackets: [
                  ["{", "}"],
                  ["[", "]"],
                  ["(", ")"],
                ],
                autoClosingPairs: [
                  { open: "{", close: "}" },
                  { open: "[", close: "]" },
                  { open: "(", close: ")" },
                  { open: '"', close: '"' },
                ],
              });
            }
            // Editor chrome follows the site's design tokens rather than
            // Monaco's stock palette, so the IDE doesn't look pasted in.
            monaco.editor.defineTheme("cfdl-dark", {
              base: "vs-dark",
              inherit: true,
              rules: [],
              // Monaco's theme API takes literal colors, not CSS variables;
              // fully transparent lets the panel's token background show.
              colors: { "editor.background": "#00000000" }, // tokens-allow: Monaco API
            });
          }}
          onMount={(ed) => {
            localEditorRef.current = ed;
            if (editorRef) editorRef.current = ed;
            ed.addAction({
              id: "cfdl.run",
              label: "Run model",
              keybindings: [
                // Cmd/Ctrl+Enter — the universal "run this" chord.
                (monacoRef.current?.KeyMod.CtrlCmd ?? 0) |
                  (monacoRef.current?.KeyCode.Enter ?? 0),
              ],
              run: () => runRef.current(),
            });
          }}
          options={{
            fontSize: 13,
            fontFamily: "var(--font-jetbrains-mono), monospace",
            minimap: { enabled: false },
            scrollBeyondLastLine: false,
            padding: { top: 12, bottom: 12 },
            renderLineHighlight: "none",
            smoothScrolling: true,
            tabSize: 2,
            automaticLayout: true,
            lineNumbersMinChars: 3,
            overviewRulerLanes: 0,
          }}
        />
      </div>
    </div>
  );
}
