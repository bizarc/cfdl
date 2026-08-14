#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { syncRegion } from "./sync/regions.mjs";

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
    .replaceAll("`docs/LANGUAGE_GUIDE.md`", "[Language guide](/docs/language-guide)")
    .replaceAll("`docs/09_user_guide.md`", "[Language guide](/docs/language-guide)")
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
    // The site does not link into the repository. Every rewrite above points
    // at a page on this site; a source that cites a repository file that has
    // no published counterpart is rewritten to the nearest page instead.
    .replaceAll("`docs/USER_GUIDE.md`", "[Python SDK](/docs/python-sdk)"));
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

/**
 * A fence opener that carries the model's own run config.
 *
 * The playground applies a default rate to anything arriving without one, so a
 * reader who clicks "open in playground" from a page stating an NPV would meet
 * a different number — which reads as the engine disagreeing with the
 * documentation rather than as two different assumptions.
 *
 * Emitted on one line because a fence's meta string is the only channel that
 * survives markdown into the renderer.
 */
function cfdlFence(dir) {
  const runPath = path.resolve(dir, "run.json");
  if (!fs.existsSync(runPath)) return "```cfdl";
  try {
    const config = JSON.parse(fs.readFileSync(runPath, "utf8"));
    return "```cfdl run=" + JSON.stringify(config);
  } catch {
    // A model whose run.json will not parse is a problem for the benchmark
    // runner to report, not a reason to emit a broken page.
    return "```cfdl";
  }
}

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

/**
 * Titles are SENTENCE CASE. Capitalise the first word and proper nouns; leave
 * the rest lower. `CRE`, `PPA`, `MACRS`, `IO` and pack names keep their case
 * because they are names, not emphasis.
 *
 * The set had drifted three ways at once — Title Case on the older tutorial
 * pages, sentence case on the newer benchmark pages, and bare slug text
 * ("credit: auto abs speed 050") wherever a page had no entry here at all. A
 * missing entry now fails rather than falling through to the slug, because
 * the fallback was invisible: it produced a plausible-looking title and only
 * looked wrong next to its neighbours.
 */
const exampleTitles = {
  minimal_model: "Minimal model",
  first_stream: "Your first stream",
  simple_contract: "A simple contract",
  with_pack: "Using an industry pack",
  multi_file: "Multi-file model",
  curves: "Curves",
  uncertainty: "Uncertainty and Monte Carlo",
  options_events: "Events and options",
  cre_lease_up: "CRE: lease-up",
  cre_developer: "CRE: developer lifecycle",
  cre_phased: "CRE: phased development",
  cre_multi_file: "CRE: multi-file model",
  cre_development_with_financing: "CRE: development with financing",
  opco_basic: "OpCo: basic operating model",
  opco_with_growth: "OpCo: growth via expressions",
  opco_multi_file: "OpCo: multi-file model"
};

function exampleTitle(name) {
  const title = exampleTitles[name];
  if (!title) {
    throw new Error(
      `examples/${name}: no title in exampleTitles (site/scripts/sync-content.mjs).\n` +
        `Add one in sentence case — a slug read as a title is how ` +
        `"credit: auto abs speed 050" reached the site.`,
    );
  }
  return title;
}

