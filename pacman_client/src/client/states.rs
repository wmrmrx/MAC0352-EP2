pub mod connected;
pub mod idle;

pub use connected::Connected;

pub use std::sync::atomic::Ordering;

pub use pacman_communication::{
    client_server::{CreateUserRequest, LoginRequest, Message, MessageEnum},
    server_client::{ConnectedUsersResponse, LeaderboardResponse, Message as ServerMessage},
};

pub use crate::client::{
    event::{watch, WatchErr},
    states::idle::Idle,
    CommonInfo,
};

pub use std::sync::{atomic::AtomicBool, mpsc::Receiver, Arc};

use super::heartbeat;
use super::shell::Shell;
use pacman_communication::{server_client, Connection};
