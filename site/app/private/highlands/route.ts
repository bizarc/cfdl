import { readFile } from "node:fs/promises";
import { join } from "node:path";

/**
 * The Highlands case study, served as a complete standalone document.
 *
 * A route handler rather than a page, because the file is a whole HTML document
 * with its own typography and palette — wrapping it in the site's layout would
 * fight both. Access is gated by `middleware.ts`, which matches `/private/:path*`
 * and answers an unauthenticated request with a challenge, so nothing here needs
 * to check credentials again.
 *
 * Static files under `public/` would have been the obvious home, except that
 * serving them can skip middleware; a route cannot.
 */
export const dynamic = "force-dynamic";

export async function GET() {
  const html = await readFile(
    join(process.cwd(), "app", "private", "highlands", "content.html"),
    "utf8",
  );

  return new Response(html, {
    headers: {
      "content-type": "text/html; charset=utf-8",
      "cache-control": "private, no-store",
      "x-robots-tag": "noindex, nofollow, noarchive",
    },
  });
}
