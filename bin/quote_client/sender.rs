//! Sending commands to the quote server over UDP.
//!
//! This module provides a small helper for encoding and sending `Command` messages
//! and for running a background PING loop to keep the subscription alive.
use crate::error::ParserError;
use crate::model::command::Command;
use std::io::{ErrorKind, Write};
use std::net::{TcpStream, UdpSocket};
use std::thread;
use std::time::Duration;

/// PING interval in milliseconds used by the background thread.
const INTERVAL_MS: u64 = 2000;

/// Helper type for sending commands to the server.
pub struct CommandSender;

// sender.rs (клиент)
impl CommandSender {
    /// Отправляет команду через TCP
    pub fn send_command(stream: &mut TcpStream, command: &Command) -> Result<(), ParserError> {
        // Формируем текстовую команду STREAM
        let tickers_str: Vec<String> = command.tickers.iter().map(|t| t.to_string()).collect();
        let command_text = format!(
            "STREAM udp://{}:{} {}\n",
            command.address, // IP клиента
            command.port,    // UDP порт клиента
            tickers_str.join(",")
        );
        let com = bincode::encode_to_vec(command, bincode::config::standard()).unwrap();

        println!("Sending command: {}", command_text.trim());
        stream.write_all(&com)?;
        Ok(())
    }

    /// Отправляет пинги через UDP
    pub fn start_ping_thread(socket: UdpSocket, target_addr: String, ping_command: Command) {
        println!("Ping thread started. Target: {}", target_addr);
        thread::spawn(move || {
            let interval = Duration::from_millis(INTERVAL_MS);
            loop {
                thread::sleep(interval);

                // Просто отправляем "PING" текст
                let ping_message = b"PING";

                match socket.send_to(ping_message, &target_addr) {
                    Ok(_) => println!("PING sent to {}", target_addr),
                    Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                        continue;
                    }
                    Err(e) => {
                        eprintln!("PING THREAD ERROR: Failed to send PING: {}", e);
                    }
                }
            }
        });
    }
}
