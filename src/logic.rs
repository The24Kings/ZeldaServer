use lurk_lcsc::Protocol;

use crate::logic::commands::Action;

pub mod commands;
pub mod config;
pub mod map;

pub enum ExtendedProtocol {
    Base(Protocol),
    Command(Action),
}
