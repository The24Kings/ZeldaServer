use std::sync::{Arc, Mutex, mpsc::Receiver};

use crate::protocol::packet::{
    pkt_accept, pkt_character, pkt_character::CharacterFlags, pkt_connection, pkt_error, pkt_room,
};
use crate::protocol::{ServerMessage, error::ErrorCode, map::Map, pkt_type::PktType, send};

pub fn server(receiver: Arc<Mutex<Receiver<ServerMessage>>>, map: &mut Map) {
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
            ServerMessage::Message(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Check to see if the recipient is a player in the map
                let player = match map
                    .players
                    .iter_mut()
                    .find(|player| player.name == content.recipient)
                {
                    Some(player) => player,
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");

                        send(ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::Other, "Player not found"),
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

                        send(ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(
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
                send(ServerMessage::Message(author.clone(), content.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send message packet: {}", e);
                });
            }
            ServerMessage::ChangeRoom(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Find the player in the map
                let (player_idx, player) =
                    match map.players.iter_mut().enumerate().find(|(_, player)| {
                        player
                            .author
                            .as_ref()
                            .map_or(false, |a| Arc::ptr_eq(a, &author))
                    }) {
                        Some((index, player)) => {
                            println!("[SERVER] Found player at index: {}", index);
                            (index, player)
                        }
                        None => {
                            eprintln!("[SERVER] Unable to find player in map");
                            continue; // Skip this iteration and wait for the next packet
                        }
                    };

                // Check if the player is already in the room
                if player.current_room == content.room_number {
                    eprintln!("[SERVER] Player is already in the room");

                    send(ServerMessage::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BadRoom, "Player is already in the room"),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration and wait for the next packet
                }

                // Check if the room is a valid connection
                let room = match map
                    .rooms
                    .iter()
                    .find(|room| room.room_number == player.current_room)
                {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");

                        send(ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BadRoom, "Room not found!"),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Check if the room has the requested connection
                let valid_connection = room
                    .connections
                    .iter()
                    .any(|exit| exit.room_number == content.room_number);

                if !valid_connection {
                    eprintln!(
                        "[SERVER] Invalid connection... Room only has: {:?}",
                        &room.connections
                    );

                    send(ServerMessage::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BadRoom, "Invalid connection!"),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration and wait for the next packet
                }

                // Find the next room in the map
                let next_room = match map
                    .rooms
                    .iter_mut()
                    .find(|room| room.room_number == content.room_number)
                {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");

                        send(ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BadRoom, "Room not found!"),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Add the player to the new room
                println!("[SERVER] Adding player to new room");
                next_room.players.push(player_idx);

                // Send the room to the client
                send(ServerMessage::Room(
                    author.clone(),
                    pkt_room::Room::from(next_room.clone()),
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send room packet: {}", e);
                });

                // TODO: Send all players and monsters in the room

                // Remove the player from their old room (Must be here to allow the mutable borrow to end)
                match map
                    .rooms
                    .iter_mut()
                    .find(|room| room.room_number == player.current_room)
                {
                    Some(room) => {
                        println!("[SERVER] Removing player from old room");
                        room.players.retain(|&index| index != player_idx);
                    }
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");

                        continue;
                    }
                }

                println!("[SERVER] Updating player room");
                let old_room = player.current_room;
                player.current_room = content.room_number;

                // Clone the player here to end the mutable borrow before alert_room
                let player_clone = player.clone();

                // Send the updated character back to the client
                send(ServerMessage::Character(
                    author.clone(),
                    player_clone.clone(),
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send character packet: {}", e);
                });

                // Alert all players in the room of the character leaving
                map.alert_room(old_room, &player_clone).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to alert players: {}", e);
                });

                // Alert all players in the new room of the character entering
                map.alert_room(content.room_number, &player_clone)
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to alert players: {}", e);
                    });

                // Send all connections to the client
                let connections = match map.get_exits(content.room_number) {
                    Some(exits) => exits,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                for new_room in connections {
                    send(ServerMessage::Connection(
                        author.clone(),
                        pkt_connection::Connection::from(new_room),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send connection packet: {}", e);
                    });
                }
            }
            ServerMessage::Fight(_author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                //TODO: Fight logic

                /*
                    Find the character in the map

                    Get all the monsters in the room, when fight it called you are
                    challenging all the monsters in the room...

                    Commence the fight; damage is calculated and sent back to the client
                    If the character is dead, send a message to the client and mark the character as dead and broadcast
                    the message to all players in the room
                */
            }
            ServerMessage::PVPFight(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Disabled, no player combat allowed
                send(ServerMessage::Error(
                    author.clone(),
                    pkt_error::Error::new(ErrorCode::NoPlayerCombat, "No player combat allowed"),
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send error packet: {}", e);
                });
            }
            ServerMessage::Loot(_author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);
                //TODO: Loot logic
                // Find the character in the map and the thing being looted
                // Loot the thing and send both the updated character and the looted thing back to the client
            }
            ServerMessage::Start(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Find the player in the map
                let (player_idx, player) =
                    match map.players.iter_mut().enumerate().find(|(_, player)| {
                        player
                            .author
                            .as_ref()
                            .map_or(false, |a| Arc::ptr_eq(a, &author))
                    }) {
                        Some((index, player)) => {
                            println!("[SERVER] Found player at index: {}", index);
                            (index, player)
                        }
                        None => {
                            eprintln!("[SERVER] Unable to find player in map");
                            continue; // Skip this iteration and wait for the next packet
                        }
                    };

                // Update the player with the new stats
                player.flags.started = true;

                // Clone the player here to end the mutable borrow before alert_room
                let player_clone = player.clone();

                // Send the updated character back to the client
                send(ServerMessage::Character(
                    author.clone(),
                    player_clone.clone(),
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send character packet: {}", e);
                });

                // Now that the mutable borrow of map (player) is done, we can call alert_room and alert all players in the starting room
                map.alert_room(0, &player_clone).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to alert players: {}", e);
                });

                map.broadcast(format!("{} has started the game!", player_clone.name))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to broadcast message: {}", e);
                    });

                // Send Starting room to the client
                let room = match map.rooms.iter_mut().find(|room| room.room_number == 0) {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Add the player to the room
                println!("[SERVER] Adding player to starting room");
                room.players.push(player_idx);

                send(ServerMessage::Room(
                    author.clone(),
                    pkt_room::Room::from(room.clone()),
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send room packet: {}", e);
                });

                //TODO: Send all players and monsters in the room

                // Send all connections to the client
                let connections = match map.get_exits(0) {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                for room in connections {
                    send(ServerMessage::Connection(
                        author.clone(),
                        pkt_connection::Connection::from(room),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send connection packet: {}", e);
                    });
                }
            }
            ServerMessage::Character(author, content) => {
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
                        send(ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::StatError, "Invalid stats"),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Check if the total stats are within the limit
                if total_stats > map.init_points {
                    println!("[SERVER] Invalid stats: {}", total_stats);

                    send(ServerMessage::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::StatError, "Invalid stats"),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration and wait for the next packet
                }

                // Check if their are stat points left and equally distribute them
                let mut updated_content = content.clone();

                if total_stats < map.init_points && content.attack < 1
                    || content.defense < 1
                    || content.regen < 1
                {
                    println!("[SERVER] Distributing remaining stat points");

                    // Distribute the remaining stat points equally
                    updated_content.attack += (map.init_points - total_stats) / 3;
                    updated_content.defense += (map.init_points - total_stats) / 3;
                    updated_content.regen += (map.init_points - total_stats) / 3;
                }

                // Check if the player is already in the map
                let (player_idx, player) = match map
                    .players
                    .iter_mut()
                    .enumerate()
                    .find(|(_, player)| player.name == content.name)
                {
                    Some((idx, player)) => {
                        println!("[SERVER] Found character in map, reactivating character.");
                        (idx, player)
                    }
                    None => {
                        map.add_player(pkt_character::Character::from(
                            Some(author.clone()),
                            &updated_content,
                        ));
                        println!("[SERVER] Added character to map.");

                        // Now get the newly added player
                        map.players
                            .iter_mut()
                            .enumerate()
                            .find(|(_, player)| player.name == content.name)
                            .expect("[SERVER] Player just added should exist") // If this fails, something is wrong
                    }
                };

                if player.flags.started {
                    eprintln!("[SERVER] Player is already in the game");

                    send(ServerMessage::Error(
                        author.clone(),
                        pkt_error::Error::new(
                            ErrorCode::PlayerExists,
                            "Player is already in the game; please leave the game and rejoin",
                        ),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration
                }

                // Reset the player's flags safely and update the author
                player.flags = CharacterFlags::activate(false);
                player.author = Some(author.clone());

                // Clone the player here to end the mutable borrow before alert_room and sending to the client
                let updated_player = player.clone();

                // Remove the player from their old room (Must be here to allow the mutable borrow to end)
                match map
                    .rooms
                    .iter_mut()
                    .find(|room| room.room_number == player.current_room)
                {
                    Some(room) => {
                        println!("[SERVER] Removing player from old room");
                        room.players.retain(|&index| index != player_idx);
                    }
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");

                        continue;
                    }
                }

                // Send the accept packet to the client
                send(ServerMessage::Accept(
                    author.clone(),
                    pkt_accept::Accept::new(PktType::Character),
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send accept packet: {}", e);
                });

                // Send the character back to the client (we changed the flags and junk)
                // We just added the player to the map, so we need to send the updated character
                send(ServerMessage::Character(author.clone(), updated_player)).unwrap_or_else(
                    |e| {
                        eprintln!("[SERVER] Failed to send character packet: {}", e);
                    },
                );
            }
            ServerMessage::Leave(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                let player = match map.players.iter_mut().find(|player| {
                    player
                        .author
                        .as_ref()
                        .map_or(false, |a| Arc::ptr_eq(a, &author))
                }) {
                    Some(player) => {
                        println!(
                            "[SERVER] Found character in map, resetting flags and disabling connection."
                        );
                        player
                    }
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Reset the player's flags safely
                player.flags = CharacterFlags::deactivate(false);
                player.author = None;

                // Clone the player's name before the mutable borrow ends
                let player_clone = player.clone();

                // Tell the server that the player has left
                map.broadcast(format!("{} has left the game.", player_clone.name))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to broadcast message: {}", e);
                    });

                // Alert all players in the room of the character leaving so they can update their UI
                map.alert_room(player_clone.current_room, &player_clone)
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to alert players: {}", e);
                    });

                // Attempt to shutdown the connection
                match author.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => {
                        println!("[SERVER] Connection shutdown successfully");
                    }
                    Err(e) => {
                        eprintln!("[SERVER] Failed to shutdown connection: {}", e);
                    }
                }
            }
            ServerMessage::Error(_, _) => {}
            ServerMessage::Accept(_, _) => {}
            ServerMessage::Room(_, _) => {}
            ServerMessage::Game(_, _) => {}
            ServerMessage::Connection(_, _) => {}
            ServerMessage::Version(_, _) => {}
        }
    }
}
