//! Error types shared between client and server.
//!
//! The `ParserError` enum unifies common failure cases for I/O, serialization,
//! channel communication, and internal logic, allowing crates to propagate a
//! single error type.
use std::io;
use std::string::FromUtf8Error;
use std::sync::PoisonError;

use thiserror::Error;

/// Unified error type shared by client and server.
#[derive(Error, Debug)]
pub enum ParserError {
    /// I/O error originating from the standard library or sockets/files.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Generic formatting/validation error with a human-readable message.
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

    /// Failure while encoding/decoding JSON via serde_json.
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Crossbeam/channel send failed (e.g., receiver dropped); contains a short context string.
    #[error("Channel send failed: {0}")]
    ChannelSend(String),

    /// Crossbeam/channel receive failed (e.g., sender closed); contains a short context string.
    #[error("Channel receive failed: {0}")]
    ChannelRecv(String),

    /// Error indicating a poisoned mutex/lock was encountered.
    #[error("Mutex Lock Poisoned: {0}")]
    MutexLock(String),

    /// Internal logic error where a requested ticker symbol could not be resolved.
    #[error("Internal Logic Error: Ticker not found: {0}")]
    TickerNotFound(String),
}

impl<T> From<PoisonError<T>> for ParserError {
    fn from(err: PoisonError<T>) -> Self {
        ParserError::MutexLock(err.to_string())
    }
}
