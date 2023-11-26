use std::sync::atomic::Ordering;

use super::{
    heartbeat, server_client, watch, CommonInfo, CreateUserRequest, Idle, LoginRequest, Message,
    MessageEnum, ServerMessage, Shell, WatchErr,
};

pub struct Connected {
    info: CommonInfo,
}

impl Connected {
    #[must_use]
    pub fn new(mut info: CommonInfo) -> Option<Self> {
        info.server.send(Message {
            connection: info.connection,
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
                Some(Self { info })
            }
            Err(_) => None,
        }
    }

    #[must_use]
    pub fn from_logout(info: CommonInfo) -> Self {
        Self { info }
    }

    pub fn run(self) {
        println!(">>> CONECTADO AO SERVIDOR COM SUCESSO!");
        let commands = ["novo", "entra", "tchau"];
        let shell = Shell::new(&commands, self.info.keep_running.clone());
        loop {
            let command = shell.prompt("SEM LOGIN");
            if command.is_empty() {
                continue;
            }
            match command[0].as_str() {
                "novo" => {
                    let (user, passwd) = (&command[1], &command[2]);
                    self.info.server.send(Message {
                        connection: self.info.connection,
                        message: MessageEnum::CreateUserRequest(CreateUserRequest {
                            user: user.clone(),
                            passwd: passwd.clone(),
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
                        connection: self.info.connection,
                        message: MessageEnum::LoginRequest(LoginRequest {
                            user: user.clone(),
                            passwd: passwd.clone(),
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
                        }
                        Err(WatchErr::Disconnection) => return,
                    }
                    let idle_client = Idle::new(self.info, user.clone());
                    return idle_client.run();
                }
                "tchau" => {
                    self.info.server.send(Message {
                        connection: self.info.connection,
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
