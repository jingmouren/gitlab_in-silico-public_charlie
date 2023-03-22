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
- [ ] Generate OpenAPI schema for input and output data models
- [ ] Logging with trace (try logging from rocket)

Future improvements:
- [ ] Update README.md
- [ ] Improve test coverage
- [ ] Create/find macro for assertion with tolerance
- [ ] API trait with controller and http client
- [ ] Command line interface
- [ ] Special (direct) handling of single company Kelly allocation

Future features:
- [ ] Constraint for maximum allowable risk of permanent loss of capital
- [ ] Constraint for no shorting (instead of filtering based on negative expected value)
- [ ] Constraint for no leverage (instead of normalization)
