#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const checkMode = process.argv.includes("--check");

const scriptDir = path.dirname(new URL(import.meta.url).pathname);
const siteDir = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(siteDir, "..");
const docsOutputRoot = path.resolve(siteDir, "content", "docs");

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

/**
 * Canonical docs occasionally carry site-absolute links written for the old
 * site layout (e.g. `/language-reference/...`). Every page now lives under
 * /docs, so rewrite any such link rather than requiring the specs to know
 * the site's route structure.
 */
function namespaceLegacyDocLinks(markdown) {
  return markdown.replace(/\]\((\/[a-zA-Z0-9\-_/#.]*)\)/g, (whole, href) => {
    if (href.startsWith("/docs") || href.startsWith("/playground") || href.startsWith("/schemas")) {
      return whole;
    }
    return `](/docs${href})`;
  });
}

function normalizeLinks(markdown) {
  return namespaceLegacyDocLinks(markdown
    .replaceAll("](schemas/CFDL_v0_1_Grammar.ebnf)", "](/schemas/CFDL_v0_1_Grammar.ebnf)")
    .replaceAll(
      '"When to use streams vs contracts" in `docs/09_user_guide.md`',
      "[When to use streams vs contracts](/docs/language-guide#when-to-use-streams-vs-contracts)"
    )
    .replaceAll("`docs/LANGUAGE_GUIDE.md`", "[Language Guide](/docs/language-guide)")
    .replaceAll("`docs/09_user_guide.md`", "[Language Guide](/docs/language-guide)")
    .replaceAll("`docs/01_language_spec.md`", "[Language Spec](/docs/specification/language-spec)")
    .replaceAll("`docs/02_grammar.md`", "[Grammar](/docs/specification/grammar)")
    .replaceAll("`docs/04_compiler_spec.md`", "[Compiler Spec](/docs/specification/compiler-spec)")
    .replaceAll("`docs/08_diagnostics.md`", "[Diagnostics](/docs/specification/diagnostics)")
    .replaceAll("`docs/07_pack_interface.md`", "[Pack Interface](/docs/specification/pack-interface)")
    .replaceAll("`docs/cfdl_v_0_1.md`", "[Language Spec](/docs/specification/language-spec)")
    .replaceAll(
      "`docs/CFDL_v0_1_Grammar.ebnf.md`",
      "[Grammar](/docs/specification/grammar)"
    )
    .replaceAll(
      "`docs/compiler_spec_v_0_1.md`",
      "[Compiler Spec](/docs/specification/compiler-spec)"
    )
    .replaceAll(
      "`docs/diagnostics_spec.md`",
      "[Diagnostics](/docs/specification/diagnostics)"
    )
    .replaceAll(
      "`docs/pack_interface_v_0_1.md`",
      "[Pack Interface](/docs/specification/pack-interface)"
    )
    .replaceAll(
      "`docs/docs_packs_guide.md`",
      "[Packs Guide](/docs/packs)"
    )
    .replaceAll(
      "`distribution/install-configure.md`",
      "[VS Code and LSP setup](/docs/install/vscode)"
    )
    .replaceAll(
      "`examples/language_tutorial/`",
      "[language tutorial examples](/docs/examples)"
    )
    .replaceAll(
      "`docs/USER_GUIDE.md`",
      "[SDK User Guide](https://github.com/bizarc/cfdl/blob/main/docs/USER_GUIDE.md)"
    ));
}

function toPosix(p) {
  return p.split(path.sep).join("/");
}

function readSource(relativePath) {
  const absolutePath = path.resolve(repoRoot, relativePath);
  return fs.readFileSync(absolutePath, "utf8");
}

/**
 * Provenance lives in frontmatter, not on the page.
 *
 * A "generated from <path> / Source: <github url>" banner above the title told
 * readers nothing they could act on — the repo does not accept external edits,
 * so it was repo plumbing published to end users. Keeping `source` here means
 * the team can still trace any page back to its canonical file (and regenerate
 * checks still work) without rendering anything.
 */
function renderDoc(frontmatter, sourcePath, body, layer) {
  const fm = [
    "---",
    ...Object.entries(frontmatter).map(([k, v]) => `${k}: ${v}`),
    `source: ${toPosix(sourcePath)}`,
    // Ownership, for the manifest check in this file.
    "generated: full",
    // Drives the banner in app/docs/[[...slug]]/page.tsx. Emitted here rather
    // than written per page, so a new specification page cannot be added
    // without being labelled as one.
    ...(layer ? [`layer: ${layer}`] : []),
    "---",
    "",
    body.trimEnd(),
    ""
  ];
  return fm.join("\n");
}

function buildCompilerSpecDigest() {
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
    "## Related reference",
    "",
    "- [Diagnostics](/docs/specification/diagnostics) — the error code guide",
    "- [IR schema](/docs/specification/ir-schema) — canonical ordering and shape",
    "- [Pack interface](/docs/specification/pack-interface) — lowering rules",
    "- [Language spec](/docs/specification/language-spec) — validation rules",
    "",
    ""
  ].join("\n");
}

