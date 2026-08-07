"use client";

import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Menu, X } from "lucide-react";
import { NAV } from "@/lib/nav";
import { cn } from "@/lib/cn";

/**
 * The site nav, below md.
 *
 * The desktop bar is `hidden md:flex` and had no counterpart, so a phone got a
 * logo, a theme toggle, and no way to reach the docs at all — the Playground
 * button drops out below sm too, which left the header with nothing but the
 * wordmark. This is that missing half.
 */
export function MobileNav() {
  const [open, setOpen] = useState(false);
  const pathname = usePathname();

  // A tapped link navigates without unmounting the header, so the panel would
  // stay open over the page it just opened. Adjusted during render rather than
  // in an effect so the panel is never painted over the new route.
  const [lastPath, setLastPath] = useState(pathname);
  if (lastPath !== pathname) {
    setLastPath(pathname);
    setOpen(false);
  }

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);

  return (
    <div className="md:hidden">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
        aria-controls="mobile-nav"
        aria-label={open ? "Close menu" : "Open menu"}
        className="rounded-md p-1.5 text-secondary transition-colors hover:bg-surface-sunken hover:text-primary"
      >
        {open ? <X className="size-5" /> : <Menu className="size-5" />}
      </button>

      {open ? (
        <>
          {/* Portaled to the body deliberately. The header sets `backdrop-blur`,
              and a backdrop-filter makes an element the containing block for
              its `fixed` descendants — so a fixed overlay rendered here
              resolved against the 56px header instead of the viewport, covered
              nothing, and left the page underneath tappable with the menu
              open. */}
          {createPortal(
            <button
              type="button"
              aria-label="Close menu"
              tabIndex={-1}
              onClick={() => setOpen(false)}
              className="fixed inset-x-0 bottom-0 top-14 z-[90] bg-surface-inverse/40"
            />,
            document.body,
          )}
          <nav
            id="mobile-nav"
            className="absolute inset-x-0 top-14 z-[95] border-b border-subtle bg-surface-page p-2 shadow-lg"
          >
            {[...NAV, { href: "/playground", label: "Playground" }].map((item) => {
              const active = pathname === item.href;
              return (
                <Link
                  key={item.href}
                  href={item.href}
                  onClick={() => setOpen(false)}
                  className={cn(
                    "block rounded-md px-3 py-2.5 text-sm transition-colors",
                    active
                      ? "bg-accent-soft text-accent-text"
                      : "text-secondary hover:bg-surface-sunken hover:text-primary",
                  )}
                >
                  {item.label}
                </Link>
              );
            })}
          </nav>
        </>
      ) : null}
    </div>
  );
}
