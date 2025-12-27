//!
//! Common types and utilities shared by the quote server and client.
//!
//! This crate aggregates:
//! - `error` — unified error type `ParserError` used across the workspace.
//! - `result` — handy `Result<T, ParserError>` alias.
//! - `tickers` — ticker symbols and parsing helpers shared by both sides.
//! - `command` — TCP command payloads exchanged between client and server.
//! - `net` — networking constants and small helpers.
#![warn(missing_docs)]
pub mod error;
pub mod result;
pub mod tickers;
pub mod command;
pub mod net;

pub use error::ParserError;
pub use result::Result;
pub use command::Command;