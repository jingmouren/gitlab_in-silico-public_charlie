use crate::model::portfolio::Portfolio;
use nalgebra::DVector;

/// [InequalityConstraint] is used to tag inequality constraints.
pub trait InequalityConstraint: Constraint {}

/// TODO: Declare EqualityConstraint when the time comes

/// [Constraint] is a super-trait providing the interface for calculating matrix contributions when
/// solving the Kelly allocation problem. The only thing needed for implementing a constraint is to
/// provide the partial derivative of the constraint function with respect to allocation fractions.
pub trait Constraint {
    /// Partial derivative of the constraint with respect to the fractions. Ends up in the matrix.
    fn d_constraint_d_fractions(&self, portfolio: &Portfolio) -> DVector<f64>;

    /// Constraint function value. Ends up in the right-hand-side of the system.
    fn function_value(&self, portfolio: &Portfolio, slack_variable: f64) -> f64;

    /// Whether this constraint is satisfied. Used to check whether we found a viable solution.
    fn is_satisfied(&self, portfolio: &Portfolio) -> bool;
}
