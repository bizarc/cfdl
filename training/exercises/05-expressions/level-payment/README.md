# The level payment

Replace the placeholder debt service with the real claim: a 250,000 loan at 7.2% annual, fully amortizing over 60 monthly payments, written with `pmt()`.

Sanity anchors before you trust yourself: the first month's interest is 250,000 × 0.6% = 1,500, so the payment must exceed that; and sixty payments must total more than 250,000 (the excess is lifetime interest).

One more thing to notice after running: the run configuration's discount rate equals the loan's own 7.2% — so you might expect the NPV of proceeds-minus-payments to be zero, the definition of a fairly priced loan. It is instead about −1,356. That gap is the rate-conversion lesson from the reading-results chapter, live: the loan's payment uses 7.2% *divided by twelve* (0.600% a month, the loan-document convention), while discounting converts 7.2% annual by *compounding* (0.582% a month). Two defensible conventions, a real difference. Explain which rate is "higher" and why the NPV comes out negative rather than positive.
