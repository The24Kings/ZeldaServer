use std::sync::{Arc, Mutex, mpsc::Receiver};

use crate::protocol::packet::{
    pkt_accept, pkt_character, pkt_character::CharacterFlags, pkt_connection, pkt_error, pkt_room,
};
use crate::protocol::{ServerMessage, error::ErrorCode, map::Map, pkt_type::PktType};

pub fn server(receiver: Arc<Mutex<Receiver<ServerMessage>>>, map: &mut Map) {
    loop {
        let packet = match receiver.lock().unwrap().recv() {
            Ok(packet) => packet,
            Err(e) => {
                eprintln!("[SERVER] Error receiving packet: {}", e);
                continue;
            }
        };

        match packet {
            ServerMessage::Message(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // ================================================================================
                // Get the recipient player and their connection fd to send them the message.
                // ================================================================================
                let player = match map
                    .players
                    .iter_mut()
                    .find(|player| player.name == content.recipient)
                {
                    Some(player) => player,
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");

                        ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::Other, "Player not found"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                let author = match &player.author {
                    Some(author) => author,
                    None => {
                        eprintln!("[SERVER] Character does not have an active connection");

                        ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(
                                ErrorCode::Other,
                                "Character does not have an active connection",
                            ),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                ServerMessage::Message(author.clone(), content.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send message packet: {}", e);
                    });
                // ^ ============================================================================ ^
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
                            continue;
                        }
                    };

                // ================================================================================
                // Check to make sure the player exists, is in the given room, and can move to the
                // given connection. Shuffle the player around to the next room and send data.
                // ================================================================================
                if player.current_room == content.room_number {
                    eprintln!("[SERVER] Player is already in the room");

                    ServerMessage::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BadRoom, "Player is already in the room"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                // Check if the room is a valid connection
                let cur_room = match map
                    .rooms
                    .iter_mut()
                    .find(|room| room.room_number == player.current_room)
                {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");

                        ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BadRoom, "Room not found!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                let valid_connection = cur_room
                    .connections
                    .iter()
                    .any(|exit| exit.room_number == content.room_number);

                if !valid_connection {
                    eprintln!(
                        "[SERVER] Invalid connection... Room only has: {:?}",
                        &cur_room.connections
                    );

                    ServerMessage::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BadRoom, "Invalid connection!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                println!("[SERVER] Removing player from old room");
                cur_room.players.retain(|&index| index != player_idx);

                // Find the next room in the map, add the player, and send it off
                match map
                    .rooms
                    .iter_mut()
                    .find(|room| room.room_number == content.room_number)
                {
                    Some(room) => {
                        // Add the player to the new room
                        println!("[SERVER] Adding player to new room");
                        room.players.push(player_idx);

                        // Send the room to the client
                        ServerMessage::Room(author.clone(), pkt_room::Room::from(room.clone()))
                            .send()
                            .unwrap_or_else(|e| {
                                eprintln!("[SERVER] Failed to send room packet: {}", e);
                            });
                    }
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");

                        ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BadRoom, "Room not found!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };
                // ^ ============================================================================ ^

                // TODO: Send all players and monsters in the room

                // ================================================================================
                // Update the player data and send it to the client
                // ================================================================================
                println!("[SERVER] Updating player room");

                let old_room = player.current_room;
                player.current_room = content.room_number;

                // Clone the player here to end the mutable borrow of map
                let player_clone = player.clone();

                // Send the updated character back to the client
                ServerMessage::Character(author.clone(), player_clone.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send character packet: {}", e);
                    });
                // ^ ============================================================================ ^

                // ================================================================================
                // Send all connections to the client
                // ================================================================================
                let connections = match map.get_exits(content.room_number) {
                    Some(exits) => exits,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue;
                    }
                };

                for new_room in connections {
                    ServerMessage::Connection(
                        author.clone(),
                        pkt_connection::Connection::from(new_room),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send connection packet: {}", e);
                    });
                }
                // ^ ============================================================================ ^

                // ================================================================================
                // Update info for all other connected clients
                // ================================================================================
                map.alert_room(old_room, &player_clone).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to alert players: {}", e);
                });

                map.alert_room(content.room_number, &player_clone)
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to alert players: {}", e);
                    });
                // ^ ============================================================================ ^
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
                ServerMessage::Error(
                    author.clone(),
                    pkt_error::Error::new(ErrorCode::NoPlayerCombat, "No player combat allowed"),
                )
                .send()
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
                            continue;
                        }
                    };

                // ================================================================================
                // Activate the character and send the information off to client
                // ================================================================================
                player.flags.started = true;

                let player_clone = player.clone();

                ServerMessage::Character(author.clone(), player_clone.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send character packet: {}", e);
                    });

                map.alert_room(0, &player_clone).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to alert players: {}", e);
                });

                map.broadcast(format!("{} has started the game!", player_clone.name))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to broadcast message: {}", e);
                    });
                // ^ ============================================================================ ^

                // ================================================================================
                // Send the starting room and connections to the client
                // ================================================================================
                let room = match map.rooms.iter_mut().find(|room| room.room_number == 0) {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue;
                    }
                };

                println!("[SERVER] Adding player to starting room");
                room.players.push(player_idx);

                ServerMessage::Room(author.clone(), pkt_room::Room::from(room.clone()))
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send room packet: {}", e);
                    });

                let connections = match map.get_exits(0) {
                    Some(room) => room,
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue;
                    }
                };

                for room in connections {
                    ServerMessage::Connection(
                        author.clone(),
                        pkt_connection::Connection::from(room),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send connection packet: {}", e);
                    });
                }
                // ^ ============================================================================ ^

                //TODO: Send all players and monsters in the room
            }
            ServerMessage::Character(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // ================================================================================
                // Check the given stats are valid, if not all points have been allocated, do so equally.
                // ================================================================================
                let total_stats = content
                    .attack
                    .checked_add(content.defense)
                    .and_then(|sum| sum.checked_add(content.regen));

                let total_stats = match total_stats {
                    Some(total) => total,
                    None => {
                        println!("[SERVER] Overflow in stats calculation");
                        ServerMessage::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::StatError, "Invalid stats"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                if total_stats > map.init_points {
                    println!("[SERVER] Invalid stats: {}", total_stats);

                    ServerMessage::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::StatError, "Invalid stats"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

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
                // ^ ============================================================================ ^

                // ================================================================================
                // Check if the player has already been created (Primary Key -> Name).
                // Create a new player and return it if not.
                // We ignore the flags from the client and set the correct ones accordingly.
                // ================================================================================
                let player = match map
                    .players
                    .iter_mut()
                    .find(|player| player.name == content.name)
                {
                    Some(player) => {
                        println!("[SERVER] Found character in map, reactivating character.");
                        player
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
                            .find(|player| player.name == content.name)
                            .expect("[SERVER] Player just added should exist") // If this fails, something is wrong
                    }
                };

                if player.flags.started {
                    eprintln!("[SERVER] Player is already in the game");

                    ServerMessage::Error(
                        author.clone(),
                        pkt_error::Error::new(
                            ErrorCode::PlayerExists,
                            "Player is already in the game; please leave the game and rejoin",
                        ),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                player.flags = CharacterFlags::activate(false);
                player.author = Some(author.clone());
                // ^ ============================================================================ ^

                // ================================================================================
                // Send an Accept packet and updated character.
                // ================================================================================
                ServerMessage::Accept(author.clone(), pkt_accept::Accept::new(PktType::Character))
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send accept packet: {}", e);
                    });

                ServerMessage::Character(author.clone(), player.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send character packet: {}", e);
                    });
                // ^ ============================================================================ ^
            }
            ServerMessage::Leave(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // ================================================================================
                // Grab the player and deactivate them, alert the server and the room that the player
                // has been deactivated, but is technically still there.
                // Shutdown the connection.
                // ================================================================================
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
                        continue;
                    }
                };

                player.flags = CharacterFlags::deactivate(false);
                player.author = None;

                let player_clone = player.clone();

                map.broadcast(format!("{} has left the game.", player_clone.name))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to broadcast message: {}", e);
                    });

                map.alert_room(player_clone.current_room, &player_clone)
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to alert players: {}", e);
                    });

                match author.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => {
                        println!("[SERVER] Connection shutdown successfully");
                    }
                    Err(e) => {
                        eprintln!("[SERVER] Failed to shutdown connection: {}", e);
                    }
                }
                // ^ ============================================================================ ^
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