/**
 * Every page this script owns outright, recorded so the manifest check below
 * can tell a generated page from an authored one without a second list to keep
 * in step.
 */
const generatedPages = new Set();

function writeGenerated(relativePath, content) {
  generatedPages.add(toPosix(relativePath));
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
      slug: '"/docs/language-guide"'
    }
  },
  {
    source: "distribution/install-configure.md",
    output: "install/vscode.md",
    frontmatter: {
      id: "install-vscode",
      title: '"VS Code and LSP"',
      slug: '"/docs/install/vscode"'
    }
  },
  {
    source: "docs/01_language_spec.md",
    output: "specification/language-spec.md",
    layer: "specification",
    frontmatter: {
      id: "language-spec",
      title: '"Language Spec (v0.1)"',
      slug: '"/docs/specification/language-spec"'
    }
  },
  {
    source: "docs/02_grammar.md",
    output: "specification/grammar.md",
    layer: "specification",
    frontmatter: {
      id: "grammar",
      title: '"Grammar (EBNF)"',
      slug: '"/docs/specification/grammar"'
    }
  },
  {
    source: "docs/04_compiler_spec.md",
    output: "specification/compiler-spec.md",
    layer: "specification",
    frontmatter: {
      id: "compiler-spec",
      title: '"Compiler Spec (v0.1)"',
      slug: '"/docs/specification/compiler-spec"'
    },
    digestOnly: true
  },
  {
    source: "docs/08_diagnostics.md",
    output: "specification/diagnostics.md",
    layer: "specification",
    frontmatter: {
      id: "diagnostics",
      title: '"Diagnostics Reference"',
      slug: '"/docs/specification/diagnostics"'
    }
  },
  {
    source: "docs/07_pack_interface.md",
    output: "specification/pack-interface.md",
    layer: "specification",
    frontmatter: {
      id: "pack-interface",
      title: '"Pack Interface (v0.1)"',
      slug: '"/docs/specification/pack-interface"'
    }
  },
  {
    source: "docs/03_expression_environment.md",
    output: "specification/expression-environment.md",
    layer: "specification",
    frontmatter: {
      id: "expression-environment",
      title: '"Expression Environment (v0.1)"',
      slug: '"/docs/specification/expression-environment"'
    }
  },
  {
    source: "docs/05_ir_schema.md",
    output: "specification/ir-schema.md",
    layer: "specification",
    frontmatter: {
      id: "ir-schema",
      title: '"IR Schema (v0.1)"',
      slug: '"/docs/specification/ir-schema"'
    }
  },
  {
    source: "docs/06_results_schema.md",
    output: "specification/results-schema.md",
    layer: "specification",
    frontmatter: {
      id: "results-schema",
      title: '"Results Schema (v0.1)"',
      slug: '"/docs/specification/results-schema"'
    }
  }
];

for (const spec of docSpecs) {
  let body = spec.digestOnly
    ? buildCompilerSpecDigest()
    : normalizeLinks(stripLeadingH1(readSource(spec.source)));
  const rendered = renderDoc(spec.frontmatter, spec.source, body, spec.layer);
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
  'slug: "/docs/examples"',
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
    `slug: "/docs/examples/${name}"`,
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
  exampleIndexLines.push(`- [${exampleTitle(name)}](/docs/examples/${name})`);
}

