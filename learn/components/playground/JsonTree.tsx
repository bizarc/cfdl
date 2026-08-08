"use client";

import { useState } from "react";
import { ChevronRight } from "lucide-react";
import { cn } from "@/lib/cn";

/**
 * Collapsible view over the Results document.
 *
 * The whole thing used to be one `JSON.stringify(results, null, 2)` in a
 * `<pre>`. That is fine for a ten-line model and useless for a real one: a
 * 500-trial run puts `monte_carlo.trial_summaries` — half a megabyte of
 * objects nobody scrolled to on purpose — between the reader and the section
 * they wanted. Collapsed by default past the top level, the same document is a
 * table of contents.
 *
 * The raw text is still one click away; this replaces the default view, not
 * the ability to read the bytes.
 */
export function JsonTree({
  value,
  name,
  depth = 0,
  defaultOpenDepth = 1,
}: {
  value: unknown;
  name?: string;
  depth?: number;
  defaultOpenDepth?: number;
}) {
  const [open, setOpen] = useState(depth < defaultOpenDepth);
  // Long arrays are the reason this component exists; rendering 2,000 trial
  // nodes the moment someone expands one would freeze the tab.
  const [limit, setLimit] = useState(50);

  const isArray = Array.isArray(value);
  const isObject = value !== null && typeof value === "object";

  if (!isObject) {
    return (
      <div className="flex gap-2 py-px pl-[1.125rem] font-mono text-[11px] leading-relaxed">
        {name !== undefined && <span className="text-secondary">{name}:</span>}
        <Scalar value={value} />
      </div>
    );
  }

  const entries = isArray
    ? (value as unknown[]).map((v, i) => [String(i), v] as const)
    : Object.entries(value as Record<string, unknown>);

  const summary = isArray ? `[${entries.length}]` : `{${entries.length}}`;
  const shown = entries.slice(0, limit);

  return (
    <div className={cn(depth > 0 && "border-l border-subtle pl-3")}>
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
        className="flex w-full items-center gap-1 rounded py-px text-left font-mono text-[11px] leading-relaxed hover:bg-surface-sunken"
      >
        <ChevronRight
          className={cn("size-3 shrink-0 text-muted transition-transform", open && "rotate-90")}
        />
        {name !== undefined && <span className="text-secondary">{name}</span>}
        <span className="text-muted">{summary}</span>
      </button>

      {open && (
        <div>
          {shown.map(([k, v]) => (
            <JsonTree
              key={k}
              name={k}
              value={v}
              depth={depth + 1}
              defaultOpenDepth={defaultOpenDepth}
            />
          ))}
          {entries.length > shown.length && (
            <button
              type="button"
              onClick={() => setLimit((n) => n + 500)}
              className="ml-[1.125rem] rounded px-1 py-px font-mono text-[11px] text-accent-text hover:underline"
            >
              show {Math.min(500, entries.length - shown.length)} more of{" "}
              {entries.length - shown.length}
            </button>
          )}
        </div>
      )}
    </div>
  );
}

function Scalar({ value }: { value: unknown }) {
  if (value === null) return <span className="text-muted">null</span>;
  if (typeof value === "number")
    return <span className="tabular-nums text-accent-text">{value}</span>;
  if (typeof value === "boolean") return <span className="text-accent-text">{String(value)}</span>;
  return <span className="text-primary">&quot;{String(value)}&quot;</span>;
}
