import { readFile } from "node:fs/promises";
import { join } from "node:path";

/**
 * Index of the private case pages.
 *
 * A route handler rather than a page, for the same reason the case pages are:
 * each is a whole HTML document with its own typography, and the site layout
 * would fight it. Access is gated by `middleware.ts` on `/private/:path*`.
 *
 * The list lives here rather than in a scan of the directory, because a page
 * appears on it only when someone decides it should.
 */
export const dynamic = "force-dynamic";

const CASES = [
  {
    href: "/private/highlands",
    title: "The Highlands",
    standfirst:
      "A ground-up mixed-use development in Rosslyn, reconstructed from the public " +
      "record. Land, both tower sales and 102 condominium closings are recorded, so " +
      "the record fixes the answer.",
    meta: ["Site Plan #445", "2011&ndash;2024", "Completed"], // tokens-allow: a site plan number, not a color
  },
  {
    href: "/private/one-rosslyn",
    title: "One Rosslyn",
    standfirst:
      "An entitled but unbuilt development four blocks away, by the same sponsor and " +
      "the same equity partner. The program and the land are recorded fact. The " +
      "economics are forecast.",
    meta: ["Site Plan #419", "2023&ndash;2037", "Unbuilt"], // tokens-allow: a site plan number, not a color
  },
];

export async function GET() {
  // Reuse the case pages' head and stylesheet so the set reads as one.
  const shell = await readFile(
    join(process.cwd(), "app", "private", "highlands", "content.html"),
    "utf8",
  );
  const head = shell
    .slice(0, shell.indexOf("<body>") + "<body>".length)
    .replace("<title>The Highlands Benchmark</title>", "<title>Private Cases</title>");

  const cards = CASES.map(
    (c) => `
    <section>
      <span class="eyebrow">${c.meta.join(" &middot; ")}</span>
      <h2><a href="${c.href}">${c.title}</a></h2>
      <p>${c.standfirst}</p>
    </section>`,
  ).join("\n    <hr>\n");

  const body = `

<div id="doc">
<div class="wrap">

  <header class="mast">
    <span class="eyebrow">CFDL &middot; Private</span>
    <h1 class="title">Two deals, four blocks apart</h1>
    <p class="standfirst">
      The same sponsor and the same equity partner, in the same submarket. One deal
      completed, so the record fixes its answer. The other is entitled and unbuilt, so
      its economics are forecast. Read them together.
    </p>
  </header>
${cards}

  <footer>
    Prepared with CFDL, a domain language for cash-flow models. These pages are shared
    privately and are access-controlled.
  </footer>

</div>
</div>

</body>
</html>
`;

  return new Response(head + body, {
    headers: {
      "content-type": "text/html; charset=utf-8",
      "cache-control": "private, no-store",
      "x-robots-tag": "noindex, nofollow, noarchive",
    },
  });
}
