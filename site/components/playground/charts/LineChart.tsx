"use client";

import { useId, useState } from "react";
import { formatCompact } from "@/lib/playground/results";

export interface LineSeries {
  name: string;
  values: number[];
  /** 1-based index into the --cfdl-chart-series-* token scale. */
  colorIndex: number;
}

/**
 * Hand-rolled time-series chart.
 *
 * Two chart shapes with known data, themed by CSS variables, is not worth a
 * charting dependency — this stays under 150 lines and inherits light/dark
 * from the design tokens for free.
 */
export function LineChart({
  series,
  labels,
  height = 220,
}: {
  series: LineSeries[];
  labels: string[];
  height?: number;
}) {
  const clipId = useId();
  const [hover, setHover] = useState<number | null>(null);

  const width = 720;
  const pad = { top: 12, right: 12, bottom: 24, left: 52 };
  const plotW = width - pad.left - pad.right;
  const plotH = height - pad.top - pad.bottom;

  const all = series.flatMap((s) => s.values);
  if (all.length === 0) return null;

  const rawMin = Math.min(...all, 0);
  const rawMax = Math.max(...all, 0);
  const span = rawMax - rawMin || 1;
  const min = rawMin - span * 0.05;
  const max = rawMax + span * 0.05;

  const n = Math.max(...series.map((s) => s.values.length));
  const x = (i: number) => pad.left + (n === 1 ? plotW / 2 : (i / (n - 1)) * plotW);
  const y = (v: number) => pad.top + plotH - ((v - min) / (max - min)) * plotH;

  const ticks = [max, (max + min) / 2, min];
  const zeroY = min <= 0 && max >= 0 ? y(0) : null;

  return (
    <div className="w-full">
      <svg
        viewBox={`0 0 ${width} ${height}`}
        className="w-full"
        role="img"
        aria-label={`Time series: ${series.map((s) => s.name).join(", ")}`}
        onMouseLeave={() => setHover(null)}
      >
        <defs>
          <clipPath id={clipId}>
            <rect x={pad.left} y={pad.top} width={plotW} height={plotH} />
          </clipPath>
        </defs>

        {ticks.map((t, i) => (
          <g key={i}>
            <line
              x1={pad.left}
              x2={width - pad.right}
              y1={y(t)}
              y2={y(t)}
              stroke="var(--cfdl-chart-grid)"
              strokeWidth="1"
            />
            <text
              x={pad.left - 8}
              y={y(t) + 3}
              textAnchor="end"
              fontSize="10"
              fill="var(--cfdl-chart-axis)"
              fontFamily="var(--font-mono)"
            >
              {formatCompact(t)}
            </text>
          </g>
        ))}

        {zeroY !== null && (
          <line
            x1={pad.left}
            x2={width - pad.right}
            y1={zeroY}
            y2={zeroY}
            stroke="var(--cfdl-chart-axis)"
            strokeWidth="1"
            strokeDasharray="3 3"
          />
        )}

        <g clipPath={`url(#${clipId})`}>
          {series.map((s) => (
            <polyline
              key={s.name}
              points={s.values.map((v, i) => `${x(i)},${y(v)}`).join(" ")}
              fill="none"
              stroke={`var(--cfdl-chart-series-${s.colorIndex})`}
              strokeWidth="1.75"
              strokeLinejoin="round"
              strokeLinecap="round"
            />
          ))}
        </g>

        {hover !== null && (
          <line
            x1={x(hover)}
            x2={x(hover)}
            y1={pad.top}
            y2={pad.top + plotH}
            stroke="var(--cfdl-chart-axis)"
            strokeWidth="1"
          />
        )}
        {hover !== null &&
          series.map((s) =>
            s.values[hover] === undefined ? null : (
              <circle
                key={s.name}
                cx={x(hover)}
                cy={y(s.values[hover])}
                r="3"
                fill={`var(--cfdl-chart-series-${s.colorIndex})`}
              />
            ),
          )}

        {/* Invisible hit targets so hover works without per-point markup. */}
        {Array.from({ length: n }).map((_, i) => (
          <rect
            key={i}
            x={x(i) - plotW / n / 2}
            y={pad.top}
            width={plotW / n}
            height={plotH}
            fill="transparent"
            onMouseEnter={() => setHover(i)}
          />
        ))}

        <text x={pad.left} y={height - 6} fontSize="10" fill="var(--cfdl-chart-axis)" fontFamily="var(--font-mono)">
          {labels[0]}
        </text>
        <text
          x={width - pad.right}
          y={height - 6}
          textAnchor="end"
          fontSize="10"
          fill="var(--cfdl-chart-axis)"
          fontFamily="var(--font-mono)"
        >
          {labels[labels.length - 1]}
        </text>
      </svg>

      <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
        {series.map((s) => (
          <span key={s.name} className="inline-flex items-center gap-1.5 text-secondary">
            <span
              className="inline-block h-2 w-2 rounded-full"
              style={{ background: `var(--cfdl-chart-series-${s.colorIndex})` }}
            />
            <span className="font-mono">{s.name}</span>
            {hover !== null && s.values[hover] !== undefined ? (
              <span className="font-mono tabular-nums text-primary">
                {formatCompact(s.values[hover])}
              </span>
            ) : null}
          </span>
        ))}
        {hover !== null ? (
          <span className="font-mono text-muted">{labels[hover]}</span>
        ) : null}
      </div>
    </div>
  );
}
