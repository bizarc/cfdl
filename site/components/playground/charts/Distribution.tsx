"use client";

import { formatCompact } from "@/lib/playground/results";

/**
 * Monte Carlo outcome distribution: a histogram of per-trial values with the
 * P5–P95 band shaded and the median marked. The point estimate and the shape
 * around it, in one picture — which is the whole pitch.
 */
export function Distribution({
  counts,
  min,
  max,
  p05,
  p50,
  p95,
  height = 180,
}: {
  counts: number[];
  min: number;
  max: number;
  p05?: number;
  p50?: number;
  p95?: number;
  height?: number;
}) {
  if (counts.length === 0) return null;

  const width = 720;
  const pad = { top: 10, right: 12, bottom: 26, left: 12 };
  const plotW = width - pad.left - pad.right;
  const plotH = height - pad.top - pad.bottom;
  const peak = Math.max(...counts);
  const barW = plotW / counts.length;

  const toX = (value: number) =>
    pad.left + ((value - min) / (max - min || 1)) * plotW;

  // Label-collision guards for endpoint labels vs the median label.
  const span = max - min || 1;
  const nearMin = p50 !== undefined && (p50 - min) / span < 0.2;
  const nearMax = p50 !== undefined && (max - p50) / span < 0.2;

  const inBand = (i: number) => {
    if (p05 === undefined || p95 === undefined) return true;
    const center = min + ((i + 0.5) / counts.length) * (max - min);
    return center >= p05 && center <= p95;
  };

  return (
    <div className="w-full">
      <svg
        viewBox={`0 0 ${width} ${height}`}
        className="w-full"
        role="img"
        aria-label="Distribution of Monte Carlo outcomes"
      >
        {counts.map((c, i) => {
          const h = peak === 0 ? 0 : (c / peak) * plotH;
          return (
            <rect
              key={i}
              x={pad.left + i * barW + 0.5}
              y={pad.top + plotH - h}
              width={Math.max(1, barW - 1)}
              height={h}
              rx="1"
              fill={
                inBand(i)
                  ? "var(--cfdl-chart-series-2)"
                  : "var(--cfdl-chart-series-1)"
              }
              opacity={inBand(i) ? 0.9 : 0.4}
            />
          );
        })}

        {p50 !== undefined && (
          <line
            x1={toX(p50)}
            x2={toX(p50)}
            y1={pad.top}
            y2={pad.top + plotH}
            stroke="var(--cfdl-text-primary)"
            strokeWidth="1.5"
          />
        )}

        {/* With a bimodal outcome the median sits near an endpoint, so drop
            the endpoint label it would collide with. */}
        {!nearMin && (
          <text x={pad.left} y={height - 8} fontSize="10" fill="var(--cfdl-chart-axis)" fontFamily="var(--font-mono)">
            {formatCompact(min)}
          </text>
        )}
        {p50 !== undefined && (
          <text
            x={Math.min(Math.max(toX(p50), pad.left + 40), width - pad.right - 40)}
            y={height - 8}
            textAnchor="middle"
            fontSize="10"
            fill="var(--cfdl-text-secondary)"
            fontFamily="var(--font-mono)"
          >
            median {formatCompact(p50)}
          </text>
        )}
        {!nearMax && (
          <text
            x={width - pad.right}
            y={height - 8}
            textAnchor="end"
            fontSize="10"
            fill="var(--cfdl-chart-axis)"
            fontFamily="var(--font-mono)"
          >
            {formatCompact(max)}
          </text>
        )}
      </svg>

      <p className="mt-1 text-xs text-muted">
        Shaded band spans P5–P95
        {p05 !== undefined && p95 !== undefined
          ? ` (${formatCompact(p05)} – ${formatCompact(p95)})`
          : ""}
        .
      </p>
    </div>
  );
}
