## The case

A 100 MW-ac solar project financed through a tax-equity partnership. A tax
investor funds 98% of the equity and takes 98% of the cash, the depreciation
and the investment credit. When its after-tax return reaches 8%, the structure
**flips**: the investor drops to 5% and the sponsor takes the rest.

The flip is not a date. It is a test, and when it lands depends on how the
project performs — so the model derives it rather than stating it.

## The reference

A national laboratory's open-source project-finance model, run in its
leveraged-partnership-flip configuration. It publishes the flip year alongside
both partners' cash, so the derived date is checkable and not only the split.

**Not vendored.** The tool was run once outside the repository and only its
output numbers were carried across.

## What it exercises

| | |
|---|---|
| Pack | `energy` |
| Declared | two typed assets, two parties, three states, one event, one waterfall |
| Language features | **a declared lifecycle**, **an event whose guard is a computed value**, the transition log, a waterfall reading its owner's state |
| Conventions | 98/2 pre-flip and 5/95 post-flip sharing, an investment credit at 30%, MACRS on a basis reduced by half the credit, level-pay debt |

The lifecycle sits on the partnership **interest** rather than the plant: the
panels do not change when the structure flips, the claim on their cash does.

**The test needs no solver.** The criterion is an internal rate of return
reaching 8%, which this language cannot compute mid-model. It does not need to
— at a fixed hurdle the two statements are one:

    IRR through period n >= 8%   <=>   NPV at 8% through period n >= 0

A discounted running sum is a recurrence, so the test is arithmetic evaluated
once a period. Nor can it be circular: the test at period *t* reads flows
through *t-1*, and every one of those periods is still pre-flip by
construction, so the sharing percentages it depends on are settled before it
is evaluated.

## The result

**The flip date is derived, and it agrees.** The transition fires at period 4
against the reference's stated flip in year 3 — the same instant under the
deal's own convention, where the year's books close, the return is tested, and
the new sharing applies to the year that follows.

Both partners' cash reproduces across all 25 operating periods, through the
flip and through the debt cliff at periods 18 and 19.

Asserted: two columns period by period, plus the transition itself.

## The delta

Worst disagreement across all 25 periods and both columns: **1.0e-6 dollars**,
which is the engine's own publication precision rather than a convention
difference.

Period 0 is not asserted. The reference books a sponsor development fee at
close that this case does not model, and it has no bearing on the flip.

## A variant the reference does not publish

The same deal on a **monthly** grid flips ten months earlier, in the second
month of year 3.

By the end of year 2 the investor is $445,000 short of its hurdle, and two
months of operating cash clear it — but an annual grid has no period between
month 24 and month 36 in which to notice, so the event cannot fire until the
next year end. The investor keeps 98% of the cash for ten months it was no
longer entitled to, worth about $3.5m here.

The grid is therefore an economic assumption whenever an event decides who
gets paid, not a presentation choice. No external source publishes the monthly
answer, so it is carried as a fixture rather than as a benchmark.
