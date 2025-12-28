//! Quotes UDP streaming server.
//!
//! This binary listens on a UDP socket and streams quote updates to clients that send a
//! subscription command. Internally, it wires together three main building blocks:
//!
//! - `QuoteGenerator` — produces quote events (`QuoteEvent`) and broadcasts them to all
//!   subscribed clients via `crossbeam_channel` senders.
//! - `QuoteReceiver` — listens for incoming UDP datagrams with client commands and parses
//!   them into a command structure (e.g., a subscription with requested tickers) along with the
//!   sender's `SocketAddr`.
//! - Per‑client stream task — a lightweight thread created for each client to filter quotes
//!   by the client's requested tickers and send matching quotes back to that client's address.
//!
//! Concurrency and shutdown:
//! - Crossbeam `select!` is used to multiplex incoming quotes and shutdown signals.
//! - Each client stream owns a `shutdown_rx` that is triggered either by a keep‑alive timeout
//!   (detected by `QuoteReceiver`) or by a global `QuoteEvent::Shutdown` broadcast from the
//!   generator when the application is terminating.
//! - Any I/O or channel receive error is surfaced as `ParserError` and logged; the specific
//!   client stream exits gracefully without impacting other clients.
//!
//! Network protocol (high‑level):
//! - Bind address: `0.0.0.0:8080` (see `BIND_ADDRESS`).
//! - Client sends a subscription command (header like `J_QUOTE`) with a list of tickers.
//! - Server spawns a stream thread for that client and starts sending JSON‑encoded quote
//!   payloads to the client's `SocketAddr`.
//!
//! Note: This file only orchestrates; details such as the exact command format, `Quote`
//! serialization, and ticker parsing live under the `model` and `receiver` modules.
#![warn(missing_docs)]
use crate::model::ping_monitor::PingMonitor;
use crate::model::quote_generator::{QuoteEvent, QuoteGenerator};
use crate::receiver::QuoteReceiver;
use crate::udp_listener::UdpPingListener;
use crossbeam_channel::{Receiver, Sender, select, unbounded};
use log::{error, info, warn};
use quote_common::ParserError;
use quote_common::Result;
use quote_common::command::Command;
use quote_common::net::{COMMAND_PORT, DATA_PORT};
use quote_common::tickers::Ticker;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;

pub mod model;
mod receiver;
mod udp_listener;

