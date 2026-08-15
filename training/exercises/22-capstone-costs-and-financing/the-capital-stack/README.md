# Fund the build

Add the capital stack.

1. Add the construction facility. The lender advances 65% of each draw — 455,000 a month — with interest accruing on the drawn balance at 7.5%. Carry the balance in a rule field; fund the interest from a `prev` read.
2. Repay the facility in full at stabilization.
3. Close the permanent loan: `cre.permanent_debt`, 9,500,000, interest-only at 5.8%, payment struck on a 30-year schedule.

Anchors: advances total 7,735,000, and the payoff repays exactly 7,735,000. Permanent debt service is 45,916.67 a month, thirty times.

After the run, read the DSCR the pack derives — stabilized NOI against interest-only service — and note how comfortable the coverage is. This deal's risk is in the build and the exit, not the coverage.