exampleIndexLines.push("");
exampleIndexLines.push("## Domain examples");
exampleIndexLines.push("");
exampleIndexLines.push("- [CRE examples](/docs/examples/cre-examples) — Commercial Real Estate: lease-up, full lifecycle, phased, multi-file, development with financing.");
exampleIndexLines.push("- [Operating Business examples](/docs/examples/operating-business-examples) — OpCo: revenue, opex, working capital, exit multiple, growth, multi-file.");
exampleIndexLines.push("");

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
    `slug: "/docs/examples/${name}"`,
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
  'slug: "/docs/language-reference"',
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
  "- [Language Spec](/docs/specification/language-spec)",
  "- [Grammar](/docs/specification/grammar)",
  "- [Expression Environment](/docs/specification/expression-environment)",
  "- [Compiler Spec](/docs/specification/compiler-spec)",
  "- [Diagnostics](/docs/specification/diagnostics)",
  "- [Pack Interface](/docs/specification/pack-interface)",
  "- [Implementation Status](/docs/specification/implementation-status)",
  "",
  "## Tools and data contracts",
  "",
  "- [CLI Reference](/docs/reference/cli)",
  "- [Run-Config Reference](/docs/reference/run-config)",
  "- [IR Schema](/docs/specification/ir-schema)",
  "- [Results Schema](/docs/specification/results-schema)",
  ""
].join("\n");

// --- Cookbooks: one page per pack, synced from packs/<pack>/README.md -------
// --- Benchmark cases, discovered once and used for both the worked-example
// --- pages below and the methodology page further down. --------------------
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

// The benchmark models are the strongest examples in the repository: each is
// diffed period-by-period against an independent reference implementation, so
// their numbers are verified rather than asserted. Publishing them gives every
// pack the same depth of worked example — energy and credit previously had
// none — without inventing models that nothing validates.

const benchmarkTitles = {
  "cre/office_two_tenant": "CRE: two-tenant office",
  "cre/retail_strip": "CRE: retail strip with expense stops",
  "credit/level_pay_pool": "Credit: level-pay auto pool",
  "credit/io_bullet_loan": "Credit: IO/bullet bridge loan",
  "credit/float_bridge_pool": "Credit: floating-rate bridge pool",
  "energy/solar_ppa_microgrid": "Energy: solar PPA microgrid",
  "energy/wind_ptc_macrs": "Energy: wind with PTC and MACRS",
  "opco/lbo_buyout": "OpCo: leveraged buyout"
};

/** The description a case states in the leading comments of case.toml. */
/**
 * A benchmark's one-line description, as the case DECLARES it.
 *
 * This used to scrape the leading `#` comments of case.toml and print them as
 * page prose. Those comments are maintainer's notes — tolerance archaeology,
 * why a figure moved, which line was wrong and for how long — and publishing
 * them put a wall of shouty-caps engineering narrative at the top of the HUD
 * page, which is one of the strongest things there is to show a reader.
 *
 * A comment is written for whoever opens the file next. A `summary` is written
 * for whoever reads the site. Keeping them separate is the whole of the
 * generate-data-author-prose rule: a TOML comment is not data.
 */
function benchmarkSummary(caseDir) {
  const raw = fs.readFileSync(path.resolve(caseDir, "case.toml"), "utf8");
  const match = raw.match(/^summary\s*=\s*"([^"]*)"/m);
  if (!match) {
    const rel = toPosix(path.relative(repoRoot, path.resolve(caseDir, "case.toml")));
    throw new Error(
      `${rel}: no \`summary\` field.\n` +
        `Every benchmark case declares a one-sentence summary for its page. ` +
        `Comments in this file are notes for maintainers and are not published.`,
    );
  }
  return match[1].trim();
}

function formatMetricValue(value) {
  return Math.abs(value) >= 1000
    ? value.toLocaleString("en-US", { maximumFractionDigits: 2 })
    : String(value);
}

const benchmarkExampleLinks = {};

