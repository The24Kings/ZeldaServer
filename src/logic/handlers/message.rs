use lurk_lcsc::{LurkError, PktError, PktMessage, send_error, send_message};
use std::{net::TcpStream, sync::Arc};
use tracing::info;

use crate::logic::state::GameState;

impl GameState {
    pub fn handle_message(&self, author: Arc<TcpStream>, content: PktMessage) {
        info!("Received: {}", content);

        // ================================================================================
        // Get the recipient player and their connection fd to send them the message.
        // ================================================================================
        let Some(player) = self.players.get(content.recipient.as_ref()) else {
            send_error!(
                author.clone(),
                PktError::new(LurkError::OTHER, "Player not found")
            );

            return;
        };

        if !GameState::ensure_started(player, &author) {
            return;
        }

        let Some(author) = &player.author else {
            send_error!(
                author.clone(),
                PktError::new(LurkError::OTHER, "Not connected")
            );

            return;
        };

        send_message!(author.clone(), content);
    }
}
