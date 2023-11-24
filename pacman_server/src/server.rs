mod database;
mod heartbeat;
mod listeners;
mod game;

use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, UdpSocket},
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use database::Database;
use pacman_communication::{client_server, server_client, Connection, PacmanMessage};

fn current_time() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

pub fn run(port: u16) {
    let database = Database::new();

    let conn_table = game::ConnectionTable::new();

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
            connection: conn,
            message: msg,
        } = msg;

        match msg {
            _ => todo!(),
        }
    }
}
