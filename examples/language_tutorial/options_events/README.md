# Events and options

An **event** fires when its condition first becomes true. It can set entity
state, deactivate a stream, and exercise an option.

An **option** is a payoff that only lands if it is exercised — here by the
event, at month 12, for 15,000.

The debt service stream stops two ways: the event deactivates it, and its own
`active when` reads the state the event set. Either alone would be enough;
together they show both mechanisms.
