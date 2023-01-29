use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

/// A scenario is represented by an investment thesis, which can be boiled down to the expected
/// intrinsic value and the estimated probability that this scenario will play out in the future
#[derive(Serialize, Deserialize, Debug)]
pub struct Scenario {
    pub(crate) thesis: String,
    pub(crate) intrinsic_value: f64,
    pub probability: f64,
}

/// Two scenarios are considered equal if their theses are equal, irrespective of the numbers
impl PartialEq<Self> for Scenario {
    fn eq(&self, other: &Self) -> bool {
        self.thesis == other.thesis
    }
}

impl Eq for Scenario {}

/// Hash key based on the thesis
impl Hash for Scenario {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.thesis.hash(hasher)
    }
}

impl Scenario {
    /// Does all validations
    pub fn validate(&self) {
        self.validate_intrinsic_value();
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

    /// Panics if we provide intrinsic value smaller than 100000 to indicate that this is the value
    /// of the whole business, and not value per share
    /// TODO: Convert panic to recoverable errors that can be handled
    fn validate_intrinsic_value(&self) {
        if self.intrinsic_value < 1e5 {
            panic!(
                "Intrinsic value of {} is smaller than 100 000. Intrinsic value represents \
                the value of the whole business, and not value per share.", self.intrinsic_value
            )
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::hash_map::DefaultHasher;
    use super::*;

    #[test]
    fn test_scenario_serialization() {
        let test_scenario: Scenario = Scenario {
            thesis: "Liquidation value".to_string(),
            intrinsic_value: 1e6,
            probability: 0.6,
        };
        let test_str = serde_yaml::to_string(&test_scenario).unwrap();

        assert_eq!(test_str, "thesis: Liquidation value\nintrinsic_value: 1000000.0\nprobability: 0.6\n");
    }

    #[test]
    fn test_scenario_deserialization() {
        let test_yaml: &str = "
            thesis: Liquidation value
            intrinsic_value: 1e6
            probability: 0.6
        ";

        let test_scenario: Scenario = serde_yaml::from_str(&test_yaml).unwrap();

        assert_eq!(test_scenario.thesis, "Liquidation value");
        assert_eq!(test_scenario.intrinsic_value, 1e6);
        assert_eq!(test_scenario.probability, 0.6);
    }

    #[test]
    #[should_panic(expected = "Negative probability is not allowed. Probability: -0.2")]
    fn test_negative_probability_panics() {
        let test_scenario = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1e10,
            probability: -0.2,
        };
        test_scenario.validate();
    }

    #[test]
    #[should_panic(expected = "Probability greater than 1 is not allowed. Probability: 1.2")]
    fn test_probability_higher_than_one_panics() {
        let test_scenario = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1e10,
            probability: 1.2,
        };
        test_scenario.validate();
    }

    #[test]
    #[should_panic(expected = "Intrinsic value of 42 is smaller than 100 000.")]
    fn test_low_intrinsic_value_panics() {
        let test_scenario = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 42.0,
            probability: 0.5,
        };
        test_scenario.validate();
    }

    #[test]
    fn two_scenarios_with_same_thesis_are_equal_irrespective_of_different_intrinsic_value() {
        let test_scenario_1 = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1.2e7,
            probability: 0.3,
        };
        let test_scenario_2 = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1.2e8,
            probability: 0.4,
        };
        assert_eq!(test_scenario_1, test_scenario_2)
    }

    #[test]
    fn two_scenarios_with_same_thesis_have_equal_hash_irrespective_of_different_intrinsic_value() {
        let mut hasher = DefaultHasher::new();
        let test_scenario_1 = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1.2e7,
            probability: 0.3,
        };
        let test_scenario_2 = Scenario {
            thesis: "Awesome thesis".to_string(),
            intrinsic_value: 1.2e8,
            probability: 0.4,
        };
        assert_eq!(test_scenario_1.hash(&mut hasher), test_scenario_2.hash(&mut hasher));
    }
}