## Reference Guide

The reference guide provides some background in the [Introduction](#introduction) section and provides a top-level
overview of the project's structure in the [About This Software](#about-this-software) section. 

### Introduction

The underlying philosophy behind the software is that one should spend the majority of their time analyzing investments
and thinking about intrinsic values and different scenarios that might play out, independently of outside thoughts and
events. The software will force you to think about unpredictable external events. In fact, it won't help you if you're
analyzing a company for which you cannot think of at least one downside scenario.

#### How do we work with this software?

This is how we use this software:
- We spend 99.99% of our time thinking about companies that we'd like to invest in, and providing different scenarios
  for each company. A scenario is described by an estimate of the intrinsic value and the probability that this
  intrinsic value will be realized at some (undefined) point in the future.
- The tool will then take our inputs and calculate optimal allocation percentage for the candidate companies, by
  maximizing long-term growth rate of the assets. This is equivalent to Kelly's approach, but generalized for multiple
  simultaneous investments in a focused portfolio.

#### What this tool won't allow you to do?

There are certain things that the tool won't allow you to do, in order to provide additional margins of safety. The
tool won't allow you to:
- Invest in companies that have negative expected return based on your inputs. In other words, it won't let you go
  short,
- Invest in companies that do not have a downside. If a company truly didn't have a downside, you should lever up
  infinitely and put all your assets into it, which is usually not a good idea unless you have a founder-like insight
  into a fairly predictable business, in which case you shouldn't need this tool. Note that these insights probably
  represent the best investment opportunities, so you should consider investing in them outside of this framework.

#### Disclaimer

Calculating intrinsic value of a company is more of an art than science, especially for a high-quality, growing
businesses within your circle of competence. And according to Charlie Munger, Warren Buffett, Mohnish Pabrai and the
like, one should focus precisely on getting such great businesses for a fair price. That means that you shouldn't take
what this tool says at face value, and you should probably use its guidance infrequently.

During the mathematical derivation of the problem (optimizing long-term growth rate), an assumption is made that the
number of similar bets with similar outcomes is very high (tends to infinity). We don't have a hard mathematical proof
that this is completely ok for a focused investment strategy, although I feel confident it is ok if one considers the
following margins of safety:
1. Avoid shorting by using the "long only" constraint,
2. Avoid excessive use of leverage by specifying the maximum total leverage ratio close to zero (leverage ratio here is 
   defined as the leverage divided by liquid assets to allocate),
3. No companies without at least one downside scenario are allowed,
4. Specifying maximum permanent loss of capital you are comfortable with, defined by the probability of the worst case
   outcome happening multiplied with the money you'll be comfortable loosing (e.g. "I'm ok to permanently lose 25% of
   the capital with probability of 1%"),
5. And by far most importantly, conservative input assumptions should be provided. This is the tricky part, because this
   tool unfortunately can't protect you from yourself.

### About This Software

The project is written in the `rust` programming language. The project has a Command-Line-Interface (CLI) and the REST
API. This section just provides the references. For building/installing the software, see [How-To Guides](HOW-TO.md).
For running the software, see [Tutorials](TUTORIALS.md).  

#### CLI application

The CLI application can be found in [charlie_cli](/src/bin/charlie_cli.rs). See
[Tutorials](./TUTORIALS.md#cli-application) for examples on how to use the CLI application.

#### Server

The server can be found in [run_server](/src/bin/run_server.rs). The service has only a few endpoints that are
documented in the [OpenAPI schema](/schema/openapi.json). The project relies on the
[Dropshot](https://docs.rs/dropshot/latest/dropshot/) library for generating REST API.

#### Project structure

Project structure is:
- `src` directory contains the source code:
  - `lib.rs` contains the library (public) interface,
  - `model` contains data model classes,
  - `validation` contains model for validation errors and the interface for performing validations,
  - `constraints` contains the interface for adding inequality constraints along with some concrete implementations,
  - Other files contain business-level logic for solving the allocation process,
  - Unit tests are separate modules in the same source files that they're testing,
  - `bin` directory contains three binaries: CLI application, server, and an application for generating the OpenAPI
          schema,
- `tests` directory contains integration tests,
- `examples` directory contains client-side examples for interacting with the server,
- `schema` directory contains the OpenAPI schema and the `index.html` used by the server,
- `demo` basic front-end with limited functionality.

#### Continuous Integration

The CI pipeline consists of:
1. Basic checks that include formatting, linter, and security,
2. Testing suite containing unit and integration tests,
3. Building of a Docker image (see [Dockerfile](/Dockerfile)) that contains both the server and the CLI application,
4. REST-API tests that run the server and run two client examples,
5. CLI tests that run the CLI application.

See [GITLAB CI](/.gitlab-ci.yml) for details.