## The case

The exit waterfall of a sponsor buyout: an accruing convertible preferred, a
management rollover, and a seven-tranche management stock option pool with
strikes laddered from $12.50 to $25.00.

An option tranche pays if it is in the money — if the exit consideration per
share exceeds its strike. But exercising a tranche adds both its strike proceeds
*and* its shares to the pool, which moves the value per share. So which options
exercise depends on the value per share, and the value per share depends on which
options exercise.

## The reference

A seven-step leveraged buyout teaching model published as a downloadable
spreadsheet, free and without registration. Its waterfall sheet publishes the
exit enterprise and equity value at six exit multiples, the in-the-money test per
tranche, option proceeds, share counts, the resulting value per share, and the
split of proceeds between sponsor and management.

**Not redistributable.** The workbook carries an "All Rights Reserved" notice, so
it is neither vendored nor wired into the test suite.

## What it exercises

| | |
|---|---|
| Pack | `opco` |
| Declared | seven options, two states, four native streams |
| Language features | options with an exercise test and a payoff; declared state read by an option guard |
| Conventions | a preferred accruing 8% and converting one-for-one, a management rollover, laddered option strikes, dilution at exit |

Two tranches are out of the money, so the case asserts a non-exercise as well
as an exercise.

## The result

Value per share **20.877119**, sponsor proceeds **487.546139**, management
proceeds **132.569706** — each matching the published figure exactly. Option
intrinsic value comes to $12.912m across the five exercised tranches.

Verified at all six published exit multiples; the exercising set is unique at
every one.

## The delta

None.

The circularity resolves without iteration because **the strikes are ordered**:
if a $20.00 option is in the money then so is every cheaper one, so any
exercising set is a prefix of the tranches sorted by strike. That reduces 128
possible subsets to eight candidates, exactly one of which is self-consistent —
a finite ordered test rather than a search.
