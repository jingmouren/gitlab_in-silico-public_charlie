use crate::model::portfolio::Portfolio;
use nalgebra::DVector;

/// [InequalityConstraint] extends the [Constraint] interface and is used for marking purposes only
/// because each inequality constraint exponentially (power of two) increases the number of problems
/// to solve, compared to the equality constraint which just adds an equation to the system.
pub trait InequalityConstraint: Constraint {}

/// TODO: Declare EqualityConstraint here when the time comes.

/// [Constraint] is a super-trait providing the interface for calculating matrix contributions when
/// solving the Kelly allocation problem. The only thing needed for implementing a constraint is to
/// provide the partial derivative of the constraint function with respect to allocation fractions.
pub trait Constraint {
    /// Partial derivative of the constraint with respect to the fractions. Ends up in the matrix.
    fn d_constraint_d_fractions(&self, portfolio: &Portfolio) -> DVector<f64>;

    /// Constraint function value. Ends up in the right-hand-side of the system.
    fn function_value(&self, portfolio: &Portfolio, slack_variable: f64) -> f64;
}
