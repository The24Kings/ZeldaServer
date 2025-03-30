use std::io::Read;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::protocol::{Packet, Parser, Type};

struct Client {
    stream: Arc<TcpStream>,
    sender: Sender<Type>,
}

impl Client {
    pub fn new(stream: Arc<TcpStream>, sender: Sender<Type>) -> Self {
        Client { stream, sender }
    }

    pub fn read(&self) -> Result<Type, std::io::Error> {
        let mut packet_type = [0; 1];

        loop {
            let bytes_read = self.stream.as_ref().read(&mut packet_type)?;

            match bytes_read {
                0 => {
                    // Connection closed
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Connection closed",
                    ));
                }
                1 => {
                    // Match the type of the packet to the enum Type
                    let packet: Option<Type> = match packet_type[0] {
                        1 => {
                            None
                        },
                        2 => {
                            None
                        },
                        3 => {
                            None
                        },
                        4 => {
                            None
                        },
                        5 => {
                            None
                        },
                        6 => {
                            None
                        },
                        7 => {
                            None
                        },
                        8 => {
                            None
                        },
                        9 => {
                            None
                        },
                        10 => {
                            None
                        },
                        11 => {
                            None // The client should never send a GAME packet, we are the only ones to send it
                        },
                        12 => {
                            None
                        },
                        13 => {
                            None
                        },
                        14 => {
                            let mut buffer = vec!(0; 4); // Version is 4 bytes (Plus the extensions)

                            let packet = Packet::read_into(self.stream.clone(), packet_type[0], &mut buffer).unwrap();

                            let object = match Parser::deserialize(packet) {
                                Ok(deserialized) => Type::Version(deserialized),
                                Err(e) => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        format!("Failed to deserialize packet: {}", e),
                                    ));
                                }
                            };

                            // Send the packet to the sender
                            Some(object)
                        },
                        _ => {
                            // Invalid packet type
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid packet type",
                            ));
                        }
                    };

                    // Send the packet to the server thread
                    match packet {
                        Some(pkt) => {
                            self.sender.send(pkt).map_err(|e| {
                                std::io::Error::new(
                                    std::io::ErrorKind::BrokenPipe,
                                    format!("Failed to send packet: {}", e),
                                )
                            })?;
                        },
                        None => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "You tried to send the server a bad packet... naughty!",
                            ));
                        }
                    }
                }
                _ => { 
                    // Invalid packet size (This should not happen)
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid packet size... How did you get here?",
                    ));
                }
            }
        }
    }
}

pub fn client(stream: Arc<TcpStream>, sender: Sender<Type>) {
    let client = Client::new(stream.clone(), sender);

    loop {
        match client.read() {
            Ok(data) => {
                // Process the data
                println!("[CLIENT] Received data: {:?}", data);
            }
            Err(e) => {
                eprintln!("[CLIENT] Error reading from stream: {}", e);

                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    eprintln!("[CLIENT] Broken pipe detected. Terminating thread.");
                    break;
                }

                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    eprintln!("[CLIENT] User closed the connection. Terminating thread.");
                    break;
                }
            }
        }
    }
}
