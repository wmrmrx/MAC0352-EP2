mod database;
mod game;
mod heartbeat;
mod listeners;

use std::sync::{Mutex, Arc};

use database::Database;
use pacman_communication::{client_server, server_client::{self, CreateUserResponse}};

use std::time::{Duration, SystemTime, UNIX_EPOCH};
pub fn current_time() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

pub fn run(port: u16) {
    let mut database = Database::new();

    let conn_table = Arc::new(Mutex::new(game::ConnectionTable::new()));

    heartbeat::watch(conn_table.clone());

    // UDP and TCP listeners are abstracted into the same interface, where both of them send messages
    // received through this channel
    let recv = listeners::start(port);
    loop {
        let msg = match recv.recv() {
            Ok(msg) => msg,
            Err(err) => {
                log::error!("Error on recv: {err}");
                break;
            }
        };
        let client_server::Message {
            connection: conn,
            message: msg,
        } = msg;

        use client_server::MessageEnum::*;
        use server_client::Message;
        match msg {
            ConnectRequest => {
                let mut conn_table = conn_table.lock().unwrap();
                if conn_table.insert(&conn) {
                    conn.send(Message::ConnectResponse);
                }
            }
            Heartbeat => {
                let mut conn_table = conn_table.lock().unwrap();
                conn_table.set_heartbeat(&conn);
            }
            CreateUserRequest(req) => {
                if database.create_user(&req.user, &req.passwd) {
                    log::info!("Created user {}", &req.user);
                    conn.send(Message::CreateUserResponse(CreateUserResponse::Ok));
                } else {
                    conn.send(Message::CreateUserResponse(CreateUserResponse::Err));
                }
            }
            LoginRequest(req) => {
                if database.login(&req.user, &req.passwd) {
                    let mut conn_table = conn_table.lock().unwrap();
                    if conn_table.login(&conn, &req.user) {
                        log::info!("Created user {} with connection {:?}", &req.user, &conn);
                    }

                } else {
                }
            }
            ChangePasswordRequest(req) => {}
            LogoutRequest => {}
            ConnectedUsersRequest => {}
            CreatePartyRequest => {}
            JoinPartyRequest => {}
            LeaderboardRequest => {}
        }
    }
}
