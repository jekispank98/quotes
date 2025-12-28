//! Sending commands to the quote server over UDP.
//!
//! This module provides a small helper for encoding and sending `Command` messages
//! and for running a background PING loop to keep the subscription alive.
use log::{debug, error, info};
use quote_common::command::Command;
use quote_common::ParserError;
use std::io::{ErrorKind, Write};
use std::net::{TcpStream, UdpSocket};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

/// PING interval in milliseconds used by the background thread.
const INTERVAL_MS: u64 = 2000;

/// Helper type for sending commands to the server.
pub struct CommandSender;

impl CommandSender {
    pub fn send_command(stream: &mut TcpStream, command: &Command) -> Result<(), ParserError> {
        let tickers_str: Vec<String> = command.tickers.iter().map(|t| t.to_string()).collect();
        let command_text = format!(
            "STREAM udp://{}:{} {}\n",
            command.address,
            command.port,
            tickers_str.join(",")
        );
        let com = serde_json::to_vec(&command)?;

        info!("Sending command: {}", command_text.trim());
        stream.write_all(&com)?;
        Ok(())
    }
    pub fn start_ping_thread(
        socket: Arc<UdpSocket>,
        target_addr: String,
        _ping_command: Command,
        shutdown: Arc<AtomicBool>,
    ) {
        info!("Ping thread started. Target: {}", target_addr);
        thread::spawn(move || {
            let interval = Duration::from_millis(INTERVAL_MS);
            while !shutdown.load(Ordering::Relaxed) {
                thread::sleep(interval);
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                let ping_message = b"PING";

                match socket.send_to(ping_message, &target_addr) {
                    Ok(_) => debug!("PING sent to {}", target_addr),
                    Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                        continue;
                    }
                    Err(e) => {
                        error!("PING THREAD ERROR: Failed to send PING: {}", e);
                    }
                }
            }
            info!("Ping thread stopping...");
        });
    }
}
