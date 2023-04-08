/// Macro for asserting that the two numbers are close to each other with a given absolute
/// tolerance. Used in tests.
#[macro_export]
macro_rules! assert_close {
    ($expected:expr, $actual:expr, $abs_tolerance:expr) => {
        assert!(
            ($actual - $expected).abs() < $abs_tolerance,
            "Expected close to {}, got {}, which is outside of tolerance {}",
            $expected,
            $actual,
            $abs_tolerance
        );
    };
}

pub use assert_close;
