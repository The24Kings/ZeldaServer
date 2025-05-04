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
            Type::Message(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Check to see if the recipient is a player in the map
                let player = match map.players.iter_mut().find(|player| player.name == content.recipient) {
                    Some(player) => player,
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");

                        send(Type::Error(
                            author.clone(),
                            Error::new(ErrorCode::Other, "Player not found"),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Check if the recipient has an active connection
                let author = match &player.author {
                    Some(author) => author,
                    None => {
                        eprintln!("[SERVER] Character does not have an active connection");

                        send(Type::Error(
                            author.clone(),
                            Error::new(
                                ErrorCode::Other,
                                "Character does not have an active connection",
                            ),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Send the message to the recipient
                send(Type::Message(author.clone(), content.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send message packet: {}", e);
                });
            }
            Type::ChangeRoom(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Find the player in the map and change their room
                let player = match map.players.iter_mut().find(|player| {
                    player.author.as_ref().map_or(false, |a| Arc::ptr_eq(a, &author))
                }) {
                    Some(player) => player,
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");
                        continue;
                    }
                };

                // Check if the player is already in the room
                if player.current_room == content.room_number {
                    eprintln!("[SERVER] Player is already in the room");

                    send(Type::Error(
                        author.clone(),
                        Error::new(ErrorCode::BadRoom, "Player is already in the room"),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration and wait for the next packet
                }

                // Find the room in the map
                let room = match map.rooms.iter().find(|room| room.room_number == content.room_number) {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");

                        send(Type::Error(
                            author.clone(),
                            Error::new(ErrorCode::BadRoom, "Room not found!"),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Check that the room has connections
                let connection_ids = match &room.connections {
                    Some(ids) => ids,
                    None => {
                        eprintln!("[SERVER] Room has no connections");

                        send(Type::Error(
                            author.clone(),
                            Error::new(ErrorCode::BadRoom, "Room has no connections!"),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Check if the room is a valid connection
                if !connection_ids.contains(&player.current_room) {
                    eprintln!("[SERVER] Invalid connection");

                    send(Type::Error(
                        author.clone(),
                        Error::new(ErrorCode::BadRoom, "Invalid connection!"),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration and wait for the next packet
                }

                // Update the player's current room
                player.current_room = content.room_number;

                // Send the updated character back to the client
                send(Type::Character(author.clone(), player.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send character packet: {}", e);
                });

                //TODO: Send all players and monsters in the room
                //TODO: Alert all players in the room that a new player has joined

                // Send the room to the client
                send(Type::Room(author.clone(), room.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send room packet: {}", e);
                });

                // Send all connections to the client
                let connections = match map.get_exits(content.room_number) {
                    Some(exits) => exits,
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

                let player = match map.players.iter_mut().find(|player| {
                    player.author.as_ref().map_or(false, |a| Arc::ptr_eq(a, &author))
                }) {
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
                let room = match map.rooms.iter().find(|room| room.room_number == 0) {
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
                match map.players.iter_mut().find(|player| player.name == content.name) {
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

                        // Reset the player's flags safely and update the author
                        player.flags = CharacterFlags::activate(false);
                        player.author = Some(author.clone());

                        println!("[SERVER] Found character in map, reactivating character.");
                    }
                    None => {
                        let mut new_player = content.clone();

                        new_player.flags = CharacterFlags::default();
                        map.add_player(new_player);

                        println!("[SERVER] Added character to map.");
                    }
                };

                //TODO: Alert all players in the room that a new player has joined

                // Send the accept packet to the client
                send(Type::Accept(author.clone(), Accept::new(10))).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send accept packet: {}", e);
                });

                // Send the character back to the client (we changed the flags and junk)
                // We just added the player to the map, so we need to send the updated character
                let player = match map.players.iter().find(|player| player.name == content.name) {
                    Some(player) => player,
                    None => &mut Character::default(),
                };

                send(Type::Character(author.clone(), player.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send character packet: {}", e);
                });
            }
            Type::Leave(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                let player = match map.players.iter_mut().find(|player| {
                    player.author.as_ref().map_or(false, |a| Arc::ptr_eq(a, &author))
                }) {
                    Some(player) => player,
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Reset the player's flags safely
                player.flags = CharacterFlags::deactivate(false);
                player.author = None;

                println!("[SERVER] Found character in map, resetting flags and disabling connection.");
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