const docSpecs = [
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
      title: '"Language spec (v0.1)"',
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
      title: '"Compiler spec (v0.1)"',
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
      title: '"Diagnostics reference"',
      slug: '"/docs/specification/diagnostics"'
    }
  },
  {
    source: "docs/07_pack_interface.md",
    output: "specification/pack-interface.md",
    layer: "specification",
    frontmatter: {
      id: "pack-interface",
      title: '"Pack interface (v0.1)"',
      slug: '"/docs/specification/pack-interface"'
    }
  },
  {
    source: "docs/03_expression_environment.md",
    output: "specification/expression-environment.md",
    layer: "specification",
    frontmatter: {
      id: "expression-environment",
      title: '"Expression environment (v0.1)"',
      slug: '"/docs/specification/expression-environment"'
    }
  },
  {
    source: "docs/05_ir_schema.md",
    output: "specification/ir-schema.md",
    layer: "specification",
    frontmatter: {
      id: "ir-schema",
      title: '"IR schema (v0.1)"',
      slug: '"/docs/specification/ir-schema"'
    }
  },
  {
    source: "docs/06_results_schema.md",
    output: "specification/results-schema.md",
    layer: "specification",
    frontmatter: {
      id: "results-schema",
      title: '"Results schema (v0.1)"',
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
  "multi_file",
  "curves",
  "uncertainty",
  "options_events"
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
  "Every example on this page is a complete model that runs. They come in three",
  "kinds: eight short lessons that build the language one construct at a time,",
  "a few longer domain models, and twenty-five benchmark models checked against",
  "published references.",
  "",
  "## Lessons",
  "",
  "Read in order. Each adds one construct to the model before it.",
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
    readme.trimEnd(),
    "",
    "## model.cfdl",
    "",
    cfdlFence(path.resolve(exampleRoot, name)),
    model,
    "```",
    ""
  ].join("\n");

  writeGenerated(`examples/${name}.md`, examplePage);
  exampleIndexLines.push(`- [${exampleTitle(name)}](/docs/examples/${name})`);
}

