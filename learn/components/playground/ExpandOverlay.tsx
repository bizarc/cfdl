"use client";

import * as RadixDialog from "@radix-ui/react-dialog";
import { Minimize2 } from "lucide-react";
import type { ReactNode } from "react";

/**
 * A results tab, filled to the viewport.
 *
 * A statement is the reason this exists. A monthly pro forma over a six-year
 * hold is 74 columns and roughly 6,500px of table; in a half-width results
 * pane the reader sees the labels, the total, and about one period. Nothing
 * about the layout of that table is wrong — there is simply more of it than
 * the pane can hold.
 *
 * DELIBERATELY NOT `window.open`. A real second window has to re-serialise the
 * results into a second React root, re-establish the theme, and survive popup
 * blocking — three failure modes bought in exchange for a window the reader
 * can drag to another monitor. This renders the same component with the same
 * live state, so a re-run updates it and nothing can drift.
 */
export function ExpandOverlay({
  open,
  onOpenChange,
  title,
  toolbar,
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  toolbar?: ReactNode;
  children: ReactNode;
}) {
  return (
    <RadixDialog.Root open={open} onOpenChange={onOpenChange}>
      <RadixDialog.Portal>
        <RadixDialog.Overlay className="fixed inset-0 z-[200] bg-surface-inverse/50 backdrop-blur-[2px] data-[state=open]:animate-in data-[state=open]:fade-in" />
        <RadixDialog.Content
          aria-describedby={undefined}
          className="fixed inset-2 z-[300] flex flex-col overflow-hidden rounded-lg border border-default bg-surface-page shadow-lg sm:inset-4"
        >
          <div className="flex shrink-0 items-center gap-3 border-b border-subtle px-4 py-2">
            <RadixDialog.Title className="text-sm font-semibold text-primary">
              {title}
            </RadixDialog.Title>
            <div className="ml-auto flex items-center gap-2">
              {toolbar}
              <RadixDialog.Close
                aria-label="Collapse"
                title="Collapse (Esc)"
                className="rounded-md p-1.5 text-muted transition-colors hover:bg-surface-sunken hover:text-primary"
              >
                <Minimize2 className="h-4 w-4" />
              </RadixDialog.Close>
            </div>
          </div>
          <div className="min-h-0 flex-1 overflow-hidden">{children}</div>
        </RadixDialog.Content>
      </RadixDialog.Portal>
    </RadixDialog.Root>
  );
}
