/**
 * Narrow, tolerant readers over the engine's Results JSON.
 *
 * Everything here treats missing sections as "not present" rather than
 * throwing: a model with no scenarios or no Monte Carlo is normal, and the
 * UI should show an empty state, not an error.
 */

export type MoneyOrNumber = number | { amount: number; currency?: string };

export interface SeriesIndex {
  calendar: string;
  start: string;
  periods: number;
}

export interface Series {
  index: SeriesIndex;
  values: MoneyOrNumber[];
}

export interface MetricSummary {
  type?: string;
  mean?: MoneyOrNumber;
  stdev?: MoneyOrNumber;
  min?: MoneyOrNumber;
  max?: MoneyOrNumber;
  p50?: MoneyOrNumber;
  [key: string]: unknown;
}

export interface Results {
  results_version?: string;
  model_hash?: string;
  engine?: { name?: string; version?: string };
  warnings?: string[];
  deterministic?: {
    status?: string;
    metrics?: Record<string, MoneyOrNumber>;
    series?: Record<string, Series>;
    annual_rollup?: { series?: Record<string, Series> };
  };
  scenarios?: {
    status?: string;
    summaries?: { name: string; metrics: Record<string, MoneyOrNumber> }[];
  };
  monte_carlo?: {
    status?: string;
    trials?: number;
    seed?: number;
    metrics?: Record<string, MetricSummary>;
    trial_summaries?: { trial: number; metrics: Record<string, MoneyOrNumber> }[];
  };
  domain_metrics?: { metrics?: Record<string, MoneyOrNumber> };
  statements?: StatementsSection;
}

/** A pack's declared statements, rendered against this run. */
export interface StatementsSection {
  pack?: string;
  statements?: Statement[];
}

export interface Statement {
  id: string;
  label: string;
  default?: boolean;
  /** Published rather than derived — see StatementGrain. */
  grain: StatementGrain;
  rows: StatementRow[];
  reconciliation?: {
    bottom_line?: number;
    model_total?: number;
    residual?: number;
  };
  diagnostics?: { code?: string; severity?: string; message?: string }[];
}

/**
 * The engine publishes column labels rather than the UI deriving them.
 *
 * It has to: an annual statement over a monthly model has ten values where the
 * model has 120, and nothing else in the document says WHICH ten periods those
 * are. `periodLabels` works off a SeriesIndex and cannot answer that.
 */
export interface StatementGrain {
  calendar: string;
  start: string;
  labels: string[];
}

export interface StatementRow {
  /** line | subtotal | ratio | spacer | residual */
  kind: string;
  label?: string;
  depth: number;
  /**
   * Rendering only. `values` is always the signed arithmetic quantity, so a
   * consumer that ignores this still adds up correctly; multiplying by it is
   * what turns a negative deduction into Argus's "less:" row of positives.
   */
  display_sign: number;
  values?: (MoneyOrNumber | null)[];
  total?: number;
  /** The streams this row drew from — the drill-down target. */
  streams?: string[];
}

export function toNumber(value: MoneyOrNumber | undefined): number | undefined {
  if (value === undefined || value === null) return undefined;
  return typeof value === "number" ? value : value.amount;
}

export function currencyOf(value: MoneyOrNumber | undefined): string | undefined {
  return typeof value === "object" && value ? value.currency : undefined;
}

/**
 * How a metric should read, inferred from its id.
 *
 * The engine returns bare ratios: an IRR of 0.19 means 19%, and printing it
 * raw (or printing 911.14 for a model with no upfront investment) is
 * unreadable. Money carries its own currency and needs no inference.
 */
function metricKind(id?: string): "percent" | "multiple" | "years" | "periods" | "plain" {
  if (!id) return "plain";
  if (/\.irr$|_rate$|^run\.annual_discount_rate$/.test(id)) return "percent";
  if (/\.moic$/.test(id)) return "multiple";
  if (/_years$/.test(id)) return "years";
  if (/_periods$/.test(id)) return "periods";
  return "plain";
}

export function formatValue(value: MoneyOrNumber | undefined, id?: string): string {
  const n = toNumber(value);
  if (n === undefined) return "—";

  const currency = currencyOf(value);
  if (currency) {
    return `${n.toLocaleString("en-US", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })} ${currency}`;
  }

  switch (metricKind(id)) {
    case "percent":
      return `${(n * 100).toLocaleString("en-US", {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      })}%`;
    case "multiple":
      return `${n.toFixed(2)}×`;
    case "years":
      return `${n.toFixed(2)} yr`;
    case "periods":
      return `${n.toLocaleString("en-US")} periods`;
    default: {
      const abs = Math.abs(n);
      const digits = abs < 1 ? 4 : 2;
      return n.toLocaleString("en-US", {
        minimumFractionDigits: Number.isInteger(n) ? 0 : digits,
        maximumFractionDigits: digits,
      });
    }
  }
}

/** Compact axis labels: 1.2M, -450k, 0.08. */
export function formatCompact(n: number): string {
  const abs = Math.abs(n);
  if (abs >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (abs >= 1_000) return `${Math.round(n / 1_000)}k`;
  if (abs >= 1) return n.toFixed(0);
  return n.toFixed(3);
}

/** Period start dates for a series index, for chart tooltips. */
export function periodLabels(index: SeriesIndex): string[] {
  const start = new Date(index.start + "T00:00:00Z");
  const labels: string[] = [];
  const monthly = index.calendar === "monthly";
  const quarterly = index.calendar === "quarterly";
  const step = monthly ? 1 : quarterly ? 3 : 12;

  for (let i = 0; i < index.periods; i++) {
    const d = new Date(start);
    if (index.calendar === "daily") d.setUTCDate(d.getUTCDate() + i);
    else d.setUTCMonth(d.getUTCMonth() + i * step);
    labels.push(
      index.calendar === "daily"
        ? d.toISOString().slice(0, 10)
        : d.toISOString().slice(0, 7),
    );
  }
  return labels;
}

/**
 * Percentiles computed from the raw trials.
 *
 * The engine's per-metric summary carries mean/stdev/min/max/p50; the wider
 * spread (P5/P25/P75/P95) is derived here from `trial_summaries` so the
 * distribution view can show a real band. Nearest-rank method.
 */
export function percentilesFromTrials(
  trials: { metrics: Record<string, MoneyOrNumber> }[],
  metric: string,
  ps: number[],
): Record<number, number> | null {
  const values = trials
    .map((t) => toNumber(t.metrics?.[metric]))
    .filter((v): v is number => v !== undefined)
    .sort((a, b) => a - b);

  if (values.length === 0) return null;

  const out: Record<number, number> = {};
  for (const p of ps) {
    const idx = Math.min(values.length - 1, Math.max(0, Math.ceil((p / 100) * values.length) - 1));
    out[p] = values[idx];
  }
  return out;
}

export function histogram(values: number[], bins = 24): { counts: number[]; min: number; max: number } {
  if (values.length === 0) return { counts: [], min: 0, max: 0 };
  const min = Math.min(...values);
  const max = Math.max(...values);
  const width = (max - min) / bins || 1;
  const counts = new Array(bins).fill(0);
  for (const v of values) {
    counts[Math.min(bins - 1, Math.floor((v - min) / width))] += 1;
  }
  return { counts, min, max };
}
