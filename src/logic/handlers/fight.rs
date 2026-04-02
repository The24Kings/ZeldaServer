use lurk_lcsc::send_error;
use lurk_lcsc::{LurkError, PktCharacter, PktError, PktFight};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::GameState;

impl GameState {
    pub fn handle_fight(&mut self, author: Arc<TcpStream>, content: PktFight) {
        info!("Received: {}", content);

        // Find the player and extract needed data in a scoped block
        let (mut attacker, current_room) = {
            let Some((_, player)) = self.player_from_stream(&author) else {
                error!("Unable to find player in map");
                return;
            };

            if !GameState::ensure_started(player, &author) {
                return;
            }

            (player.clone(), player.current_room)
        };

        // ================================================================================
        // Collect all players that will join us in battle, then get the target monster,
        // check if they exists and are dead
        // ================================================================================
        let Some(room_ref) = self.rooms.get_mut(&current_room) else {
            error!("Room not found");
            return;
        };
        let mut room = room_ref.clone();

        room.players.retain(|player| player != &attacker.name); // Remove attacker for narration purposes

        let in_battle: Vec<Arc<str>> = self
            .players
            .iter()
            .filter(|(_, p)| p.flags.is_battle() && p.current_room == current_room)
            .map(|(name, _)| name.clone())
            .collect();

        // Find the target monster index so we can send messages
        // before acquiring mutable access for the fight.
        let target_idx = {
            let Some(monsters) = self
                .rooms
                .get(&current_room)
                .and_then(|r| r.monsters.as_ref())
            else {
                send_error!(
                    author.clone(),
                    PktError::new(LurkError::NOFIGHT, "The room is eerily quiet...")
                );
                return;
            };

            let Some((idx, _)) = monsters
                .iter()
                .enumerate()
                .filter(|(_, m)| m.health > 0)
                .min_by_key(|(_, m)| (m.health, m.name.clone()))
            else {
                send_error!(
                    author.clone(),
                    PktError::new(LurkError::NOFIGHT, "No monsters alive. Let them rest.")
                );
                return;
            };

            idx
        };

        let target_name = self.rooms[&current_room]
            .monsters
            .as_ref()
            .expect("monsters confirmed present above")[target_idx]
            .name
            .clone();

        info!("Battling '{}'", target_name);
        info!("{} player(s) joining the battle", in_battle.len());

        self.message_room(
            &room,
            format!("{} is attacking {}", attacker.name, target_name),
            false,
        );

        // ================================================================================
        // Calculate the fight logic: Action Phase!
        // ================================================================================
        let battle_damage: u16 = in_battle
            .iter()
            .filter_map(|name| self.players.get(name))
            .map(|p| p.attack)
            .sum();
        let mut victory = false;

        self.message_room(
            &room,
            format!("Joining '{}' in attacking '{}'", attacker.name, target_name),
            false,
        );

        // Now acquire mutable access to the target monster for the fight
        let to_attack = &mut self
            .rooms
            .get_mut(&current_room)
            .expect("room confirmed present above")
            .monsters
            .as_mut()
            .expect("monsters confirmed present above")[target_idx];

        let damage = battle_damage.saturating_sub(to_attack.defense);
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

        room.players.insert(attacker_name); // Add the name back so the attacker gets updated

        for player in to_update {
            self.alert_room(&room, player);
        }

        self.alert_room(&room, &monster_pkt);
    }
}
