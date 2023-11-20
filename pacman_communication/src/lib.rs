//! Specifies the binary format and types for communication
//! In this module are things relevant to both the client and server

use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddrV4, TcpStream, UdpSocket},
};

use serde::{Deserialize, Serialize};

// Bytes are valid ASCII characters
pub trait PacmanMessage<'a>: Serialize + Deserialize<'a> {
    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_string(self).unwrap().into_bytes()
    }
    fn from_bytes<'b: 'a>(bytes: &'b [u8]) -> Self {
        serde_json::from_str(std::str::from_utf8(bytes).unwrap()).unwrap()
    }
}

impl<'a, T> PacmanMessage<'a> for T where T: Serialize + Deserialize<'a> {}

/// Each connection has a listener
#[derive(Serialize, Deserialize, PartialEq)]
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
    listener: Connection
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    user: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub enum CreateUserResponse {
    Created,
    Rejected,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    user: String,
    password: String,

}

#[derive(Serialize, Deserialize)]
pub enum LoginResponse {
    LoggedIn,
    Failed,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
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
    id: Option<Connection>,
    message: ClientServerMessageEnum
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
    id: Connection,
    message: ClientP2PMessageEnum
}

#[derive(Serialize, Deserialize)]
pub enum ClientP2PMessageEnum {
    LatencyCheck,
    RespondLatencyCheck,
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    ServerClient(ServerClientMessage),
    ClientServer(ServerClientMessage),
    ClientP2P(ClientP2PMessage),
}
