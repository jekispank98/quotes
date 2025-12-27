//! Result type alias shared across the workspace.
//!
//! This module defines a convenient alias that defaults the error type to the
//! common `ParserError`, so functions can simply return `Result<T>`.
use crate::error::ParserError;

/// Workspace-wide `Result` alias with `ParserError` as the default error.
pub type Result<T, E = ParserError> = std::result::Result<T, E>;