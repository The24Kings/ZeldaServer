use crate::logic::{commands::input, config::Config, map};
use crate::threads::{connection::connection, server::server};
use clap::Parser;
use std::sync::{Arc, Mutex, mpsc};
use std::{env, fs::File, net::TcpListener};
use time::{UtcOffset, format_description::parse};
use tracing::{debug, info, warn};
use tracing_subscriber::fmt::time::OffsetTime;

pub mod logic;
pub mod threads;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to bind the TCP Connection
    #[arg(short, long, default_value_t = 5051)]
    port: u16,
    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

fn main() -> ! {
    let args = Args::parse();

    dotenvy::from_filename(".env.local").expect("Failed to load .env.local file");

    // Setup tracing subscriber for logging
    let timer = parse("[year]-[month padding:zero]-[day padding:zero] [hour]:[minute]:[second]")
        .expect("Tracing time format is invalid");
    let time_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let timer = OffsetTime::new(time_offset, timer);

    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .with_line_number(true)
        .with_target(false)
        .with_timer(timer)
        .with_file(true)
        .with_ansi(true)
        .compact()
        .init();

    // Load server and client configurations
    let server_config = Arc::new(Config::load());
    let client_config = server_config.clone(); // The Arc will handle all reference counting, it's not actually cloning all the data :)

    let address = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind to address");

    info!("Listening on {address}");

    // Create a channel for communication between threads
    let (tx, rx) = mpsc::channel();
    let sender = tx.clone();
    let receiver = Arc::new(Mutex::new(rx));

    // Build the game map
    let path = env::var("MAP_FILEPATH").expect("MAP_FILEPATH must be set.");
    let file = File::open(path).expect("Failed to open map file!");
    let mut rooms = map::build(file).expect("Failed to build map from file");

    // Start the server and command input threads
    info!("Parsed map successfully");

    let _ = std::thread::spawn(move || {
        info!("Started server thread!");
        server(receiver, server_config, &mut rooms);
    });

    let _ = std::thread::spawn(move || {
        info!("Started input thread!");
        input(tx.clone());
    });

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                info!("New connection: {}", addr);

                let stream = Arc::new(stream);
                let sender = sender.clone();
                let client_config = client_config.clone();

                // Handle the connection in a separate thread
                let client_h = std::thread::spawn(move || {
                    connection(stream, sender, client_config);
                });

                debug!("Spawned client thread: {:?}", client_h.thread().id());
            }
            Err(e) => {
                warn!("Error accepting connection: {}", e);
            }
        }
    }
}
