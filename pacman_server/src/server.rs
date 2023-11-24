mod database;
mod game;
mod heartbeat;
mod listeners;

use std::{
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use database::Database;
use pacman_communication::{client_server};

fn current_time() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

pub fn run(port: u16) {
    let _database = Database::new();

    let _conn_table = game::ConnectionTable::new();

    // UDP and TCP listeners are abstracted into the same interface, where both of them send messages
    // received through this channel
    let recv = listeners::start(port);
    loop {
        let msg = match recv.recv() {
            Ok(msg) => msg,
            Err(err) => {
                log::error!("Error on recv: {err}");
                break;
            }
        };
        let client_server::Message {
            connection: _conn,
            message: msg,
        } = msg;

        todo!()
    }
}
