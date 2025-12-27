//! Shared networking constants and helpers used by client and server.

/// TCP port for a command channel (client -> server).
pub const COMMAND_PORT: u16 = 8080;
/// UDP port for data streaming and pings (server <-> client).
pub const DATA_PORT: u16 = 8081;

/// Helper to format an IPv4 address with a port like "ip:port".
pub fn addr(ip: &str, port: u16) -> String {
    format!("{}:{}", ip, port)
}
