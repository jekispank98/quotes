//! Error types used across the Quote Client.
//!
//! The `ParserError` enum unifies I/O, parsing, and (de)serialization errors so that
//! they can be propagated easily with `Result<T, ParserError>`.
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

/// Unified error type for the application.
#[derive(Error, Debug)]
pub enum ParserError {
    /// I/O error originating from the standard library or sockets/files.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// Generic formatting error with a human-readable message.
    #[error("Format error: {0}")]
    Format(String),
    /// UTF-8 conversion error when handling text content.
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),
    /// Error while parsing the ticker file into `Ticker` values.
    #[error("Parse tickers file error: {0}")]
    ParseTickersFile(String),
    /// Failure while decoding with `bincode` (invalid or truncated payloads, etc.).
    #[error("Bincode serialization/deserialization error: {0}")]
    BincodeDecode(#[from] bincode::error::DecodeError),

    /// Failure while encoding with `bincode` (I/O or serialization issues).
    #[error("Bincode serialization/deserialization error: {0}")]
    BincodeEncode(#[from] bincode::error::EncodeError),
}