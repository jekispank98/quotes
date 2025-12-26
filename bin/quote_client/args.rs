//! Command-line arguments for the Quote Client.
//!
//! This module defines the CLI interface using `clap`. See `main` for end-to-end usage.
use clap::Parser;

/// Parsed command-line arguments.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Server IP address (IPv4 or IPv6) where the quote service is running.
    #[clap(long)]
    pub server_ip: String,

    /// Local UDP port to bind for receiving quotes and sending commands.
    #[clap(long)]
    pub listen_port: String,

    /// Path to a text file with tickers to subscribe to.
    /// Tickers may be separated by commas, spaces, or new lines.
    #[clap(long)]
    pub path: String
}