"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { NAV } from "@/content/nav";
import { cn } from "@/lib/cn";

export function DocsSidebar({ className }: { className?: string }) {
  const pathname = usePathname();

  return (
    <nav className={cn("text-sm", className)} aria-label="Documentation">
      {NAV.map((section) => (
        <div key={section.title} className="mb-6">
          <h2 className="mb-2 px-3 text-xs font-semibold uppercase tracking-wider text-muted">
            {section.title}
          </h2>
          <ul className="space-y-px">
            {section.items.map((item) => {
              const active = pathname === item.slug;
              return (
                <li key={item.slug}>
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
                </li>
              );
            })}
          </ul>
        </div>
      ))}
    </nav>
  );
}
