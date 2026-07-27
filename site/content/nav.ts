/**
 * Docs navigation. Ported from the previous sidebar so the information
 * architecture (Introduction → Install → Tutorial → Guides → Packs →
 * Surfaces → Reference → Project) carries over unchanged.
 *
 * Slugs here must match the frontmatter slugs under content/docs; the docs
 * page build fails loudly if one doesn't resolve.
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
    ],
  },
  {
    title: "Install & Setup",
    items: [
      { title: "Choose a surface", slug: "/docs/install" },
      { title: "CLI", slug: "/docs/install/cli" },
      { title: "Python", slug: "/docs/install/python" },
      { title: "API server", slug: "/docs/install/api-server" },
      { title: "VS Code & LSP", slug: "/docs/install/vscode" },
      { title: "Playground", slug: "/docs/install/playground" },
    ],
  },
  {
    title: "Learn the Language",
    items: [
      { title: "Language guide", slug: "/docs/language-guide" },
      { title: "All examples", slug: "/docs/examples" },
      { title: "Minimal model", slug: "/docs/examples/minimal_model" },
      { title: "Your first stream", slug: "/docs/examples/first_stream" },
      { title: "A simple contract", slug: "/docs/examples/simple_contract" },
      { title: "Using an industry pack", slug: "/docs/examples/with_pack" },
      { title: "Multi-file model", slug: "/docs/examples/multi_file" },
    ],
  },
  // Surfaces sits directly after the language material: the notebooks inside
  // it are worked examples, and burying them below Guides and Domain Packs
  // left them ~1000px down a sidebar that does not scroll independently.
  {
    title: "Surfaces",
    items: [
      { title: "Python SDK", slug: "/docs/python-sdk" },
      { title: "API server", slug: "/docs/api-server" },
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
    title: "Guides",
    items: [
      { title: "Schedules & calendars", slug: "/docs/guides/schedules-and-calendars" },
      { title: "Contracts & packs", slug: "/docs/guides/contracts-and-packs" },
      { title: "Multi-file models", slug: "/docs/guides/multi-file-models" },
      { title: "Scenarios & run configs", slug: "/docs/guides/scenarios-and-run-configs" },
      { title: "Stochastic modeling", slug: "/docs/stochastic-modeling" },
      { title: "Curves", slug: "/docs/guides/curves" },
      { title: "Metrics", slug: "/docs/guides/metrics" },
      { title: "Reading results & IR", slug: "/docs/guides/reading-results" },
    ],
  },
  {
    title: "Domain Packs",
    items: [
      { title: "Overview", slug: "/docs/packs" },
      { title: "Energy", slug: "/docs/cookbooks/energy" },
      { title: "CRE", slug: "/docs/cookbooks/cre" },
      { title: "Credit", slug: "/docs/cookbooks/credit" },
      { title: "OpCo", slug: "/docs/cookbooks/opco" },
    ],
  },
  {
    title: "Reference",
    items: [
      { title: "Index", slug: "/docs/language-reference" },
      { title: "CLI", slug: "/docs/reference/cli" },
      { title: "Run config", slug: "/docs/reference/run-config" },
      { title: "Language spec", slug: "/docs/language-reference/language-spec" },
      { title: "Grammar", slug: "/docs/language-reference/grammar" },
      { title: "Expressions", slug: "/docs/language-reference/expression-environment" },
      { title: "Compiler spec", slug: "/docs/language-reference/compiler-spec" },
      { title: "Diagnostics", slug: "/docs/language-reference/diagnostics" },
      { title: "Pack interface", slug: "/docs/language-reference/pack-interface" },
      { title: "IR schema", slug: "/docs/language-reference/ir-schema" },
      { title: "Results schema", slug: "/docs/language-reference/results-schema" },
    ],
  },
  {
    title: "Project",
    items: [
      { title: "Benchmarks", slug: "/docs/benchmarks" },
      { title: "Implementation status", slug: "/docs/language-reference/implementation-status" },
      { title: "Troubleshooting", slug: "/docs/troubleshooting" },
      { title: "FAQ", slug: "/docs/faq" },
      { title: "Licensing", slug: "/docs/licensing" },
    ],
  },
];

export const FLAT_NAV: NavItem[] = NAV.flatMap((s) =>
  s.items.flatMap((item) => [item, ...(item.items ?? [])]),
);
