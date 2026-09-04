<!-- GENERATED diagnostics -> repair catalog by tools/gen-machine-docs.py — do not edit by hand. Regenerate: make machine-docs -->

# CFDL diagnostics — repair catalog

CFDL 0.9.0. For each diagnostic code with a minimal failing
example (`fixtures/invalid/`), this catalog shows the example, the exact
diagnostics it produces (`gold/diag/`, byte-asserted in CI), and — where one
is recorded — the minimal compile-verified fix (`fixtures/repairs/`).

Diagnostics are the repair signal: read the `code`, `message`, `span`, and
`hint`, change the model, recompile. The catalog is how an agent learns what
each code looks like in the flesh before it meets one.

**Coverage:** 218 codes in the docs/08 §7 register; 105 exemplified here; 70 of 113 examples carry a recorded fix.

## active_in_unknown_state — E1332_UNKNOWN_ACTIVE_STATE

Failing example:

```cfdl
version 0.1
model "active-in-state"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 5

entity asset suite : CRE.Asset.Unit {
  rentable_area = 10000
  state leased
}

event expiry when time.t >= 2 {
  set entity asset.suite.status = "vacant"
}

event reletting when time.t >= 4 {
  set entity asset.suite.status = "leased"
}

// The state name is CHECKED against the unit lifecycle. A misspelling is a
// compile error, where `asset.suite.status == "leasd"` would just be false
// forever and say nothing.
stream cre.rent on entity asset.suite inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  category operating.revenue.base_rent
  amount = 100
  active in state leasd
}
```

- `E1332_UNKNOWN_ACTIVE_STATE` (error): Stream 'cre.rent' is active in state 'leasd', which lifecycle 'cre.unit' does not declare.
  - hint: Declared states: vacant, leased, holdover, month_to_month.

Minimal fix (compiles):

```cfdl
version 0.1
model "active-in-state"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 5

entity asset suite : CRE.Asset.Unit {
  rentable_area = 10000
  state leased
}

event expiry when time.t >= 2 {
  set entity asset.suite.status = "vacant"
}

event reletting when time.t >= 4 {
  set entity asset.suite.status = "leased"
}

// Fix: the misspelled state `leasd` is corrected to `leased`, a state the
// `cre.unit` lifecycle actually declares.
stream cre.rent on entity asset.suite inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  category operating.revenue.base_rent
  amount = 100
  active in state leased
}
```

## arrival_action_reads_current_period — E1134_SERIES_READ_IN_LOGIC

Failing example:

```cfdl
version 0.1
model "arrival-action-reads-current-period"
time calendar monthly from 2026-01 for 6

// An action evaluates in the guard's environment: settled history only. This
// is E1134's argument, one construct over.
lifecycle unit {
  initial leased
  state leased, delinquent
  leased -> delinquent when time.t >= 2 {
    set marker = series_sum("core.rent", time.t, time.t)
  }
}

entity asset suite {
  lifecycle unit
  marker init 0.0 next prev
}

stream core.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 100
}
```

- `E1134_SERIES_READ_IN_LOGIC` (error): lifecycle 'unit' edge 'leased -> delinquent' action reads `core.rent` over a window ending at `time.t`, which is this period or later. Logic settles BEFORE this period's cash exists, so only history it can already see is readable: end the window at `time.t - 1` or earlier. A stream, a waterfall and the results layer do see the current period.

Fix: not yet recorded.

## arrival_action_sets_status — E1358_ARRIVAL_ACTION_SETS_STATUS

Failing example:

```cfdl
version 0.1
model "arrival-action-sets-status"
time calendar monthly from 2026-01 for 6

// An arrival action writes FIELDS. A `status` write would fire a second
// transition inside the same period; a transition that should cause another
// transition is topology, taken next period.
lifecycle unit {
  initial leased
  state leased, delinquent
  on enter delinquent {
    set status = "leased"
  }
  leased -> delinquent when time.t >= 2
}

entity asset suite {
  lifecycle unit
  marker init 0.0 next prev
}

stream core.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 100
}
```

- `E1358_ARRIVAL_ACTION_SETS_STATUS` (error): lifecycle 'unit' entry into 'delinquent' sets `status`. An arrival action writes fields, never the state.

Fix: not yet recorded.

## arrival_action_unknown_field — E1359_ARRIVAL_ACTION_UNKNOWN_FIELD

Failing example:

```cfdl
version 0.1
model "arrival-action-unknown-field"
time calendar monthly from 2026-01 for 6

// The field name is entity-relative, so it resolves against the entity that
// transitioned. A misspelling is a write that would land nowhere.
lifecycle unit {
  initial leased
  state leased, delinquent
  on enter delinquent {
    set markr = 1.0
  }
  leased -> delinquent when time.t >= 2
}

entity asset suite {
  lifecycle unit
  marker init 0.0 next prev
}

stream core.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 100
}
```

- `E1359_ARRIVAL_ACTION_UNKNOWN_FIELD` (error): Lifecycle 'unit' entry into 'delinquent' sets 'markr', which entity 'asset.suite' does not have — declared: marker.
  - hint: An arrival action names a field on the entity that transitioned, and one machine may be bound by several entities — every one of them needs the field. Declare it on the entity, or correct the name.

Fix: not yet recorded.

## assume_reserved_keyword — E0004_EXPECTED_TOKEN

Failing example:

```cfdl
version 0.1
model "assume-reserved-keyword"
time calendar annual from 2026-01 for 2

// A RESERVED WORD LOOKS LIKE A NAME.
//
// `term` is an ordinary English word for a quantity a model might well want to
// assume, and section 18 of the language specification reserves it. Rejecting
// the declaration is right; the question is whether the author can tell why.
//
// The message used to be "Expected identifier after 'assume'", said against a
// word that reads as a perfectly good identifier, which left the reader to
// guess. It now names the word and says where the list is.
//
// The same shape reaches an ontology field named after a keyword — `docs/13`
// §7.19, where `Credit.Asset.Loan` declares a field `term` that no model can
// write.

entity asset a : Asset.Real

assume term = 5.0

stream a.s on entity asset.a inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 10.0
}
```

- `E0004_EXPECTED_TOKEN` (error): Expected identifier after 'assume', found the reserved word 'term'. Reserved words are listed in section 18 of the language specification; choose another name.

Minimal fix (compiles):

```cfdl
version 0.1
model "assume-reserved-keyword"
time calendar annual from 2026-01 for 2

// Fix: `term` is a reserved word (spec section 18), so the assumption is
// renamed to `term_years`.

entity asset a : Asset.Real

assume term_years = 5.0

stream a.s on entity asset.a inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 10.0
}
```

## bad_missing_term — E1109_MISSING_ENTITY, E2001_CONTRACT_MISSING_TERM

Failing example:

```cfdl
version 0.1
model "bad-missing-term"
time calendar monthly from 2026-01 for 12

contract test.loan_a {
  effects {}
}
```

- `E1109_MISSING_ENTITY` (error): Model must declare at least one entity.
- `E2001_CONTRACT_MISSING_TERM` (error): Contract 'test.loan_a' is missing required 'term'.

Minimal fix (compiles):

```cfdl
version 0.1
model "bad-missing-term"
time calendar monthly from 2026-01 for 12

entity asset borrower : Asset.Financial

contract test.loan_a {
  term 2026-01..2026-12
  effects {}
}
```

## bad_schedule_out_of_bounds — E2103_SCHEDULE_OUT_OF_BOUNDS

Failing example:

```cfdl
version 0.1
model "bad-schedule-out-of-bounds"
time calendar monthly from 2026-01 for 12
entity asset borrower : Asset.Financial

stream debt.principal on entity asset.borrower {
  schedule every month on day 15 from 2025-01 to 2025-12
  amount = 100
}
```

- `E2103_SCHEDULE_OUT_OF_BOUNDS` (error): Stream 'debt.principal' schedule is outside model timeline (timeline: 2026-01-01 to 2026-12-01).

Minimal fix (compiles):

```cfdl
version 0.1
model "bad-schedule-out-of-bounds"
time calendar monthly from 2026-01 for 12
entity asset borrower : Asset.Financial

// Fix: the schedule is moved inside the model timeline (2026-01..2026-12).
stream debt.principal on entity asset.borrower {
  schedule every month on day 15 from 2026-01 to 2026-12
  amount = 100
}
```

## contract_category_ambiguous — E5030_AMBIGUOUS_CONTRACT_CATEGORY

Failing example:

```cfdl
version 0.1
model "contract-category-ambiguous"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// A permanent mortgage lowers three streams — interest, principal and proceeds
// — and the pack states a category for each. One bare `category` clause cannot
// say which it reclassifies, so it would set all three to the same value and
// make every coverage ratio computed off them wrong. E5030.
entity asset tower : CRE.Asset.RealProperty

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2026-12
  category financing.debt.interest_paid
  terms {
    principal = 1000000
    interest_rate = 0.05
    amortization_months = 300
  }
}
```

- `E5030_AMBIGUOUS_CONTRACT_CATEGORY` (error): Contract 'cre.permanent_debt' states one category, and lowers 3 streams. Each carries its own category, so one clause cannot say which it reclassifies.
  - hint: Name the stream: `category <stream> = <path>`, once per stream you mean to reclassify. The bare form is for a contract that lowers exactly one.

Fix: not yet recorded.

## contract_category_bad_root — E5022_UNKNOWN_STREAM_CATEGORY

Failing example:

```cfdl
version 0.1
model "contract-category-bad-root"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 2

// A contract may override the category its lowering rule assigns, and it is
// validated like any other: rooted in one of the three activities. E5022.
entity asset tower : CRE.Asset.RealProperty

contract cre.opex_line.rooms on entity asset.tower {
  term 2026-01..2027-01
  category departmental.rooms
  terms {
    amount = 1000
  }
}
```

- `E5022_UNKNOWN_STREAM_CATEGORY` (error): Contract 'cre.opex_line.rooms' declares category 'departmental.rooms', whose root segment 'departmental' is not one of operating, investing, financing. A category is a path into the cash flow statement, so it has to say which section it belongs to.
  - hint: A contract's `category` overrides the one its lowering rule assigns, for the leaf a pack could not have enumerated. It is validated like any other — for example `category operating.expense.rooms`.

Fix: not yet recorded.

## contract_master_type — E1374_ABSTRACT_TYPE_INSTANTIATED

Failing example:

```cfdl
version 0.1
model "contract-master-type"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

// A MASTER IS REFINED, NEVER DECLARED (docs/40 §2). `Contract.Debt` binds no
// lowering rule; a model reaches it through a pack's concrete refinement,
// which the hint names.
contract Contract.Debt loan on entity asset.tower {
  term 2026-01..2027-12
  terms {
    principal = 1000000
    interest_rate = 0.05
  }
}
```

- `E1374_ABSTRACT_TYPE_INSTANTIATED` (error): Contract 'Contract.Debt.loan' declares type 'Contract.Debt', which is a master. A master is refined, never declared: a model reaches it through a pack's concrete refinement. Concrete refinements of 'Contract.Debt': cre.construction_loan, cre.permanent_debt.

Fix: not yet recorded.

## contract_missing_term — E1372_MISSING_CONTRACT_TERM, E6001_CRE_LEASE_MISSING_BASE_RENT

Failing example:

```cfdl
version 0.1
model "contract-missing-term"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

// A REQUIRED TERM THE CONTRACT OMITS IS REFUSED against the type's effective
// roster (docs/40 §8), before any rule is expanded: `rent_year` is required
// on a unit lease.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2027-12
  terms {
    escalation = 0.03
  }
}

// A GROUP OF ALTERNATIVES the contract states none of is the same refusal:
// `Contract.Lease` requires rent per period or per year, and a lease that
// states neither has no rent. The pack's own validation says the same in
// its own words (E6001); the roster check is the language's.
contract cre.lease on entity asset.tower {
  term 2026-01..2027-12
  terms {
    lease_up_months = 3
  }
}
```

- `E1372_MISSING_CONTRACT_TERM` (error): Contract 'cre.lease_unit.tenant_a' omits term 'rent_year', which type 'CRE.Contract.UnitLease' requires.
- `E1372_MISSING_CONTRACT_TERM` (error): Contract 'cre.lease' states none of rent, rent_year; type 'CRE.Contract.Lease' requires one of them.
- `E6001_CRE_LEASE_MISSING_BASE_RENT` (error): CRE lease must state a rent: 'rent' (per period) or 'rent_year' (annual).

Fix: not yet recorded.

## contract_unknown_clause — E0004_EXPECTED_TOKEN

Failing example:

```cfdl
version 0.1
model "contract-unknown-clause"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 2

// A misspelled clause in a contract body used to be swallowed by a catch-all
// and vanish without a diagnostic. E0004.
entity asset tower : CRE.Asset.RealProperty

contract cre.opex_line.rooms on entity asset.tower {
  term 2026-01..2027-01
  categry operating.expense.rooms
  terms {
    amount = 1000
  }
}
```

- `E0004_EXPECTED_TOKEN` (error): Unexpected 'categry' in a contract body. Expected 'term', 'payment net', 'terms', 'category', 'parties', 'on entity', 'effects', or '}'.

Fix: not yet recorded.

## contract_unknown_term — E1371_UNKNOWN_CONTRACT_TERM

Failing example:

```cfdl
version 0.1
model "contract-unknown-term"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

// A TERM THE TYPE DOES NOT DECLARE IS REFUSED (docs/40 §8). Before this it
// was quietly ignored: `esclation` matched no rule placeholder, the lease
// never escalated, and nothing said so. The effective roster is the pack
// type's own terms plus its master's, so the hint names the near miss.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2027-12
  terms {
    rent_year = 480000
    esclation = 0.03
  }
}
```

- `E1371_UNKNOWN_CONTRACT_TERM` (error): Contract 'cre.lease_unit.tenant_a' states term 'esclation', which type 'CRE.Contract.UnitLease' does not declare. The term would never be read.
  - hint: Did you mean escalation?

Fix: not yet recorded.

## contract_unknown_type — E1373_UNKNOWN_CONTRACT_TYPE

Failing example:

```cfdl
version 0.1
model "contract-unknown-type"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

// The two-token form STATES the type, so a type the pack does not declare is
// refused where it is written, with the near miss (docs/40 §8; docs/13 §7.63).
contract cre.leas_unit tenant_a on entity asset.tower {
  term 2026-01..2027-12
  terms {
    rent_year = 480000
  }
}

// The fused form gets the same answer: a contract no rule lowers is a type
// the pack does not declare, not a contract missing its effects.
contract cre.leas_unit.tenant_b on entity asset.tower {
  term 2026-01..2027-12
  terms {
    rent_year = 360000
  }
}
```

- `E1373_UNKNOWN_CONTRACT_TYPE` (error): Contract 'cre.leas_unit.tenant_a' declares type 'cre.leas_unit', which the active ontology does not define, so no rule lowers it. Did you mean cre.lease_unit?
- `E1373_UNKNOWN_CONTRACT_TYPE` (error): Contract 'cre.leas_unit.tenant_b' declares type 'cre.leas_unit', which the active ontology does not define, so no rule lowers it. Did you mean cre.lease_unit?

Fix: not yet recorded.

## cre_exit_bad_cap — E6011_CRE_EXIT_INVALID_EXIT_CAP

Failing example:

```cfdl
version 0.1
model "cre-exit-bad-cap"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset property : CRE.Asset.RealProperty

contract cre.exit_cap {
  term 2031-12..2031-12
  terms {
    cap_rate = 0
    income = 180000
  }
}
```

- `E6011_CRE_EXIT_INVALID_EXIT_CAP` (error): CRE exit 'cap_rate' must be greater than 0.

Minimal fix (compiles):

```cfdl
version 0.1
model "cre-exit-bad-cap"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 72

entity asset property : CRE.Asset.RealProperty

// Fix: `exit_cap` must be greater than 0; a positive cap rate is stated.
contract cre.exit_cap {
  term 2031-12..2031-12
  terms {
    cap_rate = 0.06
    income = 180000
  }
}
```

## cre_lease_missing_base_rent — E1371_UNKNOWN_CONTRACT_TERM, E1372_MISSING_CONTRACT_TERM, E6001_CRE_LEASE_MISSING_BASE_RENT

Failing example:

```cfdl
version 0.1
model "cre-lease-missing-base-rent"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset property : CRE.Asset.RealProperty

contract cre.lease {
  term 2026-07..2027-12
  terms {
    growth = 0.02
  }
}
```

- `E1372_MISSING_CONTRACT_TERM` (error): Contract 'cre.lease' states none of rent, rent_year; type 'CRE.Contract.Lease' requires one of them.
- `E6001_CRE_LEASE_MISSING_BASE_RENT` (error): CRE lease must state a rent: 'rent' (per period) or 'rent_year' (annual).
- `E1371_UNKNOWN_CONTRACT_TERM` (error): Contract 'cre.lease' states term 'growth', which type 'CRE.Contract.Lease' does not declare. The term would never be read.
  - hint: Terms of 'CRE.Contract.Lease': rent, rent_year, escalation, free_rent_months, lease_up_months.

Minimal fix (compiles):

```cfdl
version 0.1
model "cre-lease-missing-base-rent"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset property : CRE.Asset.RealProperty

// Fix: a CRE lease must state a rent; `rent_year` is added.
contract cre.lease {
  term 2026-07..2027-12
  terms {
    rent_year = 120000
  }
}
```

## cre_pct_rent_expected_no_quantile — E1371_UNKNOWN_CONTRACT_TERM, E1372_MISSING_CONTRACT_TERM, E6066_CRE_PCT_RENT_MISSING_SALES_QUANTILE

Failing example:

