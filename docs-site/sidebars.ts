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
      label: "Language Reference",
      items: [
        "reference",
        "language-reference/language-spec",
        "language-reference/grammar",
        "language-reference/compiler-spec",
        "language-reference/diagnostics",
        "language-reference/pack-interface"
      ]
    },
    "packs",
    "install-configure",
    "troubleshooting"
  ]
};

export default sidebars;
