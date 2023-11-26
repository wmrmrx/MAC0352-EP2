use std::net::{TcpListener, TcpStream};

use super::*;

pub struct Pacman {
    info: CommonInfo,
    user: String,
    listener: TcpListener,
    connection: Option<TcpStream>,
}

impl Pacman {
    pub fn new(info: CommonInfo, user: String, listener: TcpListener) -> Self {
        Self {
            info,
            user,
            listener,
            connection: None,
        }
    }

    pub fn run(self) {
        todo!();
    }
}
