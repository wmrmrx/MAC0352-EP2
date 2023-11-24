//! Specifies the binary format and types for communication
//! In this module are things relevant to both the client and server

pub mod server_client;
pub mod client_server;
pub mod client_client;

use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket},
    time::Duration,
};

use serde::{Deserialize, Serialize};

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(33);
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_millis(300);

/// Each connection has a listener
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Connection {
    Udp(SocketAddr),
    Tcp(SocketAddr),
}

impl Connection {
    pub fn send<T: Into<PacmanMessage>>(&self, msg: T) {
        let msg: PacmanMessage = msg.into();
        match self {
            Connection::Udp(addr) => {
                let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                socket.send_to(&msg.to_bytes(), addr).unwrap();
            }
            Connection::Tcp(addr) => {
                let mut stream = TcpStream::connect(addr).unwrap();
                let _ = stream.write(&msg.to_bytes()).unwrap();
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
pub struct Peer {
    user: String,
    connection: crate::Connection
}

#[derive(Serialize, Deserialize)]
pub struct LeaderboardEntry {
    user: String,
    score: u64
}

#[derive(Serialize, Deserialize)]
pub enum PacmanMessage {
    ServerClient(server_client::Message),
    ClientServer(client_server::Message),
    ClientClient(client_client::Message),
    Error
}

impl PacmanMessage {
    pub fn to_bytes(&self) -> Box<[u8]> {
        serde_json::to_string(self).unwrap().into_bytes().into_boxed_slice()
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let Ok(string) = std::str::from_utf8(bytes) else { return Self::Error; };
        let Ok(res) = serde_json::from_str(string) else { return Self::Error; };
        res
    }
}

impl From<server_client::Message> for PacmanMessage {
    fn from(value: server_client::Message) -> PacmanMessage {
        PacmanMessage::ServerClient(value)
    }
}

impl From<client_server::Message> for PacmanMessage {
    fn from(value: client_server::Message) -> PacmanMessage {
        PacmanMessage::ClientServer(value)
    }
}

impl From<client_client::Message> for PacmanMessage {
    fn from(value: client_client::Message) -> PacmanMessage {
        PacmanMessage::ClientClient(value)
    }
}