for (const { pack, name } of benchCases) {
  const caseDir = path.resolve(benchRoot, pack, name);
  const key = `${pack}/${name}`;
  const slug = `${pack}-${name.replaceAll("_", "-")}`;
  const title = benchmarkTitles[key] ?? `${pack}: ${name.replaceAll("_", " ")}`;

  const model = fs.readFileSync(path.resolve(caseDir, "model.cfdl"), "utf8").trimEnd();
  const runConfig = fs.readFileSync(path.resolve(caseDir, "run.json"), "utf8").trimEnd();
  const metrics = JSON.parse(
    fs.readFileSync(path.resolve(caseDir, "expected_metrics.json"), "utf8")
  );

  writeGenerated(
    `examples/${slug}.md`,
    [
      "---",
      `id: benchmark-${slug}`,
      `title: "${title}"`,
      `slug: "/docs/examples/${slug}"`,
      `source: benchmarks/${key}`,
      "---",
      "",
      `# ${title}`,
      "",
      benchmarkSummary(caseDir),
      "",
      "Every number below is checked against an independent reference",
      "implementation on every commit — period by period, and on each metric,",
      "inside a declared tolerance. See [benchmark methodology](/docs/benchmarks).",
      "",
      "## The model",
      "",
      "```cfdl",
      model,
      "```",
      "",
      "## Run configuration",
      "",
      "```json",
      runConfig,
      "```",
      "",
      "## Verified results",
      "",
      "| Metric | Value | Tolerance |",
      "|---|---:|---:|",
      ...Object.entries(metrics).map(
        ([metric, spec]) =>
          `| \`${metric}\` | ${formatMetricValue(spec.value)} | ±${spec.tolerance} |`
      ),
      ""
    ].join("\n")
  );

  (benchmarkExampleLinks[pack] ??= []).push([title, `/docs/examples/${slug}`]);
}

exampleIndexLines.push("");
exampleIndexLines.push("## Benchmark models");
exampleIndexLines.push("");
exampleIndexLines.push(
  "Complete models for every pack, each checked period-by-period against an " +
    "independent reference implementation. These are the most detailed examples " +
    "on the site, and their numbers are verified rather than asserted."
);
exampleIndexLines.push("");
for (const pack of ["energy", "cre", "credit", "opco"]) {
  for (const [label, href] of benchmarkExampleLinks[pack] ?? []) {
    exampleIndexLines.push(`- [${label}](${href})`);
  }
}
writeGenerated("examples/index.md", exampleIndexLines.join("\n"));


/**
 * Domain metrics and validations, rendered from the pack's own TOML.
 *
 * Both were largely absent from the guides — a reader could see which
 * contracts a pack offered but not what it computed from them, nor what it
 * refuses to accept. Generating the tables from the source of truth means the
 * guide states exactly what the pack implements, and cannot drift from it.
 */
function parsePackMetrics(pack) {
  const file = path.resolve(repoRoot, `packs/${pack}/metrics.toml`);
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .split("[[metrics]]")
    .slice(1)
    .map((block) => {
      // `formula` is almost always the literal "sum(numerator_streams)", which
      // tells a reader nothing; the streams themselves are the useful part.
      const streams = [...block.matchAll(/"([a-z_]+\.[a-z_.]+)"/g)]
        .map((m) => m[1])
        .filter((s) => !s.startsWith("domain."));
      // A true ratio declares op = "ratio" over two other metrics;
      // denominator_streams just means those streams are netted off.
      const op = (block.match(/^op = "([^"]+)"/m) ?? [])[1] ?? "";
      const overMetrics = [
        (block.match(/^numerator_metric = "([^"]+)"/m) ?? [])[1],
        (block.match(/^denominator_metric = "([^"]+)"/m) ?? [])[1]
      ].filter(Boolean);
      return {
        id: (block.match(/^id = "([^"]+)"/m) ?? [])[1],
        kind: (block.match(/^kind = "([^"]+)"/m) ?? [])[1] ?? "",
        streams: [...new Set(streams)],
        op,
        overMetrics
      };
    })
    .filter((m) => m.id);
}

function parsePackValidations(pack) {
  const file = path.resolve(repoRoot, `packs/${pack}/validations.toml`);
  if (!fs.existsSync(file)) return [];
  return fs
    .readFileSync(file, "utf8")
    .split("[[validations]]")
    .slice(1)
    .map((block) => ({
      code: (block.match(/^code = "([^"]+)"/m) ?? [])[1],
      message: (block.match(/^message = "([^"]+)"/m) ?? [])[1] ?? ""
    }))
    .filter((v) => v.code);
}

