"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Badge } from "@/components/ds/Badge";
import { cn } from "@/lib/cn";
import type { ChapterMeta } from "@/lib/chapters";

/**
 * Course navigation: every part appears from day one — parts without
 * published chapters are listed as planned, so a reader always sees the whole
 * arc of the course, not just what exists so far.
 */
export function ChaptersSidebar({
  chapters,
  parts,
}: {
  chapters: ChapterMeta[];
  parts: Record<number, string>;
}) {
  const pathname = usePathname();

  return (
    <nav aria-label="Chapters" className="space-y-6 text-sm">
      {Object.entries(parts).map(([num, title]) => {
        const inPart = chapters.filter((c) => c.part === Number(num));
        return (
          <div key={num}>
            <p className="text-xs font-semibold uppercase tracking-wider text-muted">
              Part {num} · {title}
            </p>
            {inPart.length > 0 ? (
              <ul className="mt-2 space-y-0.5">
                {inPart.map((chapter) => {
                  const active = pathname === chapter.slug;
                  return (
                    <li key={chapter.slug}>
                      <Link
                        href={chapter.slug}
                        aria-current={active ? "page" : undefined}
                        className={cn(
                          "flex items-center gap-2 rounded-md px-2 py-1.5 transition-colors",
                          active
                            ? "bg-accent-soft font-medium text-accent-text"
                            : "text-secondary hover:bg-surface-sunken hover:text-primary",
                        )}
                      >
                        <span className="min-w-0 flex-1 truncate">{chapter.title}</span>
                        {chapter.track === "deep" && (
                          <Badge tone="neutral" className="shrink-0 px-1.5 text-[10px]">
                            deep dive
                          </Badge>
                        )}
                      </Link>
                    </li>
                  );
                })}
              </ul>
            ) : (
              <p className="mt-2 px-2 text-xs text-muted">Planned</p>
            )}
          </div>
        );
      })}
    </nav>
  );
}
