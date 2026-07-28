import Link from "next/link";
import { Logo } from "@/components/Logo";

const COLUMNS = [
  {
    title: "Learn",
    links: [
      { href: "/docs/getting-started", label: "Getting started" },
      { href: "/docs/concepts", label: "How CFDL works" },
      { href: "/docs/language-guide", label: "Language guide" },
      { href: "/playground", label: "Playground" },
    ],
  },
  {
    title: "Build",
    links: [
      { href: "/docs/install", label: "Install" },
      { href: "/docs/packs", label: "Domain packs" },
      { href: "/docs/python-sdk", label: "Python SDK" },
      { href: "/docs/api-server", label: "API server" },
    ],
  },
  {
    title: "Reference",
    links: [
      { href: "/docs/language-reference", label: "Specs & schemas" },
      { href: "/docs/reference/cli", label: "CLI reference" },
      { href: "/docs/benchmarks", label: "Benchmarks" },
      { href: "/docs/licensing", label: "Licensing" },
      { href: "https://github.com/bizarc/cfdl", label: "Source on GitHub" },
    ],
  },
];

export function SiteFooter() {
  return (
    <footer className="mt-24 border-t border-subtle bg-surface-sunken">
      <div className="mx-auto grid max-w-7xl gap-10 px-4 py-12 sm:px-6 md:grid-cols-[1.5fr_repeat(3,1fr)]">
        <div>
          <Logo />
          <p className="mt-3 max-w-xs text-sm leading-relaxed text-secondary">
            A deterministic, source-available language for cash-flow models —
            with the distribution around every number.
          </p>
        </div>

        {COLUMNS.map((col) => (
          <div key={col.title}>
            <h4 className="text-xs font-semibold uppercase tracking-wider text-muted">
              {col.title}
            </h4>
            <ul className="mt-3 space-y-2">
              {col.links.map((link) => (
                <li key={link.href}>
                  {link.href.startsWith("http") ? (
                    <a
                      href={link.href}
                      target="_blank"
                      rel="noreferrer"
                      className="text-sm text-secondary transition-colors hover:text-primary"
                    >
                      {link.label}
                    </a>
                  ) : (
                    <Link
                      href={link.href}
                      className="text-sm text-secondary transition-colors hover:text-primary"
                    >
                      {link.label}
                    </Link>
                  )}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>

      <div className="border-t border-subtle">
        <div className="mx-auto flex max-w-7xl flex-col gap-2 px-4 py-6 text-xs text-muted sm:flex-row sm:items-center sm:justify-between sm:px-6">
          <p>© {new Date().getFullYear()} Matthew McCrea. Source available under BUSL-1.1.</p>
          <p>Pre-1.0 — interfaces may change until 1.0 freezes the schemas.</p>
        </div>
      </div>
    </footer>
  );
}