function packReferenceSections(pack) {
  const metrics = parsePackMetrics(pack);
  const validations = parsePackValidations(pack);
  const out = [];

  if (metrics.length) {
    out.push(
      "",
      "## Metrics reference",
      "",
      `Computed automatically whenever a model runs with the \`${pack}\` pack, ` +
        "alongside the core metrics (NPV, IRR, MOIC, payback, WAL). " +
        "Enumerated from the pack definition, so this list is always complete.",
      "",
      "| Metric | Type | Built from |",
      "|---|---|---|",
      ...metrics.map((m) => {
        const from = m.overMetrics.length
          ? m.overMetrics.map((x) => `\`${x}\``).join(" ÷ ")
          : m.streams.length
            ? m.streams.map((s) => `\`${s}\``).join(", ")
            : "derived";
        return `| \`${m.id}\` | ${m.kind} | ${from} |`;
      })
    );
  }

  if (validations.length) {
    out.push(
      "",
      "## Validations reference",
      "",
      "Checked at compile time. Each is a stable diagnostic code that is never " +
        "renamed or reused; see [diagnostics](/docs/specification/diagnostics).",
      "",
      "| Code | Rejects |",
      "|---|---|",
      ...validations.map((v) => `| \`${v.code}\` | ${v.message} |`)
    );
  }

  return out.length ? out.join("\n") + "\n" : "";
}

const cookbookPacks = ["energy", "cre", "credit", "opco"];
const packTitles = { energy: "Energy", cre: "CRE", credit: "Credit", opco: "OpCo" };
const cookbookIndexLines = [
  "---",
  "id: cookbooks",
  'title: "Cookbooks"',
  'slug: "/docs/cookbooks"',
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
    ["CRE examples overview", "/docs/examples/cre-examples"],
    ["Lease-up", "/docs/examples/cre_lease_up"],
    ["Developer lifecycle", "/docs/examples/cre_developer"],
    ["Phased development", "/docs/examples/cre_phased"],
    ["Multi-file model", "/docs/examples/cre_multi_file"],
    ["Development with financing", "/docs/examples/cre_development_with_financing"]
  ],
  opco: [
    ["Operating Business examples overview", "/docs/examples/operating-business-examples"],
    ["Basic OpCo", "/docs/examples/opco_basic"],
    ["Growth via expressions", "/docs/examples/opco_with_growth"],
    ["Multi-file model", "/docs/examples/opco_multi_file"]
  ]
};

for (const pack of cookbookPacks) {
  const readmePath = path.resolve(repoRoot, `packs/${pack}/README.md`);
  if (!fs.existsSync(readmePath)) continue;
  let body = normalizeLinks(stripLeadingH1(fs.readFileSync(readmePath, "utf8")));
  const worked = [
    ...(benchmarkExampleLinks[pack] ?? []),
    ...(packWorkedExamples[pack] ?? [])
  ];
  body += packReferenceSections(pack);

  if (worked.length) {
    body +=
      "\n## Worked example models\n\n" +
      "Benchmark cases are validated period-by-period against an independent\n" +
      "reference implementation.\n\n" +
      worked.map(([label, href]) => `- [${label}](${href})`).join("\n") +
      "\n";
  }
  const page = renderDoc(
    {
      id: `cookbook-${pack}`,
      title: `"${packTitles[pack] ?? pack} pack guide"`,
      slug: `"/docs/cookbooks/${pack}"`
    },
    `packs/${pack}/README.md`,
    body
  );
  writeGenerated(`cookbooks/${pack}.md`, page);
  cookbookIndexLines.push(`- [${packTitles[pack] ?? pack} pack guide](/docs/cookbooks/${pack})`);
}

cookbookIndexLines.push("");
cookbookIndexLines.push("## Example notebooks");
cookbookIndexLines.push("");
cookbookIndexLines.push(
  "One Jupyter notebook per pack walks a benchmark model through the Python " +
    "SDK. Each is published with the outputs and chart it actually produced:"
);
cookbookIndexLines.push("");
for (const [title, slug] of [
  ["Solar PPA microgrid", "energy-solar-microgrid"],
  ["CRE office acquisition", "cre-office-acquisition"],
  ["Credit loan pool", "credit-loan-pool"],
  ["Operating company LBO", "opco-lbo"]
]) {
  cookbookIndexLines.push(`- [${title}](/docs/notebooks/${slug})`);
}
cookbookIndexLines.push("");

writeGenerated("cookbooks/index.md", cookbookIndexLines.join("\n"));

