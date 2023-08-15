## TODO

Minimum features for public release:
- [ ] Update front-end with constraints and limits (maybe just limit to 3 companies to make it feasible)
- [ ] Update the paper with constraints
- [ ] Command line interface
- [ ] Break-up the documentation into 4 sections following divio: Tutorials, How-To guides, Explanation, Reference

Improvements:
- [ ] Add renovate bot
- [ ] Improve test coverage
- [ ] Organize error codes in a centralized place since it's easier
- [ ] API trait with controller and http client
- [ ] Consider dependency injection for the Logger object that's passed around
- [ ] Figure out how to add basic validations to OpenAPI schema via schemars

Future features:
- [ ] Parallelize the loop over all combinations of constraints
- [ ] Simulation of different outcomes to:
    * Gauge how the assumption of infinite bets holds for a representative example (e.g. someone adding assets to the
      portfolio every quarter or so for 30+ years)
    * If feasible, this is the right way to pick the best solution out of all viable solutions. 
