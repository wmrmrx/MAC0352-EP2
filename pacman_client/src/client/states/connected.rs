use std::{sync::mpsc::RecvTimeoutError, time::Duration};

use pacman_communication::{
    client_server::{Message, MessageEnum},
    current_time,
    server_client::Message as ServerMessage,
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
        let commands = ["novo", "senha", "entra", "lideres", "l", "tchau"];
        let shell = Shell::new(&commands);
        loop {
            let command = shell.prompt();
            match command[0].as_str() {
                "novo" => {
                    let (user, passwd) = (&command[1], &command[2]);
                    if shell.login(&self.info, user.to_owned(), passwd.to_owned()) {
                        let idle_client = Idle::new(self.info);
                        return idle_client.run();
                    }
                }
                _ => todo!(),
            }
        }
    }
}
