#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const checkMode = process.argv.includes("--check");

const scriptDir = path.dirname(new URL(import.meta.url).pathname);
const docsSiteDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(docsSiteDir, "..");
const docsOutputRoot = path.resolve(docsSiteDir, "docs");
const REPO_HTTP_BASE = "https://github.com/bizarc/cfdl/blob/main";

/**
 * Remove only the top-level H1 so Docusaurus frontmatter title is authoritative.
 */
function stripLeadingH1(markdown) {
  const lines = markdown.split("\n");
  if (lines.length > 0 && lines[0].startsWith("# ")) {
    return lines.slice(1).join("\n").replace(/^\n+/, "");
  }
  return markdown;
}

function normalizeLinks(markdown) {
  return markdown
    .replaceAll(
      "](schemas/CFDL_v0_1_Grammar.ebnf)",
      `](${REPO_HTTP_BASE}/docs/schemas/CFDL_v0_1_Grammar.ebnf)`
    )
    .replaceAll(
      '"When to use streams vs contracts" in `docs/09_user_guide.md`',
      "[When to use streams vs contracts](/language-guide#when-to-use-streams-vs-contracts)"
    )
    .replaceAll("`docs/LANGUAGE_GUIDE.md`", "[Language Guide](/language-guide)")
    .replaceAll("`docs/09_user_guide.md`", "[Language Guide](/language-guide)")
    .replaceAll("`docs/01_language_spec.md`", "[Language Spec](/language-reference/language-spec)")
    .replaceAll("`docs/02_grammar.md`", "[Grammar](/language-reference/grammar)")
    .replaceAll("`docs/04_compiler_spec.md`", "[Compiler Spec](/language-reference/compiler-spec)")
    .replaceAll("`docs/08_diagnostics.md`", "[Diagnostics](/language-reference/diagnostics)")
    .replaceAll("`docs/07_pack_interface.md`", "[Pack Interface](/language-reference/pack-interface)")
    .replaceAll("`docs/cfdl_v_0_1.md`", "[Language Spec](/language-reference/language-spec)")
    .replaceAll(
      "`docs/CFDL_v0_1_Grammar.ebnf.md`",
      "[Grammar](/language-reference/grammar)"
    )
    .replaceAll(
      "`docs/compiler_spec_v_0_1.md`",
      "[Compiler Spec](/language-reference/compiler-spec)"
    )
    .replaceAll(
      "`docs/diagnostics_spec.md`",
      "[Diagnostics](/language-reference/diagnostics)"
    )
    .replaceAll(
      "`docs/pack_interface_v_0_1.md`",
      "[Pack Interface](/language-reference/pack-interface)"
    )
    .replaceAll(
      "`docs/docs_packs_guide.md`",
      "[Packs Guide](/packs)"
    )
    .replaceAll(
      "`distribution/install-configure.md`",
      "[VS Code and LSP setup](/install/vscode)"
    )
    .replaceAll(
      "`examples/language_tutorial/`",
      "[language tutorial examples](/examples)"
    )
    .replaceAll(
      "`docs/USER_GUIDE.md`",
      "[SDK User Guide](https://github.com/bizarc/cfdl/blob/main/docs/USER_GUIDE.md)"
    );
}

function toPosix(p) {
  return p.split(path.sep).join("/");
}

function readSource(relativePath) {
  const absolutePath = path.resolve(repoRoot, relativePath);
  return fs.readFileSync(absolutePath, "utf8");
}

function sourceHttpUrl(relativePath) {
  return `${REPO_HTTP_BASE}/${toPosix(relativePath)}`;
}

function renderDoc(frontmatter, sourcePath, body) {
  const fm = [
    "---",
    ...Object.entries(frontmatter).map(([k, v]) => `${k}: ${v}`),
    "---",
    "",
    `> This page is generated from \`${sourcePath}\`.`,
    `> Source: ${sourceHttpUrl(sourcePath)}`,
    "",
    body.trimEnd(),
    ""
  ];
  return fm.join("\n");
}

