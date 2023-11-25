pub mod heartbeat;
mod states;

use std::{sync::{mpsc::Receiver, Arc, atomic::AtomicBool, Mutex}, time::Duration};

use pacman_communication::{server_client, Connection};

// Common info needed for all states
pub struct CommonInfo {
    pub server: Connection,
    pub connection: Connection,
    pub recv: Receiver<server_client::Message>,
    pub keep_running: Arc<AtomicBool>,
    pub last_heartbeat: Arc<Mutex<Duration>>
}

pub fn run(server: Connection, connection: Connection, recv: Receiver<server_client::Message>, keep_running: Arc<AtomicBool>) {
    if let Some(connected_client) = states::Connected::new(server, connection, recv, keep_running) {
        println!("Connected to server!");
        connected_client.run();
    } else {
        println!("Failed to connect to server!");
    }
}
