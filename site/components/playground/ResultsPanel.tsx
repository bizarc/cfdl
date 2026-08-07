"use client";

import { Fragment, useMemo, useState } from "react";
import {
  AlertCircle,
  AlertTriangle,
  Check,
  ChevronRight,
  Copy,
  Download,
  Info,
  Maximize2,
} from "lucide-react";
import { CashFlowChart, type CashFlowBand } from "./charts/CashFlowChart";
import { Tabs } from "@/components/ds/Tabs";
import { Distribution } from "./charts/Distribution";
import { ExpandOverlay } from "./ExpandOverlay";
import { JsonTree } from "./JsonTree";
import { cn } from "@/lib/cn";
import { copyText, downloadText, toDelimited, type Cell } from "@/lib/playground/export";
import {
  currencyOf,
  formatValue,
  histogram,
  percentilesFromTrials,
  periodLabels,
  toNumber,
  type MoneyOrNumber,
  type Results,
  type Statement,
  type StatementRow,
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

/**
 * Tabs that scroll their own body.
 *
 * A sticky table header only sticks against the scroll container it lives in,
 * so a tab with a header worth pinning has to own its scrolling — the panel
 * cannot scroll on its behalf.
 */
const SELF_SCROLL: readonly Tab[] = ["Statement", "Cash flows", "JSON"];

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
  const [expanded, setExpanded] = useState(false);

  // Per-tab view state lives here rather than in the tab components, so that
  // expanding to full screen keeps the collapsed sections, the chosen column
  // grain and the series selection. Moving a subtree to a different parent
  // remounts it, and remounting would silently reset all of that.
  const [stmt, setStmt] = useState<StatementView>({
    id: null,
    grain: null,
    collapsed: new Set<number>(),
    drill: null,
  });
  const [cash, setCash] = useState<CashView>({
    annual: false,
    cumulative: false,
    series: null,
    showAll: false,
  });
  const [mcMetric, setMcMetric] = useState<string | null>(null);
  const [jsonRaw, setJsonRaw] = useState(false);

  const counts = {
    diagnostics: diagnostics.length,
    scenarios: results?.scenarios?.summaries?.length ?? 0,
    mc: results?.monte_carlo?.status === "ok" ? results.monte_carlo.trials ?? 0 : 0,
  };

  const selfScroll = SELF_SCROLL.includes(tab);

  const body = engineError ? (
    <Empty tone="err">{engineError}</Empty>
  ) : tab === "Metrics" ? (
    <MetricsTab
      results={results}
      selectedPack={selectedPack}
      modelDeclaredPack={modelDeclaredPack}
    />
  ) : tab === "Statement" ? (
    <StatementTab results={results} view={stmt} onView={setStmt} />
  ) : tab === "Cash flows" ? (
    <CashFlowsTab results={results} view={cash} onView={setCash} />
  ) : tab === "Scenarios" ? (
    <ScenariosTab results={results} />
  ) : tab === "Monte Carlo" ? (
    <MonteCarloTab results={results} metric={mcMetric} onMetric={setMcMetric} />
  ) : tab === "Diagnostics" ? (
    <DiagnosticsTab diagnostics={diagnostics} onJumpTo={onJumpTo} />
  ) : (
    <JsonTab results={results} raw={jsonRaw} onRaw={setJsonRaw} />
  );

  const bodyClass = selfScroll ? "min-h-0 flex-1 overflow-hidden" : "min-h-0 flex-1 overflow-auto p-4";

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-stretch border-b border-subtle bg-surface-sunken">
        {/* The bar has always scrolled, but with nothing to say so — JSON sat
            clipped off the right edge at 1280 and read as missing. The fade is
            the affordance; the action buttons are pulled out of the scroll
            area so they cannot be scrolled away from. */}
        <div className="relative min-w-0 flex-1">
          <Tabs
            className="border-b-0 bg-transparent"
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
          <div className="pointer-events-none absolute inset-y-0 right-0 w-8 bg-gradient-to-l from-surface-sunken to-transparent" />
        </div>

        <div className="flex shrink-0 items-center border-l border-subtle px-1.5">
          <ToolButton
            label="Expand"
            title="Expand to full screen"
            onClick={() => setExpanded(true)}
            icon={<Maximize2 className="size-3.5" />}
          />
        </div>
      </div>

      {expanded ? (
        <div className="flex min-h-0 flex-1 items-center justify-center p-4">
          <p className="text-sm text-muted">{tab} is open in the expanded view.</p>
        </div>
      ) : (
        <div className={bodyClass}>{body}</div>
      )}

      <ExpandOverlay open={expanded} onOpenChange={setExpanded} title={`${tab} · CFDL playground`}>
        <div className={cn("flex h-full min-h-0 flex-col", !selfScroll && "overflow-auto p-4")}>
          {body}
        </div>
      </ExpandOverlay>
    </div>
  );
}

