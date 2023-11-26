use std::net::{TcpListener, TcpStream, SocketAddr};

use super::*;

pub struct Ghost {
    info: CommonInfo,
    user: String,
    stream: TcpStream,
}

impl Ghost {
    pub fn new(info: CommonInfo, user: String, pacman_addr: SocketAddr) -> Self {
        todo!();
    }

    pub fn run(self) {
    }
}
