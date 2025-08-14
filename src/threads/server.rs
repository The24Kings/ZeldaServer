use std::sync::{Arc, Mutex, mpsc::Receiver};
use tracing::{debug, error, info, warn};

use crate::protocol::packet::{
    pkt_accept, pkt_character, pkt_character::CharacterFlags, pkt_connection, pkt_error, pkt_room,
};
use crate::protocol::{Protocol, error::ErrorCode, map::Map, pkt_type::PktType};

pub fn server(receiver: Arc<Mutex<Receiver<Protocol>>>, map: &mut Map) {
    loop {
        let packet = match receiver.lock().unwrap().recv() {
            Ok(packet) => packet,
            Err(e) => {
                warn!("[SERVER] Error receiving packet: {}", e);
                continue;
            }
        };

        match packet {
            Protocol::Message(author, content) => {
                info!("[SERVER] Received: {}", content);

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
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::Other, "Player not found"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                let author = match &player.author {
                    Some(author) => author,
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(
                                ErrorCode::Other,
                                "Character does not have an active connection",
                            ),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                Protocol::Message(author.clone(), content.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send message packet: {}", e);
                    });
                // ^ ============================================================================ ^
            }
            Protocol::ChangeRoom(author, content) => {
                info!("[SERVER] Received: {}", content);

                // Find the player in the map
                let (player_idx, player) =
                    match map.players.iter_mut().enumerate().find(|(_, player)| {
                        player
                            .author
                            .as_ref()
                            .map_or(false, |a| Arc::ptr_eq(a, &author))
                    }) {
                        Some((index, player)) => {
                            info!("[SERVER] Found player at index: {}", index);
                            (index, player)
                        }
                        None => {
                            error!("[SERVER] Unable to find player in map");
                            continue;
                        }
                    };

                // ================================================================================
                // Check to make sure the player exists, is in the given room, and can move to the
                // given connection. Shuffle the player around to the next room and send data.
                // ================================================================================
                if player.current_room == content.room_number {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BadRoom, "Player is already in the room"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
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
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BadRoom, "Room not found!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                let valid_connection = cur_room
                    .connections
                    .iter()
                    .any(|exit| exit.room_number == content.room_number);

                if !valid_connection {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BadRoom, "Invalid connection!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                info!("[SERVER] Removing player from old room");
                cur_room.players.retain(|&index| index != player_idx);

                // Find the next room in the map, add the player, and send it off
                let new_room = match map
                    .rooms
                    .iter_mut()
                    .find(|room| room.room_number == content.room_number)
                {
                    Some(room) => room,
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BadRoom, "Room not found!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                info!("[SERVER] Adding player to new room");
                new_room.players.push(player_idx);

                Protocol::Room(author.clone(), pkt_room::Room::from(new_room.clone()))
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send room packet: {}", e);
                    });

                let room_players = new_room.players.clone();
                let room_monsters = new_room.monsters.clone();
                // ^ ============================================================================ ^

                // ================================================================================
                // Update the player data and send it to the client
                // ================================================================================
                info!("[SERVER] Updating player room");

                let old_room = player.current_room;
                player.current_room = content.room_number;

                // Clone the player here to end the mutable borrow of map
                let player_clone = player.clone();

                // Send the updated character back to the client
                Protocol::Character(author.clone(), player_clone.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send character packet: {}", e);
                    });
                // ^ ============================================================================ ^

                // ================================================================================
                // Send all connections to the client
                // ================================================================================
                let connections = match map.exits(content.room_number) {
                    Some(exits) => exits,
                    None => {
                        error!("[SERVER] No exits for room {}", content.room_number);
                        continue;
                    }
                };

                for new_room in connections {
                    Protocol::Connection(
                        author.clone(),
                        pkt_connection::Connection::from(new_room),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send connection packet: {}", e);
                    });
                }
                // ^ ============================================================================ ^

                // ================================================================================
                // Update info for all other connected clients
                // ================================================================================
                map.alert_room(old_room, &player_clone).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });

                map.alert_room(content.room_number, &player_clone)
                    .unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to alert players: {}", e);
                    });
                // ^ ============================================================================ ^

                // ================================================================================
                // Send the all players and monsters in the room excluding the author
                // ================================================================================
                let players = room_players.iter().filter_map(|&idx| map.players.get(idx));

                debug!("[SERVER] Players: {:?}", players);

                let monsters = match &room_monsters {
                    Some(monsters) => monsters.iter(),
                    None => [].iter(),
                };

                players.for_each(|player| {
                    Protocol::Character(author.clone(), player.clone())
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send character packet: {}", e);
                        });
                });

                for monster in monsters {
                    Protocol::Character(
                        author.clone(),
                        pkt_character::Character::from_monster(monster, 0),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send character packet: {}", e);
                    });
                }
                // ^ ============================================================================ ^
            }
            Protocol::Fight(_author, content) => {
                info!("[SERVER] Received: {}", content);
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
            Protocol::PVPFight(author, content) => {
                info!("[SERVER] Received: {}", content);

                Protocol::Error(
                    author.clone(),
                    pkt_error::Error::new(ErrorCode::NoPlayerCombat, "No player combat allowed"),
                )
                .send()
                .unwrap_or_else(|e| {
                    error!("[SERVER] Failed to send error packet: {}", e);
                });
            }
            Protocol::Loot(_author, content) => {
                info!("[SERVER] Received: {}", content);
                //TODO: Loot logic
                // Find the character in the map and the thing being looted
                // Loot the thing and send both the updated character and the looted thing back to the client
            }
            Protocol::Start(author, content) => {
                info!("[SERVER] Received: {}", content);

                // Find the player in the map
                let (player_idx, player) = match map.player_from_stream(&author) {
                    Some((index, player)) => {
                        info!("[SERVER] Found player at index: {}", index);
                        (index, player)
                    }
                    None => {
                        error!("[SERVER] Unable to find player in map");
                        continue;
                    }
                };

                // ================================================================================
                // Activate the character and send the information off to client
                // ================================================================================
                player.flags.started = true;

                let player_clone = player.clone();

                Protocol::Character(author.clone(), player_clone.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send character packet: {}", e);
                    });

                map.alert_room(0, &player_clone).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });

                map.broadcast(format!("{} has started the game!", player_clone.name))
                    .unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to broadcast message: {}", e);
                    });
                // ^ ============================================================================ ^

                // ================================================================================
                // Send the starting room and connections to the client
                // ================================================================================
                let room = match map.rooms.iter_mut().find(|room| room.room_number == 0) {
                    Some(room) => room,
                    None => {
                        error!("[SERVER] Unable to find room in map");
                        continue;
                    }
                };

                info!("[SERVER] Adding player to starting room");

                room.players.push(player_idx);

                let room_players = room.players.clone();
                let room_monsters = room.monsters.clone();

                Protocol::Room(author.clone(), pkt_room::Room::from(room.clone()))
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send room packet: {}", e);
                    });

                let connections = match map.exits(0) {
                    Some(exits) => exits,
                    None => {
                        error!("[SERVER] Unable to find room in map");
                        continue;
                    }
                };

                for room in connections {
                    Protocol::Connection(author.clone(), pkt_connection::Connection::from(room))
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send connection packet: {}", e);
                        });
                }
                // ^ ============================================================================ ^

                // ================================================================================
                // Send the all players and monsters in the room excluding the author
                // ================================================================================
                let players = room_players.iter().filter_map(|&idx| map.players.get(idx));

                debug!("[SERVER] Players: {:?}", players);

                let monsters = match &room_monsters {
                    Some(monsters) => monsters.iter(),
                    None => [].iter(),
                };

                players.for_each(|player| {
                    Protocol::Character(author.clone(), player.clone())
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send character packet: {}", e);
                        });
                });

                for monster in monsters {
                    Protocol::Character(
                        author.clone(),
                        pkt_character::Character::from_monster(monster, 0),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send character packet: {}", e);
                    });
                }
                // ^ ============================================================================ ^
            }
            Protocol::Character(author, content) => {
                info!("[SERVER] Received: {}", content);

                // ================================================================================
                // Check the given stats are valid, if not all points have been allocated, do so equally.
                // ================================================================================
                let total_stats = content
                    .attack
                    .checked_add(content.defense)
                    .and_then(|sum| sum.checked_add(content.regen))
                    .unwrap_or(map.init_points + 1); // This will cause the next check to fail

                if total_stats > map.init_points {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::StatError, "Invalid stats"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                let mut updated_content = content.clone();

                if total_stats < map.init_points && content.attack < 1
                    || content.defense < 1
                    || content.regen < 1
                {
                    info!("[SERVER] Distributing remaining stat points");

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
                // Store the old room so that we may remove the player later and set ignore input room
                // ================================================================================
                let player = match map.player_from_name(&content.name) {
                    Some((_, player)) => {
                        info!("[SERVER] Reactivating character.");
                        info!("[SERVER] Player left off in: {}", player.current_room);

                        player
                    }
                    None => {
                        info!("[SERVER] Adding character to map.");

                        map.add_player(pkt_character::Character::to_default(&updated_content));

                        // Now get the newly added player
                        map.players
                            .last_mut()
                            .expect("[SERVER] Player just added should exist") // Should never fail, the previous function would panic if we run out of memory
                    }
                };

                if player.flags.started {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(
                            ErrorCode::PlayerExists,
                            "Player is already in the game.",
                        ),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                let old_room_number = player.current_room;

                player.flags = CharacterFlags::activate(false);
                player.author = Some(author.clone());
                player.current_room = 0; // Start in the first room
                // ^ ============================================================================ ^

                // ================================================================================
                // Send an Accept packet and updated character.
                // ================================================================================
                Protocol::Accept(author.clone(), pkt_accept::Accept::new(PktType::Character))
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send accept packet: {}", e);
                    });

                Protocol::Character(author.clone(), player.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send character packet: {}", e);
                    });
                // ^ ============================================================================ ^

                // ================================================================================
                // Remove the player from the room they left off in to avoid 2 players existing on
                // the map at once
                // ================================================================================
                if old_room_number == 0 {
                    continue;
                }

                match map
                    .rooms
                    .iter_mut()
                    .enumerate()
                    .find(|(_, room)| room.room_number == old_room_number)
                {
                    Some((index, room)) => {
                        info!("[SERVER] Removing player from old room");
                        room.players.retain(|&player_index| player_index != index);
                    }
                    None => {
                        warn!("[SERVER] Unable to find where the player left off in the map");
                    }
                }
                // ^ ============================================================================ ^
            }
            Protocol::Leave(author, content) => {
                info!("[SERVER] Received: {}", content);

                // ================================================================================
                // Grab the player and deactivate them, alert the server and the room that the player
                // has been deactivated, but is technically still there.
                // Shutdown the connection.
                // ================================================================================
                let player = match map.player_from_stream(&author) {
                    Some((_, player)) => player,
                    None => continue,
                };

                player.flags = CharacterFlags::deactivate(false);
                player.author = None;

                let player_clone = player.clone();

                map.broadcast(format!("{} has left the game.", player_clone.name))
                    .unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to broadcast message: {}", e);
                    });

                map.alert_room(player_clone.current_room, &player_clone)
                    .unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to alert players: {}", e);
                    });

                match author.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => {
                        info!("[SERVER] Connection shutdown successfully");
                    }
                    Err(e) => {
                        error!("[SERVER] Failed to shutdown connection: {}", e);
                    }
                }
                // ^ ============================================================================ ^
            }
            Protocol::Error(_, _) => {}
            Protocol::Accept(_, _) => {}
            Protocol::Room(_, _) => {}
            Protocol::Game(_, _) => {}
            Protocol::Connection(_, _) => {}
            Protocol::Version(_, _) => {}
        }
    }
}
