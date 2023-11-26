use std::sync::atomic::Ordering;

use super::*;

pub struct Connected {
    info: CommonInfo,
}

impl Connected {
    pub fn new(
        mut info: CommonInfo
    ) -> Option<Self> {
        info.server.send(Message {
            connection: info.connection.clone(),
            message: MessageEnum::ConnectRequest,
        });
        match watch(&info.recv, |msg| -> bool {
            matches!(msg, ServerMessage::ConnectResponse)
        }) {
            Ok(_) => {
                info.recv = heartbeat::setup(
                    info.server,
                    info.connection,
                    info.recv,
                    info.keep_running.clone(),
                );
                Some(Self {
                    info
                })
            }
            Err(_) => None,
        }
    }

    pub fn from_logout(info: CommonInfo) -> Self {
        Self {
            info
        }
    }

    pub fn run(self) {
        println!(">>> CONECTADO AO SERVIDOR COM SUCESSO!");
        let commands = ["novo", "entra", "tchau"];
        let shell = Shell::new(&commands);
        loop {
            let command = shell.prompt("");
            match command[0].as_str() {
                "novo" => {
                    let (user, passwd) = (&command[1], &command[2]);
                    self.info.server.send(Message {
                        connection: self.info.connection.clone(),
                        message: MessageEnum::CreateUserRequest(CreateUserRequest {
                            user: user.to_owned(),
                            passwd: passwd.to_owned(),
                        }),
                    });
                    match watch(&self.info.recv, |msg| -> bool {
                        matches!(msg, ServerMessage::CreateUserResponse(_))
                    }) {
                        Ok(msg) => {
                            if let ServerMessage::CreateUserResponse(
                                server_client::CreateUserResponse::Err,
                            ) = msg
                            {
                                println!("Erro ao criar usuário (talvez ele já exista)");
                            } else {
                                println!("Usuario criado com sucesso!");
                            }
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
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
                            }
                        }
                        Err(WatchErr::Timeout) => {
                            println!("Timeout esperando pelo servidor!");
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                    let idle_client = Idle::new(self.info, user.to_owned());
                    return idle_client.run();
                }
                "tchau" => {
                    self.info.server.send(Message {
                        connection: self.info.connection.clone(),
                        message: MessageEnum::Disconnect,
                    });
                    self.info.keep_running.store(false, Ordering::Relaxed);
                    return;
                }
                _ => unreachable!(),
            }
        }
    }
}
