# Build the rent roll

Add Harbor Point's five revenue claims and its operating costs.

1. Add the anchor office lease: 155,000 a month, with a twelve-month lease-up.
2. Add the retail suite as a `cre.lease_unit`. Give it escalating rent, three months free, and TI/LC on day one. Set recoveries above a 950,000 stop at a 20% share.
3. Add overage rent on the retailer's sales.
4. Add parking as `cre.ops_revenue`.
5. Add property opex that escalates at 3%.
6. Add a 5% stabilized vacancy allowance.

Check two shapes in the series after the run:

- The office lease climbs its ramp through mid-2028.
- The retail abatement line offsets exactly the three free months.

The model is still deeply negative — revenue has arrived, but nothing has funded the build. Funding is chapter 22's job.