```cfdl
version 0.1
model "cre-pct-rent-expected-no-quantile"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 2

entity asset store : CRE.Asset.RealProperty

// AN EXPECTATION NEEDS SOMETHING TO TAKE AN EXPECTATION OVER.
//
// This states a point estimate of sales, which is what the ORIGINAL contract
// takes. Accepting it here would compute an expectation over a distribution
// that does not exist, and the natural fallback — treat the point estimate as
// certain — is not a smaller version of this contract, it IS the other one,
// silently. So it is refused with the name of the contract that fits.
//
// Expect E6066_CRE_PCT_RENT_MISSING_SALES_QUANTILE.
contract cre.percentage_rent_expected.no_dist on entity asset.store {
  term 2026-01..2027-01
  terms {
    sales_year      = 1000000
    breakpoint_year = 1200000
    overage_pct     = 0.06
  }
}
```

- `E1372_MISSING_CONTRACT_TERM` (error): Contract 'cre.percentage_rent_expected.no_dist' omits term 'sales_quantile', which type 'CRE.Contract.PercentageRentExpected' requires.
- `E6066_CRE_PCT_RENT_MISSING_SALES_QUANTILE` (error): CRE expected percentage rent must name a sales distribution in 'sales_quantile'. Without one there is no distribution to take an expectation over — use 'cre.percentage_rent' for a lease underwritten on a single sales figure.
- `E1371_UNKNOWN_CONTRACT_TERM` (error): Contract 'cre.percentage_rent_expected.no_dist' states term 'sales_year', which type 'CRE.Contract.PercentageRentExpected' does not declare. The term would never be read.
  - hint: Terms of 'CRE.Contract.PercentageRentExpected': rent, rent_year, escalation, free_rent_months, lease_up_months, sales_quantile, sales_growth, breakpoint_year, overage_pct.

Minimal fix (compiles):

```cfdl
version 0.1
model "cre-pct-rent-expected-no-quantile"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 2

entity asset store : CRE.Asset.RealProperty

// Fix: an expected percentage rent needs a distribution to take an
// expectation over — a quantile curve is declared and named in
// `sales_quantile` in place of the point estimate `sales_year`.
quantile store_sales linear {
  0.00:  400000.0
  0.50: 1000000.0
  1.00: 1800000.0
}

contract cre.percentage_rent_expected.no_dist on entity asset.store {
  term 2026-01..2027-01
  terms {
    sales_quantile  = store_sales
    breakpoint_year = 1200000
    overage_pct     = 0.06
  }
}
```

## cre_unit_invalid_bounds — E6031_CRE_UNIT_INVALID_FREE_RENT, E6032_CRE_UNIT_INVALID_PRO_RATA, E6040_CRE_ROLLOVER_INVALID_PROBABILITY, E6041_CRE_ROLLOVER_INVALID_DOWNTIME

Failing example:

```cfdl
version 0.1
model "cre-unit-invalid-bounds"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 36

entity asset tower : CRE.Asset.RealProperty

// Per-tenant contracts carry a suffix. Before instance matching, suffixed
// contracts escaped domain validation entirely.
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2028-12
  terms {
    rent_year = 480000
    free_rent_months = -3
    pro_rata_share = 1.8
  }
}

contract cre.rollover.tenant_a on entity asset.tower {
  term 2029-01..2029-12
  terms {
    renewal_probability = 1.4
    renewal_rent_year = 520000
    market_rent_year = 560000
    downtime_months = -2
  }
}
```

- `E6031_CRE_UNIT_INVALID_FREE_RENT` (error): CRE lease unit 'free_rent_months' must be a whole number of months, 0 or more.
- `E6032_CRE_UNIT_INVALID_PRO_RATA` (error): CRE lease unit 'pro_rata_share' must be a fraction between 0 and 1.
- `E6040_CRE_ROLLOVER_INVALID_PROBABILITY` (error): CRE rollover 'renewal_probability' must be a probability between 0 and 1.
- `E6041_CRE_ROLLOVER_INVALID_DOWNTIME` (error): CRE rollover 'downtime_months' must be a whole number of months, 0 or more.

Minimal fix (compiles):

```cfdl
version 0.1
model "cre-unit-invalid-bounds"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 48

entity asset tower : CRE.Asset.RealProperty

// Fix: free_rent_months and downtime_months are now whole non-negative month
// counts, and pro_rata_share / renewal_probability are fractions in [0, 1].
contract cre.lease_unit.tenant_a on entity asset.tower {
  term 2026-01..2028-12
  terms {
    rent_year = 480000
    free_rent_months = 3
    pro_rata_share = 0.8
  }
}

contract cre.rollover.tenant_a on entity asset.tower {
  term 2029-01..2029-12
  terms {
    renewal_probability = 0.7
    renewal_rent_year = 520000
    market_rent_year = 560000
    downtime_months = 2
  }
}
```

## credit_invalid_rates — E9010_CREDIT_INVALID_CPR, E9011_CREDIT_INVALID_CDR, E9012_CREDIT_INVALID_SEVERITY, E9013_CREDIT_INVALID_RECOVERY_LAG

Failing example:

```cfdl
version 0.1
model "credit-invalid-rates"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset buyer : Credit.Asset.LoanPool

// A 500% CPR and a severity above 1 are impossible; the closed-form pool
// factor would happily compute a nonsense balance path from them.
contract credit.pool_level_pay.auto_a on entity asset.buyer {
  term 2026-01..2027-12
  terms {
    principal = 25000000
    interest_rate = 0.065
    term_months = 24
    cpr = 5.0
    cdr = -0.1
    severity = 1.5
    recovery_lag_months = -3
  }
}
```

- `E9010_CREDIT_INVALID_CPR` (error): Credit 'cpr' must be an annual rate between 0 and 1.
- `E9011_CREDIT_INVALID_CDR` (error): Credit 'cdr' must be an annual rate between 0 and 1.
- `E9012_CREDIT_INVALID_SEVERITY` (error): Credit 'severity' must be a fraction between 0 and 1.
- `E9013_CREDIT_INVALID_RECOVERY_LAG` (error): Credit 'recovery_lag_months' must be a whole number of months, 0 or more.

Minimal fix (compiles):

```cfdl
version 0.1
model "credit-invalid-rates"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset buyer : Credit.Asset.LoanPool

// Fix: cpr/cdr are annual rates in [0, 1], severity is a fraction in [0, 1],
// and recovery_lag_months is a whole non-negative month count.
contract credit.pool_level_pay.auto_a on entity asset.buyer {
  term 2026-01..2027-12
  terms {
    principal = 25000000
    interest_rate = 0.065
    term_months = 24
    cpr = 0.15
    cdr = 0.02
    severity = 0.45
    recovery_lag_months = 3
  }
}
```

## currency_mismatch — E2107_STREAM_CURRENCY_MISMATCH

Failing example:

```cfdl
version 0.1
model "currency-mismatch" currency INR
time calendar monthly from 2026-01 for 6

entity asset plant : Asset.Real

stream plant.revenue on entity asset.plant inflow currency INR {
  schedule every month from 2026-01 to 2026-06
  amount = 100000
}

// A USD stream in an INR model would be subtracted as if it were INR.
stream plant.fee on entity asset.plant outflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 500
}
```

- `E2107_STREAM_CURRENCY_MISMATCH` (error): Stream 'plant.fee' is in USD but the model reports in INR. Convert explicitly in the amount expression, or declare `model "..." currency USD`.

Minimal fix (compiles):

```cfdl
version 0.1
model "currency-mismatch" currency INR
time calendar monthly from 2026-01 for 6

entity asset plant : Asset.Real

stream plant.revenue on entity asset.plant inflow currency INR {
  schedule every month from 2026-01 to 2026-06
  amount = 100000
}

// Fix: the fee stream is declared in INR, matching the model's reporting
// currency (its amount now states the INR value directly).
stream plant.fee on entity asset.plant outflow currency INR {
  schedule every month from 2026-01 to 2026-06
  amount = 500
}
```

## curve_duplicate — E5008_INVALID_CURVE

Failing example:

```cfdl
version 0.1
model "curve-duplicate"
time calendar monthly from 2026-01 for 12

curve sofr {
  2026-01: 0.045
}

curve sofr {
  2026-01: 0.042
}

entity asset buyer : Asset.Financial

stream test.interest on entity asset.buyer inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = curve_value("sofr", time.date)
}
```

- `E5008_INVALID_CURVE` (error): Curve 'sofr' is declared more than once.

Minimal fix (compiles):

```cfdl
version 0.1
model "curve-duplicate"
time calendar monthly from 2026-01 for 12

// Fix: the curve 'sofr' is declared exactly once.
curve sofr {
  2026-01: 0.045
}

entity asset buyer : Asset.Financial

stream test.interest on entity asset.buyer inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = curve_value("sofr", time.date)
}
```

## dup_stream — E1003_DUPLICATE_STREAM

Failing example:

```cfdl
version 0.1
model "dup-stream"
time calendar monthly from 2026-01 for 12
entity asset borrower : Asset.Financial
stream debt.principal on entity asset.borrower
stream debt.principal on entity asset.borrower
```

- `E1003_DUPLICATE_STREAM` (error): Duplicate stream 'debt.principal'.

Minimal fix (compiles):

```cfdl
version 0.1
model "dup-stream"
time calendar monthly from 2026-01 for 12
entity asset borrower : Asset.Financial

stream debt.principal on entity asset.borrower {
  schedule every month from 2026-01 to 2026-12
  amount = 1000
}
```

## duplicate_contract — E1002_DUPLICATE_CONTRACT

Failing example:

```cfdl
version 0.1
model "duplicate-contract"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// TWO CONTRACTS UNDER ONE NAME.
//
// Both lower, and two rules striking the same stream name was reported as
// E5007_DUPLICATE_LOWERED_STREAM — the downstream symptom, and only when a
// pack is active and only if the generated names happen to collide. The cause
// is the pair of declarations below.

entity asset tower : CRE.Asset.RealProperty

contract cre.opex_line on entity asset.tower {
  term 2026-01..2026-12
  terms { amount = 100 }
}
contract cre.opex_line on entity asset.tower {
  term 2026-01..2026-12
  terms { amount = 200 }
}
```

- `E1002_DUPLICATE_CONTRACT` (error): Duplicate contract 'cre.opex_line'. Give one a suffix to keep them separable.

Fix: not yet recorded.

## duplicate_entity_id — E1360_DUPLICATE_ENTITY_ID

Failing example:

```cfdl
version 0.1
model "duplicate-entity-id"
time calendar annual from 2026-01 for 2

// An id names ONE thing for the layer above the model. Two entities
// claiming the same one would merge under any consumer that joins on it —
// so the duplicate is refused, the one check the language can make about a
// value it must not interpret.

entity asset alpha : Asset.Financial { id = "evs:asset/91" }
entity asset beta  : Asset.Financial { id = "evs:asset/91" }

stream alpha.income on entity asset.alpha inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 100
}
```

- `E1360_DUPLICATE_ENTITY_ID` (error): Entity 'asset.beta' declares id "evs:asset/91", which 'asset.alpha' already carries.
  - hint: An id names one thing for the layer above the model; a consumer joining on it would merge the two entities into one.

Fix: not yet recorded.

## duplicate_event — E1007_DUPLICATE_EVENT

Failing example:

```cfdl
version 0.1
model "duplicate-event"
time calendar annual from 2026-01 for 3

// TWO EVENTS UNDER ONE NAME BOTH FIRE.
//
// The resolver's symbol table declared an `events` map that nothing ever
// wrote, so the duplicate check could not run. Both events reached the IR and
// both fired, and the journal then carried two `event:refi` actors that cannot
// be told apart.

entity asset co : Asset.Financial

event refi when time.t >= 1 { set entity asset.co.status = "a" }
event refi when time.t >= 2 { set entity asset.co.status = "b" }
```

- `E1007_DUPLICATE_EVENT` (error): Duplicate event 'refi'.

Fix: not yet recorded.

## duplicate_metric — E1008_DUPLICATE_METRIC

Failing example:

```cfdl
version 0.1
model "duplicate-metric"
time calendar annual from 2026-01 for 3

// TWO METRICS UNDER ONE NAME.
//
// Both would publish as `metric.headline` and one would win silently.

entity asset co : Asset.Financial

metric headline = 1.0
metric headline = 2.0
```

- `E1008_DUPLICATE_METRIC` (error): Duplicate metric 'headline'.

Fix: not yet recorded.

## duplicate_option — E1006_DUPLICATE_OPTION

Failing example:

```cfdl
version 0.1
model "duplicate-option"
time calendar annual from 2026-01 for 3

// TWO OPTIONS UNDER ONE NAME MAKE `exercise option` AMBIGUOUS.
//
// Both reached the IR, and the engine resolves a forced exercise by position —
// so which one an event exercised was an accident of declaration order.

entity asset co : Asset.Financial

option renewal type Option.Call { exercise when false payoff 1 }
option renewal type Option.Call { exercise when false payoff 2 }
```

- `E1006_DUPLICATE_OPTION` (error): Duplicate option 'renewal'.

Fix: not yet recorded.

## duplicate_phase — E1004_DUPLICATE_PHASE

Failing example:

```cfdl
version 0.1
model "duplicate-phase"
time calendar annual from 2026-01 for 4

// TWO PHASES UNDER ONE NAME.
//
// A schedule naming the phase gets one of the two windows, decided by
// declaration order rather than by the model.

phase build from 2026-01 to 2026-12
phase build from 2027-01 to 2027-12

entity asset co : Asset.Financial
```

- `E1004_DUPLICATE_PHASE` (error): Duplicate phase 'build'.

Fix: not yet recorded.

## duplicate_slice — E1361_DUPLICATE_SLICE

Failing example:

```cfdl
version 0.1
model "duplicate-slice"
time calendar annual from 2026-01 for 2

entity asset a : Asset.Financial

stream a.x on entity asset.a inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 1
}

slice s { entity asset.a }
slice s { stream "a.*" }
```

- `E1361_DUPLICATE_SLICE` (error): Duplicate slice 's'.

Fix: not yet recorded.

## energy_invalid_physics — E8001_ENERGY_INVALID_DEGRADATION, E8002_ENERGY_INVALID_AVAILABILITY, E8010_ENERGY_INVALID_MACRS_LIFE, E8011_ENERGY_INVALID_TAX_RATE

Failing example:

```cfdl
version 0.1
model "energy-invalid-physics"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset solar : Energy.Asset.GenerationFacility

// availability above 1.0 and negative degradation are physically impossible;
// an unchecked pack would silently produce an inflated revenue curve.
contract energy.ppa on entity asset.solar {
  term 2026-01..2027-12
  terms {
    quantity = 4200
    price = 85
    availability = 1.4
    degradation = -0.05
  }
}

// MACRS class life must select a real IRS table.
contract energy.macrs_shield on entity asset.solar {
  term 2026-01..2027-12
  terms {
    basis = 2400000
    tax_rate = 1.8
    life = 9
  }
}
```

- `E8002_ENERGY_INVALID_AVAILABILITY` (error): Energy 'availability' must be a fraction between 0 and 1.
- `E8001_ENERGY_INVALID_DEGRADATION` (error): Energy 'degradation' must be a fraction between 0 and 1.
- `E8011_ENERGY_INVALID_TAX_RATE` (error): Energy 'tax_rate' must be a fraction between 0 and 1.
- `E8010_ENERGY_INVALID_MACRS_LIFE` (error): Energy MACRS 'life' must be one of 5, 7, 15, or 20.

Minimal fix (compiles):

```cfdl
version 0.1
model "energy-invalid-physics"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset solar : Energy.Asset.GenerationFacility

// Fix: availability and degradation are fractions in [0, 1].
contract energy.ppa on entity asset.solar {
  term 2026-01..2027-12
  terms {
    quantity = 4200
    price = 85
    availability = 0.97
    degradation = 0.005
  }
}

// Fix: tax_rate is a fraction in [0, 1] and MACRS life is a real IRS class (5).
contract energy.macrs_shield on entity asset.solar {
  term 2026-01..2027-12
  terms {
    basis = 2400000
    tax_rate = 0.21
    life = 5
  }
}
```

## entity_hierarchy_cycle — E1318_ENTITY_HIERARCHY_CYCLE

Failing example:

```cfdl
version 0.1
model "hierarchy-cycle"
time calendar annual from 2026-01 for 2
entity asset a : Asset.Real { part of asset.b }
entity asset b : Asset.Real { part of asset.c }
entity asset c : Asset.Real { part of asset.a }
stream x.y on entity asset.a inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 1
}
```

- `E1318_ENTITY_HIERARCHY_CYCLE` (error): Entity hierarchy forms a cycle: asset.a -> asset.b -> asset.c -> asset.a.
  - hint: An entity aggregates its children, so a cycle has no bottom to sum from.

Minimal fix (compiles):

```cfdl
version 0.1
model "hierarchy-cycle"
time calendar annual from 2026-01 for 2
// Fix: the cycle is broken — asset.c is no longer part of asset.a.
entity asset a : Asset.Real { part of asset.b }
entity asset b : Asset.Real { part of asset.c }
entity asset c : Asset.Real
stream x.y on entity asset.a inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 1
}
```

## entity_unknown_type — E1311_UNKNOWN_ENTITY_TYPE

Failing example:

```cfdl
version 0.1
model "entity-unknown-type"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 1

// Every failure here was silently accepted before entities had types: the
// type, the field and the state are all misspelled, and the model still ran.
entity asset tower : CRE.Asset.RealProprty {
  asset_clas = "office"
  state stabilized
}

stream cre.rent on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2026-01
  category operating.revenue.base_rent
  amount = 1
}
```

- `E1311_UNKNOWN_ENTITY_TYPE` (error): Entity 'asset.tower' declares type 'CRE.Asset.RealProprty', which the active ontology does not define.
  - hint: Known types: Asset.Financial, Asset.Intangible, Asset.Real, CRE.Asset.EquityInterest, CRE.Asset.RealProperty, CRE.Asset.Unit, CRE.Container.Portfolio, CRE.Party.Investor, CRE.Party.Lender, CRE.Party.PropertyManager, CRE.Party.Sponsor, CRE.Party.Tenant, Container.Fund, Container.Portfolio, Container.SPV, Container.Transaction, Party.

Minimal fix (compiles):

