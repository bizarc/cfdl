---
id: reference-diagnostics
title: "Diagnostics"
slug: "/docs/reference/diagnostics"
generated: regions
---

# Diagnostics

Every problem CFDL reports carries a stable code, a severity, and a source
span. The code is the part worth knowing: it does not change when the message
is reworded, so it is what to search for and what to cite.

```
ERROR[E2108_SCHEDULE_FINER_THAN_CALENDAR] A schedule may not be finer than the
model's calendar.
  --> model.cfdl:14:3
```

## Severity

**error** stops compilation or the run — nothing is produced. **warning** lets
the work finish but flags something that is probably not intended; benchmark
runs treat any warning as a failure, so a model that warns is a model to look
at. **info** is advisory.

## Reading a code

The number encodes where in the pipeline the problem was found, which usually
tells you what kind of thing is wrong:

| Range | Raised by | Typically means |
|---|---|---|
| `E0xxx` | lexer and parser | a syntax error — a missing brace, an unterminated string |
| `E11xx`–`E13xx` | resolver | structure and names — a duplicate declaration, a reference to something not declared |
| `E2xxx` | validation | a declaration that parses but cannot mean anything, such as a schedule finer than the calendar |
| `E3xxx` | expressions | typing and evaluation |
| `E4xxx` | pack loading | a pack that could not be read or accepted |
| `E5xxx` | lowering | the compiler could not turn a valid model into IR |
| `E6xxx`–`E9xxx` | pack validations | a domain rule a pack declares, such as a missing lease term |

A pack owns its own range, so a domain rule can be added without colliding with
the language's codes.

## The register

Every code CFDL can emit. This table is generated from the specification's
register, so it cannot fall behind the language.