// --- Benchmark methodology page --------------------------------------------

const benchLines = [
  "---",
  "id: benchmarks",
  'title: "Benchmark methodology"',
  'slug: "/docs/benchmarks"',
  "---",
  "",
  "Every pack is gated by a parity suite: each CFDL model is diffed against an",
  "**independent reference**, period-by-period and on summary metrics, inside a",
  "tolerance the case declares.",
  "",
  "Two kinds of reference, and the difference matters. Most cases carry a",
  "`reference_gen.py` — a second implementation written against the same",
  "specification, which catches arithmetic and lowering errors. Some are",
  "reconciled instead against an **external** model or published schedule, and",
  "those carry a `NOTES.md` and no generator: two of your own implementations",
  "agreeing is not evidence when both came from one assumption, and every",
  "convention defect found so far has come from the external kind.",
  "",
  "## How a case is built",
  "",
  "Each `benchmarks/<pack>/<case>/` directory contains:",
  "",
  "- `model.cfdl` — the CFDL model;",
  "- `run.json` — the run configuration;",
  "- `case.toml` — the pack name and per-period tolerance;",
  "- `expected.csv` — period-level expectations from the reference: the model",
  "  total, or each stream in its own column;",
  "- `expected_metrics.json` — summary metrics, each with its own tolerance;",
  "- either `reference_gen.py`, the independent implementation that produces the",
  "  expected files, or `NOTES.md`, recording the external reconciliation — what",
  "  was compared, what diverged, and how to repeat it.",
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
  "> Each case says in its `case.toml` where its figures came from, and which",
  "> are still awaiting practitioner verification.",
  ""
];
writeGenerated("benchmarks.md", benchLines.join("\n"));

// --- Stage JSON schemas at their $id path (static/schemas/...) --------------
const schemaStaticDir = path.resolve(siteDir, "public", "schemas");
// The grammar is staged alongside the JSON schemas so the reference page can
// offer a download instead of sending readers into the repository.
const stagedSchemas = {
  "CFDL_v0_1_IR.schema.json": "ir.schema.json",
  "CFDL_v0_1_Results.schema.json": "results.schema.json",
  "CFDL_v0_1_Grammar.ebnf": "CFDL_v0_1_Grammar.ebnf"
};
for (const [schema, sourceName] of Object.entries(stagedSchemas)) {
  const src = path.resolve(repoRoot, "docs", "schemas", sourceName);
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

// --- Manifest: every page is owned by someone, and says so -----------------
//
// A page under content/docs is either written by this script, or authored by a
// human who declares it with `generated: none` (all bytes theirs) or
// `generated: regions` (theirs except the marked blocks).
//
// The failure this catches is the one a restructure creates: move a generated
// page to a new slug and the old file stays behind, still served, now orphaned
// from its source and quietly frozen. Nothing else notices — it is a valid
// markdown file with valid frontmatter, and `check-links` is happy because it
// still resolves. Requiring an owner makes the orphan a build failure.
function walkPages(dir, acc = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.resolve(dir, entry.name);
    if (entry.isDirectory()) walkPages(full, acc);
    else if (entry.name.endsWith(".md")) acc.push(full);
  }
  return acc;
}

const unowned = [];
for (const file of walkPages(docsOutputRoot)) {
  const rel = toPosix(path.relative(docsOutputRoot, file));
  if (generatedPages.has(rel)) continue;
  const head = fs.readFileSync(file, "utf8").slice(0, 600);
  if (/^generated:\s*(none|regions|full)\s*$/m.test(head)) continue;
  unowned.push(rel);
}
if (unowned.length > 0) {
  throw new Error(
    "Pages with no declared owner:\n" +
      unowned.map((f) => `  content/docs/${f}`).join("\n") +
      "\n\nEach page is either generated by this script, or authored. An authored " +
      "page must say so in its frontmatter:\n" +
      "  generated: none      all bytes are yours\n" +
      "  generated: regions   yours, except blocks marked <!-- cfdl:generated <key> -->\n" +
      "\nA page left behind by a slug move shows up here rather than being served " +
      "forever with no source.",
  );
}

if (checkMode) {
  console.log(`content sync check passed (${generatedPages.size} generated pages)`);
} else {
  console.log(`content sync completed (${generatedPages.size} generated pages)`);
}