```cfdl
version 0.1
model "entity-unknown-type"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 1

// Fix: the type, field, and state are spelled as the ontology declares them —
// CRE.Asset.RealProperty, asset_class, and lifecycle state `operating`.
entity asset tower : CRE.Asset.RealProperty {
  asset_class = "office"
  state stabilized
}

stream cre.rent on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2026-01
  category operating.revenue.base_rent
  amount = 1
}
```

## event_action_unknown_targets — E1301_UNRESOLVED_ENTITY_REF

Failing example:

```cfdl
version 0.1
model "event-action-unknown-targets"
time calendar annual from 2026-01 for 3

// Every target an event names was unresolved before: a misspelling matched
// nothing and the action was silently inert — the stream it was meant to stop
// kept paying, with no diagnostic at any stage and no warning at run time.
//
// Two of the three typos below are reported here. The entity is resolved in the
// resolver and fails the model at that stage, which is why the STREAM and the
// OPTION are not also listed: both are checked in the compiler, after lowering,
// where the streams a contract produced and the lowered options are known, and
// the compiler does not run once resolution has failed.
// `fixtures/invalid/event_stream_typo` pins the stream one on its own.

entity asset co : Asset.Financial

stream loan.debt_service on entity asset.co outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 100
}

option refi_fee type Option.Call {
  exercise when false
  payoff 10
}

event refi when time.t >= 1 {
  set entity asset.ghost.status = "refinanced"
  deactivate stream loan.dbt_service
  exercise option refi_fe
}
```

- `E1301_UNRESOLVED_ENTITY_REF` (error): Event 'refi' references unknown entity 'asset.ghost'.

Minimal fix (compiles):

```cfdl
version 0.1
model "event-action-unknown-targets"
time calendar annual from 2026-01 for 3

// Fix: every event target now names a declared thing — entity asset.co,
// stream loan.debt_service, and option refi_fee.

entity asset co : Asset.Financial

stream loan.debt_service on entity asset.co outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 100
}

option refi_fee type Option.Call {
  exercise when false
  payoff 10
}

event refi when time.t >= 1 {
  set entity asset.co.status = "refinanced"
  deactivate stream loan.debt_service
  exercise option refi_fee
}
```

## event_stream_typo — E1302_UNRESOLVED_STREAM_REF

Failing example:

```cfdl
version 0.1
model "event-stream-typo"
time calendar annual from 2026-01 for 3

// A MISSPELLED STREAM TARGET STILL MATCHES NOTHING.
//
// The check moved: a stream reference in an event is resolved after contract
// lowering, where the streams a contract produces are known as well as the
// ones the model declared. What it does for a typo is unchanged — the action
// would otherwise be silently inert, and the stream it was meant to stop would
// keep paying with no diagnostic and no warning at run time.
//
// The hint lists every stream in the model, so the near miss is visible.

entity asset co : Asset.Financial

stream loan.debt_service on entity asset.co outflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 100
}

event refi when time.t >= 1 {
  deactivate stream loan.dbt_service
}
```

- `E1302_UNRESOLVED_STREAM_REF` (error): Event 'refi' references unknown stream 'loan.dbt_service'.
  - hint: Streams in this model, declared and contract-lowered: loan.debt_service.

Fix: not yet recorded.

## event_when_not_bool — E2201_EVENT_WHEN_NOT_BOOL

Failing example:

```cfdl
version 0.1
model "event-when-not-bool"
time calendar annual from 2026-01 for 3

// A GUARD THAT IS NOT A CONDITION.
//
// The engine took a non-boolean guard as `false` and carried on: the event
// never fired, `deterministic.warnings` said so per period, and `status`
// stayed `ok`. That is the shape §7.71 already refused for series reads.
//
// The guard here is a literal, so it decides the same way on every period and
// every run — which is what makes it checkable before the model is run at all.

entity asset co : Asset.Financial

event refi when 42 { set entity asset.co.status = "refinanced" }
```

- `E2201_EVENT_WHEN_NOT_BOOL` (error): Event 'refi' fires `when 42`, which is not a condition.
  - hint: A guard must be true or false. The engine would take a non-boolean as `false`, so the event would never fire.

Fix: not yet recorded.

## expr_illegal_op — E3004_EXPR_ILLEGAL_OP

Failing example:

```cfdl
version 0.1
model "expr-illegal-op"
time calendar annual from 2026-01 for 3

// AN OPERATOR APPLIED TO THE WRONG KIND OF OPERAND.
//
// `and` joins conditions, not numbers. Same outcome as a type error before
// this was checked: 0, a warning, and `status: ok`.

entity asset co : Asset.Financial

stream a.rent on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 10 and 3
}
```

- `E3004_EXPR_ILLEGAL_OP` (error): Stream 'a.rent' amount cannot evaluate: expected bool, got number.
  - hint: Every value in this expression is a literal, so it evaluates the same way on every period and every run.

Fix: not yet recorded.

## expr_parse_error — E3001_EXPR_PARSE_ERROR

Failing example:

```cfdl
version 0.1
model "expr-parse-error"
time calendar monthly from 2026-01 for 3
entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower inflow currency USD {
  schedule every month from 2026-01 to 2026-03
  amount = 1200 * (1 +
}
```

- `E3001_EXPR_PARSE_ERROR` (error): unexpected end of expression

Minimal fix (compiles):

```cfdl
version 0.1
model "expr-parse-error"
time calendar monthly from 2026-01 for 3
entity asset borrower : Asset.Financial

// Fix: the truncated amount expression is completed.
stream lease.rent on entity asset.borrower inflow currency USD {
  schedule every month from 2026-01 to 2026-03
  amount = 1200 * (1 + 0.02)
}
```

## expr_type_error — E3003_EXPR_TYPE_ERROR

Failing example:

```cfdl
version 0.1
model "expr-type-error"
time calendar annual from 2026-01 for 3

// OPERANDS THE OPERATOR CANNOT COMBINE.
//
// The amount evaluated to 0 with a warning, so the stream paid nothing and the
// run reported `status: ok`. Every value here is a literal: the failure is the
// model's, on every period and every run.

entity asset co : Asset.Financial

stream a.rent on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = "100" + 1
}
```

- `E3003_EXPR_TYPE_ERROR` (error): Stream 'a.rent' amount cannot evaluate: cannot apply Add to text and number.
  - hint: Every value in this expression is a literal, so it evaluates the same way on every period and every run.

Fix: not yet recorded.

## field_declared_twice — E1128_FIELD_DECLARED_TWICE

Failing example:

```cfdl
version 0.1
model "field-declared-twice"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 3

// ONE NAME, ONE MEANING. A field stated with '=' and a field with a rule bind
// the same path, so declaring both leaves one of them silently winning.

entity asset tranche : Credit.Asset.Tranche {
  seniority = 1
  original_balance = 275.0
  original_balance init 275.0 next prev - 25.0
}
```

- `E1128_FIELD_DECLARED_TWICE` (error): Field 'asset.tranche.original_balance' is declared twice — once with '=' and once with a rule. A field is one value; state it as a fact or give it a rule, not both.

Minimal fix (compiles):

```cfdl
version 0.1
model "field-declared-twice"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 3

// Fix: 'original_balance' is declared once — the rule form is kept and the
// duplicate '=' declaration removed.

entity asset tranche : Credit.Asset.Tranche {
  seniority = 1
  original_balance init 275.0 next prev - 25.0
}
```

## field_rule_reads_field — E1127_FIELD_RULE_READS_FIELD

Failing example:

```cfdl
version 0.1
model "field-rule-reads-field"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 3

// A RULE MAY NOT READ A FIELD BY ITS BARE NAME.
//
// `asset.senior.balance` means that field's value at period CLOSE. Inside a
// rule, close has not happened yet — so rather than quietly meaning the period
// before, it is rejected and the spelling that says so is named.
//
// Silence would be the worst answer. A family path resolves through the
// open-world entity root, so an unrejected read returns null and evaluates to
// zero: a wrong number with nothing to see.

entity asset senior : Credit.Asset.Tranche {
  seniority = 1
  balance init 100.0 next max(0.0, prev - 10.0)
}

entity asset junior : Credit.Asset.Tranche {
  seniority = 2
  balance init 50.0 next prev - asset.senior.balance * 0.1
}
```

- `E1127_FIELD_RULE_READS_FIELD` (error): Field 'asset.junior.balance' reads another entity's field in 'next'. A field names this period's value at close, which does not exist yet inside a rule. Write `prev <entity>.<field>` for the previous period, or read it from a stream or waterfall, which see period-close values.

Minimal fix (compiles):

```cfdl
version 0.1
model "field-rule-reads-field"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 3

// Fix: the junior rule reads the senior balance as `prev.asset.senior.balance`
// — the previous period's value, which exists inside a rule.

entity asset senior : Credit.Asset.Tranche {
  seniority = 1
  balance init 100.0 next max(0.0, prev - 10.0)
}

entity asset junior : Credit.Asset.Tranche {
  seniority = 2
  balance init 50.0 next prev - prev.asset.senior.balance * 0.1
}
```

## import_cycle — E1201_IMPORT_CYCLE

Failing example — a.cfdl:

```cfdl
import "b.cfdl"
```

Failing example — b.cfdl:

```cfdl
import "a.cfdl"
```

Failing example:

```cfdl
version 0.1
model "import_cycle"
time calendar monthly from 2026-01 for 1
import "a.cfdl"
```

- `E1201_IMPORT_CYCLE` (error): Import cycle detected: 'b.cfdl' depends on 'a.cfdl' through a cycle.

Minimal fix (compiles) — a.cfdl:

```cfdl
// Fix: a.cfdl still imports b.cfdl, but b.cfdl no longer imports a.cfdl,
// breaking the cycle.
import "b.cfdl"
```

Minimal fix (compiles) — b.cfdl:

```cfdl
// Fix: the import of a.cfdl is removed to break the cycle.
entity asset a : Asset.Real
```

Minimal fix (compiles):

```cfdl
version 0.1
model "import_cycle"
time calendar monthly from 2026-01 for 1
import "a.cfdl"
```

## import_not_found — E1202_IMPORT_NOT_FOUND

Failing example:

```cfdl
version 0.1
model "import_not_found"
time calendar monthly from 2026-01 for 1
import "missing.cfdl"
```

- `E1202_IMPORT_NOT_FOUND` (error): Imported module 'missing.cfdl' was not found.

Minimal fix (compiles) — missing.cfdl:

```cfdl
// Fix: this file is the module the model imports; it exists now.
entity asset a : Asset.Real
```

Minimal fix (compiles):

```cfdl
version 0.1
model "import_not_found"
time calendar monthly from 2026-01 for 1
// Fix: the imported module now exists alongside the model.
import "missing.cfdl"
```

## import_outside_root — E1203_IMPORT_OUTSIDE_MODEL_ROOT

Failing example:

```cfdl
version 0.1
model "import_outside_root"
time calendar monthly from 2026-01 for 1
import "../../outside_shared/outside.cfdl"
```

- `E1203_IMPORT_OUTSIDE_MODEL_ROOT` (error): Import path '../../outside_shared/outside.cfdl' escapes the model root.

Minimal fix (compiles):

```cfdl
version 0.1
model "import_outside_root"
time calendar monthly from 2026-01 for 1
// Fix: the shared module is imported from inside the model root instead of
// escaping it with '../..'.
import "outside.cfdl"
```

Minimal fix (compiles) — outside.cfdl:

```cfdl
// Fix: the previously-external module now lives inside the model root.
entity asset a : Asset.Real
```

## inherited_field_near_miss — E1313_UNKNOWN_ENTITY_FIELD

Failing example:

```cfdl
version 0.1
model "inherited-field-near-miss"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 2

// `yearbuilt` is a near miss of `year_built` — a field the Unit type does
// not declare itself but INHERITS from CRE.Asset.RealProperty. Before
// inheritance the typo sailed through as "the modeller's own field"; a
// value nobody reads is the quiet kind of wrong this project keeps closing.

entity asset suite : CRE.Asset.Unit {
  rentable_area = 1200.0
  yearbuilt = 1987.0
}

stream unit.rent on entity asset.suite inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  category operating.revenue.other
  amount = 100
}
```

- `E1313_UNKNOWN_ENTITY_FIELD` (error): Entity 'asset.suite' of type 'CRE.Asset.Unit' sets 'yearbuilt', which the type does not declare.
  - hint: Declared fields: asset_class, rentable_area, use, year_built.

Fix: not yet recorded.

## invalid_date_literal — E0005_INVALID_DATE_LITERAL

Failing example:

```cfdl
version 0.1
model "invalid-date-literal"
time calendar annual from 2026-13 for 2

// SHAPE IS NOT VALIDITY.
//
// The lexer accepted four digits, a dash and two more, which `2026-13`
// satisfies, and nothing checked the calendar afterwards. The model compiled:
// the IR carried `"start": "2026-13-01"` and only the RUN refused it, one
// stage and one artifact too late. February is checked the same way —
// 2025-02-29 is refused and 2024-02-29 is not.

entity asset co : Asset.Financial
```

- `E0005_INVALID_DATE_LITERAL` (error): '2026-13' is not a real calendar date. Dates are `YYYY-MM` or `YYYY-MM-DD`.

Fix: not yet recorded.

## lex_unterminated_block_comment — E0003_UNTERMINATED_BLOCK_COMMENT

Failing example:

```cfdl
version "0.1"
/* unterminated block comment
model "lex_block"
```

- `E0003_UNTERMINATED_BLOCK_COMMENT` (error): Unterminated block comment.

Minimal fix (compiles):

```cfdl
version 0.1
/* Fix: the block comment is terminated. */
model "lex_block"
time calendar monthly from 2026-01 for 1
entity asset a : Asset.Real
```

## lex_unterminated_string — E0002_UNTERMINATED_STRING

Failing example:

```cfdl
version "0.1"
model "lex_unterminated
time calendar monthly from 2026-01 for 1
```

- `E0002_UNTERMINATED_STRING` (error): Unterminated string literal.

Minimal fix (compiles):

```cfdl
version 0.1
// FIX: close the model-name string literal (version is a bare number, and the
// model gains an entity so it compiles past the lexer).
model "lex_unterminated"
time calendar monthly from 2026-01 for 1

entity asset borrower : Asset.Financial
```

## lifecycle_augment_topology — E1357_LIFECYCLE_AUGMENT_TOPOLOGY

Failing example:

```cfdl
version 0.1
model "lifecycle-augment-topology"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 5

entity asset suite : CRE.Asset.Unit {
  rentable_area = 10000
  months_in_state = 0
}

// A model may add arrival actions to a pack's machine and nothing else. The
// pack's machine is the checkable contract; a model needing different topology
// declares a separate machine under its own name.
lifecycle cre.unit {
  initial vacant
  state vacant, leased
  on enter leased {
    set months_in_state = 0
  }
}

stream cre.rent on entity asset.suite inflow currency USD {
  schedule every year from 2026-01 to 2030-01
  category operating.revenue.base_rent
  amount = 100
}
```

- `E1357_LIFECYCLE_AUGMENT_TOPOLOGY` (error): Lifecycle 'cre.unit' is declared by a pack, and this block states initial and state.
  - hint: A model may add arrival actions to a pack's machine — `on enter <state>` and actions on an existing edge — and nothing else. To change the states or the edges, declare a separate machine under its own name and bind the entity to that instead.

Fix: not yet recorded.

## lifecycle_conflict — E1350_LIFECYCLE_CONFLICT

Failing example:

```cfdl
version 0.1
model "lifecycle-conflict"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 3

// ONE MACHINE PER ENTITY. CRE.Asset.Unit's type already declares the
// cre.unit lifecycle; binding a model-declared one too would leave two
// authorities over one status.
lifecycle unit { initial a  state a, b }

entity asset x : CRE.Asset.Unit {
  lifecycle unit
  rentable_area = 1
}
```

- `E1350_LIFECYCLE_CONFLICT` (error): Entity 'asset.x' binds a model-declared lifecycle, but its type 'CRE.Asset.Unit' already declares lifecycle 'cre.unit'. One machine per entity — drop the binding, or use an untyped entity.

Minimal fix (compiles):

```cfdl
version 0.1
model "lifecycle-conflict"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 3

// FIX: one machine per entity — the type CRE.Asset.Unit already declares the
// cre.unit lifecycle, so drop the model-declared machine and the binding.

entity asset x : CRE.Asset.Unit {
  rentable_area = 1
}
```

## lifecycle_duplicate — E1352_DUPLICATE_LIFECYCLE

Failing example:

```cfdl
version 0.1
model "lifecycle-duplicate"
time calendar monthly from 2026-01 for 3

// ONE MACHINE, ONE DECLARATION. Two blocks sharing a name would leave the
// binding ambiguous about which relation governs the entity.
lifecycle unit { initial a  state a }
lifecycle unit { initial a  state a }

entity asset x { lifecycle unit }
```

- `E1352_DUPLICATE_LIFECYCLE` (error): Lifecycle 'unit' is declared twice. One machine, one declaration — merge the edges into one block.

Minimal fix (compiles):

```cfdl
version 0.1
model "lifecycle-duplicate"
time calendar monthly from 2026-01 for 3

// FIX: one machine, one declaration — the two identical blocks are merged
// into a single declaration.
lifecycle unit { initial a  state a }

entity asset x { lifecycle unit }
```

## lifecycle_edge_unknown_state — E1316_UNKNOWN_LIFECYCLE_STATE

Failing example:

```cfdl
version 0.1
model "lifecycle-edge-unknown-state"
time calendar monthly from 2026-01 for 3

// THE STATES ARE ENUMERATED SO THIS IS AN ERROR, NOT A PHANTOM STATE.
// An inferred machine would silently create 'downtme' here — the exact
// stringly-typed failure E1332 exists to catch on `active in state`.
lifecycle unit {
  initial leased
  state leased, downtime
  leased -> downtme when time.t == 1
}

entity asset x { lifecycle unit  state leased }
```

- `E1316_UNKNOWN_LIFECYCLE_STATE` (error): Lifecycle 'unit' has an edge naming 'downtme', which is not a declared state. Declared: downtime, leased.

