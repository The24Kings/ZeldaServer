use serde::Serialize;
use std::io;
use std::sync::mpsc::Sender;
use tracing::{error, info};

use crate::logic::ExtendedProtocol;
use crate::send_ext_cmd;

#[derive(Serialize)]
pub struct Action {
    pub kind: Box<str>,
    pub argv: Vec<String>,
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

pub fn input(sender: Sender<ExtendedProtocol>, prefix: String) -> ! {
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
        let action = Action { kind, argv };

        send_ext_cmd!(sender, action);
    }
}
