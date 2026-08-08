"use client";

import { useEffect, useState } from "react";
import { cn } from "@/lib/cn";

export interface TocEntry {
  id: string;
  text: string;
  depth: number;
}

export function TableOfContents({ entries }: { entries: TocEntry[] }) {
  const [activeId, setActiveId] = useState<string | null>(null);

  useEffect(() => {
    if (entries.length === 0) return;

    const observer = new IntersectionObserver(
      (records) => {
        const visible = records
          .filter((r) => r.isIntersecting)
          .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
        if (visible[0]) setActiveId(visible[0].target.id);
      },
      // Bias toward the heading nearest the top of the reading area.
      { rootMargin: "-80px 0px -70% 0px", threshold: 0 },
    );

    for (const entry of entries) {
      const el = document.getElementById(entry.id);
      if (el) observer.observe(el);
    }
    return () => observer.disconnect();
  }, [entries]);

  if (entries.length === 0) return null;

  return (
    <nav aria-label="On this page" className="text-sm">
      <h2 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted">
        On this page
      </h2>
      <ul className="space-y-1 border-l border-subtle">
        {entries.map((entry) => (
          <li key={entry.id}>
            <a
              href={`#${entry.id}`}
              className={cn(
                "-ml-px block border-l py-1 pl-3 transition-colors",
                entry.depth >= 3 && "pl-6",
                activeId === entry.id
                  ? "border-accent text-accent-text"
                  : "border-transparent text-secondary hover:text-primary",
              )}
            >
              {entry.text}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  );
}
