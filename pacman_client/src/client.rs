use std::sync::mpsc::Receiver;

use pacman_communication::{server_client, Connection};

/// Conn is the address from which we listen to new messages
pub fn run(server: Connection, listener: Connection, recv: Receiver<server_client::Message>) {
}
