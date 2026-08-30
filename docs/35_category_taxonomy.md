# CFDL — The category taxonomy

Status: implemented in full. See §7.

A stream's `category` is the one thing that says what a flow *is*. Aggregation
reads it, subtotals fold it, and statements present what it folds. This note
proposes that the taxonomy be settled once, in the language, against the
international standard that already defines it — and that packs and models
classify into that one taxonomy by the same rule.

It closes `docs/13` §7.55's category half. The presentation half is §4 below.

---

## 1. What is wrong today

### 1.1 There are two validity rules, and activating a pack narrows the language

With no pack active, any dotted path rooted in `operating`, `investing` or
`financing` is valid. The diagnostic says so itself:

    E5022: whose root segment 'banana' is not one of operating, investing,
    financing. A category is a path into the cash flow statement, so it has to
    say which section it belongs to.

With a pack active, `packs/<pack>/pack.toml` declares `categories = [...]`, a
*closed vocabulary*, and that list replaces the open rule. The same category is
then valid or invalid depending on whether a pack is loaded:

| category | pack-less | with `cre` |
|---|---|---|
| `investing.acquisition.purchase` | valid | **E5022** |
| `operating.expense.rooms` | valid | **E5022** |

That is one mechanism for the language and a second for packs, and it
contradicts §7.55's own statement of intent — that the category roots are the
language's and a pack-less model classifies its streams correctly.

### 1.2 A contract instance cannot classify itself, and is not told

A `category` clause written on a contract instance parses, compiles clean, and
is discarded. Verified: a model declaring

    contract cre.opex_line.rooms on entity asset.p {
      category operating.expense.rooms
      ...
    }

compiles with no diagnostic, and the IR carries the lowering rule's value:

    stream: cre.opex.line.rooms   category: operating.expense.opex
    contracts: [('cre.opex_line.rooms', None)]

Note the category written there is not in the pack's closed list, so had the
clause been honoured it should have raised E5022. It raised nothing.

The mechanism is worse than a dropped field. `parse_contract_stmt`'s clause
dispatch ends in

    TokenKind::Punct(Punct::RBrace) => depth = depth.saturating_sub(1),
    _ => {}

a catch-all that silently swallows **every** token it does not recognise. Not
just `category`: any misspelled clause, or one the grammar does not have,
vanishes from a contract body without a diagnostic.

### 1.3 An uncategorized stream is silently absent from every subtotal

With a pack active, a native stream carrying no `category` compiles and runs.
Its cash is real — it reaches `model.total` and the entity roll-up — and it
folds into no subtotal at all. Verified on a two-period probe:

    model.total           2,000
    entity.asset.p.total  2,000
    domain.cre.noi            0

Nothing warns at compile time. Only the statement's residual row shows it, and
only if a reader looks.

### 1.4 A model cannot report the instances it is encouraged to declare

`docs/07` invites a modeller to instance `cre.opex_line` per expense. Every
instance carries the same category, and the statement's itemized rows select by
*stream name*. So a modeller who names their own instances gets correct
subtotals and a statement that files them all under `Unclassified`
(`W3500_STATEMENT_UNCLASSIFIED_STREAM`). The behavior is documented and
deliberate; whether it is acceptable is what this note asks.

---

## 2. What the standards say

### 2.1 Three activities, and the language already has them right

IAS 7 classifies every cash flow as operating, investing or financing. That is
the root set CFDL already enforces, and it is independently correct.

IFRS 18, effective for annual reporting periods beginning on or after
1 January 2027, extends the same three categories to the statement of profit or
loss (alongside income taxes and discontinued operations) and requires cash flow
classification to be consistent with profit-or-loss classification. A single
category on a stream, serving both the cash flow statement and an operating
statement, is therefore aligned with where the standards are going.

### 2.2 The standards define level 1 and stop

Neither IAS 7 nor IFRS 18 enumerates a second level. The IFRS Accounting
Taxonomy confirms it structurally: its schema is a flat list of 5,512 elements,
and the grouping that a reader sees lives in the presentation linkbases, not in
the element names. There is no level-2 vocabulary to borrow.

