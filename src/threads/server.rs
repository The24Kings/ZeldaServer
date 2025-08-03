use std::sync::{Arc, Mutex, mpsc::Receiver};

use crate::protocol::{
    ServerMessage,
    error::ErrorCode,
    map::Map,
    packet::{
        pkt_accept::Accept,
        pkt_character::{Character, CharacterFlags},
        pkt_connection::Connection,
        pkt_error::Error,
        pkt_room,
    },
    send,
};

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

                        send(ServerMessage::Error(
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
                send(ServerMessage::Message(author.clone(), content.clone())).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send message packet: {}", e);
                });
            }
            ServerMessage::ChangeRoom(author, content) => {
                println!("[SERVER] Received: \n{:#?}", content);

                // Find the player in the map
                let (_, player) = match map.players.iter_mut().enumerate().find(|(_, player)| {
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
                        Error::new(ErrorCode::BadRoom, "Player is already in the room"),
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
                            Error::new(ErrorCode::BadRoom, "Room not found!"),
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
                        Error::new(ErrorCode::BadRoom, "Invalid connection!"),
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
                    Some(room) => room.clone(),
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");

                        send(ServerMessage::Error(
                            author.clone(),
                            Error::new(ErrorCode::BadRoom, "Room not found!"),
                        ))
                        .unwrap_or_else(|e| {
                            eprintln!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Add the player to the new room
                println!("[SERVER] Adding player to new room");
                // new_room.players.push(player_entry.0); FIXME: I need to find a better way to handle where players are

                // Send the room to the client
                send(ServerMessage::Room(
                    author.clone(),
                    pkt_room::Room::from(&next_room),
                ))
                .unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send room packet: {}", e);
                });

                // TODO: Send all players and monsters in the room

                // Remove the player from their old room (Must be here to allow the mutable borrow to end)
                // match map
                //     .rooms
                //     .iter_mut()
                //     .find(|room| room.room_number == player_entry.1.current_room)
                // {
                //     Some(room) => {
                //         println!("[SERVER] Removing player from old room");
                //         room.players FIXME: I need to find a better way to handle where players are
                //             .retain(|&player_index| player_index != player_entry.0);
                //     }
                //     None => {
                //         eprintln!("");

                //         continue;
                //     }
                // }

                println!("[SERVER] Updating player room");
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

                // Alert all players in the room of the character change
                map.alert_room(content.room_number, player_clone)
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
                        Connection::from(new_room),
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
                    Error::new(ErrorCode::NoPlayerCombat, "No player combat allowed"),
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

                let player = match map.players.iter_mut().find(|player| {
                    player
                        .author
                        .as_ref()
                        .map_or(false, |a| Arc::ptr_eq(a, &author))
                }) {
                    Some(player) => player,
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

                // Now that the mutable borrow of map (player) is done, we can call alert_room
                map.alert_room(0, player_clone.clone()).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to alert players: {}", e);
                });

                map.broadcast(format!("{} has started the game!", player_clone.name))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to broadcast message: {}", e);
                    });

                // Send Starting room to the client
                let room = match map.rooms.iter_mut().find(|room| room.room_number == 0) {
                    Some(room) => room.clone(),
                    None => {
                        eprintln!("[SERVER] Unable to find room in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Add the player to the room
                // println!("[SERVER] Adding player to starting room");
                // room.players.push(player_idx); FIXME: I need to find a better way to handle where players are

                send(ServerMessage::Room(
                    author.clone(),
                    pkt_room::Room::from(&room),
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
                        Connection::from(room),
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

                    send(ServerMessage::Error(
                        author.clone(),
                        Error::new(ErrorCode::StatError, "Invalid stats"),
                    ))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue; // Skip this iteration and wait for the next packet
                }

                // Check if the player is already in the map
                match map
                    .players
                    .iter_mut()
                    .find(|player| player.name == content.name)
                {
                    Some(player) => {
                        if player.flags.started {
                            eprintln!("[SERVER] Player is already in the game");

                            send(ServerMessage::Error(
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

                        // Remove the player from the room they left off in FIXME: I need to find a better way to handle where players are
                        // match map
                        //     .rooms
                        //     .iter_mut()
                        //     .enumerate()
                        //     .find(|(_, room)| room.room_number == player.current_room)
                        // {
                        //     Some((index, room)) => {
                        //         println!("[SERVER] Removing player from old room");
                        //         room.players.retain(|&player_index| player_index != index);
                        //     }
                        //     None => {
                        //         eprintln!(
                        //             "[SERVER] Unable to find where teh player left off in the map"
                        //         );
                        //     }
                        // }

                        println!("[SERVER] Found character in map, reactivating character.");
                    }
                    None => {
                        map.add_player(Character::from(Some(author.clone()), &content));

                        println!("[SERVER] Added character to map.");
                    }
                };

                map.alert_room(content.current_room, content.clone())
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to alert players: {}", e);
                    });

                // Send the accept packet to the client
                send(ServerMessage::Accept(author.clone(), Accept::new(10))).unwrap_or_else(|e| {
                    eprintln!("[SERVER] Failed to send accept packet: {}", e);
                });

                // Send the character back to the client (we changed the flags and junk)
                // We just added the player to the map, so we need to send the updated character
                let player = match map
                    .players
                    .iter()
                    .find(|player| player.name == content.name)
                {
                    Some(player) => player,
                    None => &mut Character::default(),
                };

                send(ServerMessage::Character(author.clone(), player.clone())).unwrap_or_else(
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
                    Some(player) => player,
                    None => {
                        eprintln!("[SERVER] Unable to find player in map");
                        continue; // Skip this iteration and wait for the next packet
                    }
                };

                // Reset the player's flags safely
                player.flags = CharacterFlags::deactivate(false);
                player.author = None;

                // Clone the player's name before the mutable borrow ends
                let player_name = player.name.clone();

                println!(
                    "[SERVER] Found character in map, resetting flags and disabling connection."
                );

                map.broadcast(format!("{} has left the game.", player_name))
                    .unwrap_or_else(|e| {
                        eprintln!("[SERVER] Failed to broadcast message: {}", e);
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
