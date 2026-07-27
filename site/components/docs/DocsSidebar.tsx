"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { NAV, type NavItem } from "@/content/nav";
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
        </div>
      ))}
    </nav>
  );
}
