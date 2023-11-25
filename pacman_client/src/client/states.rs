use std::time::Duration;

use pacman_communication::{
    client_server::{Message, MessageEnum},
    server_client::Message as ServerMessage
};

use super::Info;

const TICK: Duration = Duration::from_millis(33);

pub struct Unconnected;

impl Unconnected {
    pub fn try_connect(self, info: Info) {
        info.server.send(Message {
            connection: info.connection.clone(),
            message: MessageEnum::ConnectRequest
        });
        loop {
            if let ServerMessage::ConnectResponse = info.recv.recv().unwrap {
            }
        }
        println!("Couldn't connect!");
    }
}

struct Connected;

struct Idle {
}

struct Pacman;

struct Ghost;

trait IsConnected: Sized {
    fn disconnect(self, info: Info) {
        println!("Disconnecting from server!");
    }
}
impl IsConnected for Connected {}
impl IsConnected for Pacman {}
impl IsConnected for Idle {}
impl IsConnected for Ghost {}

trait IsLoggedIn: Sized + IsConnected {
    fn logout(self, info: Info) {
    }
}
impl IsLoggedIn for Idle {}
impl IsLoggedIn for Pacman {}
impl IsLoggedIn for Ghost {}
