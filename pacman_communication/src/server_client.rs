use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    Heartbeat,
    ConnectResponse,
    CreateUserResponse(CreateUserResponse),
    LoginResponse(LoginResponse),
    ChangePasswordResponse(ChangePasswordResponse),
    LogoutResponse,
    CreateGameResponse(CreateGameResponse),
    JoinGameResponse(JoinGameResponse),
    ConnectedUsersResponse(ConnectedUsersResponse),
    LeaderboardResponse(LeaderboardResponse),
    NotConnected,
}

#[derive(Serialize, Deserialize)]
pub enum CreateUserResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize)]
pub enum LoginResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize)]
pub enum ChangePasswordResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize)]
pub struct ConnectedUsersResponse {
    pub users: Box<[String]>,
}

#[derive(Serialize, Deserialize)]
pub enum CreateGameResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize)]
pub enum JoinGameResponse {
    Ok(SocketAddr),
    Err,
}

#[derive(Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub top10: Box<[crate::LeaderboardEntry]>,
}
