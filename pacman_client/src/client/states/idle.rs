use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

use pacman_communication::{
    client_server::{CreateGameRequest, JoinGameRequest},
    server_client::{CreateGameResponse, JoinGameResponse},
};

use crate::client::states::{ghost::Ghost, pacman::Pacman};

use super::{
    watch, CommonInfo, Connected, ConnectedUsersResponse, LeaderboardResponse, Message,
    MessageEnum, Ordering, ServerMessage, Shell, WatchErr,
};

pub struct Idle {
    info: CommonInfo,
    user: String,
}

impl Idle {
    #[must_use]
    pub fn new(info: CommonInfo, user: String) -> Self {
        Self { info, user }
    }

    pub fn run(self) {
        let commands = ["lideres", "l", "inicia", "desafio", "sai", "tchau"];

        let shell = Shell::new(&commands);
        loop {
            let command = shell.prompt(&format!("{} - IDLE", &self.user));
            match command[0].as_str() {
                "lideres" => {
                    self.info.server.send(Message {
                        connection: self.info.connection,
                        message: MessageEnum::LeaderboardRequest,
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
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                }
                "l" => {
                    self.info.server.send(Message {
                        connection: self.info.connection,
                        message: MessageEnum::ConnectedUsersRequest,
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::ConnectedUsersResponse(_))
                    }) {
                        Ok(msg) => {
                            let ServerMessage::ConnectedUsersResponse(ConnectedUsersResponse { users }) = msg else { unreachable!() };
                            println!("Usuários online:");
                            for user in users.iter() {
                                println!("- {user}");
                            }
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                }
                "tchau" => {
                    self.info.server.send(Message {
                        connection: self.info.connection,
                        message: MessageEnum::Disconnect,
                    });
                    self.info.keep_running.store(false, Ordering::Relaxed);
                    return;
                }
                "desafio" => {
                    let pacman = command[1].as_str();
                    self.info.server.send(Message {
                        connection: self.info.connection,
                        message: MessageEnum::JoinGameRequest(JoinGameRequest {
                            pacman: pacman.to_owned(),
                        }),
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::JoinGameResponse(_))
                    }) {
                        Ok(msg) => {
                            let ServerMessage::JoinGameResponse(response) = msg else { unreachable!() };
                            if let JoinGameResponse::Ok(pacman_addr) = response {
                                println!("Servidor aceitou o desafio!");
                                return Ghost::new_and_run(
                                    self.info,
                                    self.user,
                                    pacman_addr,
                                    pacman.to_owned(),
                                );
                            } else {
                                println!("Servidor rejeitou o desafio!");
                            }
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                }
                "sai" => {
                    self.info.server.send(Message {
                        connection: self.info.connection,
                        message: MessageEnum::LogoutRequest,
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::LogoutResponse)
                    }) {
                        Ok(_msg) => {
                            println!("Logout feito com sucesso!");
                            let connected_client = Connected::from_logout(self.info);
                            return connected_client.run();
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                }
                "inicia" => {
                    let listener =
                        TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                    let addr = listener.local_addr().unwrap();
                    self.info.server.send(Message {
                        connection: self.info.connection,
                        message: MessageEnum::CreateGameRequest(CreateGameRequest {
                            listener_addr: addr,
                        }),
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::CreateGameResponse(_))
                    }) {
                        Ok(msg) => {
                            let ServerMessage::CreateGameResponse(response) = msg else { unreachable!(); };
                            if let CreateGameResponse::Ok = response {
                                println!("Created game with success");
                                let pacman_client = Pacman::new(self.info, self.user, listener);
                                return pacman_client.run();
                            } else {
                                println!("Couldn't create a game!");
                            }
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
