# 21 — Documentation standards audit

Assessment of the published CFDL documentation — cfdl.dev (`site/`) and
learn.cfdl.dev (`learn/`) — against ASD-STE100 Simplified Technical English and
three adjacent standards.

This document is an audit. It changes no prose, adds no gate, and touches no
build. Its output is a recommendation, a companion style guide
(`22_cfdl_controlled_english.md`), a terminology register (`terminology.toml`),
and a set of backlog items.

---

## 1. Summary

The documentation estate is **70,438 words of published prose across 160 files**
with no style guide, no glossary, and no prose linter. The only editorial control
is `tools/check-site-voice.py`, which gates *provenance* — it keeps internal
engineering narrative and marketing ornament off the site — and says nothing
about sentence construction, vocabulary, or terminology.

The consequences are measurable rather than aesthetic:

- The same words are spelled two ways in the same corpus (`amortisation` 21 vs
  `amortization` 3; `behavior` 25 vs `behaviour` 6; `modeling` 28 vs
  `modelling` 5).
- One concept carries three names (`run configuration` 57, `run config` 7,
  `run settings` 2).
- The two properties are written in different registers: `learn` averages 20.5
  words per sentence against `site`'s 9.6, with **44% of learn's sentences over
  STE's 20-word procedural limit**.
- The three normative specifications use MUST/SHOULD/MAY **143 times** and
  **nothing in the repository defines those terms**.

