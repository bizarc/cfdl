import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    {
      type: "category",
      label: "Introduction",
      collapsed: false,
      items: ["index", "getting-started"]
    },
    {
      type: "category",
      label: "Install & Setup",
      items: ["install/install-vscode"]
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
      items: ["stochastic-modeling"]
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
        "cookbooks/cookbooks",
        "examples/cre-examples",
        "examples/operating-business-examples"
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
        "licensing"
      ]
    }
  ]
};

export default sidebars;
