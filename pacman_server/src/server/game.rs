use pacman_communication::{Connection, current_time};
use std::{collections::BTreeMap, net::SocketAddr, time::Duration};

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
    pub last_heartbeat: Duration,
}

pub struct ConnectionTable {
    connections: BTreeMap<Connection, ConnectionData>,
    users: BTreeMap<String, Connection>,
    pacmans: BTreeMap<String, Option<String>>, // Map : PacmanUsername -> Option<GhostUsername>
    // Every game must have a pacman, but not
    // necessarily a ghost
    ghosts: BTreeMap<String, String>, // Map : GhostUsername -> PacmanUsername
}

impl ConnectionTable {
    #[must_use]
    pub fn new() -> Self {
        Self {
            connections: BTreeMap::new(),
            users: BTreeMap::new(),
            pacmans: BTreeMap::new(),
            ghosts: BTreeMap::new(),
        }
    }

    pub fn get_connections(&self) -> &BTreeMap<Connection, ConnectionData> {
        &self.connections
    }

    pub fn get_users(&self) -> &BTreeMap<String, Connection> {
        &self.users
    }


    /// Kick connection from game
    /// Returns true if kicked from a game
    pub fn kick(&mut self, conn: &Connection) -> bool {
        let Some(conn_data) = self.connections.get_mut(conn) else { return false; };
        let Some(user) = conn_data.user.as_ref() else { return false; };
        use GameStatus::*;
        match conn_data.status {
            Pacman(_) => {
                log::info!("Kicking pacman (connection: {conn:?}, user: {user}). Also kicking ghost from the game if it exists.");
                conn_data.status = Idle;
                if let Some(ghost) = self.pacmans.remove(user).unwrap() {
                    let ghost_conn = self.users.get(&ghost).unwrap();
                    log::info!(
                        "Kicking ghost (connection {ghost_conn:?}, user: {ghost}) from the game."
                    );
                    self.connections.get_mut(ghost_conn).unwrap().status = Idle;
                    self.ghosts.remove(&ghost).unwrap();
                }
                true
            }
            Ghost => {
                log::info!("Kicking ghost (connection {conn:?}, user: {user}) from the game.");
                conn_data.status = Idle;
                let pacman = self.ghosts.remove(user).unwrap();
                *self.pacmans.get_mut(&pacman).unwrap() = None;
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

    pub fn set_heartbeat(&mut self, conn: &Connection) {
        if let Some(conn_data) = self.connections.get_mut(conn) {
            conn_data.last_heartbeat = current_time();
        }
    }

    pub fn login(&mut self, conn: &Connection, user: &str) -> bool {
        if let Some(conn_data) = self.connections.get_mut(conn) {
            if self.users.get(user).is_none() {
                log::info!("Connection {conn:?} logged in as {user}");
                conn_data.user = Some(user.to_owned());
                self.users.insert(user.to_owned(), conn.clone());
                return true;
            }
        }
        false
    }

    // Returns true if the connection was inserted, false if it already existed
    pub fn insert(&mut self, conn: &Connection) -> bool {
        if self.connections.get(conn).is_some() {
            false
        } else {
            log::info!("Connection added: {conn:?}");
            self.connections.insert(
                conn.clone(),
                ConnectionData {
                    user: None,
                    status: GameStatus::Idle,
                    last_heartbeat: current_time(),
                },
            );
            true
        }
    }

    pub fn create_game(&mut self, conn: &Connection, listener_addr: SocketAddr) -> bool {
        let Some(conn_data) = self.connections.get_mut(conn) else { return false; };
        let Some(user) = conn_data.user.as_mut() else { return false; };
        if conn_data.status != GameStatus::Idle {
            false
        } else {
            log::info!("User {user} with connection {conn:?} created a game on {listener_addr:?}");
            conn_data.status = GameStatus::Pacman(listener_addr);
            self.pacmans.insert(user.to_owned(), None);
            true
        }
    }

    /// Returns the `listener_addr` of pacman if joining the game was sucessful
    pub fn join_game(&mut self, conn: &Connection, pacman: &str) -> Option<SocketAddr> {
        let Some(conn_data) = self.connections.get(conn) else { return None; };
        let Some(user) = conn_data.user.as_ref() else { return None; };
        let Some(pacman_conn) = self.users.get_mut(pacman) else { return None; };
        let Some(pacman_conn_data) = self.connections.get(pacman_conn) else { return None; };
        let GameStatus::Pacman(addr) = pacman_conn_data.status else { return None; };
        let Some(other_player) = self.pacmans.get_mut(pacman) else { return None; };
        if other_player.is_some() { return None; }
        *other_player = Some(user.to_owned());
        self.ghosts.insert(user.to_owned(), pacman.to_owned());
        log::info!("Ghost (user: {user}, connection: {conn:?}) joined game created by user {pacman} with connection {pacman_conn:?}");
        Some(addr)
    }
}
