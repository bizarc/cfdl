# Auto ABS note classes — maintainer's notes

The published write-up is `CASE.md`. This file carries what a maintainer
needs and a reader of the site does not.

## How the model is put together

- The trust is `entity container trust : Container.SPV`, and the 43
  sub-pools are `part of container.trust`. The container's roll-up is a
  consolidation of its pools' cash and answers no structural question;
  the pool's remaining principal is the pack's own statement subtotal,
  `domain.credit.balance_outstanding`, which this case reports and does
  not need for any test (a no-loss deal has no overcollateralization test to
  run). A balance readable in the causal plane — the thing a clean-up call
  or an OC test would guard on — is not published by the relation today;
  `docs/13` §7.98.
- Two structure-owned accounts carry the two amounts the indenture defines
  on each distribution date. `interest_collections` is the pool interest
  streams plus the pack's negative servicing streams plus the trust's
  fee stream (`credit.trust.fees`, the administration fee); `principal_collections` is scheduled principal plus
  prepayments. One account fed with everything was tried first and is
  wrong: interest left over after the coupons falls through to the senior
  principal step, and A-2 is paid $1.1m of interest as principal at the
  first distribution. The published grid shows the issuer repays principal
  from principal collected only, which is what the two accounts say.
- Each class is a `credit.note` on the trust: a face, a coupon, and
  `principal_account` naming the holder's account. The note lowers its
  claim (`face − prev.<class>_principal`) and the interest due on it as
  fields on the trust, and each step pays one of them and names the note
  and line it pays, so the results attribute every allocation to its class.
  Each holder OWNS the account that receives its principal, so
  `prev.<class>_principal` is the class's cumulative repayment. Each class also has a
  structure-owned `<class>_interest` account, because a party may own at
  most one account (`docs/01` §10.6). That limitation is wrong for a
  noteholder, who has a principal position and an interest position, and
  is recorded as a language item rather than worked around silently.
- `prev.<account>` is absent at period 0, not zero; the note's claim field
  starts at the face (`init`) and reads the account from the second
  distribution on (`next`), so no step needs a first-period guard.
- Every step is `min(remaining, claim)`. On the principal waterfall that
  is what makes the pay-down sequential; on the interest waterfall it is
  also what satisfies `E1344` (a waterfall must say where the remainder
  goes) without a residual step. Nothing is paid out of the excess: the
  interest beyond the coupons and the $13.75m of overcollateralization stay
  in the two collection accounts, which is a stated choice rather than an
  omission — the exhibit names no certificateholder.
- Class A-1 is declared at a face of zero. It was paid in full in January
  2018 and the exhibit does not tabulate it; declaring it keeps the note
  stack the exhibit describes, and costs one party, two accounts and two
  zero-paying steps.

## What the fees do to the totals

The exhibit assumes a 1.00% per annum servicing fee and a $1,500 monthly
administration fee. Both are stated as one trust-level stream each —
`credit.trust.servicing_fee`, 1.00%/12 of the pool balance carried into the
month (the initial pool less principal collected to date), and
`credit.trust.admin_fee`. The pack can lower servicing per sub-pool
(`servicing_fee` on `credit.loan`, default 0); stated per pool
the 43 streams sum to the same 9,395,813.31 to the cent, and the single
trust-level stream is the deal's own statement of the fee. Neither the
servicer nor the administrator is modeled as a payee: the fees leave the
trust's cash and the notes never see them, which is all the grid depends
on. Neither fee touches a principal column, so the six payment columns and
the six balance columns are unaffected. `model.total` falls by their sum:

```
servicing, 1.00% on the amortizing pool     9,395,813.31
administration, 64 × 1,500                     96,000.00
model.total  589,606,387.86  →  580,114,574.55
```

`model.total` is a regression anchor from this model, so
`expected_metrics.json` carries the new figure.

## The balance columns in `expected.csv`

The six `account.<class>_principal` columns are the exhibit's own
percent-outstanding column at 0% ABS, restated as dollars of cumulative
principal: `face × (1 − pct / 100)`, per class per distribution date. The
grid stops listing a class once it retires; those periods are the face in
full. The payment columns are unchanged from the differenced grid the case
always asserted. To regenerate the balance columns:

```python
import csv
faces = {"A-2": 112026644.00, "A-3": 271370000.00, "A-4": 86010000.00,
         "B": 22220000.00, "C": 18510000.00, "D": 13750000.00}
pct = {}
for row in csv.DictReader(open("published_grid.csv")):
    d = row["distribution_date"]
    if d == "Certificate Closing Date" or d.startswith("WAL"):
        continue
    pct.setdefault(row["class"], []).append(float(row["abs_0.00"]))
# period t of class c: faces[c] * (1 - pct[c][t] / 100), or faces[c] once
# the grid stops listing the class
```

Tolerances on the balance columns equal the payment bands (face × 1e-4):
the print floor of a whole-cent percentage is face × 5e-5, and three tail
cells sit 0.0003–0.0005 points above it, as CASE.md records.

## What this case does not cover

Losses, overcollateralization, triggers, the reserve, and the clean-up call
— by the exhibit's own assumptions, and because the to-call columns are not
asserted. The seven prepayment speeds the exhibit publishes are one case
per speed today (`auto_abs_speed_050`, `auto_abs_speed_150`); making them
scenarios of this one needs per-period scenario assertions,
`docs/13` §7.23.
