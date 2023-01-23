use std::hash::{Hash, Hasher};

#[derive(Eq)]
pub struct Scenario {
    thesis: String,
    intrinsic_value: f64,
    pub probability: f64,
}

/// Two scenarios are considered equal if their theses are equal.
impl PartialEq<Self> for Scenario {
    fn eq(&self, other: &Self) -> bool {
        self.thesis == other.thesis
    }
}

/// Hash key based on the thesis (hash keys of two objects must be equal if they evaluate to
/// being equal using PartialEq)
impl Hash for Scenario {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.thesis.hash(hasher)
    }
}

impl Scenario {
    /// Create a new instance of Scenario that does validation after initialization
    pub fn new(thesis: String, intrinsic_value: f64, probability: f64) -> Scenario {
        let scenario = Scenario { thesis, intrinsic_value, probability };

        scenario.validate();

        return scenario;
    }

    /// Does all validations. Used after construction
    fn validate(&self) {
        self.validate_probability_bounds();
    }

    /// Panics if we provide probability lower than zero and higher than one
    /// TODO: Convert panics to recoverable errors that can be handled
    fn validate_probability_bounds(&self) {
        if self.probability < 0.0 {
            panic!("Negative probability is not allowed. Probability: {}", self.probability)
        }

        if self.probability > 1.0 {
            panic!("Probability greater than 1 is not allowed. Probability: {}", self.probability)
        }
    }
}