Minimal fix (compiles):

```cfdl
version 0.1
model "lifecycle-edge-unknown-state"
time calendar monthly from 2026-01 for 3

// FIX: the edge's target was the typo 'downtme'; it now names the declared
// state 'downtime'.
lifecycle unit {
  initial leased
  state leased, downtime
  leased -> downtime when time.t == 1
}

entity asset x { lifecycle unit  state leased }
```

## lifecycle_no_initial — E1351_LIFECYCLE_NO_INITIAL

Failing example:

```cfdl
version 0.1
model "lifecycle-no-initial"
time calendar monthly from 2026-01 for 3

// EVERY MACHINE OPENS SOMEWHERE. A machine with no initial state has no
// answer to "what is the entity's status before anything fires".
lifecycle unit {
  state a, b
  a -> b when time.t == 1
}

entity asset x { lifecycle unit  state a }
```

- `E1351_LIFECYCLE_NO_INITIAL` (error): Lifecycle 'unit' declares no initial state. Every machine opens somewhere — add `initial <state>`.

Minimal fix (compiles):

```cfdl
version 0.1
model "lifecycle-no-initial"
time calendar monthly from 2026-01 for 3

// FIX: every machine opens somewhere — declare `initial a`.
lifecycle unit {
  initial a
  state a, b
  a -> b when time.t == 1
}

entity asset x { lifecycle unit  state a }
```

## lifecycle_unreachable_write — E1353_UNREACHABLE_STATE_WRITE

Failing example:

```cfdl
version 0.1
model "lifecycle-unreachable-write"
time calendar monthly from 2026-01 for 3

// NO EDGE ENTERS 'c', SO THE WRITE CAN NEVER BE LEGAL — whatever state the
// entity is in at run. That certainty is what makes the refusal
// compile-time (docs/28 §6.1 rule 3); the run-time half, where the
// from-state is a fact, lives in the engine.
lifecycle unit {
  initial a
  state a, b, c
  a -> b when time.t == 1
}

entity asset x { lifecycle unit  state a }

event jump when time.t == 1 {
  set entity asset.x.status = "c"
}
```

- `E1353_UNREACHABLE_STATE_WRITE` (error): Event 'jump' sets 'asset.x.status' to 'c', but no edge of lifecycle 'unit' enters that state — the write can never be legal.
  - hint: Declare the edge — declaring it is what brings the move into existence — or drop the write.

Minimal fix (compiles):

```cfdl
version 0.1
model "lifecycle-unreachable-write"
time calendar monthly from 2026-01 for 3

// FIX: declare an edge entering 'c' — declaring it is what brings the move
// into existence, making the event's write legal.
lifecycle unit {
  initial a
  state a, b, c
  a -> b when time.t == 1
  b -> c when time.t == 2
}

entity asset x { lifecycle unit  state a }

event jump when time.t == 1 {
  set entity asset.x.status = "c"
}
```

## lifecycle_unresolved_ref — E1349_UNRESOLVED_LIFECYCLE_REF

Failing example:

```cfdl
version 0.1
model "lifecycle-unresolved-ref"
time calendar monthly from 2026-01 for 3

// THE BINDING NAMES A MACHINE THAT EXISTS. `lifecycle ghost` with no
// declaration is the same class of error as an unknown ontology type.
entity asset x { lifecycle ghost  state a }
```

- `E1349_UNRESOLVED_LIFECYCLE_REF` (error): Entity 'asset.x' binds lifecycle 'ghost', which is not declared. Declare it as `lifecycle ghost { ... }`.

Minimal fix (compiles):

```cfdl
version 0.1
model "lifecycle-unresolved-ref"
time calendar monthly from 2026-01 for 3

// FIX: the binding must name a machine that exists — declare `lifecycle
// ghost` with the state the entity opens in.
lifecycle ghost { initial a  state a }

entity asset x { lifecycle ghost  state a }
```

## metric_forward_ref — E1354_METRIC_FORWARD_REF

Failing example:

```cfdl
version 0.1
model "metric-forward-ref"
time calendar annual from 2026-01 for 3

// A METRIC READS THE METRICS ABOVE IT.
//
// Declaration order is the rule waterfalls already follow, which makes the
// dependency an order rather than a graph: the engine's fold always finds a
// value already computed. `headline` reads a metric declared below it, so
// there would be nothing there when it ran.

entity asset co : Asset.Financial

metric headline = metric.base + 1.0
metric base     = 2.0
```

- `E1354_METRIC_FORWARD_REF` (error): Metric 'headline' reads 'base', which is declared below it or not at all.
  - hint: Metrics compose in declaration order, so a metric may read the metrics above it. Move the declaration up, or correct the name.

Fix: not yet recorded.

## metric_self_ref — E1354_METRIC_FORWARD_REF

Failing example:

```cfdl
version 0.1
model "metric-self-ref"
time calendar annual from 2026-01 for 3

// A METRIC IS NOT A RECURRENCE.
//
// Reading itself asks for the previous value of something computed once, at
// the horizon, over a projection that has already finished. A running
// quantity is a field the walk advances and a distribution moves; a metric
// folds the result.

entity asset co : Asset.Financial

metric running = metric.running + 1.0
```

- `E1354_METRIC_FORWARD_REF` (error): Metric 'running' reads itself.
  - hint: A metric is a fold over the finished projection, not a recurrence; carry a running quantity as a field the walk advances.

Fix: not yet recorded.

## metric_unknown_series — E1365_METRIC_UNKNOWN_SERIES

Failing example:

```cfdl
version 0.1
model "metric-unknown-series"
time calendar annual from 2026-01 for 3

// A NAME NOTHING BINDS IS REFUSED, NOT READ AS ZERO.
//
// `docs/13` §7.85: `series_sum` returns 0.0 for a selector that matches
// nothing, which is right for a `.*` selector — matching nothing is a stated
// possibility there — and wrong for a name spelled out in full. In a stream
// the miss is a warning (`W5022`), because the series it produces is there to
// be looked at. In a metric there is nothing to look at: a fold publishes ONE
// number, under a name the author chose, and a wrong one is indistinguishable
// from a right one. `series_sum("total.nonsense.xyz", 0, 11)` published 0 with
// no diagnostic at all.

entity asset proj : Asset.Financial

stream ops.rev on entity asset.proj inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 100
}

metric typo = series_sum("total.nonsense.xyz", 0, 2)
```

- `E1365_METRIC_UNKNOWN_SERIES` (error): Metric 'typo' folds series 'total.nonsense.xyz', which this model does not publish.
  - hint: Check the spelling. A metric may fold any series this model publishes: a stream by its own name or as `stream.<name>`, a waterfall step, `entity.<symbol>.net_cash_flow`, `account.<name>`, an entity field, a money subtotal, a slice's net as `slice.<name>`, or `model.net_cash_flow`. A selector ending in `.*` states that matching nothing is intended.

Fix: not yet recorded.

## missing_time — E1103_MISSING_TIME, E1109_MISSING_ENTITY

Failing example:

```cfdl
version 0.1
model "missing-time"
```

- `E1103_MISSING_TIME` (error): Model is missing required 'time' statement.
- `E1109_MISSING_ENTITY` (error): Model must declare at least one entity.

Minimal fix (compiles):

```cfdl
version 0.1
model "missing-time"
time calendar monthly from 2026-01 for 12

entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower {
  schedule every month from 2026-01 to 2026-12
  amount = 1000
}
```

## near_miss_field — E1313_UNKNOWN_ENTITY_FIELD

Failing example:

```cfdl
version 0.1
model "near-miss-field"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 3

// A PACK DECLARES A FLOOR, NOT A CEILING.
//
// A modeller may add fields a pack's vocabulary does not cover — that is how a
// model says something the pack did not anticipate, and it was already true of
// fields carrying a rule.
//
// What still fails is a NEAR MISS. `senority` beside a declared `seniority` is
// a typo, and allowing it would make the value a field nobody reads: the quiet
// kind of wrong, where the model looks like it says something and does not.

entity asset tlb : Credit.Asset.Tranche {
  senority = 1
}
```

- `E1313_UNKNOWN_ENTITY_FIELD` (error): Entity 'asset.tlb' of type 'Credit.Asset.Tranche' sets 'senority', which the type does not declare.
  - hint: Declared fields: original_balance, seniority.

Minimal fix (compiles):

```cfdl
version 0.1
model "near-miss-field"
use pack "credit" version "0.1.0"
time calendar annual from 2026-01 for 3

// FIX: the near-miss typo 'senority' is corrected to the declared field
// 'seniority'.

entity asset tlb : Credit.Asset.Tranche {
  seniority = 1
}
```

## not_implemented_compile — E1109_MISSING_ENTITY

Failing example:

```cfdl
version 0.1
model "smoke"
time calendar monthly from 2026-01 for 1
```

- `E1109_MISSING_ENTITY` (error): Model must declare at least one entity.

Minimal fix (compiles):

```cfdl
version 0.1
model "smoke"
time calendar monthly from 2026-01 for 1

// FIX: a model must declare at least one entity.
entity asset borrower : Asset.Financial
```

## opco_bad_exit_multiple — E7021_OPCO_EXIT_INVALID_MULTIPLE

Failing example:

```cfdl
version 0.1
model "opco-bad-exit-multiple"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity asset business : OpCo.Asset.Enterprise

contract opco.exit_multiple {
  term 2030-12..2030-12
  terms {
    multiple = 0
    base = 500000
  }
}
```

- `E7021_OPCO_EXIT_INVALID_MULTIPLE` (error): OpCo exit 'multiple' must be greater than 0.

Minimal fix (compiles):

```cfdl
version 0.1
model "opco-bad-exit-multiple"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 60

entity asset business : OpCo.Asset.Enterprise

// FIX: exit_multiple must be greater than 0; 0 becomes a real multiple.
contract opco.exit_multiple {
  term 2030-12..2030-12
  terms {
    multiple = 7.0
    base = 500000
  }
}
```

## opco_bad_schedule — E7002_OPCO_LINE_INVALID_SCHEDULE

Failing example:

```cfdl
version 0.1
model "opco-bad-schedule"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset business : OpCo.Asset.Enterprise

contract opco.opex_line {
  term 2027-12..2026-01
  terms {
    amount = 50000
  }
}
```

- `E7002_OPCO_LINE_INVALID_SCHEDULE` (error): OpCo line term range is missing, invalid, or outside model timeline.

Minimal fix (compiles):

```cfdl
version 0.1
model "opco-bad-schedule"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset business : OpCo.Asset.Enterprise

// FIX: the term range was reversed; start now precedes end and both sit
// inside the model timeline.
contract opco.opex_line {
  term 2026-01..2027-12
  terms {
    amount = 50000
  }
}
```

## opco_missing_amount — E1372_MISSING_CONTRACT_TERM, E7001_OPCO_LINE_MISSING_AMOUNT

Failing example:

```cfdl
version 0.1
model "opco-missing-amount"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset business : OpCo.Asset.Enterprise

contract opco.revenue_line {
  term 2026-01..2027-12
  terms {
    growth_rate = 0.02
  }
}
```

- `E1372_MISSING_CONTRACT_TERM` (error): Contract 'opco.revenue_line' states none of amount, amount_year; type 'OpCo.Contract.RevenueLine' requires one of them.
- `E7001_OPCO_LINE_MISSING_AMOUNT` (error): OpCo line must state a size: 'amount' (per period) or 'amount_year' (annual).

Minimal fix (compiles):

```cfdl
version 0.1
model "opco-missing-amount"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset business : OpCo.Asset.Enterprise

// FIX: the line must state a size — give it an annual amount.
contract opco.revenue_line {
  term 2026-01..2027-12
  terms {
    amount_year = 600000
    growth_rate = 0.02
  }
}
```

## option_master_type — E1374_ABSTRACT_TYPE_INSTANTIATED

Failing example:

```cfdl
version 0.1
model "option-master-type"
time calendar annual from 2026-01 for 3

// A MASTER IS REFINED, NEVER DECLARED (docs/40 §2). `Contract.Option` is the
// master every election refines; an option names one of its concrete
// refinements, which the hint lists.
entity asset co : Asset.Financial

option renewal type Contract.Option { exercise when false payoff 1 }
```

- `E1374_ABSTRACT_TYPE_INSTANTIATED` (error): Option 'renewal' declares type 'Contract.Option', which is a master. A master is refined, never declared.
  - hint: Concrete elections: Option.Call, Option.Put, Option.Refinance, Option.Renewal.

Fix: not yet recorded.

## option_unknown_type — E1373_UNKNOWN_CONTRACT_TYPE

Failing example:

```cfdl
version 0.1
model "option-unknown-type"
time calendar annual from 2026-01 for 3

// AN OPTION'S TYPE RESOLVES OR IS REFUSED (docs/13 §7.67; docs/40 §8). The
// language base carries four generic elections — Option.Call, Option.Put,
// Option.Renewal, Option.Refinance — and a pack adds its own; a name that is
// none of them was accepted silently before, so a typo was a type.
entity asset co : Asset.Financial

option renewal type Option.Cal { exercise when false payoff 1 }
```

- `E1373_UNKNOWN_CONTRACT_TYPE` (error): Option 'renewal' declares type 'Option.Cal', which the active ontology does not define.
  - hint: Did you mean Option.Call?

Fix: not yet recorded.

## pack_actual_amortization_basis — E5027_ACTUAL_AMORTIZATION_BASIS

Failing example:

```cfdl
version 0.1
model "pack-actual-amortization-basis"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 361

// A LEVEL PAYMENT CANNOT BE STRUCK FROM A VARYING DIVISOR.
//
// `amortization_day_count` chooses what the constant payment is struck on. An
// Actual convention expands to `(360 / time.days_in_period)`, which is a
// per-period value, and the annuity it feeds applies it to every remaining
// period — so January strikes a payment as if all 359 remaining months had 31
// days, and February as if they all had 28. The payment then moves with month
// length, which no loan document does.
//
// Measured on exactly this loan before the check existed: the payment swung
// 460.68 over twelve months — 7,349.63 in a 31-day month, 6,888.95 in
// February. That is not a pool effect. There is no pool here: no prepayment,
// no defaults, one balance. The closed form is applying a period-local divisor
// to a whole remaining term, and it is wrong for a single loan too.
//
// What an Actual/360 loan document says is the OTHER pairing, and it compiles:
// strike the payment on `30/360` and accrue interest on `act/360`. The payment
// then holds at 7,194.61 while interest moves with month length — 6,200.00 in
// January, 5,594.43 in February — and principal absorbs the difference.
// `fixtures/valid/pack_amortization_day_count` pins that spelling.

entity asset loan : Credit.Asset.LoanPool

contract credit.pool_level_pay on entity asset.loan {
  term 2026-01..2056-01
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 360
    cpr = 0
    cdr = 0
    day_count = "act/360"
    amortization_day_count = "act/360"
  }
}
```

- `E5027_ACTUAL_AMORTIZATION_BASIS` (error): Contract 'credit.pool_level_pay' declares amortization_day_count = 'act/360'. A level payment is struck once and held; an Actual basis makes it move with month length, because the divisor is period-local and the annuity applies it to every remaining period. Strike the payment on '30/360' or '30e/360' and accrue interest on the Actual basis with `day_count`, which is what an Actual/360 loan document says.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-actual-amortization-basis"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 361

// FIX: what an Actual/360 loan document says is the compiling pairing —
// strike the payment on `30/360`, accrue interest on `act/360`.

entity asset loan : Credit.Asset.LoanPool

contract credit.pool_level_pay on entity asset.loan {
  term 2026-01..2056-01
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 360
    cpr = 0
    cdr = 0
    day_count = "act/360"
    amortization_day_count = "30/360"
  }
}
```

## pack_ambiguous_amount — E7010_OPCO_LINE_AMBIGUOUS_AMOUNT

Failing example:

```cfdl
version 0.1
model "pack-ambiguous-amount"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset business : OpCo.Asset.Enterprise

// `amount` is per period and `amount_year` is annual: two ways to say the
// same thing. A lowering rule sums them with zero defaults, because templates
// have no conditional, so giving both quietly means 240,000 a period rather
// than either figure the author had in mind.
contract opco.revenue_line {
  term 2026-01..2026-12
  terms {
    amount = 120000
    amount_year = 1440000
  }
}
```

- `E7010_OPCO_LINE_AMBIGUOUS_AMOUNT` (error): OpCo line states both 'amount' (per period) and 'amount_year' (annual); they would be summed. Give one.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-ambiguous-amount"
use pack "opco" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset business : OpCo.Asset.Enterprise

// FIX: give one size, not both — keep the per-period `amount` and drop
// `amount_year`.
contract opco.revenue_line {
  term 2026-01..2026-12
  terms {
    amount = 120000
  }
}
```

## pack_period_term_not_literal — E5017_PERIOD_TERM_NOT_LITERAL

Failing example:

```cfdl
version 0.1
model "pack-period-term-not-literal"
use pack "testpack" version "0.1.0"
time calendar quarterly from 2026-01 for 8

entity asset borrower : Asset.Financial

assume tenor = 24

// A months-denominated term is converted into the rule's own periods at
// compile time, so it has to be a literal. Deferring it to an input moves the
// value to run time, where the conversion can no longer happen.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2027-10
  terms {
    amount_year = 120000
    term_months = inputs.tenor
  }
}
```

- `E5017_PERIOD_TERM_NOT_LITERAL` (error): Pack lowering rule 'emit_cadence_probe' converts term 'term_months' from months into periods, so it must be a literal; contract 'test.cadence_probe' supplies `inputs.tenor`.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-period-term-not-literal"
use pack "testpack" version "0.1.0"
time calendar quarterly from 2026-01 for 8

entity asset borrower : Asset.Financial

// FIX: a months-denominated term is converted at compile time, so it must be
// a literal — `term_months = 24` rather than `inputs.tenor`.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2027-10
  terms {
    amount_year = 120000
    term_months = 24
  }
}
```

## pack_reserved_term_prefix — E5016_RESERVED_TERM_PREFIX

