"use client";

import * as RadixCheckbox from "@radix-ui/react-checkbox";
import * as RadixSlider from "@radix-ui/react-slider";
import { Check } from "lucide-react";
import { useId, type InputHTMLAttributes, type ReactNode, type SelectHTMLAttributes } from "react";
import { cn } from "@/lib/cn";

/** Shared label + optional hint/error wrapper, so every control is labeled. */
export function Field({
  label,
  hint,
  error,
  htmlFor,
  children,
  className,
}: {
  label?: string;
  hint?: string;
  error?: string;
  htmlFor?: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("space-y-1", className)}>
      {label ? (
        <label htmlFor={htmlFor} className="block text-xs font-medium text-secondary">
          {label}
        </label>
      ) : null}
      {children}
      {error ? (
        <p role="alert" className="text-xs text-err">
          {error}
        </p>
      ) : hint ? (
        <p className="text-[11px] text-muted">{hint}</p>
      ) : null}
    </div>
  );
}

const CONTROL_BASE =
  "w-full rounded-md border bg-surface-raised px-2.5 py-1.5 text-sm text-primary " +
  "transition-colors placeholder:text-muted disabled:opacity-50";

export function Input({
  className,
  invalid,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { invalid?: boolean }) {
  return (
    <input
      aria-invalid={invalid || undefined}
      className={cn(
        CONTROL_BASE,
        "font-mono",
        invalid ? "border-err" : "border-default hover:border-strong",
        className,
      )}
      {...props}
    />
  );
}

export function Select({
  className,
  children,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      className={cn(CONTROL_BASE, "border-default font-mono hover:border-strong", className)}
      {...props}
    >
      {children}
    </select>
  );
}

export function Checkbox({
  checked,
  onCheckedChange,
  label,
  id,
}: {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  label: string;
  id?: string;
}) {
  const generated = useId();
  const controlId = id ?? generated;

  return (
    <div className="flex items-center gap-2">
      <RadixCheckbox.Root
        id={controlId}
        checked={checked}
        onCheckedChange={(value) => onCheckedChange(value === true)}
        className={cn(
          "flex h-4 w-4 shrink-0 items-center justify-center rounded border transition-colors",
          checked ? "border-transparent bg-accent" : "border-strong bg-surface-raised",
        )}
      >
        <RadixCheckbox.Indicator>
          <Check className="h-3 w-3 text-accent-fg" />
        </RadixCheckbox.Indicator>
      </RadixCheckbox.Root>
      <label htmlFor={controlId} className="cursor-pointer text-xs text-secondary">
        {label}
      </label>
    </div>
  );
}

export function Slider({
  value,
  onValueChange,
  min = 0,
  max = 1,
  step = 0.01,
  label,
  format,
}: {
  value: number;
  onValueChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  label: string;
  format?: (value: number) => string;
}) {
  return (
    <Field label={label}>
      <div className="flex items-center gap-3">
        <RadixSlider.Root
          value={[value]}
          onValueChange={([next]) => onValueChange(next)}
          min={min}
          max={max}
          step={step}
          aria-label={label}
          className="relative flex h-4 flex-1 touch-none select-none items-center"
        >
          <RadixSlider.Track className="relative h-1 grow rounded-full bg-surface-sunken">
            <RadixSlider.Range className="absolute h-full rounded-full bg-accent" />
          </RadixSlider.Track>
          <RadixSlider.Thumb className="block h-3.5 w-3.5 rounded-full border-2 border-accent bg-surface-page transition-transform hover:scale-110" />
        </RadixSlider.Root>
        <span className="w-14 text-right font-mono text-xs tabular-nums text-primary">
          {format ? format(value) : value}
        </span>
      </div>
    </Field>
  );
}
