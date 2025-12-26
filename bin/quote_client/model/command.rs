//! Command messages sent to the quote server.
//!
//! A `Command` can either be a subscription request (`J_QUOTE`) with a list of
//! tickers or a keep-alive `PING` message. Values are serialized with `bincode`
//! for compact UDP transmission.
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use crate::model::tickers::Ticker;

/// Header value for subscription commands.
const HEADER: &str = "J_QUOTE";
/// Header value for keep-alive pings.
const PING: &str = "PING";
/// Connection type, currently always UDP.
const CONNECTION: &str = "udp";

/// Command payload sent to the server.
#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct Command {
    /// Command kind. Either `J_QUOTE` or `PING`.
    pub header: String,
    /// Transport protocol name (e.g., `udp`).
    pub connection: String,
    /// Client IP address for the server to send quotes to.
    pub address: String,
    /// Client port as a string.
    pub port: String,
    /// List of tickers to subscribe to (empty for `PING`).
    pub tickers: Vec<Ticker>,
}

impl Command {
    /// Creates a new subscription (`J_QUOTE`) command.
    pub fn new(address: &str, port: &str, tickers: Vec<Ticker>) -> Self {
        Command {
            header: String::from(HEADER),
            connection: String::from(CONNECTION),
            address: String::from(address),
            port: String::from(port),
            tickers
        }
    }

    /// Creates a new keep-alive `PING` command.
    pub fn new_ping(ip: &str, port: &str) -> Self {
        Command {
            header: String::from(PING),
            connection: String::from(CONNECTION),
            address: String::from(ip),
            port: String::from(port),
            tickers: Vec::new()
        }
    }
}
