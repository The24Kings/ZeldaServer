use lurk_lcsc::LurkError;
use lurk_lcsc::PktFight;
use lurk_lcsc::send_error;
use lurk_lcsc::{PktCharacter, PktError};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::map;
use crate::logic::state::GameState;

impl GameState {
    pub fn handle_fight(&mut self, author: Arc<TcpStream>, content: PktFight) {
        info!("Received: {}", content);

        // Find the player in the map
        let player = match map::player_from_stream(&mut self.players, author.clone()) {
            Some((_, player)) => player,
            None => {
                error!("Unable to find player in map");
                return;
            }
        };

        if !GameState::ensure_started(player, &author) {
            return;
        }

        // ================================================================================
        // Collect all players that will join us in battle, then get the target monster,
        // check if they exists and are dead
        // ================================================================================
        let mut attacker = player.clone();
        let current_room = player.current_room;

        let mut room = match self.rooms.get_mut(&current_room) {
            Some(room) => room.clone(), // To allow me to message the whole room without borrow checker issues
            None => {
                error!("Room not found");
                return;
            }
        };

        room.players.retain(|player| player != &attacker.name); // Remove attacker for narration purposes

        let in_battle: Vec<Arc<str>> = self
            .players
            .iter()
            .filter(|(_, p)| p.flags.is_battle() && p.current_room == current_room)
            .map(|(name, _)| name.clone())
            .collect();

        let monsters = match self.rooms.get_mut(&current_room) {
            Some(room) => &mut room.monsters,
            None => {
                error!("Player isn't in a valid room");
                return;
            }
        };

        let to_attack = match monsters {
            Some(monsters) => monsters
                .iter_mut()
                .filter(|m| m.health > 0)
                .min_by_key(|m| (m.health, m.name.clone())),
            None => {
                send_error!(
                    author.clone(),
                    PktError::new(LurkError::NOFIGHT, "The room is eerily quiet...")
                );

                return;
            }
        };

        let to_attack = match to_attack {
            Some(m) => m,
            None => {
                send_error!(
                    author.clone(),
                    PktError::new(LurkError::NOFIGHT, "No monsters alive. Let them rest.")
                );

                return;
            }
        };

        info!("Battling '{}'", to_attack.name);
        info!("{} player(s) joining the battle", in_battle.len());

        map::message_room(
            &self.players,
            &room,
            format!("{} is attacking {}", attacker.name, to_attack.name),
            false,
        );

        // ================================================================================
        // Calculate the fight logic: Action Phase!
        // ================================================================================
        let players_in_battle: Vec<_> = in_battle
            .iter()
            .filter_map(|name| self.players.get(name))
            .collect();
        let mut victory = false;

        map::message_room(
            &self.players,
            &room,
            format!(
                "Joining '{}' in attacking '{}'",
                attacker.name, to_attack.name
            ),
            false,
        );

        let damage = players_in_battle
            .iter()
            .map(|player| player.attack)
            .sum::<u16>()
            .saturating_sub(to_attack.defense);
        let damage = damage.try_into().unwrap_or(i16::MAX); // We went out of bounds on damage, cap to i16 MAX int

        to_attack.health = to_attack.health.saturating_sub(damage);

        info!("'{}' dealt {} damage", attacker.name, damage);

        if to_attack.health <= 0 {
            victory = true;

            info!("'{}' defeated '{}'", attacker.name, to_attack.name);
        }

        // ================================================================================
        // Calculate the fight logic: Defense Phase!
        // ================================================================================
        if !victory {
            let damage = to_attack.attack.saturating_sub(attacker.defense);
            let damage = damage.try_into().unwrap_or(i16::MAX); // We went out of bounds on damage, cap to i16 MAX int

            attacker.health = attacker.health.saturating_sub(damage);

            info!(
                "'{}' took {} damage from '{}'",
                attacker.name, damage, to_attack.name
            );

            if attacker.health <= 0 {
                info!("'{}' killed '{}'", to_attack.name, attacker.name);
            }
        }

        // ================================================================================
        // Calculate the fight logic: End Phase!
        // ================================================================================
        if attacker.flags.is_alive() {
            let regen = attacker.regen.try_into().unwrap_or(i16::MAX);

            info!("'{}' regenerated: {}", attacker.name, regen);

            attacker.health = attacker.health.saturating_add(regen); // We went out of bounds on regen, cap to i16 MAX int
        }

        // ================================================================================
        // Update player HashMap with new stats and send all the updated players/ monster
        // to client
        // ================================================================================
        info!("Updating players in fight");

        let attacker_name = attacker.name.clone();
        let _ = self.players.insert(attacker_name.clone(), attacker); // Move, not clone

        for name in &in_battle {
            if let Some(player) = self.players.get(name) {
                let _ = self.players.insert(name.clone(), player.clone());
            }
        }

        let monster_pkt: PktCharacter = to_attack.into();
        let to_update = in_battle.iter().filter_map(|name| self.players.get(name));

        room.players.push(attacker_name); // Add the name back so the attacker gets updated

        for player in to_update {
            self.alert_room(&room, player);
        }

        self.alert_room(&room, &monster_pkt);
    }
}