**Level 2 should therefore be open, not fixed.** An earlier draft of this note
proposed `operating.revenue.* / operating.expense.* / operating.deduction.*`.
That is wrong twice over: it conflates *classification* (which activity produced
the flow) with *direction* (which every stream already declares as `inflow` or
`outflow`), and the remainder of what it encodes — that vacancy shows above
effective gross income as contra-revenue — is presentation, which belongs to a
statement row rather than to a category.

### 2.3 Income tax is a line item, not a root

The taxonomy has exactly three activity roots:

    CashFlowsFromUsedInOperatingActivities
    CashFlowsFromUsedInInvestingActivities
    CashFlowsFromUsedInFinancingActivities

and no tax root. Tax appears as a line item classified into one of the three:

    IncomeTaxesPaidClassifiedAsOperatingActivities
    IncomeTaxesPaidRefundClassifiedAsFinancingActivities

matching IAS 7, under which tax cash flows are operating unless specifically
identifiable with financing or investing. IFRS 18's fifth category is a
profit-or-loss classification; the cash flow statement stays at three.

A fourth `income_tax` root was proposed and is withdrawn on this evidence. CFDL
still has nowhere to put a tax stream, which is a real gap — two workbooks in
the research corpus run cash flow through a tax ledger — but the fix is
`operating.income_tax.*` at level 2, not a root.

### 2.4 The taxonomy's names are a denormalised form of ours

The taxonomy encodes classification as a suffix, minting one element per
line item and permitted activity. CFDL expresses the same fact compositionally:

| IFRS taxonomy element | CFDL category |
|---|---|
| `InterestPaidClassifiedAsFinancingActivities` | `financing.interest_paid` |
| `InterestPaidClassifiedAsOperatingActivities` | `operating.interest_paid` |
| `InterestReceivedClassifiedAsInvestingActivities` | `investing.interest_received` |
| `DividendsPaidClassifiedAsFinancingActivities` | `financing.dividends_paid` |
| `IncomeTaxesPaidClassifiedAsOperatingActivities` | `operating.income_tax_paid` |

XBRL element names are flat, so the taxonomy has to denormalise. A dotted path
does not.

### 2.5 IFRS 18 removes the classification choice, and a pack should state which it applies

Under IAS 7 as it stands, an entity chooses: interest paid as operating or
financing, interest and dividends received as operating or investing, dividends
paid as financing or operating, applied consistently.

IFRS 18's consequential amendments remove that choice. For an entity **without**
specified main business activities: interest paid and dividends paid are
financing; interest received and dividends received are investing. For an entity
**with** specified main business activities — one that provides financing to
customers, or invests in assets — dividends paid remain financing, and interest
and dividends received follow the classification of the corresponding income and
expense in the statement of profit or loss.

Main business activity is a property of a domain, not of an individual deal, so
**the pack states it once** and its lowering rules follow:

| pack | main business activity | interest paid | interest received |
|---|---|---|---|
| `cre` | none specified | `financing.interest_paid` | `investing.interest_received` |
| `opco` | none specified | `financing.interest_paid` | `investing.interest_received` |
| `energy` | none specified | `financing.interest_paid` | `investing.interest_received` |
| `credit` | provides financing to customers | follows profit or loss | `operating.interest_received` |

`cre`'s current `financing.debt.interest_paid` becomes *correct under the standard* rather
than one permitted option of three. `credit` is a real change: a lender's
interest received is operating, not investing.

---

## 3. Proposal — one taxonomy, three authors

**One validity rule, everywhere.** A category is a dotted path whose first
segment is `operating`, `investing` or `financing`. Arbitrary depth below.
Identical whether or not a pack is active. This is the rule the language already
applies pack-less; the change is that loading a pack no longer narrows it.

**Three authors, one rule.**

- A **pack** states the category for its own contracts, in the lowering rule.
  It should: the pack knows what a `cre.permanent_debt` payment is.
- A **model** states the category for a stream no pack owns — the custom stream
  case, which already works pack-less.
- A **model** may override a pack instance where the pack's default is wrong for
  the deal, PER STREAM. A contract lowers one or more streams and its pack
  states a category for each, so an override names which one it means:
  `category <stream> = <path>`. The bare form is sugar for a contract lowering
  exactly one stream and is refused where it would flatten several (`E5030`) —
  a permanent mortgage whose interest, principal and proceeds all became one
  category made every coverage ratio computed off them wrong, silently.

