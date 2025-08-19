use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use tracing::{debug, error, info, warn};

use crate::commands::ActionKind;
use crate::config::Config;
use crate::protocol::game::{self, Room};
use crate::protocol::packet::{
    pkt_accept, pkt_character, pkt_character::CharacterFlags, pkt_connection, pkt_error,
    pkt_message, pkt_room,
};
use crate::protocol::{Protocol, error::ErrorCode, pkt_type::PktType};

pub fn server(
    receiver: Arc<Mutex<Receiver<Protocol>>>,
    config: Arc<Config>,
    rooms: &mut HashMap<u16, Room>,
) {
    let mut players: HashMap<String, pkt_character::Character> = HashMap::new();

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
                let player = match players.get(&content.recipient) {
                    Some(player) => player,
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::OTHER, "Player not found"),
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
                                ErrorCode::OTHER,
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
                let player = match game::player_from_stream(&mut players, author.clone()) {
                    Some((_, player)) => player,
                    None => {
                        error!("[SERVER] Unable to find player in map");
                        continue;
                    }
                };

                let cur_room_id = player.current_room;
                let nxt_room_id = content.room_number;

                // ================================================================================
                // Check to make sure the player exists, is in the given room, and can move to the
                // given connection. Shuffle the player around to the next room and send data.
                // ================================================================================
                if cur_room_id == nxt_room_id {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BADROOM, "Player is already in the room"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }
                // Check if the room is a valid connection
                let cur_room = match rooms.get_mut(&cur_room_id) {
                    Some(room) => room,
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BADROOM, "Room not found!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                match cur_room.connections.get(&nxt_room_id) {
                    Some(exit) => {
                        info!("[SERVER] Found connection: '{}'", exit.title);
                    }
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BADROOM, "Invalid connection!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                }

                info!("[SERVER] Setting current room to: {}", nxt_room_id);
                player.current_room = nxt_room_id;

                info!("[SERVER] Removing player from old room");
                cur_room.players.retain(|name| *name != player.name);

                // Find the next room in the map, add the player, and send it off
                let new_room = match rooms.get_mut(&nxt_room_id) {
                    Some(room) => room,
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BADROOM, "Room not found!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                info!("[SERVER] Adding player to new room");
                new_room.players.push(player.name.clone());

                Protocol::Room(author.clone(), pkt_room::Room::from(new_room.clone()))
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send room packet: {}", e);
                    });

                // Ends mutable borrow of new_room
                let room_players = new_room.players.clone();
                let room_monsters = new_room.monsters.clone();
                // ^ ============================================================================ ^

                // ================================================================================
                // Update the player data and send it to the client
                // ================================================================================
                info!("[SERVER] Updating player room");

                // Send the updated character back to the client
                Protocol::Character(author.clone(), player.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send character packet: {}", e);
                    });
                // ^ ============================================================================ ^

                // ================================================================================
                // Send all connections to the client
                // ================================================================================
                let connections = match game::exits(&rooms, nxt_room_id) {
                    Some(exits) => exits,
                    None => {
                        error!("[SERVER] No exits for room {}", nxt_room_id);
                        continue;
                    }
                };

                for (_, new_room) in connections {
                    Protocol::Connection(
                        author.clone(),
                        pkt_connection::Connection::from(&new_room),
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
                let player = player.clone(); // End mutable borrow of player

                game::alert_room(&players, &rooms, cur_room_id, &player).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });

                game::alert_room(&players, &rooms, content.room_number, &player).unwrap_or_else(
                    |e| {
                        warn!("[SERVER] Failed to alert players: {}", e);
                    },
                );
                // ^ ============================================================================ ^

                // ================================================================================
                // Send the all players and monsters in the room excluding the author
                // ================================================================================
                let players = room_players.iter().filter_map(|name| players.get(name));

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
                    pkt_error::Error::new(ErrorCode::NOPLAYERCOMBAT, "No player combat allowed"),
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
                let player = match game::player_from_stream(&mut players, author.clone()) {
                    Some((_, player)) => {
                        info!("[SERVER] Found player '{}'", player.name);
                        player
                    }
                    None => {
                        error!("[SERVER] Unable to find player in map");
                        continue;
                    }
                };

                // ================================================================================
                // Activate the character and send the information off to client
                // ================================================================================
                player.flags |= CharacterFlags::STARTED;

                let player = player.clone(); // End mutable borrow of player

                Protocol::Character(author.clone(), player.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send character packet: {}", e);
                    });

                game::alert_room(&players, &rooms, 0, &player).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });

                game::broadcast(&players, format!("{} has started the game!", player.name))
                    .unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to broadcast message: {}", e);
                    });
                // ^ ============================================================================ ^

                // ================================================================================
                // Send the starting room and connections to the client
                // ================================================================================
                let room = match rooms.get_mut(&0) {
                    Some(room) => room,
                    None => {
                        error!("[SERVER] Unable to find room in map");
                        continue;
                    }
                };

                info!("[SERVER] Adding player to starting room");

                room.players.push(player.name);

                // End mutable borrow of room
                let room_players = room.players.clone();
                let room_monsters = room.monsters.clone();

                Protocol::Room(author.clone(), pkt_room::Room::from(room.clone()))
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send room packet: {}", e);
                    });

                let connections = match game::exits(&rooms, 0) {
                    Some(exits) => exits,
                    None => {
                        error!("[SERVER] Unable to find room in map");
                        continue;
                    }
                };

                for (_, room) in connections {
                    Protocol::Connection(author.clone(), pkt_connection::Connection::from(&room))
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send connection packet: {}", e);
                        });
                }
                // ^ ============================================================================ ^

                // ================================================================================
                // Send the all players and monsters in the room excluding the author
                // ================================================================================
                let players = room_players.iter().filter_map(|name| players.get(name));

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
                    .unwrap_or(config.initial_points + 1); // This will cause the next check to fail

                if total_stats > config.initial_points {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::STATERROR, "Invalid stats"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                let mut updated_content = content.clone();

                if total_stats < config.initial_points && content.attack < 1
                    || content.defense < 1
                    || content.regen < 1
                {
                    info!("[SERVER] Distributing remaining stat points");

                    // Distribute the remaining stat points equally
                    updated_content.attack += (config.initial_points - total_stats) / 3;
                    updated_content.defense += (config.initial_points - total_stats) / 3;
                    updated_content.regen += (config.initial_points - total_stats) / 3;
                }
                // ^ ============================================================================ ^

                // ================================================================================
                // Check if the player has already been created (Primary Key -> Name).
                // Create a new player and return it if not.
                // We ignore the flags from the client and set the correct ones accordingly.
                // Store the old room so that we may remove the player later and set ignore input room
                // ================================================================================
                let player = match players.get_mut(&content.name) {
                    Some(player) => {
                        info!("[SERVER] Reactivating character.");
                        info!("[SERVER] Player left off in: {}", player.current_room);

                        player
                    }
                    None => {
                        info!("[SERVER] Adding character to map.");

                        game::add_player(
                            &mut players,
                            pkt_character::Character::to_default(&updated_content),
                        );

                        // Now get the newly added player
                        players.get_mut(&updated_content.name).unwrap() // Should never panic because we JUST added this player to the map...
                    }
                };

                if player.flags.contains(CharacterFlags::STARTED) {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(
                            ErrorCode::PLAYEREXISTS,
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

                player.flags =
                    CharacterFlags::ALIVE | CharacterFlags::READY | CharacterFlags::BATTLE;
                player.author = Some(author.clone());
                player.current_room = 0; // Start in the first room
                // ^ ============================================================================ ^

                // ================================================================================
                // Send an Accept packet and updated character.
                // ================================================================================
                Protocol::Accept(author.clone(), pkt_accept::Accept::new(PktType::CHARACTER))
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

                let player = player.clone(); // End mutable borrow of player

                let room = match rooms.get_mut(&old_room_number) {
                    Some(room) => room,
                    None => {
                        warn!("[SERVER] Unable to find where the player left off in the map");
                        continue;
                    }
                };

                room.players.retain(|name| name != &player.name);

                game::message_room(
                    &players,
                    &rooms,
                    old_room_number,
                    format!("{}'s corpse disappeared into a puff of smoke.", player.name),
                )
                .unwrap_or_else(|e| {
                    error!("[SERVER] Failed to message room: {}", e);
                });

                game::alert_room(&players, &rooms, old_room_number, &player).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });
                // ^ ============================================================================ ^
            }
            Protocol::Leave(author, content) => {
                info!("[SERVER] Received: {}", content);

                // ================================================================================
                // Grab the player and deactivate them, alert the server and the room that the player
                // has been deactivated, but is technically still there.
                // Shutdown the connection.
                // ================================================================================
                let player = match game::player_from_stream(&mut players, author.clone()) {
                    Some((_, player)) => player,
                    None => continue,
                };

                player.flags = CharacterFlags::empty();
                player.author = None;

                let player = player.clone(); // End mutable borrow of player

                game::broadcast(&players, format!("{} has left the game.", player.name))
                    .unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to broadcast message: {}", e);
                    });

                game::alert_room(&players, &rooms, player.current_room, &player).unwrap_or_else(
                    |e| {
                        warn!("[SERVER] Failed to alert players: {}", e);
                    },
                );

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
            Protocol::Command(action) => {
                info!("[SERVER] Received: {}", action);

                match action.kind {
                    ActionKind::BROADCAST => {
                        info!("Placeholder broadcast command!");
                    }
                    ActionKind::HELP => {
                        info!("{}", config.help_cmd);
                    }
                    ActionKind::MESSAGE => {
                        if action.argc < 3 {
                            error!("Message command requires at least 2 arguments");
                            continue;
                        }

                        let name = action.argv[1].clone();
                        let content = action.argv[2..].join(" ");

                        let recipient = players.get(&name).map(|p| p.author.clone()).flatten();

                        match recipient {
                            Some(recipient) => {
                                Protocol::Message(
                                    recipient.clone(),
                                    pkt_message::Message {
                                        message_type: PktType::MESSAGE,
                                        message_len: content.len() as u16,
                                        recipient: name,
                                        sender: "Server".to_string(),
                                        narration: false,
                                        message: content,
                                    },
                                )
                                .send()
                                .unwrap_or_else(|e| {
                                    error!("[SERVER] Failed to send message packet: {}", e);
                                });
                            }
                            None => {
                                error!("[SERVER] Player not found: {}", action.argv[1]);
                            }
                        }
                    }
                    ActionKind::NUKE => {
                        info!("Placeholder nuke command!");
                    }
                    ActionKind::OTHER => {
                        error!("Unsupported command!");
                    }
                }
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
