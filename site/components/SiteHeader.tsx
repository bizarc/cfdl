import Link from "next/link";
import { Logo } from "@/components/Logo";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Button } from "@/components/ds/Button";

const NAV = [
  { href: "/docs/getting-started", label: "Docs" },
  { href: "/docs/packs", label: "Packs" },
  { href: "/docs/benchmarks", label: "Benchmarks" },
  { href: "/docs/language-reference", label: "Reference" },
];

/** lucide-react dropped brand marks, so the GitHub glyph lives here. */

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-[100] border-b border-subtle bg-surface-page/85 backdrop-blur">
      <div className="mx-auto flex h-14 max-w-7xl items-center gap-6 px-4 sm:px-6">
        <Link href="/" className="shrink-0">
          <Logo />
        </Link>

        <nav className="hidden items-center gap-1 md:flex">
          {NAV.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className="rounded-md px-3 py-1.5 text-sm text-secondary transition-colors hover:bg-surface-sunken hover:text-primary"
            >
              {item.label}
            </Link>
          ))}
        </nav>

        <div className="ml-auto flex items-center gap-2">
          <Button asChild size="sm" variant="secondary" className="hidden sm:inline-flex">
            <Link href="/playground">Playground</Link>
          </Button>
          <ThemeToggle />
        </div>
      </div>
    </header>
  );
}
