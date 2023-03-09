use crate::model::portfolio::{Portfolio, PortfolioCandidates};
use crate::model::result::{AnalysisResult, CompleteResult};

/// An interface for portfolio allocation and analysis
trait PortfolioAPI {
    /// Returns a portfolio defined with allocation fractions out of given candidates
    fn allocate(candidates: PortfolioCandidates) -> CompleteResult;

    /// Returns an analysis of the portfolio
    fn analyze(portfolio: Portfolio) -> AnalysisResult;
}
