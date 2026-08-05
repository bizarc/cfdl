# Site information architecture — design

Status: **proposal.**

The current site is a port. `site/content/nav.ts` says so in its own header
comment — *"Ported from the previous sidebar so the information architecture
carries over unchanged"* — and the shape it carried over is a repository's
`docs/` tree, which is organised by **what file exists** rather than by **what
someone came to do**.

That is not a criticism of the pages. Most of them are good. It is a statement
about the container: 84 pages arranged for a reader who already knows the
project, presented to readers who do not.

---

## 1. What is working, and stays

Named explicitly so the rewrite does not churn them:

- **The homepage, design system, and logo.** Not in scope here.
- **The playground, and reaching it fast.** The strongest single asset. A
  visitor can run a model without installing anything.
- **The eight-section spine** — Introduction, Install & Setup, Learn the
  Language, Surfaces, Guides, Domain Packs, Reference, Project. The *shape* is
  sound. What is wrong is what sits inside the sections, not the sections.

This proposal therefore **re-cuts contents and adds two sections**. It does not
throw away the navigation the site already has.

---

## 2. The three problems

### 2.1 Sections mix audiences

Four distinct readers are interleaved throughout, and none of them has a path:

| Reader | Comes to | Currently must visit |
|---|---|---|
| **Evaluator** | decide whether CFDL is real | Project (bottom of the nav) |
| **Modeller** | write a `.cfdl` model | Learn the Language, Guides, Domain Packs, Examples |
| **Pack author** | extend CFDL to a new domain | one Reference page |
| **Integrator** | embed the engine | Install, Surfaces, Reference |

The sharpest case is **Project**, which holds five pages for five different
readers: Benchmarks (evaluator), Implementation status (internal),
Troubleshooting (modeller), FAQ (mixed), Licensing (buyer). It is a bin, not a
section.

The second sharpest is **pack authoring**, which is a whole discipline —
category vocabularies, lowering rules, templates, contracts and terms, metrics,
subtotals, statements, validations — served by a single page filed under
Reference.

### 2.2 Internal rationale is published as product documentation

A repository records **why**: what was tried, what was wrong, what is missing.
That record is valuable and should keep being written. It is not product
documentation, and shipping it to a product site tells a prospective user that
the tool is a work in progress.

Present on the site today, or one link from it:

- `implementation-status` — an enumeration of what is not built.
- Benchmark `NOTES.md` reconciliation narratives — the debugging story behind a
  figure, where the reader wants the figure.
- Correction notes in reference pages — *"this item originally gave two reasons,
  and the first was wrong."*

The repo keeps all of it. The site publishes the **conclusions**.

The exception, and it is deliberate: **validation evidence is not rationale.**
"We reproduce HUD's four published DSCRs exactly" is a claim about the product,
belongs on the site, and is the single most persuasive thing there is to say.

### 2.3 Duplicate and orphaned spines

Mechanical, and each is a decision someone deferred:

- `api-server.md` **and** `install/api-server.md`
- `reference.md` **and** `reference/`
- `language-guide.md` **and** `language-reference/`
- `cookbooks/` **and** `packs/` — the same four packs, twice
- 40 pages under `examples/`, of which 13 appear in the nav

---

## 3. The proposed architecture

Ten sections. Each has **one** primary reader, named here and not shipped as a
label on the page.

### 1. Introduction — *evaluator, newcomer*

```
Overview                     what CFDL is, in one screen
How CFDL works               concepts: streams, packs, folds, the ledger
Quickstart                   a number on screen in five minutes
Validation                   ← MOVED UP from Project
```

**Validation moves to the front door.** It is currently the last section of the
site. It is the answer to the first question a professional asks, and no
competitor can copy the page: 21 benchmark cases reconciling to published
figures from HUD's Sample workbook (public domain, committed in-repo), MIT's
Real Estate Finance model, Damodaran, and GNMA pool conventions. Published
source beside CFDL's output, with a *run it* button.

### 2. Install & Setup — *integrator, modeller*

Unchanged, minus the duplicate. Choose a surface → CLI, Python, API server,
VS Code & LSP, Playground.

### 3. Learn the Language — *modeller*

A genuine ordered tutorial and nothing else:

```
Language guide
Minimal model
Your first stream
A simple contract
Using an industry pack
Multi-file model
```

The eight **benchmark models** currently listed here move out. They are
reference deals, not lessons; a reader on lesson three does not want a
leveraged buyout.

### 4. Model with a Pack — *modeller* (was Domain Packs)

```
Choosing a pack
CRE  ·  Credit  ·  Energy  ·  OpCo
```

`cookbooks/` and `packs/` merge. One page per pack: what it models, the
contracts it offers, its category vocabulary, and a worked deal.

### 5. Guides — *modeller*

The existing eight, plus the one the engine now needs:

```
Schedules & calendars
Contracts & packs
Multi-file models
Scenarios & run configs
Stochastic modeling
Curves
Metrics
Statements & reporting        ← NEW
Reading results & IR
Troubleshooting               ← MOVED from Project
```

