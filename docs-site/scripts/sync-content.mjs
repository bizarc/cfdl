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
    .replaceAll("`docs/LANGUAGE_GUIDE.md`", "[Language Guide](/language-guide)")
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
      "[Install and Configure](/install-configure)"
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

const docSpecs = [
  {
    source: "docs/LANGUAGE_GUIDE.md",
    output: "language-guide.md",
    frontmatter: {
      id: "language-guide",
      title: '"Language Guide"',
      slug: '"/language-guide"'
    }
  },
  {
    source: "docs/docs_packs_guide.md",
    output: "packs.md",
    frontmatter: {
      id: "packs",
      title: '"Packs Guide"',
      slug: '"/packs"'
    }
  },
  {
    source: "distribution/install-configure.md",
    output: "install-configure.md",
    frontmatter: {
      id: "install-configure",
      title: '"Install and Configure"',
      slug: '"/install-configure"'
    }
  },
  {
    source: "docs/cfdl_v_0_1.md",
    output: "language-reference/language-spec.md",
    frontmatter: {
      id: "language-spec",
      title: '"Language Spec (v0.1)"',
      slug: '"/language-reference/language-spec"'
    }
  },
  {
    source: "docs/CFDL_v0_1_Grammar.ebnf.md",
    output: "language-reference/grammar.md",
    frontmatter: {
      id: "grammar",
      title: '"Grammar (EBNF)"',
      slug: '"/language-reference/grammar"'
    }
  },
  {
    source: "docs/compiler_spec_v_0_1.md",
    output: "language-reference/compiler-spec.md",
    frontmatter: {
      id: "compiler-spec",
      title: '"Compiler Spec (v0.1)"',
      slug: '"/language-reference/compiler-spec"'
    },
    digestOnly: true
  },
  {
    source: "docs/diagnostics_spec.md",
    output: "language-reference/diagnostics.md",
    frontmatter: {
      id: "diagnostics",
      title: '"Diagnostics Reference"',
      slug: '"/language-reference/diagnostics"'
    }
  },
  {
    source: "docs/pack_interface_v_0_1.md",
    output: "language-reference/pack-interface.md",
    frontmatter: {
      id: "pack-interface",
      title: '"Pack Interface (v0.1)"',
      slug: '"/language-reference/pack-interface"'
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

  const readme = stripLeadingH1(fs.readFileSync(readmePath, "utf8"));
  const model = fs.readFileSync(modelPath, "utf8").trimEnd();

  const examplePage = [
    "---",
    `id: example-${name}`,
    `title: "${name.replaceAll("_", " ")}"`,
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
  exampleIndexLines.push(`- [${name}](/examples/${name})`);
}

exampleIndexLines.push("");
exampleIndexLines.push("## Domain examples");
exampleIndexLines.push("");
exampleIndexLines.push("- [CRE examples](/examples/cre-examples) — Commercial Real Estate: lease-up, full lifecycle, phased, multi-file, development with financing.");
exampleIndexLines.push("- [Operating Business examples](/examples/operating-business-examples) — OpCo: revenue, opex, working capital, exit multiple, growth, multi-file.");
exampleIndexLines.push("");
writeGenerated("examples/index.md", exampleIndexLines.join("\n"));

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
  "## Reference pages",
  "",
  "- [Language Spec](/language-reference/language-spec)",
  "- [Grammar](/language-reference/grammar)",
  "- [Compiler Spec](/language-reference/compiler-spec)",
  "- [Diagnostics](/language-reference/diagnostics)",
  "- [Pack Interface](/language-reference/pack-interface)",
  ""
].join("\n");

writeGenerated("reference.md", referenceIndex);

if (checkMode) {
  console.log("content sync check passed");
} else {
  console.log("content sync completed");
}
