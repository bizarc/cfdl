import type { HTMLAttributes } from "react";
import { cn } from "@/lib/cn";

type Tone = "neutral" | "accent" | "ok" | "warn" | "err";

const TONES: Record<Tone, string> = {
  neutral: "bg-surface-sunken text-secondary border-default",
  accent: "bg-accent-soft text-accent-text border-transparent",
  ok: "bg-ok-soft text-ok border-transparent",
  warn: "bg-warn-soft text-warn border-transparent",
  err: "bg-err-soft text-err border-transparent",
};

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  tone?: Tone;
}

export function Badge({ className, tone = "neutral", ...props }: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5",
        "text-xs font-medium",
        TONES[tone],
        className,
      )}
      {...props}
    />
  );
}
