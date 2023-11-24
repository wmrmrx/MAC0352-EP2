use serde::{Deserialize, Serialize};
use crate::Connection;

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub connection: Connection,
    pub message: MessageEnum,
}

#[derive(Serialize, Deserialize)]
pub enum MessageEnum {
    ConnectRequest,
    Heartbeat,
    CreateUserRequest(CreateUserRequest),
    LoginRequest(LoginRequest),
    ChangePasswordRequest(ChangePasswordRequest),
    LogoutRequest,
    ConnectedUsersRequest,
    CreatePartyRequest,
    JoinPartyRequest,
    LeaderboardRequest,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub user: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub user: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}