Not everything fails. Paragraph length substantially conforms (8% / 4% / 0% of
paragraphs exceed STE's six-sentence cap), contractions are effectively absent
from published prose (9 instances in 70,438 words), and the deliberate register
split between Specification and Reference layers is a documented design decision
that this audit endorses rather than flags.

**Recommendation: do not adopt ASD-STE100 as written.** Adopt a CFDL-derived
controlled English that takes STE's writing rules as its base, tiered by content
type, with every divergence recorded and reasoned. The reasoning is in §5.

---

## 2. Scope and method

Measured by `tools/audit-measure.py`. Every figure below is printed by that
script; none is estimated. Re-run it to check any number here:

```
python3 tools/audit-measure.py
```

It is not wired into the makefile and is not a gate. These figures describe a
corpus that is supposed to change, so a target that failed when prose moved
would be measuring the wrong thing. Enforcement belongs in
`tools/check-site-voice.py` — see `22_cfdl_controlled_english.md` §5.

"Prose" means what a reader reads as sentences. Removed before any count: YAML
frontmatter, fenced code blocks, inline code spans (replaced by a single token —
a backticked `model.payback_years` is one lexical item to a reader, not three
words), JSX/HTML tags, table rows, ATX headings, and MDX import/export lines. A
sentence is a run terminated by `.`, `!`, or `?` containing more than two words;
the filter drops bare list labels and stripping residue, which are fragments
rather than short sentences.

Table rows are excluded deliberately. Cells are fragments, and leaving them in
halves the measured mean — the figures below would look far better and mean
nothing.

| Corpus | Files | Prose words | Sentences |
|---|---|---|---|
| `site/content/docs` | 111 `.md` | 41,984 | 4,385 |
| `learn/content/chapters` | 27 `.mdx` | 26,136 | 1,274 |
| `training/exercises/*/*/README.md` | 22 `.md` | 2,318 | 117 |
| **Total** | **160** | **70,438** | **5,776** |

Not measured: prose hardcoded in TSX (`site/app/page.tsx`,
`site/components/SiteFooter.tsx`, playground microcopy — roughly 800 words), and
JSON schema `description` strings served at `/schemas`. Both are in scope for the
style guide and should be added to any future gate.

---

## 3. The standard

The request named "ASD-STE1000". No standard of that designation exists. The
intended standard is **ASD-STE100, Simplified Technical English**, issued by the
AeroSpace and Defence Industries Association of Europe. **Issue 9, published
15 January 2025**, is current; 31 of its 53 rules were refined for clarity and
555 dictionary entries revised.

Structure:

- **Part 1, Writing Rules** — 53 rules in 9 sections, covering word choice, noun
  phrases, verbs, sentences, procedures, descriptive writing, safety
  instructions, punctuation, and writing practices.
- **Part 2, Dictionary** — approximately 950 approved words. Each has **one
  approved meaning and one approved part of speech**. A large not-approved list
  gives an approved alternative for each entry.

The substantive limits this audit measures against:

| Constraint | Limit |
|---|---|
| Sentence length, procedural | 20 words |
| Sentence length, descriptive | 25 words |
| Paragraph length, descriptive | 6 sentences |
| Noun cluster | 3 words |
| Passive voice | Descriptive text only, and only where the actor is genuinely unknown or irrelevant |
| `-ing` forms | Only as a technical noun or part of one; not as a verb form or modifier |

A copy is free on request from asd-ste100.org. Section names and rule numbers
below are described thematically rather than cited by number, because rule
numbering was not verified against an official copy; **obtain one before the
style guide quotes a rule number.**

**Tooling reality.** There is no free conforming checker. Commercial options are
Congree, Acrolinx, and HyperSTE (Etteplan); TechScribe publishes a term checker.
Vale can encode a large subset of the writing rules and the not-approved word
list, and is the realistic path here — but it cannot verify dictionary
conformance, because the dictionary is copyrighted and cannot be redistributed
in a repository.

**What STE was designed for.** Aircraft maintenance procedures, read under time
pressure, frequently by non-native English speakers, where a misread instruction
is a safety event. Its constraints follow from that. It was not designed for a
programming language reference or a pedagogical curriculum, and §5 sets out where
that difference matters.

---

## 4. Conformance findings

Verdicts are per corpus. **Fail** means the corpus violates the rule at a rate
that a gate would flag on most pages; **partial** means violations are real but
localized; **conforms** means no material finding.

### 4.1 Sentence length — `site` conforms, `learn` fails

> **RESOLVED.** Both training rows were remediated — the exercise prompts under
> register item 6, the chapters under item 12 after it was reopened (2026-08-25).
> Current figures, from the same script:
>
> | Corpus | Mean | Median | p90 | Max | >20 words | >25 words |
> |---|---|---|---|---|---|---|
> | `learn/content/chapters` | 17.4 | 17 | 29 | 55 | 34% | 19% |
> | `training` exercise prompts | 11.0 | 10 | 20 | 25 | 9% | 0% |
>
> The chapters' residual maximum is a measurement artefact, not prose: the
> sentence splitter does not break on a period inside `**bold.**`, so it reports
> some two-sentence runs as one. The longest genuine chapter sentence is 35
> words. The figures below are the measurement that prompted the work and are
> left as recorded.

| Corpus | Mean | Median | p90 | Max | >20 words | >25 words |
|---|---|---|---|---|---|---|
| `site/content/docs` | 9.6 | 9 | 15 | 52 | 3% | 2% |
| `learn/content/chapters` | 20.5 | 18 | 37 | 67 | **44%** | **31%** |
| `training` exercise prompts | 19.8 | 19 | 34 | 63 | **43%** | 21% |

The site's docs already sit inside both STE limits by a wide margin. The
training material does not, and the exercise prompts — the most procedural
content in the estate, the text a reader follows while typing — are as long as
the chapters.

That last row is the finding that matters most. A 63-word instruction is a
defect regardless of which standard is adopted.

### 4.2 Paragraph length — conforms

| Corpus | Paragraphs | Mean sentences | Max | >6 sentences |
|---|---|---|---|---|
| `site/content/docs` | 1,325 | 3.3 | 44 | 8% |
| `learn/content/chapters` | 457 | 2.8 | 19 | 4% |
| `training` exercise prompts | 52 | 2.2 | 6 | 0% |

No action needed. The long tail (a 44-sentence paragraph) is an artefact of
tight-packed list blocks, not of genuine walls of text.

### 4.3 Voice and verb forms — partial

| Corpus | Passive sentences | `-ing` used as a modifier |
|---|---|---|
| `site/content/docs` | 440 (10%) | 164 (4%) |
| `learn/content/chapters` | 110 (9%) | 79 (6%) |
| `training` exercise prompts | 5 (4%) | 3 (3%) |

Roughly one sentence in ten is passive. STE permits passive in descriptive text
where the actor is unknown or irrelevant, so a large share of these are
defensible; what is not defensible is passive in procedures.

The generated example pages carry passive prose structurally, from two different
places, and the distinction decides where the fix goes:

- The shared template at `site/scripts/sync-content.mjs:805` puts "Every number
  below is checked against an independent reference implementation on every
  commit" on **38 pages**. This is a descriptive passive with an irrelevant
  actor and is defensible under STE. One edit would change all 38, but no edit
  is required.
- The per-case sources carry "The source cannot be published, so its conventions
  are recreated independently of the model and compared period by period" on
  **20 pages** — doubly passive with an unstated agent, and this one is not
  defensible. It lives in `benchmarks/*/*/case.toml` and `benchmarks/*/*/CASE.md`,
  not in the template, so it is roughly 20 edits rather than one.

Contractions are effectively absent: **9 instances in 70,438 words**, all in
`.md`/`.mdx`. (`can't` appears in `site/app/page.tsx`, which is outside the
measured corpus.) This conforms and needs only to be written down so it stays
true.

### 4.4 Vocabulary consistency — fail, and the cheapest thing to fix

> **RESOLVED.** US spelling was applied across published content and its
> generating sources: 431 replacements across 41 forms. The decision and the
> full map are in `terminology.toml`. The figures below are the measurement that
> prompted it and are left as recorded.
>
> One correction the fix produced, worth more than the fix: this section
> measured *published prose* and found 7 conflicting pairs. The sweep had to run
> over the **sources** that generate that prose, and those held 41. A published
> page is a rendering; measuring it understates what is actually inconsistent.
> Re-measure sources, not pages.

STE's central rule is one word, one form, one meaning. Both spellings of the
same word are currently published:

| | | | |
|---|---|---|---|
| `modeling` 28 | vs | `modelling` 5 | |
| `amortisation` 21 | vs | `amortization` 3 | **UK form dominant** |
| `amortising` 19 | vs | `amortizing` 4 | **UK form dominant** |
| `behavior` 25 | vs | `behaviour` 6 | |
| `license` 9 | vs | `licence` 4 | |
| `amortize` 1 | vs | `amortise` 1 | |
| `catalog` 1 | vs | `catalogue` 2 | |

Note the split is not a clean US/UK divide: `behavior` and `modeling` lean
American while `amortisation` and `amortising` lean British. There is no
convention in force, only accretion. A single decision plus a find-and-replace
closes this, and `terminology.toml` records the decision so it stays closed.

Competing names for one concept:

| Concept | Forms in use |
|---|---|
| The run settings object | `run configuration` 57, `run config` 7, `run settings` 2 |
| The output artefact | `results document` 9, `output document` 2 |
| Activating a UI control | `hit` 7, `click` 1, `press` 1 |

The third is a straight STE violation of a different kind: `hit` is not an
approved instruction verb, and `getting-started.md:31` reads
"Hit **Run**. The compiler and engine execute entirely in your browser." This is
the single most-read procedural line on the site.

### 4.5 Noun clusters — partial, and mostly a registration problem

167 candidate clusters of four or more content words. The most frequent:

```
  6  seven published weighted average lives
  4  period signed cash flows
  2  recoveries above expense stops
  2  payment amortising loan pool
  2  term power purchase agreement
  2  widely used academic valuation spreadsheet
```

Most are legitimate domain terms. `term power purchase agreement` is what the
instrument is called; rewriting it to satisfy a three-word cap would make the
documentation worse and wrong. STE's own answer is the mechanism to use: register
them as approved Technical Names, and the cap stops applying. That is what
`terminology.toml` is for.

The residue after registration — `widely used academic valuation spreadsheet`,
`unnoticed line looks entirely plausible` — is genuine cluster sprawl and should
be rewritten.

### 4.6 Procedural form — fail in `learn`, partial in `site`

STE requires an instruction to be an imperative, one action per sentence. The
training chapters consistently write procedures as *descriptions of a discipline*
instead. From `03-reading-results.mdx`:

> The discipline for challenging any figure in the output, in the order that
> finds the problem fastest:
>
> 1. **Which streams feed it?** Names answer this — a total over `practice.*` is
>    legible because the taxonomy was designed in the model.

The numbered steps are questions and claims, not instructions. A reader
following along has to convert each one into an action themselves. The same
shape recurs in `15-diagnostics-as-a-discipline.mdx` ("The five-minute method")
and `13-multi-file-models-and-style.mdx`.

This is the clearest place where STE would improve the training material rather
than damage it, and it is addressed by the Tier A/C boundary in the style guide.

### 4.7 The deliberate register split — conforms, and must be protected

Two register decisions are already documented and should survive any adoption:

- `site/components/docs/SpecificationBanner.tsx:7` states the specification
  pages "are precise and unwelcoming by design — they exist so a second
  implementation could be written from them."
- `docs/19_training_guide_plan.md` sets the per-chapter shape and the dual-track
  audience for the curriculum.

A single controlled language applied flat across the estate would erase both.
Any adopted standard must be tiered.

---

## 5. Fitness: where STE helps and where it does harm

**Where STE earns its keep.** The install pages, `troubleshooting.md`,
`getting-started.md`, the exercise prompts, and the diagnostics reference are
task content read under mild pressure by someone trying to get something to
work. Short imperative sentences, one action each, a fixed verb for each action,
and no ambiguous pronouns are straightforwardly better there. The `Hit **Run**`
finding and the 43%-over-limit exercise prompts both live in this band.

**Where a literal application does harm.** The curriculum's method is to build a
concept in the reader's head before naming it, and it does that with metaphor,
italic stress, and compound sentences that hold two ideas in tension. Chapter 1
opens:

> Every financial model is a claim: *if these assumptions hold, this is the
> cash*.

STE bans that construction — the colon-apposition, the italic semantic stress,
and `claim` used in a sense the dictionary does not approve. It is also the
thesis sentence of the entire course. Enforcing STE here would not make the
sentence clearer; it would delete the idea.

The same applies to the specification layer for the opposite reason. Those pages
are terse and normative because a second implementer must be able to work from
them. STE's dictionary constraint would force paraphrase of terms that are
precise, and paraphrase in a normative document is a defect.

**Conclusion.** STE's *writing rules* are broadly right for this estate. STE's
*dictionary*, applied whole, is wrong for it — the approved word list excludes
nearly the entire finance and compiler vocabulary this documentation exists to
convey. The mechanism STE provides for exactly this situation (Technical Names
and Technical Verbs) is the right lever, but using it at this vocabulary's scale
means the result is a CFDL-specific controlled language that is *derived from*
STE, not a claim of conformance to it.

Claiming STE conformance without the dictionary would be false. The style guide
therefore names the thing CFDL-CE and states its provenance honestly.

---

## 6. Adjacent standards

### 6.1 RFC 2119 / BCP 14 — adopt, highest value per unit effort

The three normative specifications use RFC 2119 keywords heavily and define none
of them:

| File | MUST / SHALL / SHOULD / MAY / REQUIRED / OPTIONAL / RECOMMENDED |
|---|---|
| `docs/01_language_spec.md` | 67 |
| `docs/04_compiler_spec.md` | 45 |
| `docs/07_pack_interface.md` | 31 |
| **Total** | **143** |

**Files citing RFC 2119 or BCP 14 anywhere in the repository: 0.**

These documents exist so a second implementation can be written from them. A
second implementer currently has to guess whether "should" is a requirement or
advice. The fix is one short section per specification. It is the cheapest
correctness improvement available in the entire estate.

### 6.2 ISO/IEC/IEEE 26514:2022 and IEC/IEEE 82079-1:2019 — adopt as the frame

These are the standards actually written for this class of artefact — 26514 for
the design and development of information for software users, 82079-1 for
information for use generally. Where STE governs sentences, these govern whether
the information product is complete, findable, and fit for its audience.

Findings against their requirements:

- **No glossary exists** anywhere in `site/`, `learn/`, or `docs/`. **RESOLVED** — `/docs/glossary` is generated from the register. Both
  standards require defined terms for a product with specialist vocabulary, and
  this product has two overlapping specialist vocabularies (finance and compiler
  construction). The curriculum introduces `grain`, `latch`, `pot`, `reversion`,
  `takeout`, `promote`, `catch-up`, and `lowering` with inline bold definitions
  that a reader cannot navigate back to. `terminology.toml` is the remedy and
  should generate a published glossary page.
- **No `description` frontmatter on any of the 111 site doc pages.** **RESOLVED** — all pages carry one, and a gate keeps it that way. Every page
  has `id` and `title`; none has a description, so no page has a meta
  description for search results or link previews. `learn` does this correctly —
  all 27 chapters carry one — so the fix is to copy a convention that already
  exists in the repository.
- **No machine-readable document-type field.** `layer` exists but is set on only
  8 pages (the specification layer). There is no way to tell a tutorial from a
  reference page programmatically, which is why the style guide's tiers are
  expressed as path globs rather than as a frontmatter query. Adding an
  `audience` or `doctype` field would make the tiering directly checkable.
- **Navigation coverage is adequate.** 71 slugs appear in `site/content/nav.ts`;
  40 example pages are absent from the sidebar but are linked from
  `site/content/docs/examples/index.md` (48 links). Reachable, so not a finding —
  recorded here because it looks like one until checked.

### 6.3 WCAG 2.2 AA / EN 301 549 — assess, not yet assessed

Both sites are Next.js applications with a custom design system, a theme toggle,
an interactive playground, and syntax-highlighted code blocks. Each of those is
a common source of contrast, focus-order, and keyboard-trap failures.

**This audit did not test accessibility conformance and makes no claim about
it.** Doing so needs an axe or Lighthouse pass against a running build, plus
manual keyboard and screen-reader checks that automation cannot cover.

The reason to schedule it: EN 301 549 is the European harmonised standard, and
the European Accessibility Act's obligations have applied since 28 June 2025. If
CFDL is sold into the EU, WCAG 2.2 AA is a legal baseline rather than a quality
goal. Contrast tokens and focus states are the likely first failures given the
theme toggle, and `learn/scripts/check-tokens.sh` already exists as a place a
contrast check could live.

---

## 7. What constrains remediation

`site/scripts/sync-content.mjs` generates most of `content/docs`. Editing a
generated page in place is overwritten by `npm run prebuild`, so the true cost of
a prose fix depends entirely on where the bytes come from:

| Ownership | Pages | Prose words | Where the edit goes |
|---|---|---|---|
| `generated: none` | 32 | 8,157 | **The page itself — directly editable** |
| `source:` (benchmarks) | 38 | 15,209 | `benchmarks/`, or the shared template in `sync-content.mjs` |
| `generated: full` | 14 | 14,003 | `docs/*.md`, `distribution/install-configure.md` |
| script manifest (no marker) | 17 | 2,014 | `examples/*/README.md` |
| `generated: regions` | 10 | 2,601 | Outside `<!-- cfdl:generated -->` fences |
| **Total** | **111** | **41,984** | |

Two consequences worth pricing correctly:

- **Only 19% of the site's prose is directly editable.** Any estimate that treats
  111 pages as 111 editing tasks is wrong in both directions.
- **The 38 benchmark pages share one template**, at `sync-content.mjs:805`. Any
  change to the sentence it emits is one edit that corrects 38 pages, which makes
  it the highest-leverage prose surface in the estate. The sentence it currently
  emits does not need changing (§4.3) — the leverage is worth knowing about
  before it does.
- **Per-case prose is not shared.** The doubly-passive redistribution note (20
  pages), the U+00D7 multiplication sign (4 files), and `$33.6mm`-style currency
  (37 files) all live in `benchmarks/*/*/case.toml` and `CASE.md`. These are
  per-file edits and should not be estimated as template work.

The 17 manifest pages are not unowned; `sync-content.mjs` fails the build on a
genuinely unowned page. They are generated from `examples/*/README.md` and simply
do not label themselves.

---

## 8. Recommendation

1. **Adopt CFDL-CE** (`22_cfdl_controlled_english.md`) — STE's writing rules as
   the base, tiered by content type, with each divergence stated and reasoned.
   Do not claim ASD-STE100 conformance; the dictionary is not adopted and saying
   otherwise would be false.
2. **Adopt the terminology register** (`terminology.toml`) as the single source
   for approved forms, Technical Names, and Technical Verbs. It resolves every
   §4.4 conflict, exempts legitimate finance terms from the noun-cluster rule,
   and can generate the glossary that §6.2 shows is missing.
3. **Adopt RFC 2119** in the three normative specifications.
4. **Adopt 26514 / 82079-1** as the frame for structure and findability —
   concretely: add a glossary, add `description` frontmatter, add a doctype
   field.
5. **Schedule a WCAG 2.2 AA assessment.** Do not claim conformance until it runs.
6. **Obtain the official ASD-STE100 Issue 9 copy** before the style guide cites
   any rule by number.

Enforcement, when it comes, should extend `tools/check-site-voice.py` rather than
introduce a parallel prose linter. It already discovers every site-facing source
in its `sources()` function, already has an escape-hatch convention, and is
already wired into `make ci`. A second tool with its own file list would drift
from that one — which is the failure mode the makefile comments describe.

---

## 9. Remediation register

Ordered by value per unit of effort. Backlog items are appended to
`docs/13_feature_backlog.md`.

| # | Action | Effort | Why it ranks here |
|---|---|---|---|
| 1 | ~~Define RFC 2119 keywords in the three specs~~ **DONE** | Hours | 143 undefined keywords in documents meant to support a second implementation |
| 2 | ~~Resolve spelling conflicts per `terminology.toml`~~ **DONE** | Hours | Mechanical; ends a visible inconsistency permanently |
| 3 | ~~Fix `Hit **Run**` and settle one verb for control activation~~ **DONE** | Minutes | The most-read procedural line on the site |
| 4 | ~~Fix number and currency formats~~ **DONE** | Hours | 6 valuation multiples (`8.0×`→`8.0x`) and 164 currency figures (`$33.6mm`→`$33.6m`) across 63 files. The multiplication sign was **kept** wherever it is doing arithmetic (`6,000 × 12 = 72,000`) or naming a grid (`3×3`); only valuation multiples changed |
| 5 | ~~Settle `run configuration` as the single term~~ **DONE in prose** | Hours | Three names for one object. The slug `/docs/guides/scenarios-and-run-configs` still reads `run-configs`; changing it is a URL change needing a redirect, so it was left alone |
| 6 | ~~Rewrite exercise prompts to Tier A~~ **DONE** | Days | All 22 rewritten: numbered imperative steps, predictions as imperatives. Mean sentence length 19.8→11.4 words; over-20-word share 43%→11% (the residue is descriptive, where 25 is the limit); max 63→35. Every anchor number verified unchanged against the diff |
| 7 | ~~Add `description` frontmatter to 111 site pages~~ **DONE** | Days | All 112 pages now carry one, enforced by `site/scripts/check-descriptions.mjs`. Generated pages derive it from what already exists — a benchmark case's own `summary`, an example README's first sentence — so there is no second wording to keep true |
| 8 | ~~Generate a glossary page from `terminology.toml`~~ **DONE** | Days | `/docs/glossary`, 47 terms, generated by `tools/gen-glossary.py` and freshness-checked by `make glossary-check` |
| 9 | ~~Convert `learn` procedural sections to imperative steps~~ **DONE** | Days | Four sections converted (ch. 3 descent, ch. 6 routing, ch. 12 distribution checks, ch. 13 style rules), five cross-references updated with them. Ch. 15 and ch. 19 were inspected and left alone: their steps were already condition-first imperatives, and rewriting conformant prose to satisfy an audit line would be churn |
| 10 | ~~WCAG 2.2 AA assessment and remediation~~ **ASSESSED AND REMEDIATED** (`docs/23`) | Unknown until assessed | Five findings, all fixed and re-verified: the muted token failed contrast in both themes, the splitter lacked its ARIA value trio, scroll regions were keyboard-unreachable. Zero axe violations after. The human AT pass is backlog 7.35; conformance is not claimed until it runs |
| 11 | ~~Extend `check-site-voice.py` with CFDL-CE rules~~ **DONE** | Days | The mechanical subset: retired spellings and synonyms load from `terminology.toml` at run time, plus number formats, `hit`, and contractions. The specifications are now checked too (CE rules only — the narrative exemption stands). Negative-tested: fires on each rule, exempts code spans, arithmetic `×`, and `ste-allow:` lines |
| 12 | ~~Reduce `learn` sentence length toward the 25-word descriptive limit~~ **DONE** | Weeks | Reopened 2026-08-25 after being closed without action on 2026-08-14, and done targeted rather than mechanically: only the 154 sentences over 35 words were rewritten, across 25 of the 27 chapters. Mean 20.7→17.4; over-25 share 31%→19%. No metaphor, italic thesis sentence, technical claim, or anchor number changed |

Item 12 was declined once and then done. The reasoning is worth keeping, because
both decisions were defensible on the same evidence.

The 2026-08-14 decision closed it without action: the chapters' long sentences
hold two ideas in tension on purpose, and shortening them mechanically trades
pedagogy for conformance. That objection is correct — for the 25-to-35-word
band, where a sentence is genuinely one thought with a qualifier attached.

The 2026-08-25 reopening accepted that objection and scoped around it. Above 35
words the sentences were not paired ideas but three-clause pileups: a claim, a
parenthetical, and an em-dash aside, stacked. The tail was rewritten and the
band was left alone, so the 25-to-35 sentences the first decision protected are
still there. Where a long sentence turned out to be an enumeration wearing prose
clothes, it became a vertical list under S5 — day-count bases, the five
waterfall step shapes, a diagnostic's parts, the four shipped packs.

Tier C's S1–S2 remain a target rather than a gate threshold either way. With
this, every item on the register is closed: eleven done, one assessed with its
human follow-up filed as backlog 7.35.
