use std::io::Read;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use super::packet::character::{self, Character};
use super::Type;
use super::packet::{Packet, Parser, leave::Leave, start::Start, fight::Fight};

#[derive(Debug, Clone)]
pub struct Client {
    pub stream: Arc<TcpStream>,
    pub sender: Sender<Type>,
    pub character: Option<Character>,
}

impl Client {
    pub fn new(stream: Arc<TcpStream>, sender: Sender<Type>, character: Option<Character>) -> Self {
        Client { stream, sender, character }
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

        println!("[CLIENT] Read packet type: {}", packet_type[0]);

        // Match the type of the packet to the enum Type
        let packet: Option<Type> = match packet_type[0] {
            1 => { // MESSAGE
                let mut buffer = vec!(0; 66);

                let packet = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (0, 1))?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Type::Message(self.stream.clone(), deserialized),
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
                let mut buffer = vec!(0; 2);

                let packet = Packet::read_into(self.stream.clone(), packet_type[0], &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Type::ChangeRoom(self.stream.clone(), deserialized),
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
            3 => { // FIGHT
                Some(Type::Fight(self.stream.clone(), Fight::default()))
            },
            4 => { // PVPFIGHT
                let mut buffer = vec!(0; 32);

                let packet = Packet::read_into(self.stream.clone(), packet_type[0], &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Type::PVPFight(self.stream.clone(), deserialized),
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
            5 => { // LOOT
                let mut buffer = vec!(0; 32);

                let packet = Packet::read_into(self.stream.clone(), packet_type[0], &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Type::Loot(self.stream.clone(), deserialized),
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
            6 => { // START
                Some(Type::Start(self.stream.clone(), Start::default()))
            },
            7 => { // ERROR
                let mut buffer = vec!(0; 3);

                let _ = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (1, 2))?;

                // Ignore this packet, the clients shouldn't be sending us this
                None
            },
            8 => { // ACCEPT
                let mut buffer = vec!(0; 1);

                let _ = Packet::read_into(self.stream.clone(), packet_type[0], &mut buffer)?;

                // Ignore this packet, the clients shouldn't be sending us this
                None
            },
            9 => { // ROOM
                let mut buffer = vec!(0; 36);

                let _ = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (34, 35))?; // Consueme all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            },
            10 => { // CHARACTER
                let mut buffer = vec!(0; 47);

                let packet = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (45, 46))?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Type::Character(self.stream.clone(), deserialized),
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
            11 => { // GAME
                let mut buffer = vec!(0; 6); 

                let _ = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (4, 5))?; // Consueme all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            },
            12 => { // LEAVE
                Some(Type::Leave(self.stream.clone(), Leave::default()))
            },
            13 => { // CONNECTION
                let mut buffer = vec!(0; 36);

                let _ = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (34, 35))?; // Consueme all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            },
            14 => { // VERSION
                let mut buffer = vec!(0; 4);

                let _ = Packet::read_extended(self.stream.clone(), packet_type[0], &mut buffer, (2, 3))?; // Consueme all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            },
            _ => {
                // Invalid packet type
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
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