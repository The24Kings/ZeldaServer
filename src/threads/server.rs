use std::sync::{Arc, Mutex, mpsc::Receiver};

use crate::protocol::{error::ErrorCode, map::Map, packet::error::Error, send, Type};

pub fn server(receiver: Arc<Mutex<Receiver<Type>>>, _map: &Map) {
    loop {
        // Wait for a packet from the receiver
        let packet = match receiver.lock().unwrap().recv() {
            Ok(packet) => packet,
            Err(e) => {
                eprintln!("[SERVER] Error receiving packet: {}", e);
                continue; // Skip this iteration and wait for the next packet
            }
        };

        // Match the type of the packet to the enum Type
        match packet {
            Type::Message(content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the recipient in the map and send the message to them
                // If the recipient is not found, send an error packet back to the sender
            }
            Type::ChangeRoom(content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the character in the map, move it to the new room if possible
                // and send the updated character back to the client
                // Alert the other players in the room about the change and alert all players in the new room
            }
            Type::Fight(content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // TODO: Fight logic

                /*
                    Find the character in the map

                    Get all the monsters in the room, when fight it called you are
                    challenging all the monsters in the room...

                    Commence the fight; damage is calculated and sent back to the client
                    If the character is dead, send a message to the client and mark the character as dead and broadcast
                    the message to all players in the room
                */
            }
            Type::PVPFight(content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Disabled, no player combat allowed
                send(Type::Error(Error {
                    author: content.author.clone(),
                    message_type: 7,
                    error: ErrorCode::NoPlayerCombat,
                    message_len: 24,
                    message: "No player combat allowed".to_string(),
                }))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send error packet: {}", e);
                });
            }
            Type::Loot(content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the character in the map and the thing being looted
                // Loot the thing and send both the updated character and the looted thing back to the client
            }
            Type::Start(content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the character in the map and mark it as active send it back to the client
            }
            Type::Character(content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                /*
                   Check if the character is already in the map
                   If not, add it to the map in the starting room

                   Send the character back to the client with the new info (flags, staring room, etc)
                   If they are already in the map, just send the character back to the client with the updated flags and connection info
                */
            }
            Type::Leave(content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the character in the map and mark it as inactive
                // If the character is not in the map, just ignore it
            }
            Type::Error(_) => {}
            Type::Accept(_) => {}
            Type::Room(_) => {}
            Type::Game(_) => {}
            Type::Connection(_) => {}
            Type::Version(_) => {}
        }
    }
}