Failing example:

```cfdl
version 0.1
model "pack-reserved-term-prefix"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset borrower : Asset.Financial

// Lowering rules resolve cadence placeholders before contract terms, so a term
// under a reserved prefix would never be read. Term keys may legitimately be
// dotted, so this is reachable by accident, not only by perversity.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2026-12
  terms {
    amount_year = 120000
    term_months = 12
    model.periods_per_year = 3
  }
}
```

- `E5016_RESERVED_TERM_PREFIX` (error): Contract 'test.cadence_probe' declares term 'model.periods_per_year', but 'model.' is reserved for cadence placeholders that lowering rules resolve before contract terms. The term would never be read. Rename it.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-reserved-term-prefix"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset borrower : Asset.Financial

// FIX: 'model.' is reserved for cadence placeholders, so the term
// `model.periods_per_year` is dropped — it would never be read.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2026-12
  terms {
    amount_year = 120000
    term_months = 12
  }
}
```

## pack_rule_cadence_unsupported — E5014_RULE_CADENCE_UNSUPPORTED

Failing example:

```cfdl
version 0.1
model "pack-rule-cadence-unsupported"
use pack "testpack" version "0.1.0"
time calendar quarterly from 2026-01 for 8

entity asset borrower : Asset.Financial

// testpack is unconstrained, but this one rule divides by a literal 12 and so
// declares itself monthly-only. The per-rule gate is what lets a pack ship a
// mix of neutral and month-locked rules mid-migration rather than being
// blocked wholesale.
contract test.monthly_only_contract on entity asset.borrower {
  term 2026-01..2027-10
  terms {
    amount_year = 120000
  }
}
```

- `E5014_RULE_CADENCE_UNSUPPORTED` (error): Pack lowering rule 'emit_monthly_only_fee' lowers correctly on monthly calendars; this model declares 'quarterly'. It would produce amounts scaled to the wrong period.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-rule-cadence-unsupported"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 22

entity asset borrower : Asset.Financial

// FIX: the rule declares itself monthly-only, so the model runs on the
// monthly calendar it supports (same 2026-01..2027-10 window).
contract test.monthly_only_contract on entity asset.borrower {
  term 2026-01..2027-10
  terms {
    amount_year = 120000
  }
}
```

## pack_rule_prev_first_period — E1129_PREV_IN_FIRST_PERIOD

Failing example:

```cfdl
version 0.1
model "pack-rule-prev-first-period"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 4

entity asset borrower : Asset.Financial

// The same contract as `pack_rule_reads_prev_field`, with its term starting AT
// the model's first period instead of one after it.
//
// The rule strikes interest on the average of the field's opening and closing
// value, so at t = 0 it reads a close that does not exist. The engine's
// previous-value map is empty there: without this check the read warns and
// substitutes zero for that ONE period while every later period is correct —
// and the run still reports ok. One wrong period inside a right series is the
// hardest shape to spot.
//
// The message names the CONTRACT rather than the stream's schedule, because
// the pack owns the schedule and moving the term is the remedy available to
// whoever wrote this model.
contract test.avg_balance_contract on entity asset.borrower {
  term 2026-01..2026-04
  terms {
    draw = 100
    rate = 0.01
  }
}
```

- `E1129_PREV_IN_FIRST_PERIOD` (error): Stream 'testpack.avg_balance_interest', lowered from contract 'test.avg_balance_contract', reads a field's previous period but runs from the model's first period, where there is none. Start the contract's term one period after the model, or have the rule carry the opening value as a field of its own.
  - hint: A field's previous period is the close before this one; the first period has no close before it.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-rule-prev-first-period"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 4

entity asset borrower : Asset.Financial

// FIX: start the contract's term one period after the model, so the rule's
// previous-period read always has a close to read.
contract test.avg_balance_contract on entity asset.borrower {
  term 2026-02..2026-04
  terms {
    draw = 100
    rate = 0.01
  }
}
```

## pack_template_missing_term — E5006_MISSING_CONTRACT_TERM

Failing example:

```cfdl
version 0.1
model "pack-template-missing-term"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 6

entity asset borrower : Asset.Financial

// Missing required term `rate` (no default declared for it in the rule).
contract test.fee_contract {
  term 2026-01..2026-06
}
```

- `E5006_MISSING_CONTRACT_TERM` (error): Pack lowering rule 'emit_parameterized_fee' requires contract term 'rate' (no default declared); contract 'test.fee_contract' does not provide it.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-template-missing-term"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 6

entity asset borrower : Asset.Financial

// FIX: supply the required term `rate` (the rule declares no default for it).
contract test.fee_contract {
  term 2026-01..2026-06
  terms {
    rate = 25
  }
}
```

## pack_term_months_not_divisible — E5015_TERM_MONTHS_NOT_DIVISIBLE

Failing example:

```cfdl
version 0.1
model "pack-term-months-not-divisible"
use pack "testpack" version "0.1.0"
time calendar annual from 2026-01 for 4

entity asset borrower : Asset.Financial

// term_months is a count of payment periods: it goes into pow exponents and
// annuity term arguments. 30 months is two and a half annual periods, which
// no closed form can express — a 30-month loan is not 2.5 annual payments.
// Thresholds pro-rate; counts cannot, so this is an error rather than a round.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2029-01
  terms {
    amount_year = 120000
    term_months = 30
  }
}
```

- `E5015_TERM_MONTHS_NOT_DIVISIBLE` (error): Pack lowering rule 'emit_cadence_probe' uses term 'term_months' as a count of payment periods, but 30 months is 2.5 periods at annual frequency. Use a multiple of 12 months, declare a finer payment_frequency, or model on a finer calendar.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-term-months-not-divisible"
use pack "testpack" version "0.1.0"
time calendar annual from 2026-01 for 4

entity asset borrower : Asset.Financial

// FIX: term_months must be a whole number of annual periods — 36 months is
// exactly three, where 30 was two and a half.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2029-01
  terms {
    amount_year = 120000
    term_months = 36
  }
}
```

## pack_term_start_off_grid — E5018_TERM_START_OFF_GRID

Failing example:

```cfdl
version 0.1
model "pack-term-start-off-grid"
use pack "testpack" version "0.1.0"
time calendar quarterly from 2026-01 for 8

entity asset borrower : Asset.Financial

// Periods begin 2026-01 and step three months: 2026-01, 2026-04, 2026-07 …
// A term starting 2026-02 begins one month into a period, so counting elapsed
// periods from it is short by a partial period for the whole contract.
contract test.fee_contract on entity asset.borrower {
  term 2026-02..2027-11
  terms {
    rate = 100
  }
}
```

- `E5018_TERM_START_OFF_GRID` (error): Contract 'test.fee_contract' starts 2026-02-01 but the model's quarterly periods begin 2026-01-01 and step from there. A term must start on a period boundary, or elapsed-period counting is off by a partial period.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-term-start-off-grid"
use pack "testpack" version "0.1.0"
time calendar quarterly from 2026-01 for 8

entity asset borrower : Asset.Financial

// FIX: the term now starts on a period boundary — 2026-01, where the model's
// quarterly grid begins — instead of one month into a period (and ends inside
// the timeline).
contract test.fee_contract on entity asset.borrower {
  term 2026-01..2027-10
  terms {
    rate = 100
  }
}
```

## pack_unknown_day_count — E5019_UNKNOWN_DAY_COUNT

Failing example:

```cfdl
version 0.1
model "pack-unknown-day-count"
use pack "credit" version "0.1.0"
time calendar monthly from 2025-01 for 13

entity asset buyer : Credit.Asset.LoanPool

// A misspelled convention must not fall back to a default silently: the
// difference between act/360 and act/365 is about 1.4% of interest.
contract credit.pool_io_bullet.p on entity asset.buyer {
  term 2025-01..2025-12
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 12
    cpr = 0
    cdr = 0
    severity = 0
    recovery_lag_months = 0
    day_count = "actual/360"
  }
}
```

- `E5019_UNKNOWN_DAY_COUNT` (error): Contract 'credit.pool_io_bullet.p' declares day_count = 'actual/360'. Supported: 30/360, 30e/360, act/360, act/365.

Minimal fix (compiles):

```cfdl
version 0.1
model "pack-unknown-day-count"
use pack "credit" version "0.1.0"
time calendar monthly from 2025-01 for 13

entity asset buyer : Credit.Asset.LoanPool

// Fix: spell the convention as one of the supported names — "act/360",
// not "actual/360".
contract credit.pool_io_bullet.p on entity asset.buyer {
  term 2025-01..2025-12
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 12
    cpr = 0
    cdr = 0
    severity = 0
    recovery_lag_months = 0
    day_count = "act/360"
  }
}
```

## parse_unexpected_token — E0001_UNEXPECTED_TOKEN

Failing example:

```cfdl
version 0.1
model "parse_unexpected"
time calendar monthly from 2026-01 for 12
foobar
```

- `E0001_UNEXPECTED_TOKEN` (error): Unexpected token <identifier>.

Minimal fix (compiles):

```cfdl
version 0.1
model "parse_unexpected"
time calendar monthly from 2026-01 for 12
// Fix: the stray `foobar` token is removed (an entity is added, since a model
// must declare at least one).
entity asset co : Asset.Financial
```

## participant_return_not_a_party — E1356_PARTICIPANT_RETURN_NOT_A_PARTY

Failing example:

```cfdl
version 0.1
model "participant-return-not-a-party"
time calendar annual from 2026-01 for 3

// A RETURN BELONGS TO A PARTICIPANT, AND IS FOLDED OVER THEIR ACCOUNT.
//
// The party here is declared and is a party, and still has nothing to fold:
// no account names it as owner, so there is no record of what it contributed
// or received. A step's payee would not do — attributing through stream names
// is the question docs/13 §7.43 records, and it is not this one.
//
// The argument is a REFERENCE rather than text, which is what lets this be a
// diagnostic at all: the compiler resolves `party.silent` against the declared
// entities, checks it is a party, and checks an account names it.

entity asset deal : Asset.Financial
entity party silent : Party { name = "Never Paid" }

metric silent_return = irr(party.silent)
```

- `E1356_PARTICIPANT_RETURN_NOT_A_PARTY` (error): Metric 'silent_return' folds the return of 'party.silent', which owns no account.
  - hint: A participant's return is folded over the party's own account: contributions are negative inflows and receipts are allocations in. Declare `account <name> { owner party.silent … }` and pay the waterfall's steps into it.

Fix: not yet recorded.

## participant_return_outside_metric — E1355_PARTICIPANT_RETURN_OUTSIDE_METRIC

Failing example:

```cfdl
version 0.1
model "participant-return-outside-metric"
time calendar annual from 2026-01 for 3

// A RETURN IS A FOLD OVER THE FINISHED PROJECTION.
//
// Reading one in a stream amount asks for a return on cash this stream has
// not produced yet. The evaluator refuses it, but a stream amount that fails
// to evaluate warns and substitutes zero — so before this check the model ran
// `status: ok` on a column of zeroes.

entity asset deal : Asset.Financial
entity party lp   : Party { name = "Limited Partner" }

account lp_capital {
  owner party.lp
  from -100.0
}

stream deal.fee on entity asset.deal inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = irr(party.lp) * 100.0
}
```

- `E1355_PARTICIPANT_RETURN_OUTSIDE_METRIC` (error): Stream 'deal.fee' amount folds a participant's return.
  - hint: `irr` and `moic` are folds over the finished projection, so they belong in a `metric` declaration. Reading one here would ask for a return on cash this expression has not produced yet.

Fix: not yet recorded.

## party_role_unbound — E1322_UNKNOWN_PARTY_ROLE

Failing example:

```cfdl
version 0.1
model "party-role-unbound"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 15

entity asset pool : Credit.Asset.LoanPool
entity party obligors : Party { name = "Obligors" }

// ROLES ARE THE TYPE'S, RESOLVED THROUGH ITS MASTER (docs/40 §5). A credit
// pool refines `Contract.Debt`: its `holder` is the master's `lender`, and
// the master's `borrower` is UNBOUND — the obligors behind a purchased pool
// are many and unnamed, so the agreement has no such party in this form.
// Binding it is refused with the roles a model may bind.
contract credit.pool_level_pay.smoke on entity asset.pool {
  term 2026-01..2027-03
  terms {
    principal = 1200000
    interest_rate = 0.06
    term_months = 12
  }
  parties {
    borrower = party.obligors
  }
}
```

