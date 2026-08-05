"use client";

import { useId, useState } from "react";

export interface CashFlowBand {
  name: string;
  values: number[];
  /** 1-based index into the --cfdl-chart-series-* token scale. */
  colorIndex: number;
}

/**
 * Cash in and out, per period, as stacked bars with a net line over them.
 *
 * A LINE CHART WAS THE WRONG SHAPE. Cash flows are discrete events on a grid,
 * not samples of a continuous quantity, and drawing them as lines implied
 * interpolation between periods that does not exist. Worse, six lines that all
 * hug zero except one spike — an exit sale is routinely 70x a month's rent —
 * read as five flat lines and a cliff.
 *
 * Stacking fixes the comparison the reader actually wants. Inflows stack up
 * from zero and outflows stack down, so the height of each side is the period's
 * gross activity and the net line is where it lands. A tall bar in one period
 * now reads as "something large happened here", which is true, instead of
 * flattening every other period into the axis.
 *
 * SCALE IS STILL REAL. A reversion dwarfs the operating flows no matter how it
 * is drawn, and pretending otherwise by clipping or log-scaling would misstate
 * the numbers. What this does is make the small periods legible ANYWAY: each
 * bar is drawn with a minimum visible height, so a 45k month next to a 3.2M
 * sale is still a bar you can see and hover, rather than a sub-pixel sliver.
 */
export function CashFlowChart({
  bands,
  net,
  labels,
  height = 260,
}: {
  bands: CashFlowBand[];
  net?: number[];
  labels: string[];
  height?: number;
}) {
  const clipId = useId();
  const [hover, setHover] = useState<number | null>(null);

  const width = 720;
  const pad = { top: 12, right: 12, bottom: 24, left: 60 };
  const plotW = width - pad.left - pad.right;
  const plotH = height - pad.top - pad.bottom;

  const n = Math.max(...bands.map((b) => b.values.length), net?.length ?? 0, 0);
  if (n === 0) return null;

  // Each period's stack runs from the sum of its negatives to the sum of its
  // positives. The extent is those two, never the individual values, or the
  // axis would not contain the bars it is scaling.
  let lo = 0;
  let hi = 0;
  for (let i = 0; i < n; i++) {
    let up = 0;
    let down = 0;
    for (const b of bands) {
      const v = b.values[i] ?? 0;
      if (v >= 0) up += v;
      else down += v;
    }
    hi = Math.max(hi, up, net?.[i] ?? 0);
    lo = Math.min(lo, down, net?.[i] ?? 0);
  }
  const span = hi - lo || 1;
  const min = lo - span * 0.05;
  const max = hi + span * 0.05;

  const y = (v: number) => pad.top + plotH - ((v - min) / (max - min)) * plotH;
  const slot = plotW / n;
  const barW = Math.max(1, Math.min(slot * 0.7, 18));
  const x = (i: number) => pad.left + slot * i + slot / 2;
  const zeroY = y(0);

  const fmt = (v: number) =>
    Math.abs(v) >= 1e6
      ? `${(v / 1e6).toFixed(1)}M`
      : Math.abs(v) >= 1e3
        ? `${(v / 1e3).toFixed(0)}k`
        : v.toFixed(0);

  const netPath = net
    ? net.map((v, i) => `${i === 0 ? "M" : "L"}${x(i)},${y(v)}`).join(" ")
    : null;

  return (
    <div className="w-full">
      <svg
        viewBox={`0 0 ${width} ${height}`}
        className="w-full"
        role="img"
        aria-label={`Cash flow per period: ${bands.map((b) => b.name).join(", ")}`}
        onMouseLeave={() => setHover(null)}
      >
        <defs>
          <clipPath id={clipId}>
            <rect x={pad.left} y={pad.top} width={plotW} height={plotH} />
          </clipPath>
        </defs>

        {[max, (max + min) / 2, min].map((t, i) => (
          <g key={i}>
            <line
              x1={pad.left}
              x2={width - pad.right}
              y1={y(t)}
              y2={y(t)}
              stroke="var(--cfdl-border-subtle)"
              strokeWidth={1}
            />
            <text
              x={pad.left - 8}
              y={y(t) + 3}
              textAnchor="end"
              fontSize={9}
              fill="var(--cfdl-text-muted)"
            >
              {fmt(t)}
            </text>
          </g>
        ))}

        <line
          x1={pad.left}
          x2={width - pad.right}
          y1={zeroY}
          y2={zeroY}
          stroke="var(--cfdl-border-default)"
          strokeWidth={1}
        />

        <g clipPath={`url(#${clipId})`}>
          {Array.from({ length: n }, (_, i) => {
            let up = 0;
            let down = 0;
            return (
              <g key={i} opacity={hover === null || hover === i ? 1 : 0.35}>
                {bands.map((b) => {
                  const v = b.values[i] ?? 0;
                  if (v === 0) return null;
                  const base = v >= 0 ? up : down;
                  const top = base + v;
                  if (v >= 0) up = top;
                  else down = top;
                  const yA = y(base);
                  const yB = y(top);
                  // A minimum height keeps a small period visible beside a
                  // reversion instead of collapsing to nothing.
                  const h = Math.max(Math.abs(yA - yB), 1);
                  return (
                    <rect
                      key={b.name}
                      x={x(i) - barW / 2}
                      y={Math.min(yA, yB)}
                      width={barW}
                      height={h}
                      fill={`var(--cfdl-chart-series-${b.colorIndex})`}
                    />
                  );
                })}
              </g>
            );
          })}

          {netPath && (
            <path
              d={netPath}
              fill="none"
              stroke="var(--cfdl-text-primary)"
              strokeWidth={1.5}
              strokeLinejoin="round"
            />
          )}
        </g>

        {/* Full-height hover targets: a bar can be one pixel wide, which is not
            a thing anyone can point at. */}
        {Array.from({ length: n }, (_, i) => (
          <rect
            key={i}
            x={pad.left + slot * i}
            y={pad.top}
            width={slot}
            height={plotH}
            fill="transparent"
            onMouseEnter={() => setHover(i)}
          />
        ))}

        {hover !== null && (
          <line
            x1={x(hover)}
            x2={x(hover)}
            y1={pad.top}
            y2={pad.top + plotH}
            stroke="var(--cfdl-text-muted)"
            strokeWidth={1}
            strokeDasharray="3 3"
          />
        )}

        <text x={pad.left} y={height - 6} fontSize={9} fill="var(--cfdl-text-muted)">
          {labels[0]}
        </text>
        <text
          x={width - pad.right}
          y={height - 6}
          textAnchor="end"
          fontSize={9}
          fill="var(--cfdl-text-muted)"
        >
          {labels[labels.length - 1]}
        </text>
      </svg>

      {hover !== null && (
        <div className="mt-1 flex flex-wrap items-baseline gap-x-3 gap-y-0.5 text-[11px]">
          <span className="font-medium text-secondary">{labels[hover]}</span>
          {bands
            .filter((b) => (b.values[hover] ?? 0) !== 0)
            .map((b) => (
              <span key={b.name} className="text-muted">
                <span
                  className="mr-1 inline-block size-1.5 rounded-full align-middle"
                  style={{ background: `var(--cfdl-chart-series-${b.colorIndex})` }}
                />
                {b.name} <span className="tabular-nums">{fmt(b.values[hover] ?? 0)}</span>
              </span>
            ))}
          {net && (
            <span className="font-medium text-primary">
              net <span className="tabular-nums">{fmt(net[hover] ?? 0)}</span>
            </span>
          )}
        </div>
      )}
    </div>
  );
}
