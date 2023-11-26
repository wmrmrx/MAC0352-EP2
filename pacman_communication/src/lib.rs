//! Specifies the binary format and types for communication
//! In this module are things relevant to both the client and server

pub mod client_server;
pub mod game;
pub mod server_client;

use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(20);

/// Each connection has a listener
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Copy)]
pub enum Connection {
    Udp(SocketAddr),
    Tcp(SocketAddr), // Using TCP just like UDP for simplicity
}

pub fn current_time() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

impl Connection {
    pub fn send<T: PacmanMessage>(&self, msg: T) {
        match self {
            Connection::Udp(addr) => {
                let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                let _ = socket.send_to(&msg.to_bytes(), addr);
            }
            Connection::Tcp(addr) => {
                let Ok(mut stream) = TcpStream::connect(addr) else {return; };
                let _ = stream.write(&msg.to_bytes());
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Direction {
    North,
    West,
    East,
    South,
}

impl Direction {
    pub fn as_vector(&self) -> [isize; 2] {
        use Direction::*;
        match self {
            North => [-1, 0],
            West => [0, -1],
            East => [0, 1],
            South => [1, 0],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct LeaderboardEntry {
    pub score: u64,
    pub user: String,
}

pub trait PacmanMessage: Sized + std::fmt::Debug {
    fn to_bytes(&self) -> Box<[u8]>;
    fn from_bytes(bytes: &[u8]) -> Option<Self>;
}

impl PacmanMessage for server_client::Message {
    fn to_bytes(&self) -> Box<[u8]> {
        serde_json::to_string(self)
            .unwrap()
            .into_bytes()
            .into_boxed_slice()
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Ok(string) = std::str::from_utf8(bytes) else { return None; };
        let Ok(res) = serde_json::from_str::<Self>(string) else { return None; };
        Some(res)
    }
}

impl PacmanMessage for client_server::Message {
    fn to_bytes(&self) -> Box<[u8]> {
        serde_json::to_string(self)
            .unwrap()
            .into_bytes()
            .into_boxed_slice()
    }
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let Ok(string) = std::str::from_utf8(bytes) else { return None; };
        let Ok(res) = serde_json::from_str::<Self>(string) else { return None; };
        Some(res)
    }
}
