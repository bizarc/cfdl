import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    {
      type: "category",
      label: "Introduction",
      collapsed: false,
      items: ["index", "concepts", "getting-started"]
    },
    {
      type: "category",
      label: "Install & Setup",
      items: [
        "install/install-index",
        "install/install-cli",
        "install/install-python",
        "install/install-api-server",
        "install/install-vscode",
        "install/install-playground"
      ]
    },
    {
      type: "category",
      label: "Learn the Language",
      items: [
        "language-guide",
        "examples/examples",
        "examples/example-minimal_model",
        "examples/example-first_stream",
        "examples/example-simple_contract",
        "examples/example-with_pack",
        "examples/example-multi_file"
      ]
    },
    {
      type: "category",
      label: "Guides",
      items: [
        "guides/guide-schedules",
        "guides/guide-contracts-packs",
        "guides/guide-multi-file",
        "guides/guide-scenarios",
        "stochastic-modeling",
        "guides/guide-curves",
        "guides/guide-metrics",
        "guides/guide-reading-results"
      ]
    },
    {
      type: "category",
      label: "Domain Packs",
      items: [
        "packs/packs-overview",
        "cookbooks/cookbook-energy",
        "cookbooks/cookbook-cre",
        "cookbooks/cookbook-credit",
        "cookbooks/cookbook-opco",
        "cookbooks/cookbooks"
      ]
    },
    {
      type: "category",
      label: "Surfaces",
      items: [
        "python-sdk",
        "api-server",
        { type: "link", label: "Playground", href: "/playground" }
      ]
    },
    {
      type: "category",
      label: "Reference",
      items: [
        "reference",
        "reference/reference-cli",
        "reference/reference-run-config",
        "language-reference/language-spec",
        "language-reference/grammar",
        "language-reference/expression-environment",
        "language-reference/compiler-spec",
        "language-reference/diagnostics",
        "language-reference/pack-interface",
        "language-reference/ir-schema",
        "language-reference/results-schema"
      ]
    },
    {
      type: "category",
      label: "Project",
      items: [
        "benchmarks",
        "language-reference/implementation-status",
        "troubleshooting",
        "faq",
        "licensing"
      ]
    }
  ]
};

export default sidebars;
