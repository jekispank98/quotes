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
mod error;
mod model;
mod sender;
mod result;

use crate::error::ParserError;
use crate::model::command::Command;
use crate::model::quote::Quote;
use crate::model::tickers::{Ticker, TickerParser};
use crate::sender::CommandSender;
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::io::ErrorKind;
use std::net::{TcpStream, UdpSocket};
use std::path::PathBuf;
use std::time::Duration;
use result::Result;
use crate::args::Args;

/// UDP port on which the quote server is expected to listen.
/// TCP port for server commands
const SERVER_COMMAND_PORT: &str = "8080";
/// UDP port for server data streaming
const SERVER_DATA_PORT: &str = "8081";

/// Runs a blocking loop that receives `Quote` messages from the given UDP `socket`
/// and prints them to stdout. Returns an error if receiving or decoding fails.
// lib - функция start_receiver_loop
fn start_receiver_loop(socket: UdpSocket) -> Result<(), ParserError> {
    println!(
        "Quote receiver running on port: {}",
        socket.local_addr()?
    );
    let mut buf = [0u8; 1024];

    loop {
        match socket.recv(&mut buf) {
            Ok(size) => {
                // Попробуем декодировать как Quote
                if let Ok((quote, _)) = bincode::decode_from_slice::<Quote, _>(
                    &buf[..size],
                    bincode::config::standard(),
                ) {
                    println!(
                        "QUOTE: {} Price={:.2} Volume={} Time={}",
                        quote.ticker, quote.price, quote.volume, quote.timestamp
                    );
                } else {
                    // Если не Quote, может быть ping или другой контрольный пакет
                    let message = String::from_utf8_lossy(&buf[..size]);
                    if message == "PING" || message.contains("PING") {
                        println!("Received PING from server");
                    } else {
                        println!("Received unknown message: {}", message);
                    }
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                    continue;
                }
                if e.kind() == ErrorKind::ConnectionReset {
                    continue;
                }
                eprintln!("Receive data error: {}", e);
                return Err(ParserError::Format(format!("{}", e.to_string())));
            }
        }
    }
}

fn main() -> Result<(), ParserError> {
    let args = Args::parse();

    let server_ip = args.server_ip.trim().replace("\"", "").to_string();
    let listen_port = args.listen_port.trim().replace("\"", "").to_string();

    // TCP адрес сервера для команд
    let server_command_address = format!("{}:{}", server_ip, SERVER_COMMAND_PORT); // TCP:8080
    // UDP адрес сервера для пингов (если пинги идут на тот же порт)
    let server_udp_address = format!("{}:{}", server_ip, SERVER_DATA_PORT); // UDP:8081

    let listen_address = format!("0.0.0.0:{}", listen_port);

    let file_path = normalize_path(&args.path);

    if is_file_exist(&file_path) {
        let file = File::open(file_path)
            .map_err(ParserError::Io)
            .expect("Failed to open file");
        let buf = BufReader::new(file);

        let tickers = Ticker::parse_from_file(buf)?;
        println!("Tickers: {:?}", tickers);

        // 1. Создаем UDP сокет для получения данных
        let client_udp_socket = UdpSocket::bind(&listen_address)?;
        client_udp_socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        let client_local_addr = client_udp_socket.local_addr()?;

        println!("UDP client listening on: {}", client_local_addr);

        // 2. Создаем TCP соединение для отправки команды
        println!("Connecting to TCP server at {}", server_command_address);
        let mut tcp_stream = TcpStream::connect(&server_command_address)
            .map_err(|e| ParserError::Format(format!("Failed to connect to server: {}", e)))?;

        // 3. Создаем команду с UDP адресом для получения данных
        let command = Command::new(
            &client_local_addr.ip().to_string(),  // IP клиента
            &listen_port,                         // UDP порт сервера для данных
            tickers.clone(),
        );

        println!(
            "Preparing to send J_QUOTE to TCP server {}",
            server_command_address
        );

        // 4. Отправляем команду через TCP
        match CommandSender::send_command(&mut tcp_stream, &command) {
            Ok(_) => {
                println!("Initial command sent to server {}.", server_command_address);
            }
            Err(e) => {
                println!("Sending error to server {}.", e.to_string());
                return Err(ParserError::Format(format!("{}", e.to_string()))) },
        };

        // 5. Создаем UDP сокет для пингов
        // Клиент должен отправлять пинги на UDP порт сервера
        let ping_udp_socket = UdpSocket::bind("0.0.0.0:0")?; // Любой свободный порт
        let ping_command = Command::new_ping(
            &client_local_addr.ip().to_string(),
            &client_local_addr.port().to_string(),
        );

        // 6. Запускаем поток для пингов (отправляем на UDP порт сервера)
        CommandSender::start_ping_thread(
            ping_udp_socket,
            server_udp_address.clone(),
            ping_command
        );

        println!("Client is running. Press Ctrl+C to exit.");
        return start_receiver_loop(client_udp_socket);
    }

    Ok(())
}

fn normalize_path(raw: &str) -> PathBuf {
    let trimmed = raw.trim();
    let no_quotes = trimmed
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(trimmed);
    PathBuf::from(no_quotes)
}

fn is_file_exist(path: &PathBuf) -> bool {
    path.exists() && path.is_file()
}
