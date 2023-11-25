use std::{sync::{mpsc::RecvTimeoutError, atomic::Ordering}, time::Duration};

use pacman_communication::{
    client_server::{LoginRequest, Message, MessageEnum, CreateUserRequest},
    current_time,
    server_client::{Message as ServerMessage, LeaderboardResponse, ConnectedUsersResponse},
};

use crate::client::{
    event::{watch, WatchErr},
    states::idle::Idle,
    CommonInfo,
};

use super::*;

pub struct Connected {
    info: CommonInfo,
}

impl Connected {
    pub fn new(
        server: Connection,
        connection: Connection,
        mut recv: Receiver<ServerMessage>,
        keep_running: Arc<AtomicBool>,
    ) -> Option<Self> {
        server.send(Message {
            connection: connection.clone(),
            message: MessageEnum::ConnectRequest,
        });
        match watch(&recv, |msg| -> bool {
            matches!(msg, ServerMessage::ConnectResponse)
        }) {
            Ok(_) => {
                recv = heartbeat::setup(
                    server.clone(),
                    connection.clone(),
                    recv,
                    keep_running.clone(),
                );
                Some(Self {
                    info: CommonInfo {
                        server,
                        connection,
                        recv,
                        keep_running,
                    },
                })
            }
            Err(_) => None,
        }
    }

    pub fn run(self) {
        println!(">>> CONECTADO AO SERVIDOR COM SUCESSO!");
        let commands = ["novo", "entra", "lideres", "l", "tchau"];
        let shell = Shell::new(&commands);
        loop {
            let command = shell.prompt();
            match command[0].as_str() {
                "novo" => {
                    let (user, passwd) = (&command[1], &command[2]);
                    self.info.server.send(Message {
                        connection: self.info.connection.clone(),
                        message: MessageEnum::CreateUserRequest(CreateUserRequest{
                            user: user.to_owned(),
                            passwd: passwd.to_owned(),
                        }),
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::CreateUserResponse(_))
                    }) {
                        Ok(msg) => {
                            if let ServerMessage::CreateUserResponse(server_client::CreateUserResponse::Err) = msg {
                                println!("Erro ao criar usuário (talvez ele já exista");
                            }
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                            continue;
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                }
                "entra" => {
                    let (user, passwd) = (&command[1], &command[2]);

                    self.info.server.send(Message {
                        connection: self.info.connection.clone(),
                        message: MessageEnum::LoginRequest(LoginRequest {
                            user: user.to_owned(),
                            passwd: passwd.to_owned(),
                        }),
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::LoginResponse(_))
                    }) {
                        Ok(msg) => {
                            if let ServerMessage::LoginResponse(server_client::LoginResponse::Err) =
                                msg
                            {
                                println!("Login não aceito, talvez a senha ou o usuário podem estar errados");
                                continue;
                            }
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                            continue;
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                    let idle_client = Idle::new(self.info);
                    return idle_client.run();
                }
                "lideres" => {
                    self.info.server.send(Message {
                        connection: self.info.connection.clone(),
                        message: MessageEnum::LeaderboardRequest
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::LeaderboardResponse(_))
                    }) {
                        Ok(msg) => {
                            let ServerMessage::LeaderboardResponse(LeaderboardResponse{top10}) = msg else { unreachable!() };
                            println!("Lideres: {top10:?}");
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                            continue;
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                }
                "l" => {
                    self.info.server.send(Message {
                        connection: self.info.connection.clone(),
                        message: MessageEnum::ConnectedUsersRequest
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::ConnectedUsersResponse(_))
                    }) {
                        Ok(msg) => {
                            let ServerMessage::ConnectedUsersResponse(ConnectedUsersResponse { users }) = msg else { unreachable!() };
                            println!("Usuários online:");
                            for user in users.into_iter() {
                                println!("- {user}");
                            }
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                            continue;
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                }
                "tchau" => {
                    self.info.server.send(Message {
                        connection: self.info.connection.clone(),
                        message: MessageEnum::Disconnect
                    });
                    self.info.keep_running.store(false, Ordering::Relaxed);
                    return;
                }
                _ => unreachable!()
            }
        }
    }
}
