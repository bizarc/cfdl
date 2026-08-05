"use client";

import { useMemo, useState } from "react";
import { AlertCircle, AlertTriangle, Info } from "lucide-react";
import { LineChart, type LineSeries } from "./charts/LineChart";
import { CashFlowChart, type CashFlowBand } from "./charts/CashFlowChart";
import { Tabs } from "@/components/ds/Tabs";
import { Distribution } from "./charts/Distribution";
import { cn } from "@/lib/cn";
import {
  formatValue,
  histogram,
  percentilesFromTrials,
  periodLabels,
  toNumber,
  type Results,
} from "@/lib/playground/results";
import type { Diagnostic } from "@/lib/playground/protocol";

const TABS = [
  "Metrics",
  "Statement",
  "Cash flows",
  "Scenarios",
  "Monte Carlo",
  "Diagnostics",
  "JSON",
] as const;
type Tab = (typeof TABS)[number];

export function ResultsPanel({
  results,
  diagnostics,
  engineError,
  onJumpTo,
  selectedPack,
  modelDeclaredPack,
}: {
  results: Results | null;
  diagnostics: Diagnostic[];
  engineError: string | null;
  onJumpTo?: (file: string | undefined, line: number) => void;
  selectedPack?: string;
  modelDeclaredPack?: string;
}) {
  const [tab, setTab] = useState<Tab>("Metrics");

  const counts = {
    diagnostics: diagnostics.length,
    scenarios: results?.scenarios?.summaries?.length ?? 0,
    mc: results?.monte_carlo?.status === "ok" ? results.monte_carlo.trials ?? 0 : 0,
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <Tabs
        className="shrink-0"
        value={tab}
        onValueChange={(id) => setTab(id as Tab)}
        items={TABS.map((t) => ({
          id: t,
          label: t,
          badge:
            t === "Diagnostics"
              ? counts.diagnostics
              : t === "Scenarios"
                ? counts.scenarios
                : undefined,
          badgeTone: t === "Diagnostics" ? ("err" as const) : ("neutral" as const),
        }))}
      />

      <div className="min-h-0 flex-1 overflow-auto p-4">
        {engineError ? (
          <Empty tone="err">{engineError}</Empty>
        ) : tab === "Metrics" ? (
          <MetricsTab
            results={results}
            selectedPack={selectedPack}
            modelDeclaredPack={modelDeclaredPack}
          />
        ) : tab === "Statement" ? (
          <StatementTab results={results} />
        ) : tab === "Cash flows" ? (
          <CashFlowsTab results={results} />
        ) : tab === "Scenarios" ? (
          <ScenariosTab results={results} />
        ) : tab === "Monte Carlo" ? (
          <MonteCarloTab results={results} />
        ) : tab === "Diagnostics" ? (
          <DiagnosticsTab diagnostics={diagnostics} onJumpTo={onJumpTo} />
        ) : (
          <JsonTab results={results} />
        )}
      </div>
    </div>
  );
}

function Empty({ children, tone }: { children: React.ReactNode; tone?: "err" }) {
  return (
    <p className={cn("text-sm", tone === "err" ? "text-err" : "text-muted")}>{children}</p>
  );
}

