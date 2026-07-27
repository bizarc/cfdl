import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { Disclosure, Tabs } from "./Tabs";

const meta = { title: "Design System/Tabs", component: Tabs } satisfies Meta<typeof Tabs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const WithBadges: Story = {
  render: function TabsStory() {
    const [tab, setTab] = useState("metrics");
    return (
      <div className="overflow-hidden rounded-lg border border-default">
        <Tabs
          value={tab}
          onValueChange={setTab}
          items={[
            { id: "metrics", label: "Metrics" },
            { id: "cash", label: "Cash flows" },
            { id: "diagnostics", label: "Diagnostics", badge: 3, badgeTone: "err" },
          ]}
        />
        <div className="p-4 text-sm text-secondary">Panel: {tab}</div>
      </div>
    );
  },
};

/** Keeps dense controls out of the way while still summarising their state. */
export const CollapsedDisclosure: Story = {
  render: function DisclosureStory() {
    const [open, setOpen] = useState(false);
    return (
      <Disclosure open={open} onOpenChange={setOpen} title="Run config" summary="8.0% · 500 trials">
        <p className="text-sm text-secondary">Controls live here.</p>
      </Disclosure>
    );
  },
};
