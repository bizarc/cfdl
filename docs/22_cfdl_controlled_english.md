# 22 — CFDL Controlled English (CFDL-CE)

The writing standard for everything published on cfdl.dev and learn.cfdl.dev.

CFDL-CE is **derived from ASD-STE100 Simplified Technical English, Issue 9**. It
adopts STE's writing rules, tiered by content type. It does **not** adopt STE's
approved-word dictionary, and therefore **does not claim ASD-STE100
conformance** — see §6. The evidence behind each rule is in
`21_documentation_standards_audit.md`; the approved forms are in
`terminology.toml`.

Status: adopted as the standard. Not yet enforced by any gate.

---

## 1. Why a derived standard and not STE itself

STE was written for aircraft maintenance procedures read under time pressure,
often by non-native speakers, where a misread instruction is a safety event. Its
rules for instructions are excellent and transfer directly. Its dictionary —
about 950 words, each with one approved meaning and one approved part of speech —
excludes nearly the whole vocabulary this documentation exists to convey.
`amortization`, `waterfall`, `covenant`, `lowering`, and `span` are not approved
words, and paraphrasing them would make normative documents less precise, not
more.

STE anticipates this and provides Technical Names and Technical Verbs as the
escape. At this vocabulary's scale, using that escape produces a
CFDL-specific controlled language rather than STE. CFDL-CE says so plainly
instead of overclaiming.

---

## 2. Tiers

Every published file belongs to exactly one tier. Tiers are path globs so they
stay mechanically checkable; there is currently no frontmatter field that
distinguishes a tutorial from a reference page.

| Tier | Content | Paths |
|---|---|---|
| **A** | Procedural — the reader is doing something now | `site/content/docs/install/**`, `site/content/docs/getting-started.md`, `site/content/docs/troubleshooting.md`, `training/exercises/*/*/README.md`, `examples/language_tutorial/*/README.md` |
| **B** | Reference and normative | `site/content/docs/reference/**`, `site/content/docs/specification/**`, `docs/0*.md`, `docs/1[2457]_*.md`, JSON schema `description` strings |
| **C** | Conceptual and instructional | `site/content/docs/{concepts,object-model,language-guide,stochastic-modeling,faq,benchmarks}.md`, `site/content/docs/guides/**`, `site/content/docs/packs/**`, `learn/content/chapters/*.mdx`, `benchmarks/*/*/CASE.md` |
| **D** | Marketing | `site/app/page.tsx`, `site/components/SiteFooter.tsx`, playground microcopy |

A page edited at its source inherits the tier of the page it generates. The
specification pages under `site/content/docs/specification/` are byte-copies of
`docs/0*.md`; both are Tier B, and the edit goes to `docs/`.

**Tier A is the tier that matters.** It is where STE was designed to operate,
where the audit found the worst violations, and where conformance is
non-negotiable. Tiers C and D exist mostly to record what is deliberately *not*
constrained, so that a future gate does not fire on prose that is correct.

---

## 3. Rules

`•` applies · `—` does not apply. Where a tier relaxes a rule, the reason is
stated; an unexplained divergence is a defect in this document.

### 3.1 Sentences

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| S1 | Maximum 20 words in an instruction | • | • | • | — |
| S2 | Maximum 25 words in descriptive text | • | • | • | — |
| S3 | Maximum 6 sentences in a paragraph | • | • | • | • |
| S4 | One instruction per sentence | • | • | • | — |
| S5 | Use a vertical list for more than one action | • | • | • | — |

S1–S2, Tier C: this is a target, not a gate threshold. Long sentences in the
curriculum are often long in order to hold two ideas in tension, and splitting
one of those costs comprehension rather than buying it. Write shorter where
shorter is clearer. Do not split a sentence whose halves mean less apart.

The 2026-08-25 chapter pass (register item 12) settled where that defence stops.
It holds for a sentence in the 25-to-35-word band, which is usually one thought
with a qualifier attached. It does not hold past roughly 35 words, where the
sentence is a pileup of three clauses rather than a pair of ideas — a claim, a
parenthetical, and an em-dash aside, stacked. Treat 35 words as the point where
the burden shifts: below it, keep the sentence unless shorter is clearer; above
it, split it unless you can say what the halves lose.

