import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { Dialog } from "./Dialog";
import { Button } from "./Button";
import { Field, Input } from "./Field";

const meta = {
  title: "Design System/Dialog",
  component: Dialog,
} satisfies Meta<typeof Dialog>;

export default meta;
type Story = StoryObj<typeof meta>;

/**
 * The replacement for window.prompt/confirm/alert. Validation is visible in
 * place and the confirming action stays disabled until the input is valid —
 * a native prompt could only reject silently.
 */
export const NewFile: Story = {
  render: function NewFileStory() {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("contracts.cfdl");
    const error = name.trim().endsWith(".cfdl") ? undefined : "File names must end with .cfdl";

    return (
      <>
        <Button onClick={() => setOpen(true)}>Open dialog</Button>
        <Dialog
          open={open}
          onOpenChange={setOpen}
          title="New file"
          description="Imported from model.cfdl with an import statement."
          footer={
            <>
              <Button variant="secondary" size="sm" onClick={() => setOpen(false)}>
                Cancel
              </Button>
              <Button size="sm" disabled={Boolean(error)} onClick={() => setOpen(false)}>
                Create file
              </Button>
            </>
          }
        >
          <Field label="File name" error={error}>
            <Input value={name} invalid={Boolean(error)} onChange={(e) => setName(e.target.value)} />
          </Field>
        </Dialog>
      </>
    );
  },
};