- `E1322_UNKNOWN_PARTY_ROLE` (error): Contract 'credit.pool_level_pay.smoke' binds role 'borrower', which type 'Credit.Contract.LevelPayPool' leaves unbound: the agreement has no such party in this form.
  - hint: Roles a model binds: holder (the master's lender).

Fix: not yet recorded.

## payment_terms_on_date — E0004_EXPECTED_TOKEN

Failing example:

```cfdl
version 0.1
model "payment-terms-on-date"
time calendar monthly from 2026-01 for 12

entity asset co : Asset.Financial

// A one-shot flow has no accrual period to settle after: the date it names is
// already the date the cash moves.
stream co.fee on entity asset.co outflow currency USD {
  schedule on 2026-03 net 30
  amount = 500
}
```

- `E0004_EXPECTED_TOKEN` (error): Payment terms do not apply to `schedule on <date>`: a one-shot flow has no accrual period to settle after. State the date the cash moves.

Minimal fix (compiles):

```cfdl
version 0.1
model "payment-terms-on-date"
time calendar monthly from 2026-01 for 12

entity asset co : Asset.Financial

// Fix: a one-shot flow states the date the cash moves, so the `net 30`
// payment terms are dropped from the on-date schedule.
stream co.fee on entity asset.co outflow currency USD {
  schedule on 2026-03
  amount = 500
}
```

## prev_in_first_period — E1129_PREV_IN_FIRST_PERIOD

Failing example:

```cfdl
version 0.1
model "prev-in-first-period"
time calendar annual from 2026-01 for 4

// THERE IS NO CLOSE BEFORE THE FIRST CLOSE.
//
// A stream may read a field's previous period — that is what retires the
// `_open` pattern — but not in the model's first period, where no previous
// period exists.
//
// Left to the engine the read resolves to nothing and the stream evaluates to
// zero: a wrong number arrived at by accident, in a model that looks like it
// says something. The same refusal `E1123` and `E1126` make one step along.
//
// The fix is either to start the stream a period later, as a debt schedule
// does, or to carry the opening value as a field that states what it is at
// period 0.

entity asset tlb : Asset.Financial {
  balance init 275.0 next max(0.0, prev - 25.0)
}

stream opco.interest on entity asset.tlb outflow currency USD {
  schedule every year from 2026-01 to 2029-01
  amount = (prev.asset.tlb.balance + asset.tlb.balance) / 2.0 * 0.06
}
```

- `E1129_PREV_IN_FIRST_PERIOD` (error): Stream 'opco.interest' reads a field's previous period but runs from the model's first period, where there is none. Start the stream one period later, or carry the opening value as a field of its own.
  - hint: A field's previous period is the close before this one; the first period has no close before it.

Minimal fix (compiles):

```cfdl
version 0.1
model "prev-in-first-period"
time calendar annual from 2026-01 for 4

// The fix: start the stream one period later, as a debt schedule does —
// the first close exists by the time the stream first reads it.

entity asset tlb : Asset.Financial {
  balance init 275.0 next max(0.0, prev - 25.0)
}

stream opco.interest on entity asset.tlb outflow currency USD {
  schedule every year from 2027-01 to 2029-01
  amount = (prev.asset.tlb.balance + asset.tlb.balance) / 2.0 * 0.06
}
```

## quantile_not_monotone — E5028_INVALID_QUANTILE

Failing example:

```cfdl
version 0.1
model "quantile-not-monotone"
time calendar monthly from 2026-01 for 2

entity asset a : Asset.Financial

// A QUANTILE FUNCTION CANNOT FALL.
//
// The value at the 90th percentile is below the value at the 50th, which is
// not a distribution. It matters beyond tidiness: `quantile_of` inverts this
// function, so a non-monotone declaration would leave a threshold lookup with
// two answers and nothing to choose between them. Rejecting it here is what
// makes the inverse well-defined rather than usually right.
//
// Expect E5028_INVALID_QUANTILE.
quantile broken {
  0.00:  10.0
  0.50: 100.0
  0.90:  40.0
}

stream a.line on entity asset.a inflow currency USD {
  category operating.revenue.other
  schedule every month from 2026-01 to 2026-02
  amount = quantile_at("broken", 0.5)
}
```

- `E5028_INVALID_QUANTILE` (error): Quantile 'broken' falls from 100 to 40 as share increases. A quantile function is non-decreasing; without that, `quantile_of` has no single answer and a threshold lookup would silently pick one of several.

Minimal fix (compiles):

```cfdl
version 0.1
model "quantile-not-monotone"
time calendar monthly from 2026-01 for 2

entity asset a : Asset.Financial

// Fix: a quantile function is non-decreasing, so the value at the 90th
// percentile is raised above the value at the 50th.
quantile broken {
  0.00:  10.0
  0.50: 100.0
  0.90: 140.0
}

stream a.line on entity asset.a inflow currency USD {
  category operating.revenue.other
  schedule every month from 2026-01 to 2026-02
  amount = quantile_at("broken", 0.5)
}
```

## quantile_share_out_of_range — E5028_INVALID_QUANTILE

Failing example:

```cfdl
version 0.1
model "quantile-share-out-of-range"
time calendar monthly from 2026-01 for 2

entity asset a : Asset.Financial

// A SHARE IS A FRACTION, NOT A COUNT.
//
// 8760 is the hours in a year, and writing it here is the mistake the [0, 1]
// range exists to catch: the measure belongs to whatever READS the quantile,
// which is what lets one price stack serve a 20 MW battery and a 200 MW one.
// Put the hours in the contract and the shares stay dimensionless.
//
// Expect E5028_INVALID_QUANTILE.
quantile hours {
  0.0:    11.0
  8760.0: 512.0
}

stream a.line on entity asset.a inflow currency USD {
  category operating.revenue.other
  schedule every month from 2026-01 to 2026-02
  amount = quantile_at("hours", 0.5)
}
```

- `E5028_INVALID_QUANTILE` (error): Quantile 'hours' has share 8760, which is outside 0..1. A share is a fraction of the measure, and the measure itself belongs to the contract that reads it.

Minimal fix (compiles):

```cfdl
version 0.1
model "quantile-share-out-of-range"
time calendar monthly from 2026-01 for 2

entity asset a : Asset.Financial

// Fix: shares are dimensionless fractions in [0, 1] — the 8760 hours belong
// to whatever contract reads the quantile, not to the shares themselves.
quantile hours {
  0.0:  11.0
  1.0: 512.0
}

stream a.line on entity asset.a inflow currency USD {
  category operating.revenue.other
  schedule every month from 2026-01 to 2026-02
  amount = quantile_at("hours", 0.5)
}
```

## run_monte_carlo_zero_trials — E0004_EXPECTED_TOKEN

Failing example:

```cfdl
version 0.1
model "run-monte-carlo-zero-trials"
time calendar monthly from 2026-01 for 6

// `trials` MUST BE POSITIVE, which the diagnostic always said and the check
// did not enforce: `parse::<u64>()` accepted 0.
//
// What that produced: the model compiled, the engine's `trials > 0` guard
// declined to set up the run, and the results carried no Monte Carlo section
// at all — a run mode asked for, accepted, and silently not performed.
//
// Found by mutation testing (`docs/30`). The engine's guard had no reachable
// case distinguishing `> 0` from `>= 0`, which is what a surviving mutant
// means: not that the guard is wrong, but that nothing could tell.

entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 1000
}

run monte_carlo trials 0 seed 1
```

- `E0004_EXPECTED_TOKEN` (error): Expected positive integer after 'trials'.

Minimal fix (compiles):

```cfdl
version 0.1
model "run-monte-carlo-zero-trials"
time calendar monthly from 2026-01 for 6

// Fix: `trials` must be a positive integer.

entity asset borrower : Asset.Financial

stream lease.rent on entity asset.borrower inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  amount = 1000
}

run monte_carlo trials 1 seed 1
```

## schedule_finer_than_calendar — E2108_SCHEDULE_FINER_THAN_CALENDAR

Failing example:

```cfdl
version 0.1
model "schedule-finer-than-calendar"
time calendar monthly from 2026-01 for 24

entity asset co : Asset.Financial

// A weekly schedule on a monthly grid is unrepresentable: four or five
// occurrences fall in each period and collapse into a single payment, so the
// model paid 24 times over two years instead of about 104 — silently.
stream co.rent on entity asset.co inflow currency USD {
  schedule every week from 2026-01 to 2027-12
  amount = 1000
}
```

- `E2108_SCHEDULE_FINER_THAN_CALENDAR` (error): Stream 'co.rent' pays every week but the model's calendar is monthly. Occurrences inside one period share that period's environment and cannot be told apart, so an amount that varies over time would be computed once and multiplied. Use an interval of monthly or longer, or declare a finer calendar.

Minimal fix (compiles):

```cfdl
version 0.1
model "schedule-finer-than-calendar"
time calendar monthly from 2026-01 for 24

entity asset co : Asset.Financial

// Fix: the schedule cadence matches the calendar grid — monthly, not weekly.
stream co.rent on entity asset.co inflow currency USD {
  schedule every month from 2026-01 to 2027-12
  amount = 1000
}
```

## schedule_mid_conflict — E2109_SCHEDULE_CONFLICTING_PLACEMENT

Failing example:

```cfdl
version 0.1
model "schedule-mid-conflict"
time calendar annual from 2026-01 for 4
entity asset co : Asset.Financial

// Two positions in one period is a contradiction, not a refinement.
stream co.mid_and_due on entity asset.co inflow currency USD {
  schedule every year mid on day 15 from 2026-01 to 2029-01
  amount = 100
}

// Payment terms are resolved on the calendar and move cash between periods;
// `mid` positions cash inside whichever period it lands in. Composing them
// needs a design decision, so they are rejected rather than guessed at.
stream co.mid_and_net on entity asset.co inflow currency USD {
  schedule every year mid net 30 from 2026-01 to 2029-01
  amount = 100
}

// A day rule already names where the cash sits.
stream co.mid_and_day on entity asset.co inflow currency USD {
  schedule every year mid on day 15 from 2026-01 to 2029-01
  amount = 100
}
```

- `E2109_SCHEDULE_CONFLICTING_PLACEMENT` (error): Stream 'co.mid_and_due' combines `mid` with a day rule, which places the same cash on a stated date. A schedule states one position in its period, not two.
- `E2109_SCHEDULE_CONFLICTING_PLACEMENT` (error): Stream 'co.mid_and_net' combines `mid` with `net` payment terms, which are resolved on the calendar rather than as a position in the period. A schedule states one position in its period, not two.
- `E2109_SCHEDULE_CONFLICTING_PLACEMENT` (error): Stream 'co.mid_and_day' combines `mid` with a day rule, which places the same cash on a stated date. A schedule states one position in its period, not two.

Minimal fix (compiles):

```cfdl
version 0.1
model "schedule-mid-conflict"
time calendar annual from 2026-01 for 4
entity asset co : Asset.Financial

// Fix: a schedule states ONE position in its period, so each stream keeps a
// single placement — the day rule, the payment terms, or `mid` — not two.
stream co.mid_and_due on entity asset.co inflow currency USD {
  schedule every year on day 15 from 2026-01 to 2029-01
  amount = 100
}

stream co.mid_and_net on entity asset.co inflow currency USD {
  schedule every year net 30 from 2026-01 to 2029-01
  amount = 100
}

stream co.mid_and_day on entity asset.co inflow currency USD {
  schedule every year mid from 2026-01 to 2029-01
  amount = 100
}
```

## schedule_stub_rejected — E0004_EXPECTED_TOKEN

Failing example:

```cfdl
version 0.1
model "schedule-stub-rejected"
time calendar monthly from 2026-01 for 12

entity asset co : Asset.Financial

// `stub` is lexed and was previously discarded in silence, so a model could
// ask for a short front stub and get a full period with no diagnostic.
stream co.rent on entity asset.co inflow currency USD {
  schedule every month from 2026-01 to 2026-12 stub short_front
  amount = 1000
}
```

- `E0004_EXPECTED_TOKEN` (error): Stub periods are not supported. Remove `stub`, or express the partial period as its own schedule.

Minimal fix (compiles):

```cfdl
version 0.1
model "schedule-stub-rejected"
time calendar monthly from 2026-01 for 12

entity asset co : Asset.Financial

// Fix: stub periods are not supported, so the `stub short_front` clause is
// removed; a partial period would be its own schedule.
stream co.rent on entity asset.co inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 1000
}
```

## series_read_in_guard — E1134_SERIES_READ_IN_LOGIC

Failing example:

```cfdl
version 0.1
model "series-read-in-guard"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// AN EVENT'S GUARD CANNOT READ A STREAM.
//
// The unit should go to downtime when rent stops arriving, which is how a
// servicer behaves. Events are simulated before any stream is evaluated, so
// the read binds nothing and the comparison is false in every period.
//
// Measured before this was refused: rent is 0 from July, the guard's
// condition is plainly true, and the event NEVER FIRES —
// `deterministic.transitions` is null. The engine did report it, once per
// period, in `deterministic.warnings`: "series `cre.rent` is not available in
// this context; using false". That is a precise message in a place a modeller
// reading numbers does not look — the run reports `status: ok`, the CLI
// prints nothing, and the exit code is 0.
//
// The evidence that a warning is not enough is in this repository:
// `valid/evaluation_order` carried four of these warnings in its blessed
// golden, describing a guard that never fired, and no gate objected. A
// condition that produces wrong numbers is an error.

entity asset suite : CRE.Asset.Unit {
  rentable_area = 1000
  state leased
}

stream cre.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  category operating.revenue.base_rent
  amount = 100
  active in state leased
}

event vacate when series_sum("cre.rent", time.t, time.t) < 50 {
  set entity asset.suite.status = "vacant"
}
```

- `E1134_SERIES_READ_IN_LOGIC` (error): event 'vacate' guard reads `cre.rent` over a window ending at `time.t`, which is this period or later. Logic settles BEFORE this period's cash exists, so only history it can already see is readable: end the window at `time.t - 1` or earlier. A stream, a waterfall and the results layer do see the current period.

Minimal fix (compiles):

```cfdl
version 0.1
model "series-read-in-guard"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 12

// Fix: logic settles before this period's cash exists, so the guard's window
// ends at `time.t - 1` — the settled history it can already see.

entity asset suite : CRE.Asset.Unit {
  rentable_area = 1000
  state leased
}

stream cre.rent on entity asset.suite inflow currency USD {
  schedule every month from 2026-01 to 2026-06
  category operating.revenue.base_rent
  amount = 100
  active in state leased
}

event vacate when series_sum("cre.rent", time.t - 1, time.t - 1) < 50 {
  set entity asset.suite.status = "vacant"
}
```

## slice_bad_category_root — E1364_SLICE_CATEGORY_ROOT

Failing example:

```cfdl
version 0.1
model "slice-bad-category-root"
time calendar annual from 2026-01 for 2

entity asset a : Asset.Financial

stream a.x on entity asset.a inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 1
}

// A selector that could never match anything is a typo, not a choice.
slice s { category "revenue.royalty" }
```

- `E1364_SLICE_CATEGORY_ROOT` (error): Slice 's' selects category 'revenue.royalty', whose root 'revenue' is not one of operating, investing, financing.
  - hint: A category is a path into the cash flow statement; a selector that could never match anything is a typo, not a choice.

Fix: not yet recorded.

## slice_unknown_entity — E1362_SLICE_UNKNOWN_ENTITY

Failing example:

```cfdl
version 0.1
model "slice-unknown-entity"
time calendar annual from 2026-01 for 2

entity asset a : Asset.Financial

stream a.x on entity asset.a inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 1
}

// A slice selects by REFERENCE, and a reference is what the compiler can
// check — a misspelled entity is refused, not silently matched to nothing.
slice s { entity asset.nonesuch }
```

- `E1362_SLICE_UNKNOWN_ENTITY` (error): Slice 's' selects entity 'asset.nonesuch', which is not declared.
  - hint: A slice selects by reference, and a reference is what the compiler can check — correct the name or declare the entity.

Fix: not yet recorded.

## slice_unknown_line — E1375_UNKNOWN_LINE_ROLE

Failing example:

```cfdl
version 0.1
model "slice-unknown-line"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2027-12
  terms {
    principal = 3000000
    interest_rate = 0.06
    amortization_months = 300
  }
}

// A line is a ROLE a master names (docs/40 §6). One nothing produces is
// refused with the near miss, as a type is — a selector that could never
// match anything is a typo, not a choice.
slice debt_interest {
  type Contract.Debt
  line intrest
}
```

- `E1375_UNKNOWN_LINE_ROLE` (error): Slice 'debt_interest' selects line 'intrest', which no contract type in the active ontology produces.
  - hint: Did you mean interest?

Fix: not yet recorded.

## slice_unknown_type — E1363_SLICE_UNKNOWN_TYPE

Failing example:

```cfdl
version 0.1
model "slice-unknown-type"
time calendar annual from 2026-01 for 2

entity asset a : Asset.Financial

stream a.x on entity asset.a inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 1
}

slice s { type Contract.Imaginary }
```

- `E1363_SLICE_UNKNOWN_TYPE` (error): Slice 's' selects type 'Contract.Imaginary', which the active ontology does not define.
  - hint: Known contract types: Contract.CapitalExpenditure, Contract.Debt, Contract.Deduction, Contract.Derivative, Contract.Expense, Contract.Insurance, Contract.Lease, Contract.Line, Contract.Offtake, Contract.Option, Contract.Purchase, Contract.Revenue, Contract.Sale, Contract.Service, Contract.Tax, Contract.WorkingCapital, Option.Call, Option.Put, Option.Refinance, Option.Renewal.

Fix: not yet recorded.

## statement_authored_and_generated — E1369_STATEMENT_AUTHORED_AND_GENERATED

Failing example:

```cfdl
version 0.1
model "statement-authored-and-generated"
time calendar annual from 2026-01 for 2

// A STATEMENT IS AUTHORED OR GENERATED, NEVER BOTH.
//
// `docs/13` §7.55. A generated statement partitions the cash by construction —
// a hierarchy covers its own tree, so the lines always add to the whole. An
// authored statement partitions it by the author's care. Mixed, neither
// guarantee holds: the authored line below claims the same stream the
// generated rows already claimed, so the bottom line would count it twice and
// the reconciliation that makes a statement trustworthy would become noise.

entity asset co : Asset.Financial

stream ops.rent on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  category operating.revenue.base_rent
  amount = 100
}

statement both {
  structure entity
  depth     2
  line "Rent" { category "operating.revenue.base_rent" }
}
```

- `E1369_STATEMENT_AUTHORED_AND_GENERATED` (error): Statement 'both' states both a structure and its own rows.
  - hint: A statement either names a `structure` and lets the rows follow from the tree, or states its rows. Remove one.

Fix: not yet recorded.

## statement_row_unknown_type — E1363_SLICE_UNKNOWN_TYPE

Failing example:

```cfdl
version 0.1
model "statement-row-unknown-type"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2027-12
  terms {
    principal = 3000000
    interest_rate = 0.06
    amortization_months = 300
  }
}

// A row's `type` is checked as a slice's is: an ontology type the active
// vocabulary does not define is refused with the known types named.
statement lender {
  label "Lender view"
  line  "Debt service" { type Contract.Imaginary }
}
```

- `E1363_SLICE_UNKNOWN_TYPE` (error): Statement 'lender' row "Debt service" selects type 'Contract.Imaginary', which the active ontology does not define.
  - hint: Known contract types: CRE.Contract.ConstructionFunding, CRE.Contract.ConstructionLoan, CRE.Contract.Disposition, CRE.Contract.DispositionAtCap, CRE.Contract.DispositionAtForwardCap, CRE.Contract.Lease, CRE.Contract.OperatingExpense, CRE.Contract.OperatingRevenue, CRE.Contract.PercentageRent, CRE.Contract.PercentageRentExpected, CRE.Contract.PermanentDebt, CRE.Contract.PurchaseOption, CRE.Contract.RenewalOption, CRE.Contract.Rollover, CRE.Contract.UnitLease, CRE.Contract.VacancyAllowance, Contract.CapitalExpenditure, Contract.Debt, Contract.Deduction, Contract.Derivative, Contract.Expense, Contract.Insurance, Contract.Lease, Contract.Line, Contract.Offtake, Contract.Option, Contract.Purchase, Contract.Revenue, Contract.Sale, Contract.Service, Contract.Tax, Contract.WorkingCapital, Option.Call, Option.Put, Option.Refinance, Option.Renewal.

Fix: not yet recorded.

## statement_series_row_claims — E1370_STATEMENT_SERIES_ROW_CLAIMS

Failing example:

```cfdl
version 0.1
model "statement-series-row-claims"
time calendar annual from 2026-01 for 3

// A ROW DRAWS A PUBLISHED SERIES OR CLAIMS CASH, NEVER BOTH (`E1370`).
//
// A series row presents a fold and claims nothing; a category claims streams
// into the bottom line. Both on one row could only be resolved by a precedence
// the reader cannot see, which is a silently ignored clause — the failure
// §7.55 exists to end. Refused instead.

entity asset property : Asset.Financial

stream ops.rent on entity asset.property inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.base_rent
  amount = 1000
}

statement memo {
  label "Operating memo"
  line "Net (memo)" { series "model.net_cash_flow" category "operating.*" }
}
```

- `E1370_STATEMENT_SERIES_ROW_CLAIMS` (error): Statement 'memo' has a row drawing series 'model.net_cash_flow' beside a claim clause.
  - hint: A row draws a published series or claims cash, never both. Remove the `series`, or the other draw clauses.

Fix: not yet recorded.

## statement_unknown_structure — E1367_STATEMENT_UNKNOWN_STRUCTURE

Failing example:

```cfdl
version 0.1
model "statement-unknown-structure"
time calendar annual from 2026-01 for 2

// A STATEMENT PRESENTS A HIERARCHY THE ENGINE CAN BUILD, OR IT IS REFUSED.
//
// `docs/13` §7.55. Rendering a structure nobody implements would produce one
// residual row and nothing else — technically complete, and useless. A
// presentation that silently shows nothing is the failure this entry exists to
// end, so it is a compile error rather than a note inside the output.

entity asset co : Asset.Financial

stream ops.rent on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 100
}

statement wrong {
  structure region
  depth     2
}
```

- `E1367_STATEMENT_UNKNOWN_STRUCTURE` (error): Statement 'wrong' presents structure 'region', which is not one this engine builds.
  - hint: Known structures: entity, category.

Fix: not yet recorded.

## stream_active_not_bool — E2202_STREAM_ACTIVE_NOT_BOOL

Failing example:

```cfdl
version 0.1
model "stream-active-not-bool"
time calendar annual from 2026-01 for 3

// AN ACTIVATION PREDICATE THAT IS NOT A CONDITION.
//
// Taken as `false`, so the stream never paid — a zero column, a warning the
// CLI does not print, and `status: ok`.

entity asset co : Asset.Financial

stream a.rent on entity asset.co inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  amount = 100
  active when 7
}
```

- `E2202_STREAM_ACTIVE_NOT_BOOL` (error): Stream 'a.rent' is `active when 7`, which is not a condition.
  - hint: An activation predicate must be true or false. The engine would take a non-boolean as `false`, so the stream would never pay.

Fix: not yet recorded.

## stream_missing_category — E5029_STREAM_MISSING_CATEGORY

Failing example:

```cfdl
version 0.1
model "stream-missing-category"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 2

// A stream with no category, while a pack is active. Its cash reaches
// model.total and folds into no subtotal, so every domain metric is computed
// as though it were not there. E5029.
entity asset tower : CRE.Asset.RealProperty

stream misc.windfall on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2027-01
  amount = 250
}
```

- `E5029_STREAM_MISSING_CATEGORY` (error): Stream 'misc.windfall' declares no category, and pack 'cre' is active. Its cash would reach model.total and fold into no subtotal — invisible to every domain metric, silently.
  - hint: State what the flow IS, as a path into the cash flow statement: `category operating.revenue.rent`, `category financing.debt.interest_paid`. A category is only optional when no pack is active, because then nothing folds.

Fix: not yet recorded.

## stream_reads_waterfall_step — E1346_STREAM_READS_WATERFALL_STEP

Failing example:

```cfdl
version 0.1
model "stream-reads-waterfall-step"
time calendar annual from 2020-01 for 6

// A STREAM MAY NOT READ A WATERFALL STEP.
//
// The spelling below is the one an author reaches for: a management fee that
// tracks what the waterfall actually paid out. It reads as a dependency and it
// cannot be one — every waterfall runs after every stream, and a step's series
// publishes when its waterfall finishes, visible to a later waterfall's `from`
// and to nothing else (docs/03 §3.2). At the moment any stream evaluates there
// is nothing under that name to read; `series_sum` answered zero for it in
// silence, because the step counted as a known producer and no check looked.
//
// Dependency-ordered waves make deep chains between STREAMS legal, which makes
// this neighbouring read the more tempting to write — and it still cannot
// work, so it now says so at compile time instead of contributing nothing.

entity asset fund : Asset.Financial
entity party lp   : Party { name = "Limited Partners" }
entity party gp   : Party { name = "General Partner" }

assume proceeds = 3000000.0

stream fund.sale_proceeds on entity asset.fund inflow currency USD {
  schedule on 2020-01
  amount = inputs.proceeds
}

stream fund.fee_on_distributions on entity asset.fund outflow currency USD {
  schedule every year from 2020-01 to 2025-01
  amount = 0.01 * series_sum("fund.distribution.residual", 0, time.t)
}

waterfall fund.distribution on entity asset.fund {
  from available
  schedule every year from 2020-01 to 2025-01

  pay residual to party.gp = remaining
}
```

- `E1346_STREAM_READS_WATERFALL_STEP` (error): Stream 'fund.fee_on_distributions' reads series 'fund.distribution.residual', which is a waterfall step. Steps publish when their waterfall finishes, and every waterfall runs after every stream — so this read could only ever aggregate to zero.
  - hint: A step's series is visible to a later waterfall's `from` and to nothing else. Model the quantity the step pays as a stream or a field if a stream needs to read it.

Minimal fix (compiles):

```cfdl
version 0.1
model "stream-reads-waterfall-step"
time calendar annual from 2020-01 for 6

// Fix: a stream may not read a waterfall step — steps publish after every
// stream has run. The fee is based on the stream the waterfall draws from,
// which IS visible to a dependency-ordered stream.

entity asset fund : Asset.Financial
entity party lp   : Party { name = "Limited Partners" }
entity party gp   : Party { name = "General Partner" }

assume proceeds = 3000000.0

stream fund.sale_proceeds on entity asset.fund inflow currency USD {
  schedule on 2020-01
  amount = inputs.proceeds
}

stream fund.fee_on_distributions on entity asset.fund outflow currency USD {
  schedule every year from 2020-01 to 2025-01
  amount = 0.01 * series_sum("fund.sale_proceeds", 0, time.t)
}

waterfall fund.distribution on entity asset.fund {
  from available
  schedule every year from 2020-01 to 2025-01

  pay residual to party.gp = remaining
}
```

## stream_unknown_category — E5022_UNKNOWN_STREAM_CATEGORY

Failing example:

```cfdl
version 0.1
model "stream-unknown-category"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 3
entity asset tower : CRE.Asset.RealProperty

// A category has to name one of the closed set the pack declares, because a
// category is what a fold aggregates on. An unlisted one is worse than a
// missing one: the stream still reports as a line, so the statement looks
// complete while the subtotal it should have joined is quietly short.
//
// `operating_revenue` is plausible — it is what another system might call this,
// and it is one underscore away from the real `operating.revenue.other` — which
// is the point. Without E5022 it would compile clean and create a bucket
// nothing names.
stream cre.other.income on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating_revenue
  amount = 1000
}
```

- `E5022_UNKNOWN_STREAM_CATEGORY` (error): Stream 'cre.other.income' declares category 'operating_revenue', whose root segment 'operating_revenue' is not one of operating, investing, financing. A category is a path into the cash flow statement, so it has to say which section it belongs to.
  - hint: Any dotted path rooted in operating, investing or financing is valid, with or without a pack — for example `operating.revenue.rent`. A pack's category list is a recommendation, not a gate.

Minimal fix (compiles):

```cfdl
version 0.1
model "stream-unknown-category"
use pack "cre" version "0.1.0"
time calendar annual from 2026-01 for 3
entity asset tower : CRE.Asset.RealProperty

// The category names one of the pack's closed set: operating.revenue.other.
stream cre.other.income on entity asset.tower inflow currency USD {
  schedule every year from 2026-01 to 2028-01
  category operating.revenue.other
  amount = 1000
}
```

## stream_unknown_item — E0004_EXPECTED_TOKEN

Failing example:

```cfdl
version 0.1
model "stream-unknown-item"
time calendar monthly from 2026-01 for 12

entity asset co : Asset.Financial

// A stream body used to swallow anything it did not recognise. `payment net 60
// days` on its own line compiled clean and had no effect at all — the correct
// form is inline, `schedule every month net 60 from …`. The same silence hid
// every typo'd key.
stream co.receipts on entity asset.co inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  payment net 60 days
  amount = 10000
}
```

- `E0004_EXPECTED_TOKEN` (error): Unexpected keyword 'payment' in a stream body. Expected 'schedule', 'amount', 'active when', 'category', or '}'. Payment terms go inside the schedule: `schedule every month net 30 from …`.

Minimal fix (compiles):

```cfdl
version 0.1
model "stream-unknown-item"
time calendar monthly from 2026-01 for 12

