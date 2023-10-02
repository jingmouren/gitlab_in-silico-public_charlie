## Tutorials

The tutorials assume that you have successfully built the project following the instructions in the
[How-To Guides](/HOW-TO.md#clone-and-build-the-project).

### CLI application

The best place to start is to run one of the test examples via the CLI application. The two examples can be run with:
1. `./target/release/charle_cli allocate ./tests/test_data_no_constraints.yaml`
2. `./target/release/charle_cli allocate ./tests/test_data_with_constraints.yaml`

The inputs are given in a `.yaml` file, where the first example contains just a list of `candidate` companies, each
defined with a ticker, description, market capitalization and a list of `scenarios`. The probabilities of all scenarios
for a candidate company must sum up to 1 (100%). In addition to probability, a scenario is defined by a thesis and an
intrinsic value estimate.

The difference between the first and the second example is the constraints. In `./tests/test_data_with_constraints.yaml`
one can see the settings for four available constraints:
1. `long_only`: Does not allow shorting (negative fractions).
2. `max_individual_allocation`: Does not allow more than X% of capital to be invested in a single company.
3. `max_total_leverage_ratio`: Limits the use of leverage in a portfolio to a specified amount. Zero means no 
                               leverage.
4. `max_permanent_loss_of_capital`: Models the maximum allowable permanent loss of capital in a worst-case scenario.
                                    This constraint can be read as: "Under a worst-case scenario, I'm comfortable
                                    losing X of capital with probability P."

Note that including the constraints increases significantly the time to find the numerical solution. If there are no
constraints, there's only one viable solution to find. If all the four constraints are specified, there are
`2^(2N + 2)` systems to solve. For example, for `N = 10` candidate companies, there are `2^22` systems to solve, which
is approximately 4 million. 

### Clients for the server application

Before running the example client applications, make sure that the server is running by following the steps in the
[How-To Guides](/HOW-TO.md#server-application).

After the server is up and running, you can try running the two clients in `examples`, with the following commands:
- `cargo run --example allocate_client`
- `cargo run --example analyze_client`

You can also open up the browser and go to `http://localhost:8000/demo` and use the simple front-end. Note that the
front-end does not include constraints yet.
