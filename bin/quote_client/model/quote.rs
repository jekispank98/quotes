//! Quote payload received from the server.
//!
//! Quotes are sent as `bincode`-encoded messages over UDP and decoded by the client.
use bincode::{Decode, Encode};

/// Market quote data for a single ticker.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Quote {
    /// Ticker symbol (e.g., `AAPL`).
    pub ticker: String,
    /// Last traded price.
    pub price: f64,
    /// Traded volume associated with the quote.
    pub volume: u32,
    /// Event timestamp in milliseconds since the UNIX epoch.
    pub timestamp: u64,
}