use lurk_lcsc::{LurkError, PktCharacter, PktError, PktLoot};
use lurk_lcsc::{send_error, send_to};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info};

use crate::logic::GameState;

impl GameState {
    pub fn handle_loot(&mut self, author: Arc<TcpStream>, content: PktLoot) {
        info!("Received: {}", content);

        // Find the player, validate, and extract needed data
        let (player_name, current_room) = {
            let Some((name, player)) = self.player_from_stream(&author) else {
                error!("Unable to find player in map");
                return;
            };
            info!("Found player '{}'", name);

            if !GameState::ensure_started(player, &author) {
                return;
            }

            (name.clone(), player.current_room)
        };

        // ================================================================================
        // Get the target monster, check if they exists and are dead, then shuffle the
        // gold to the player.
        // ================================================================================
        let Some(room) = self.rooms.get_mut(&current_room) else {
            error!("Player isn't in a valid room");
            return;
        };

        let Some(monsters) = &mut room.monsters else {
            send_error!(
                author.clone(),
                PktError::new(LurkError::OTHER, "No monsters to loot!")
            );

            return;
        };

        let Some(to_loot) = monsters
            .iter_mut()
            .find(|m| m.name.as_ref() == content.target_name.as_ref())
        else {
            send_error!(
                author.clone(),
                PktError::new(LurkError::BADMONSTER, "Monster doesn't exist!")
            );

            return;
        };

        if to_loot.health > 0 {
            send_error!(
                author.clone(),
                PktError::new(LurkError::BADMONSTER, "Monster is still alive!")
            );

            return;
        }

        if to_loot.gold == 0 {
            send_error!(
                author.clone(),
                PktError::new(LurkError::BADMONSTER, "Monster already looted!")
            );

            return;
        }

        // Shuffle gold to player
        let gold = to_loot.gold;
        to_loot.gold = 0;

        let monster_pkt = PktCharacter::from(to_loot);

        // Re-lookup player to transfer gold
        let Some(player) = self.players.get_mut(&player_name) else {
            error!("Player disappeared during loot");
            return;
        };
        player.gold += gold;

        // ================================================================================
        // Send updated player and monster back to author
        // ================================================================================
        let _ = send_to(author.as_ref(), player);
        let _ = send_to(author.as_ref(), &monster_pkt);
    }
}
