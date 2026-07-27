import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { CodeActions } from "./CodeActions";

const MODEL = `version 0.1
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

const SHELL = `cfdl compile first-model --out first-model/ir.json`;

/**
 * The hover-revealed actions on a documentation code block.
 *
 * These are always rendered inside a positioned `group` wrapper, so every
 * story supplies one — the component is absolutely positioned and has no
 * meaning on its own.
 */
const meta = {
  title: "Docs/CodeActions",
  component: CodeActions,
  parameters: {
    docs: {
      description: {
        component:
          "Copy and open-in-playground actions for a docs code block. Hover the block (or Tab into it) to reveal them.",
      },
    },
  },
  args: { code: MODEL, lang: "cfdl" },
  decorators: [
    (Story) => (
      <div className="group relative max-w-2xl rounded-lg border border-default bg-surface-code p-4 pr-24 font-mono text-xs leading-relaxed text-secondary">
        <pre className="overflow-x-auto">{MODEL}</pre>
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof CodeActions>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A whole model: both actions are offered. */
export const WholeModel: Story = {};

/**
 * A shell command, or any snippet that is not a complete model, gets Copy
 * only. Opening a fragment in the playground would land it in the editor and
 * fail to compile, which reads as a broken button rather than a helpful one.
 */
export const NotAModel: Story = {
  args: { code: SHELL, lang: "bash" },
  decorators: [
    (Story) => (
      <div className="group relative max-w-2xl rounded-lg border border-default bg-surface-code p-4 pr-24 font-mono text-xs leading-relaxed text-secondary">
        <pre className="overflow-x-auto">{SHELL}</pre>
        <Story />
      </div>
    ),
  ],
};

/**
 * Held visible to inspect the resting state. In the docs the actions sit at
 * `opacity-0` until hover or focus, so they never compete with the code.
 */
export const AlwaysVisible: Story = {
  args: { className: "opacity-100" },
};
