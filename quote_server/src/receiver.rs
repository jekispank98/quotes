use quote_common::ParserError;
use quote_common::command::Command;
use crossbeam_channel::Sender;
use log::{error, info, debug};
use std::io::Read;
use std::net::{SocketAddr, TcpListener};

/// TCP command receiver that accepts client subscription requests over TCP.
///
/// Creates a listening socket and parses incoming `Command` messages from clients.
/// For each successfully decoded command, the receiver emits the command together
/// with the target client's UDP `SocketAddr` into a provided channel.
pub struct QuoteReceiver {
    /// The underlying TCP listening socket.
    pub(crate) socket: TcpListener,
}

impl QuoteReceiver {
    /// Bind a new TCP receiver to the provided `bind_addr` (e.g., `0.0.0.0:8080`).
    pub fn new(bind_addr: &str) -> Result<Self, ParserError> {
        let socket = TcpListener::bind(bind_addr)?;
        Ok(Self { socket })
    }

    /// Blocking loop that accepts TCP connections, reads a single `Command` per
    /// connection, and forwards it to `tx` with a computed UDP target address.
    pub(crate) fn receive_loop_with_channel(
        self,
        tx: Sender<(Command, SocketAddr)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // [5:critical] Здесь очень строгая обработка ошибок - если любой клиент
        // пришлёт команду, которую твой сервер не понимает, то основнйо поток сервера
        // завершится и он больше не будет принимать запросы от клиентов. Надо
        // обрабатывать ошибки в **обработке команд от конкретного клиента** так, чтобы
        // сервер не переставал работать с другими.

        info!(
            "Command TCP server is started on {}",
            self.socket.local_addr()?
        );

        for stream in self.socket.incoming() {
            match stream {
                Ok(mut stream) => {
                    let client_tcp_addr = stream.peer_addr()?;
                    debug!("client_tcp_addr: {:?}", &client_tcp_addr);
                    let mut buf = [0u8; 1024];

                    match stream.read(&mut buf) {
                        Ok(size) => {
                            let cmd: Command = serde_json::from_slice(&buf[..size])
                                .map_err(|e| format!("JSON error: {}", e))?;
                            {
                                info!("Received command {:?}", cmd);

                                let port: u16 = cmd
                                    .port
                                    .parse()
                                    .map_err(|e| format!("Invalid UDP port in command: {}", e))?
                                    ;
                                let target_udp_addr = SocketAddr::new(client_tcp_addr.ip(), port);

                                tx.send((cmd, target_udp_addr))?;
                            }
                        }
                        Err(e) => error!("Read TCP error: {}", e),
                    }
                }
                Err(e) => error!("TCP connection error: {}", e),
            }
        }
        Ok(())
    }
}
