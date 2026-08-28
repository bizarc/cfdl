# Trap the cash, then let it go

The starter's deal pays its residual straight through — even in the month after collections failed. Add the trap.

1. Add a `trapped` step above the residual. Divert `remaining` into the trap account while `asset.suite.status == "trapped"`.
2. Add a `release` waterfall drawing `from trap`. Pay the balance to the sponsor when the status is `"normal"` again.

Predict before you run:

- The machine reads *settled* rent, so the breach at month 3 traps month 4's cash, not month 3's.
- The trap balance holds exactly one month's rent, for exactly one month.

Then check the series:

- `account.trap` reads `0, 0, 0, 0, 100, 0, …` — funded at the breach, drained at the cure.
- `release.released` pays the 100 in month 5, the first `normal` month.
- No dollar is lost: the sponsor's lifetime total is the same 1,100 the rent produced.

The trap changed *when* the sponsor was paid, never *whether*. Timing is the whole construct.
