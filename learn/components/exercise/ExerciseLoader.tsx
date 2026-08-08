"use client";

import dynamic from "next/dynamic";

/**
 * The runner owns a web worker and Monaco, neither of which exists on the
 * server — same ssr:false boundary the site's playground uses.
 */
export const ExerciseRunnerClient = dynamic(() => import("./ExerciseRunner"), {
  ssr: false,
  loading: () => (
    <div className="not-prose flex h-72 items-center justify-center rounded-lg border border-default text-sm text-muted">
      Loading exercise…
    </div>
  ),
});
