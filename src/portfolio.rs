use std::collections::{HashMap, HashSet};
use crate::company::Company;

/// Portfolio is a map of companies with associated fractions/allocations (e.g. company ABC is 20%
/// of the portfolio)
type Portfolio = HashMap<Company, f64>;

/// (Portfolio) Candidates is a set of companies under consideration for investment
type Candidates = HashSet<Company>;