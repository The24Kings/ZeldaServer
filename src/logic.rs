use lurk_lcsc::Protocol;

use crate::logic::commands::Action;

pub mod commands;
pub mod config;
pub mod handlers;
pub mod map;
pub mod state;

pub enum ExtendedProtocol {
    Base(Protocol),
    Command(Action),
}

#[macro_export]
macro_rules! send_ext_base {
    ($sender:expr, $protocol_variant:expr) => {
        $sender
            .send(ExtendedProtocol::Base($protocol_variant))
            .unwrap_or_else(|_| {
                ::tracing::error!("Failed to send {} packet", $protocol_variant);
            });
    };
}

#[macro_export]
macro_rules! send_ext_cmd {
    ($sender:expr, $command:expr) => {{
        let cmd = $command;
        let cmd_str = cmd.to_string();
        $sender
            .send(ExtendedProtocol::Command(cmd))
            .unwrap_or_else(|_| {
                ::tracing::error!("Failed to send {} packet", cmd_str);
            });
    }};
}