function buildCompilerSpecDigest(sourcePath) {
  const sourceUrl = sourceHttpUrl(sourcePath);
  return [
    "This page is a usability-focused digest of the compiler spec for model authors and SDK integrators.",
    "",
    "## Use this page for",
    "",
    "- Understanding compiler stage responsibilities",
    "- Knowing deterministic guarantees and ordering rules",
    "- Finding the validation/diagnostic sections quickly",
    "",
    "## Compiler flow at a glance",
    "",
    "1. Load and normalize files",
    "2. Lex and parse with spans",
    "3. Resolve imports and symbols",
    "4. Validate structure/types/schedules",
    "5. Lower normalized AST into deterministic IR",
    "6. Emit canonical IR JSON",
    "",
    "## Determinism guarantees",
    "",
    "- Same sources + pack version + compiler version must emit deterministic IR.",
    "- Arrays in IR are canonically ordered (entities/contracts/streams/etc).",
    "- Deterministic IDs are derived from stable keys.",
    "",
    "## Sections to read in the full spec",
    "",
    "- AST model and spans",
    "- Validation rules and required statements",
    "- Lowering and normalization rules",
    "- IR assembly and canonical ordering",
    "- Diagnostics contract and error code guide",
    "",
    "## Full compiler spec",
    "",
    `- [Open full compiler spec source](${sourceUrl})`,
    "",
    "If you need strict implementation-level details, use the full source spec above as authoritative.",
    ""
  ].join("\n");
}

function writeGenerated(relativePath, content) {
  const targetPath = path.resolve(docsOutputRoot, relativePath);
  fs.mkdirSync(path.dirname(targetPath), { recursive: true });

  if (checkMode) {
    if (!fs.existsSync(targetPath)) {
      throw new Error(`Missing generated file: ${toPosix(path.relative(repoRoot, targetPath))}`);
    }
    const current = fs.readFileSync(targetPath, "utf8");
    if (current !== content) {
      throw new Error(`Generated file is stale: ${toPosix(path.relative(repoRoot, targetPath))}`);
    }
    return;
  }

  fs.writeFileSync(targetPath, content, "utf8");
}

const exampleTitles = {
  minimal_model: "Minimal Model",
  first_stream: "Your First Stream",
  simple_contract: "A Simple Contract",
  with_pack: "Using an Industry Pack",
  multi_file: "Multi-File Model",
  cre_lease_up: "CRE: Lease-Up",
  cre_developer: "CRE: Developer Lifecycle",
  cre_phased: "CRE: Phased Development",
  cre_multi_file: "CRE: Multi-File Model",
  cre_development_with_financing: "CRE: Development with Financing",
  opco_basic: "OpCo: Basic Operating Model",
  opco_with_growth: "OpCo: Growth via Expressions",
  opco_multi_file: "OpCo: Multi-File Model"
};

function exampleTitle(name) {
  return exampleTitles[name] ?? name.replaceAll("_", " ");
}

const docSpecs = [
  {
    source: "docs/09_user_guide.md",
    output: "language-guide.md",
    frontmatter: {
      id: "language-guide",
      title: '"Language Guide"',
      slug: '"/language-guide"'
    }
  },
  {
    source: "distribution/install-configure.md",
    output: "install/vscode.md",
    frontmatter: {
      id: "install-vscode",
      title: '"VS Code and LSP"',
      slug: '"/install/vscode"'
    }
  },
  {
    source: "docs/01_language_spec.md",
    output: "language-reference/language-spec.md",
    frontmatter: {
      id: "language-spec",
      title: '"Language Spec (v0.1)"',
      slug: '"/language-reference/language-spec"'
    }
  },
  {
    source: "docs/02_grammar.md",
    output: "language-reference/grammar.md",
    frontmatter: {
      id: "grammar",
      title: '"Grammar (EBNF)"',
      slug: '"/language-reference/grammar"'
    }
  },
  {
    source: "docs/04_compiler_spec.md",
    output: "language-reference/compiler-spec.md",
    frontmatter: {
      id: "compiler-spec",
      title: '"Compiler Spec (v0.1)"',
      slug: '"/language-reference/compiler-spec"'
    },
    digestOnly: true
  },
  {
    source: "docs/08_diagnostics.md",
    output: "language-reference/diagnostics.md",
    frontmatter: {
      id: "diagnostics",
      title: '"Diagnostics Reference"',
      slug: '"/language-reference/diagnostics"'
    }
  },
  {
    source: "docs/07_pack_interface.md",
    output: "language-reference/pack-interface.md",
    frontmatter: {
      id: "pack-interface",
      title: '"Pack Interface (v0.1)"',
      slug: '"/language-reference/pack-interface"'
    }
  },
  {
    source: "docs/03_expression_environment.md",
    output: "language-reference/expression-environment.md",
    frontmatter: {
      id: "expression-environment",
      title: '"Expression Environment (v0.1)"',
      slug: '"/language-reference/expression-environment"'
    }
  },
  {
    source: "docs/05_ir_schema.md",
    output: "language-reference/ir-schema.md",
    frontmatter: {
      id: "ir-schema",
      title: '"IR Schema (v0.1)"',
      slug: '"/language-reference/ir-schema"'
    }
  },
  {
    source: "docs/06_results_schema.md",
    output: "language-reference/results-schema.md",
    frontmatter: {
      id: "results-schema",
      title: '"Results Schema (v0.1)"',
      slug: '"/language-reference/results-schema"'
    }
  },
  {
    source: "docs/10_implementation_status.md",
    output: "language-reference/implementation-status.md",
    frontmatter: {
      id: "implementation-status",
      title: '"Implementation Status"',
      slug: '"/language-reference/implementation-status"'
    }
  }
];

