//! Shared protocol command type used by client and server.
//!
//! A `Command` can either be a subscription request (`J_QUOTE`) with a list of
//! tickers or a keep-alive `PING` message. Values are serialized with `bincode`
//! for compact transmission.
use std::net::SocketAddr;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::tickers::Ticker;

/// Header value for subscription commands.
pub const HEADER: &str = "J_QUOTE";
/// Header value for keep-alive pings.
pub const PING: &str = "PING";
/// Transport kind (currently UDP).
/// Keep the lowercase to match the existing client value.
pub const CONNECTION: &str = "udp";

/// Command payload sent between client and server.
#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize)]
pub struct Command {
    /// Command kind. Either `J_QUOTE` or `PING`.
    pub header: String,
    /// Transport protocol name (e.g., `udp`).
    pub connection: String,
    /// IP address.
    pub address: String,
    /// Port as a string.
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
            tickers,
        }
    }

    /// Creates a new keep-alive `PING` command.
    pub fn new_ping(address: &str, port: &str) -> Self {
        Command {
            header: String::from(PING),
            connection: String::from(CONNECTION),
            address: String::from(address),
            port: String::from(port),
            tickers: Vec::new(),
        }
    }

    /// Build UDP socket address from the fields.
    pub fn get_udp_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.address, self.port).parse()
    }

    /// Build TCP socket address from the fields.
    pub fn get_tcp_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.address, self.port).parse()
    }
}