One rewrite to reach for first. A long sentence that lists things — bases,
shapes, parts, options, separated by commas or semicolons — is an enumeration
wearing prose clothes, and S5's vertical list is both shorter and more
scannable. That accounted for six of the pass's rewrites.

### 3.2 Voice and verbs

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| V1 | Use the active voice | • | • | • | • |
| V2 | Passive only where the actor is genuinely unknown or irrelevant | — | • | • | • |
| V3 | Instructions are imperative | • | • | • | — |
| V4 | No `-ing` form as a modifier or verb form | • | • | • | — |
| V5 | Use simple present or simple past; one tense per passage | • | • | • | • |
| V6 | No contractions | • | • | • | • |

V2, Tier A: passive is not permitted in an instruction at all. An instruction
without an actor is an instruction the reader cannot follow.

V4: `-ing` remains correct as a technical noun (`lowering`, `underwriting`,
`netting`) — those are registered in `terminology.toml`. The banned form is the
participial modifier: write "Use the CLI to compile the model", not "Using the
CLI, compile the model".

V6: the audit found 9 contractions in 70,438 words. This rule records existing
practice rather than demanding a change. The measured corpora now hold none, and
the two `can't`s the audit named in TSX are both fixed — `site/app/page.tsx`, and
`learn/app/page.tsx`, which the audit never measured (2026-08-25).

TSX microcopy remains the estate's blind spot. `tools/check-site-voice.py` reads
Markdown and MDX sources, so prose hardcoded in a component is checked by nobody;
that is how a second `can't` survived the audit that reported the first. Anything
user-facing written in a `.tsx` file is Tier D at minimum and still bound by W1–W3
and V6.

### 3.3 Words

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| W1 | One word, one form — use the spelling in `terminology.toml` | • | • | • | • |
| W2 | One concept, one term — no synonyms for a defined thing | • | • | • | • |
| W3 | Use the approved verb for an action (`click`, not `hit` or `press`) | • | • | • | • |
| W4 | Multi-word domain terms must be registered as Technical Names | • | • | • | • |
| W5 | Noun clusters: maximum 3 words unless registered under W4 | • | • | • | — |
| W6 | No marketing ornament | • | • | • | • |
| W7 | Define a term on first use, or link to the glossary | — | • | • | — |

W1–W3 are the rules the audit found broken most consistently and are the
cheapest to enforce, because they reduce to a word list.

W6 is already enforced by `tools/check-site-voice.py`; `terminology.toml` seeds
its `[[not_approved]]` section from that same list so the two cannot drift.

### 3.4 Clarity

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| C1 | No ambiguous `it`, `this`, `that` — repeat the noun | • | • | • | — |
| C2 | Use articles; no telegraphic style | • | • | • | — |
| C3 | Do not carry meaning in italics or bold alone | • | • | — | — |
| C4 | No metaphor as the primary explanation | • | • | — | — |
| C5 | Keep related words together; no long subordinate chains | • | • | • | — |

**C3 and C4 are the deliberate Tier C relaxations, and they are the most
important entries in this document.**

The curriculum's method is to build a concept before naming it, and it does that
with metaphor and semantic stress. Chapter 1 opens "Every financial model is a
claim: *if these assumptions hold, this is the cash*" — the italics carry the
meaning and STE bans the construction. It is also the thesis sentence of the
course. Chapter 2 calls time "the spine", entities "the cast", and streams "the
atoms".

These are permitted in Tier C on two conditions:

1. The metaphor is registered in `terminology.toml` under `[[pedagogical]]`,
   with the plain term it stands for.
2. The plain term appears too. A metaphor introduces a concept; it does not
   replace its name.

Registration is the point. It converts a stylistic habit into a recorded
decision that a reviewer can audit and a future writer can find.

### 3.5 Procedures

| # | Rule | A | B | C | D |
|---|---|---|---|---|---|
| P1 | A step is an imperative naming one action | • | • | • | — |
| P2 | A step is not a question or a claim | • | • | • | — |
| P3 | State the condition before the action | • | • | • | — |
| P4 | State the expected result after an action with a visible outcome | • | — | • | — |

P1–P2, Tier C: this is the audit's §4.6 finding. The training chapters write
procedures as descriptions of a discipline — "The discipline for challenging any
figure in the output…", followed by numbered steps that are questions
("**Which streams feed it?**"). A reader following along must convert each into
an action.

