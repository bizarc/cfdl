import Link from "next/link";
import { Logo } from "@/components/Logo";
import { firstChapter, partGroups } from "@/lib/chapters";

const EXTERNAL = [
  { href: "https://cfdl.dev/docs/getting-started", label: "Getting started" },
  { href: "https://cfdl.dev/docs/language-guide", label: "Language guide" },
  { href: "https://cfdl.dev/playground", label: "Playground" },
  { href: "https://cfdl.dev/docs/specification", label: "Specification" },
];

export function LearnFooter() {
  const start = firstChapter();
  // Every part that has chapters, as a jump to where it begins.
  const parts = partGroups().filter((p) => p.chapters.length > 0);

  return (
    <footer className="mt-24 border-t border-subtle bg-surface-sunken">
      <div className="mx-auto grid max-w-7xl gap-10 px-4 py-12 sm:px-6 md:grid-cols-[1.5fr_1fr_1fr]">
        <div>
          <Logo />
          <p className="mt-3 max-w-xs text-sm leading-relaxed text-secondary">
            The course companion to cfdl.dev — learn to author cash-flow
            models, from first stream to a full deal.
          </p>
          {start && (
            <Link
              href={start.slug}
              className="mt-4 inline-flex text-sm font-medium text-accent-text hover:underline"
            >
              Start the course →
            </Link>
          )}
        </div>

        <div>
          <h4 className="text-xs font-semibold uppercase tracking-wider text-muted">
            The course
          </h4>
          <ul className="mt-3 space-y-2">
            {parts.map((part) => (
              <li key={part.part}>
                <Link
                  href={part.chapters[0].slug}
                  className="text-sm text-secondary transition-colors hover:text-primary"
                >
                  {part.title}
                </Link>
              </li>
            ))}
            <li>
              <Link
                href="/course-kit"
                className="text-sm text-secondary transition-colors hover:text-primary"
              >
                Course kit
              </Link>
            </li>
          </ul>
        </div>

        <div>
          <h4 className="text-xs font-semibold uppercase tracking-wider text-muted">
            On cfdl.dev
          </h4>
          <ul className="mt-3 space-y-2">
            {EXTERNAL.map((link) => (
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
