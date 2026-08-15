# 23 — Accessibility assessment (WCAG 2.2 AA)

Assessment of cfdl.dev and learn.cfdl.dev against WCAG 2.2 level AA, and the
remediation applied from it. Backlog item 10 from the documentation standards
audit (`docs/21`).

**This document does not claim WCAG 2.2 AA conformance.** It records what an
automated and scripted assessment found, what was fixed, and what a conformance
claim would still require (§6). The distinction matters: automation covers
roughly a third of WCAG's success criteria; the rest need a human with
assistive technology.

---

## 1. Scope and method

**Tooling.** axe-core 4.13.0, injected into live pages and run with the
`wcag2a`, `wcag2aa`, `wcag21a`, `wcag21aa`, `wcag22aa` rule tags. Scripted DOM
probes for what axe samples poorly: keyboard focus order, `:focus-visible`
styling, scroll-container reachability, landmark structure, reflow.

**Surfaces.** Production builds (`next build` + `next start`) of both apps,
and — because a local build is not what a reader reaches — the deployed
**cfdl.dev** and **learn.cfdl.dev**, which matched the local results exactly.

**Pages.** Site: landing, docs index, getting-started, concepts, glossary,
diagnostics reference (largest tables), language spec (longest page), a
generated benchmark example, a notebook page (images), the playground
(interactive). Learn: home, two chapters including one with the embedded
exercise editor. Both themes, via the production theme path (stored preference
plus reload — see §5 for why that matters).

**Viewports.** Desktop, 1500px (playground splitter renders only ≥1024px), and
320px for reflow.

---

## 2. Findings

| # | Finding | WCAG SC | Severity | Where |
|---|---|---|---|---|
| F1 | `--cfdl-text-muted` (#6b7888) below 4.5:1 in **both themes** — 4.49:1 on white, 4.18:1 on the dark page, 3.81:1 on the dark raised surface | 1.4.3 Contrast (Minimum) | Serious, systemic — every page, 5–20 nodes each | Both apps |
| F2 | Playground splitter: focusable `role="separator"` without `aria-valuenow/min/max`. ARIA requires the value trio on a focusable separator — a screen reader announced a resizer it could not describe | 4.1.2 Name, Role, Value | Critical | Site |
| F3 | Scrollable regions unreachable by keyboard: wide-table wrappers, code-block `pre`s, and the playground results panel scroll horizontally or vertically with no way to focus them | 2.1.1 Keyboard | Serious | Both apps |
| F4 | Learn home: a link inside muted body text distinguishable only by color (2.18:1 against the surrounding text), underline on hover only | 1.4.1 Use of Color | Serious | Learn |
| F5 | No skip link; first tab stop is the logo | 2.4.1 Bypass Blocks | Advisory — landmarks (`header`/`nav`/`main`/`footer`) exist and satisfy the SC | Both apps |

**What passed.** Landmark structure, one `h1` per page, `html lang`, descriptive
titles, a global `:focus-visible` outline (`2px solid var(--cfdl-accent-ring)`,
offset 2), reflow at 320px with no horizontal page scroll in either app,
image alt text on the notebook pages, no keyboard trap detected in the
playground editor, and — after the earlier standards work — link text and
heading structure throughout. axe's 2.2-specific rules (target size, focus
appearance heuristics) reported nothing.

---

## 3. Remediation applied

All five findings were fixed in the same change as this assessment, and each
fix was verified by re-running the failing check against a rebuilt production
server.

**F1 — the muted token, split per theme.** Light and dark both mapped
`--cfdl-text-muted` to the same primitive (`--p-slate-500`), and the two themes
needed to move in **opposite directions** — light darker, dark lighter — so no
single primitive could fix both. The semantic token now carries its own value
per theme:

| Theme | Was | Now | Page | Raised | Sunken |
|---|---|---|---|---|---|
| Light | #6b7888 (4.49) | **#5f6c7b** | 5.36:1 | 5.04:1 | — |
| Dark | #6b7888 (4.18) | **#8391a3** | 5.86:1 | 5.34:1 | 6.03:1 |

`--cfdl-chart-axis` keeps slate-500: figures render on
`--cfdl-surface-figure`, which is white in both themes, where #6b7888 is
exactly 4.50:1.

**F2 — the splitter now carries the value trio**, wired to the split state and
mirroring the 20/80 clamp in `move()`. Verified live: focus, press ArrowLeft,
`aria-valuenow` moves 50 → 48. The keyboard operability was already there; the
element just could not report itself.

**F3 — scroll containers are focusable and named.** The table wrapper and
code-block `pre` in `mdx-components.tsx` (both apps) take `tabIndex={0}`; the
table wrapper is a named `region`. The playground results panel and its
expanded overlay likewise.

**F4 — the learn home link is underlined at rest**, `decoration-accent-text/40`
strengthening on hover.

**F5 — not fixed.** Landmarks satisfy 2.4.1; a skip link remains worth adding
the next time the header is touched, and is noted in the backlog entry rather
than done here.

**Verification.** After rebuild, axe reports **zero violations** on every page
in the sweep, both themes, including the playground at splitter width and the
diagnostics reference with its tables and code blocks.

---

## 4. What automation cannot claim, and what remains

A conformance claim needs a human assistive-technology pass. Specifically:

- **Screen reader sessions** (VoiceOver/Safari at minimum) over the docs
  reading flow, the playground round trip (edit → run → read results), and a
  learn exercise (read prompt → edit → run → compare). The exercise editor
  exposes `role="textbox"` with an accessible name and a hidden IME surface —
  present, but whether the *experience* is usable is exactly what a rule
  cannot say.
- **2.2's judgment criteria**: focus appearance in every state, dragging
  alternatives (the splitter has arrow keys — verify equivalents exist
  wherever dragging appears), consistent help placement.
- **Cognitive review** of error messages and the run-configuration dialog.
- **Zoom at 200/400%** beyond mechanical reflow: whether content *order* still
  reads.

This is backlog 7.35. Until it is done, the honest public statement is "built
to WCAG 2.2 AA; formal conformance assessment in progress" — not a claim.

---

## 5. Notes for whoever runs this next

- **Test themes through the production path** — set the stored preference and
  reload. Stamping `data-theme` on a live page mid-session produced mixed token
  states (dark accents on white ground, 2.05:1) that look like catastrophic
  bugs and are artifacts: no such state is reachable in production, where
  next-themes stamps the attribute before first paint and there are no
  `prefers-color-scheme` token blocks to fight it. Two false findings in this
  assessment died on that hill before the method was corrected.
- The splitter renders only ≥1024px and the playground panel layout changes
  with width; sweep both sides of the breakpoint.
- axe's `incomplete` bucket on these pages is mostly gradient-background
  contrast it cannot compute; spot-check those by hand.
- The deployed sites matched local builds byte-for-typical-page during this
  assessment; if that ever stops being true, assess the deploy, not the build.
