import { cn } from "@/lib/cn";

/**
 * The mark: a cash-flow bar series resolving into a distribution curve —
 * the deterministic number and the shape around it, which is the product.
 */
export function LogoMark({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 32 32"
      fill="none"
      aria-hidden="true"
      className={cn("h-6 w-6", className)}
    >
      <rect
        x="2"
        y="18"
        width="4"
        height="10"
        rx="1"
        fill="currentColor"
        opacity="0.45"
      />
      <rect
        x="8"
        y="13"
        width="4"
        height="15"
        rx="1"
        fill="currentColor"
        opacity="0.65"
      />
      <rect x="14" y="8" width="4" height="20" rx="1" fill="currentColor" />
      <path
        d="M20 28c0-9 2.2-16 6-16s6 7 6 16"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        opacity="0.85"
      />
    </svg>
  );
}

export function Logo({ className }: { className?: string }) {
  return (
    <span className={cn("inline-flex items-center gap-2", className)}>
      <LogoMark className="text-accent-text" />
      <span className="text-[15px] font-semibold tracking-tight text-primary">
        CFDL
      </span>
    </span>
  );
}
