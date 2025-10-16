use serde::Serialize;
use std::sync::mpsc::Sender;
use std::{env, io};
use tracing::{error, info};

use crate::logic::ExtendedProtocol;

#[derive(Serialize)]
pub struct Action {
    pub kind: Box<str>,
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

pub fn input(sender: Sender<ExtendedProtocol>) -> ! {
    let prefix = env::var("CMD_PREFIX").expect("[INPUT] CMD_PREFIX must be set");

    info!("[INPUT] Listening for commands with prefix: '{}'", prefix);

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

        info!("[INPUT] Parsing command.");

        // Sanitize and Tokenize
        let input = input[prefix.len()..].trim().to_string();
        let argv: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        let argc = argv.len();

        let kind = argv[0].to_ascii_lowercase();

        sender
            .send(ExtendedProtocol::Command(Action {
                kind: kind.into(),
                argv,
                argc,
            }))
            .unwrap_or_else(|_| {
                error!("[INPUT] Failed to send command packet");
            })
    }
}
