"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { getEngineClient, type RunOutcome } from "@/lib/playground/client";
import type { RunConfig } from "@/lib/playground/protocol";

export type EngineStatus = "starting" | "ready" | "running" | "error";

export function useEngine() {
  const clientRef = useRef(getEngineClient());
  const [status, setStatus] = useState<EngineStatus>("starting");
  const [readyMs, setReadyMs] = useState<number | null>(null);

  // Warm the engine on mount, before any interaction — by the time someone
  // presses Run (or a deep link auto-runs) the module is already compiled.
  useEffect(() => {
    const started = performance.now();
    let cancelled = false;

    clientRef.current
      .ready()
      .then(() => {
        if (cancelled) return;
        setReadyMs(Math.round(performance.now() - started));
        setStatus("ready");
      })
      .catch(() => {
        if (!cancelled) setStatus("error");
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const run = useCallback(
    async (args: {
      files: Record<string, string>;
      root: string;
      config?: RunConfig;
      pack?: string;
    }): Promise<RunOutcome> => {
      setStatus("running");
      const outcome = await clientRef.current.run(args);
      setStatus("ready");
      return outcome;
    },
    [],
  );

  const cancel = useCallback(() => {
    clientRef.current.cancel();
    setStatus("ready");
  }, []);

  return { status, readyMs, run, cancel };
}
