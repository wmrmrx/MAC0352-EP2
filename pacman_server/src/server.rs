mod database;
mod game;
mod heartbeat;
mod listeners;

use std::sync::{Arc, Mutex};

use database::Database;
use pacman_communication::{
    client_server,
    server_client::{
        self, ChangePasswordResponse, ConnectedUsersResponse, CreateGameResponse,
        CreateUserResponse, JoinGameResponse, LeaderboardResponse, LoginResponse,
    },
};

use crate::server::game::GameStatus;

pub fn run(port: u16) {
    let mut database = Database::new();

    let conn_table = Arc::new(Mutex::new(game::ConnectionTable::new()));

    heartbeat::setup(conn_table.clone());

    // UDP and TCP listeners are abstracted into the same interface, where both of them send messages
    // received through this channel
    let recv = listeners::start(port);
    loop {
        let msg = match recv.recv() {
            Ok(msg) => msg,
            Err(err) => {
                eprintln!("Error on recv: {err}");
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
            Disconnect => {
                let mut conn_table = conn_table.lock().unwrap();
                conn_table.remove(&conn);
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
                        conn.send(Message::LoginResponse(LoginResponse::Ok));
                        continue;
                    }
                }
                conn.send(Message::LoginResponse(LoginResponse::Err));
            }
            ChangePasswordRequest(req) => {
                let conn_table = conn_table.lock().unwrap();
                if let Some(conn_data) = conn_table.get_connections().get(&conn) {
                    if let Some(user) = conn_data.user.as_ref() {
                        if database.change_password(user, &req.old_passwd, &req.new_passwd) {
                            log::info!(
                                "User {} with connection {:?} changed password",
                                &user,
                                &conn
                            );
                            conn.send(Message::ChangePasswordResponse(ChangePasswordResponse::Ok));
                            continue;
                        }
                    }
                }
                conn.send(Message::ChangePasswordResponse(ChangePasswordResponse::Err));
            }
            LogoutRequest => {
                let mut conn_table = conn_table.lock().unwrap();
                if let Some(conn_data) = conn_table.get_connections().get(&conn).cloned() {
                    if let Some(user) = conn_data.user.as_ref().cloned() {
                        if conn_table.logout(&conn) {
                            log::info!("User {} with connection {:?} has logout", &user, &conn);
                            conn.send(Message::LogoutResponse);
                        }
                    }
                }
            }
            ConnectedUsersRequest => {
                let conn_table = conn_table.lock().unwrap();
                let mut users = Vec::new();
                for user in conn_table.get_users().keys() {
                    let conn = conn_table.get_users().get(user).unwrap();
                    let conn_data = conn_table.get_connections().get(conn).unwrap();
                    match conn_data.status {
                        GameStatus::Idle => {
                            users.push(format!("{user}: fazendo nada"));
                        }
                        GameStatus::Ghost => {
                            users.push(format!(
                                "{user}: em jogo com {}",
                                conn_table.get_ghosts().get(user).unwrap()
                            ));
                        }
                        GameStatus::Pacman(_) => {
                            if let Some(other_user) = conn_table.get_pacmans().get(user).unwrap() {
                                users.push(format!("{user}: em jogo com {other_user}"));
                            } else {
                                users.push(format!(
                                    "{user}: em jogo com ninguém (pode ser desafiado)"
                                ));
                            }
                        }
                    }
                }
                conn.send(Message::ConnectedUsersResponse(ConnectedUsersResponse {
                    users: users.into_boxed_slice(),
                }));
            }
            QuitGameRequest => {
                let mut conn_table = conn_table.lock().unwrap();
                let _ = conn_table.kick(&conn);
            }
            CreateGameRequest(req) => {
                let mut conn_table = conn_table.lock().unwrap();
                if conn_table.create_game(&conn, req.listener_addr) {
                    conn.send(Message::CreateGameResponse(CreateGameResponse::Ok));
                } else {
                    conn.send(Message::CreateGameResponse(CreateGameResponse::Err));
                }
            }
            JoinGameRequest(req) => {
                let mut conn_table = conn_table.lock().unwrap();
                if let Some(addr) = conn_table.join_game(&conn, &req.pacman) {
                    conn.send(Message::JoinGameResponse(JoinGameResponse::Ok(addr)));
                } else {
                    conn.send(Message::CreateGameResponse(CreateGameResponse::Err));
                }
            }
            LeaderboardRequest => {
                conn.send(Message::LeaderboardResponse(LeaderboardResponse {
                    top10: database.get_leaderboard(),
                }));
            }
            AddLeaderboardEntry(entry) => {
                database.add_leaderboard_entry(entry);
            }
        }
    }
}
