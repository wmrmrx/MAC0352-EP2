pub mod heartbeat;
mod states;

use std::{sync::{mpsc::Receiver, Arc, atomic::AtomicBool}};

use pacman_communication::{server_client, Connection};

// Common info needed for all states
pub struct Info {
    pub server: Connection,
    pub connection: Connection,
    pub recv: Receiver<server_client::Message>,
    pub keep_running: Arc<AtomicBool>,
}

pub fn run(server: Connection, connection: Connection, recv: Receiver<server_client::Message>, keep_running: Arc<AtomicBool>) {
    // Start the state machine
    let unconnected_client = states::Unconnected;
    unconnected_client.try_connect(Info{
        server,
        connection,
        recv,
        keep_running,
    }   );
}
