# Uncertainty

An assumption can be a **distribution** rather than a number, which is what
turns one model into a range of outcomes.

`assume growth ~ Normal(...)` is sampled per trial. A deterministic run uses the
distribution's mean, so the same model answers both questions without being
rewritten.

`run.json` asks for 500 trials with a fixed seed, so the run reproduces exactly.
The results carry percentiles alongside the deterministic figures.
