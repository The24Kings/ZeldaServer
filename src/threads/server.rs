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
) -> ! {
    let mut players: HashMap<Arc<str>, pkt_character::Character> = HashMap::new();

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

                // TODO: If they message a monster... like the deku under the tree, it might open the door

                // ================================================================================
                // Get the recipient player and their connection fd to send them the message.
                // ================================================================================
                let player = match players.get(content.recipient.as_str()) {
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

                if !player.flags.is_started() && !player.flags.is_ready() {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::NOTREADY, "Start the game first!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

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
            } // Protocol::MESSAGE
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

                if !player.flags.is_started() && !player.flags.is_ready() {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::NOTREADY, "Start the game first!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

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

                let cur_room = cur_room.clone(); // End mutable borrow of cur_room

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

                let new_room = new_room.clone(); // End mutable borrow of new_room
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

                game::alert_room(&players, &cur_room, &player).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });

                game::alert_room(&players, &new_room, &player).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });
                // ^ ============================================================================ ^

                // ================================================================================
                // Send the all players and monsters in the room excluding the author
                // ================================================================================
                let players = new_room.players.iter().filter_map(|name| players.get(name));

                debug!("[SERVER] Players: {:?}", players);

                players.for_each(|player| {
                    Protocol::Character(author.clone(), player.clone())
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send character packet: {}", e);
                        });
                });

                let monsters = match &new_room.monsters {
                    Some(monsters) => monsters.iter(),
                    None => [].iter(),
                };

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
            } // Protocol::CHANGEROOM
            Protocol::Fight(author, content) => {
                info!("[SERVER] Received: {}", content);

                // Find the player in the map
                let player = match game::player_from_stream(&mut players, author.clone()) {
                    Some((_, player)) => player,
                    None => {
                        error!("[SERVER] Unable to find player in map");
                        continue;
                    }
                };

                if !player.flags.is_started() && !player.flags.is_ready() {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::NOTREADY, "Start the game first!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                // ================================================================================
                // Collect all players that will join us in battle, then get the target monster,
                // check if they exists and are dead
                // ================================================================================
                let mut attacker = player.clone();
                let current_room = player.current_room;

                let mut room = match rooms.get_mut(&current_room) {
                    Some(room) => room.clone(), // To allow me to message the whole room without borrow checker issues
                    None => {
                        error!("[SERVER] Room not found");
                        continue;
                    }
                };

                room.players.retain(|player| player != &attacker.name); // Remove attacker for narration purposes

                let in_battle: Vec<Arc<str>> = players
                    .iter()
                    .filter(|(_, p)| p.flags.is_battle() && p.current_room == current_room)
                    .map(|(name, _)| name.clone())
                    .collect();

                let monsters = match rooms.get_mut(&current_room) {
                    Some(room) => &mut room.monsters,
                    None => {
                        error!("[SERVER] Player isn't in a valid room");
                        continue;
                    }
                };

                let to_attack = match monsters {
                    Some(monsters) => monsters
                        .iter_mut()
                        .filter(|m| m.health > 0)
                        .min_by_key(|m| (m.health, m.name.clone())),
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(
                                ErrorCode::NOFIGHT,
                                "The room is eerily quiet...",
                            ),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                let to_attack = match to_attack {
                    Some(m) => m,
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::NOFIGHT, "Let the dead rest."),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                info!("[SERVER] Battling '{}'", to_attack.name);
                info!("[SERVER] {} player(s) joining the battle", in_battle.len());

                game::message_room(
                    &players,
                    &room,
                    format!("{} is attacking {}", attacker.name, to_attack.name),
                    false,
                )
                .unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to send message: {}", e);
                });
                // ^ ============================================================================ ^

                // ================================================================================
                // Calculate the fight logic: Action Phase!
                // ================================================================================
                let players_in_battle: Vec<_> = in_battle
                    .iter()
                    .filter_map(|name| players.get(name))
                    .collect();
                let mut victory = false;

                game::message_room(
                    &players,
                    &room,
                    format!(
                        "Joining '{}' in attacking '{}'",
                        attacker.name, to_attack.name
                    ),
                    false,
                )
                .unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to send message: {}", e);
                });

                let damage = players_in_battle
                    .iter()
                    .map(|player| player.attack)
                    .sum::<u16>()
                    .saturating_sub(to_attack.defense);
                let damage = damage.try_into().unwrap_or(i16::MAX); // We went out of bounds on damage, cap to i16 MAX int

                to_attack.health = to_attack.health.saturating_sub(damage);

                info!("[SERVER] '{}' dealt {} damage", attacker.name, damage);

                if to_attack.health <= 0 {
                    victory = true;

                    info!("[SERVER] '{}' defeated '{}'", attacker.name, to_attack.name);
                }
                // ^ ============================================================================ ^

                // ================================================================================
                // Calculate the fight logic: Defense Phase!
                // ================================================================================
                if !victory {
                    let damage = to_attack.attack.saturating_sub(attacker.defense);
                    let damage = damage.try_into().unwrap_or(i16::MAX); // We went out of bounds on damage, cap to i16 MAX int

                    attacker.health = attacker.health.saturating_sub(damage);

                    info!(
                        "[SERVER] '{}' took {} damage from '{}'",
                        attacker.name, damage, to_attack.name
                    );

                    if attacker.health <= 0 {
                        info!("[SERVER] '{}' killed '{}'", to_attack.name, attacker.name);
                    }
                }
                // ^ ============================================================================ ^

                // ================================================================================
                // Calculate the fight logic: End Phase!
                // ================================================================================
                if attacker.flags.is_alive() {
                    let regen = attacker.regen.try_into().unwrap_or(i16::MAX);

                    info!("[SERVER] '{}' regenerated: {}", attacker.name, regen);

                    attacker.health = attacker.health.saturating_add(regen); // We went out of bounds on regen, cap to i16 MAX int
                }
                // ^ ============================================================================ ^

                // ================================================================================
                // Update player HashMap with new stats and send all the updated players/ monster
                // to client
                // ================================================================================
                info!("[SERVER] Updating players in fight");

                let _ = players.insert(attacker.name.clone(), attacker.clone());

                for name in &in_battle {
                    if let Some(player) = players.get(name) {
                        let _ = players.insert(name.clone(), player.clone());
                    }
                }

                let to_update = in_battle.iter().filter_map(|name| players.get(name));

                room.players.push(attacker.name.clone()); // Add the name back so the attacker gets updated

                for player in to_update {
                    game::alert_room(&players, &room, player).unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to alert players: {}", e);
                    });
                }

                game::alert_room(
                    &players,
                    &room,
                    &pkt_character::Character::from_monster(&to_attack, current_room),
                )
                .unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });
                // ^ ============================================================================ ^
            } // Protocol::FIGHT
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
            } // Protocol::PVPFIGHT
            Protocol::Loot(author, content) => {
                info!("[SERVER] Received: {}", content);

                // Find the player in the map
                let player = match game::player_from_stream(&mut players, author.clone()) {
                    Some((name, player)) => {
                        info!("[SERVER] Found player '{}'", name);
                        player
                    }
                    None => {
                        error!("[SERVER] Unable to find player in map");
                        continue;
                    }
                };

                if !player.flags.is_started() && !player.flags.is_ready() {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::NOTREADY, "Start the game first!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                // ================================================================================
                // Get the target monster, check if they exists and are dead, then shuffle the
                // gold to the player.
                // ================================================================================
                let monsters = match rooms.get_mut(&player.current_room) {
                    Some(room) => &mut room.monsters,
                    None => {
                        error!("[SERVER] Player isn't in a valid room");
                        continue;
                    }
                };

                let to_loot = match monsters {
                    Some(monsters) => monsters
                        .iter_mut()
                        .find(|m| m.name.as_ref() == content.target_name.as_str()),
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::OTHER, "No monsters to loot!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                let to_loot = match to_loot {
                    Some(m) => m,
                    None => {
                        Protocol::Error(
                            author.clone(),
                            pkt_error::Error::new(ErrorCode::BADMONSTER, "Monster doesn't exist!"),
                        )
                        .send()
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to send error packet: {}", e);
                        });

                        continue;
                    }
                };

                if to_loot.health > 0 {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BADMONSTER, "Monster is still alive!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });
                }

                if to_loot.gold == 0 {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::BADMONSTER, "Monster already looted!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

                // Shuffle gold to player
                let gold = to_loot.gold;

                to_loot.gold = 0;
                player.gold += gold;
                // ^ ============================================================================ ^

                // ================================================================================
                // Send updated player and monster back to author
                // ================================================================================
                Protocol::Character(author.clone(), player.clone())
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send character packet: {}", e);
                    });

                Protocol::Character(
                    author.clone(),
                    pkt_character::Character::from_monster(to_loot, player.current_room),
                )
                .send()
                .unwrap_or_else(|e| {
                    error!("[SERVER] Failed to send character packet: {}", e);
                });

                // ^ ============================================================================ ^
            } // Protocol::LOOT
            Protocol::Start(author, content) => {
                info!("[SERVER] Received: {}", content);

                // Find the player in the map
                let player = match game::player_from_stream(&mut players, author.clone()) {
                    Some((name, player)) => {
                        info!("[SERVER] Found player '{}'", name);
                        player
                    }
                    None => {
                        error!("[SERVER] Unable to find player in map");
                        continue;
                    }
                };

                if !player.flags.is_ready() {
                    Protocol::Error(
                        author.clone(),
                        pkt_error::Error::new(ErrorCode::NOTREADY, "Supply of valid player first!"),
                    )
                    .send()
                    .unwrap_or_else(|e| {
                        error!("[SERVER] Failed to send error packet: {}", e);
                    });

                    continue;
                }

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

                game::alert_room(&players, &room, &player).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });

                game::broadcast(&players, format!("{} has started the game!", player.name))
                    .unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to broadcast message: {}", e);
                    });

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
            } // Protocol::START
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
                let name = content.name.clone();

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

                updated_content.flags = CharacterFlags::reset(); // Ignore player provided stats
                // ^ ============================================================================ ^

                // ================================================================================
                // Add the player to the map and get a mutable ref to it
                // We ignore the flags from the client and set the correct ones accordingly.
                // Store the old room so that we may remove the player later and set ignore input room
                // ================================================================================
                let player = match players.get_mut(&name) {
                    Some(player) => {
                        info!("[SERVER] Obtained player");
                        player
                    }
                    None => {
                        info!("[SERVER] Could not find player; inserting and trying again");
                        let _ = players.insert(name.clone(), updated_content.clone());

                        players.get_mut(&name).unwrap() // We just inserted so this is okay; we want to panic if insert fails
                    }
                };

                if player.flags.is_started() {
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
                    &room,
                    format!("{}'s corpse disappeared into a puff of smoke.", player.name),
                    true,
                )
                .unwrap_or_else(|e| {
                    error!("[SERVER] Failed to message room: {}", e);
                });

                game::alert_room(&players, &room, &player).unwrap_or_else(|e| {
                    warn!("[SERVER] Failed to alert players: {}", e);
                });
                // ^ ============================================================================ ^
            } // Protocol::CHARACTER
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

                let room = match rooms.get(&player.current_room) {
                    Some(room) => room,
                    None => {
                        warn!("[SERVER] Unable to find where the player left off in the map");
                        continue;
                    }
                };

                game::broadcast(&players, format!("{} has left the game.", player.name))
                    .unwrap_or_else(|e| {
                        warn!("[SERVER] Failed to broadcast message: {}", e);
                    });

                game::alert_room(&players, &room, &player).unwrap_or_else(|e| {
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
            } // Protocol::LEAVE
            Protocol::Command(action) => {
                info!("[SERVER] Received: {}", action);

                match action.kind {
                    ActionKind::HELP => {
                        info!("{}", config.help_cmd);
                    }
                    ActionKind::BROADCAST => {
                        if action.argc < 2 {
                            error!("[SERVER] Broadcast command requires at least 2 arguments");
                            continue;
                        }

                        let message = action.argv[1..].join(" ");

                        game::broadcast(&players, message).unwrap_or_else(|e| {
                            error!("[SERVER] Failed to broadcast message: {}", e);
                        });
                    }
                    ActionKind::MESSAGE => {
                        if action.argc < 3 {
                            error!("[SERVER] Message command requires at least 3 arguments");
                            continue;
                        }

                        let name = action.argv[1].clone();
                        let content = action.argv[2..].join(" ");

                        let recipient = players
                            .get(name.as_str())
                            .map(|p| p.author.clone())
                            .flatten();

                        match recipient {
                            Some(recipient) => {
                                Protocol::Message(
                                    recipient.clone(),
                                    pkt_message::Message::server(&name, &content),
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
                        info!("[SERVER] Nuke command received, removing disconnected players");

                        let to_remove: Vec<Arc<str>> = players
                            .iter()
                            .filter(|(_, player)| player.author.is_none())
                            .map(|(name, _)| name.clone())
                            .collect();

                        // Remove from main list
                        players.retain(|name, _| !to_remove.contains(name));

                        // Remove from room list
                        for room in rooms.values_mut() {
                            room.players.retain(|name| !to_remove.contains(name));
                        }

                        if to_remove.len() == 0 {
                            continue;
                        }

                        info!("[SERVER] Removed {} disconnected players", to_remove.len());

                        game::broadcast(
                            &players,
                            "Disconnected players have been removed; ChangeRoom to update player list!".to_string(),
                        )
                        .unwrap_or_else(|e| {
                            error!("[SERVER] Failed to broadcast message: {}", e);
                        });
                    }
                    ActionKind::OTHER => {
                        error!("[SERVER] Unsupported command!");
                    }
                }
            } // Protocol::COMMAND
            Protocol::Error(_, _) => {}
            Protocol::Accept(_, _) => {}
            Protocol::Room(_, _) => {}
            Protocol::Game(_, _) => {}
            Protocol::Connection(_, _) => {}
            Protocol::Version(_, _) => {}
        }
    }
}
