pub mod heartbeat;
pub mod shell;
mod states;

use std::sync::{mpsc::Receiver, Arc, atomic::AtomicBool};

use pacman_communication::{server_client, Connection};

pub fn run(server: Connection, connection: Connection, recv: Receiver<server_client::Message>, keep_running: Arc<AtomicBool>) {
    if let Some(connected_client) = states::Connected::new(server, connection, recv, keep_running) {
        println!("Connected to server!");
        connected_client.run();
    } else {
        println!("Failed to connect to server!");
    }
}
