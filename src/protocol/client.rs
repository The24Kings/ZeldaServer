use std::io::Read;
use std::sync::mpsc::Sender;
use tracing::info;

use super::packet::{Packet, Parser, pkt_fight, pkt_leave, pkt_start};
use super::{ServerMessage, Stream, pkt_type::PktType};

#[derive(Debug, Clone)]
pub struct Client {
    pub stream: Stream,
    pub sender: Sender<ServerMessage>,
}

impl Client {
    pub fn new(stream: Stream, sender: Sender<ServerMessage>) -> Self {
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

        info!("Read packet type: {}", packet_type);

        // Match the type of the packet to the enum Type
        let packet: Option<ServerMessage> = match packet_type {
            PktType::Message => {
                let mut buffer = vec![0; 66];

                let packet = Packet::read_extended(&self.stream, packet_type, &mut buffer, (0, 1))?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => ServerMessage::Message(self.stream.clone(), deserialized),
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
            PktType::ChangeRoom => {
                let mut buffer = vec![0; 2];

                let packet = Packet::read_into(&self.stream, packet_type, &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => {
                        ServerMessage::ChangeRoom(self.stream.clone(), deserialized)
                    }
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
            PktType::Fight => Some(ServerMessage::Fight(
                self.stream.clone(),
                pkt_fight::Fight::default(),
            )),
            PktType::PVPFight => {
                let mut buffer = vec![0; 32];

                let packet = Packet::read_into(&self.stream, packet_type, &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => ServerMessage::PVPFight(self.stream.clone(), deserialized),
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
            PktType::Loot => {
                let mut buffer = vec![0; 32];

                let packet = Packet::read_into(&self.stream, packet_type, &mut buffer)?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => ServerMessage::Loot(self.stream.clone(), deserialized),
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
            PktType::Start => Some(ServerMessage::Start(
                self.stream.clone(),
                pkt_start::Start::default(),
            )),
            PktType::Error => {
                let mut buffer = vec![0; 3];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (1, 2))?;

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::Accept => {
                let mut buffer = vec![0; 1];

                let _ = Packet::read_into(&self.stream, packet_type, &mut buffer)?;

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::Room => {
                let mut buffer = vec![0; 36];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (34, 35))?; // Consume all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::Character => {
                let mut buffer = vec![0; 47];

                let packet =
                    Packet::read_extended(&self.stream, packet_type, &mut buffer, (45, 46))?;

                let object = match Parser::deserialize(packet) {
                    Ok(deserialized) => ServerMessage::Character(self.stream.clone(), deserialized),
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
            PktType::Game => {
                let mut buffer = vec![0; 6];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (4, 5))?; // Consume all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::Leave => {
                // LEAVE
                Some(ServerMessage::Leave(
                    self.stream.clone(),
                    pkt_leave::Leave::default(),
                ))
            }
            PktType::Connection => {
                let mut buffer = vec![0; 36];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (34, 35))?; // Consume all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::Version => {
                let mut buffer = vec![0; 4];

                let _ = Packet::read_extended(&self.stream, packet_type, &mut buffer, (2, 3))?; // Consume all data in stream

                // Ignore this packet, the clients shouldn't be sending us this
                None
            }
            PktType::Default => {
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
                self.sender.send(pkt).map_err(|e| {
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
