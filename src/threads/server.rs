use std::sync::{Arc, Mutex, mpsc::Receiver};

use crate::protocol::{
    Type,
    error::ErrorCode,
    map::Map,
    packet::{
        accept::Accept,
        character::{Character, CharacterFlags},
        error::Error,
    },
    send,
};

pub fn server(receiver: Arc<Mutex<Receiver<Type>>>, map: &mut Map) {
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
                if let Some(player) = map.find_player_conn(content.author.clone()) {
                    player.flags.started = true;

                    send(Type::Character(player.clone())).unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send character packet: {}", e);
                    });
                }
            }
            Type::Character(content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Check to make sure the character's stats are valid
                let total_stats = content.attack + content.defense + content.regen;

                // Send an error packet if the stats are invalid
                if total_stats > map.init_points {
                    send(Type::Error(Error {
                        author: content.author.clone(),
                        message_type: 7,
                        error: ErrorCode::StatError,
                        message_len: 13,
                        message: "Invalid stats".to_string(),
                    }))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration and wait for the next packet
                }

                // Check if the character is already in the map reset the flags
                // If not, add it to the map in the starting room
                if let Some(player) = map.find_player(content.name.clone()) {
                    player.flags = CharacterFlags::default();

                    println!("[SERVER] Found character in map, resetting flags.");
                } else {
                    map.add_player(Character::new(content.clone()));

                    println!("[SERVER] Added character to map!");
                }

                // Send an accept packet to the client
                send(Type::Accept(Accept {
                    author: content.author.clone(),
                    message_type: 8,
                    accept_type: 10,
                }))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send accept packet: {}", e);
                });

                // Send the character back to the client with the new info (flags, staring room, etc)
                send(Type::Character(
                    map.find_player(content.name.clone())
                        .map(|player| player.clone())
                        .unwrap_or_default(), // We just added this character, so it should be in the map, but just in case
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send character packet: {}", e);
                });
            }
            Type::Leave(content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the character in the map and mark it as inactive, not ready, not started, and do not join battle
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
