# Source

**Fannie Mae REMIC Trust 2019-2.** Guaranteed REMIC Pass-Through Certificates,
$307,727,958. Dealer J.P. Morgan Securities LLC. Priced 24 January 2019, settled
30 January 2019. Distributions monthly on the 25th, commencing February 2019.

Two documents, both published by Fannie Mae:

- the **Prospectus Supplement dated 24 January 2019**, which specifies this
  series; and
- the **Single-Family REMIC Prospectus dated 1 November 2018**, the base
  prospectus it supplements, which supplies the class-type definitions, the
  interest and notional-balance conventions, and the definition of PSA.

## The series

Three groups, eight classes.

| group | class | original balance | principal type | interest type | rate |
|---|---|---:|---|---|---|
| 1 | CD | 124,676,530 | SC/PT | FIX | 3.25% |
| 1 | CI | 51,004,035 *notional* | NTL | FIX/IO | 5.50% |
| 2 | EA | 34,678,994 | SC/PT | FIX | 3.25% |
| 2 | EI | 12,137,647 *notional* | NTL | FIX/IO | 5.00% |
| 3 | AB | 148,372,434 | PT | FIX | 3.25% |
| 3 | IO | 51,930,352 *notional* | NTL | FIX/IO | 5.00% |
| — | R, RL | 0 | NPR | NPR | 0 |

Groups 1 and 2 are **Structured Collateral** — their assets are seventeen REMIC
and RCR certificates from Fannie Mae deals issued 2002–2006, listed in Exhibit A
of the supplement. Group 3 is backed directly by mortgage-backed securities and
is the group modelled here.

## Group 3

**Characteristics of the Group 3 MBS** (page S-4):

| | |
|---|---:|
| Approximate principal balance | $148,372,434 |
| Pass-through rate | 5.00% |
| Range of weighted average coupons | 5.25% to 7.50% |
| Range of weighted average remaining terms | 150 to 360 months |

**Assumed characteristics of the underlying mortgage loans** (page S-4),
statistical information as of 1 January 2019:

| | |
|---|---:|
| Principal balance | $148,372,434 |
| Original term to maturity | 360 months |
| Remaining term to maturity | 173 months |
| Loan age | 175 months |
| Interest rate | 5.451% |

The 0.451% between the 5.451% loan coupon and the 5.00% pass-through rate is the
servicing and guaranty strip, which is why the model carries it as
`servicing_fee`: what reaches the trust must be 5.00% for the class coupons to
reconstruct it.

**Distributions of principal** (page S-9), quoted in full:

> The Group 3 Principal Distribution Amount to AB until retired.
>
> The "Group 3 Principal Distribution Amount" is the principal then paid on the
> Group 3 MBS.

**Notional classes** (page S-5): the notional principal balance of IO equals
**35.0000000674% of the AB Class**, measured immediately before the related
distribution date.

**Distributions of interest** (page S-8): interest paid on each certificate on a
distribution date is "one month's interest on the outstanding balance of that
Certificate immediately prior to that Distribution Date". All interest-bearing
classes are Delay Classes.

## What is asserted

The **decrement table** on page S-14 — "Percent of Original Principal Balances
Outstanding", AB and IO Classes — at **198% PSA**, the pricing speed of the seven
published (0%, 100%, 198%, 300%, 400%, 700%, 1000%).

It states, for each January from 2020 to 2049, the percentage of the original
principal balance (for IO, the original notional balance) still outstanding after
that month's distribution. Thirty dates for each of two classes, all asserted in
`expected.csv`, together with the weighted average life of 4.7 years published in
the same table.

The percentages are whole numbers, and the table marks with an asterisk any
balance "greater than 0% and less than 0.5%". That rounding is the whole of the
tolerance in `case.toml`: half a percent of each class's original balance.

The interest legs are asserted from the same table — a published balance times a
coupon stated on the cover — and the residual step is asserted at zero, which is
the strip identity.

## Structuring assumptions

Stated on page S-9 as the "Pricing Assumptions":

1. The mortgage loans underlying the Group 3 MBS have the assumed
   characteristics quoted above.
2. The mortgage loans prepay at the constant percentages of PSA specified in the
   related tables.
3. The settlement date for the certificates is 30 January 2019.
4. Each distribution date occurs on the 25th day of a month.

Under the **0% PSA** column only, the loans are instead assumed to have an
original and remaining term of 360 months and to bear interest at 7.50%. That
column is not the one this case takes.

PSA itself is defined in the base prospectus (page 32): 100% PSA is an annual
prepayment rate of 0.2% in the first month after origination, rising 0.2% each
month to 6% at month 30 and constant thereafter.

Fannie Mae guarantees that required payments of principal and interest are
available for distribution on time, so no loss assumption arises. The
certificates are not guaranteed by the United States.
