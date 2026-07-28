"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { ChevronRight } from "lucide-react";
import { NAV, type NavItem, type NavSection } from "@/content/nav";
import { cn } from "@/lib/cn";

function SidebarLink({ item, pathname }: { item: NavItem; pathname: string }) {
  const active = pathname === item.slug;
  return (
    <Link
      href={item.slug}
      aria-current={active ? "page" : undefined}
      className={cn(
        "block rounded-md px-3 py-1.5 transition-colors",
        active
          ? "bg-accent-soft font-medium text-accent-text"
          : "text-secondary hover:bg-surface-sunken hover:text-primary",
      )}
    >
      {item.title}
    </Link>
  );
}

/** True when this section owns the page being viewed. */
function sectionContains(section: NavSection, pathname: string): boolean {
  return section.items.some(
    (item) => item.slug === pathname || item.items?.some((c) => c.slug === pathname),
  );
}

function Section({ section, pathname }: { section: NavSection; pathname: string }) {
  // Fully expanded, every section at once, the sidebar ran past 1600px — long
  // enough that whole sections sat below the fold and read as missing. Only the
  // section you are reading opens by default; the rest stay one line each, so
  // the shape of the documentation is visible at a glance.
  const current = sectionContains(section, pathname);
  const [open, setOpen] = useState(current);

  return (
    <div className="mb-1">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
        className={cn(
          "flex w-full items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-semibold uppercase tracking-wider transition-colors",
          current ? "text-primary" : "text-muted hover:text-secondary",
        )}
      >
        <ChevronRight
          className={cn("h-3 w-3 shrink-0 transition-transform", open && "rotate-90")}
          aria-hidden
        />
        {section.title}
      </button>

      {open ? (
        <ul className="mb-4 space-y-px">
          {section.items.map((item) => (
            <li key={item.slug}>
              <SidebarLink item={item} pathname={pathname} />
              {item.items ? (
                <ul className="ml-3 space-y-px border-l border-subtle pl-1">
                  {item.items.map((child) => (
                    <li key={child.slug}>
                      <SidebarLink item={child} pathname={pathname} />
                    </li>
                  ))}
                </ul>
              ) : null}
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}

export function DocsSidebar({ className }: { className?: string }) {
  const pathname = usePathname();

  return (
    <nav className={cn("text-sm", className)} aria-label="Documentation">
      {NAV.map((section) => (
        // Keyed by pathname so landing on a new page re-opens the section that
        // owns it, rather than preserving whatever was toggled before.
        <Section key={`${section.title}:${pathname}`} section={section} pathname={pathname} />
      ))}
    </nav>
  );
}
