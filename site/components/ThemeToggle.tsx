"use client";

import { useTheme } from "next-themes";
import { Moon, Sun } from "lucide-react";

/**
 * Both icons are always rendered and CSS picks one from the `data-theme`
 * attribute on <html>. That keeps server and client markup identical (no
 * hydration mismatch) without a mounted flag, and the icon is correct on
 * first paint instead of after hydration.
 */
export function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();

  return (
    <button
      type="button"
      aria-label="Toggle color theme"
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
      className="inline-flex h-9 w-9 items-center justify-center rounded-md border border-transparent text-secondary transition-colors hover:bg-surface-sunken hover:text-primary"
    >
      <Moon className="h-4 w-4 [html[data-theme='dark']_&]:hidden" />
      <Sun className="hidden h-4 w-4 [html[data-theme='dark']_&]:block" />
    </button>
  );
}
