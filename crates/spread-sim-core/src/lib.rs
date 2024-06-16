//! Core data structures and algorithms for the concurrent programming project.

use thiserror::Error;

pub mod model;
pub mod simulation;
pub mod validator;

/// Error indicating an insufficient padding.
#[derive(Debug, Error)]
#[error("padding of {0} is insufficient")]
pub struct InsufficientPaddingError(usize);

impl InsufficientPaddingError {
    pub fn new(padding: usize) -> Self {
        Self(padding)
    }
}