exampleIndexLines.push("");
exampleIndexLines.push("## Domain models");
exampleIndexLines.push("");
exampleIndexLines.push("Longer models that put the constructs together.");
exampleIndexLines.push("");
exampleIndexLines.push("- [CRE examples](/docs/examples/cre-examples) — lease-up, developer lifecycle, phased development, multi-file, development with financing.");
exampleIndexLines.push("- [Operating business examples](/docs/examples/operating-business-examples) — revenue, opex, working capital, exit multiple, growth, multi-file.");
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

  // No provenance banner. The site is the product's documentation and stands
  // on its own: a reader has no repository to look in, and naming one implies
  // a place to go that does not exist for them.
  const body = [];

  for (const file of cfdlFiles) {
    const content = fs.readFileSync(path.resolve(dir, file), "utf8").trimEnd();
    body.push(`## ${file}`);
    body.push("");
    body.push(cfdlFence(dir));
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
  "- [CLI reference](/docs/reference/cli)",
  "- [Run-config reference](/docs/reference/run-config)",
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
  "bespoke/ppiaf_toll_highway": "Bespoke: tolled highway PPP concession",
  "cre/hud_home_multifamily": "CRE: HOME-funded affordable multifamily",
  "cre/mit_rentleg_plaza": "CRE: rent-regulated plaza",
  "cre/office_two_tenant": "CRE: two-tenant office",
  "cre/one_lincoln_street": "CRE: office development joint venture",
  "cre/one_lincoln_street_contract":
    "CRE: office development, through the pack contract",
  "cre/retail_strip": "CRE: retail strip with expense stops",
  "credit/auto_abs_speed_050": "Credit: auto ABS at 0.5x prepayment speed",
  "credit/auto_abs_speed_150": "Credit: auto ABS at 1.5x prepayment speed",
  "credit/auto_abs_tranches": "Credit: auto ABS note classes",
  "credit/auto_abs_wal": "Credit: auto ABS weighted average life",
  "credit/float_bridge_pool": "Credit: floating-rate bridge pool",
  "credit/fnma_remic_2019_2_g3": "Credit: Fannie Mae REMIC with a stripped coupon",
  "credit/fnma_remic_2019_2_g3_psa000": "Credit: Fannie Mae REMIC at 0% PSA",
  "credit/fnma_remic_2019_2_g3_psa100": "Credit: Fannie Mae REMIC at 100% PSA",
  "credit/fnma_remic_2019_2_g3_psa300": "Credit: Fannie Mae REMIC at 300% PSA",
  "credit/fnma_remic_2019_2_g3_psa400": "Credit: Fannie Mae REMIC at 400% PSA",
  "credit/fnma_remic_2019_2_g3_psa700": "Credit: Fannie Mae REMIC at 700% PSA",
  "credit/fnma_remic_2019_2_g3_psa1000": "Credit: Fannie Mae REMIC at 1000% PSA",
  "credit/io_bullet_loan": "Credit: IO/bullet bridge loan",
  "credit/level_pay_pool": "Credit: level-pay auto pool",
  "credit/mbs_pool_by_loan": "Credit: a mortgage pool modeled loan by loan",
  "credit/mbs_pool_conventions": "Credit: mortgage pool conventions",
  "credit/mbs_pool_ramped": "Credit: mortgage pool on a prepayment ramp",
  "energy/crest_solar_cost_based": "Energy: cost-based solar feed-in tariff",
  "energy/merchant_capacity": "Energy: merchant generator with capacity revenue",
  "energy/solar_ppa_microgrid": "Energy: solar PPA microgrid",
  "energy/utility_pv_singleowner": "Energy: utility-scale PV, single owner",
  "energy/tax_equity_flip": "Energy: a tax-equity flip, with the date derived",
  "energy/wind_ptc_macrs": "Energy: wind with PTC and MACRS",
  "opco/banker_dcf_conventions": "OpCo: banker DCF conventions",
  "opco/damodaran_fcff": "OpCo: free cash flow to firm",
  "opco/gordon_growth_coned": "OpCo: stable-growth dividend discount",
  "opco/lbo_buyout": "OpCo: leveraged buyout",
  "opco/lbo_circular_interest": "OpCo: LBO debt schedule with average-balance interest",
  "opco/lbo_financing_cases": "OpCo: one buyout at three capital structures",
  "opco/lbo_option_pool_exit": "OpCo: LBO exit waterfall with an option pool",
  "opco/saas_sbc_convention_fork": "OpCo: SaaS DCF and the stock-compensation fork"
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

// The published description of a case: what the deal is, what the reference is,
// what the case exercises, how well it matched, and what any residual means.
//
// A separate file from `case.toml` because it is prose, and separate from
// NOTES.md because NOTES is maintainer narrative and is deliberately not
// published — the same split `summary` and this file's comments already keep.
// Optional while the set is being written; `check-benchmark-cases.py` is what
// requires it.
function benchmarkCase(caseDir) {
  const file = path.resolve(caseDir, "CASE.md");
  if (!fs.existsSync(file)) return null;
  return fs.readFileSync(file, "utf8").trimEnd();
}

/**
 * What the case actually checks, whichever form that takes.
 *
 * Eight of the twenty-five cases assert no summary metric: their reference
 * publishes a full cash-flow table, so the assertion is the table itself,
 * every line in every period. Rendering only the metric table left those pages
 * with a "Verified results" heading and nothing under it, which reads as
 * nothing having been verified — the opposite of the truth. The period-level
 * assertion is stated first for every case, and the metric table follows when
 * the case has one.
 */
/**
 * The tolerance actually applied to each asserted series, and how many values
 * are asserted at all.
 *
 * `case.toml` carries a `period_tolerance` default and an optional
 * `[tolerance]` table overriding it per column; `tools/benchmark-runner.py`
 * falls back to 0.01 when neither is given, and this mirrors that so the page
 * states what the harness enforces rather than what the file happens to say
 * first.
 *
 * A blank cell in `expected.csv` is "not asserted", so series x periods
 * overcounts every sparse case — 6 x 60 reads as 360 checks where 180 are made.
 */
function caseTolerances(caseDir, columns, rows) {
  const toml = fs.readFileSync(path.resolve(caseDir, "case.toml"), "utf8");
  const fallback = (toml.match(/^period_tolerance\s*=\s*(\S+)/m) ?? [])[1] ?? "0.01";
  const overrides = {};
  // The `[tolerance]` table, read to the next table header or end of file.
  // JavaScript has no `\Z`, so end-of-input is `$` asserted with nothing left
  // after it — the obvious spelling silently matched nothing at all.
  const table = toml.match(
    /^\[tolerance\][^\n]*\n([\s\S]*?)(?=^\[[^\n]*\]\s*$|$(?![\s\S]))/m
  );
  if (table) {
    for (const line of table[1].split("\n")) {
      const kv = line.match(/^\s*(?:"([^"]+)"|([\w.]+))\s*=\s*(\S+)/);
      if (kv) overrides[kv[1] ?? kv[2]] = kv[3];
    }
  }
  const tolerances = Object.fromEntries(
    columns.map((c) => [c, overrides[c] ?? fallback])
  );
  let asserted = 0;
  for (const row of rows.slice(1)) {
    for (const cell of row.split(",").slice(1)) {
      if (cell.trim() !== "") asserted += 1;
    }
  }
  return { tolerances, asserted };
}

function verifiedResults(caseDir, metrics) {
  const lines = [];
  const csv = path.resolve(caseDir, "expected.csv");
  if (fs.existsSync(csv)) {
    const rows = fs
      .readFileSync(csv, "utf8")
      .split("\n")
      .filter((line) => line.trim() !== "");
    const columns = rows[0].split(",").slice(1).map((c) => c.trim());
    const { tolerances, asserted } = caseTolerances(caseDir, columns, rows);
    // ONE NUMBER ONLY WHEN ONE NUMBER IS TRUE. `period_tolerance` is the
    // default, not the rule: six of the cases override it per column, and
    // printing the default as though it governed every series misstated them
    // in both directions — `auto_abs_tranches` claimed ±0.01 while checking
    // its classes at 1,375 to 27,137, and `fnma_remic_2019_2_g3` claimed
    // ±741,862 while pinning its residual to a cent.
    const distinct = [...new Set(columns.map((c) => tolerances[c]))];
    const uniform = distinct.length === 1 ? distinct[0] : null;
    lines.push(
      `Checked period by period: **${columns.length} series** across ` +
        `**${rows.length - 1} periods** — **${asserted} values** in all` +
        (uniform
          ? `, each within ±${uniform} of the reference.`
          : `, each within the tolerance shown.`),
      "",
      ...columns.map(
        (column) =>
          `- \`${column}\`` + (uniform ? "" : ` — within ±${tolerances[column]}`)
      ),
      ""
    );
  }
  // Scenario assertions, when the case declares them. A case whose whole
  // subject is how a number moves with an input published only its base column
  // — the two variants that make the point were checked on every commit and
  // shown nowhere.
  const scenariosPath = path.resolve(caseDir, "expected_scenarios.json");
  if (fs.existsSync(scenariosPath)) {
    const scenarios = JSON.parse(fs.readFileSync(scenariosPath, "utf8"));
    const names = Object.keys(scenarios);
    const columns = [...new Set(names.flatMap((n) => Object.keys(scenarios[n])))];
    lines.push(
      "Checked per scenario, each a full run under its own parameters:",
      "",
      `| Scenario | ${columns.map((c) => `\`${c}\``).join(" | ")} |`,
      `|---|${columns.map(() => "---:").join("|")}|`,
      ...names.map(
        (n) =>
          `| \`${n}\` | ` +
          columns
            .map((c) => (scenarios[n][c] ? formatMetricValue(scenarios[n][c].value) : "—"))
            .join(" | ") +
          " |",
      ),
      "",
    );
  }

  if (Object.keys(metrics).length > 0) {
    lines.push(
      "Summary metrics for the base run:",
      "",
      "| Metric | Value | Tolerance |",
      "|---|---:|---:|",
      ...Object.entries(metrics).map(
        ([metric, spec]) =>
          `| \`${metric}\` | ${formatMetricValue(spec.value)} | ±${spec.tolerance} |`
      )
    );
  }
  return lines;
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
  const title = benchmarkTitles[key];
  if (!title) {
    throw new Error(
      `benchmarks/${key}: no title in benchmarkTitles (site/scripts/sync-content.mjs).\n` +
        `Add one in sentence case, prefixed by the pack — "OpCo: leveraged buyout".`,
    );
  }

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
      ...(benchmarkCase(caseDir) ? [benchmarkCase(caseDir), ""] : []),
      "## The model",
      "",
      cfdlFence(caseDir),
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
      ...verifiedResults(caseDir, metrics),
      ""
    ].join("\n")
  );

  (benchmarkExampleLinks[pack] ??= []).push({
    title,
    href: `/docs/examples/${slug}`,
    summary: benchmarkSummary(caseDir),
  });
}

