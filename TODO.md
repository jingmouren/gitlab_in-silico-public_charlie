## TODO

Minimum:
- [x] Calculation of expected value
- [x] Calculation of probability of loss of capital
- [x] Improve test coverage of company_returns field
- [x] Allocation via Kelly's formula for multiple investments
- [x] Integration tests
- [x] Logging instead of printing
- [x] Refactor validation
- [x] Server POC
- [x] Bring back integration tests
- [x] Analysis endpoint
- [x] Error handling
- [x] Refactor data models: move them into appropriate modules and simplify composition
- [x] Add warning if a company got filtered
- [x] Add filtering for companies without downside
- [x] Run a server and post two examples in the pipeline
- [x] Generate OpenAPI schema for input and output data models
- [x] Move from rocket to dropshot
- [x] Generate OpenAPI schema in dropshot
- [x] Set-up logging
- [x] Figure out how to serve Swagger UI

Future improvements:
- [x] Update README.md
- [x] Rename the project
- [x] Pass logging level to create_logger function to suppress logs in tests
- [x] Write/find macro for assertion with tolerance
- [x] Add a test for monitoring schema and index.html changes
- [x] Consider adding a test for changes in index.html as well
- [x] Improve test coverage
- [x] Create config file for server config
- [x] Build docker image and push to gitlab registry
- [ ] Update README.md w.r.t. constraints
- [ ] Figure out how to add basic validations to OpenAPI schema via schemars
- [ ] API trait with controller and http client
- [ ] Command line interface
- [ ] Consider dependency injection for the Logger object that's passed around
- [ ] Add renovate bot

Future features:
- [x] Constraint for maximum allowable risk of permanent loss of capital
- [ ] Constraint for no shorting (instead of filtering based on negative expected value)
- [ ] Constraint for no leverage (instead of normalization)
- [ ] Simulation of different outcomes to gauge how the assumption of infinite bets holds for a representative example
      (e.g. someone adding assets to the portfolio every quarter or so for 30+ years) 
