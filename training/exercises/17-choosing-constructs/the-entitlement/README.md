# Where the entitlement lives

The management fee starves as collections shrink — 60, 50, 40, 30 against 30 of senior debt and 15 of fee. Add the missing deferred-fee step with `owed.mgmt_fee - paid.mgmt_fee`, ranked just after the fee itself.

Work the quarters by hand first: when does the fee first fall short, and in that quarter, does the deferred step catch anything? (Careful — the deferral only receives what *survives* the steps above it.) Then run and read the owed-versus-paid columns against your arithmetic.

The construct lesson: you could build this with a balance field and a paydown stream — smell four from the chapter. Here the entitlement sits inside the priority structure that governs it, and the pot arithmetic is the engine's guarantee instead of yours.
