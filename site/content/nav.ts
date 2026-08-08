/**
 * Docs navigation.
 *
 * Sections are cut by what the reader is doing, in the order they usually do
 * it: understand it, install it, learn it, look something up, read a worked
 * model, check the normative text.
 *
 * "Learn the language" teaches in order and is meant to be read through.
 * "How-to guides" answer one question each and are meant to be dipped into.
 * "Examples" holds complete models, including the benchmark set. Nothing
 * appears in two sections.
 *
 * Titles here are sentence case, matching the frontmatter `title` of the page
 * they point at. Slugs must match those frontmatter slugs; the docs page build
 * fails loudly if one does not resolve.
 */
export interface NavItem {
  title: string;
  slug: string;
  /** One level of nesting: rendered indented beneath the parent entry. */
  items?: NavItem[];
}

export interface NavSection {
  title: string;
  items: NavItem[];
}

export const NAV: NavSection[] = [
  {
    title: "Introduction",
    items: [
      { title: "Overview", slug: "/docs" },
      { title: "How CFDL works", slug: "/docs/concepts" },
      { title: "Getting started", slug: "/docs/getting-started" },
      { title: "Validation", slug: "/docs/benchmarks" },
    ],
  },
  {
    title: "Install and setup",
    items: [
      { title: "Choose a surface", slug: "/docs/install" },
      { title: "CLI", slug: "/docs/install/cli" },
      { title: "Python", slug: "/docs/install/python" },
      { title: "API server", slug: "/docs/install/api-server" },
      { title: "VS Code and LSP", slug: "/docs/install/vscode" },
      { title: "Playground", slug: "/docs/install/playground" },
    ],
  },
  {
    title: "Learn the language",
    items: [
      { title: "The object model", slug: "/docs/object-model" },
      { title: "Language guide", slug: "/docs/language-guide" },
      { title: "Minimal model", slug: "/docs/examples/minimal_model" },
      { title: "Your first stream", slug: "/docs/examples/first_stream" },
      { title: "A simple contract", slug: "/docs/examples/simple_contract" },
      { title: "Using an industry pack", slug: "/docs/examples/with_pack" },
      { title: "Multi-file model", slug: "/docs/examples/multi_file" },
      { title: "Curves", slug: "/docs/examples/curves" },
      { title: "Uncertainty and Monte Carlo", slug: "/docs/examples/uncertainty" },
      { title: "Events and options", slug: "/docs/examples/options_events" },
    ],
  },
  {
    title: "Surfaces",
    items: [
      { title: "Python SDK", slug: "/docs/python-sdk" },
      { title: "API server", slug: "/docs/api-server" },
    ],
  },
  {
    title: "How-to guides",
    items: [
      { title: "Schedules and calendars", slug: "/docs/guides/schedules-and-calendars" },
      { title: "Contracts and packs", slug: "/docs/guides/contracts-and-packs" },
      { title: "Multi-file models", slug: "/docs/guides/multi-file-models" },
      { title: "Scenarios and run configs", slug: "/docs/guides/scenarios-and-run-configs" },
      { title: "Stochastic modeling", slug: "/docs/stochastic-modeling" },
      { title: "Curves", slug: "/docs/guides/curves" },
      { title: "Metrics", slug: "/docs/guides/metrics" },
      { title: "Waterfalls", slug: "/docs/guides/waterfalls" },
      { title: "Statements and reporting", slug: "/docs/guides/statements" },
      { title: "Reading results and IR", slug: "/docs/guides/reading-results" },
      { title: "Troubleshooting", slug: "/docs/troubleshooting" },
    ],
  },
  {
    title: "Domain packs",
    items: [
      { title: "Overview", slug: "/docs/packs" },
      { title: "Energy", slug: "/docs/packs/energy" },
      { title: "CRE", slug: "/docs/packs/cre" },
      { title: "Credit", slug: "/docs/packs/credit" },
      { title: "OpCo", slug: "/docs/packs/opco" },
    ],
  },
  // Examples is its own section rather than a tail on the tutorial. A reader on
  // lesson three does not want a leveraged buyout, and someone looking for a
  // worked deal should not have to know it lives under a section about
  // learning the language.
  {
    title: "Examples",
    items: [
      { title: "Browse all examples", slug: "/docs/examples" },
      { title: "Energy: solar PPA microgrid", slug: "/docs/examples/energy-solar-ppa-microgrid" },
      { title: "Energy: wind PTC + MACRS", slug: "/docs/examples/energy-wind-ptc-macrs" },
      { title: "CRE: two-tenant office", slug: "/docs/examples/cre-office-two-tenant" },
      { title: "CRE: retail strip", slug: "/docs/examples/cre-retail-strip" },
      { title: "Credit: level-pay pool", slug: "/docs/examples/credit-level-pay-pool" },
      { title: "Credit: IO/bullet loan", slug: "/docs/examples/credit-io-bullet-loan" },
      { title: "Credit: floating bridge", slug: "/docs/examples/credit-float-bridge-pool" },
      { title: "OpCo: leveraged buyout", slug: "/docs/examples/opco-lbo-buyout" },
      {
        title: "Notebooks",
        slug: "/docs/notebooks",
        items: [
          { title: "Energy", slug: "/docs/notebooks/energy-solar-microgrid" },
          { title: "CRE", slug: "/docs/notebooks/cre-office-acquisition" },
          { title: "Credit", slug: "/docs/notebooks/credit-loan-pool" },
          { title: "OpCo", slug: "/docs/notebooks/opco-lbo" },
        ],
      },
    ],
  },
  {
    title: "Reference",
    items: [
      { title: "Overview", slug: "/docs/reference" },
      { title: "CLI", slug: "/docs/reference/cli" },
      { title: "Run config", slug: "/docs/reference/run-config" },
      { title: "Expressions", slug: "/docs/reference/expressions" },
      { title: "Pack contracts", slug: "/docs/reference/packs" },
      { title: "Metrics", slug: "/docs/reference/metrics" },
      { title: "Statements", slug: "/docs/reference/statements" },
      { title: "Diagnostics", slug: "/docs/reference/diagnostics" },
    ],
  },
  // Last, and after everything on a modeller's path. These pages are normative
  // and unwelcoming by design; a reader who wants them will look for them, and
  // one who does not should not meet them on the way to something else. Every
  // page carries a banner pointing at its Reference counterpart.
  {
    title: "Specification",
    items: [
      { title: "Overview", slug: "/docs/specification" },
      { title: "Language spec", slug: "/docs/specification/language-spec" },
      { title: "Grammar", slug: "/docs/specification/grammar" },
      { title: "Expressions", slug: "/docs/specification/expression-environment" },
      { title: "Compiler spec", slug: "/docs/specification/compiler-spec" },
      { title: "Diagnostics", slug: "/docs/specification/diagnostics" },
      { title: "Pack interface", slug: "/docs/specification/pack-interface" },
      { title: "IR schema", slug: "/docs/specification/ir-schema" },
      { title: "Results schema", slug: "/docs/specification/results-schema" },
    ],
  },
  {
    title: "About",
    items: [
      { title: "FAQ", slug: "/docs/faq" },
      { title: "Licensing", slug: "/docs/licensing" },
    ],
  },
];

export const FLAT_NAV: NavItem[] = NAV.flatMap((s) =>
  s.items.flatMap((item) => [item, ...(item.items ?? [])]),
);

/**
 * Neighbours for page-foot navigation, scoped to the current section.
 *
 * Chaining all 51 pages into one sequence implied a reading order that does
 * not exist: the last Guide ran on into Domain packs, and reference pages
 * offered a "next" nobody wants to follow. Within a section the sequence is
 * real — the tutorial and the guides are meant to be read in order — so
 * pagination stops at the section boundary instead of inventing a path.
 */
export function sectionNeighbours(slug: string): {
  section?: string;
  prev?: NavItem;
  next?: NavItem;
} {
  for (const section of NAV) {
    const flat = section.items.flatMap((item) => [item, ...(item.items ?? [])]);
    const index = flat.findIndex((item) => item.slug === slug);
    if (index === -1) continue;
    return {
      section: section.title,
      prev: index > 0 ? flat[index - 1] : undefined,
      next: index < flat.length - 1 ? flat[index + 1] : undefined,
    };
  }
  return {};
}
