"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { ArrowRight, Lightbulb } from "lucide-react";
import { cn } from "@/lib/cn";
import { Checkbox, Field, Input, Select, Slider } from "@/components/ds/Field";
import { Disclosure } from "@/components/ds/Tabs";
import examples from "@/content/playground-examples.json";
import type { RunConfig } from "@/lib/playground/protocol";

export interface PlaygroundExample {
  id: string;
  title: string;
  category: string;
  order: number;
  summary: string;
  tryThis: string;
  docsHref: string;
  source: string;
  root: string;
  pack?: string;
  config?: RunConfig;
  files: Record<string, string>;
}

export const EXAMPLES = examples as PlaygroundExample[];

/** Tutorial entries read as a numbered path; the rest are a reference shelf. */
const CATEGORY_ORDER = ["Tutorial", "Real deals", "Stochastic"];

export function Sidebar({
  activeId,
  onPick,
  config,
  onConfigChange,
  pack,
  onPackChange,
  modelDeclaredPack,
}: {
  activeId: string | null;
  onPick: (example: PlaygroundExample) => void;
  config: RunConfig;
  onConfigChange: (config: RunConfig) => void;
  pack: string;
  onPackChange: (pack: string) => void;
  modelDeclaredPack?: string;
}) {
  const [configOpen, setConfigOpen] = useState(false);

  const grouped = useMemo(() => {
    const map = new Map<string, PlaygroundExample[]>();
    for (const example of EXAMPLES) {
      const list = map.get(example.category) ?? [];
      list.push(example);
      map.set(example.category, list);
    }
    return CATEGORY_ORDER.filter((c) => map.has(c)).map(
      (c) => [c, map.get(c)!] as const,
    );
  }, []);

  const mc = config.monte_carlo;
  const rate = config.deterministic?.annual_discount_rate ?? 0;

  const summary = `${(rate * 100).toFixed(1)}% · ${mc ? `${mc.trial_count} trials` : "deterministic"}`;

  return (
    <div className="flex h-full flex-col gap-5 overflow-y-auto p-3">
      <Disclosure
        open={configOpen}
        onOpenChange={setConfigOpen}
        title="Run config"
        summary={summary}
      >
        <Slider
          label="Discount rate (annual)"
          value={rate}
          onValueChange={(value) =>
            onConfigChange({
              ...config,
              deterministic: { ...config.deterministic, annual_discount_rate: value },
            })
          }
          min={0}
          max={0.25}
          step={0.005}
          format={(v) => `${(v * 100).toFixed(1)}%`}
        />

        <Checkbox
          label="Monte Carlo"
          checked={Boolean(mc)}
          onCheckedChange={(enabled) =>
            onConfigChange({
              ...config,
              monte_carlo: enabled ? { trial_count: 500, seed: 42, ...mc } : undefined,
            })
          }
        />

        {mc ? (
          <div className="grid grid-cols-2 gap-2">
            <Field label="Trials">
              <Input
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
              />
            </Field>
            <Field label="Seed" hint="Same seed ⇒ same results">
              <Input
                type="number"
                value={mc.seed}
                onChange={(e) =>
                  onConfigChange({
                    ...config,
                    monte_carlo: { ...mc, seed: Number(e.target.value) || 0 },
                  })
                }
              />
            </Field>
          </div>
        ) : null}

        <Field
          label="Pack (domain metrics)"
          hint={
            modelDeclaredPack
              ? `This model declares the ${modelDeclaredPack} pack.`
              : "This model doesn't use a pack."
          }
        >
          <Select value={pack} onChange={(e) => onPackChange(e.target.value)}>
            <option value="">none</option>
            {["cre", "energy", "credit", "opco"].map((p) => (
              <option key={p} value={p}>
                {p}
              </option>
            ))}
          </Select>
        </Field>
      </Disclosure>

      {grouped.map(([category, items]) => (
        <section key={category}>
          <h2 className="mb-2 px-1 text-xs font-semibold uppercase tracking-wider text-muted">
            {category === "Tutorial" ? "Start here" : category}
          </h2>
          <ul className="space-y-1">
            {items.map((example, index) => {
              const active = activeId === example.id;
              return (
                <li key={example.id}>
                  <button
                    type="button"
                    onClick={() => onPick(example)}
                    className={cn(
                      "w-full rounded-md px-2 py-2 text-left transition-colors",
                      active ? "bg-accent-soft" : "hover:bg-surface-sunken",
                    )}
                  >
                    <span className="flex items-baseline gap-2">
                      {category === "Tutorial" ? (
                        <span
                          className={cn(
                            "flex h-4 w-4 shrink-0 items-center justify-center rounded-full text-[10px] font-semibold tabular-nums",
                            active
                              ? "bg-accent text-accent-fg"
                              : "bg-surface-sunken text-muted",
                          )}
                        >
                          {index + 1}
                        </span>
                      ) : null}
                      <span
                        className={cn(
                          "text-xs font-medium",
                          active ? "text-accent-text" : "text-primary",
                        )}
                      >
                        {example.title}
                      </span>
                    </span>
                    <span className="mt-1 block text-[11px] leading-snug text-muted">
                      {example.summary}
                    </span>
                  </button>

                  {active ? (
                    <div className="mt-1 space-y-2 rounded-md border border-default bg-surface-raised p-2">
                      <p className="flex gap-1.5 text-[11px] leading-snug text-secondary">
                        <Lightbulb className="mt-px h-3 w-3 shrink-0 text-accent-text" />
                        {example.tryThis}
                      </p>
                      <Link
                        href={example.docsHref}
                        className="inline-flex items-center gap-1 text-[11px] font-medium text-accent-text hover:underline"
                      >
                        Read the guide
                        <ArrowRight className="h-3 w-3" />
                      </Link>
                    </div>
                  ) : null}
                </li>
              );
            })}
          </ul>
        </section>
      ))}
    </div>
  );
}
