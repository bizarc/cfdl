"use client";

import { useEffect } from "react";

/**
 * Warms the browser cache with the engine bundle while the reader is on a
 * content page, so opening the playground (or running a docs cell) starts
 * from a local copy instead of a ~490 KB download.
 *
 * Deliberately deferred to idle and skipped on metered or slow connections —
 * this is an optimization, never a tax on someone who only came to read.
 */
export function EnginePrefetch() {
  useEffect(() => {
    const connection = (
      navigator as Navigator & {
        connection?: { saveData?: boolean; effectiveType?: string };
      }
    ).connection;

    if (connection?.saveData) return;
    if (connection?.effectiveType && /2g/.test(connection.effectiveType)) return;

    const schedule =
      typeof window.requestIdleCallback === "function"
        ? window.requestIdleCallback
        : (cb: () => void) => window.setTimeout(cb, 2000);

    // Must carry the same cache-busting build id the worker uses
    // (lib/playground/engine.worker.ts), or the prefetch warms a URL nobody
    // requests and every visitor pays for the engine twice.
    const build = process.env.NEXT_PUBLIC_WASM_BUILD ?? "dev";

    const handle = schedule(() => {
      for (const href of [
        `/wasm/cfdl_wasm.js?v=${build}`,
        `/wasm/cfdl_wasm_bg.wasm?v=${build}`,
      ]) {
        if (document.querySelector(`link[rel="prefetch"][href="${href}"]`)) continue;
        const link = document.createElement("link");
        link.rel = "prefetch";
        link.href = href;
        if (href.endsWith(".wasm")) link.as = "fetch";
        link.crossOrigin = "anonymous";
        document.head.appendChild(link);
      }
    });

    return () => {
      if (typeof window.cancelIdleCallback === "function" && typeof handle === "number") {
        window.cancelIdleCallback(handle);
      }
    };
  }, []);

  return null;
}
