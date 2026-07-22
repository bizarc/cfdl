import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    "index",
    "getting-started",
    "language-guide",
    {
      type: "category",
      label: "Examples",
      items: [
        "examples/examples",
        "examples/example-minimal_model",
        "examples/example-first_stream",
        "examples/example-simple_contract",
        "examples/example-with_pack",
        "examples/example-multi_file",
        "examples/cre-examples",
        "examples/operating-business-examples"
      ]
    },
    {
      type: "category",
      label: "Cookbooks",
      items: [
        "cookbooks/cookbooks",
        "cookbooks/cookbook-energy",
        "cookbooks/cookbook-cre",
        "cookbooks/cookbook-credit",
        "cookbooks/cookbook-opco"
      ]
    },
    {
      type: "category",
      label: "Language Reference",
      items: [
        "reference",
        "language-reference/language-spec",
        "language-reference/grammar",
        "language-reference/expression-environment",
        "language-reference/compiler-spec",
        "language-reference/diagnostics",
        "language-reference/pack-interface",
        "language-reference/ir-schema",
        "language-reference/results-schema",
        "language-reference/implementation-status"
      ]
    },
    "packs",
    "benchmarks",
    "stochastic-modeling",
    {
      type: "category",
      label: "Surfaces",
      items: ["python-sdk", "api-server"]
    },
    "install-configure",
    "troubleshooting",
    "licensing"
  ]
};

export default sidebars;
