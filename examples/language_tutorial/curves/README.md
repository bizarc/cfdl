# Curves

A **curve** is a dated series — a forward price, an index, a rate path —
declared once and read by date.

This model declares a three-point power price curve and drives revenue from it,
so the price path lives in one place instead of being restated in every amount.

`linear` interpolates between the stated points; `step` holds the last point
forward. Read a curve with `curve_value(name, date)`.

Note that `obs.*` is a different thing: observations supplied at run time rather
than declared in the model.
