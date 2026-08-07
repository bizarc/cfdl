/**
 * Getting results out of the browser.
 *
 * The playground renders every number, but a reader who wants to check one
 * against their own model needs it in a spreadsheet, not in a pane. These are
 * the two exits: a downloaded file, and the clipboard.
 */

export type Cell = string | number | null | undefined;

/**
 * RFC 4180 quoting for CSV; plain join for TSV.
 *
 * TSV exists because it is what a spreadsheet accepts from the clipboard —
 * pasting CSV lands in one column unless the user runs a text-import wizard.
 * Tabs and newlines are stripped rather than quoted for the same reason:
 * clipboard TSV has no escape that Excel honours on paste.
 */
export function toDelimited(rows: Cell[][], sep: "," | "\t" = ","): string {
  const cell = (v: Cell): string => {
    if (v === null || v === undefined) return "";
    const s = String(v);
    if (sep === "\t") return s.replace(/[\t\r\n]+/g, " ");
    return /[",\r\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s;
  };
  return rows.map((r) => r.map(cell).join(sep)).join("\r\n");
}

/** Triggers a file download without leaving the page. */
export function downloadText(filename: string, text: string, mime = "text/plain"): void {
  const url = URL.createObjectURL(new Blob([text], { type: `${mime};charset=utf-8` }));
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  // Revoking synchronously can cancel the download in some browsers.
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

/**
 * Clipboard write that reports whether it worked.
 *
 * `navigator.clipboard` is unavailable on insecure origins and can be denied
 * by permission, so callers show "Copied" only on a true return rather than
 * assuming.
 */
export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

/** Safe for a filename on every OS the site is read on. */
export function slugForFile(s: string): string {
  return s.replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "") || "cfdl";
}
