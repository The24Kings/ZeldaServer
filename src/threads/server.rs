use std::sync::{Arc, Mutex, mpsc::Receiver};

use crate::protocol::{
    Type,
    error::ErrorCode,
    map::Map,
    packet::{
        accept::Accept,
        character::{Character, CharacterFlags},
        connection::Connection,
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
            Type::Message(_author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the recipient in the map and send the message to them
                // If the recipient is not found, send an error packet back to the sender
            }
            Type::ChangeRoom(_author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the character in the map, move it to the new room if possible
                // and send the updated character back to the client
                // Alert the other players in the room about the change and alert all players in the new room
            }
            Type::Fight(_author, content) => {
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
            Type::PVPFight(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Disabled, no player combat allowed
                send(Type::Error(
                    author.clone(),
                    Error::new(ErrorCode::NoPlayerCombat, "No player combat allowed"),
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send error packet: {}", e);
                });
            }
            Type::Loot(_author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                // Find the character in the map and the thing being looted
                // Loot the thing and send both the updated character and the looted thing back to the client
            }
            Type::Start(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                let player = match map.find_player_conn(&author) {
                    Some(player) => player,
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Update the player with the new stats
                player.flags.started = true;

                // Send the updated character back to the client
                send(Type::Character(author.clone(), player.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send character packet: {}", e);
                });

                // Send Starting room to the client
                let room = match map.find_room(0) {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                send(Type::Room(author.clone(), room.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send room packet: {}", e);
                });

                //TODO: Send all players and monsters in the room
                //TODO: Alert all players in the room that a new player has joined

                // Send all connections to the client
                let connections = match map.get_exits(0) {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                for room in connections {
                    send(Type::Connection(author.clone(), Connection::from(&room))).unwrap_or_else(
                        |e| {
                            eprintln!("[SERVER] Failed to send connection packet: {}", e);
                        },
                    );
                }
            }
            Type::Character(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Check to make sure the character's stats are valid
                let total_stats = content
                    .attack
                    .checked_add(content.defense)
                    .and_then(|sum| sum.checked_add(content.regen));

                let total_stats = match total_stats {
                    Some(total) => total,
                    None => {
                        println!("[SERVER] Overflow in stats calculation");
                        send(Type::Error(
                            author.clone(),
                            Error::new(ErrorCode::StatError, "Invalid stats"),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                if total_stats > map.init_points {
                    println!("[SERVER] Invalid stats: {}", total_stats);

                    send(Type::Error(
                        author.clone(),
                        Error::new(ErrorCode::StatError, "Invalid stats"),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration and wait for the next packet
                }

                // Check if the player is already in the map
                match map.find_player(&content.name) {
                    Some(player) => {
                        if player.flags.started {
                            eprintln!("[SERVER] Player is already in the game");

                            send(Type::Error(
                                author.clone(),
                                Error::new(ErrorCode::PlayerExists, "Player is already in the game; please leave the game and rejoin"),
                            ))
                            .unwrap_or_else(|e| {
                                eprintln!("[SERVER] Failed to send error packet: {}", e);
                            });

                            continue; // Skip this iteration
                        }

                        // Reset the player's flags safely
                        player.flags = CharacterFlags::activate(false);

                        // Update with new connection data
                        player.author = author.clone();

                        println!("[SERVER] Found character in map, resetting flags.");
                    }
                    None => {
                        let mut new_player = content.clone();

                        new_player.flags = CharacterFlags::default();
                        map.add_player(new_player);

                        println!("[SERVER] Added character to map.");
                    }
                };

                // Send the accept packet to the client
                send(Type::Accept(author.clone(), Accept::new(10))).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send accept packet: {}", e);
                });

                // Send the character back to the client (we changed the flags and junk)
                // We just added the player to the map, so we need to send the updated character
                let player = match map.find_player(&content.name) {
                    Some(player) => player,
                    None => &mut Character::default(),
                };

                send(Type::Character(author.clone(), player.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send character packet: {}", e);
                });
            }
            Type::Leave(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                let player = match map.find_player_conn(&author) {
                    Some(player) => player,
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Reset the player's flags safely
                player.flags = CharacterFlags::deactivate(false);

                println!("[SERVER] Found character in map, resetting flags.");
            }
            Type::Error(_, _) => {}
            Type::Accept(_, _) => {}
            Type::Room(_, _) => {}
            Type::Game(_, _) => {}
            Type::Connection(_, _) => {}
            Type::Version(_, _) => {}
        }
    }
}
