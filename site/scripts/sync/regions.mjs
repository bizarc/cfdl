/**
 * Managed regions: script-owned blocks inside author-owned pages.
 *
 * THE PROBLEM THIS SOLVES. Until now a documentation page was owned entirely by
 * one side or the other. `writeGenerated` overwrites the whole file, and
 * `sync:check` byte-compares the whole file, so a page fed by a generator could
 * not carry a sentence written for a reader — which is why the site's reference
 * pages read like the internal specifications they are copied from.
 *
 * A region inverts that. The generator owns the bytes between two markers and
 * nothing else, so a page can be authored prose with a table of diagnostic
 * codes, or pack metrics, or schema fields, kept current inside it.
 *
 * DELIBERATELY A PORT of the discipline in `tools/schema_sync.py:embed_in_doc`,
 * which already solved this for the schema pages: find exactly one block,
 * replace its interior, and RAISE rather than guess if the page does not hold
 * exactly one. The failure this avoids is the quiet one — a generator that
 * appends when it cannot find its anchor, or skips silently, leaves a page that
 * has stopped receiving updates and says nothing about it. `npm run prebuild`
 * runs unconditionally on every build including Vercel's, so a silent skip
 * would drift invisibly for as long as nobody looked.
 *
 * Markers are HTML comments so they survive markdown rendering unseen:
 *
 *   <!-- cfdl:generated diagnostics-catalog -->
 *   ...script-owned lines...
 *   <!-- /cfdl:generated diagnostics-catalog -->
 */
import fs from "node:fs";
import path from "node:path";

/** Anchor text for a region's opening and closing markers. */
export function openMarker(key) {
  return `<!-- cfdl:generated ${key} -->`;
}

export function closeMarker(key) {
  return `<!-- /cfdl:generated ${key} -->`;
}

/**
 * Locate a region's interior, or explain precisely why it cannot be located.
 *
 * Returns `{ start, end }` as line indices bounding the interior (exclusive of
 * the markers themselves).
 */
function locate(lines, key, label) {
  const open = openMarker(key);
  const close = closeMarker(key);

  const opens = [];
  const closes = [];
  lines.forEach((line, i) => {
    if (line.trim() === open) opens.push(i);
    if (line.trim() === close) closes.push(i);
  });

  if (opens.length === 0 && closes.length === 0) {
    throw new Error(
      `${label}: no region '${key}'. Expected a pair of markers:\n` +
        `  ${open}\n  ${close}\n` +
        `A generated region cannot be created implicitly — add the markers to ` +
        `the page so it is visible in the source that a generator owns that block.`,
    );
  }
  if (opens.length !== 1 || closes.length !== 1) {
    throw new Error(
      `${label}: region '${key}' must appear exactly once, found ` +
        `${opens.length} opening and ${closes.length} closing markers.`,
    );
  }
  if (closes[0] < opens[0]) {
    throw new Error(`${label}: region '${key}' closes before it opens.`);
  }
  return { start: opens[0] + 1, end: closes[0] };
}

/**
 * Replace a region's interior, or in check mode report whether it is stale.
 *
 * `body` is an array of lines, without the markers.
 *
 * Returns `null` when the file is already correct, or a description of the
 * staleness when it is not. Throws — never returns — when the markers are
 * missing or malformed, in BOTH modes: a page that has lost its anchor is a
 * defect whether or not anyone is currently checking.
 */
export function syncRegion({ filePath, key, body, checkMode, repoRoot }) {
  const label = repoRoot ? path.relative(repoRoot, filePath) : filePath;

  if (!fs.existsSync(filePath)) {
    throw new Error(`${label}: page does not exist, so region '${key}' cannot be written.`);
  }

  const original = fs.readFileSync(filePath, "utf8");
  const lines = original.split("\n");
  const { start, end } = locate(lines, key, label);

  const current = lines.slice(start, end);
  const desired = Array.isArray(body) ? body : String(body).split("\n");

  if (current.length === desired.length && current.every((l, i) => l === desired[i])) {
    return null;
  }

  if (checkMode) {
    return `region '${key}' in ${label} is stale`;
  }

  const next = [...lines.slice(0, start), ...desired, ...lines.slice(end)];
  fs.writeFileSync(filePath, next.join("\n"), "utf8");
  return null;
}

/** Every region key present in a file, for manifest verification. */
export function regionKeys(markdown) {
  const keys = [];
  const re = /^<!-- cfdl:generated ([a-z0-9-]+) -->$/gm;
  let match;
  while ((match = re.exec(markdown)) !== null) {
    keys.push(match[1]);
  }
  return keys;
}
