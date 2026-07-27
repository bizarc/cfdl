import type { Metadata } from "next";
import { SiteHeader } from "@/components/SiteHeader";
import { SiteFooter } from "@/components/SiteFooter";
import { DesignSystemShowcase } from "@/components/design/DesignSystemShowcase";

export const metadata: Metadata = {
  title: "Design system",
  description:
    "CFDL's design tokens, components, and interaction patterns — rendered from the real components.",
};

const SEMANTIC_TOKENS = [
  {
    group: "Surface",
    tokens: [
      ["--cfdl-surface-page", "Page background"],
      ["--cfdl-surface-raised", "Cards, panels, popovers"],
      ["--cfdl-surface-sunken", "Toolbars, table headers, wells"],
      ["--cfdl-surface-code", "Code blocks and editors"],
    ],
  },
  {
    group: "Text",
    tokens: [
      ["--cfdl-text-primary", "Body and headings"],
      ["--cfdl-text-secondary", "Supporting copy"],
      ["--cfdl-text-muted", "Labels, captions, metadata"],
      ["--cfdl-text-accent", "Links and emphasis"],
    ],
  },
  {
    group: "Border",
    tokens: [
      ["--cfdl-border-subtle", "Dividers inside a surface"],
      ["--cfdl-border-default", "Component outlines"],
      ["--cfdl-border-strong", "Hover and focus outlines"],
    ],
  },
  {
    group: "Status",
    tokens: [
      ["--cfdl-status-ok", "Success, engine ready"],
      ["--cfdl-status-warn", "Warnings"],
      ["--cfdl-status-err", "Errors, failed compiles"],
    ],
  },
  {
    group: "Chart",
    tokens: [
      ["--cfdl-chart-series-1", "First series"],
      ["--cfdl-chart-series-2", "Second series / in-band"],
      ["--cfdl-chart-series-3", "Third series"],
      ["--cfdl-chart-grid", "Gridlines"],
    ],
  },
];

const PATTERNS = [
  {
    title: "Never use native dialogs",
    // dialogs-allow: prose describing the rule, not a call
    body:
      "alert(), confirm(), and prompt() can't be styled, block the main thread, and are suppressed in some embedding contexts. Use <Dialog> — it traps focus, closes on Escape, and validates inline where a prompt could only reject silently. Enforced by scripts/check-no-native-dialogs.mjs.",
  },
  {
    title: "Colour comes from semantic tokens only",
    body: "Components name a role (bg-surface-raised, text-secondary), never a colour. Both themes then follow for free. scripts/check-tokens.sh fails the build on a raw hex outside tokens.css; a third-party API that demands a literal takes a documented // tokens-allow: comment.",
  },
  {
    title: "Async work shows progress and stays cancellable",
    body: "Anything that can outlast a frame runs off the main thread and reports state — the engine badge moves through starting → ready → running, and long runs expose Stop. Never leave a control looking idle while work is happening.",
  },
  {
    title: "Empty states explain themselves",
    body: "An empty panel says why it is empty and what to do about it: 'No scenarios in this run — add a scenarios block to the run config.' A zero where a number was expected is worse than nothing, because it reads as a broken calculation.",
  },
  {
    title: "Validate where the user is looking",
    body: "Show the specific problem next to the field that has it, and disable the confirming action until it is resolved. Don't reject input silently and don't wait until submit to explain.",
  },
  {
    title: "Every control has a label",
    body: "Use <Field> to pair a label, hint, and error with a control. Icon-only buttons carry aria-label. Focus is always visible via the global :focus-visible ring.",
  },
];

export default function DesignSystemPage() {
  return (
    <>
      <SiteHeader />
      <main className="mx-auto w-full max-w-5xl flex-1 px-4 py-10 sm:px-6">
        <h1 className="text-3xl font-semibold tracking-tight text-primary">Design system</h1>
        <p className="mt-3 max-w-2xl text-base leading-relaxed text-secondary">
          Tokens, components, and interaction patterns for cfdl.dev. This page imports the
          real components, so it cannot drift from what ships.
        </p>

        <DesignSystemShowcase semanticTokens={SEMANTIC_TOKENS} patterns={PATTERNS} />
      </main>
      <SiteFooter />
    </>
  );
}