The rule for Tier C: **the prose around a procedure may explain; the numbered
steps must instruct.** Keep the framing sentence. Make step 1 "Identify the
streams that feed the figure", and let the explanation follow it.

---

## 4. Conventions STE does not cover

STE has no rules for a software documentation site. These are CFDL's own.

**Code in prose.** Set identifiers, file names, commands, and diagnostic codes
in backticks. A backticked identifier is one lexical item — it does not count
toward a noun cluster or a sentence-length limit. Do not inflect an identifier
to fit a sentence: write "the `stream` construct", not "the streams construct".

**Diagnostic codes.** Cite as `` `E0123` `` and link to the diagnostics
reference on first use in a page.

**Normative keywords.** Tier B only. Use MUST, MUST NOT, SHOULD, SHOULD NOT, MAY
with their RFC 2119 / BCP 14 meanings, and cite BCP 14 in the document that uses
them. The audit found 143 such keywords across three specifications and no
citation anywhere. Do not use these words in upper case in Tier A, C, or D,
where they carry no normative force and only look like they do.

**Headings.** Sentence case, matching the page's frontmatter `title`. This is
already the convention, recorded in the header comment of
`site/content/nav.ts`.

**Numbers, currency, units.**

- Write multiplication as `8.0x`, not `8.0×` (U+00D7). Four files under
  `benchmarks/*/*/` currently use the multiplication sign, and they are
  published.
- Write `$33.6m`, not `$33.6mm`. `mm` reads as millimetres and is not a standard
  abbreviation for millions outside a trading desk. 37 benchmark source files
  currently use it.
- Use a plain hyphen (U+002D). `docs/01_language_spec.md` contains one
  non-breaking hyphen (U+2011) inherited from an earlier editor. One is enough to
  make the rule worth stating: it is invisible in review and it breaks search.
- Dates in ISO 8601 (`2026-08-14`).

**Frontmatter.** Every published page carries `title` and `description`. The
`learn` chapters already do this; no `site` doc page does. The description is one
sentence and is what a search result shows.

---

## 5. Conformance and the escape hatch

Three levels, so a page can be honest about where it stands:

- **Conforming** — meets every rule for its tier.
- **Conforming with exceptions** — carries `ste-allow:` annotations, each with a
  reason.
- **Not assessed** — the default for anything not yet reviewed.

The reserved annotation form mirrors the existing `site-allow:` convention in
`tools/check-site-voice.py`:

```
ste-allow: <rule id> <reason>
```

For example: `ste-allow: S2 the split sentence loses the causal link`.

A reason is required. An annotation without one is a defect, because the value
of the escape hatch is the record it leaves, not the suppression it performs.

**The mechanical subset is enforced.** `tools/check-site-voice.py` checks every
site-facing source — the specifications included — for retired spellings (W1,
from the register's `[spelling.map]`), retired synonyms (W2), `hit` aimed at a
control (W3), the number formats of §4, and contractions (V6). Word lists load
from `terminology.toml` at run time, so the gate and the register cannot drift,
and `ste-allow: <rule id> <reason>` waives a line.

**What the gate deliberately does not check:** sentence length (S1–S2), voice
(V1–V2), imperative form (P1–P2), and everything else that requires judgment.
Those live in review against this document, not in a regex — a gate that flags
judgment gets disabled, and then it checks nothing.

---

## 6. Relationship to other standards

| Standard | Status |
|---|---|
| **ASD-STE100 Issue 9** | Writing rules adopted and tiered. Dictionary not adopted. **Conformance is not claimed.** |
| **RFC 2119 / BCP 14** | Adopted for Tier B. |
| **ISO/IEC/IEEE 26514:2022**, **IEC/IEEE 82079-1:2019** | Adopted as the frame for structure, completeness, and findability — glossary, descriptions, document types. |
| **WCAG 2.2 AA / EN 301 549** | Applies to the sites as software. Not yet assessed; no conformance claimed. |

Rule numbers from ASD-STE100 are deliberately not cited in this document. The
official Issue 9 copy has not been obtained, and citing a rule number that turns
out to be wrong is worse than describing the rule. Obtain the copy — it is free
on request from asd-ste100.org — and add the numbers then.
