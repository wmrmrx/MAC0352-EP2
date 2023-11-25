use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub enum CreateUserResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LoginResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChangePasswordResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectedUsersResponse {
    pub users: Box<[String]>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CreateGameResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JoinGameResponse {
    Ok(SocketAddr),
    Err,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaderboardResponse {
    pub top10: Box<[crate::LeaderboardEntry]>,
}