function MetricsTab({
  results,
  selectedPack,
  modelDeclaredPack,
}: {
  results: Results | null;
  selectedPack?: string;
  modelDeclaredPack?: string;
}) {
  const metrics = results?.deterministic?.metrics;
  const domain = results?.domain_metrics?.metrics;
  if (!metrics) return <Empty>Run a model to see metrics.</Empty>;

  const headline = ["model.npv", "model.irr", "model.moic", "model.payback_years"].filter(
    (k) => metrics[k] !== undefined,
  );
  // "All metrics" was one alphabetical list, which interleaved things that have
  // nothing to do with each other — a run's discount rate next to an entity
  // total next to a stream total. The metric names already encode their kind as
  // a prefix; this groups on it rather than inventing a taxonomy.
  //
  // `run.*` is deliberately last and named "Run configuration": those are
  // INPUTS echoed back, not results, and reading them as outputs is exactly the
  // confusion the flat list created.
  const GROUPS: { title: string; match: (k: string) => boolean }[] = [
    { title: "Model", match: (k) => k.startsWith("model.") },
    { title: "Per stream", match: (k) => k.startsWith("stream.") },
    { title: "Per entity", match: (k) => k.startsWith("entity.") },
    { title: "Per option", match: (k) => k.startsWith("option.") },
    { title: "Run configuration", match: (k) => k.startsWith("run.") },
  ];
  const rest = Object.entries(metrics).filter(([k]) => !headline.includes(k));
  const grouped = GROUPS.map((g) => ({
    title: g.title,
    rows: rest.filter(([k]) => g.match(k)),
  })).filter((g) => g.rows.length > 0);
  // Anything a future engine emits under a prefix nobody listed still shows up,
  // rather than vanishing from the panel because the taxonomy did not know it.
  const ungrouped = rest.filter(([k]) => !GROUPS.some((g) => g.match(k)));

  // A pack the model doesn't declare produces metrics that match no streams —
  // a column of 0.00 that reads as a broken calculation. Say what happened.
  const packMismatch =
    Boolean(selectedPack) && selectedPack !== modelDeclaredPack;

  return (
    <div className="space-y-6">
      {headline.length > 0 && (
        <div className="grid gap-3 sm:grid-cols-2 2xl:grid-cols-4">
          {headline.map((k) => (
            <div key={k} className="min-w-0 rounded-lg border border-default bg-surface-raised p-3">
              <p className="truncate font-mono text-[11px] text-muted" title={k}>
                {k}
              </p>
              <p className="mt-1 font-mono text-lg font-semibold tabular-nums text-primary">
                {formatValue(metrics[k], k)}
              </p>
            </div>
          ))}
        </div>
      )}

      {grouped.map((g) => (
        <MetricTable key={g.title} title={g.title} rows={g.rows} />
      ))}
      <MetricTable title="Other" rows={ungrouped} />

      {packMismatch ? (
        <div className="rounded-lg border border-default bg-surface-sunken p-3">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-muted">
            Domain metrics
          </h3>
          <p className="mt-1.5 text-xs leading-relaxed text-secondary">
            This model doesn&apos;t use the{" "}
            <code className="font-mono">{selectedPack}</code> pack
            {modelDeclaredPack ? (
              <>
                {" "}
                (it declares <code className="font-mono">{modelDeclaredPack}</code>)
              </>
            ) : (
              " (it declares none)"
            )}
            , so its metrics match no streams. Add{" "}
            <code className="font-mono">use pack &quot;{selectedPack}&quot;</code> to the
            model, or set the pack selector to match.
          </p>
        </div>
      ) : domain && Object.keys(domain).length > 0 ? (
        <MetricTable title="Domain metrics" rows={Object.entries(domain)} />
      ) : null}
    </div>
  );
}

