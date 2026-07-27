"use client";

import { useEffect, useRef, useState } from "react";
import { heroResults } from "./hero-demo-data";
import { cn } from "@/lib/cn";

const money = (n: number) =>
  n.toLocaleString("en-US", { style: "currency", currency: "USD", maximumFractionDigits: 0 });

/**
 * The pitch, animated: a deterministic NPV lands first, then the same model's
 * distribution fills in behind it. Numbers come from a real engine run
 * (see hero-demo-data.ts) — no wasm on the landing page.
 */
export function HeroDemo({ codeHtml }: { codeHtml: string }) {
  const [phase, setPhase] = useState<"code" | "point" | "dist">("code");
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Readers who ask for reduced motion get the finished state immediately;
    // the global reduced-motion rule already collapses the transitions.
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const t1 = setTimeout(() => setPhase("point"), reduced ? 0 : 900);
    const t2 = setTimeout(() => setPhase("dist"), reduced ? 0 : 2000);
    return () => {
      clearTimeout(t1);
      clearTimeout(t2);
    };
  }, []);

  const { monteCarlo: mc, deterministic } = heroResults;
  const peak = Math.max(...mc.histogram);

  return (
    <div ref={ref} className="grid gap-4">
      <div
        className="overflow-hidden rounded-lg border border-default bg-surface-code"
        aria-label="Example CFDL model"
      >
        <div className="flex items-center gap-2 border-b border-subtle px-4 py-2">
          <span className="font-mono text-xs text-muted">solar-ppa.cfdl</span>
        </div>
        <div
          className="overflow-x-auto p-4 font-mono text-[12.5px] leading-relaxed [&_pre]:bg-transparent"
          dangerouslySetInnerHTML={{ __html: codeHtml }}
        />
      </div>

      <div className="grid gap-3 rounded-lg border border-default bg-surface-raised p-5">
        <div className="flex items-baseline justify-between gap-4">
          <span className="text-xs font-medium uppercase tracking-wider text-muted">
            NPV @ 8%
          </span>
          <span
            className={cn(
              "font-mono text-2xl font-semibold tabular-nums text-primary transition-all duration-500",
              phase === "code" ? "opacity-0 translate-y-1" : "opacity-100 translate-y-0",
            )}
          >
            {money(deterministic.npv)}
          </span>
        </div>

        <div
          className={cn(
            "grid gap-3 transition-all duration-700",
            phase === "dist" ? "opacity-100" : "opacity-0",
          )}
        >
          <div className="flex items-end gap-[3px]" aria-hidden="true">
            {mc.histogram.map((count, i) => (
              <span
                key={i}
                className="flex-1 rounded-t-[2px] transition-[height] duration-700 ease-out"
                style={{
                  height: phase === "dist" ? `${Math.max(4, (count / peak) * 64)}px` : "4px",
                  transitionDelay: `${i * 24}ms`,
                  background:
                    i >= 2 && i <= 12
                      ? "var(--cfdl-chart-series-2)"
                      : "var(--cfdl-chart-series-1)",
                  opacity: i >= 2 && i <= 12 ? 0.9 : 0.45,
                }}
              />
            ))}
          </div>

          <dl className="grid grid-cols-3 gap-3 border-t border-subtle pt-3 text-center">
            {[
              ["P5", mc.p05],
              ["Median", mc.p50],
              ["P95", mc.p95],
            ].map(([label, value]) => (
              <div key={label as string}>
                <dt className="text-[11px] uppercase tracking-wider text-muted">
                  {label as string}
                </dt>
                <dd className="mt-0.5 font-mono text-sm tabular-nums text-secondary">
                  {money(value as number)}
                </dd>
              </div>
            ))}
          </dl>

          <p className="text-xs text-muted">
            {mc.trials.toLocaleString()} seeded trials (seed {mc.seed}) — same
            model file, reproducible byte-for-byte.
          </p>
        </div>
      </div>
    </div>
  );
}
