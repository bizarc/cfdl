import { Logo } from "@/components/Logo";

const LINKS = [
  { href: "https://cfdl.dev/docs/getting-started", label: "Getting started" },
  { href: "https://cfdl.dev/docs/language-guide", label: "Language guide" },
  { href: "https://cfdl.dev/playground", label: "Playground" },
  { href: "https://cfdl.dev/docs/specification", label: "Specification" },
];

export function LearnFooter() {
  return (
    <footer className="mt-24 border-t border-subtle bg-surface-sunken">
      <div className="mx-auto flex max-w-7xl flex-col gap-8 px-4 py-12 sm:px-6 md:flex-row md:items-start md:justify-between">
        <div>
          <Logo />
          <p className="mt-3 max-w-xs text-sm leading-relaxed text-secondary">
            The course companion to cfdl.dev — learn to author cash-flow
            models, from first stream to a full deal.
          </p>
        </div>

        <div>
          <h4 className="text-xs font-semibold uppercase tracking-wider text-muted">
            On cfdl.dev
          </h4>
          <ul className="mt-3 space-y-2">
            {LINKS.map((link) => (
              <li key={link.href}>
                <a
                  href={link.href}
                  target="_blank"
                  rel="noreferrer"
                  className="text-sm text-secondary transition-colors hover:text-primary"
                >
                  {link.label}
                </a>
              </li>
            ))}
          </ul>
        </div>
      </div>

      <div className="border-t border-subtle">
        <div className="mx-auto flex max-w-7xl flex-col gap-2 px-4 py-6 text-xs text-muted sm:flex-row sm:items-center sm:justify-between sm:px-6">
          <p>© {new Date().getFullYear()} Matthew McCrea. All rights reserved.</p>
          <p>Pre-1.0 — course content tracks the language as it evolves.</p>
        </div>
      </div>
    </footer>
  );
}
