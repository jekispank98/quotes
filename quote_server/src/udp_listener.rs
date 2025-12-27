use crate::model::ping_monitor::PingMonitor;
use log::{debug};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;

/// Lightweight UDP listener that receives PING datagrams from clients
/// and updates the in-memory `PingMonitor` with the sender address.
pub struct UdpPingListener;

impl UdpPingListener {
    /// Spawn a background thread that reads UDP packets from `socket` and,
    /// when a `PING` message is observed, updates `ping_monitor` for the sender.
    pub fn start(socket: Arc<UdpSocket>, ping_monitor: Arc<Mutex<PingMonitor>>) {
        thread::spawn(move || {
            let mut buf = [0u8; 128];
            loop {
                if let Ok((size, addr)) = socket.recv_from(&mut buf) {
                    if size >= 4 && &buf[..4] == b"PING" {
                        debug!("Received ping from {}", addr);
                        let mut monitor = ping_monitor.lock().unwrap();
                        monitor.update_ping(addr);
                    }
                }
            }
        });
    }
}