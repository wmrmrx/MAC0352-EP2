//! Specifies the binary format and types for communication
//! In this module are things relevant to both the client and server

use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddrV4, TcpStream, UdpSocket}, time::Duration,
};

use serde::{Deserialize, Serialize};

pub const HEARTBEAT_INTERVAL: Duration = Duration::from_millis(100);
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_millis(300);

/// Each connection has a listener
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Connection {
    Udp(SocketAddrV4),
    Tcp(SocketAddrV4), // Using TCP like UDP for simplicity (open a new connection per stream)
}

impl Connection {
    pub fn send(&self, bytes: &[u8]) {
        match self {
            Connection::Udp(addr) => {
                let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                socket.send_to(bytes, addr).unwrap();
            }
            Connection::Tcp(addr) => {
                let mut stream = TcpStream::connect(addr).unwrap();
                let _ = stream.write(bytes).unwrap();
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
pub struct ConnectRequest {
    pub listener: Connection
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub user: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub enum CreateUserResponse {
    Created,
    Rejected,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub user: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub enum LoginResponse {
    LoggedIn,
    Failed,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
pub enum ChangePasswordResponse {
    Success,
    NogLoggedIn,
    Fail,
}

#[derive(Serialize, Deserialize)]
pub struct ConnectedUsersResponse {
    user: String,
    old_password: String,
    new_password: String,
}

#[derive(Serialize, Deserialize)]
pub enum CreatePartyResponse {
    Success,
    AlreadyExists,
    NotLoggedIn
}

#[derive(Serialize, Deserialize)]
pub enum JoinPartyResponse {
    Successg,
    AlreadyExists,
    NotLoggedIn
}

#[derive(Serialize, Deserialize)]
pub enum ServerClientMessage {
    ConnectResponse,
    Heartbeat,
    CreateUserResponse(CreateUserResponse),
    LoginResponse(LoginResponse),
    ChangePasswordResponse(ChangePasswordResponse),
    LogoutResponse,
    ConnectedUsersResponse(ConnectedUsersResponse),
    CreatePartyResponse(CreatePartyResponse),
    JoinPartyResponse(JoinPartyResponse)
}

#[derive(Serialize, Deserialize)]
pub struct ClientServerMessage {
    pub id: Option<Connection>,
    pub message: ClientServerMessageEnum
}

#[derive(Serialize, Deserialize)]
pub enum ClientServerMessageEnum {
    ConnectRequest(ConnectRequest),
    Heartbeat,
    CreateUserRequest(CreateUserRequest),
    LoginRequest(LoginRequest),        
    ChangePasswordRequest(ChangePasswordRequest),
    LogoutRequest,
    ConnectedUsersRequest,
    CreatePartyRequest,
    JoinPartyRequest
}

/// Generic P2P client messages
#[derive(Serialize, Deserialize)]
pub struct ClientP2PMessage {
    pub id: Connection,
    pub message: ClientP2PMessageEnum
}

#[derive(Serialize, Deserialize)]
pub enum ClientP2PMessageEnum {
    LatencyCheck,
    RespondLatencyCheck,
}

#[derive(Serialize, Deserialize)]
pub enum PacmanMessage {
    ServerClient(ServerClientMessage),
    ClientServer(ClientServerMessage),
    ClientP2P(ClientP2PMessage),
}

impl PacmanMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_string(self).unwrap().into_bytes()
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_str(std::str::from_utf8(bytes).unwrap()).unwrap()
    }
}
