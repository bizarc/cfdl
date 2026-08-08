import Link from "next/link";
import { Logo } from "@/components/Logo";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";

const NAV = [{ href: "/#curriculum", label: "Curriculum" }];

export function LearnHeader() {
  return (
    <header className="sticky top-0 z-[100] border-b border-subtle bg-surface-page/85 backdrop-blur">
      <div className="mx-auto flex h-14 max-w-7xl items-center gap-4 px-4 sm:px-6">
        <Link href="/" className="flex shrink-0 items-center gap-2">
          <Logo />
          <Badge tone="accent">Academy</Badge>
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
            <a href="https://cfdl.dev" target="_blank" rel="noreferrer">
              cfdl.dev
            </a>
          </Button>
          <ThemeToggle />
        </div>
      </div>
    </header>
  );
}
