# The level payment

Replace the placeholder debt service with the real claim.

1. Write the payment with `pmt()`: a 250,000 loan at 7.2% annual, fully amortizing over 60 monthly payments.

Anchor the number before you trust it:

- The first month's interest is 250,000 × 0.6% = 1,500, so the payment must exceed 1,500.
- Sixty payments must total more than 250,000. The excess is lifetime interest.

After the run, look at the NPV. The run configuration discounts at the loan's own 7.2%, so a fairly priced loan should net to zero — the result is instead about −1,356. The gap is rate conversion, live. The payment uses 7.2% divided by twelve: 0.600% a month, the loan-document convention. Discounting compounds 7.2% into 0.582% a month. Both conventions are defensible, and the difference is real. Explain which rate is higher and why the NPV comes out negative.
