mod database;
mod listeners;

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

#[derive(Clone, PartialEq)]
enum GameStatus {
    Pacman(SocketAddr),
    Ghost,
    Idle,
}

#[derive(Clone)]
struct ConnectionData {
    user: Option<String>,
    status: GameStatus,
}

struct ConnectionTable {
    connections: BTreeMap<Connection, ConnectionData>,
    users: BTreeMap<String, Connection>,
}

impl ConnectionTable {
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: BTreeMap::new(),
            users: BTreeMap::new(),
        }
    }

    fn kick_all(&mut self) {
        for (conn, conn_data) in self.connections.iter_mut() {
            use GameStatus::*;
            match conn_data.status {
                Pacman(_) | Ghost => {
                    conn_data.status = GameStatus::Idle;
                    log::info!(
                        "Kicking connection {conn:?} with user {user} from the game.",
                        user = conn_data.user.unwrap_or("err".to_owned())
                    );
                }
                Idle => {}
            }
        }
    }

    /// Kick connection from game
    /// Returns true if kicked from a game
    pub fn kick(&mut self, conn: &Connection) -> bool {
        let Some(conn_data) = self.connections.get_mut(conn) else { return false; };
        use GameStatus::*;
        match conn_data.status {
            Pacman(_) => {
                log::info!("Pacman (host) is being kicked. Kicking everyone from the game.");
                self.kick_all();
                true
            }
            Ghost => {
                log::info!(
                    "Kicking connection {conn:?} with user {user} from the game.",
                    user = conn_data.user.unwrap_or("err".to_owned())
                );
                conn_data.status = GameStatus::Idle;
                true
            }
            Idle => false,
        }
    }

    fn logout(&mut self, conn: &Connection) -> bool {
        self.kick(conn);
        let Some(conn_data) = self.connections.get_mut(conn) else { return false; };
        if let Some(user) = conn_data.user {
            log::info!("User {user} with connection {conn:?} logging out");
            self.users.remove(&user);
            conn_data.user = None;
            true
        } else {
            false
        }
    }

    // Returns true if the connection was removed
    fn remove(&mut self, conn: &Connection) -> bool {
        self.logout(conn);
        let res = self.connections.remove(conn).is_some();
        if res {
            log::info!("Connection {conn:?} disconnected");
        }
        res
    }

    #[must_use]
    pub fn get(&mut self, connection: &Connection) -> Option<&ConnectionData> {
        self.connections.get(connection)
    }

    // Returns true if the connection was inserted, false if it already existed
    fn insert(&self, conn: &Connection) -> bool {
        if let Some(_) = self.connections.get(conn) {
            false
        } else {
            log::info!("Connection added: {conn:?}");
            self.connections.insert(
                conn.clone(),
                ConnectionData {
                    user: None,
                    status: GameStatus::Idle,
                },
            );
            true
        }
    }
}

pub fn run(port: u16) {
    let database = Database::new();

    let conn_table = ConnectionTable::new();

    // UDP and TCP listeners are abstracted into the same interface, where both of them send messages
    // received through this channel
    let (send, recv) = channel();

    listeners::start(port, send.clone());
    loop {
        let msg = match recv.recv() {
            Ok(msg) => msg,
            Err(err) => {
                log::error!("Error on recv: {err}");
                break;
            }
        };
        let (conn, msg) = if let PacmanMessage::ClientServer(client_server::Message {
            connection: conn,
            message: msg,
        }) = msg
        {
            (conn, msg)
        } else {
            eprintln!("Server received message not meant for it!");
            continue;
        };

        match msg {
            _ => todo!(),
        }
    }
}
