"use client";

import dynamic from "next/dynamic";

/**
 * The IDE is a tool, not content: there is nothing useful to server-render,
 * and skipping SSR lets it read the share hash and local draft in state
 * initializers instead of a post-mount effect (no flash, no hydration
 * mismatch).
 */
const Playground = dynamic(
  () => import("./Playground").then((m) => m.Playground),
  {
    ssr: false,
    loading: () => (
      <div className="flex flex-1 items-center justify-center py-24 text-sm text-muted">
        Loading the playground…
      </div>
    ),
  },
);

export function PlaygroundLoader() {
  return <Playground />;
}
