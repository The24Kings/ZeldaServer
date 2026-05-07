use lurk_lcsc::Protocol;
use lurk_sansio::{ClientId, GameEngine};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, mpsc::Receiver};
use std::time::Instant;
use tracing::{debug, warn};

use crate::logic::command::handle_command;
use crate::logic::execute::{disconnect_client, execute_output};
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

        let disconnect_id = match packet {
            ExtendedProtocol::Connect(id, stream) => {
                debug!("Registering {}", id);
                clients.insert(id, stream);
                None
            }
            ExtendedProtocol::Client(id, protocol) => {
                let is_leave = matches!(protocol, Protocol::Leave(_));
                if let Some(input) = translate(id, protocol) {
                    engine.handle_input(input);
                }
                if is_leave { Some(id) } else { None }
            }
            ExtendedProtocol::Command(action) => {
                handle_command(&mut engine, &clients, &config, action);
                None
            }
        };

        let delta = start.elapsed();
        let duration = delta.as_secs() as f64 + delta.subsec_nanos() as f64 / 1_000_000_000.0;

        debug!("Took: {:.9} seconds to process packet.", duration);

        let start = Instant::now();

        // Drain all outputs and execute IO
        while let Some(output) = engine.poll_output() {
            execute_output(&output, &mut clients, &engine);
        }

        let delta = start.elapsed();
        let duration = delta.as_secs() as f64 + delta.subsec_nanos() as f64 / 1_000_000_000.0;

        debug!("Took: {:.9} seconds to process I/O.", duration);

        // Ensure disconnection even if the engine didn't emit Output::Disconnect
        if let Some(id) = disconnect_id {
            disconnect_client(&id, &mut clients);
        }
    }
}