function MetricTable({
  title,
  rows,
}: {
  title: string;
  rows: [string, Parameters<typeof formatValue>[0]][];
}) {
  if (rows.length === 0) return null;
  return (
    <div>
      <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted">{title}</h3>
      <dl className="divide-y divide-subtle rounded-lg border border-default">
        {rows.map(([k, v]) => (
          <div key={k} className="flex items-baseline justify-between gap-4 px-3 py-1.5">
            <dt className="min-w-0 truncate font-mono text-xs text-secondary" title={k}>
              {k}
            </dt>
            <dd className="font-mono text-xs tabular-nums text-primary">
              {formatValue(v, k)}
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}

function CashFlowsTab({ results }: { results: Results | null }) {
  const [annual, setAnnual] = useState(false);
  // Grain and accumulation are independent questions, so they are two controls
  // rather than three buttons in one row.
  //
  // CUMULATIVE EXISTS BECAUSE OF SCALE. A reversion is routinely 70x a month's
  // rent, and on a per-period axis that one bar compresses every operating
  // period into the zero line. No drawing fixes that — the range is real. What
  // changes is the question: accumulated, the same spike is the moment the
  // curve crosses back over, and the trough before it is the capital at risk.
  // It is the J-curve every RE and PE reader already knows how to read, and it
  // makes `model.payback_years` visible rather than merely reported.
  const [cumulative, setCumulative] = useState(false);
  const source = annual
    ? results?.deterministic?.annual_rollup?.series
    : results?.deterministic?.series;

  const chart = useMemo(() => {
    if (!source) return null;

    // This tab charts CASH, and two things had to be excluded before the six
    // slots meant anything.
    //
    // RATIOS. `domain.cre.dscr` lives between about 0.5 and 1.5 and was being
    // drawn on an axis scaled to millions, so it rendered as a flat line on
    // zero. A ratio carries no currency, which is exactly how it is detected
    // here — a money value is `{ amount, currency }`, a ratio is a bare number.
    //
    // FOLDS BEFORE STREAMS. The picker took the first six names
    // alphabetically, and `domain` sorts before `stream` — so the moment
    // subtotals were published, every actual stream was evicted from the
    // default chart by aggregates OF those streams. Charting both double-counts
    // the same cash anyway. Streams rank first now; folds fill the remainder.
    const isMoney = (name: string) =>
      source[name].values.some((v) => v !== null && typeof v === "object");
    const vals = (name: string) => source[name].values.map((v) => toNumber(v) ?? 0);

    // FOLDS ARE NOT COMPONENTS. `domain.*` are aggregates OF the streams, so
    // stacking both draws the same cash twice. They are excluded from the bars
    // outright rather than ranked below them.
    const componentNames = Object.keys(source).filter(
      (k) => isMoney(k) && k !== "model.net_cash_flow" && !k.startsWith("domain."),
    );
    if (componentNames.length === 0) return null;

    // Ranked by how much cash each moves, not by name. Alphabetical order put
    // whatever happened to sort first in front of whatever mattered.
    const weight = (name: string) =>
      vals(name).reduce((acc, v) => acc + Math.abs(v), 0);
    const ranked = [...componentNames].sort((a, b) => weight(b) - weight(a));
    const shown = ranked.slice(0, 6);

    const bands: CashFlowBand[] = shown.map((name, i) => ({
      name: name.replace(/^(stream|option)\./, ""),
      colorIndex: (i % 6) + 1,
      values: vals(name),
    }));

    // Everything past the sixth is summed into one band rather than dropped, so
    // the stack still sums to the net line and the chart cannot quietly
    // misrepresent the period.
    const remainder = ranked.slice(6);
    if (remainder.length > 0) {
      const n = Math.max(...shown.map((s) => source[s].values.length));
      const acc = new Array(n).fill(0);
      for (const name of remainder) {
        vals(name).forEach((v, i) => {
          acc[i] += v;
        });
      }
      bands.push({ name: `${remainder.length} others`, colorIndex: 6, values: acc });
    }

    let net = source["model.net_cash_flow"] ? vals("model.net_cash_flow") : undefined;

    if (cumulative) {
      const running = (xs: number[]) => {
        let acc = 0;
        return xs.map((v) => (acc += v));
      };
      for (const b of bands) b.values = running(b.values);
      if (net) net = running(net);
    }

    const labels = periodLabels(source[shown[0]].index);
    return { bands, net, labels, hidden: remainder.length, names: ranked, cumulative };
  }, [source, cumulative]);

  if (!chart) return <Empty>Run a model to see cash flows.</Empty>;

  const hasAnnual = Boolean(results?.deterministic?.annual_rollup?.series);

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1">
        {hasAnnual && (
          <div className="flex gap-1">
            {[
              ["Periodic", false],
              ["Annual", true],
            ].map(([label, value]) => (
              <button
                key={String(label)}
                type="button"
                onClick={() => setAnnual(value as boolean)}
                className={cn(
                  "rounded-md px-2.5 py-1 text-xs transition-colors",
                  annual === value
                    ? "bg-accent-soft text-accent-text"
                    : "text-muted hover:text-secondary",
                )}
              >
                {label}
              </button>
            ))}
          </div>
        )}
        <div className="flex gap-1">
          {[
            ["Per period", false],
            ["Cumulative", true],
          ].map(([label, value]) => (
            <button
              key={String(label)}
              type="button"
              onClick={() => setCumulative(value as boolean)}
              className={cn(
                "rounded-md px-2.5 py-1 text-xs transition-colors",
                cumulative === value
                  ? "bg-accent-soft text-accent-text"
                  : "text-muted hover:text-secondary",
              )}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      <CashFlowChart bands={chart.bands} net={chart.net} labels={chart.labels} />
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px]">
        {chart.net && (
          <span className="text-secondary">
            <span className="mr-1 inline-block h-px w-3 align-middle bg-current" />
            net cash flow
          </span>
        )}
        {chart.bands.map((b) => (
          <span key={b.name} className="text-muted">
            <span
              className="mr-1 inline-block size-1.5 rounded-full align-middle"
              style={{ background: `var(--cfdl-chart-series-${b.colorIndex})` }}
            />
            {b.name}
          </span>
        ))}
      </div>
      {chart.hidden > 0 ? (
        <p className="text-xs text-muted">
          The six largest movers are shown separately; the remaining {chart.hidden} are
          summed into one band, so the bars still total the net line. Every series is in
          the JSON tab.
        </p>
      ) : null}

      <div className="overflow-x-auto rounded-lg border border-default">
        <table className="w-full text-xs">
          <thead>
            <tr className="border-b border-default bg-surface-sunken">
              <th className="px-3 py-2 text-left font-semibold text-primary">Period</th>
              {chart.bands.map((b) => (
                <th key={b.name} className="px-3 py-2 text-right font-mono font-medium text-secondary">
                  {b.name}
                </th>
              ))}
              {chart.net && (
                <th className="px-3 py-2 text-right font-mono font-semibold text-primary">
                  net
                </th>
              )}
            </tr>
          </thead>
          <tbody>
            {chart.labels.slice(0, 60).map((label, i) => (
              <tr key={label} className="border-b border-subtle last:border-0">
                <td className="px-3 py-1.5 font-mono text-muted">{label}</td>
                {chart.bands.map((b) => (
                  <td key={b.name} className="px-3 py-1.5 text-right font-mono tabular-nums text-secondary">
                    {b.values[i]?.toLocaleString("en-US", { maximumFractionDigits: 0 }) ?? "—"}
                  </td>
                ))}
                {chart.net && (
                  <td className="px-3 py-1.5 text-right font-mono font-medium tabular-nums text-primary">
                    {chart.net[i]?.toLocaleString("en-US", { maximumFractionDigits: 0 }) ?? "—"}
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {chart.labels.length > 60 ? (
        <p className="text-xs text-muted">First 60 periods shown.</p>
      ) : null}
    </div>
  );
}

function ScenariosTab({ results }: { results: Results | null }) {
  const summaries = results?.scenarios?.summaries ?? [];
  if (summaries.length === 0) {
    return (
      <Empty>
        No scenarios in this run. Add a `scenarios` block to the run config to compare
        variants side by side.
      </Empty>
    );
  }

  const metricNames = Array.from(
    new Set(summaries.flatMap((s) => Object.keys(s.metrics))),
  ).sort();

  return (
    <div className="overflow-x-auto rounded-lg border border-default">
      <table className="w-full text-xs">
        <thead>
          <tr className="border-b border-default bg-surface-sunken">
            <th className="px-3 py-2 text-left font-semibold text-primary">Metric</th>
            {summaries.map((s) => (
              <th key={s.name} className="px-3 py-2 text-right font-semibold text-primary">
                {s.name}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {metricNames.map((m) => (
            <tr key={m} className="border-b border-subtle last:border-0">
              <td className="px-3 py-1.5 font-mono text-secondary">{m}</td>
              {summaries.map((s) => (
                <td key={s.name} className="px-3 py-1.5 text-right font-mono tabular-nums text-primary">
                  {formatValue(s.metrics[m], m)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function MonteCarloTab({ results }: { results: Results | null }) {
  const mc = results?.monte_carlo;
  const [metric, setMetric] = useState<string | null>(null);

  const metricNames = useMemo(() => Object.keys(mc?.metrics ?? {}).sort(), [mc]);
  const active = metric ?? (metricNames.includes("model.npv") ? "model.npv" : metricNames[0]);

  const derived = useMemo(() => {
    if (!mc?.trial_summaries || !active) return null;
    const values = mc.trial_summaries
      .map((t) => toNumber(t.metrics?.[active]))
      .filter((v): v is number => v !== undefined);
    if (values.length === 0) return null;
    return {
      values,
      hist: histogram(values),
      pct: percentilesFromTrials(mc.trial_summaries, active, [5, 25, 50, 75, 95]),
    };
  }, [mc, active]);

  if (!mc || mc.status !== "ok") {
    return (
      <Empty>
        Monte Carlo did not run. Add a `monte_carlo` block (trial_count and seed) to the run
        config, and at least one `assume … ~ Distribution(…)` to the model.
      </Empty>
    );
  }

  const summary = active ? mc.metrics?.[active] : undefined;

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-center gap-3">
        <span className="text-xs text-muted">
          {mc.trials?.toLocaleString()} trials · seed {mc.seed}
        </span>
        {metricNames.length > 1 ? (
          <select
            value={active}
            onChange={(e) => setMetric(e.target.value)}
            aria-label="Metric"
            className="rounded-md border border-default bg-surface-raised px-2 py-1 font-mono text-xs text-primary"
          >
            {metricNames.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
        ) : null}
      </div>

      {derived ? (
        <Distribution
          counts={derived.hist.counts}
          min={derived.hist.min}
          max={derived.hist.max}
          p05={derived.pct?.[5]}
          p50={derived.pct?.[50]}
          p95={derived.pct?.[95]}
        />
      ) : null}

      <div className="grid gap-3 sm:grid-cols-2">
        <MetricTable
          title="Engine summary"
          rows={
            summary
              ? (["mean", "stdev", "min", "max", "p50"] as const)
                  .filter((k) => summary[k] !== undefined)
                  .map((k) => [k, summary[k]] as [string, never])
              : []
          }
        />
        {derived?.pct ? (
          <MetricTable
            title="Percentiles (from trials)"
            rows={[5, 25, 50, 75, 95].map((p) => [`p${p}`, derived.pct![p]] as [string, never])}
          />
        ) : null}
      </div>
    </div>
  );
}

function DiagnosticsTab({
  diagnostics,
  onJumpTo,
}: {
  diagnostics: Diagnostic[];
  onJumpTo?: (file: string | undefined, line: number) => void;
}) {
  if (diagnostics.length === 0) {
    return <Empty>No diagnostics — the model compiled cleanly.</Empty>;
  }

  return (
    <ul className="space-y-3">
      {diagnostics.map((d, i) => {
        const Icon =
          d.severity === "warning" ? AlertTriangle : d.severity === "info" ? Info : AlertCircle;
        const tone =
          d.severity === "warning"
            ? "border-warn bg-warn-soft text-warn"
            : d.severity === "info"
              ? "border-default bg-surface-sunken text-secondary"
              : "border-err bg-err-soft text-err";
        return (
          <li key={i} className={cn("rounded-md border-l-2 p-3", tone)}>
            <div className="flex flex-wrap items-center gap-2">
              <Icon className="h-3.5 w-3.5" />
              <code className="font-mono text-xs">{d.code}</code>
              {d.span ? (
                <button
                  type="button"
                  onClick={() => onJumpTo?.(d.file, d.span!.start_line)}
                  className="font-mono text-xs text-muted underline-offset-2 hover:underline"
                >
                  {d.file ? `${d.file}:` : ""}
                  {d.span.start_line}:{d.span.start_col}
                </button>
              ) : null}
            </div>
            <p className="mt-1.5 text-sm text-primary">{d.message}</p>
            {d.hint ? <p className="mt-1 text-xs text-secondary">Hint: {d.hint}</p> : null}
            {d.notes?.length ? (
              <ul className="mt-1 space-y-0.5">
                {d.notes.map((n, j) => (
                  <li key={j} className="text-xs text-secondary">
                    · {n}
                  </li>
                ))}
              </ul>
            ) : null}
          </li>
        );
      })}
    </ul>
  );
}

function JsonTab({ results }: { results: Results | null }) {
  if (!results) return <Empty>Run a model to see the raw Results JSON.</Empty>;
  return (
    <div className="space-y-3">
      <p className="text-xs text-muted">
        Results schema {results.results_version} · engine {results.engine?.version} · model hash{" "}
        <code className="font-mono">{results.model_hash?.slice(0, 12)}…</code>
      </p>
      <pre className="overflow-x-auto rounded-lg border border-default bg-surface-code p-3 font-mono text-[11px] leading-relaxed text-secondary">
        {JSON.stringify(results, null, 2)}
      </pre>
    </div>
  );
}

/**
 * A pack's declared statement, rendered as a pro forma.
 *
 * The arithmetic is all done: every value here was folded by the engine per
 * period. This arranges it — order, indent, labels, and the display sign that
 * turns a stored negative into Argus's "less:" row of positives. `values` stays
 * the signed quantity, so the column still adds up for anyone reading the JSON.
 *
 * Column labels come from `statement.grain.labels` rather than `periodLabels`.
 * They have to: an annual statement over a monthly model has ten columns where
 * the model has 120, and a SeriesIndex cannot say which ten.
 */
function StatementTab({ results }: { results: Results | null }) {
  const statements = results?.statements?.statements;
  const [selected, setSelected] = useState<string | null>(null);
  const [drill, setDrill] = useState<number | null>(null);

  const active = useMemo(() => {
    if (!statements?.length) return null;
    return (
      statements.find((s) => s.id === selected) ??
      statements.find((s) => s.default) ??
      statements[0]
    );
  }, [statements, selected]);

  if (!statements?.length) {
    return (
      <Empty>
        Run a model with a pack that declares statements to see a pro forma.
      </Empty>
    );
  }
  if (!active) return <Empty>No statement to show.</Empty>;

  const labels = active.grain?.labels ?? [];
  const recon = active.reconciliation;
  // Published always and asserted rather than corrected: if the rows do not add
  // up to the model's cash, the number saying so is on screen.
  const residual = recon?.residual ?? 0;
  const reconciles = Math.abs(residual) < 0.01;

  return (
    <div className="space-y-4">
      {statements.length > 1 && (
        <div className="flex flex-wrap gap-1">
          {statements.map((s) => (
            <button
              key={s.id}
              type="button"
              onClick={() => {
                setSelected(s.id);
                setDrill(null);
              }}
              className={cn(
                "rounded-md px-2.5 py-1 text-xs transition-colors",
                s.id === active.id
                  ? "bg-accent-soft text-accent-text"
                  : "text-muted hover:text-secondary",
              )}
            >
              {s.label}
              <span className="ml-1.5 opacity-60">{s.grain?.calendar}</span>
            </button>
          ))}
        </div>
      )}

      {active.diagnostics?.map((d, i) => (
        <div
          key={i}
          className="flex items-start gap-2 rounded-md border border-warn/40 bg-warn/5 p-2 text-xs text-secondary"
        >
          <AlertTriangle className="mt-px size-3.5 shrink-0 text-warn" />
          <span>
            {d.code ? <span className="font-mono opacity-70">{d.code} </span> : null}
            {d.message}
          </span>
        </div>
      ))}

      <div className="overflow-x-auto rounded-lg border border-default">
        <table className="w-full border-collapse text-xs">
          <thead>
            <tr className="border-b border-default bg-surface-raised">
              <th className="sticky left-0 z-10 bg-surface-raised px-3 py-2 text-left font-medium text-secondary">
                {active.label}
              </th>
              <th className="px-3 py-2 text-right font-medium text-secondary">Total</th>
              {labels.map((l) => (
                <th
                  key={l}
                  className="whitespace-nowrap px-3 py-2 text-right font-normal text-muted"
                >
                  {l}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {active.rows.map((row, i) => {
              if (row.kind === "spacer") {
                return (
                  <tr key={i}>
                    <td colSpan={labels.length + 2} className="h-3" />
                  </tr>
                );
              }
              const emphasis =
                row.kind === "subtotal" || row.kind === "ratio" || row.kind === "residual";
              const canDrill = Boolean(row.streams?.length);
              return (
                <tr
                  key={i}
                  onClick={() => canDrill && setDrill(drill === i ? null : i)}
                  className={cn(
                    "border-b border-default/50",
                    emphasis && "font-medium text-primary",
                    row.kind === "residual" && "text-warn",
                    canDrill && "cursor-pointer hover:bg-surface-raised",
                    drill === i && "bg-surface-raised",
                  )}
                >
                  <td
                    className={cn(
                      "sticky left-0 z-10 whitespace-nowrap bg-surface px-3 py-1.5 text-left",
                      emphasis && "bg-surface-raised",
                      drill === i && "bg-surface-raised",
                    )}
                    style={{ paddingLeft: `${0.75 + row.depth * 0.85}rem` }}
                  >
                    {row.label}
                    {canDrill && (
                      <span className="ml-1.5 text-[10px] opacity-40">
                        {row.streams!.length}
                      </span>
                    )}
                  </td>
                  <td className="whitespace-nowrap px-3 py-1.5 text-right tabular-nums">
                    {row.total === undefined
                      ? ""
                      : fmtSigned(row.total * (row.display_sign ?? 1))}
                  </td>
                  {labels.map((l, t) => {
                    const v = row.values?.[t];
                    const n = v === null || v === undefined ? undefined : toNumber(v);
                    return (
                      <td
                        key={l}
                        className="whitespace-nowrap px-3 py-1.5 text-right tabular-nums text-secondary"
                      >
                        {n === undefined
                          ? "—"
                          : row.kind === "ratio"
                            ? n.toFixed(4)
                            : fmtSigned(n * (row.display_sign ?? 1))}
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {drill !== null && active.rows[drill]?.streams?.length ? (
        <div className="rounded-lg border border-default p-3">
          <p className="mb-2 text-xs font-medium text-secondary">
            {active.rows[drill].label} draws from
          </p>
          <ul className="space-y-1">
            {active.rows[drill].streams!.map((s) => (
              <li key={s} className="font-mono text-xs text-muted">
                {s}
              </li>
            ))}
          </ul>
        </div>
      ) : null}

      {recon && (
        <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-xs text-muted">
          <span>
            Bottom line{" "}
            <span className="tabular-nums text-secondary">{fmtSigned(recon.bottom_line ?? 0)}</span>
          </span>
          <span>
            Model total{" "}
            <span className="tabular-nums text-secondary">{fmtSigned(recon.model_total ?? 0)}</span>
          </span>
          <span className={cn(reconciles ? "text-muted" : "text-warn")}>
            Residual <span className="tabular-nums">{residual.toFixed(6)}</span>
          </span>
        </div>
      )}
    </div>
  );
}

/** Thousands-separated, two decimals, parenthesised when negative. */
function fmtSigned(n: number): string {
  const s = Math.abs(n).toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
  return n < 0 ? `(${s})` : s;
}
