//! Quote payload received from the server.
//!
//! Quotes are sent as JSON-encoded messages over UDP and decoded by the client via `serde_json`.
use serde::Deserialize;

// [2:critical] эта структура есть и в клиенте, и в сервере. Давай перенесём её в quote_common.
// В задании есть пункт "Архитектура и организация кода: логическое разделение на модули/функции,
// отсутствие дублирования кода."

/// Market quote data for a single ticker.
#[derive(Debug, Clone, Deserialize)]
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