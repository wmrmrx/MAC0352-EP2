use crate::Connection;
use serde::{Deserialize, Serialize};

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
    pub passwd: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub user: String,
    pub passwd: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_passwd: String,
    pub new_passwd: String,
}
