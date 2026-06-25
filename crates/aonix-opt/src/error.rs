//! Optimizer error type.

use thiserror::Error;

/// An optimizer failure. The MVP only surfaces internal model errors (a
/// rebuilt circuit failing structural validation, or a simulation error
/// during the equivalence check). A *correct* transformation never triggers
/// these; they are a defensive safety net.
#[derive(Debug, Error)]
pub enum OptError {
    /// A circuit reconstruction or simulation returned a model error.
    #[error("optimizer internal model error: {0}")]
    Internal(#[from] aonix_core::circuit_model::AonixError),
}
