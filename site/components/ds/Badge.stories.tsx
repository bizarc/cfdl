import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { Badge } from "./Badge";

const meta = {
  title: "Design System/Badge",
  component: Badge,
  args: { children: "Engine ready" },
  argTypes: {
    tone: { control: "select", options: ["neutral", "accent", "ok", "warn", "err"] },
  },
} satisfies Meta<typeof Badge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Neutral: Story = {};
export const Ok: Story = { args: { tone: "ok" } };
export const Error: Story = { args: { tone: "err", children: "Engine failed to load" } };

/** Tone carries the meaning; never rely on the text alone. */
export const AllTones: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Badge>Neutral</Badge>
      <Badge tone="accent">Running</Badge>
      <Badge tone="ok">Engine ready</Badge>
      <Badge tone="warn">Deprecated</Badge>
      <Badge tone="err">Failed</Badge>
    </div>
  ),
};
