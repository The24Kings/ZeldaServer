use std::env;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Config {
    pub cmd_prefix: String,
    pub map_path: String,
    pub description_path: String,
    pub stat_limit: u16,
    pub initial_points: u16,
    pub major_rev: u8,
    pub minor_rev: u8,
    pub help_cmd: String,
}

impl Config {
    pub fn load() -> Self {
        info!("[CONFIG] Loading configuration...");

        let cmd_prefix = env::var("CMD_PREFIX").unwrap_or_else(|_| "!".into());
        let map_path = env::var("MAP_FILEPATH").expect("[CONFIG] MAP_FILEPATH must be set.");
        let description_path =
            env::var("DESC_FILEPATH").expect("[CONFIG] DESC_FILEPATH must be set.");
        let stat_limit = env::var("STAT_LIMIT")
            .expect("[CONFIG] STAT_LIMIT must be set.")
            .parse()
            .expect("[CONFIG] Failed to parse STAT_LIMIT");
        let initial_points = env::var("INITIAL_POINTS")
            .expect("[CONFIG] INITIAL_POINTS must be set.")
            .parse()
            .expect("[CONFIG] Failed to parse INITIAL_POINTS");
        let major_rev = env::var("MAJOR_REV")
            .expect("[CONFIG] MAJOR_REV must be set.")
            .parse()
            .expect("[CONFIG] Failed to parse MAJOR_REV");
        let minor_rev = env::var("MINOR_REV")
            .expect("[CONFIG] MINOR_REV must be set.")
            .parse()
            .expect("[CONFIG] Failed to parse MINOR_REV");
        let help_cmd = "
Usage:
    ${CMD_PREFIX}help                           - Display this help message
    ${CMD_PREFIX}broadcast <content>            - Send a message to all players
    ${CMD_PREFIX}message <recipient> <content>  - Send a private message to a player
    ${CMD_PREFIX}nuke                           - Remove all disconnected players on the map"
            .replace("${CMD_PREFIX}", &cmd_prefix);

        info!("[CONFIG] Successfully loaded configuration!");

        Config {
            cmd_prefix,
            map_path,
            description_path,
            stat_limit,
            initial_points,
            major_rev,
            minor_rev,
            help_cmd,
        }
    }
}
