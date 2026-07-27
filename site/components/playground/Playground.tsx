"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Play, Square, Loader2, CheckCircle2, AlertCircle } from "lucide-react";
import { Button } from "@/components/ds/Button";
import { Badge } from "@/components/ds/Badge";
import { useEngine } from "./useEngine";
import type { Diagnostic, RunConfig } from "@/lib/playground/protocol";

const STARTER = `version 0.1
model "first-model"
time calendar monthly from 2026-01 for 24

entity legal company

assume growth ~ Normal(mean=0.02, stdev=0.01, clip=[0.0, 0.05])

stream legal.revenue on entity legal.company inflow currency USD {
  schedule every monthly from 2026-01 to 2027-12
  amount = 10000 * pow(1 + inputs.growth, time.t / 12.0)
}
`;

const RUN_CONFIG: RunConfig = {
  deterministic: { annual_discount_rate: 0.08 },
  monte_carlo: { trial_count: 500, seed: 42 },
};

type MetricValue = number | { amount: number; currency?: string };

function formatMetric(value: MetricValue): string {
  if (typeof value === "number") {
    return Number.isInteger(value) ? value.toString() : value.toFixed(4);
  }
  const amount = value.amount.toLocaleString("en-US", {
    maximumFractionDigits: 2,
  });
  return value.currency ? `${amount} ${value.currency}` : amount;
}

interface Results {
  deterministic?: { metrics?: Record<string, MetricValue> };
  monte_carlo?: {
    status?: string;
    trials?: number;
    seed?: number;
    metrics?: Record<string, Record<string, MetricValue>>;
  };
}