// The order the benchmark groups are presented in, on both the examples index
// and the benchmarks table. `bespoke` is not a pack — it holds cases written
// from the bare language — so it sits last, after the four packs.
const benchmarkGroups = ["energy", "cre", "credit", "opco", "bespoke"];

const packLabels = {
  bespoke: "Without a pack",
  energy: "Energy",
  cre: "Commercial real estate",
  credit: "Credit",
  opco: "Operating businesses",
};

exampleIndexLines.push("");
exampleIndexLines.push("## Benchmark models");
exampleIndexLines.push("");
exampleIndexLines.push(
  "Complete models for every pack, each checked period by period against an " +
    "independent reference implementation. These detailed examples have been " +
    "verified. How that is done is on the [validation](/docs/benchmarks) page."
);
// Grouped by pack, each with the line the case declares about itself. A flat
// list of twenty-five titles asked the reader to guess from a title alone
// which model was the one they wanted.
for (const pack of benchmarkGroups) {
  const cases = benchmarkExampleLinks[pack] ?? [];
  if (cases.length === 0) continue;
  exampleIndexLines.push("");
  exampleIndexLines.push(`### ${packLabels[pack]}`);
  exampleIndexLines.push("");
  for (const { title, href, summary } of cases) {
    exampleIndexLines.push(`- [${title}](${href}) — ${summary}`);
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

const packGuides = ["energy", "cre", "credit", "opco"];
const packTitles = { energy: "Energy", cre: "CRE", credit: "Credit", opco: "OpCo" };

// Domain example pages folded into each pack guide (instead of separate
// sidebar entries next to the pack guides).
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

for (const pack of packGuides) {
  const readmePath = path.resolve(repoRoot, `packs/${pack}/README.md`);
  if (!fs.existsSync(readmePath)) continue;
  let body = normalizeLinks(stripLeadingH1(fs.readFileSync(readmePath, "utf8")));
  const worked = [
    ...(benchmarkExampleLinks[pack] ?? []),
    ...(packWorkedExamples[pack] ?? [])
  ];
}


// --- Benchmark methodology page --------------------------------------------


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

// --- Generated data regions inside authored pages ---------------------------
//
// GENERATE DATA, AUTHOR PROSE. A diagnostic register or a schema's field list
// is data — derivable, and wrong the moment it is copied by hand. The sentences
// around it are not. A region lets both live on one page: the script owns the
// bytes between its markers and nothing else, so the prose survives every
// rebuild and the table cannot fall behind its source.

/** Every code the specification's register declares, with the family it belongs to. */
function diagnosticsCatalogue() {
  const src = fs.readFileSync(path.resolve(repoRoot, "docs", "08_diagnostics.md"), "utf8");
  const rows = [];
  let family = "";
  let inRegister = false;
  for (const line of src.split("\n")) {
    if (/^##\s+7\)/.test(line)) { inRegister = true; continue; }
    if (inRegister && /^##\s+(?!#)/.test(line)) break;
    if (!inRegister) continue;
    const heading = line.match(/^###\s+7\.\d+\s+(.+?)\s*\(/);
    if (heading) { family = heading[1]; continue; }
    const code = line.match(/^-\s+`([EWI]\d+)_([A-Z0-9_]+)`\s*(?:—\s*(.*))?$/);
    if (code) {
      rows.push({ code: `${code[1]}_${code[2]}`, family, meaning: (code[3] ?? "").trim() });
      continue;
    }
    // The register wraps a long description onto continuation lines.
    if (rows.length > 0 && /^\s{2,}\S/.test(line)) {
      const last = rows[rows.length - 1];
      last.meaning = `${last.meaning} ${line.trim()}`.trim();
    }
  }
  if (rows.length === 0) {
    throw new Error("docs/08_diagnostics.md: no codes found in the register (section 7).");
  }
  const seen = new Set();
  const out = ["| Code | Family | Meaning |", "|---|---|---|"];
  for (const { code, family: f, meaning } of rows) {
    if (seen.has(code)) continue;
    seen.add(code);
    // Pipes would break the table; the register writes prose, not cells.
    const text = (meaning || "").replace(/\|/g, "\\|");
    out.push(`| \`${code}\` | ${f} | ${text} |`);
  }
  out.push("");
  out.push(`*${seen.size} codes.*`);
  return out;
}

/** Every statement each pack ships, read from the packs themselves. */
function packStatements() {
  const rows = [];
  for (const pack of packGuides) {
    const file = path.resolve(repoRoot, "packs", pack, "statements.toml");
    if (!fs.existsSync(file)) continue;
    const raw = fs.readFileSync(file, "utf8");
    // One [[statements]] block at a time; `id`, `label` and `grain` are the
    // only keys read, and rows are skipped entirely.
    for (const block of raw.split(/^\[\[statements\]\]$/m).slice(1)) {
      const upto = block.split(/^\[\[/m)[0];
      const id = upto.match(/^id\s*=\s*"([^"]+)"/m);
      const label = upto.match(/^label\s*=\s*"([^"]+)"/m);
      const grain = upto.match(/^grain\s*=\s*"([^"]+)"/m);
      const isDefault = /^default\s*=\s*true/m.test(upto);
      if (!id) continue;
      rows.push({
        pack,
        id: id[1],
        label: label ? label[1] : id[1],
        grain: grain ? grain[1] : "model grid",
        isDefault,
      });
    }
  }
  if (rows.length === 0) throw new Error("no statements found in any pack");
  const out = ["| Pack | Statement | Reported at |", "|---|---|---|"];
  for (const r of rows) {
    const name = r.isDefault ? `${r.label} *(default)*` : r.label;
    out.push(`| \`${r.pack}\` | ${name} | ${r.grain} |`);
  }
  return out;
}

/**
 * Every builtin the engine accepts, read from its dispatch table.
 *
 * The source is `crates/cfdl-calc/src/funcs.rs` — the implementation, not a
 * document about it. A function cannot appear here unless the engine actually
 * has it, and cannot be added to the engine without appearing here.
 *
 * GROUPING IS EDITORIAL and lives below; the LIST is not. Anything the engine
 * gains that this file has not been told about lands under "Other" rather than
 * being silently dropped, so the page is always complete even when it is not
 * yet organised.
 */
const BUILTIN_GROUPS = [
  ["Arithmetic", ["abs", "min", "max", "clamp", "exp", "ln", "pow", "sum", "avg"]],
  ["Rounding", ["round", "round_up", "round_down", "round_to"]],
  ["Dates", [
    "date", "parse_date", "edate", "eomonth", "days_between", "months_between",
    "year_frac", "roll", "is_business_day", "add_business_days",
  ]],
  ["Time value of money", ["pv", "fv", "pmt", "ipmt", "ppmt", "nper", "rate"]],
  ["Domain", ["macrs_rate", "cpr_to_smm", "cpr_to_periodic"]],
  ["Choice", ["if"]],
  ["Curves", ["curve_value"]],
  ["Series folds", ["series_sum", "series_avg"]],
];

function expressionBuiltins() {
  const found = new Set();

  const src = fs.readFileSync(
    path.resolve(repoRoot, "crates", "cfdl-calc", "src", "funcs.rs"),
    "utf8",
  );
  for (const match of src.matchAll(/^\s+"([a-z_0-9]+)"\s*=>/gm)) {
    found.add(match[1]);
  }

  // SPECIAL FORMS are dispatched in the evaluator rather than the function
  // table, because they do not evaluate their arguments the ordinary way —
  // `if` evaluates one branch, `curve_value` reaches the host for a lookup, and
  // the series folds read a whole series rather than a value. Reading only
  // funcs.rs published a list that claimed to be exact and was missing four.
  const evalSrc = fs.readFileSync(
    path.resolve(repoRoot, "crates", "cfdl-calc", "src", "eval.rs"),
    "utf8",
  );
  for (const match of evalSrc.matchAll(/name == "([a-z_0-9]+)"/g)) {
    found.add(match[1]);
  }
  if (found.size === 0) {
    throw new Error("crates/cfdl-calc/src/funcs.rs: no builtins found in the dispatch table.");
  }
  const placed = new Set();
  const out = [];
  for (const [heading, names] of BUILTIN_GROUPS) {
    const present = names.filter((n) => found.has(n));
    if (present.length === 0) continue;
    present.forEach((n) => placed.add(n));
    out.push(`**${heading}** — ${present.map((n) => `\`${n}\``).join(", ")}`);
    out.push("");
  }
  const rest = [...found].filter((n) => !placed.has(n)).sort();
  if (rest.length > 0) {
    out.push(`**Other** — ${rest.map((n) => `\`${n}\``).join(", ")}`);
    out.push("");
  }
  out.push(`*${found.size} functions.*`);
  return out;
}

/** One pack's contracts: terms it reads and streams it emits. */
function contractsFor(pack) {
  const file = path.resolve(repoRoot, "packs", pack, "lowering", "rules.toml");
  if (!fs.existsSync(file)) return [];
  const raw = fs.readFileSync(file, "utf8");
  const byContract = new Map();
  for (const block of raw.split(/^\[\[rules\]\]$/m).slice(1)) {
    const name = block.match(/^contract_name\s*=\s*"([^"]+)"/m);
    if (!name) continue;
    const entry = byContract.get(name[1]) ?? { streams: [], terms: new Set() };
    const stream = block.match(/^stream_name\s*=\s*"([^"]+)"/m);
    if (stream) {
      entry.streams.push(
        stream[1]
          .replace(/\{\{contract\.dot_suffix\}\}/g, "[.suffix]")
          .replace(/\{\{contract\.suffix\}\}/g, "[suffix]")
          .replace(/\{\{contract\.[a-z_0-9]+\}\}/g, "[\u2026]"),
      );
    }
    for (const m of block.matchAll(/\{\{contract\.([a-z_0-9]+)\}\}/g)) {
      if (!/^(term_|suffix|dot_suffix)/.test(m[1])) entry.terms.add(m[1]);
    }
    byContract.set(name[1], entry);
  }
  const out = ["| Contract | Terms it reads | Streams it emits |", "|---|---|---|"];
  for (const [name, e] of byContract) {
    const terms = [...e.terms].sort();
    const streams = [...new Set(e.streams)];
    out.push(
      `| \`${name}\` | ${terms.length ? terms.map((x) => `\`${x}\``).join(", ") : "\u2014"} | ` +
        `${streams.map((x) => `\`${x}\``).join(", ")} |`,
    );
  }
  return out;
}

/** Every contract each pack offers, with the streams it emits and terms it reads. */
function packContracts() {
  const out = [];
  for (const pack of packGuides) {
    const file = path.resolve(repoRoot, "packs", pack, "lowering", "rules.toml");
    if (!fs.existsSync(file)) continue;
    const raw = fs.readFileSync(file, "utf8");
    const byContract = new Map();
    for (const block of raw.split(/^\[\[rules\]\]$/m).slice(1)) {
      const name = block.match(/^contract_name\s*=\s*"([^"]+)"/m);
      if (!name) continue;
      const entry = byContract.get(name[1]) ?? { streams: [], terms: new Set() };
      const stream = block.match(/^stream_name\s*=\s*"([^"]+)"/m);
      if (stream) {
        // A rule's stream name is a template. `{{contract.dot_suffix}}` is how a
        // pack lets one contract type be declared more than once — the suffix a
        // model gives the contract lands here, so two leases produce two
        // distinct streams. Rendering the placeholder raw would put template
        // syntax on a page for readers who never see a lowering rule.
        entry.streams.push(
          stream[1]
            .replace(/\{\{contract\.dot_suffix\}\}/g, "[.suffix]")
            .replace(/\{\{contract\.suffix\}\}/g, "[suffix]")
            .replace(/\{\{contract\.[a-z_0-9]+\}\}/g, "[…]"),
        );
      }
      for (const m of block.matchAll(/\{\{contract\.([a-z_0-9]+)\}\}/g)) {
        // `term_*`, `suffix` and `dot_suffix` are supplied by the contract's
        // own declaration rather than written in its `terms` block.
        if (!/^(term_|suffix|dot_suffix)/.test(m[1])) entry.terms.add(m[1]);
      }
      byContract.set(name[1], entry);
    }
    if (byContract.size === 0) continue;
    out.push(`### \`${pack}\``);
    out.push("");
    out.push("| Contract | Terms it reads | Streams it emits |");
    out.push("|---|---|---|");
    for (const [name, e] of byContract) {
      const terms = [...e.terms].sort();
      const streams = [...new Set(e.streams)];
      out.push(
        `| \`${name}\` | ${terms.length ? terms.map((x) => `\`${x}\``).join(", ") : "—"} | ` +
          `${streams.map((x) => `\`${x}\``).join(", ")} |`,
      );
    }
    out.push("");
  }
  if (out.length === 0) throw new Error("no pack contracts found");
  return out;
}

/** Every metric each pack declares. */
function packMetrics() {
  const out = [];
  for (const pack of packGuides) {
    const file = path.resolve(repoRoot, "packs", pack, "metrics.toml");
    if (!fs.existsSync(file)) continue;
    const raw = fs.readFileSync(file, "utf8");
    const rows = [];
    for (const block of raw.split(/^\[\[metrics\]\]$/m).slice(1)) {
      const id = block.match(/^id\s*=\s*"([^"]+)"/m);
      if (!id) continue;
      const kind = block.match(/^kind\s*=\s*"([^"]+)"/m);
      const formula = block.match(/^formula\s*=\s*"([^"]+)"/m);
      rows.push({
        id: id[1],
        kind: kind ? kind[1] : "",
        formula: formula ? formula[1].replace(/\|/g, "\\|") : "",
      });
    }
    if (rows.length === 0) continue;
    out.push(`### \`${pack}\``);
    out.push("");
    out.push("| Metric | Kind | Definition |");
    out.push("|---|---|---|");
    for (const r of rows) out.push(`| \`${r.id}\` | ${r.kind} | ${r.formula} |`);
    out.push("");
  }
  if (out.length === 0) throw new Error("no pack metrics found");
  return out;
}

const dataRegions = [
  { page: "reference/diagnostics.md", key: "diagnostics-catalogue", body: diagnosticsCatalogue() },
  { page: "reference/statements.md", key: "pack-statements", body: packStatements() },
  { page: "reference/expressions.md", key: "expression-builtins", body: expressionBuiltins() },
  { page: "reference/packs.md", key: "pack-contracts", body: packContracts() },
  { page: "reference/metrics.md", key: "pack-metrics", body: packMetrics() },
  ...packGuides.map((pack) => ({
    page: `packs/${pack}.md`,
    key: `contracts-${pack}`,
    body: contractsFor(pack),
  })),
  {
    page: "benchmarks.md",
    key: "benchmark-cases",
    // Names alone told a reader nothing — `mbs_pool_ramped` is a directory,
    // not a description. Each row now says what the deal is and links to the
    // full write-up: the reference, what the case exercises, how closely it
    // matched, and what any residual means.
    body: [
      "| Case | What it is |",
      "|---|---|",
      ...benchmarkGroups.flatMap((pack) =>
        (benchmarkExampleLinks[pack] ?? []).map(
          ({ title, href, summary }) => `| [${title}](${href}) | ${summary} |`,
        ),
      ),
      "",
      `*${benchCases.length} cases.*`,
    ],
  },
];

// --- Homepage counts -------------------------------------------------------
//
// The landing page stated "8 benchmark cases" and "59 golden fixtures" long
// after both had roughly tripled. Hand-typed numbers on a page nobody
// regenerates go stale silently and understate the thing they exist to state,
// so they are counted from the repository here and read from this file.
const stats = {
  packs: packGuides.length,
  benchmarkCases: benchCases.length,
  // Both halves: a model that must compile and run to a fixed output, and a
  // model that must be REJECTED with a named diagnostic. The second half is
  // half the fixtures and is the part that keeps error messages from rotting.
  goldenFixtures: ["valid", "invalid"].reduce((total, kind) => {
    const dir = path.resolve(repoRoot, "fixtures", kind);
    if (!fs.existsSync(dir)) return total;
    return (
      total +
      fs.readdirSync(dir, { withFileTypes: true }).filter((d) => d.isDirectory()).length
    );
  }, 0),
};
const statsPath = path.resolve(siteDir, "content", "stats.json");
const statsBody = JSON.stringify(stats, null, 2) + "\n";
if (checkMode) {
  if (!fs.existsSync(statsPath) || fs.readFileSync(statsPath, "utf8") !== statsBody) {
    throw new Error("Generated file is stale: site/content/stats.json");
  }
} else {
  fs.writeFileSync(statsPath, statsBody, "utf8");
}

const staleRegions = [];
for (const { page, key, body } of dataRegions) {
  const stale = syncRegion({
    filePath: path.resolve(docsOutputRoot, page),
    key,
    body,
    checkMode,
    repoRoot,
  });
  if (stale) staleRegions.push(stale);
}
if (staleRegions.length > 0) {
  throw new Error(
    staleRegions.join("\n") +
      "\n\nRun `npm run sync:content` to refresh the generated blocks.",
  );
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
