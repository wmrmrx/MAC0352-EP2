use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

use pacman_communication::{client_server::CreateGameRequest, server_client::CreateGameResponse};

use super::*;

pub struct Idle {
    info: CommonInfo,
    user: String,
}

impl Idle {
    pub fn new(info: CommonInfo, user: String) -> Self {
        Self { info, user }
    }
    pub fn run(self) {
        println!("You are idle and logged in as {}", &self.user);
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
                            continue;
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
                            continue;
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
                            } else {
                                println!("Couldn't create a game!");
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
