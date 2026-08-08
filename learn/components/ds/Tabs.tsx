"use client";

import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

export interface TabItem {
  id: string;
  label: string;
  /** Small count shown after the label (diagnostics, scenarios, …). */
  badge?: number;
  badgeTone?: "neutral" | "err";
}

/**
 * Underlined tab bar. Kept as a controlled, roving-focus list rather than a
 * Radix Tabs wrapper because panels here are rendered by the parent, and the
 * bar needs to scroll horizontally inside a narrow pane.
 */
export function Tabs({
  items,
  value,
  onValueChange,
  className,
}: {
  items: TabItem[];
  value: string;
  onValueChange: (id: string) => void;
  className?: string;
}) {
  return (
    <div
      role="tablist"
      className={cn(
        "flex items-center gap-1 overflow-x-auto border-b border-subtle bg-surface-sunken px-2",
        className,
      )}
    >
      {items.map((item) => {
        const active = item.id === value;
        return (
          <button
            key={item.id}
            role="tab"
            type="button"
            aria-selected={active}
            onClick={() => onValueChange(item.id)}
            className={cn(
              "relative whitespace-nowrap px-3 py-2 text-xs font-medium transition-colors",
              active
                ? "text-primary after:absolute after:inset-x-2 after:bottom-0 after:h-0.5 after:rounded-full after:bg-accent"
                : "text-muted hover:text-secondary",
            )}
          >
            {item.label}
            {item.badge !== undefined && item.badge > 0 ? (
              <span
                className={cn(
                  "ml-1.5 rounded-full px-1.5 py-0.5 text-[10px] tabular-nums",
                  item.badgeTone === "err"
                    ? "bg-err-soft text-err"
                    : "bg-surface-raised text-muted",
                )}
              >
                {item.badge}
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}

/** Collapsible section used for progressive disclosure in dense panels. */
export function Disclosure({
  open,
  onOpenChange,
  title,
  summary,
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  summary?: string;
  children: ReactNode;
}) {
  return (
    <div className="rounded-lg border border-default">
      <button
        type="button"
        onClick={() => onOpenChange(!open)}
        aria-expanded={open}
        className="flex w-full items-center justify-between gap-2 px-3 py-2 text-left"
      >
        <span className="text-xs font-semibold uppercase tracking-wider text-muted">
          {title}
        </span>
        <span className="flex items-center gap-2">
          {!open && summary ? (
            <span className="font-mono text-[11px] text-secondary">{summary}</span>
          ) : null}
          <span className="text-xs text-muted">{open ? "−" : "+"}</span>
        </span>
      </button>
      {open ? <div className="space-y-3 border-t border-subtle p-3">{children}</div> : null}
    </div>
  );
}
