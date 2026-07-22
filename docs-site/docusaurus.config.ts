import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

const config: Config = {
  title: "CFDL Docs",
  tagline: "Cash Flow Domain Language onboarding and reference",
  url: "https://bizarc.github.io",
  baseUrl: process.env.DOCS_BASE_URL || "/cfdl/",
  organizationName: "bizarc",
  projectName: "cfdl",
  trailingSlash: false,
  onBrokenLinks: "throw",
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: "warn"
    }
  },
  i18n: {
    defaultLocale: "en",
    locales: ["en"]
  },
  themes: ["@easyops-cn/docusaurus-search-local"],
  plugins: [],
  presets: [
    [
      "classic",
      {
        docs: {
          routeBasePath: "/",
          sidebarPath: "./sidebars.ts",
          editUrl: "https://github.com/bizarc/cfdl/tree/main/docs-site/",
          showLastUpdateAuthor: true,
          showLastUpdateTime: true
        },
        blog: false,
        // Enabled for the /playground custom page (src/pages/playground.tsx).
        pages: {},
        theme: {
          customCss: "./src/css/custom.css"
        }
      } satisfies Preset.Options
    ]
  ],
  themeConfig: {
    navbar: {
      title: "CFDL",
      items: [
        { to: "/getting-started", label: "Getting Started", position: "left" },
        { to: "/language-guide", label: "Language Guide", position: "left" },
        { to: "/examples", label: "Examples", position: "left" },
        { to: "/language-reference", label: "Reference", position: "left" },
        { to: "/playground", label: "Playground", position: "left" },
        {
          href: "https://github.com/bizarc/cfdl",
          label: "GitHub",
          position: "right"
        }
      ]
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Guides",
          items: [
            { label: "Getting Started", to: "/getting-started" },
            { label: "Language Guide", to: "/language-guide" },
            { label: "Examples", to: "/examples" },
            { label: "Install and Configure", to: "/install-configure" }
          ]
        },
        {
          title: "Reference",
          items: [
            { label: "Reference Index", to: "/language-reference" },
            { label: "Language Spec", to: "/language-reference/language-spec" },
            { label: "Diagnostics", to: "/language-reference/diagnostics" },
            {
              label: "Repository",
              href: "https://github.com/bizarc/cfdl"
            }
          ]
        }
      ],
      copyright: `Copyright ${new Date().getFullYear()} CFDL`
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["json", "toml", "bash"]
    },
    colorMode: {
      defaultMode: "dark",
      respectPrefersColorScheme: true
    },
    announcementBar: {
      id: "onboarding",
      content:
        "Start with Getting Started, then Language Guide, Examples, and full Reference.",
      isCloseable: true
    }
  } satisfies Preset.ThemeConfig
};

export default config;
