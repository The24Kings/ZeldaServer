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
                    let packet = match packet_type[0] {
                        1 => {
                            Type::Message(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        2 => {
                            Type::ChangeRoom(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        3 => {
                            Type::Fight(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        4 => {
                            Type::PVPFight(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        5 => {
                            Type::Loot(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        6 => {
                            Type::Start(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        7 => {
                            Type::Error(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        8 => {
                            Type::Accept(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        9 => {
                            Type::Room(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        10 => {
                            Type::Character(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        11 => {
                            Type::Game(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        12 => {
                            Type::Leave(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        13 => {
                            Type::Connection(
                                Default::default(), //TODO: implement deserialization
                            )
                        },
                        14 => {
                            let mut buffer = vec!(0; 4);

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
                            object
                        },
                        _ => {
                            // Invalid packet type
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid packet type",
                            ));
                        }
                    };

                    // Send the packet to the sender
                    self.sender.send(packet.clone()).map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to send packet: {}", e),
                        )
                    })?;

                    return Ok(packet);
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
                break;
            }
        }
    }
}
