use std::{time::Duration, sync::{mpsc::RecvTimeoutError, atomic::Ordering}};

use pacman_communication::{
    client_server::{Message, MessageEnum},
    server_client::Message as ServerMessage, current_time
};

use super::*;

pub struct Connected {
    _dummy: () // dummy field to make constructor private
}

impl Connected {
    pub fn new(info: &mut Info) -> Option<Self> {
        info.server.send(Message {
            connection: info.connection.clone(),
            message: MessageEnum::ConnectRequest
        });
        // 10 seconds for timeout
        let timeout = Duration::from_secs(10);
        let start = current_time();
        loop {
            if current_time() - start > timeout {
                break;
            }
            match info.recv.recv_timeout(RECV_TIMEOUT) {
                Ok(message) => {
                    if let ServerMessage::ConnectResponse = message {
                        heartbeat::setup(info.server, info.recv);
                        let connected_client = Some(Connected {
                            _dummy: ()
                        });
                        return connected_client;
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

    pub fn run(self, info: Info) {
    }
}
