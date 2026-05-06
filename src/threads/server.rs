use lurk_sansio::{ClientId, GameEngine};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use std::time::Instant;
use tracing::{debug, warn};

use crate::logic::command::handle_command;
use crate::logic::execute::execute_output;
use crate::logic::translate::translate;
use crate::logic::{Config, ExtendedProtocol};

pub fn server(
    receiver: Arc<Mutex<Receiver<ExtendedProtocol>>>,
    config: Arc<Config>,
    mut engine: GameEngine,
) -> ! {
    let mut clients: HashMap<ClientId, Arc<TcpStream>> = HashMap::new();

    loop {
        let packet = match receiver.lock().unwrap().recv() {
            Ok(packet) => packet,
            Err(e) => {
                warn!("Error receiving packet: {}", e);
                continue;
            }
        };

        let start = Instant::now();

        match packet {
            ExtendedProtocol::Connect(id, stream) => {
                debug!("Registering {}", id);
                clients.insert(id, stream);
            }
            ExtendedProtocol::Client(id, protocol) => {
                if let Some(input) = translate(id, protocol) {
                    engine.handle_input(input);
                }
            }
            ExtendedProtocol::Command(action) => {
                handle_command(&mut engine, &clients, &config, action);
            }
        }

        // Drain all outputs and execute IO
        while let Some(output) = engine.poll_output() {
            execute_output(&output, &clients, &engine);
        }

        let delta = start.elapsed();
        debug!(
            "Took: {}.{} seconds to process packet.",
            delta.as_secs(),
            delta.subsec_nanos()
        );
    }
}
