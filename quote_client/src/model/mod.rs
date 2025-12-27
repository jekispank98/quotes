//! Data model types exchanged with the quote server.
//!
//! This module groups simple serializable types used by the client:
//! - `quote` — market quote payloads received from the server.
//! - `tickers` — ticker symbols and parsing helpers.
pub mod quote;
