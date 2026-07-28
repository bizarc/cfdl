"use client";

import { useState } from "react";
import { Button } from "@/components/ds/Button";
import { Badge } from "@/components/ds/Badge";
import { Card, CardBody, CardTitle } from "@/components/ds/Card";
import { Dialog } from "@/components/ds/Dialog";
import { Checkbox, Field, Input, Select, Slider } from "@/components/ds/Field";
import { Disclosure, Tabs } from "@/components/ds/Tabs";
import { CodeActions } from "@/components/docs/CodeActions";

const SAMPLE_MODEL = `version 0.1
model "first-model"

time calendar monthly from 2026-01 for 24

entity legal company

assume growth = 0.02

stream revenue {
  entity  = company
  inflow
  currency = "USD"
  schedule monthly from 2026-01 to 2027-12
  amount   = 10000 * (1 + inputs.growth) ^ time.t
}
`;

interface TokenGroup {
  group: string;
  tokens: string[][];
}

interface Pattern {
  title: string;
  body: string;
}

export function DesignSystemShowcase({
  semanticTokens,
  patterns,
}: {
  semanticTokens: TokenGroup[];
  patterns: Pattern[];
}) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [checked, setChecked] = useState(true);
  const [rate, setRate] = useState(0.08);
  const [tab, setTab] = useState("metrics");
  const [disclosureOpen, setDisclosureOpen] = useState(false);
  const [name, setName] = useState("model.cfdl");

  return (
    <div className="mt-12 space-y-16">
      <Section
        title="Interaction patterns"
        description="The rules. Where a rule is machine-checkable, CI checks it."
      >
        <div className="grid gap-4 sm:grid-cols-2">
          {patterns.map((p) => (
            <Card key={p.title}>
              <CardTitle>{p.title}</CardTitle>
              <CardBody>{p.body}</CardBody>
            </Card>
          ))}
        </div>
      </Section>

      <Section
        title="Semantic tokens"
        description="Components reference these, never a raw colour. Both themes resolve from the same names — toggle the theme to see it."
      >
        <div className="space-y-6">
          {semanticTokens.map(({ group, tokens }) => (
            <div key={group}>
              <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted">
                {group}
              </h3>
              <div className="grid gap-2 sm:grid-cols-2">
                {tokens.map(([token, usage]) => (
                  <div
                    key={token}
                    className="flex items-center gap-3 rounded-md border border-default p-2"
                  >
                    <span
                      className="h-8 w-8 shrink-0 rounded border border-subtle"
                      style={{ background: `var(${token})` }}
                    />
                    <span className="min-w-0">
                      <code className="block truncate font-mono text-[11px] text-primary">
                        {token}
                      </code>
                      <span className="text-[11px] text-muted">{usage}</span>
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </Section>

      <Section title="Type scale" description="Inter for UI, JetBrains Mono for code and numbers.">
        <div className="space-y-3 rounded-lg border border-default p-5">
          <p className="text-4xl font-semibold tracking-tight text-primary">Display · 2.25rem</p>
          <p className="text-2xl font-semibold tracking-tight text-primary">Heading · 1.5rem</p>
          <p className="text-base text-secondary">Body · 1rem, 1.6 line height</p>
          <p className="text-sm text-secondary">Small · 0.875rem</p>
          <p className="text-xs text-muted">Caption · 0.75rem</p>
          <p className="font-mono text-sm tabular-nums text-primary">
            Mono · 1,954,958.82 USD
          </p>
        </div>
      </Section>

      <Section title="Buttons" description="Primary for the main action; one per view.">
        <div className="flex flex-wrap items-center gap-3 rounded-lg border border-default p-5">
          <Button>Primary</Button>
          <Button variant="secondary">Secondary</Button>
          <Button variant="ghost">Ghost</Button>
          <Button disabled>Disabled</Button>
          <Button size="sm">Small</Button>
          <Button size="lg">Large</Button>
        </div>
      </Section>

      <Section title="Badges" description="Status at a glance; tone carries the meaning.">
        <div className="flex flex-wrap items-center gap-3 rounded-lg border border-default p-5">
          <Badge>Neutral</Badge>
          <Badge tone="accent">Accent</Badge>
          <Badge tone="ok">Engine ready</Badge>
          <Badge tone="warn">Warning</Badge>
          <Badge tone="err">Failed</Badge>
        </div>
      </Section>

      <Section
        title="Form controls"
        description="Always wrapped in a Field so label, hint, and error stay attached to the control."
      >
        <div className="grid gap-4 rounded-lg border border-default p-5 sm:grid-cols-2">
          <Field label="File name" hint="Must end with .cfdl">
            <Input value={name} onChange={(e) => setName(e.target.value)} />
          </Field>
          <Field label="Invalid state" error="File names must end with .cfdl">
            <Input defaultValue="contracts.txt" invalid />
          </Field>
          <Field label="Pack">
            <Select defaultValue="cre">
              <option value="">none</option>
              <option value="cre">cre</option>
              <option value="energy">energy</option>
            </Select>
          </Field>
          <div className="flex items-end">
            <Checkbox label="Monte Carlo" checked={checked} onCheckedChange={setChecked} />
          </div>
          <div className="sm:col-span-2">
            <Slider
              label="Discount rate (annual)"
              value={rate}
              onValueChange={setRate}
              min={0}
              max={0.25}
              step={0.005}
              format={(v) => `${(v * 100).toFixed(1)}%`}
            />
          </div>
        </div>
      </Section>

      <Section
        title="Dialog"
        description="The replacement for alert/confirm/prompt. Focus is trapped, Escape closes, and validation is visible in place."
      >
        <div className="rounded-lg border border-default p-5">
          <Button onClick={() => setDialogOpen(true)}>Open dialog</Button>
          <Dialog
            open={dialogOpen}
            onOpenChange={setDialogOpen}
            title="New file"
            description="Imported from model.cfdl with an import statement."
            footer={
              <>
                <Button variant="secondary" size="sm" onClick={() => setDialogOpen(false)}>
                  Cancel
                </Button>
                <Button size="sm" onClick={() => setDialogOpen(false)}>
                  Create file
                </Button>
              </>
            }
          >
            <Field label="File name">
              <Input defaultValue="contracts.cfdl" autoFocus />
            </Field>
          </Dialog>
        </div>
      </Section>

      <Section
        title="Tabs and disclosure"
        description="Tabs for peer views; disclosure to keep dense controls out of the way until wanted."
      >
        <div className="space-y-4">
          <div className="overflow-hidden rounded-lg border border-default">
            <Tabs
              value={tab}
              onValueChange={setTab}
              items={[
                { id: "metrics", label: "Metrics" },
                { id: "cash", label: "Cash flows" },
                { id: "diagnostics", label: "Diagnostics", badge: 2, badgeTone: "err" },
              ]}
            />
            <div className="p-4 text-sm text-secondary">Panel: {tab}</div>
          </div>

          <Disclosure
            open={disclosureOpen}
            onOpenChange={setDisclosureOpen}
            title="Run config"
            summary="8.0% · 500 trials"
          >
            <p className="text-sm text-secondary">
              Collapsed by default, with a summary of its state so nothing is hidden.
            </p>
          </Disclosure>
        </div>
      </Section>

      <Section
        title="Code block actions"
        description="Hover the block, or Tab into it, to reveal Copy and Open in playground. Actions rest at zero opacity so they never compete with the code, and appear on focus-within so they stay reachable without a mouse."
      >
        <div className="space-y-4">
          <div className="group relative overflow-hidden rounded-lg border border-default bg-surface-code p-4 pr-24">
            <pre className="overflow-x-auto font-mono text-xs leading-relaxed text-secondary">
              {SAMPLE_MODEL}
            </pre>
            <CodeActions code={SAMPLE_MODEL} lang="cfdl" />
          </div>

          <div className="group relative overflow-hidden rounded-lg border border-default bg-surface-code p-4 pr-24">
            <pre className="overflow-x-auto font-mono text-xs leading-relaxed text-secondary">
              cfdl compile first-model --out first-model/ir.json
            </pre>
            <CodeActions
              code="cfdl compile first-model --out first-model/ir.json"
              lang="bash"
            />
          </div>

          <p className="text-sm text-secondary">
            Only a whole model offers Open in playground. A fragment would land in the
            editor and fail to compile, which reads as a broken button rather than a
            helpful one — so the second block above shows Copy alone.
          </p>
        </div>
      </Section>
    </div>
  );
}

function Section({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <section>
      <h2 className="text-xl font-semibold tracking-tight text-primary">{title}</h2>
      <p className="mb-5 mt-1 max-w-2xl text-sm leading-relaxed text-secondary">{description}</p>
      {children}
    </section>
  );
}
