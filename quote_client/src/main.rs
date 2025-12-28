//! Quote Client — a UDP client that subscribes to stock quotes from a server and prints
//! received quotes to stdout. It reads a list of tickers from a text file, sends an
//! initial `J_QUOTE` subscription command to the server, keeps the connection alive
//! with periodic `PING`s, and continuously listens for incoming quotes.
//!
//! Usage example (CLI):
//! ```bash
//! quote_client --server-ip 192.168.0.10 --listen-port 55555 --path ./tickers.txt
//! ```
//!
//! The ticker file should contain symbols separated by commas, spaces, or new lines.
//! See `model::tickers` for details.
#![warn(missing_docs)]
mod args;
mod model;
mod sender;

use crate::args::Args;
use crate::model::quote::Quote;
use crate::sender::CommandSender;
use clap::Parser;
use log::{debug, error, info, warn};
use quote_common::command::Command;
use quote_common::tickers::Ticker;
use quote_common::tickers::TickerParser;
use quote_common::ParserError;
use quote_common::Result;
use std::fs::File;
use std::io::BufReader;
use std::io::ErrorKind;
use std::net::{TcpStream, UdpSocket};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

/// UDP port on which the quote server is expected to listen.
use quote_common::net::{COMMAND_PORT, DATA_PORT};


/// Runs a blocking loop that receives `Quote` messages from the given UDP `socket`
/// and prints them to stdout. Returns an error if receiving or decoding fails.
fn start_receiver_loop(socket: Arc<UdpSocket>, shutdown: Arc<AtomicBool>) -> Result<(), ParserError> {
    info!("Quote receiver running on: {}", socket.local_addr()?);
    let mut buf = [0u8; 2048];

    while !shutdown.load(Ordering::Relaxed) {
        match socket.recv(&mut buf) {
            Ok(size) => {
                match serde_json::from_slice::<Quote>(&buf[..size]) {
                    Ok(quote) => {
                        info!("QUOTE: {} Price={:.2} Volume={} Time={}",
                            quote.ticker, quote.price, quote.volume, quote.timestamp);
                    }
                    Err(_) => {
                        debug!("Received non-JSON message: {}", String::from_utf8_lossy(&buf[..size]));
                    }
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                    continue;
                }
                error!("Receive data error: {}", e);
                return Err(ParserError::Format(e.to_string()));
            }
        }
    }
    info!("Receiver loop stopping...");
    Ok(())
}

fn main() -> Result<(), ParserError> {
    init_logger();
    let args = Args::parse();
    let shutdown = Arc::new(AtomicBool::new(false));
    {
        let shutdown = shutdown.clone();
        ctrlc::set_handler(move || {
            info!("Ctrl+C received. Shutting down client...");
            shutdown.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl+C handler");
    }

    let server_ip = args.server_ip.trim().replace("\"", "").to_string();
    let listen_port = args.listen_port.trim().replace("\"", "").to_string();

    let server_command_address = format!("{}:{}", server_ip, COMMAND_PORT);
    let server_udp_address = format!("{}:{}", server_ip, DATA_PORT);
    let mut listen_address = format!("0.0.0.0:{}", listen_port);
    if listen_port == DATA_PORT.to_string() {
        warn!(
            "--listen-port={} matches the server port DATA_PORT ({}). A free local port will be selected.",
            listen_port, DATA_PORT
        );
        listen_address = "0.0.0.0:0".to_string();
    }

    let file_path = normalize_path(&args.path);

    if is_file_exist(&file_path) {
        let file = File::open(file_path)
            .map_err(ParserError::Io)
            .expect("Failed to open file");
        let buf = BufReader::new(file);

        let tickers = Ticker::parse_from_file(buf)?;
        info!("Tickers: {:?}", tickers);
        let client_udp_socket = Arc::new(UdpSocket::bind(&listen_address)?);
        client_udp_socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        let client_local_addr = client_udp_socket.local_addr()?;

        info!("UDP client listening on: {}", client_local_addr);

        info!("Connecting to TCP server at {}", server_command_address);
        let mut tcp_stream = TcpStream::connect(&server_command_address)
            .map_err(|e| ParserError::Format(format!("Failed to connect to server: {}", e)))?;

        let command = Command::new(
            &client_local_addr.ip().to_string(),
            &client_local_addr.port().to_string(),
            tickers.clone(),
        );

        info!(
            "Preparing to send J_QUOTE to TCP server {}",
            server_command_address
        );

        match CommandSender::send_command(&mut tcp_stream, &command) {
            Ok(_) => {
                info!("Initial command sent to server {}.", server_command_address);
            }
            Err(e) => {
                error!("Sending error to server: {}", e.to_string());
                return Err(ParserError::Format(e.to_string()));
            }
        };

        let ping_command = Command::new_ping(
            &client_local_addr.ip().to_string(),
            &client_local_addr.port().to_string(),
        );

        CommandSender::start_ping_thread(
            client_udp_socket.clone(),
            server_udp_address.clone(),
            ping_command,
            shutdown.clone(),
        );

        info!("Client is running. Press Ctrl+C to exit.");
        return start_receiver_loop(client_udp_socket, shutdown);
    }

    Ok(())
}

fn init_logger() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
}

/// Normalize a CLI-provided path string by trimming whitespace and matching quotes.
///
/// This allows passing Windows paths in quotes without breaking parsing.
fn normalize_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    let no_quotes = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed);
    PathBuf::from(no_quotes)
}

/// Returns `true` if the provided path exists and is a regular file.
fn is_file_exist(path: &PathBuf) -> bool {
    path.exists() && path.is_file()
}