entity asset co : Asset.Financial

// Fix: payment terms go inside the schedule clause, not on their own line.
stream co.receipts on entity asset.co inflow currency USD {
  schedule every month net 60 from 2026-01 to 2026-12
  amount = 10000
}
```

## term_clip_out_of_bounds — E5011_TERM_CLIP_OUT_OF_BOUNDS

Failing example:

```cfdl
version 0.1
model "term-clip-out-of-bounds"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

// degradation must be a fraction between 0 and 1; this clip reaches 1.4.
assume degr ~ Normal(mean=0.005, stdev=0.2, clip=[0.0, 1.4])

contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = 5000
    price = 3000
    escalation = 0.0
    degradation = inputs.degr
    availability = 1.0
  }
}
```

- `E5011_TERM_CLIP_OUT_OF_BOUNDS` (error): Contract 'energy.ppa.plant_a' term 'degradation' defers to input 'degr', whose clip [0, 1.4] can produce values outside the range this term allows (0 to 1). Tighten the clip.

Minimal fix (compiles):

```cfdl
version 0.1
model "term-clip-out-of-bounds"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

// Fix: degradation is a fraction, so the clip is tightened to [0, 1].
assume degr ~ Normal(mean=0.005, stdev=0.2, clip=[0.0, 1.0])

contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = 5000
    price = 3000
    escalation = 0.0
    degradation = inputs.degr
    availability = 1.0
  }
}
```

## term_expr_in_literal_slot — E5026_TERM_EXPR_IN_LITERAL_SLOT

Failing example:

```cfdl
version 0.1
model "term-expr-in-literal-slot"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

// `payment_frequency` is spliced into the rule's schedule, which is a
// frequency word, not an expression. An expression here is not late — it is
// wrong.
contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2027-12
  terms {
    principal = 6000000
    interest_rate = 0.05 + 0.005
    amortization_months = 300
    payment_frequency = 1 + 2
  }
}
```

- `E5026_TERM_EXPR_IN_LITERAL_SLOT` (error): Pack lowering rule 'cre_permanent_debt_interest' uses term 'payment_frequency' as a literal (a frequency or day count), so it cannot hold an expression; contract 'cre.permanent_debt' supplies `1 + 2`.
- `E5026_TERM_EXPR_IN_LITERAL_SLOT` (error): Pack lowering rule 'cre_permanent_debt_interest' uses term 'payment_frequency' in a slot that is not an expression (a name, date, frequency, or count), so it cannot hold an expression; contract 'cre.permanent_debt' supplies `1 + 2`.
- `E5026_TERM_EXPR_IN_LITERAL_SLOT` (error): Pack lowering rule 'cre_permanent_debt_principal' uses term 'payment_frequency' as a literal (a frequency or day count), so it cannot hold an expression; contract 'cre.permanent_debt' supplies `1 + 2`.
- `E5026_TERM_EXPR_IN_LITERAL_SLOT` (error): Pack lowering rule 'cre_permanent_debt_principal' uses term 'payment_frequency' in a slot that is not an expression (a name, date, frequency, or count), so it cannot hold an expression; contract 'cre.permanent_debt' supplies `1 + 2`.

Minimal fix (compiles):

```cfdl
version 0.1
model "term-expr-in-literal-slot"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset tower : CRE.Asset.RealProperty

// Fix: `payment_frequency` is spliced into the rule's schedule as a frequency
// word, so it cannot hold an expression. Dropped here to take the default
// (the model calendar, monthly).
contract cre.permanent_debt on entity asset.tower {
  term 2026-01..2027-12
  terms {
    principal = 6000000
    interest_rate = 0.05 + 0.005
    amortization_months = 300
  }
}
```

## term_expr_in_period_slot — E5017_PERIOD_TERM_NOT_LITERAL

Failing example:

```cfdl
version 0.1
model "term-expr-in-period-slot"
use pack "testpack" version "0.1.0"
time calendar quarterly from 2026-01 for 8

entity asset borrower : Asset.Financial

// A months-denominated term is converted into the rule's own periods at
// compile time, so it must be a literal. An expression moves the value to
// run time, where the conversion can no longer happen.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2027-10
  terms {
    amount_year = 120000
    term_months = 12 + 12
  }
}
```

- `E5017_PERIOD_TERM_NOT_LITERAL` (error): Pack lowering rule 'emit_cadence_probe' converts term 'term_months' from months into periods, so it must be a literal; contract 'test.cadence_probe' supplies `12 + 12`.

Minimal fix (compiles):

```cfdl
version 0.1
model "term-expr-in-period-slot"
use pack "testpack" version "0.1.0"
time calendar quarterly from 2026-01 for 8

entity asset borrower : Asset.Financial

// Fix: `term_months` is converted from months into periods at compile time,
// so it must be a literal — 24, not `12 + 12`.
contract test.cadence_probe on entity asset.borrower {
  term 2026-01..2027-10
  terms {
    amount_year = 120000
    term_months = 24
  }
}
```

## term_expr_invalid — E5025_TERM_EXPR_INVALID

Failing example:

```cfdl
version 0.1
model "term-expr-invalid"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

// A term may hold an expression. This exact model was the INVALID fixture
// `term_trailing_tokens`: the arithmetic used to be discarded in silence, so
// it compiled as mwh_year = 1000 — then it became a parse error — and now it
// means what it says. The results must equal the same model stating 1500.
contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = (1000 + 500
    price = 3000
    escalation = 0.0
    degradation = 0.005
    availability = 1.0
  }
}
```

- `E5025_TERM_EXPR_INVALID` (error): Contract 'energy.ppa.plant_a' term 'quantity' is an expression that does not compile [E3001_EXPR_PARSE_ERROR]: expected `)`, found end of expression

Minimal fix (compiles):

```cfdl
version 0.1
model "term-expr-invalid"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

// Fix: the expression's parenthesis is closed, so the term compiles and means
// what it says — 1500.
contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = (1000 + 500)
    price = 3000
    escalation = 0.0
    degradation = 0.005
    availability = 1.0
  }
}
```

## term_unit_mismatch — E5024_TERM_UNIT_MISMATCH

Failing example:

```cfdl
version 0.1
model "term-unit-mismatch"
use pack "energy" version "0.1.0"
time calendar annual from 2026-01 for 3

// THE TRAP THE PACK'S OWN COMMENT SPENDS A PARAGRAPH ON. The production credit
// is expressed per MWh, and 0.1 c/kWh is $1.00/MWh — so a model that thinks in
// cents states a number a hundred times too small, and the result is a
// plausible-looking credit that is wrong by two orders of magnitude.
//
// Before units, nothing noticed. The unit annotation is an assertion about what
// the number means, the rule declares the truth, and the two must agree.

entity asset plant : Energy.Asset.GenerationFacility

contract energy.ptc on entity asset.plant {
  term 2026-01..2028-01
  terms {
    quantity       = 250000 "MWh/yr"
    amount = 2.75 "c/kWh"
  }
}
```

- `E5024_TERM_UNIT_MISMATCH` (error): Contract 'energy.ptc' term 'amount' is stated in c/kWh, but the rule expresses it in USD/MWh.
  - hint: Restate the value in USD/MWh. Units are not converted: the number in the model is the number the engine uses.

Minimal fix (compiles):

```cfdl
version 0.1
model "term-unit-mismatch"
use pack "energy" version "0.1.0"
time calendar annual from 2026-01 for 3

// Fix: the credit is stated in the rule's own unit — 2.75 c/kWh is
// 27.50 USD/MWh.

entity asset plant : Energy.Asset.GenerationFacility

contract energy.ptc on entity asset.plant {
  term 2026-01..2028-01
  terms {
    quantity       = 250000 "MWh/yr"
    amount = 27.50 "USD/MWh"
  }
}
```

## term_unknown_input — E5010_TERM_UNKNOWN_INPUT

Failing example:

```cfdl
version 0.1
model "term-unknown-input"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

assume annual_yield = 5000

// A term naming an input that was never declared is an error, so a typo
// cannot resolve to nothing at runtime.
contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = inputs.anual_yield
    price = 3000
    escalation = 0.0
    degradation = 0.005
    availability = 1.0
  }
}
```

- `E5010_TERM_UNKNOWN_INPUT` (error): Contract 'energy.ppa.plant_a' term 'quantity' references input 'anual_yield', which is not declared. Add `assume anual_yield = <value>` or `assume anual_yield ~ <Dist>(...)`.

Minimal fix (compiles):

```cfdl
version 0.1
model "term-unknown-input"
use pack "energy" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity asset plant : Energy.Asset.GenerationFacility

assume annual_yield = 5000

// Fix: the term names the declared input — `annual_yield`, not the typo
// `anual_yield`.
contract energy.ppa.plant_a on entity asset.plant {
  term 2026-01..2026-12
  terms {
    quantity = inputs.annual_yield
    price = 3000
    escalation = 0.0
    degradation = 0.005
    availability = 1.0
  }
}
```

## unknown_field_read — E1131_UNKNOWN_FIELD_READ

Failing example:

```cfdl
version 0.1
model "unknown-field-read"
time calendar annual from 2026-01 for 3

// A MISSPELLED FIELD IS A TYPO, NOT AN ABSENCE.
//
// Field paths resolve through the `entity` root, which is open-world by design:
// a lifecycle status may not exist until an event writes it, so a guard reading
// one has to evaluate before that happens.
//
// Declared fields are not like that. The model states them, so the compiler
// knows them, and a name that is not among them is a mistake. Left open-world
// it reads as null — and null in arithmetic becomes zero, which is a wrong
// number with nothing to see.

entity asset tlb : Asset.Financial {
  balance init 275.0 next max(0.0, prev - 25.0)
}

stream credit.repay on entity asset.tlb outflow currency USD {
  schedule every year from 2027-01 to 2028-01
  amount = asset.tlb.blance
}
```

- `E1131_UNKNOWN_FIELD_READ` (error): Stream 'credit.repay' reads 'asset.tlb.blance', which that entity does not declare.
  - hint: Declare the field on the entity, or correct the name. Unrejected this reads as null and becomes zero in arithmetic.

Minimal fix (compiles):

```cfdl
version 0.1
model "unknown-field-read"
time calendar annual from 2026-01 for 3

// Fix: the stream reads the field the entity declares — `balance`, not the
// typo `blance`.

entity asset tlb : Asset.Financial {
  balance init 275.0 next max(0.0, prev - 25.0)
}

stream credit.repay on entity asset.tlb outflow currency USD {
  schedule every year from 2027-01 to 2028-01
  amount = asset.tlb.balance
}
```

## unknown_time_read — E1133_UNKNOWN_TIME_READ

Failing example:

```cfdl
version 0.1
model "unknown-time-read"
time calendar monthly from 2026-01 for 12

entity asset a : Asset.Real

// `time.` is a CLOSED vocabulary — t, date, days_in_period, phase, ppy — so a
// miss is a typo. Unrejected this evaluated to zero every period and the run
// still reported ok.
stream a.s on entity asset.a inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 1000 * time.elapsed_years
}
```

- `E1133_UNKNOWN_TIME_READ` (error): Stream 'a.s' reads 'time.elapsed_years', which is not a time binding.
  - hint: The bindings are `time.t`, `time.date`, `time.days_in_period`, `time.phase`, `time.ppy`. Unrejected this evaluates to zero and the run still reports ok.

Minimal fix (compiles):

```cfdl
version 0.1
model "unknown-time-read"
time calendar monthly from 2026-01 for 12

entity asset a : Asset.Real

// The binding is time.t (elapsed periods); years elapsed is time.t / time.ppy.
stream a.s on entity asset.a inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 1000 * (time.t / time.ppy)
}
```

## unresolved_contract_subject_ref — E1301_UNRESOLVED_ENTITY_REF

Failing example:

```cfdl
version 0.1
model "unresolved-contract-subject-ref"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset property : CRE.Asset.RealProperty

