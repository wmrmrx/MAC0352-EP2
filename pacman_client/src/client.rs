pub mod event;
pub mod heartbeat;
pub mod shell;
pub mod states;

use std::sync::{atomic::AtomicBool, mpsc::Receiver, Arc};

use pacman_communication::{server_client, Connection};

// Common info needed for all states
pub struct CommonInfo {
    pub server: Connection,
    pub connection: Connection,
    pub recv: Receiver<server_client::Message>,
    pub keep_running: Arc<AtomicBool>,
}

pub fn run(
    server: Connection,
    connection: Connection,
    recv: Receiver<server_client::Message>,
    keep_running: Arc<AtomicBool>,
) {
    if let Some(connected_client) = states::Connected::new(CommonInfo { server, connection, recv, keep_running }) {
        println!("Connected to server!");
        connected_client.run();
    } else {
        println!("Failed to connect to server!");
    }
}