function Empty({ children, tone }: { children: React.ReactNode; tone?: "err" }) {
  return <p className={cn("text-sm", tone === "err" ? "text-err" : "text-muted")}>{children}</p>;
}

/** Empty state for a tab that owns its own scrolling, and so its own padding. */
function PaddedEmpty({ children }: { children: React.ReactNode }) {
  return (
    <div className="p-4">
      <Empty>{children}</Empty>
    </div>
  );
}

/** Small icon/label button used across the tab toolbars. */
function ToolButton({
  label,
  title,
  onClick,
  icon,
  showLabel = false,
}: {
  label: string;
  title?: string;
  onClick: () => void;
  icon: React.ReactNode;
  showLabel?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={title ?? label}
      aria-label={label}
      className="flex items-center gap-1.5 rounded-md px-2 py-1.5 text-xs text-muted transition-colors hover:bg-surface-raised hover:text-primary"
    >
      {icon}
      {showLabel ? label : null}
    </button>
  );
}

/** A copy button that reports success rather than assuming it. */
function CopyButton({ text, label = "Copy" }: { text: () => string; label?: string }) {
  const [done, setDone] = useState(false);
  return (
    <ToolButton
      label={label}
      showLabel
      icon={done ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
      onClick={async () => {
        if (await copyText(text())) {
          setDone(true);
          setTimeout(() => setDone(false), 1500);
        }
      }}
    />
  );
}

/** Segmented control — two or three mutually exclusive view options. */
function Segmented<T extends string | boolean>({
  value,
  onChange,
  options,
  label,
}: {
  value: T;
  onChange: (v: T) => void;
  options: [string, T][];
  label?: string;
}) {
  return (
    <div className="flex items-center gap-1" role="group" aria-label={label}>
      {label ? <span className="mr-0.5 text-[11px] text-muted">{label}</span> : null}
      {options.map(([text, v]) => (
        <button
          key={String(v)}
          type="button"
          onClick={() => onChange(v)}
          aria-pressed={value === v}
          className={cn(
            "rounded-md px-2.5 py-1 text-xs transition-colors",
            value === v
              ? "bg-accent-soft text-accent-text"
              : "text-muted hover:text-secondary",
          )}
        >
          {text}
        </button>
      ))}
    </div>
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
  const packMismatch = Boolean(selectedPack) && selectedPack !== modelDeclaredPack;

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

// ---------------------------------------------------------------------------
// Cash flows
// ---------------------------------------------------------------------------

interface CashView {
  annual: boolean;
  cumulative: boolean;
  /** null means "the six largest movers, chosen for you". */
  series: string[] | null;
  showAll: boolean;
}

const CASH_ROWS_PREVIEW = 60;

function CashFlowsTab({
  results,
  view,
  onView,
}: {
  results: Results | null;
  view: CashView;
  onView: (v: CashView) => void;
}) {
  const { annual, cumulative, series, showAll } = view;
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
    // FOLDS ARE NOT COMPONENTS. `domain.*` are aggregates OF the streams, so
    // stacking both draws the same cash twice. They are excluded from the bars
    // outright rather than ranked below them.
    const isMoney = (name: string) =>
      source[name].values.some((v) => v !== null && typeof v === "object");
    const vals = (name: string) => source[name].values.map((v) => toNumber(v) ?? 0);

    const componentNames = Object.keys(source).filter(
      (k) => isMoney(k) && k !== "model.net_cash_flow" && !k.startsWith("domain."),
    );
    if (componentNames.length === 0) return null;

    // Ranked by how much cash each moves, not by name. Alphabetical order put
    // whatever happened to sort first in front of whatever mattered.
    const weight = (name: string) => vals(name).reduce((acc, v) => acc + Math.abs(v), 0);
    const ranked = [...componentNames].sort((a, b) => weight(b) - weight(a));

    // Auto mode keeps the old top-six behaviour and folds the tail into one
    // band so the stack still sums to the net line. An explicit selection is
    // taken literally — a reader who picked three streams means three.
    const auto = series === null;
    const shown = auto ? ranked.slice(0, 6) : ranked.filter((n) => series.includes(n));
    if (shown.length === 0) {
      return { bands: [], net: undefined, labels: [], hidden: 0, ranked, auto, empty: true };
    }

    const bands: CashFlowBand[] = shown.map((name, i) => ({
      name: name.replace(/^(stream|option)\./, ""),
      colorIndex: (i % 6) + 1,
      values: vals(name),
    }));

    const remainder = auto ? ranked.slice(6) : [];
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

    // The net line is the reference every band is read against, so it stays
    // regardless of which components are selected.
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
    return { bands, net, labels, hidden: remainder.length, ranked, auto, empty: false };
  }, [source, cumulative, series]);

  if (!chart) return <PaddedEmpty>Run a model to see cash flows.</PaddedEmpty>;

  const hasAnnual = Boolean(results?.deterministic?.annual_rollup?.series);
  const visibleRows = showAll ? chart.labels.length : Math.min(CASH_ROWS_PREVIEW, chart.labels.length);

  const csv = () => {
    const rows: Cell[][] = [
      ["Period", ...chart.bands.map((b) => b.name), ...(chart.net ? ["net"] : [])],
    ];
    chart.labels.forEach((label, i) => {
      rows.push([
        label,
        ...chart.bands.map((b) => b.values[i] ?? ""),
        ...(chart.net ? [chart.net[i] ?? ""] : []),
      ]);
    });
    return rows;
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="shrink-0 space-y-3 p-4 pb-3">
        <div className="flex flex-wrap items-center gap-x-4 gap-y-1">
          {hasAnnual && (
            <Segmented
              label="Grain"
              value={annual}
              onChange={(v) => onView({ ...view, annual: v })}
              options={[
                ["Periodic", false],
                ["Annual", true],
              ]}
            />
          )}
          {/* Grain and accumulation are independent questions, so they are two
              controls rather than three buttons in one row.

              CUMULATIVE EXISTS BECAUSE OF SCALE. A reversion is routinely 70x a
              month's rent, and on a per-period axis that one bar compresses
              every operating period into the zero line. Accumulated, the same
              spike is the moment the curve crosses back over, and the trough
              before it is the capital at risk — the J-curve every RE and PE
              reader already knows how to read. */}
          <Segmented
            label="Basis"
            value={cumulative}
            onChange={(v) => onView({ ...view, cumulative: v })}
            options={[
              ["Per period", false],
              ["Cumulative", true],
            ]}
          />
          <div className="ml-auto flex items-center gap-1">
            <CopyButton label="Copy TSV" text={() => toDelimited(csv(), "\t")} />
            <ToolButton
              label="CSV"
              title="Download cash flows as CSV"
              showLabel
              icon={<Download className="size-3.5" />}
              onClick={() =>
                downloadText("cfdl-cash-flows.csv", toDelimited(csv(), ","), "text/csv")
              }
            />
          </div>
        </div>

        <SeriesPicker
          ranked={chart.ranked}
          selected={series}
          onChange={(next) => onView({ ...view, series: next })}
        />
      </div>

      {chart.empty ? (
        <PaddedEmpty>No series selected. Pick at least one above.</PaddedEmpty>
      ) : (
        <div className="min-h-0 flex-1 overflow-auto px-4 pb-4">
          <CashFlowChart bands={chart.bands} net={chart.net} labels={chart.labels} />

          <div className="mt-3 flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px]">
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
            <p className="mt-2 text-xs text-muted">
              The six largest movers are shown separately; the remaining {chart.hidden} are
              summed into one band, so the bars still total the net line. Choose them
              individually above, or read every series in the JSON tab.
            </p>
          ) : null}

          <div className="mt-4 overflow-x-auto rounded-lg border border-default">
            <table className="w-full text-xs">
              <thead className="sticky top-0 z-20">
                <tr className="bg-surface-sunken">
                  <th className="sticky left-0 z-30 whitespace-nowrap border-b border-default bg-surface-sunken px-3 py-2 text-left font-semibold text-primary">
                    Period
                  </th>
                  {chart.bands.map((b) => (
                    <th
                      key={b.name}
                      className="whitespace-nowrap border-b border-default px-3 py-2 text-right font-mono font-medium text-secondary"
                    >
                      {b.name}
                    </th>
                  ))}
                  {chart.net && (
                    <th className="whitespace-nowrap border-b border-default px-3 py-2 text-right font-mono font-semibold text-primary">
                      net
                    </th>
                  )}
                </tr>
              </thead>
              <tbody>
                {chart.labels.slice(0, visibleRows).map((label, i) => (
                  <tr key={label} className="group border-b border-subtle last:border-0 hover:bg-surface-raised">
                    <td className="sticky left-0 z-10 whitespace-nowrap bg-surface-page px-3 py-1.5 font-mono text-muted group-hover:bg-surface-raised">
                      {label}
                    </td>
                    {chart.bands.map((b) => (
                      <td
                        key={b.name}
                        className="px-3 py-1.5 text-right font-mono tabular-nums text-secondary"
                      >
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
          {chart.labels.length > CASH_ROWS_PREVIEW ? (
            <div className="mt-2 flex items-center gap-3 text-xs text-muted">
              <span>
                Showing {visibleRows} of {chart.labels.length} periods.
              </span>
              <button
                type="button"
                onClick={() => onView({ ...view, showAll: !showAll })}
                className="text-accent-text hover:underline"
              >
                {showAll ? `Show first ${CASH_ROWS_PREVIEW}` : "Show all periods"}
              </button>
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
}

/**
 * Which series to draw.
 *
 * The top-six-by-magnitude default is a good guess and a bad rule: a reader
 * checking one small stream against their own model had no way to isolate it,
 * because it was inside the "N others" band by construction.
 */
function SeriesPicker({
  ranked,
  selected,
  onChange,
}: {
  ranked: string[];
  selected: string[] | null;
  onChange: (next: string[] | null) => void;
}) {
  const [open, setOpen] = useState(false);
  const isOn = (name: string) =>
    selected === null ? ranked.indexOf(name) < 6 : selected.includes(name);

  const toggle = (name: string) => {
    const base = selected ?? ranked.slice(0, 6);
    onChange(base.includes(name) ? base.filter((n) => n !== name) : [...base, name]);
  };

  return (
    <div className="rounded-lg border border-default">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
        className="flex w-full items-center gap-1.5 px-3 py-1.5 text-left"
      >
        <ChevronRight className={cn("size-3 text-muted transition-transform", open && "rotate-90")} />
        <span className="text-xs font-medium text-secondary">Series</span>
        <span className="text-[11px] text-muted">
          {selected === null
            ? `top ${Math.min(6, ranked.length)} of ${ranked.length}, chosen automatically`
            : `${selected.length} of ${ranked.length} selected`}
        </span>
        {selected !== null && (
          <span
            role="button"
            tabIndex={0}
            onClick={(e) => {
              e.stopPropagation();
              onChange(null);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.stopPropagation();
                onChange(null);
              }
            }}
            className="ml-auto text-[11px] text-accent-text hover:underline"
          >
            Reset
          </span>
        )}
      </button>
      {open && (
        <div className="@container/picker grid gap-x-4 gap-y-1 border-t border-subtle p-3 @xl/picker:grid-cols-2 @4xl/picker:grid-cols-3">
          {ranked.map((name) => (
            <label key={name} className="flex min-w-0 items-center gap-2 text-[11px]">
              <input
                type="checkbox"
                checked={isOn(name)}
                onChange={() => toggle(name)}
                className="size-3 shrink-0 accent-[var(--cfdl-accent-solid)]"
              />
              <span className="truncate font-mono text-secondary" title={name}>
                {name}
              </span>
            </label>
          ))}
        </div>
      )}
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

function MonteCarloTab({
  results,
  metric,
  onMetric,
}: {
  results: Results | null;
  metric: string | null;
  onMetric: (m: string) => void;
}) {
  const mc = results?.monte_carlo;

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
            onChange={(e) => onMetric(e.target.value)}
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

// ---------------------------------------------------------------------------
// JSON
// ---------------------------------------------------------------------------

/**
 * The raw Results document, and the provenance that goes with it.
 *
 * The identifying facts — schema version, engine build, model hash, seed —
 * were one line of 11px muted text above a wall of JSON, which is an odd place
 * to put the values that decide whether two runs are the same run. They are
 * the header now, and the hash is selectable and copyable in full rather than
 * elided to twelve characters.
 */
function JsonTab({
  results,
  raw,
  onRaw,
}: {
  results: Results | null;
  raw: boolean;
  onRaw: (v: boolean) => void;
}) {
  // Stringifying a 500-trial document on every render was megabytes of work
  // per keystroke elsewhere in the panel.
  const text = useMemo(() => (results ? JSON.stringify(results, null, 2) : ""), [results]);

  if (!results) return <PaddedEmpty>Run a model to see the raw Results JSON.</PaddedEmpty>;

  const hash = results.model_hash ?? "";
  const filename = `cfdl-results${hash ? `-${hash.slice(0, 8)}` : ""}.json`;
  const mc = results.monte_carlo;

  const facts: [string, string | undefined][] = [
    ["Results schema", results.results_version],
    ["Engine", [results.engine?.name, results.engine?.version].filter(Boolean).join(" ") || undefined],
    ["Pack", results.statements?.pack],
    ["Deterministic", results.deterministic?.status],
    ["Scenarios", results.scenarios?.summaries?.length?.toString()],
    [
      "Monte Carlo",
      mc?.status === "ok" ? `${mc.trials?.toLocaleString()} trials · seed ${mc.seed}` : mc?.status,
    ],
  ];

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="shrink-0 space-y-3 p-4 pb-3">
        {/* Container queries, not viewport ones: this card is inside a pane
            whose width the reader controls with the splitter, and a three-column
            grid keyed off a 1280px viewport truncated "cfdl-engine 0.1.0" to
            "cfdl-engi…" in a 476px pane. */}
        <div className="@container rounded-lg border border-default bg-surface-raised p-3">
          <dl className="grid gap-x-6 gap-y-2 @md:grid-cols-2 @3xl:grid-cols-3">
            {facts
              .filter(([, v]) => v)
              .map(([k, v]) => (
                <div key={k} className="min-w-0">
                  <dt className="text-[10px] uppercase tracking-wider text-muted">{k}</dt>
                  <dd className="truncate font-mono text-xs text-primary" title={v}>
                    {v}
                  </dd>
                </div>
              ))}
          </dl>
          {hash ? (
            <div className="mt-3 flex items-center gap-2 border-t border-subtle pt-2.5">
              <span className="shrink-0 text-[11px] uppercase tracking-wider text-muted">
                Model hash
              </span>
              <code className="min-w-0 flex-1 truncate font-mono text-[11px] text-secondary" title={hash}>
                {hash}
              </code>
              <CopyButton label="Copy hash" text={() => hash} />
            </div>
          ) : null}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <Segmented
            value={raw}
            onChange={onRaw}
            label="View"
            options={[
              ["Tree", false],
              ["Raw", true],
            ]}
          />
          <div className="ml-auto flex items-center gap-1">
            <CopyButton label="Copy JSON" text={() => text} />
            <ToolButton
              label="Download"
              title={`Download ${filename}`}
              showLabel
              icon={<Download className="size-3.5" />}
              onClick={() => downloadText(filename, text, "application/json")}
            />
          </div>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-auto px-4 pb-4">
        {raw ? (
          <pre className="overflow-x-auto rounded-lg border border-default bg-surface-code p-3 font-mono text-[11px] leading-relaxed text-secondary">
            {text}
          </pre>
        ) : (
          <div className="rounded-lg border border-default bg-surface-code p-3">
            <JsonTree value={results} name="results" defaultOpenDepth={1} />
          </div>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Statement
// ---------------------------------------------------------------------------

type Grain = "native" | "quarter" | "year";

interface StatementView {
  id: string | null;
  /** null = "not chosen yet", so the default can depend on the statement. */
  grain: Grain | null;
  collapsed: Set<number>;
  drill: number | null;
}

/**
 * The label column, sized against the PANE rather than the viewport.
 *
 * A fixed 16rem is right when the statement is expanded to full screen and
 * absurd in a half-width pane, where it spends more than half the available
 * width on row labels that are then scrolled away from their numbers. These
 * are container queries, so the same table narrows its labels in the pane and
 * widens them in the overlay with no state to keep in sync.
 */
const LABEL_COL =
  "w-40 min-w-40 max-w-40 @2xl:w-56 @2xl:min-w-56 @2xl:max-w-56 @4xl:w-72 @4xl:min-w-72 @4xl:max-w-72";
/** Must track LABEL_COL exactly — it is the Total column's sticky offset. */
const TOTAL_LEFT = "left-40 @2xl:left-56 @4xl:left-72";

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
 *
 * COLUMNS ARE ROLLED UP HERE, NOT IN THE ENGINE. A monthly statement over a
 * six-year hold is 72 period columns — about 6,500px of table, of which a
 * half-width pane shows one. Summing them into years is not a different
 * statement, it is the same statement read at the grain a human asks for
 * first; the engine's monthly numbers are one click away and the export
 * follows whatever is on screen.
 */
function StatementTab({
  results,
  view,
  onView,
}: {
  results: Results | null;
  view: StatementView;
  onView: (v: StatementView) => void;
}) {
  const statements = results?.statements?.statements;

  const active = useMemo(() => {
    if (!statements?.length) return null;
    return (
      statements.find((s) => s.id === view.id) ??
      statements.find((s) => s.default) ??
      statements[0]
    );
  }, [statements, view.id]);

  // Memoised only so the `?? []` fallback doesn't hand `groupColumns` a fresh
  // array identity on every render.
  const labels = useMemo(() => active?.grain?.labels ?? [], [active]);
  const calendar = active?.grain?.calendar ?? "monthly";
  // Twenty-four columns is roughly where a half-width pane stops showing the
  // shape of the deal and starts showing a scrollbar.
  const grain: Grain = view.grain ?? (labels.length > 24 && canRollUp(calendar) ? "year" : "native");

  const cols = useMemo(
    () => groupColumns(labels, calendar, grain),
    [labels, calendar, grain],
  );

  const groups = useMemo(
    () => (active ? subtotalGroups(active.rows) : new Map<number, [number, number]>()),
    [active],
  );
  const hidden = useMemo(() => hiddenRows(groups, view.collapsed), [groups, view.collapsed]);

  if (!statements?.length) {
    return (
      <PaddedEmpty>Run a model with a pack that declares statements to see a pro forma.</PaddedEmpty>
    );
  }
  if (!active) return <PaddedEmpty>No statement to show.</PaddedEmpty>;

  const recon = active.reconciliation;
  // Published always and asserted rather than corrected: if the rows do not add
  // up to the model's cash, the number saying so is on screen.
  const residual = recon?.residual ?? 0;
  const reconciles = Math.abs(residual) < 0.01;
  const currency = statementCurrency(active);

  const table = () => {
    const rows: Cell[][] = [["Line", "Total", ...cols.map((c) => c.label)]];
    for (const row of active.rows) {
      if (row.kind === "spacer") continue;
      const sign = row.display_sign ?? 1;
      rows.push([
        `${"  ".repeat(row.depth)}${row.label ?? ""}`,
        row.total === undefined ? "" : row.total * sign,
        ...cols.map((c) => {
          const v = aggregate(row, c.idx);
          return v === undefined ? "" : row.kind === "ratio" ? v : v * sign;
        }),
      ]);
    }
    return rows;
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="shrink-0 space-y-3 p-4 pb-3">
        {statements.length > 1 && (
          <div className="flex flex-wrap gap-1">
            {statements.map((s) => (
              <button
                key={s.id}
                type="button"
                onClick={() =>
                  onView({ ...view, id: s.id, drill: null, collapsed: new Set(), grain: null })
                }
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

        <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
          {canRollUp(calendar) && (
            <Segmented<Grain>
              label="Columns"
              value={grain}
              onChange={(g) => onView({ ...view, grain: g })}
              options={
                calendar === "monthly"
                  ? [
                      ["Monthly", "native"],
                      ["Quarterly", "quarter"],
                      ["Annual", "year"],
                    ]
                  : [
                      ["Quarterly", "native"],
                      ["Annual", "year"],
                    ]
              }
            />
          )}

          {groups.size > 0 && (
            <button
              type="button"
              onClick={() =>
                onView({
                  ...view,
                  collapsed:
                    view.collapsed.size > 0 ? new Set() : new Set(groups.keys()),
                })
              }
              className="rounded-md px-2 py-1 text-xs text-muted transition-colors hover:bg-surface-raised hover:text-primary"
            >
              {view.collapsed.size > 0 ? "Expand all" : "Collapse detail"}
            </button>
          )}

          <div className="ml-auto flex items-center gap-1">
            <CopyButton label="Copy TSV" text={() => toDelimited(table(), "\t")} />
            <ToolButton
              label="CSV"
              title="Download this statement as CSV"
              showLabel
              icon={<Download className="size-3.5" />}
              onClick={() =>
                downloadText(
                  `cfdl-${active.id.replace(/[^a-z0-9._-]+/gi, "-")}.csv`,
                  toDelimited(table(), ","),
                  "text/csv",
                )
              }
            />
          </div>
        </div>

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
      </div>

      <div className="@container mx-4 min-h-0 flex-1 overflow-auto rounded-lg border border-default">
        <table className="w-full border-collapse text-xs">
          <thead className="sticky top-0 z-30">
            <tr>
              <th
                className={cn(
                  "sticky left-0 z-40 border-b border-default bg-surface-raised px-3 py-2 text-left font-medium text-secondary",
                  LABEL_COL,
                )}
              >
                <div className="flex items-baseline gap-1.5 overflow-hidden">
                  <span className="truncate">{active.label}</span>
                  {currency ? <span className="font-normal text-muted">({currency})</span> : null}
                </div>
              </th>
              <th
                className={cn(
                  "sticky z-40 whitespace-nowrap border-b border-l border-default bg-surface-raised px-3 py-2 text-right font-medium text-secondary",
                  TOTAL_LEFT,
                )}
              >
                Total
              </th>
              {cols.map((c) => (
                <th
                  key={c.label}
                  className="whitespace-nowrap border-b border-default bg-surface-raised px-3 py-2 text-right font-normal text-muted"
                >
                  {c.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {active.rows.map((row, i) => {
              if (hidden.has(i)) return null;
              if (row.kind === "spacer") {
                return (
                  <tr key={i}>
                    <td colSpan={cols.length + 2} className="h-3" />
                  </tr>
                );
              }
              const emphasis =
                row.kind === "subtotal" || row.kind === "ratio" || row.kind === "residual";
              const canDrill = Boolean(row.streams?.length);
              const group = groups.get(i);
              const isCollapsed = view.collapsed.has(i);
              const sign = row.display_sign ?? 1;
              const bg = emphasis || view.drill === i ? "bg-surface-raised" : "bg-surface-page";

              return (
                <Fragment key={i}>
                  <tr
                    onClick={() =>
                      canDrill && onView({ ...view, drill: view.drill === i ? null : i })
                    }
                    className={cn(
                      "group border-b border-default/50",
                      emphasis && "font-medium text-primary",
                      row.kind === "residual" && "text-warn",
                      canDrill && "cursor-pointer",
                      "hover:bg-surface-raised",
                      view.drill === i && "bg-surface-raised",
                    )}
                  >
                    <td
                      className={cn(
                        "sticky left-0 z-10 px-3 py-1.5 text-left group-hover:bg-surface-raised",
                        LABEL_COL,
                        bg,
                      )}
                      style={{ paddingLeft: `${0.75 + row.depth * 0.85}rem` }}
                      title={row.label}
                    >
                      {/* Truncation lives on the inner span, not the cell. A
                          `white-space: nowrap` cell has a min-content width of
                          the whole label, which an auto-layout table honours —
                          the column then grows past the width the sticky Total
                          offset was computed from, and the two overlap. */}
                      <div className="flex items-center overflow-hidden">
                        {group ? (
                          <button
                            type="button"
                            onClick={(e) => {
                              e.stopPropagation();
                              const next = new Set(view.collapsed);
                              if (next.has(i)) next.delete(i);
                              else next.add(i);
                              onView({ ...view, collapsed: next });
                            }}
                            aria-expanded={!isCollapsed}
                            aria-label={`${isCollapsed ? "Expand" : "Collapse"} ${row.label ?? "section"}`}
                            className="mr-1 -ml-1 shrink-0 rounded p-0.5 text-muted hover:bg-surface-sunken hover:text-primary"
                          >
                            <ChevronRight
                              className={cn("size-3 transition-transform", !isCollapsed && "rotate-90")}
                            />
                          </button>
                        ) : null}
                        <span className="truncate">{row.label}</span>
                        {isCollapsed && group ? (
                          <span className="ml-1.5 shrink-0 text-[10px] text-muted">
                            +{group[1] - group[0] + 1}
                          </span>
                        ) : null}
                        {canDrill && (
                          <span className="ml-1.5 shrink-0 text-[10px] opacity-40">
                            {row.streams!.length}
                          </span>
                        )}
                      </div>
                    </td>
                    <td
                      className={cn(
                        "sticky z-10 whitespace-nowrap border-l border-default/50 px-3 py-1.5 text-right tabular-nums group-hover:bg-surface-raised",
                        TOTAL_LEFT,
                        bg,
                      )}
                    >
                      {row.total === undefined ? "" : fmtSigned(row.total * sign)}
                    </td>
                    {cols.map((c) => {
                      const n = aggregate(row, c.idx);
                      return (
                        <td
                          key={c.label}
                          className="whitespace-nowrap px-3 py-1.5 text-right tabular-nums text-secondary"
                        >
                          {n === undefined
                            ? "—"
                            : row.kind === "ratio"
                              ? n.toFixed(4)
                              : fmtSigned(n * sign)}
                        </td>
                      );
                    })}
                  </tr>

                  {view.drill === i && row.streams?.length ? (
                    <tr className="border-b border-default/50 bg-surface-sunken">
                      <td colSpan={cols.length + 2} className="px-3 py-2">
                        <p className="mb-1 text-[11px] text-muted">
                          {row.label} draws from {row.streams.length} stream
                          {row.streams.length === 1 ? "" : "s"}
                        </p>
                        <ul className="flex flex-wrap gap-x-4 gap-y-0.5">
                          {row.streams.map((s) => (
                            <li key={s} className="font-mono text-[11px] text-secondary">
                              {s}
                            </li>
                          ))}
                        </ul>
                      </td>
                    </tr>
                  ) : null}
                </Fragment>
              );
            })}
          </tbody>
        </table>
      </div>

      {recon && (
        // Pinned outside the scroll container: this is the line that says
        // whether the statement adds up, and it used to sit past 6,500px of
        // table where nobody scrolled to find it.
        <div className="flex shrink-0 flex-wrap items-center gap-x-6 gap-y-1 border-t border-subtle px-4 py-2 text-xs text-muted">
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

/** Only calendars with a defined number of periods per year can be rolled up. */
function canRollUp(calendar: string): boolean {
  return calendar === "monthly" || calendar === "quarterly";
}

interface ColGroup {
  label: string;
  idx: number[];
}

/**
 * Period columns chunked to the requested grain.
 *
 * Labels are the engine's, and are parsed rather than recomputed: the engine
 * knows which periods a statement's columns correspond to and the UI does not.
 * A label that doesn't parse falls back to naming the chunk by its endpoints,
 * which is wrong-looking rather than silently wrong.
 */
function groupColumns(labels: string[], calendar: string, grain: Grain): ColGroup[] {
  const size =
    grain === "native" || !canRollUp(calendar)
      ? 1
      : calendar === "monthly"
        ? grain === "quarter"
          ? 3
          : 12
        : grain === "quarter"
          ? 1
          : 4;

  if (size <= 1) return labels.map((label, i) => ({ label, idx: [i] }));

  const out: ColGroup[] = [];
  for (let i = 0; i < labels.length; i += size) {
    const idx = Array.from({ length: Math.min(size, labels.length - i) }, (_, k) => i + k);
    out.push({ label: chunkLabel(labels, idx, grain), idx });
  }
  return out;
}

function chunkLabel(labels: string[], idx: number[], grain: Grain): string {
  const first = labels[idx[0]] ?? "";
  const m = /^(\d{4})(?:-(\d{2}))?/.exec(first);
  if (!m) return idx.length === 1 ? first : `${first}–${labels[idx[idx.length - 1]] ?? ""}`;
  if (grain === "year") return m[1];
  const month = Number(m[2] ?? "1");
  return `${m[1]} Q${Math.floor((month - 1) / 3) + 1}`;
}

/**
 * One column's value for a row.
 *
 * Flows sum across the chunk. Ratios cannot — a DSCR is not the sum of three
 * monthly DSCRs — so they take the last period in the chunk, which is the
 * convention a quarterly debt test already uses.
 */
function aggregate(row: StatementRow, idx: number[]): number | undefined {
  if (row.kind === "ratio") {
    for (let k = idx.length - 1; k >= 0; k--) {
      const n = toNumber(row.values?.[idx[k]] ?? undefined);
      if (n !== undefined) return n;
    }
    return undefined;
  }
  let sum = 0;
  let seen = false;
  for (const i of idx) {
    const n = toNumber(row.values?.[i] ?? undefined);
    if (n !== undefined) {
      sum += n;
      seen = true;
    }
  }
  return seen ? sum : undefined;
}

/**
 * Which rows each subtotal owns.
 *
 * A pro forma puts detail ABOVE its subtotal, so a subtotal's children are the
 * run of deeper rows immediately preceding it. Spacers end the run — they are
 * how a statement separates sections.
 */
function subtotalGroups(rows: StatementRow[]): Map<number, [number, number]> {
  const out = new Map<number, [number, number]>();
  rows.forEach((row, i) => {
    if (row.kind !== "subtotal") return;
    let start = i;
    for (let j = i - 1; j >= 0; j--) {
      const r = rows[j];
      if (r.kind === "spacer" || r.depth <= row.depth) break;
      start = j;
    }
    if (start < i) out.set(i, [start, i - 1]);
  });
  return out;
}

function hiddenRows(
  groups: Map<number, [number, number]>,
  collapsed: Set<number>,
): Set<number> {
  const hidden = new Set<number>();
  for (const i of collapsed) {
    const g = groups.get(i);
    if (!g) continue;
    for (let j = g[0]; j <= g[1]; j++) hidden.add(j);
  }
  return hidden;
}

/** The currency the statement is denominated in, shown once in the header. */
function statementCurrency(statement: Statement): string | undefined {
  for (const row of statement.rows) {
    for (const v of row.values ?? []) {
      const c = currencyOf((v ?? undefined) as MoneyOrNumber | undefined);
      if (c) return c;
    }
  }
  return undefined;
}

/** Thousands-separated, two decimals, parenthesised when negative. */
function fmtSigned(n: number): string {
  const s = Math.abs(n).toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
  return n < 0 ? `(${s})` : s;
}