for (const spec of docSpecs) {
  let body = spec.digestOnly
    ? buildCompilerSpecDigest(spec.source)
    : normalizeLinks(stripLeadingH1(readSource(spec.source)));
  const rendered = renderDoc(spec.frontmatter, spec.source, body);
  writeGenerated(spec.output, rendered);
}

const exampleRoot = path.resolve(repoRoot, "examples/language_tutorial");
// Fixed tutorial order (learning sequence), not alphabetical
const tutorialOrder = [
  "minimal_model",
  "first_stream",
  "simple_contract",
  "with_pack",
  "multi_file"
];
const existingDirs = new Set(
  fs
    .readdirSync(exampleRoot, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name)
);
const exampleDirs = tutorialOrder.filter((name) => existingDirs.has(name));
// Append any extra dirs not in tutorialOrder (e.g. future examples), sorted
const extraDirs = [...existingDirs].filter((name) => !tutorialOrder.includes(name)).sort();
exampleDirs.push(...extraDirs);

const exampleIndexLines = [
  "---",
  "id: examples",
  'title: "Examples"',
  'slug: "/examples"',
  "---",
  "",
  "Use these examples to learn the language and run real models.",
  "",
  "## Tutorial (language_tutorial)",
  ""
];

for (const name of exampleDirs) {
  const readmePath = path.resolve(exampleRoot, name, "README.md");
  const modelPath = path.resolve(exampleRoot, name, "model.cfdl");
  if (!fs.existsSync(readmePath) || !fs.existsSync(modelPath)) {
    continue;
  }

  const readme = normalizeLinks(stripLeadingH1(fs.readFileSync(readmePath, "utf8")));
  const model = fs.readFileSync(modelPath, "utf8").trimEnd();

  const examplePage = [
    "---",
    `id: example-${name}`,
    `title: "${exampleTitle(name)}"`,
    `slug: "/examples/${name}"`,
    "---",
    "",
    `> Generated from \`examples/language_tutorial/${name}/\`.`,
    "",
    readme.trimEnd(),
    "",
    "## model.cfdl",
    "",
    "```cfdl",
    model,
    "```",
    ""
  ].join("\n");

  writeGenerated(`examples/${name}.md`, examplePage);
  exampleIndexLines.push(`- [${exampleTitle(name)}](/examples/${name})`);
}

exampleIndexLines.push("");
exampleIndexLines.push("## Domain examples");
exampleIndexLines.push("");
exampleIndexLines.push("- [CRE examples](/examples/cre-examples) — Commercial Real Estate: lease-up, full lifecycle, phased, multi-file, development with financing.");
exampleIndexLines.push("- [Operating Business examples](/examples/operating-business-examples) — OpCo: revenue, opex, working capital, exit multiple, growth, multi-file.");
exampleIndexLines.push("");
writeGenerated("examples/index.md", exampleIndexLines.join("\n"));

// Domain examples (CRE and OpCo): generate pages that embed code so the site shows structure without repo access
const domainExampleOrder = [
  "cre_lease_up",
  "cre_developer",
  "cre_phased",
  "cre_multi_file",
  "cre_development_with_financing",
  "opco_with_growth",
  "opco_basic",
  "opco_multi_file"
];
const domainExampleRoot = path.resolve(repoRoot, "examples");