contract cre.revenue_line on entity real_estate.missing {
  term 2026-01..2027-12
  terms {
    amount = 30000
  }
}
```

- `E1301_UNRESOLVED_ENTITY_REF` (error): Contract 'cre.revenue_line' references unknown entity 'real_estate.missing'.

Minimal fix (compiles):

```cfdl
version 0.1
model "unresolved-contract-subject-ref"
use pack "cre" version "0.1.0"
time calendar monthly from 2026-01 for 24

entity asset property : CRE.Asset.RealProperty

// Fix: the contract's subject names the declared entity, asset.property.
contract cre.revenue_line on entity asset.property {
  term 2026-01..2027-12
  terms {
    amount = 30000
  }
}
```

## unresolved_entity_ref — E1301_UNRESOLVED_ENTITY_REF

Failing example:

```cfdl
version 0.1
model "unresolved-entity"
time calendar monthly from 2026-01 for 12
entity asset borrower : Asset.Financial
stream debt.principal on entity lease.missing
```

- `E1301_UNRESOLVED_ENTITY_REF` (error): Stream 'debt.principal' references unknown entity 'lease.missing'.

Minimal fix (compiles):

```cfdl
version 0.1
model "unresolved-entity"
time calendar monthly from 2026-01 for 12
entity asset borrower : Asset.Financial
// Fix: the stream names the declared entity, asset.borrower (plus the
// minimal body a stream requires: a schedule and an amount).
stream debt.principal on entity asset.borrower outflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = 100
}
```

## waterfall_no_schedule — E1348_WATERFALL_NO_SCHEDULE

Failing example:

```cfdl
version 0.1
model "waterfall-no-schedule"
time calendar annual from 2020-01 for 5

// A WATERFALL MUST SAY WHEN IT DISTRIBUTES.
//
// The schedule is not decoration on a distribution; it is half of what the
// distribution says. Between its scheduled periods a waterfall's pot
// ACCUMULATES — the cash builds in the account and is split when the date
// arrives — so "every quarter" and "once at exit" are different deals, not
// two spellings of one.
//
// The omission used to compile. It lowered to `on <time.start>`: one
// distribution, in the first period, of whatever that period happened to
// produce. A pot of 500 across five periods paid 500, 0, 0, 0, 0 and said
// nothing about the 2,000 it never distributed. The engine believed the
// opposite — no schedule meant every period — but the compiler never let that
// branch run, so two components disagreed and the silent one won.
//
// There is no default that is right often enough to be silent, so the author
// states it. `fixtures/valid/waterfall_nested_split` shows both shapes.

entity asset fund : Asset.Financial
entity party lp   : Party { name = "Limited Partners" }
entity party gp   : Party { name = "General Partner" }

stream fund.operating_cash on entity asset.fund inflow currency USD {
  schedule every year from 2020-01 to 2024-01
  amount = 500.0
}

waterfall fund.distribution on entity asset.fund {
  from available

  pay residual to party.gp = remaining
}
```

- `E1348_WATERFALL_NO_SCHEDULE` (error): Waterfall 'fund.distribution' does not say when it distributes.
  - hint: Add a `schedule` — `schedule on <date>` for a single distribution (an exit), `schedule every <period> from <date> to <date>` for a recurring one. Between its scheduled periods the pot accumulates.

Fix: not yet recorded.

## waterfall_series_reads_own_step — E1342_WATERFALL_SERIES_NOT_VISIBLE

Failing example:

```cfdl
version 0.1
model "waterfall-series-reads-own-step"
time calendar annual from 2020-01 for 6

// A STEP MAY NOT READ ITS OWN WATERFALL'S PAYMENTS.
//
// The shortfall spelling below is the one an author reaches for: pay the
// preferred return, less whatever this step has already paid. It reads as
// arithmetic and it is not — a waterfall's steps publish when the waterfall
// finishes, so at the moment this step runs there is nothing under that name
// to read. `series_sum` answers zero for a name it cannot see, the subtraction
// takes nothing away, and the step pays the full preferred every period it
// runs. Six periods, six full payments, no diagnostic.
//
// A step is a pure function of the pot: accept it, allocate, move on. What the
// shortfall spelling actually wants is a BALANCE — a quantity the distribution
// draws down and a field carries forward — not the account reconstructing its
// own postings.
//
// Reading an EARLIER waterfall is the documented composition and still
// compiles; `fixtures/valid/waterfall_nested_split` pins that.

entity asset fund : Asset.Financial
entity party lp   : Party { name = "Limited Partners" }
entity party gp   : Party { name = "General Partner" }

assume called_capital = 10000000.0
assume pref_rate      = 0.08
assume proceeds       = 3000000.0

stream fund.sale_proceeds on entity asset.fund inflow currency USD {
  schedule on 2020-01
  amount = inputs.proceeds
}

waterfall fund.distribution on entity asset.fund {
  from available
  schedule every year from 2020-01 to 2025-01

  pay preferred to party.lp = inputs.called_capital * inputs.pref_rate
                                - series_sum("fund.distribution.preferred", 0, time.t)
  pay residual  to party.gp = remaining
}
```

- `E1342_WATERFALL_SERIES_NOT_VISIBLE` (error): Waterfall 'fund.distribution' step 'preferred' reads series 'fund.distribution.preferred', which this waterfall has not finished paying.
  - hint: A step is a pure function of the pot: accept, allocate, move on. Read an earlier step's payment this period with `paid.<step>`; for a running total, carry the quantity as a balance a field advances and the distribution moves.

Minimal fix (compiles):

```cfdl
version 0.1
model "waterfall-series-reads-own-step"
time calendar annual from 2020-01 for 6

// Fix: a step may not read its own waterfall's payments — a step is a pure
// function of the pot. The shortfall subtraction is dropped; tracking what
// has already been paid is a balance a field carries forward, not the account
// reconstructing its own postings.

entity asset fund : Asset.Financial
entity party lp   : Party { name = "Limited Partners" }
entity party gp   : Party { name = "General Partner" }

assume called_capital = 10000000.0
assume pref_rate      = 0.08
assume proceeds       = 3000000.0

stream fund.sale_proceeds on entity asset.fund inflow currency USD {
  schedule on 2020-01
  amount = inputs.proceeds
}

waterfall fund.distribution on entity asset.fund {
  from available
  schedule every year from 2020-01 to 2025-01

  pay preferred to party.lp = inputs.called_capital * inputs.pref_rate
  pay residual  to party.gp = remaining
}
```

## Registered codes with no example yet

Documented in docs/08 §7, awaiting a minimal failing fixture:

- `E1001_DUPLICATE_ENTITY` — two entities share a name.
- `E1005_DUPLICATE_ASSUME` — two assumptions share a name.
- `E1101_MISSING_VERSION` — no `version` declaration. It states which language version the model is written against.
- `E1102_MISSING_MODEL` — no `model` declaration, so the model has no name.
- `E1104_MULTIPLE_VERSION` — `version` is declared more than once.
- `E1105_MULTIPLE_MODEL` — `model` is declared more than once.
- `E1106_MULTIPLE_TIME` — `time` is declared more than once. A model has one timeline.
- `E1107_MULTIPLE_USE_PACK` — more than one `use pack`. A model draws contracts from a single pack.
- `E1108_USE_PACK_NOT_IN_MODEL_FILE` — `use pack` appears in an imported file rather than the model's own. The pack applies to the whole model, so it is declared where the model is.
- `E1123_PREV_OUTSIDE_NEXT` — `prev` names a recurrence's own previous value and
- `E1125_NO_STATE_NAMESPACE` — an expression reads `state.<name>`. There is no
- `E1304_UNRESOLVED_OPTION_REF` — an event exercises an option that is not declared.
- `E1306_INVALID_ENTITY_REF_FORMAT` — entity ref, stream name, or contract name is not a qualified name with at least two segments (dotted hierarchy).
- `E1310_ENTITY_BLOCK_WITHOUT_TYPE` — an entity uses a block but declares no type, so there is nothing to check the block against.
- `E1312_MISSING_REQUIRED_FIELD` — an entity omits a field its type requires.
- `E1314_UNKNOWN_PARENT_ENTITY` — `part of` names an entity that is not declared. Hierarchy is optional; a declared parent is not.
- `E1315_ENTITY_PART_OF_ITSELF` — an entity is its own parent.
- `E1317_TYPE_HAS_NO_LIFECYCLE` — an entity declares a starting state but its type has no lifecycle.
- `E1320_UNKNOWN_PARTY_ENTITY` — a contract or option binds a role to an entity that is not declared.
- `E1321_NOT_A_PARTY` — a role is bound to an asset. A contract is between parties.
- `E1330_CONFLICTING_ACTIVE_CLAUSES` — a stream declares both `active when` and `active in state`. Use one: `active in state` for a lifecycle state, `active when` for anything else.
- `E1331_OWNER_HAS_NO_LIFECYCLE` — a stream is active in a lifecycle state but its owner's type declares no lifecycle.
- `E1340_WATERFALL_NO_SOURCE` — a waterfall declares no `from`, so there is no
- `E1341_WATERFALL_FORWARD_REF` — a step's `paid.<step>` names a step declared
- `E1343_WATERFALL_DUPLICATE_STEP` — two steps in one waterfall share a name,
- `E1344_WATERFALL_NO_REMAINDER` — a waterfall never says where the remainder
- `E1345_WATERFALL_STEP_NO_AMOUNT` — a step says nothing about what it pays.
- `E1347_UNRESOLVED_ACCOUNT_REF` — a step allocates `to account <name>` and no
- `E1366_DUPLICATE_STATEMENT` — two statements share a name. Same rule as a metric and a slice: one name, one presentation.
- `E1368_STATEMENT_UNKNOWN_REFERENCE` — a statement filters by a slice, or shows a metric, that the model does not declare. A presentation that silently shows nothing is the failure §7.55 exists to end.
- `E2002_CONTRACT_MISSING_EFFECTS` — a contract produces no streams, so it has no effect on the model. Under a pack that declares contract types, a contract no rule lowers is a type the pack does not declare and is reported as `E1373` instead.
- `E2101_STREAM_MISSING_SCHEDULE` — a stream has no `schedule`, so there is no period for its cash to land in.
- `E2102_STREAM_MISSING_AMOUNT` — a stream has no `amount`.
- `E2104_SCHEDULE_INVALID_RANGE` — a schedule's `to` is before its `from`.
- `E2105_SCHEDULE_INVALID_DAY_OF_MONTH` — a day rule names a day outside 1–31.
- `E2106_SCHEDULE_PHASE_NOT_FOUND` — a schedule is anchored to a phase that is not declared.
- `E2301_ASSUME_UNKNOWN_DIST` — a random assumption names a distribution that
- `E2302_ASSUME_INVALID_PARAM` — a distribution parameter is not a number, or
- `E2303_ASSUME_MISSING_PARAM` — a distribution is missing a parameter it
- `E2304_ASSUME_INVALID_CLIP` — a `clip=[lo, hi]` is malformed or inverted.
- `E2401_OPTION_MISSING_EXERCISE` — an option declares no `exercise when`, so
- `E2402_OPTION_MISSING_PAYOFF` — an option declares no `payoff`, so exercising
- `E4004_MISSING_PACK` — the named pack could not be loaded — not found, or found and rejected.
- `E5002_IR_SCHEMA_VALIDATION_FAILED` — the IR the compiler produced does not satisfy the published IR schema, or the IR being read does not.
- `E5003_IR_EMIT_FAILED` — the IR could not be written.
- `E5004_INVALID_LOWERING_RULE` — a pack's lowering rule is malformed.
- `E5005_PHASE_NOT_FOUND` — a lowering rule anchors to a phase the model does not declare.
- `E5007_DUPLICATE_LOWERED_STREAM` — two contracts lower to the same stream name. Give one a suffix.
- `E5009_LOWERED_EXPR_INVALID` — a pack lowering rule expanded to an amount
- `E5012_RULE_INVALID_INTERVAL` — a lowering rule's `schedule_every` is not
- `E5013_PACK_CADENCE_UNSUPPORTED` — the model's calendar is not one the pack
- `E5020_LOWERED_FIELD_INVALID` — a pack lowering rule expanded to a field
- `E5021_DUPLICATE_LOWERED_FIELD` — two contracts lower to one field name with
- `E5023_SUBTOTAL_UNKNOWN_CATEGORY` — a pack subtotal folds a category no rule
- `E6002_CRE_LEASE_INVALID_TERM_RANGE` — 
- `E6003_CRE_LEASE_UP_MISSING_MONTHS` — 
- `E6010_CRE_EXIT_MISSING_EXIT_CAP` — 
- `E6012_CRE_EXIT_MISSING_NOI_VALUE` — 
- `E6020_CRE_OPS_MISSING_AMOUNT` — 
- `E6030_CRE_LEASE_AMBIGUOUS_RENT` — a CRE lease states both `base_rent` (per period) and `base_rent_year` (annual). They would be summed; give one.
- `E6033_CRE_UNIT_INVALID_ESCALATION` — a lease unit's `escalation` is below -1, which would make rent negative on the first step.
- `E6054_CRE_DEBT_INVALID_AMORT` — `amortization_months` strikes the payment and is
- `E6055_CRE_DEBT_INVALID_IO_MONTHS` — whole months, 0 or more
- `E6056_CRE_DEBT_INVALID_BALLOON_FLAG` — `balloon_at_maturity` is 0 or 1
- `E6057_CRE_CONSTRUCTION_INVALID_EQUITY_COMMITMENT` — zero or greater; zero is
- `E6058_CRE_CONSTRUCTION_INVALID_RATE` — a nominal annual rate in [0, 1], which
- `E6059_CRE_CONSTRUCTION_INVALID_DRAW_ACCRUAL_FRACTION` — where in the period a
- `E6060_CRE_CONSTRUCTION_INVALID_TERM_RANGE` — the build must sit inside the
- `E6061_CRE_OPEX_LINE_MISSING_AMOUNT` — an operating expense line states
- `E6062_CRE_OPEX_LINE_PCT_FIXED_RANGE` — the fixed SHARE, in [0, 1]; catches 81
- `E6063_CRE_OPEX_LINE_OCCUPANCY_RANGE` — a ratio of occupied space, in [0, 1];
- `E6064_CRE_REVENUE_LINE_MISSING_AMOUNT` — a revenue line states `amount` or
- `E6065_CRE_CONSTRUCTION_INVALID_CAPITALIZE_INTEREST` — a construction loan's
- `E6067_CRE_PCT_RENT_INVALID_OVERAGE_PCT` — a fraction between 0 and 1.
- `E7003_OPCO_LINE_INVALID_GROWTH` — 
- `E7011_OPCO_TAXES_AMBIGUOUS_DA` — OpCo cash taxes state both `da_monthly` (per period) and `da_year` (annual). They would be summed; give one.
- `E7012_OPCO_TAXES_MISSING_RATE` — a cash-taxes contract states neither
- `E7013_OPCO_WC_MISSING_AMOUNT_OR_RULE` — 
- `E7014_OPCO_WC_INVALID_SCHEDULE` — 
- `E7020_OPCO_EXIT_MISSING_MULTIPLE` — 
- `E7022_OPCO_EXIT_MISSING_BASE_VALUE` — 
- `E7023_OPCO_EXIT_INVALID_SCHEDULE` — 
- `E7024_OPCO_EXIT_EBITDA_INVALID_MULTIPLE` — 
- `E7025_OPCO_PERPETUITY_RATE_NOT_ABOVE_GROWTH` — a growing perpetuity needs
- `E7026_OPCO_PERPETUITY_MISSING_BASE_VALUE` — the terminal-period flow the
- `E7027_OPCO_PERPETUITY_MISSING_DISCOUNT_RATE` — the terminal capitalization
- `E7028_OPCO_PERPETUITY_MISSING_GROWTH` — state 0 for a flat perpetuity.
- `E7029_OPCO_PERPETUITY_INVALID_SELLING_COSTS` — a fraction between 0 and 1.
- `E7030_OPCO_DEBT_INVALID_AMORT` — 
- `E7031_OPCO_DEBT_INVALID_RATE` — 
- `E8003_ENERGY_INVALID_ESCALATION` — 
- `E8004_ENERGY_INVALID_PRICE_ESCALATION` — 
- `E8020_ENERGY_DEBT_INVALID_RATE` — 
- `E8021_ENERGY_DEBT_INVALID_TERM_MONTHS` — 
- `E8022_ENERGY_DEBT_INVALID_PRINCIPAL` — 
- `E9001_CREDIT_INVALID_BALANCE` — 
- `E9002_CREDIT_INVALID_RATE` — 
- `E9003_CREDIT_INVALID_TERM_MONTHS` — 
- `E9014_CREDIT_INVALID_SERVICING_FEE` — 
- `E9015_CREDIT_INVALID_PREPAY_PENALTY` — 
- `E9016_CREDIT_INVALID_PSA_SPEED` — `psa_speed` is a MULTIPLE of the standard
- `E9017_CREDIT_INVALID_SDA_SPEED` — `sda_speed` is a multiple of the standard
- `E9018_CREDIT_INVALID_ABS_SPEED` — `abs_speed` is the Absolute Prepayment
- `E9019_CREDIT_INVALID_AGE_MONTHS` — `age_months` is the pool's weighted
- `E9020_CREDIT_RATE_FLOOR_ABOVE_CAP` — 
- `W3001_EXPR_TYPE_UNKNOWN` — an expression's type could not be determined ahead of evaluation. It still runs; the warning notes the check was skipped.
- `W3002_OBS_REF_EXTRACTION_FAILED` — an observation reference could not be read out of an expression, so the run may not know it needs that input.
- `W3500_STATEMENT_UNCLASSIFIED_STREAM` — cash that no row of the statement
- `W3501_STATEMENT_STREAM_DOUBLE_COUNTED` — a stream claimed by more than one
- `W3502_STATEMENT_BOTTOM_LINE_RESIDUAL` — the statement's rows do not sum to
- `W3503_STATEMENT_UNKNOWN_STRUCTURE` — a model-declared statement asks for a
- `W5022_UNKNOWN_SERIES_REFERENCE` — a series reduction (`series_sum`,
- `W5023_UNRECOGNISED_PACK_CATEGORY` — a stream's category is well-rooted and
