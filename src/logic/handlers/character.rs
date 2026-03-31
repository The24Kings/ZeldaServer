use lurk_lcsc::{CharacterFlags, LurkError};
use lurk_lcsc::{PktCharacter, PktError, PktType};
use lurk_lcsc::{send_accept, send_character, send_error};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{info, warn};

use crate::logic::state::GameState;

impl GameState {
    pub fn handle_character(&mut self, author: Arc<TcpStream>, content: PktCharacter) {
        info!("Received: {}", content);

        // ================================================================================
        // Check the given stats are valid
        // ================================================================================
        let total_stats = content
            .attack
            .checked_add(content.defense)
            .and_then(|sum| sum.checked_add(content.regen))
            .unwrap_or(self.config.initial_points + 1); // This will cause the next check to fail

        if total_stats > self.config.initial_points {
            send_error!(
                author.clone(),
                PktError::new(LurkError::STATERROR, "Invalid stats")
            );

            return;
        }

        // ================================================================================
        // Add the player to the map and get a mutable ref to it
        // We ignore the flags from the client and set the correct ones accordingly.
        // Store the old room so that we may remove the player later and set ignore input room
        // ================================================================================
        let player = match self.players.get_mut(&content.name) {
            Some(player) => {
                info!("Obtained player");
                player
            }
            None => {
                info!("Could not find player; inserting and trying again");
                let _ = self.players.insert(
                    content.name.clone(),
                    PktCharacter::with_defaults_from(&content),
                );

                self.players.get_mut(&content.name).unwrap() // We just inserted so this is okay; we want to panic if insert fails
            }
        };

        if player.flags.is_started() {
            send_error!(
                author.clone(),
                PktError::new(LurkError::PLAYEREXISTS, "Player is already in the game.")
            );

            return;
        }

        let old_room_number = player.current_room;

        player.flags = CharacterFlags::alive();
        player.author = Some(author.clone());
        player.current_room = 0; // Start in the first room

        // ================================================================================
        // Send an Accept packet and updated character.
        // ================================================================================
        send_accept!(author.clone(), PktType::CHARACTER);

        send_character!(author.clone(), player.clone());

        // ================================================================================
        // Remove the player from the room they left off in to avoid 2 players existing on
        // the map at once
        // ================================================================================
        if old_room_number == 0 {
            return;
        }

        let player = player.clone(); // End mutable borrow of player

        let room = match self.rooms.get_mut(&old_room_number) {
            Some(room) => room,
            None => {
                warn!("Unable to find where the player left off in the map");
                return;
            }
        };

        room.players.retain(|name| name != &player.name);

        let room = room.clone(); // End mutable borrow of room

        self.message_room(
            &room,
            format!("{}'s corpse disappeared into a puff of smoke.", player.name),
            true,
        );

        self.alert_room(&room, &player);
    }
}