for (const name of domainExampleOrder) {
  const dir = path.resolve(domainExampleRoot, name);
  if (!fs.existsSync(dir) || !fs.statSync(dir).isDirectory()) continue;

  const modelPath = path.resolve(dir, "model.cfdl");
  if (!fs.existsSync(modelPath)) continue;

  const cfdlFiles = ["model.cfdl"];
  const optionalFiles = ["time.cfdl", "structure.cfdl", "contracts.cfdl"];
  for (const f of optionalFiles) {
    if (fs.existsSync(path.resolve(dir, f))) cfdlFiles.push(f);
  }

  const body = [
    `> Generated from \`examples/${name}/\`. Code is shown below so you can see structure and elements without repo access.`,
    ""
  ];

  for (const file of cfdlFiles) {
    const content = fs.readFileSync(path.resolve(dir, file), "utf8").trimEnd();
    body.push(`## ${file}`);
    body.push("");
    body.push("```cfdl");
    body.push(content);
    body.push("```");
    body.push("");
  }

  const readmePath = path.resolve(dir, "README.md");
  if (fs.existsSync(readmePath)) {
    const readme = normalizeLinks(stripLeadingH1(fs.readFileSync(readmePath, "utf8"))).trimEnd();
    body.unshift(readme, "", "---", "");
  }

  const examplePage = [
    "---",
    `id: example-${name.replaceAll("_", "-")}`,
    `title: "${exampleTitle(name)}"`,
    `slug: "/examples/${name}"`,
    "---",
    "",
    ...body
  ].join("\n");

  writeGenerated(`examples/${name}.md`, examplePage);
}

const referenceIndex = [
  "---",
  "id: reference",
  'title: "Language Reference"',
  'slug: "/language-reference"',
  "---",
  "",
  "This section contains language reference material generated from canonical source files.",
  "",
  "## How to use this section",
  "",
  "- Start with **Language Spec** for grammar/semantics.",
  "- Use **Grammar** for syntax form.",
  "- Use **Compiler Spec** digest for implementation flow and jump to full source when needed.",
  "- Use **Diagnostics** when fixing errors.",
  "",
  "## Language",
  "",
  "- [Language Spec](/language-reference/language-spec)",
  "- [Grammar](/language-reference/grammar)",
  "- [Expression Environment](/language-reference/expression-environment)",
  "- [Compiler Spec](/language-reference/compiler-spec)",
  "- [Diagnostics](/language-reference/diagnostics)",
  "- [Pack Interface](/language-reference/pack-interface)",
  "- [Implementation Status](/language-reference/implementation-status)",
  "",
  "## Tools and data contracts",
  "",
  "- [CLI Reference](/reference/cli)",
  "- [Run-Config Reference](/reference/run-config)",
  "- [IR Schema](/language-reference/ir-schema)",
  "- [Results Schema](/language-reference/results-schema)",
  ""
].join("\n");

writeGenerated("reference.md", referenceIndex);

// --- Cookbooks: one page per pack, synced from packs/<pack>/README.md -------
const cookbookPacks = ["energy", "cre", "credit", "opco"];
const packTitles = { energy: "Energy", cre: "CRE", credit: "Credit", opco: "OpCo" };
const cookbookIndexLines = [
  "---",
  "id: cookbooks",
  'title: "Cookbooks"',
  'slug: "/cookbooks"',
  "---",
  "",
  "Per-industry recipes: the contract types, terms, and metrics each pack ships,",
  "with worked example notebooks. Generated from each pack's README.",
  "",
  "## Packs",
  ""
];

// Domain example pages folded into each pack guide (instead of separate
// sidebar entries next to the cookbooks).
const packWorkedExamples = {
  cre: [
    ["CRE examples overview", "/examples/cre-examples"],
    ["Lease-up", "/examples/cre_lease_up"],
    ["Developer lifecycle", "/examples/cre_developer"],
    ["Phased development", "/examples/cre_phased"],
    ["Multi-file model", "/examples/cre_multi_file"],
    ["Development with financing", "/examples/cre_development_with_financing"]
  ],
  opco: [
    ["Operating Business examples overview", "/examples/operating-business-examples"],
    ["Basic OpCo", "/examples/opco_basic"],
    ["Growth via expressions", "/examples/opco_with_growth"],
    ["Multi-file model", "/examples/opco_multi_file"]
  ]
};

