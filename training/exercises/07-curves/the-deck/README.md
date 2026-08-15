# Restore the deck

The starter blended a three-point rising price deck into one average number. Put the deck back.

1. Declare a `linear` curve named `power_price` with the quoted points.
2. Read the curve with `curve_value("power_price", time.date)`.

Predict the direction of two changes before you run:

- The lifetime total: does a rising deck raise or lower it against the flat 44.35 blend?
- The NPV: the deck's cheap megawatt-hours come early and its dear ones late, and discounting cares about exactly that.

After the run, look at the tail. 2028's months hold flat at 46.20 — the curve's clamp at work.