<!-- cfdl:generated diagnostics-catalogue -->
| Code | Family | Meaning |
|---|---|---|
| `E0001_UNEXPECTED_TOKEN` | Parse errors | the parser met a token it cannot use here. |
| `E0002_UNTERMINATED_STRING` | Parse errors | a string literal opens and never closes. |
| `E0003_UNTERMINATED_BLOCK_COMMENT` | Parse errors | a `/*` block comment opens and never closes. |
| `E0004_EXPECTED_TOKEN` | Parse errors | something specific was required at this position and is missing. The message names what. |
| `E0005_INVALID_DATE_LITERAL` | Parse errors | a date is not a real calendar date, or not in `YYYY-MM` or `YYYY-MM-DD` form. |
| `E1201_IMPORT_CYCLE` | Module/import | two files import each other, directly or through a chain. |
| `E1202_IMPORT_NOT_FOUND` | Module/import | an imported file does not exist at that path. |
| `E1203_IMPORT_OUTSIDE_MODEL_ROOT` | Module/import | an import reaches outside the model's directory. A model is self-contained, so it can be moved or shared without carrying hidden dependencies. |
| `E1101_MISSING_VERSION` | Global structure | no `version` declaration. It states which language version the model is written against. |
| `E1102_MISSING_MODEL` | Global structure | no `model` declaration, so the model has no name. |
| `E1103_MISSING_TIME` | Global structure | no `time` declaration. Without a timeline there is no grid to evaluate amounts on. |
| `E1104_MULTIPLE_VERSION` | Global structure | `version` is declared more than once. |
| `E1105_MULTIPLE_MODEL` | Global structure | `model` is declared more than once. |
| `E1106_MULTIPLE_TIME` | Global structure | `time` is declared more than once. A model has one timeline. |
| `E1107_MULTIPLE_USE_PACK` | Global structure | more than one `use pack`. A model draws contracts from a single pack. |
| `E1108_USE_PACK_NOT_IN_MODEL_FILE` | Global structure | `use pack` appears in an imported file rather than the model's own. The pack applies to the whole model, so it is declared where the model is. |
| `E1109_MISSING_ENTITY` | Global structure | no entity is declared. Every stream belongs to one. |
| `E1120_STATE_MISSING_INIT` | Global structure | a `state` has no `init`. The value at period 0 is required, not defaulted: a recurrence with an unstated base case would evaluate to zero for every period, and an out-of-range read returns zero silently, so the error would never surface. |
| `E1121_STATE_MISSING_NEXT` | Global structure | a `state` has no `next`. It would hold its initial value forever, which an `assume` expresses more clearly. |
| `E1122_STATE_DUPLICATE_NAME` | Global structure | two states share a name. |
| `E1123_STATE_PREV_OUTSIDE_NEXT` | Global structure | `prev` appears in `init`. There is no period |
| `E1127_FIELD_RULE_READS_FIELD` | Global structure | a field's rule names another field by its family path. A field means this period's value at close, which does not exist yet inside a rule; `prev <entity>.<field>` says the previous period. Unrejected it would resolve through the open-world entity root, return null and evaluate to zero. |
| `E1128_FIELD_DECLARED_TWICE` | Global structure | a field is declared both with `=` and with a rule. Both bind the same path, so one would silently win. before the first, so the initial value cannot depend on a previous one. |
| `E1124_STATE_SAME_PERIOD_READ` | Global structure | `state.<name>` appears inside a `next`. That path reads the **current** period, which is the same-period edge the design exists to prevent; `prev.<name>` reads another state's previous value. |
| `E1126_STATE_INIT_READS_STATE` | Global structure | `state.<name>` appears inside an `init`. Every state is seeded at period 0 together, so there is no order in which one already holds a value for another to read: the expression evaluated to zero and said nothing. The same edge `next` rejects, one period earlier. Inline the expression, or read the state from a stream or waterfall, both of which see period-close values. |
| `E1125_STATE_UNKNOWN_REFERENCE` | Global structure | a `state.<name>` or `prev.<name>` names a state that is not declared. Without this the reference reaches the engine, which warns and substitutes zero — so an entire series evaluates to nothing while the run still reports `status: ok`. |
| `E1001_DUPLICATE_ENTITY` | Symbols and references | two entities share a name. |
| `E1002_DUPLICATE_CONTRACT` | Symbols and references | two contracts share a name. Give one a suffix to keep them separable. |
| `E1003_DUPLICATE_STREAM` | Symbols and references | two streams share a name. |
| `E1004_DUPLICATE_PHASE` | Symbols and references | two phases share a name. |
| `E1005_DUPLICATE_ASSUME` | Symbols and references | two assumptions share a name. |
| `E1006_DUPLICATE_OPTION` | Symbols and references | two options share a name. |
| `E1007_DUPLICATE_EVENT` | Symbols and references | two events share a name. |
| `E1301_UNRESOLVED_ENTITY_REF` | Symbols and references | a stream, contract or event action names an entity that is not declared. |
| `E1302_UNRESOLVED_STREAM_REF` | Symbols and references | an event activates or deactivates a stream that is not declared. Event action targets were never resolved, so a misspelling matched nothing and the action was silently inert: the stream it was meant to stop kept paying, with no diagnostic and no warning. |
| `E1303_UNRESOLVED_CONTRACT_REF` | Symbols and references | an event activates or deactivates a contract that is not declared. |
| `E1304_UNRESOLVED_OPTION_REF` | Symbols and references | an event exercises an option that is not declared. Checked in the compiler rather than the resolver, because options are not in the symbol tables. |
| `E1310_ENTITY_BLOCK_WITHOUT_TYPE` | Symbols and references | an entity uses a block but declares no type, so there is nothing to check the block against. |
| `E1311_UNKNOWN_ENTITY_TYPE` | Symbols and references | an entity declares a type the active ontology does not define. The known types are listed. |
| `E1312_MISSING_REQUIRED_FIELD` | Symbols and references | an entity omits a field its type requires. |
| `E1313_UNKNOWN_ENTITY_FIELD` | Symbols and references | an entity sets a field its type does not declare. The declared fields are listed. |
| `E1314_UNKNOWN_PARENT_ENTITY` | Symbols and references | `part of` names an entity that is not declared. Hierarchy is optional; a declared parent is not. |
| `E1315_ENTITY_PART_OF_ITSELF` | Symbols and references | an entity is its own parent. |
| `E1330_CONFLICTING_ACTIVE_CLAUSES` | Symbols and references | a stream declares both `active when` and `active in state`. Use one: `active in state` for a lifecycle state, `active when` for anything else. |
| `E1331_OWNER_HAS_NO_LIFECYCLE` | Symbols and references | a stream is active in a lifecycle state but its owner's type declares no lifecycle. |
| `E1332_UNKNOWN_ACTIVE_STATE` | Symbols and references | a stream is active in a state its owner's lifecycle does not declare. A state name is checked against the lifecycle; a string comparison such as `entity.state.status == "leasd"` is not, and stays false for every period. |
| `E1318_ENTITY_HIERARCHY_CYCLE` | Symbols and references | `part of` forms a cycle. Reported once, from the cycle's lexicographically first entity, rather than once per member. An entity aggregates its children, so a cycle has no bottom to sum from. |
| `E1316_UNKNOWN_LIFECYCLE_STATE` | Symbols and references | an entity starts in a state its lifecycle does not declare. This is the misspelled status made impossible rather than merely unlikely. |
| `E1317_TYPE_HAS_NO_LIFECYCLE` | Symbols and references | an entity declares a starting state but its type has no lifecycle. |
| `E1320_UNKNOWN_PARTY_ENTITY` | Symbols and references | a contract or option binds a role to an entity that is not declared. |
| `E1321_NOT_A_PARTY` | Symbols and references | a role is bound to an asset. A contract is between parties. |
| `E1322_UNKNOWN_PARTY_ROLE` | Symbols and references | a role is bound that the contract type does not declare. The declared roles are listed; a role belongs to the agreement, not to the entity. |
| `E1305_UNRESOLVED_PHASE_REF` | Symbols and references | a schedule names a phase that is not declared. |
| `E1306_INVALID_ENTITY_REF_FORMAT` | Symbols and references | entity ref, stream name, or contract name is not a qualified name with at least two segments (dotted hierarchy). |
| `E2001_CONTRACT_MISSING_TERM` | Contracts and streams | a contract omits a term its pack requires. The message names it; see the pack's contract table. |
| `E2002_CONTRACT_MISSING_EFFECTS` | Contracts and streams | a contract produces no streams, so it has no effect on the model. |
| `E2003_CONTRACT_CURRENCY_REQUIRED` | Contracts and streams | a contract does not state its currency and none can be inferred. |
| `E2101_STREAM_MISSING_SCHEDULE` | Contracts and streams | a stream has no `schedule`, so there is no period for its cash to land in. |
| `E2102_STREAM_MISSING_AMOUNT` | Contracts and streams | a stream has no `amount`. |
| `E2103_SCHEDULE_OUT_OF_BOUNDS` | Contracts and streams | a schedule reaches outside the model timeline. The bound is the cash horizon **plus** any `project <n>` tail, since the engine evaluates streams over both; a schedule may reach into the tail deliberately to feed a `series_sum` valuation. Applied to hand-written streams during validation and mirrored onto pack-lowered ones during lowering, so a pack cannot express what a model may not. |
| `E2104_SCHEDULE_INVALID_RANGE` | Contracts and streams | a schedule's `to` is before its `from`. |
| `E2105_SCHEDULE_INVALID_DAY_OF_MONTH` | Contracts and streams | a day rule names a day outside 1–31. |
| `E2106_SCHEDULE_PHASE_NOT_FOUND` | Contracts and streams | a schedule is anchored to a phase that is not declared. |
| `E2107_STREAM_CURRENCY_MISMATCH` | Contracts and streams | a stream's currency differs from the model's reporting currency. Cash flows are summed period by period, so the two would be added as if they were the same unit. Convert explicitly in the amount expression, or declare the model in that currency. |
| `E2108_SCHEDULE_FINER_THAN_CALENDAR` | Contracts and streams | the schedule's interval is finer than the model's calendar cadence. The occurrences are not lost: a period holds many accruals and their amounts **sum**, which is the same machinery a settlement lag uses. What cannot be done is telling them apart — an accrual is stored as a model period index, so occurrences inside one period share an environment, and an amount that varies over time is computed once and multiplied rather than summed across the occurrences. A constant amount would be exact; anything else is silently wrong, so both are rejected. Use a coarser interval, or declare a finer calendar. |
| `E2109_SCHEDULE_CONFLICTING_PLACEMENT` | Contracts and streams | a schedule combines `mid` with `due`, a day rule, or `net` payment terms. Each states where in its period the cash sits; two placements is a contradiction, not a refinement. |
| `E2201_EVENT_WHEN_NOT_BOOL` | Events and actions | an event's `when` is not a true/false expression. |
| `E2202_STREAM_ACTIVE_NOT_BOOL` | Events and actions | a stream's `active when` is not a true/false expression. |
| `E2203_ACTION_SET_FIELD_INVALID` | Events and actions | an event sets an entity field that does not exist or cannot hold that value. |
| `E3001_EXPR_PARSE_ERROR` | Expressions / typing | an expression is not valid CFDL. |
| `E3002_EXPR_UNKNOWN_IDENT` | Expressions / typing | an expression names something not in scope. Bindings are `time.*`, `inputs.*`, `model.*`, `entity.*`, `cfg.*`, `obs.*` and declared states. |
| `E3003_EXPR_TYPE_ERROR` | Expressions / typing | an expression combines types that cannot combine, such as a date and a number. |
| `E3004_EXPR_ILLEGAL_OP` | Expressions / typing | an operator is not defined for these operands. |
| `W3001_EXPR_TYPE_UNKNOWN` | Expressions / typing | an expression's type could not be determined ahead of evaluation. It still runs; the warning notes the check was skipped. |
| `W3002_OBS_REF_EXTRACTION_FAILED` | Expressions / typing | an observation reference could not be read out of an expression, so the run may not know it needs that input. |
| `E4001_UNKNOWN_TYPE_ID` | Pack errors | a declaration names a type the active pack does not define. |
| `E4002_INVALID_ENTITY_ATTR` | Pack errors | an entity field is not one the pack declares, or holds the wrong kind of value. |
| `E4003_INVALID_CONTRACT_TERMS` | Pack errors | a contract's terms do not satisfy the pack's schema for that contract. |
| `E4004_MISSING_PACK` | Pack errors | the named pack could not be loaded — not found, or found and rejected. |
| `E5001_ID_GENERATION_FAILED` | Lowering/emission | the compiler could not derive a stable identifier for a declaration. |
| `E5002_IR_SCHEMA_VALIDATION_FAILED` | Lowering/emission | the IR the compiler produced does not satisfy the published IR schema, or the IR being read does not. |
| `E5003_IR_EMIT_FAILED` | Lowering/emission | the IR could not be written. |
| `E5004_INVALID_LOWERING_RULE` | Lowering/emission | a pack's lowering rule is malformed. |
| `E5005_PHASE_NOT_FOUND` | Lowering/emission | a lowering rule anchors to a phase the model does not declare. |
| `E5006_MISSING_CONTRACT_TERM` | Lowering/emission | a lowering rule reads a contract term the contract does not supply. |
| `E5007_DUPLICATE_LOWERED_STREAM` | Lowering/emission | two contracts lower to the same stream name. Give one a suffix. |
| `E5008_INVALID_CURVE` | Lowering/emission | duplicate curve name, duplicate point date, or malformed point in a `curve` statement |
| `E5009_LOWERED_EXPR_INVALID` | Lowering/emission | a pack lowering rule expanded to an amount expression the parser rejects. Without this the engine evaluates the failed expression as zero and continues with only a warning. |
| `E5020_LOWERED_STATE_INVALID` | Lowering/emission | a pack lowering rule expanded to a `state` `init` or `next` the parser rejects. Same reasoning as `E5009`: the engine's fallback for a failed state is zero, which would flatten every stream reading it rather than fail loudly. |
| `E5021_DUPLICATE_LOWERED_STATE` | Lowering/emission | two contracts lower to one state name with *different* recurrences, so one would silently win. Give the rule's `state_name` a per-contract discriminator (`{{contract.suffix_ident}}`). Identical definitions collapse instead, which is what several contracts sharing one curve should do. |
| `W3500_STATEMENT_UNCLASSIFIED_STREAM` | Lowering/emission | cash that no row of the statement claims, usually a hand-written stream carrying no `category`. It is collected into a visible `residual` row rather than dropped, so the bottom line still reconciles and the omission is on the page instead of in the difference. The pack loader checks the same property for declared CATEGORIES statically; this is the half that needs a run, because a stream with no category at all is invisible until one happens. |
| `W3501_STATEMENT_STREAM_DOUBLE_COUNTED` | Lowering/emission | a stream claimed by more than one row. Worse than an omission: the bottom line is then wrong in a direction that looks entirely plausible. |
| `W3502_STATEMENT_BOTTOM_LINE_RESIDUAL` | Lowering/emission | the statement's rows do not sum to `model.total` within half a cent. Asserted, never corrected. |
| `E5022_UNKNOWN_STREAM_CATEGORY` | Lowering/emission | a stream declares `category <path>` that the active pack does not list in its manifest `categories`. A category is a dotted path into the cash flow statement (`operating.deduction.abatement`) and is what a fold aggregates on, so an unlisted one would leave the stream reported as a line and counted in no subtotal — visible and wrong, rather than absent and obvious. Use one the pack declares, or add it to the pack's vocabulary. With no pack in use there is no vocabulary, so any category is unknown. A pack whose own vocabulary is not rooted in `operating`, `investing` or `financing` fails to load rather than reaching this check. |
| `E5010_TERM_UNKNOWN_INPUT` | Lowering/emission | a contract term references `inputs.<name>` for an input that is not declared. Declare it with `assume <name> = …` or `assume <name> ~ <Dist>(…)`. |
| `E5012_RULE_INVALID_INTERVAL` | Lowering/emission | a lowering rule's `schedule_every` is not one of `day`, `week`, `month`, `quarter`, `year`. |
| `E5011_TERM_CLIP_OUT_OF_BOUNDS` | Lowering/emission | a term defers to an input whose `clip` can produce values outside the range the pack allows for that term. The value itself cannot be checked until the run, but the clip states the range the driver can reach, so it can be. |
| `E5013_PACK_CADENCE_UNSUPPORTED` | Lowering/emission | the model's calendar is not one the pack declares in `cadences`. A pack whose expressions divide annual figures by a literal 12 assumes one period is one month; on any other grid the *schedule* adapts correctly and only the *amount* does not, so the model produces plausible figures out by a factor of twelve. Refusing to lower is the only safe option. Use a calendar the pack supports, or a pack that supports the calendar. |
| `E5014_RULE_CADENCE_UNSUPPORTED` | Lowering/emission | as above, but declared by one lowering rule rather than the whole pack. This exists so a pack can carry neutral and month-locked rules side by side while it is being migrated, instead of being gated wholesale. |
| `E5018_TERM_START_OFF_GRID` | Lowering/emission | a pack contract's `term_start` does not fall on one of the model's period boundaries. Periods step from the model's start by whole calendar units, and elapsed-period counting measures whole steps from the term, so a term beginning mid-period counts short for the contract's whole life. Always satisfied on a monthly calendar, where every `YYYY-MM` term is a boundary. |
| `E5015_TERM_MONTHS_NOT_DIVISIBLE` | Lowering/emission | a `_months` term used as a count of payment periods does not divide into whole periods on this grid. A 30-month loan is not two and a half annual payments, and no closed form can express one, so this is an error rather than a rounding. Thresholds such as `free_rent_months` pro-rate instead and never reach here. |
| `E5016_RESERVED_TERM_PREFIX` | Lowering/emission | a contract term begins `model.`, `time.`, `periods.` or `whole_periods.`. Lowering rules resolve those prefixes before contract terms, so the term would be shadowed and never read. Term keys may legitimately be dotted, so this is reachable by accident. |
| `E5017_PERIOD_TERM_NOT_LITERAL` | Lowering/emission | a `_months` term that a rule converts into periods defers to `inputs.<name>`. The conversion happens at compile time and an input is not known until the run. |
| `E5019_UNKNOWN_DAY_COUNT` | Lowering/emission | a contract's `day_count` or `amortization_day_count` is not one of `30/360`, `30e/360`, `act/360`, `act/365`. Not defaulted silently: the gap between act/360 and act/365 is roughly 1.4% of interest. |
| `E6001_CRE_LEASE_MISSING_BASE_RENT` | Pack domain validations |  |
| `E6002_CRE_LEASE_INVALID_TERM_RANGE` | Pack domain validations |  |
| `E6003_CRE_LEASE_UP_MISSING_MONTHS` | Pack domain validations |  |
| `E6010_CRE_EXIT_MISSING_EXIT_CAP` | Pack domain validations |  |
| `E6011_CRE_EXIT_INVALID_EXIT_CAP` | Pack domain validations |  |
| `E6012_CRE_EXIT_MISSING_NOI_VALUE` | Pack domain validations |  |
| `E6020_CRE_OPS_MISSING_AMOUNT` | Pack domain validations |  |
| `E6021_CRE_OPS_INVALID_SCHEDULE` | Pack domain validations | pair: the first owns absent-or-unparseable, the second parsed-but-not-positive for the nominal annual rate |
| `E6054_CRE_DEBT_INVALID_AMORT` | Pack domain validations | `amort_months` strikes the payment and is normally longer than the loan's term |
| `E6055_CRE_DEBT_INVALID_IO_MONTHS` | Pack domain validations | whole months, 0 or more |
| `E6056_CRE_DEBT_INVALID_BALLOON_FLAG` | Pack domain validations | `balloon_at_maturity` is 0 or 1 |
| `E7001_OPCO_LINE_MISSING_AMOUNT` | Pack domain validations |  |
| `E7002_OPCO_LINE_INVALID_SCHEDULE` | Pack domain validations |  |
| `E7003_OPCO_LINE_INVALID_GROWTH` | Pack domain validations |  |
| `E7025_OPCO_PERPETUITY_RATE_NOT_ABOVE_GROWTH` | Pack domain validations | a growing perpetuity needs `discount_rate` strictly above `growth_rate`. At or below it the denominator reaches zero and then goes negative, so the contract would return a huge value and then a negative one with nothing to say the model had stopped meaning anything. |
| `E7026_OPCO_PERPETUITY_MISSING_BASE_VALUE` | Pack domain validations | the terminal-period flow the perpetuity is struck on. |
| `E7027_OPCO_PERPETUITY_MISSING_DISCOUNT_RATE` | Pack domain validations | the terminal capitalisation rate, stated on the contract rather than taken from the run's discount rate. |
| `E7028_OPCO_PERPETUITY_MISSING_GROWTH` | Pack domain validations | state 0 for a flat perpetuity. |
| `E7029_OPCO_PERPETUITY_INVALID_SELLING_COSTS` | Pack domain validations | a fraction between 0 and 1. |
| `E7012_OPCO_TAXES_MISSING_RATE` | Pack domain validations | a cash-taxes contract states neither `tax_rate` nor `tax_rate_curve`. `tax_rate` carries a default of 0 so a curve may stand alone; without this check, stating neither would silently model a business that pays no tax. |
| `E7013_OPCO_WC_MISSING_AMOUNT_OR_RULE` | Pack domain validations |  |
| `E7014_OPCO_WC_INVALID_SCHEDULE` | Pack domain validations |  |
| `E7020_OPCO_EXIT_MISSING_MULTIPLE` | Pack domain validations |  |
| `E7021_OPCO_EXIT_INVALID_MULTIPLE` | Pack domain validations |  |
| `E7022_OPCO_EXIT_MISSING_BASE_VALUE` | Pack domain validations |  |
| `E7023_OPCO_EXIT_INVALID_SCHEDULE` | Pack domain validations |  |
| `E7024_OPCO_EXIT_EBITDA_INVALID_MULTIPLE` | Pack domain validations |  |
| `E7030_OPCO_DEBT_INVALID_AMORT` | Pack domain validations |  |
| `E7031_OPCO_DEBT_INVALID_RATE` | Pack domain validations |  |
| `E8001_ENERGY_INVALID_DEGRADATION` | Pack domain validations |  |
| `E8002_ENERGY_INVALID_AVAILABILITY` | Pack domain validations |  |
| `E8003_ENERGY_INVALID_ESCALATION` | Pack domain validations |  |
| `E8004_ENERGY_INVALID_PRICE_ESCALATION` | Pack domain validations |  |
| `E8010_ENERGY_INVALID_MACRS_LIFE` | Pack domain validations |  |
| `E8011_ENERGY_INVALID_TAX_RATE` | Pack domain validations |  |
| `E8020_ENERGY_DEBT_INVALID_RATE` | Pack domain validations |  |
| `E8021_ENERGY_DEBT_INVALID_TERM_MONTHS` | Pack domain validations |  |
| `E8022_ENERGY_DEBT_INVALID_PRINCIPAL` | Pack domain validations |  |
| `E9001_CREDIT_INVALID_BALANCE` | Pack domain validations |  |
| `E9002_CREDIT_INVALID_RATE` | Pack domain validations |  |
| `E9003_CREDIT_INVALID_TERM_MONTHS` | Pack domain validations |  |
| `E9010_CREDIT_INVALID_CPR` | Pack domain validations |  |
| `E9011_CREDIT_INVALID_CDR` | Pack domain validations |  |
| `E9012_CREDIT_INVALID_SEVERITY` | Pack domain validations |  |
| `E9013_CREDIT_INVALID_RECOVERY_LAG` | Pack domain validations |  |
| `E9014_CREDIT_INVALID_SERVICING_FEE` | Pack domain validations |  |
| `E9015_CREDIT_INVALID_PREPAY_PENALTY` | Pack domain validations |  |
| `E9016_CREDIT_INVALID_PSA_SPEED` | Pack domain validations | `psa_speed` is a MULTIPLE of the standard prepayment curve, so 1.5 means 150% PSA. Must be 0..10; 0 selects the flat `cpr` path. |
| `E9017_CREDIT_INVALID_SDA_SPEED` | Pack domain validations | `sda_speed` is a multiple of the standard default assumption. Must be 0..10; 0 selects the flat `cdr` path. |
| `E9018_CREDIT_INVALID_ABS_SPEED` | Pack domain validations | `abs_speed` is the Absolute Prepayment Model speed: the fraction of ORIGINAL balance prepaying each month. Already monthly, so unlike `cpr`/`cdr` it is not converted. Must be 0..1. |
| `E9019_CREDIT_INVALID_AGE_MONTHS` | Pack domain validations | `age_months` is the pool's weighted average age at closing. PSA, SDA and the ABS model are all indexed from ORIGINATION, so a seasoned pool starts part-way up the ramp; leaving it at the default 0 on a seasoned pool understates prepayment. Non-negative integer. |
| `E9020_CREDIT_RATE_FLOOR_ABOVE_CAP` | Pack domain validations |  |

*156 codes.*
<!-- /cfdl:generated diagnostics-catalogue -->

## Related

- [Troubleshooting](/docs/troubleshooting) — the failures people hit most, and
  what to do about them.
- [Diagnostics specification](/docs/specification/diagnostics) — the normative
  definition: the diagnostic object, its required fields, severity semantics,
  and the deprecation policy for codes.
