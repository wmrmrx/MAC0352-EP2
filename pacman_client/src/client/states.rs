pub mod connected;
pub mod idle;

pub use connected::Connected;

use std::
    sync::{atomic::AtomicBool, mpsc::Receiver, Arc}
;

use super::heartbeat;
use super::shell::Shell;
use pacman_communication::{server_client, Connection};
