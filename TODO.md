## TODO

Improvements:
- [ ] Add renovate bot
- [ ] Improve test coverage
- [ ] Organize error codes in a centralized place since it's easier
- [ ] Consider dependency injection for the Logger object that's passed around
- [ ] Figure out how to add basic validations to OpenAPI schema via schemars

Future features:
- [ ] Update front-end with constraints and limits (maybe just limit to 3 companies to make it feasible)
- [ ] Parallelize the loop over all combinations of constraints
- [ ] Simulation of different outcomes to:
    * Gauge how the assumption of infinite bets holds for a representative example (e.g. someone adding assets to the
      portfolio every quarter or so for 30+ years)
    * If feasible, this is the right way to pick the best solution out of all viable solutions. 
