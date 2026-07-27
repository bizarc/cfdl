"use client";

import { useState } from "react";
import { Check, Copy, PlayCircle } from "lucide-react";
import { sharePath } from "@/lib/playground/share";
import { cn } from "@/lib/cn";

/**
 * Copy and open-in-playground actions for a documentation code block.
 *
 * Getting a snippet from the docs into the playground used to mean selecting
 * it by hand and pasting; both actions here remove that step. "Open in
 * playground" reuses the playground's own share encoding, so the link carries
 * the model in the URL fragment and the playground runs it on arrival —
 * nothing is uploaded anywhere.
 */
export function CodeActions({
  code,
  lang,
  className,
}: {
  code: string;
  lang?: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);

  const onCopy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard can be denied (permissions, insecure context); the code is
      // still selectable, so fail quietly rather than interrupting the reader.
    }
  };

  // Only whole models can be opened: a fragment would land in the editor and
  // fail to compile, which is a worse experience than no button.
  const isRunnableModel = lang === "cfdl" && /^\s*version\s/m.test(code);
  // Root-relative on purpose: this renders on the server too, and an
  // origin-dependent href would hydrate to a different value.
  const playgroundHref = isRunnableModel
    ? sharePath({ files: { "model.cfdl": code }, root: "model.cfdl" })
    : null;

  return (
    <div
      className={cn(
        "absolute right-2 top-2 flex items-center gap-1",
        // Visible on hover and whenever anything inside has focus, so the
        // actions stay reachable by keyboard.
        "opacity-0 transition-opacity focus-within:opacity-100 group-hover:opacity-100",
        className,
      )}
    >
      {playgroundHref ? (
        <a
          href={playgroundHref}
          className="inline-flex items-center gap-1 rounded-md border border-default bg-surface-raised px-2 py-1 text-[11px] font-medium text-secondary transition-colors hover:border-strong hover:text-primary"
        >
          <PlayCircle className="h-3 w-3" />
          Open in playground
        </a>
      ) : null}

      <button
        type="button"
        onClick={onCopy}
        aria-label={copied ? "Copied" : "Copy code"}
        className="inline-flex items-center gap-1 rounded-md border border-default bg-surface-raised px-2 py-1 text-[11px] font-medium text-secondary transition-colors hover:border-strong hover:text-primary"
      >
        {copied ? (
          <>
            <Check className="h-3 w-3 text-ok" />
            Copied
          </>
        ) : (
          <>
            <Copy className="h-3 w-3" />
            Copy
          </>
        )}
      </button>
    </div>
  );
}
