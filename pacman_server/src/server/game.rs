use pacman_communication::Connection;
use std::{collections::BTreeMap, net::SocketAddr};

#[derive(Clone, PartialEq)]
pub enum GameStatus {
    Pacman(SocketAddr), // Pacman must have a TCPListener in this address
    Ghost,
    Idle,
}

#[derive(Clone)]
pub struct ConnectionData {
    pub user: Option<String>,
    pub status: GameStatus,
}

pub struct ConnectionTable {
    connections: BTreeMap<Connection, ConnectionData>,
    users: BTreeMap<String, Connection>,
    party: Vec<Connection>,
}

impl ConnectionTable {
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: BTreeMap::new(),
            users: BTreeMap::new(),
            party: Vec::new(),
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
                        user = conn_data.user.clone().unwrap_or("err".to_owned())
                    );
                }
                Idle => {}
            }
        }
        self.party.clear();
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
                    user = conn_data.user.clone().unwrap_or("err".to_owned())
                );
                for i in 0..self.party.len() {
                    if self.party[i] == *conn {
                        self.party.remove(i);
                        break;
                    }
                }
                conn_data.status = GameStatus::Idle;
                true
            }
            Idle => false,
        }
    }

    pub fn logout(&mut self, conn: &Connection) -> bool {
        self.kick(conn);
        let Some(conn_data) = self.connections.get_mut(conn) else { return false; };
        if let Some(user) = conn_data.user.as_ref() {
            log::info!("User {user} with connection {conn:?} logging out");
            self.users.remove(user);
            conn_data.user = None;
            true
        } else {
            false
        }
    }

    // Returns true if the connection was removed
    pub fn remove(&mut self, conn: &Connection) -> bool {
        self.logout(conn);
        let res = self.connections.remove(conn).is_some();
        if res {
            log::info!("Connection {conn:?} disconnected");
        }
        res
    }

    // Returns true if the connection was inserted, false if it already existed
    pub fn insert(&mut self, conn: &Connection) -> bool {
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

    #[must_use]
    pub fn get(&mut self, connection: &Connection) -> Option<&ConnectionData> {
        self.connections.get(connection)
    }
}
