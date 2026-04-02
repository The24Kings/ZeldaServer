use lurk_lcsc::{CharacterFlags, PktLeave};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::logic::GameState;

impl GameState {
    pub fn handle_leave(&mut self, author: Arc<TcpStream>, content: PktLeave) {
        info!("Received: {}", content);

        // ================================================================================
        // Grab the player and deactivate them, extract name for later lookups
        // ================================================================================
        let (player_name, current_room) = {
            let Some((_, player)) = self.player_from_stream(&author) else {
                return;
            };

            player.flags = CharacterFlags::empty();
            player.author = None;

            (player.name.clone(), player.current_room)
        };

        // ================================================================================
        // Alert the server and the room
        // ================================================================================
        let Some(room) = self.rooms.get(&current_room) else {
            warn!("Unable to find where the player left off in the map");
            return;
        };

        self.broadcast(format!("{} has left the game.", player_name));

        if let Some(player) = self.players.get(&player_name) {
            self.alert_room(room, player);
        }

        match author.shutdown(std::net::Shutdown::Both) {
            Ok(_) => info!("Connection shutdown successfully"),
            Err(e) => error!("Failed to shutdown connection: {}", e),
        }
    }
}
