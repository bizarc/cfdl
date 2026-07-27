import { highlight } from "@/lib/shiki";
import { cn } from "@/lib/cn";

export interface CodeBlockProps {
  code: string;
  lang?: string;
  filename?: string;
  className?: string;
}

/**
 * Server component: highlighting happens at build time, so no syntax-theme
 * JavaScript ships to the reader.
 */
export async function CodeBlock({
  code,
  lang = "cfdl",
  filename,
  className,
}: CodeBlockProps) {
  const html = await highlight(code, lang);

  return (
    <figure
      className={cn(
        "overflow-hidden rounded-lg border border-default bg-surface-code",
        className,
      )}
    >
      {filename ? (
        <figcaption className="border-b border-subtle px-4 py-2 font-mono text-xs text-muted">
          {filename}
        </figcaption>
      ) : null}
      <div
        className="overflow-x-auto p-4 font-mono text-[13px] leading-relaxed [&_pre]:bg-transparent"
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </figure>
  );
}
