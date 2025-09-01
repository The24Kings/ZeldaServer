use lurk_lcsc::{Packet, Parser, PktFight, PktLeave, PktStart, PktType, Protocol};
use std::sync::{Arc, mpsc::Sender};
use std::{io::Read, net::TcpStream};
use tracing::info;

use crate::logic::ExtendedProtocol;

#[derive(Debug, Clone)]
pub struct Client {
    pub stream: Arc<TcpStream>,
    pub sender: Sender<ExtendedProtocol>,
}

impl Client {
    pub fn new(stream: Arc<TcpStream>, sender: Sender<ExtendedProtocol>) -> Self {
        Client { stream, sender }
    }

    pub fn read(&self) -> Result<(), std::io::Error> {
        let mut buffer = [0; 1];
        let bytes_read = self.stream.as_ref().read(&mut buffer)?;
        let packet_type: PktType = buffer[0].into();

        if bytes_read != 1 {
            // Connection closed
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed",
            ));
        }

        info!("[CLIENT] Read packet type: {}", packet_type);

        // Match the type of the packet to the enum Type
        let packet = match packet_type {
            PktType::MESSAGE => {
                let mut buffer = vec![0; 66];

                let packet = Packet::read_extended(&self.stream, packet_type, &mut buffer, (0, 1))?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Protocol::Message(self.stream.clone(), deserialized),
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to deserialize packet: {}", e),
                        ));
                    }
                };

                // Send the packet to the sender
                Some(object)
            }
            PktType::CHANGEROOM => {
                let mut buffer = vec![0; 2];

                let packet = Packet::read_into(&self.stream, packet_type, &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Protocol::ChangeRoom(self.stream.clone(), deserialized),
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to deserialize packet: {}", e),
                        ));
                    }
                };

                // Send the packet to the sender
                Some(object)
            }
            PktType::FIGHT => {
                // Only 1 byte; no need to consume buffer
                Some(Protocol::Fight(self.stream.clone(), PktFight::default()))
            }
            PktType::PVPFIGHT => {
                let mut buffer = vec![0; 32];

                let packet = Packet::read_into(&self.stream, packet_type, &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Protocol::PVPFight(self.stream.clone(), deserialized),
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to deserialize packet: {}", e),
                        ));
                    }
                };

                // Send the packet to the sender
                Some(object)
            }
            PktType::LOOT => {
                let mut buffer = vec![0; 32];

                let packet = Packet::read_into(&self.stream, packet_type, &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Protocol::Loot(self.stream.clone(), deserialized),
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to deserialize packet: {}", e),
                        ));
                    }
                };

                // Send the packet to the sender
                Some(object)
            }
            PktType::START => {
                // Only 1 byte; no need to consume buffer
                Some(Protocol::Start(self.stream.clone(), PktStart::default()))
            }
            PktType::ERROR => {
                let mut buffer = vec![0; 3];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (1, 2))?;

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::ACCEPT => {
                let mut buffer = vec![0; 1];

                let _ = Packet::read_into(&self.stream, packet_type, &mut buffer)?;

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::ROOM => {
                let mut buffer = vec![0; 36];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (34, 35))?; // Consume all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::CHARACTER => {
                let mut buffer = vec![0; 47];

                let packet =
                    Packet::read_extended(&self.stream, packet_type, &mut buffer, (45, 46))?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => Protocol::Character(self.stream.clone(), deserialized),
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to deserialize packet: {}", e),
                        ));
                    }
                };

                // Send the packet to the sender
                Some(object)
            }
            PktType::GAME => {
                let mut buffer = vec![0; 6];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (4, 5))?; // Consume all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::LEAVE => {
                // LEAVE
                Some(Protocol::Leave(self.stream.clone(), PktLeave::default()))
            }
            PktType::CONNECTION => {
                let mut buffer = vec![0; 36];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (34, 35))?; // Consume all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::VERSION => {
                let mut buffer = vec![0; 4];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (2, 3))?; // Consume all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::DEFAULT => {
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
                self.sender.send(ExtendedProtocol::Base(pkt)).map_err(|e| {
                    // If the send fails with SendError, it means the server thread has closed
                    std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        format!("Failed to send packet: {}", e),
                    )
                })?;

                Ok(())
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "You tried to send the server a bad packet... naughty!",
            )),
        }
    }
}
