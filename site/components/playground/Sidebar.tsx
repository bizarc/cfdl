"use client";

import { useMemo } from "react";
import { cn } from "@/lib/cn";
import examples from "@/content/playground-examples.json";
import type { RunConfig } from "@/lib/playground/protocol";

export interface PlaygroundExample {
  id: string;
  title: string;
  description: string;
  category: string;
  source: string;
  root: string;
  pack?: string;
  config?: RunConfig;
  files: Record<string, string>;
}

export const EXAMPLES = examples as PlaygroundExample[];

export function Sidebar({
  activeId,
  onPick,
  config,
  onConfigChange,
  pack,
  onPackChange,
}: {
  activeId: string | null;
  onPick: (example: PlaygroundExample) => void;
  config: RunConfig;
  onConfigChange: (config: RunConfig) => void;
  pack: string;
  onPackChange: (pack: string) => void;
}) {
  const grouped = useMemo(() => {
    const map = new Map<string, PlaygroundExample[]>();
    for (const example of EXAMPLES) {
      const list = map.get(example.category) ?? [];
      list.push(example);
      map.set(example.category, list);
    }
    return [...map.entries()];
  }, []);

  const mc = config.monte_carlo;
  const rate = config.deterministic?.annual_discount_rate ?? 0;

  const setRate = (value: number) =>
    onConfigChange({
      ...config,
      deterministic: { ...config.deterministic, annual_discount_rate: value },
    });

  const setMonteCarlo = (enabled: boolean) =>
    onConfigChange({
      ...config,
      monte_carlo: enabled ? { trial_count: 500, seed: 42, ...mc } : undefined,
    });

  return (
    <div className="flex h-full flex-col gap-6 overflow-y-auto p-3">
      <section>
        <h2 className="mb-2 px-1 text-xs font-semibold uppercase tracking-wider text-muted">
          Run config
        </h2>
        <div className="space-y-3 rounded-lg border border-default p-3">
          <label className="block">
            <span className="text-xs text-secondary">Discount rate (annual)</span>
            <div className="mt-1 flex items-center gap-2">
              <input
                type="range"
                min={0}
                max={0.25}
                step={0.005}
                value={rate}
                onChange={(e) => setRate(Number(e.target.value))}
                className="flex-1 accent-[var(--cfdl-accent-solid)]"
              />
              <span className="w-12 text-right font-mono text-xs tabular-nums text-primary">
                {(rate * 100).toFixed(1)}%
              </span>
            </div>
          </label>

          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={Boolean(mc)}
              onChange={(e) => setMonteCarlo(e.target.checked)}
              className="accent-[var(--cfdl-accent-solid)]"
            />
            <span className="text-xs text-secondary">Monte Carlo</span>
          </label>

          {mc ? (
            <div className="grid grid-cols-2 gap-2">
              <label className="block">
                <span className="text-[11px] text-muted">Trials</span>
                <input
                  type="number"
                  min={1}
                  max={100000}
                  value={mc.trial_count}
                  onChange={(e) =>
                    onConfigChange({
                      ...config,
                      monte_carlo: { ...mc, trial_count: Number(e.target.value) || 1 },
                    })
                  }
                  className="mt-0.5 w-full rounded border border-default bg-surface-raised px-2 py-1 font-mono text-xs text-primary"
                />
              </label>
              <label className="block">
                <span className="text-[11px] text-muted">Seed</span>
                <input
                  type="number"
                  value={mc.seed}
                  onChange={(e) =>
                    onConfigChange({
                      ...config,
                      monte_carlo: { ...mc, seed: Number(e.target.value) || 0 },
                    })
                  }
                  className="mt-0.5 w-full rounded border border-default bg-surface-raised px-2 py-1 font-mono text-xs text-primary"
                />
              </label>
            </div>
          ) : null}

          <label className="block">
            <span className="text-xs text-secondary">Pack (domain metrics)</span>
            <select
              value={pack}
              onChange={(e) => onPackChange(e.target.value)}
              className="mt-1 w-full rounded border border-default bg-surface-raised px-2 py-1 font-mono text-xs text-primary"
            >
              <option value="">none</option>
              {["cre", "energy", "credit", "opco"].map((p) => (
                <option key={p} value={p}>
                  {p}
                </option>
              ))}
            </select>
          </label>
        </div>
      </section>

      <section>
        <h2 className="mb-2 px-1 text-xs font-semibold uppercase tracking-wider text-muted">
          Examples
        </h2>
        <div className="space-y-4">
          {grouped.map(([category, items]) => (
            <div key={category}>
              <p className="mb-1 px-1 text-[11px] font-medium text-muted">{category}</p>
              <ul className="space-y-px">
                {items.map((example) => (
                  <li key={example.id}>
                    <button
                      type="button"
                      onClick={() => onPick(example)}
                      className={cn(
                        "w-full rounded-md px-2 py-1.5 text-left transition-colors",
                        activeId === example.id
                          ? "bg-accent-soft"
                          : "hover:bg-surface-sunken",
                      )}
                    >
                      <span
                        className={cn(
                          "block text-xs font-medium",
                          activeId === example.id ? "text-accent-text" : "text-primary",
                        )}
                      >
                        {example.title}
                      </span>
                      <span className="mt-0.5 block text-[11px] leading-snug text-muted">
                        {example.description}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