**The pack's list demotes from gate to advice.** `categories = [...]` becomes a
recommended vocabulary. A well-rooted category that is not on it is valid and
raises a warning naming the near match, so `operating.expence.opex` is still
caught. Packs already ship conventional vocabulary as suggestion rather than law
in `templates.toml`; this makes categories consistent with that.

**An uncategorized stream warns** when a pack is active (§1.3). Silent exclusion
from every subtotal is worth money.

---

## 4. The presentation half

Opening the namespace fixes aggregation and not presentation. Thirteen
`cre.opex_line.<name>` instances would fold correctly into `domain.cre.noi` and
still render in the residual row, because statement rows select stream names.

The complement is a row that **itemizes a category subtree** — one line per leaf
under `operating.expense.*` — rather than a row per enumerated stream name. That
is the same mechanism applied at the presentation layer, not a second grammar,
and it is materially less surface than a model-declared statement.

Whether a model may also declare whole statements is left open here. It is the
larger half of §7.55 and it should be decided after this lands, because a
category subtree row may remove most of the demand for it.

---

## 5. Migration

Renaming level 2 and 3 to the shapes in §2.4 touches:

- four packs: `pack.toml` category lists, `lowering/rules.toml`,
  `statements.toml`, `metrics.toml`;
- every benchmark model that names a category, and its IR, results and
  diagnostic goldens;
- `docs/07`, and the diagnostic text for E5022.

It is mechanical and wide. `make gold-update` would need reading rather than
blind blessing, because a category rename moves the rows a statement renders.

**The IFRS 18 boundary is dated.** It applies to periods beginning on or after
1 January 2027. Models with earlier horizons were entitled to the IAS 7 policy
choice. Whether the packs simply adopt the new treatment, or record both and
select on the model's horizon, is a decision to take deliberately rather than by
default.

---

## 6. Open questions

1. May a model open a node the pack left closed, or only use the three roots?
   This note says the roots are the only gate, which means yes.
2. Should the warning in §3 be an error behind a strict flag?
3. Does the category subtree row (§4) remove enough demand to defer
   model-declared statements indefinitely, or only to postpone them?

---

## Sources

- IAS 7 *Statement of Cash Flows*, IFRS Foundation.
- IFRS 18 *Presentation and Disclosure in Financial Statements*, and its
  consequential amendments to IAS 7; effective 1 January 2027.
- IFRS Accounting Taxonomy 2025 core schema,
  `xbrl.ifrs.org/taxonomy/2025-03-27/full_ifrs/full_ifrs-cor_2025-03-27.xsd`
  — 5,512 elements, read for the activity roots and the classification suffixes
  quoted in §2.3 and §2.4.

Diagnostics and IR output quoted in §1 were reproduced against
`cfdl-engine 0.7.0` on the `cre` pack v0.1.0.


---

## 7. What shipped

All of it.

- **One validity rule.** A pack no longer narrows the language's; `E5022`
  reports only a bad root, with the same text pack or no pack.
- **`E5029_STREAM_MISSING_CATEGORY`.** An uncategorized stream is an error while
  a pack is active and legal without one.
- **Categories per lowered stream.** A rule that emits a stream states its
  category, checked at pack load; a rule that lowers only a field is exempt,
  because a field is not classified into a cash flow statement.
- **Per-stream override,** `category <stream> = <path>`, with `E5030` refusing
  the bare form where it would flatten several streams onto one category.
- **`W5023_UNRECOGNIZED_PACK_CATEGORY`,** naming a near match one edit away.
- **The vocabulary of §2.4,** thirteen renames onto the taxonomy's normalized
  form, and `operating.income_tax.*` giving tax the home §2.3 says it needs.
- **The stance of §2.5.** Each pack states its main business activity and the
  treatment that follows: `cre`, `opco` and `energy` have none specified;
  `credit` provides financing to customers, so its interest received is
  operating rather than investing.

Presentation is unchanged. `docs/13` §7.55's statement half is untouched and
remains open.
