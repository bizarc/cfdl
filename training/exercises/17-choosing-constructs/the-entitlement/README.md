# Where the entitlement lives

The management fee starves as collections shrink — 60, 50, 40, 30 against 30 of senior debt and 15 of fee.

1. Work the quarters by hand first. Find the quarter where the fee first falls short. Then decide whether the deferred step catches anything that quarter — the deferral only receives what *survives* the steps above it.
2. Add the missing deferred-fee step with `owed.mgmt_fee - paid.mgmt_fee`, ranked just after the fee itself.
3. Run. Read the owed-versus-paid columns against your arithmetic.

The construct lesson: a balance field and a paydown stream could build the same thing — smell four from the chapter. Inside the waterfall, the entitlement sits in the priority structure that governs it, and the pot arithmetic is the engine's guarantee instead of yours.