**Statements & reporting is a genuine gap.** Subtotals, statements, categories,
reporting grain and the annual rollup all shipped, and no guide explains them.
It should cover: what a category is, how a subtotal folds one, why an annual
coverage ratio is recomputed rather than averaged, and how to read the
reconciliation.

### 6. Examples — *modeller* (new top level)

40 pages need a **browsable, filterable index**, not a sidebar list. Filter by
pack, by feature (amortisation, rollover, MACRS, waterfalls), and by whether the
case reconciles to a published source. Notebooks belong here too — they are
worked examples, not a surface.

### 7. Authoring a Pack — *pack author* (new section)

The section that does not exist. A pack is how CFDL reaches a domain it does not
yet cover, and the entire discipline is currently one reference page.

```
Pack anatomy               pack.toml, entrypoints, layout
Category vocabulary        the operating/investing/financing roots and why they are closed
Contracts & terms          declaring terms, templates, `{{contract.*}}` expansion
Lowering rules             how a contract becomes streams
Metrics                    lifetime scalars
Subtotals & statements     per-period folds, rows, display signs, completeness
Validations                pack-declared checks and diagnostic codes
Testing a pack             fixtures, goldens, benchmark cases
Publishing a pack          versioning and compatibility
```

### 8. Surfaces — *integrator*

CLI, Python SDK, API server, VS Code & LSP, Playground, WebAssembly. Notebooks
move to Examples.

### 9. Reference — *all readers, no prose*

Terse, complete, **generated wherever a generator can produce it**:

```
Language spec  ·  Grammar  ·  Expression environment
Diagnostics    ·  Compiler spec
IR schema      ·  Results schema  ·  Pack interface schema
CLI flags      ·  Run config
```

Two rules for this section. **No narrative** — a reference page answers "what is
the exact form of X", nothing else. **Generated where possible** — the results
schema page already regenerates from `docs/schemas/results.schema.json`, and the
same should hold for the pack reference (from the pack TOMLs) and diagnostics
(from the code catalogue), so drift becomes impossible rather than discouraged.

### 10. About — *buyer* (was Project)

```
Licensing
Release notes
```

`implementation-status` is retired from the site. It is an internal artifact and
stays in the repo.

---

## 4. What is cut from the site

Kept in the repository, not published:

- `implementation-status` — the gap enumeration
- `13_feature_backlog.md`
- Design proposals and rejected designs, including `15_streams_and_the_grid.md`
- Correction notes recording what an earlier version got wrong
- Benchmark `NOTES.md` reconciliation narratives — the site publishes the
  reconciled figure and its source, not the investigation

**Not cut:** the benchmark results, the sources they reconcile to, and the fact
that CI re-verifies them on every commit.

### The word "fold"

`fold` is engine vocabulary — a functional-programming term for the operation,
used precisely in the Rust and in design notes. It should not appear on the
site, and it needs no synonym invented for it, because the domain already has
the words: a stream carries a **category**, a **subtotal** aggregates categories
per period, a **ratio** divides two subtotals, and a **statement** presents
them. Those need no translation for a financial reader.

Removed from the generated schema pages, which were its main route onto the
site.

### Still tied to the repository

The GitHub link is gone from the header and footer, but **thirteen content pages
still reference the repository**, and the install pages are not cosmetic: the
CLI, Python SDK and API server pages each begin `git clone`. That is the only
documented way to install CFDL today. Cutting it needs a distribution channel
first — published crates, a PyPI wheel, or signed release binaries — and until
then removing the links would leave the product with no install path.

---

## 5. Two structural moves

Both are things a repo-shaped site cannot do, and both use assets that already
exist.

**The playground becomes the substrate, not a destination.** The engine already
runs in the browser; today it is fenced into `/playground`. Every example in the
docs should run in place. `check-doc-examples.py` already proves the pack-guide
examples "do what their prose claims" — extending that gate to every snippet
gives the documentation a guarantee a ported site cannot make: nothing here is
stale, because CI runs it.

**The statement view becomes a documentation surface.** It is the artifact a
practitioner recognises. It now exists (Stage 10) and lives only in a playground
tab; a pro forma with drill-down from a row to its contributing streams belongs
beside the prose.

---

## 6. Sequencing

1. **De-duplicate** — the four duplicate spines in §2.3. Mechanical, no writing.
2. ~~**Re-cut the nav**~~ — **DONE.** `nav.ts` only, no page moves: Validation
   is now the fourth entry of Introduction rather than the last entry of the
   last section; "Project" is dissolved into "About" (FAQ, Licensing) with
   Troubleshooting moved to Guides and `implementation-status` off the site
   entirely. The header comment records why.
3. **Write Authoring a Pack** — the largest writing task, and the one that
   unblocks external pack authors.
4. **Write Statements & reporting** — the shipped-but-undocumented capability.
5. **Build the Examples index** — filterable, replacing 40 sidebar entries.
6. **Extend the doc-examples gate** to every snippet.
7. **Generate the pack and diagnostics reference.**

Steps 1 and 2 are a day and remove most of the audience mixing. Steps 3 and 4
are the content gaps. Steps 5–7 are the differentiation.