for (const pack of cookbookPacks) {
  const readmePath = path.resolve(repoRoot, `packs/${pack}/README.md`);
  if (!fs.existsSync(readmePath)) continue;
  let body = normalizeLinks(stripLeadingH1(fs.readFileSync(readmePath, "utf8")));
  const worked = packWorkedExamples[pack];
  if (worked) {
    body +=
      "\n## Worked example models\n\n" +
      worked.map(([label, href]) => `- [${label}](${href})`).join("\n") +
      "\n";
  }
  const page = renderDoc(
    {
      id: `cookbook-${pack}`,
      title: `"${packTitles[pack] ?? pack} pack guide"`,
      slug: `"/cookbooks/${pack}"`
    },
    `packs/${pack}/README.md`,
    body
  );
  writeGenerated(`cookbooks/${pack}.md`, page);
  cookbookIndexLines.push(`- [${packTitles[pack] ?? pack} pack guide](/cookbooks/${pack})`);
}

cookbookIndexLines.push("");
cookbookIndexLines.push("## Example notebooks");
cookbookIndexLines.push("");
cookbookIndexLines.push(
  "Runnable Jupyter notebooks (one per pack) live in " +
    `[\`examples/notebooks/\`](${REPO_HTTP_BASE}/examples/notebooks): ` +
    "solar PPA microgrid, CRE office acquisition, credit loan pool, and an OpCo LBO."
);
cookbookIndexLines.push("");
writeGenerated("cookbooks/index.md", cookbookIndexLines.join("\n"));

// --- Benchmark methodology page --------------------------------------------
const benchRoot = path.resolve(repoRoot, "benchmarks");
const benchCases = [];
if (fs.existsSync(benchRoot)) {
  for (const pack of fs.readdirSync(benchRoot).sort()) {
    const packDir = path.resolve(benchRoot, pack);
    if (!fs.statSync(packDir).isDirectory()) continue;
    for (const name of fs.readdirSync(packDir).sort()) {
      const caseDir = path.resolve(packDir, name);
      if (!fs.statSync(caseDir).isDirectory()) continue;
      if (!fs.existsSync(path.resolve(caseDir, "model.cfdl"))) continue;
      benchCases.push({ pack, name });
    }
  }
}

const benchLines = [
  "---",
  "id: benchmarks",
  'title: "Benchmark methodology"',
  'slug: "/benchmarks"',
  "---",
  "",
  "Every pack is gated by a parity suite: each CFDL model is diffed against an",
  "**independent reference** implementation, period-by-period and on summary",
  "metrics, inside a tolerance the case declares.",
  "",
  "## How a case is built",
  "",
  "Each `benchmarks/<pack>/<case>/` directory contains:",
  "",
  "- `model.cfdl` — the CFDL model;",
  "- `run.json` — the run configuration;",
  "- `case.toml` — the pack name and per-period tolerance;",
  "- `expected.csv` — period-level net cash flow from the reference;",
  "- `expected_metrics.json` — summary metrics, each with its own tolerance;",
  "- `reference_gen.py` — the independent reference that produces the expected",
  "  files (a month-by-month recursion, distinct from the engine's evaluation).",
  "",
  "`tools/benchmark-runner.py` compiles and runs each case with the `cfdl` CLI",
  "and fails if any period or metric drifts outside tolerance. Schedule math is",
  "held decimal-exact; IRR-class iteratives use a bps tolerance.",
  "",
  "## Cases",
  "",
  "| Pack | Case |",
  "|---|---|",
  ...benchCases.map((c) => `| ${c.pack} | \`${c.name}\` |`),
  "",
  "> Reference models are independent implementations; those still awaiting",
  "> practitioner verification say so in their `case.toml`.",
  ""
];
writeGenerated("benchmarks.md", benchLines.join("\n"));

// --- Stage JSON schemas at their $id path (static/schemas/...) --------------
const schemaStaticDir = path.resolve(docsSiteDir, "static", "schemas");
for (const schema of ["CFDL_v0_1_IR.schema.json", "CFDL_v0_1_Results.schema.json"]) {
  const src = path.resolve(repoRoot, "docs", "schemas",
    schema.replace("CFDL_v0_1_IR", "ir").replace("CFDL_v0_1_Results", "results"));
  const content = fs.readFileSync(src, "utf8");
  const target = path.resolve(schemaStaticDir, schema);
  if (checkMode) {
    if (!fs.existsSync(target) || fs.readFileSync(target, "utf8") !== content) {
      throw new Error(`Staged schema is stale or missing: static/schemas/${schema}`);
    }
  } else {
    fs.mkdirSync(schemaStaticDir, { recursive: true });
    fs.writeFileSync(target, content, "utf8");
  }
}

if (checkMode) {
  console.log("content sync check passed");
} else {
  console.log("content sync completed");
}
