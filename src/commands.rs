use std::io;
use std::sync::mpsc::Sender;

use serde::Serialize;
use tracing::{error, info};

use crate::protocol::Protocol;

#[derive(Serialize)]
pub enum ActionKind {
    BROADCAST,
    HELP,
    MESSAGE,
    NUKE,
    OTHER,
}

#[derive(Serialize)]
pub struct Action {
    pub kind: ActionKind,
    pub argv: Vec<String>,
    pub argc: usize,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Action".to_string())
        )
    }
}

pub fn input(sender: Sender<Protocol>) -> ! {
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

        if !input.starts_with("!") {
            continue;
        }

        info!("[COMMAND] Parsing command.");

        // Sanitize and Tokenize
        let input = input[1..].trim().to_string().to_ascii_lowercase();
        let argv: Vec<String> = input.split(' ').map(|s| s.to_string()).collect();
        let argc = argv.len();

        let kind = match argv[0].as_str() {
            "broadcast" => ActionKind::BROADCAST,
            "help" => ActionKind::HELP,
            "message" => ActionKind::MESSAGE,
            "nuke" => ActionKind::NUKE,
            _ => ActionKind::OTHER,
        };

        sender
            .send(Protocol::Command(Action { kind, argv, argc }))
            .unwrap_or_else(|_| {
                error!("[COMMAND] Failed to send command packet");
            })
    }
}
