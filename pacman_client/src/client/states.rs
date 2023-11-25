pub mod connected;

pub use connected::Connected;

use std::{sync::{mpsc::Receiver, Arc, atomic::AtomicBool}, time::Duration};

use super::heartbeat;
use pacman_communication::{server_client, Connection};

const RECV_TIMEOUT: Duration = Duration::from_millis(33);

// Common info needed for all states
pub struct CommonInfo {
    pub server: Connection,
    pub connection: Connection,
    pub recv: Receiver<server_client::Message>,
    pub keep_running: Arc<AtomicBool>,
}

// #[must_use]
// struct Idle {
// }
// 
// #[must_use]
// struct Pacman;
// 
// #[must_use]
// struct Ghost;
// 
// trait ClientState {}
// 
// trait IsConnected: Sized {
//     fn disconnect(self, _: Info) {
//         println!("Disconnecting from server!");
//         drop(self);
//     }
// }
// impl IsConnected for Connected {}
// impl IsConnected for Pacman {}
// impl IsConnected for Idle {}
// impl IsConnected for Ghost {}
// 
// trait IsLoggedIn: Sized + IsConnected {
//     fn logout(self, _: Info) {
//     }
// }
// impl IsLoggedIn for Idle {}
// impl IsLoggedIn for Pacman {}
// impl IsLoggedIn for Ghost {}