export function Playground() {
  const { status, readyMs, run, cancel } = useEngine();
  const [source, setSource] = useState(STARTER);
  const [results, setResults] = useState<Results | null>(null);
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([]);
  const [engineError, setEngineError] = useState<string | null>(null);
  const [elapsed, setElapsed] = useState<number | null>(null);
  const autoRan = useRef(false);

  const execute = useCallback(
    async (code: string) => {
      const started = performance.now();
      const outcome = await run({
        files: { "model.cfdl": code },
        root: "model.cfdl",
        config: RUN_CONFIG,
      });
      setElapsed(Math.round(performance.now() - started));

      if (outcome.status === "ok") {
        setResults(outcome.results as Results);
        setDiagnostics([]);
        setEngineError(null);
      } else if (outcome.status === "diagnostics") {
        setDiagnostics(outcome.diagnostics);
        setResults(null);
        setEngineError(null);
      } else {
        setEngineError(outcome.message);
        setResults(null);
        setDiagnostics([]);
      }
    },
    [run],
  );

  // Results on arrival: as soon as the engine is warm, run the starter once so
  // the page shows real output instead of an empty pane awaiting a click.
  useEffect(() => {
    if (status !== "ready" || autoRan.current) return;
    autoRan.current = true;
    void execute(source);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status]);

  const busy = status === "running";
  const npv = results?.deterministic?.metrics?.["model.npv"];
  const mcNpv = results?.monte_carlo?.metrics?.["model.npv"];

  return (
    <div className="mx-auto w-full max-w-7xl flex-1 px-4 py-6 sm:px-6">
      <div className="mb-4 flex flex-wrap items-center gap-3">
        <h1 className="text-xl font-semibold tracking-tight text-primary">Playground</h1>
        <EngineBadge status={status} readyMs={readyMs} />
        <div className="ml-auto flex items-center gap-2">
          {busy ? (
            <Button variant="secondary" size="sm" onClick={cancel}>
              <Square className="h-3.5 w-3.5" />
              Stop
            </Button>
          ) : null}
          <Button size="sm" onClick={() => execute(source)} disabled={status === "starting" || busy}>
            {busy ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Play className="h-3.5 w-3.5" />
            )}
            Run
          </Button>
        </div>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <section className="flex flex-col overflow-hidden rounded-lg border border-default">
          <header className="border-b border-subtle bg-surface-sunken px-4 py-2">
            <span className="font-mono text-xs text-muted">model.cfdl</span>
          </header>
          <textarea
            value={source}
            onChange={(e) => setSource(e.target.value)}
            spellCheck={false}
            aria-label="CFDL model source"
            className="min-h-[420px] flex-1 resize-none bg-surface-code p-4 font-mono text-[13px] leading-relaxed text-primary outline-none"
          />
        </section>

        <section className="flex flex-col overflow-hidden rounded-lg border border-default">
          <header className="flex items-center gap-2 border-b border-subtle bg-surface-sunken px-4 py-2">
            <span className="text-xs font-medium text-secondary">Results</span>
            {elapsed !== null && !busy ? (
              <span className="ml-auto font-mono text-xs text-muted">{elapsed} ms</span>
            ) : null}
          </header>

          <div className="min-h-[420px] flex-1 overflow-auto p-4">
            {engineError ? (
              <p className="text-sm text-err">{engineError}</p>
            ) : diagnostics.length > 0 ? (
              <ul className="space-y-3">
                {diagnostics.map((d, i) => (
                  <li key={i} className="rounded-md border border-default bg-err-soft p-3">
                    <div className="flex items-center gap-2">
                      <AlertCircle className="h-3.5 w-3.5 text-err" />
                      <code className="font-mono text-xs text-err">{d.code}</code>
                      {d.span ? (
                        <span className="font-mono text-xs text-muted">
                          line {d.span.start_line}:{d.span.start_col}
                        </span>
                      ) : null}
                    </div>
                    <p className="mt-1.5 text-sm text-primary">{d.message}</p>
                    {d.hint ? <p className="mt-1 text-xs text-secondary">{d.hint}</p> : null}
                  </li>
                ))}
              </ul>
            ) : results ? (
              <div className="space-y-6">
                {npv !== undefined ? (
                  <div>
                    <p className="text-xs uppercase tracking-wider text-muted">NPV</p>
                    <p className="mt-1 font-mono text-2xl font-semibold tabular-nums text-primary">
                      {formatMetric(npv)}
                    </p>
                  </div>
                ) : null}

                {mcNpv ? (
                  <div>
                    <p className="text-xs uppercase tracking-wider text-muted">
                      Monte Carlo — {results?.monte_carlo?.trials?.toLocaleString()} trials, seed{" "}
                      {results?.monte_carlo?.seed}
                    </p>
                    <dl className="mt-2 grid grid-cols-2 gap-x-6 gap-y-1.5 sm:grid-cols-3">
                      {["mean", "stdev", "p05", "p50", "p95"].map((key) =>
                        mcNpv[key] !== undefined ? (
                          <div key={key} className="flex justify-between gap-2 text-sm">
                            <dt className="text-muted">{key}</dt>
                            <dd className="font-mono tabular-nums text-secondary">
                              {formatMetric(mcNpv[key])}
                            </dd>
                          </div>
                        ) : null,
                      )}
                    </dl>
                  </div>
                ) : null}

                <details>
                  <summary className="cursor-pointer text-xs text-muted hover:text-secondary">
                    All metrics
                  </summary>
                  <dl className="mt-3 space-y-1.5">
                    {Object.entries(results?.deterministic?.metrics ?? {}).map(([k, v]) => (
                      <div key={k} className="flex justify-between gap-4 text-sm">
                        <dt className="font-mono text-xs text-muted">{k}</dt>
                        <dd className="font-mono tabular-nums text-secondary">{formatMetric(v)}</dd>
                      </div>
                    ))}
                  </dl>
                </details>
              </div>
            ) : (
              <p className="text-sm text-muted">
                {status === "starting" ? "Starting the engine…" : "Press Run."}
              </p>
            )}
          </div>
        </section>
      </div>

      <p className="mt-4 text-xs text-muted">
        The compiler and engine run entirely in your browser, off the main
        thread. A full IDE — multi-file editing, cash-flow charts, scenario and
        Monte Carlo workbenches — is the next increment.
      </p>
    </div>
  );
}

function EngineBadge({ status, readyMs }: { status: string; readyMs: number | null }) {
  if (status === "starting") {
    return (
      <Badge>
        <Loader2 className="h-3 w-3 animate-spin" />
        Starting engine
      </Badge>
    );
  }
  if (status === "error") {
    return (
      <Badge tone="err">
        <AlertCircle className="h-3 w-3" />
        Engine failed to load
      </Badge>
    );
  }
  if (status === "running") {
    return (
      <Badge tone="accent">
        <Loader2 className="h-3 w-3 animate-spin" />
        Running
      </Badge>
    );
  }
  return (
    <Badge tone="ok">
      <CheckCircle2 className="h-3 w-3" />
      Engine ready{readyMs !== null ? ` · ${readyMs} ms` : ""}
    </Badge>
  );
}
