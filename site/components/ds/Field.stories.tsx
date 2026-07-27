import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { Checkbox, Field, Input, Select, Slider } from "./Field";

const meta = {
  title: "Design System/Form controls",
  component: Field,
} satisfies Meta<typeof Field>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Every control is wrapped so label, hint, and error stay attached to it. */
export const Text: Story = {
  render: () => (
    <div className="max-w-xs space-y-4">
      <Field label="File name" hint="Must end with .cfdl">
        <Input defaultValue="model.cfdl" />
      </Field>
      <Field label="File name" error="File names must end with .cfdl">
        <Input defaultValue="contracts.txt" invalid />
      </Field>
    </div>
  ),
};

export const Choice: Story = {
  render: function ChoiceStory() {
    const [checked, setChecked] = useState(true);
    return (
      <div className="max-w-xs space-y-4">
        <Field label="Pack">
          <Select defaultValue="cre">
            <option value="">none</option>
            <option value="cre">cre</option>
            <option value="energy">energy</option>
          </Select>
        </Field>
        <Checkbox label="Monte Carlo" checked={checked} onCheckedChange={setChecked} />
      </div>
    );
  },
};

export const Range: Story = {
  render: function RangeStory() {
    const [rate, setRate] = useState(0.08);
    return (
      <div className="max-w-xs">
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
    );
  },
};
