use lurk_lcsc::{PktCharacter, PktMessage, PktType, send_to};
use lurk_sansio::{ClientId, GameEngine};
use serde::Serialize;
use std::net::TcpStream;
use std::sync::Arc;
use std::{collections::HashMap, io};
use tracing::{error, info};

use crate::logic::{Config, GameSender};

#[derive(Serialize)]
pub struct Command {
    pub kind: Box<str>,
    pub argv: Vec<String>,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Action".to_string())
        )
    }
}

pub fn input(sender: GameSender, prefix: String) -> ! {
    info!("Listening for commands with prefix: '{}'", prefix);

    loop {
        // Take input from the console.
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(e) => {
                error!("Could not read stdin: {e}");
                continue;
            }
        }

        if !input.starts_with(prefix.as_str()) {
            continue;
        }

        info!("Parsing command.");

        // Sanitize and Tokenize
        let input = input[prefix.len()..].trim().to_string();
        let argv: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();

        let kind = argv[0].to_ascii_lowercase().into();
        let action = Command { kind, argv };

        sender.send_cmd(action);
    }
}

/// Handle admin commands using the GameEngine API.
pub fn handle_command(
    engine: &mut GameEngine,
    clients: &HashMap<ClientId, Arc<TcpStream>>,
    config: &Config,
    action: Command,
) {
    info!("Received: {}", action);

    match action.kind.as_ref() {
        "help" => {
            info!("{}", config.help_cmd);
        }
        "broadcast" => {
            if action.argv.len() < 2 {
                error!("Broadcast command requires at least 2 arguments");
                return;
            }
            let message: Box<str> = action.argv[1..].join(" ").into();

            for (id, stream) in clients {
                let name = engine
                    .players()
                    .iter()
                    .find(|(_, ps)| ps.client == Some(*id))
                    .map(|(n, _)| n.clone());
                if let Some(name) = name {
                    let pkt = PktMessage::server(&name, &message);
                    let _ = send_to(stream.as_ref(), &pkt);
                }
            }
        }
        "message" => {
            if action.argv.len() < 3 {
                error!("Message command requires at least 3 arguments");
                return;
            }
            let name = &action.argv[1];
            let content = action.argv[2..].join(" ");

            let recipient = engine.players().get(name.as_str()).and_then(|ps| ps.client);

            let Some(client_id) = recipient else {
                error!("Player not found or disconnected: {}", name);
                return;
            };
            let Some(stream) = clients.get(&client_id) else {
                error!("No stream for {}", client_id);
                return;
            };

            let pkt = PktMessage::server(name, &content);
            let _ = send_to(stream.as_ref(), &pkt);
        }
        "nuke" => {
            info!("Nuke command received, removing disconnected players");

            let to_remove: Vec<Arc<str>> = engine
                .players()
                .iter()
                .filter(|(_, ps)| ps.client.is_none())
                .map(|(name, _)| name.clone())
                .collect();

            if to_remove.is_empty() {
                info!("No disconnected players");
                return;
            }

            engine
                .players_mut()
                .retain(|name, _| !to_remove.contains(name));
            for room in engine.rooms_mut().values_mut() {
                room.players.retain(|name| !to_remove.contains(name));
            }

            info!("Removed {} disconnected players", to_remove.len());

            let message =
                "Disconnected players have been removed; ChangeRoom to update player list!";
            for (id, stream) in clients {
                let name = engine
                    .players()
                    .iter()
                    .find(|(_, ps)| ps.client == Some(*id))
                    .map(|(n, _)| n.clone());
                if let Some(name) = name {
                    let pkt = PktMessage::server(&name, message);
                    let _ = send_to(stream.as_ref(), &pkt);
                }
            }
        }
        "revive" => {
            info!("Revive command received, reviving all dead monsters");

            let mut alerts: Vec<(u16, Vec<lurk_sansio::Character>)> = Vec::new();
            let mut revived_count = 0usize;

            for room in engine.rooms_mut().values_mut() {
                if let Some(monsters) = &mut room.monsters {
                    let revived: Vec<lurk_sansio::Character> = monsters
                        .iter_mut()
                        .filter(|m| m.health <= 0 && m.max_health > 0)
                        .map(|m| {
                            m.health = m.max_health;
                            m.to_character()
                        })
                        .collect();

                    if !revived.is_empty() {
                        revived_count += revived.len();
                        alerts.push((room.room_number, revived));
                    }
                }
            }

            if revived_count == 0 {
                info!("No monsters to revive");
                return;
            }

            for (room_number, characters) in &alerts {
                let Some(room) = engine.rooms().get(room_number) else {
                    continue;
                };
                for character in characters {
                    let pkt = PktCharacter {
                        packet_type: PktType::CHARACTER,
                        name: character.name.clone(),
                        flags: character.flags,
                        attack: character.attack,
                        defense: character.defense,
                        regen: character.regen,
                        health: character.health,
                        gold: character.gold,
                        current_room: character.current_room,
                        description_len: character.description.len() as u16,
                        description: character.description.clone(),
                    };
                    for player_name in &room.players {
                        let Some(ps) = engine.players().get(player_name) else {
                            continue;
                        };
                        let Some(client_id) = ps.client else {
                            continue;
                        };
                        let Some(stream) = clients.get(&client_id) else {
                            continue;
                        };
                        let _ = send_to(stream.as_ref(), &pkt);
                    }
                }
            }

            let message = "All dead monsters have been revived!";
            for (id, stream) in clients {
                let name = engine
                    .players()
                    .iter()
                    .find(|(_, ps)| ps.client == Some(*id))
                    .map(|(n, _)| n.clone());
                if let Some(name) = name {
                    let pkt = PktMessage::server(&name, message);
                    let _ = send_to(stream.as_ref(), &pkt);
                }
            }

            info!("Revived {} monster(s)", revived_count);
        }
        _ => {
            error!("Unsupported command!");
        }
    }
}
