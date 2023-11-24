//! Specifies the binary format and types for communication
//! In this module are things relevant to both the client and server

pub mod client_server;
pub mod ghost_pacman;
pub mod pacman_ghost;
pub mod server_client;

use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket},
    time::Duration,
};

use serde::{Deserialize, Serialize};

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(100);
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_millis(1000);

/// Each connection has a listener
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Connection {
    Udp(SocketAddr),
    Tcp(SocketAddr), // Using TCP just like UDP for simplicity
}

impl Connection {
    pub fn send<T: PacmanMessage>(&self, msg: T) {
        match self {
            Connection::Udp(addr) => {
                let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                let _ = socket.send_to(&msg.to_bytes(), addr);
            }
            Connection::Tcp(addr) => {
                let mut stream = TcpStream::connect(addr).unwrap();
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

#[derive(Serialize, Deserialize)]
pub struct LeaderboardEntry {
    user: String,
    score: u64,
}

pub trait PacmanMessage: Sized {
    fn to_bytes(&self) -> Box<[u8]>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, ()>;
}

impl PacmanMessage for server_client::Message {
    fn to_bytes(&self) -> Box<[u8]> {
        serde_json::to_string(self)
            .unwrap()
            .into_bytes()
            .into_boxed_slice()
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let Ok(string) = std::str::from_utf8(bytes) else { return Err(()); };
        let Ok(res) = serde_json::from_str(string) else { return Err(()); };
        res
    }
}

impl PacmanMessage for client_server::Message {
    fn to_bytes(&self) -> Box<[u8]> {
        serde_json::to_string(self)
            .unwrap()
            .into_bytes()
            .into_boxed_slice()
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let Ok(string) = std::str::from_utf8(bytes) else { return Err(()); };
        let Ok(res) = serde_json::from_str(string) else { return Err(()); };
        res
    }
}

impl PacmanMessage for pacman_ghost::Message {
    fn to_bytes(&self) -> Box<[u8]> {
        serde_json::to_string(self)
            .unwrap()
            .into_bytes()
            .into_boxed_slice()
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let Ok(string) = std::str::from_utf8(bytes) else { return Err(()); };
        let Ok(res) = serde_json::from_str(string) else { return Err(()); };
        res
    }
}

impl PacmanMessage for ghost_pacman::Message {
    fn to_bytes(&self) -> Box<[u8]> {
        serde_json::to_string(self)
            .unwrap()
            .into_bytes()
            .into_boxed_slice()
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let Ok(string) = std::str::from_utf8(bytes) else { return Err(()); };
        let Ok(res) = serde_json::from_str(string) else { return Err(()); };
        res
    }
}
