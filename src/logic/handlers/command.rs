use lurk_lcsc::send_message;
use lurk_lcsc::{PktCharacter, PktMessage};
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::commands::Action;
use crate::logic::state::GameState;

impl GameState {
    pub fn handle_command(&mut self, action: Action) {
        info!("Received: {}", action);

        match action.kind.as_ref() {
            "help" => {
                info!("{}", self.config.help_cmd);
            }
            "broadcast" => {
                if action.argv.len() < 2 {
                    error!("Broadcast command requires at least 2 arguments");
                    return;
                }

                let message = action.argv[1..].join(" ");

                self.broadcast(message);
            }
            "message" => {
                if action.argv.len() < 3 {
                    error!("Message command requires at least 3 arguments");
                    return;
                }

                let name = action.argv[1].clone();
                let content = action.argv[2..].join(" ");

                let recipient = self
                    .players
                    .get(name.as_str())
                    .and_then(|p| p.author.clone());

                let Some(recipient) = recipient else {
                    error!("Player not found: {}", action.argv[1]);
                    return;
                };

                send_message!(recipient.clone(), PktMessage::server(&name, &content));
            }
            "nuke" => {
                info!("Nuke command received, removing disconnected players");

                let to_remove: Vec<Arc<str>> = self
                    .players
                    .iter()
                    .filter(|(_, player)| player.author.is_none())
                    .map(|(name, _)| name.clone())
                    .collect();

                if to_remove.is_empty() {
                    info!("No disconnected players");
                    return;
                }

                // Remove from main list and room lists
                self.players.retain(|name, _| !to_remove.contains(name));
                for room in self.rooms.values_mut() {
                    room.players.retain(|name| !to_remove.contains(name));
                }

                info!("Removed {} disconnected players", to_remove.len());

                self.broadcast(String::from(
                    "Disconnected players have been removed; ChangeRoom to update player list!",
                ));
            }
            "revive" => {
                info!("Revive command received, reviving all dead monsters");

                let mut alerts = Vec::new();
                let mut revived_count = 0usize;

                for room in self.rooms.values_mut() {
                    if let Some(monsters) = &mut room.monsters {
                        let pkts: Vec<PktCharacter> = monsters
                            .iter_mut()
                            .filter(|m| m.health <= 0 && m.max_health > 0)
                            .map(|m| {
                                m.health = m.max_health;
                                PktCharacter::from(m)
                            })
                            .collect();

                        if !pkts.is_empty() {
                            revived_count += pkts.len();
                            alerts.push((room.clone(), pkts));
                        }
                    }
                }

                if revived_count == 0 {
                    info!("No monsters to revive");
                    return;
                }

                for (room, pkts) in &alerts {
                    for pkt in pkts {
                        self.alert_room(room, pkt);
                    }
                }

                self.broadcast(String::from("All dead monsters have been revived!"));
                info!("Revived {} monster(s)", revived_count);
            }
            _ => {
                error!("Unsupported command!");
            }
        }
    }
}