/// Stream task for a single client.
///
/// Listens for quote events on `data_rx`, filters them by the client's `tickers`, and
/// forwards matching quotes to the client's `target_addr` via the provided UDP `socket`.
/// The task terminates when either:
/// - a shutdown signal is received on `stop_rx`, or
/// - a `QuoteEvent::Shutdown` is received from the quote generator, or
/// - a send/receive error occurs.
///
/// Errors are propagated as `ParserError` so the caller can log and recover per client.
pub fn handle_client_stream(
    socket: Arc<UdpSocket>,
    target_addr: SocketAddr,
    tickers: Vec<Ticker>,
    data_rx: Receiver<QuoteEvent>,
    stop_rx: Receiver<()>,
) -> Result<(), ParserError> {
    // [6:non-critical] Лучше здесь использовать HashSet, иначе клиент может послать 1000000 тикеров
    // (может даже одинаковых) и ты на него будешь тратить O(1000000) вместо O(1).
    let tickers_str: Vec<String> = tickers.iter().map(|t| t.to_string()).collect();

    loop {
        select! {
            recv(stop_rx) -> _ => break,
            recv(data_rx) -> msg => match msg {
                Ok(QuoteEvent::Quote(quote)) => {
                    if tickers_str.contains(&quote.ticker) {
                        match quote.to_json_bytes() {
                            Ok(data) => {
                                if let Err(e) = socket.send_to(&data, target_addr) {
                                    error!("Failed to send UDP packet to {}: {}", target_addr, e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed to serialize quote to JSON: {}", e);
                                break;
                            }
                        }
                    }
                },
                Ok(QuoteEvent::Shutdown) => break,
                Err(e) => {
                    error!("Ошибка при получении сообщения: {}", e);
                    break;
                },
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), ParserError> {
    init_logger();
    let udp_socket = Arc::new(UdpSocket::bind(format!("0.0.0.0:{}", DATA_PORT))?);
    info!("UDP socket created on: {}", udp_socket.local_addr()?);
    let ping_socket = Arc::clone(&udp_socket);
    let ping_monitor = Arc::new(Mutex::new(PingMonitor::new(5)));
    let (stop_tx, stop_rx) = unbounded::<SocketAddr>();
    let ping_monitor_clone = Arc::clone(&ping_monitor);
    thread::spawn(move || {
        UdpPingListener::start(ping_socket, ping_monitor_clone);
    });
    let stop_tx_clone = stop_tx.clone();
    let ping_monitor_for_checker = Arc::clone(&ping_monitor);
    thread::spawn(move || {
        start_ping_monitor(ping_monitor_for_checker, stop_tx_clone);
    });

    let (cmd_tx, cmd_rx) = unbounded::<(Command, SocketAddr)>();
    let tcp_receiver = QuoteReceiver::new(&format!("0.0.0.0:{}", COMMAND_PORT))?;
    thread::spawn(move || {
        if let Err(e) = tcp_receiver.receive_loop_with_channel(cmd_tx) {
            error!("Receiver loop failed: {:?}", e);
        };
    });

    let subscription_tx = QuoteGenerator::start();
    let mut active_streams: HashMap<SocketAddr, (Sender<()>, Sender<QuoteEvent>)> = HashMap::new();

    loop {
        select! {
            recv(cmd_rx) -> msg => if let Ok((cmd, target_udp_addr)) = msg {
                let (shutdown_tx, shutdown_rx) = unbounded::<()>();
                let (client_data_tx, client_data_rx) = unbounded::<QuoteEvent>();

                if let Err(e) = subscription_tx.send(client_data_tx.clone()) {
                    error!("Failed to subscribe client: {}", e);
                    continue;
                }
                active_streams.insert(target_udp_addr, (shutdown_tx, client_data_tx));

                let socket_clone = Arc::clone(&udp_socket);
                let tickers = cmd.tickers;

                thread::spawn(move || {
                    if let Err(e) = handle_client_stream(
                        socket_clone,
                        target_udp_addr,
                        tickers,
                        client_data_rx,
                        shutdown_rx,
                    ) {
                        error!("Client stream error: {:?}", e);
                    }
                });
                info!("A stream has been created for the client on a UDP address.: {}", target_udp_addr);
            },

            recv(stop_rx) -> addr => if let Ok(client_addr) = addr {
                // продолжение [1:critical] - вот здесь как-раз у тебя `client_addr` - это
                // адрес PING-сокета от сервера, а в `active_streams` лежат адреса, на которые
                // ты отсылаешь котировки => `.remove()` вернёт false.
                if let Some((shutdown_tx, _)) = active_streams.remove(&client_addr) {
                    let _ = shutdown_tx.send(());
                    info!("Stream for {} closed: ping timeout", client_addr);
                } else {
                    panic!("Вот сюда ты не должен попадать, но попадаешь при выключении клиента")
                }
            }
        }
    }
}

fn start_ping_monitor(ping_monitor: Arc<Mutex<PingMonitor>>, stop_tx: Sender<SocketAddr>) {
    thread::spawn(move || {
        let check_interval = std::time::Duration::from_secs(1);

        loop {
            thread::sleep(check_interval);
            let timed_out_clients = {
                let mut monitor = ping_monitor.lock().unwrap();
                monitor.check_timeouts()
            };
            for client_addr in timed_out_clients {
                if let Err(e) = stop_tx.send(client_addr) {
                    eprintln!("Error sending timeout notification: {}", e);
                }
            }
        }
    });
}
fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
}
