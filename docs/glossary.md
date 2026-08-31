# Glossary

Every term CFDL uses in a specific sense, with the one meaning it carries.

Terms are listed with one definition each. Where a term is an abbreviation,
the expansion given is the form to use on first mention.

## Language

The constructs a model is written from.

**account**

A declared cash location whose balance accumulates across periods under the balance law; drawn by a waterfall, credited by a step, read by logic as settled history.

**assumption**

A declared input to the model, constant or stochastic. Declared with `assume`.

**config namespace**

The `cfg.<path>` expression binding, which reads scenario knobs from the run configuration. Distinct from the run configuration itself.

**container**

An entity family for a grouping that scopes cash without producing it — a fund, a portfolio, an SPV, a transaction.

**contract**

A first-class agreement carrying terms and effects.

**curve**

A named series of values indexed by time.

**entity**

A thing the model is about. An asset produces or consumes cash; a party contracts, owns, or lends.

**event**

Something that happens. An event fires on each occurrence, and nothing restricts it to happening once.

**fact field**

A field on an entity that takes a literal and nothing else. A value that moves must be a rule field.

**grain**

How finely the model slices time and things. Timeline grain and entity grain are independent choices.

**lifecycle**

A declared finite state machine: enumerated states, an initial one, and guarded edges declared only as used. A core-language construct that packs tailor to domains.

**master type**

A base type in the language's ontology that pack types refine — what a thing is, before a domain specializes it.

**metric**

A derived summary figure over the model's cash flows.

**option**

An electable right held by a party.

**pack**

A domain vocabulary supplying types, roles, and terms to a model.

**priced amount**

A stream amount whose series window reaches forward: a valuation setting a causal amount, evaluated after the causal cells settle, refused where the graph cycles.

**quantile**

A named series of values indexed by cumulative share.

**refinement**

The recorded is-a edge from a pack type to the type it specializes, declared with refines.

**rule field**

A field carrying a recurrence: `init` gives the first period's value, `next` is evaluated each later period with `prev` bound to the field's own previous value.

**scenario**

A named set of overrides applied to a run.

**schedule**

The set of dates on which a stream pays.

**statement**

A per-period report published from the model's results.

**stream**

A dated, directed movement of cash attached to an entity and laid out on the timeline.

**waterfall**

An ordered distribution of cash through steps by seniority.

## Compiler

What the toolchain does with a model, and what it reports.

**diagnostic**

A compiler message carrying a stable code and a source span.

**IR** — intermediate representation (IR)

The canonical JSON intermediate representation a model compiles to.

**lowering**

The compiler stage that reduces language constructs to the canonical IR. A technical noun, so exempt from the rule against -ing forms.

**span**

The region of source a diagnostic points at.

## Finance

Domain terms the packs and the documentation use in their standard sense.

**cap rate**

The capitalization rate applied to stabilized income to derive value.

**catch-up**

The waterfall tier that restores a party to its target share after a preferred return is paid.

**covenant**

A contractual test the borrower must satisfy.

**DSCR** — debt service coverage ratio (DSCR)

Debt service coverage ratio.

**EBITDA**

Earnings before interest, taxes, depreciation, and amortization.

**expense stop**

The expense level above which recoveries are billed to a tenant.

**IRR** — internal rate of return (IRR)

Internal rate of return: the discount rate at which NPV is zero. Undefined for a series that never changes sign.

**lease-up**

The period during which vacant space is let to stabilized occupancy.

**MOIC** — multiple of invested capital (MOIC)

Multiple of invested capital.

**NPV** — net present value (NPV)

Net present value.

**PPA** — power purchase agreement (PPA)

Power purchase agreement.

**promote**

The sponsor's disproportionate share of profit above a return hurdle.

**reversion**

The terminal value realized at the end of a hold.

**takeout**

Permanent financing that repays construction debt.

**term power purchase agreement**

A power offtake contract for a fixed term.

**weighted average life**

The average time to principal return, weighted by principal repaid.

**working capital**

Receivables, payables, and inventory, modeled on a days-based policy.

## Verbs

Each of these describes one action and is used for no other.

**accrue**

To recognize an amount in a period without paying it.

**compile**

To translate a model into the canonical IR.

**diff**

To compare two artefacts and report the differences.

**discount**

To apply a discount rate to a future cash flow.

**escalate**

To increase an amount on a stated schedule.

**evaluate**

To compute a value for a period.

**lower**

To reduce a language construct to its IR form.

**resolve**

To bind a name to its declaration.

**seed**

To fix the random draw so a stochastic run reproduces.
