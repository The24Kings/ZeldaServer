use std::io::Read;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use super::Type;
use super::packet::{Parser, Packet, leave::Leave};

pub struct Client {
    pub stream: Arc<TcpStream>,
    pub sender: Sender<Type>,
}

impl Client {
    pub fn new(stream: Arc<TcpStream>, sender: Sender<Type>) -> Self {
        Client { stream, sender }
    }

    pub fn read(&self) -> Result<Type, std::io::Error> {
        let mut packet_type = [0; 1];

        let bytes_read = self.stream.as_ref().read(&mut packet_type)?;

        if bytes_read != 1 {
            // Connection closed
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed",
            ));
        }

        println!(
            "[CLIENT] Read packet type: {}", 
            packet_type
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );

        // Match the type of the packet to the enum Type
        let packet: Option<Type> = match packet_type[0] {
            1 => { // MESSAGE
                let mut buffer = vec!(0; 66);

                let packet = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (0, 1))?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Type::Message(deserialized),
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
            2 => { // CHANGEROOM
                None
            },
            3 => { // FIGHT
                None
            },
            4 => { // PVPFIGHT
                None
            },
            5 => { // LOOT
                None
            },
            6 => { // START
                None
            },
            7 => { // ERROR
                None
            },
            8 => { // ACCEPT
                None
            },
            9 => { // ROOM
                None
            },
            10 => { // CHARACTER
                None
            },
            11 => { // GAME
                let mut buffer = vec!(0; 6); 

                let packet = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (4, 5))?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Type::Game(deserialized),
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to deserialize packet: {}", e),
                        ));
                    }
                };

                // Send the packet to the sender
                Some(object) //TODO: This is for testing purposes, remove it later
                //None
            },
            12 => { // LEAVE
                Some(Type::Leave(
                    Leave {
                        author: Some(self.stream.clone()),
                        ..Leave::default()
                    }
                ))
            },
            13 => { // CONNECTION
                None
            },
            14 => { // VERSION
                let mut buffer = vec!(0; 4);

                let packet = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (2, 3))?;

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
                Some(object) //TODO: This is for testing purposes, remove it later
                //None
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
                self.sender.send(pkt.clone()).map_err(|e| { // If the send fails with SendError, it means the server thread has closed
                    std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        format!("Failed to send packet: {}", e),
                    )
                })?;

                Ok(pkt)
            },
            None => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "You tried to send the server a bad packet... naughty!",
                ))
            }
        }
    }
}