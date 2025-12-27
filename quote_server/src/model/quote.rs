//! Quote data model and JSON encoding helpers.
//!
//! A `Quote` is the payload sent to clients. It contains the ticker symbol, the last
//! traded price, a synthetic volume, and a millisecond UTC timestamp. This module also
//! provides helper methods for generating synthetic prices and for encoding quotes to JSON.

use quote_common::ParserError;
use quote_common::tickers::Ticker;
use chrono::Utc;
use rand::Rng;
use serde::{Serialize, Deserialize};

/// Market quote for a single ticker symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Symbol identifier (string form of `Ticker`).
    pub ticker: String,
    /// Last traded price.
    pub price: f64,
    /// Synthetic trade volume associated with this tick.
    pub volume: u32,
    /// UTC timestamp in milliseconds since Unix epoch.
    pub timestamp: u64,
}

impl Quote {
    /// Calculate the next synthetic price using a small random walk around `current_price`.
    ///
    /// The change is sampled uniformly from the range `[-1%, +1%]` and the result is
    /// clamped to a minimum positive value to avoid non-sensical zero/negative prices.
    ///
    /// - current_price: last known price for the symbol.
    /// - Returns: a new price value for the next tick.
    pub fn next_price(current_price: f64) -> f64 {
        let mut rng = rand::rng();
        let change: f64 = rng.random_range(-0.01..0.01);
        let new_price = current_price * (1.0 + change);
        new_price.max(0.01)
    }

    /// Generate a new `Quote` for the given `ticker` using `current_price` as a base.
    ///
    /// Volume is synthesized based on the ticker: liquid names (AAPL/MSFT/TSLA) get a
    /// higher baseline; others receive a smaller baseline. The price is derived from
    /// [`Self::next_price`].
    ///
    /// - ticker: target symbol identifier.
    /// - current_price: last price used as a base for the next tick.
    /// - Returns: a fully-populated `Quote` with JSON-serializable fields.
    pub fn generate_new(ticker: &Ticker, current_price: f64) -> Result<Quote, ParserError> {
        let mut rng = rand::rng();
        let volume = match ticker {
            Ticker::AAPL | Ticker::MSFT | Ticker::TSLA => {
                1000 + rng.random_range(0..5000) as u32
            },
            _ => 100 + rng.random_range(0..1000) as u32,
        };

        Ok(Quote {
            ticker: ticker.to_string(),
            price: Self::next_price(current_price),
            volume,
            timestamp: Utc::now().timestamp_millis() as u64,
        })
    }

    /// Encode the quote to JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, ParserError> {
        let json = serde_json::to_vec(self)?;
        Ok(json)
    }
}
