use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Message {
    ConnectResponse,
    CreateUserResponse(CreateUserResponse),
    LoginResponse(LoginResponse),
    ChangePasswordResponse(ChangePasswordResponse),
    LogoutResponse,
    CreatePartyResponse(CreatePartyResponse),
    JoinPartyResponse(JoinPartyResponse),
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
    pub pacman: (String, SocketAddr),
    pub ghosts: String,
}

#[derive(Serialize, Deserialize)]
pub enum CreatePartyResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize)]
pub enum JoinPartyResponse {
    Ok,
    Err,
}

#[derive(Serialize, Deserialize)]
pub struct LeaderboardResponse {
    top10: Box<[crate::LeaderboardEntry]>,
}
