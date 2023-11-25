use std::{time::Duration, sync::mpsc::RecvTimeoutError};

use pacman_communication::{
    client_server::{Message, MessageEnum},
    server_client::Message as ServerMessage, current_time
};

use super::*;

pub struct Connected {
    info: CommonInfo
}

impl Connected {
    pub fn new(server: Connection, connection: Connection, mut recv: Receiver<ServerMessage>, keep_running: Arc<AtomicBool>) -> Option<Self> {
        server.send(Message {
            connection: connection.clone(),
            message: MessageEnum::ConnectRequest
        });
        // 10 seconds for timeout
        let timeout = Duration::from_secs(10);
        let start = current_time();
        loop {
            if current_time() - start > timeout {
                break;
            }
            match recv.recv_timeout(RECV_TIMEOUT) {
                Ok(message) => {
                    if let ServerMessage::ConnectResponse = message {
                        recv = heartbeat::setup(server.clone(), connection.clone(), recv, keep_running.clone());
                        let connected_client = Connected {
                            info: CommonInfo { server, connection, recv, keep_running }
                        };
                        return Some(connected_client);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
        None
    }

    pub fn run(self) {
        println!(">>> CONECTADO AO SERVIDOR COM SUCESSO!");
        let commands = ["novo", "senha", "entra", "lideres"];
    }
}
