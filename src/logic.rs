use lurk_lcsc::Protocol;

use crate::logic::commands::Action;

pub mod commands;
pub mod config;
pub mod map;

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
                ::tracing::error!("[CONNECT] Failed to send {} packet", $protocol_variant);
            });
    };
}

#[macro_export]
macro_rules! send_ext_cmd {
    ($sender:expr, $command:expr) => {
        $sender
            .send(ExtendedProtocol::Command($command))
            .unwrap_or_else(|_| {
                ::tracing::error!("[CONNECT] Failed to send {} packet", $command);
            });
    };
}
