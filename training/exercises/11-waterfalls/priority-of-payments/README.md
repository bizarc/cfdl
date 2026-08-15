# Transcribe the priority of payments

The trust receives 100,000 a quarter. The documents pay a 4,500 servicing fee, then 62,000 of debt service, then everything remaining to the investor.

1. Write the waterfall in document order.
2. Pay from `asset.trust.available_funds`.

Check by allocation: every quarter must account for exactly 100,000 across the three steps — 4,500 + 62,000 + 33,500.

Then stress the structure in your head. Set collections to 60,000 and name who is short, and by how much. The engine's answer, when you try it, is in the owed-versus-paid columns.
