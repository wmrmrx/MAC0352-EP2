use std::net::SocketAddr;

use crate::{Connection, LeaderboardEntry};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub connection: Connection,
    pub message: MessageEnum,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageEnum {
    ConnectRequest,
    Disconnect,
    Heartbeat,
    CreateUserRequest(CreateUserRequest),
    LoginRequest(LoginRequest),
    ChangePasswordRequest(ChangePasswordRequest),
    LogoutRequest,
    QuitGameRequest,
    ConnectedUsersRequest,
    CreateGameRequest(CreateGameRequest),
    JoinGameRequest(JoinGameRequest),
    LeaderboardRequest,
    AddLeaderboardEntry(LeaderboardEntry),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserRequest {
    pub user: String,
    pub passwd: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginRequest {
    pub user: String,
    pub passwd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePasswordRequest {
    pub old_passwd: String,
    pub new_passwd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGameRequest {
    pub listener_addr: SocketAddr,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinGameRequest {
    pub pacman: String,
}
