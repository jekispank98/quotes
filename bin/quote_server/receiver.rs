use crate::error::ParserError;
use crate::model::command::Command;
use crossbeam_channel::Sender;
use std::io::Read;
use std::net::{SocketAddr, TcpListener};

pub struct QuoteReceiver {
    pub(crate) socket: TcpListener,
}

impl QuoteReceiver {
    pub fn new(bind_addr: &str) -> Result<Self, ParserError> {
        let socket = TcpListener::bind(bind_addr)?;
        Ok(Self { socket })
    }
    pub(crate) fn receive_loop_with_channel(
        self,
        tx: Sender<(Command, SocketAddr)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Command TCP server is started on {}",
            self.socket.local_addr()?
        );

        for stream in self.socket.incoming() {
            match stream {
                Ok(mut stream) => {
                    let client_tcp_addr = stream.peer_addr()?;
                    println!("client_tcp_addr: {:?}", &client_tcp_addr);
                    let mut buf = [0u8; 1024];

                    match stream.read(&mut buf) {
                        Ok(size) => {
                            if let Ok(command) = bincode::decode_from_slice::<Command, _>(
                                &buf[..size],
                                bincode::config::standard(),
                            ) {
                                println!("Received command {:?}", command);
                                let target_udp_addr = command.0.get_udp_addr()?;
                                tx.send((command.0, target_udp_addr))?;
                            }
                        }
                        Err(e) => eprintln!("Ошибка чтения TCP: {}", e),
                    }
                }
                Err(e) => eprintln!("Ошибка TCP соединения: {}", e),
            }
        }
        Ok(())
    }
}
