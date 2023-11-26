use std::net::{SocketAddr, TcpStream};

use super::*;

pub struct Ghost {
    info: CommonInfo,
    user: String,
    stream: TcpStream,
}

impl Ghost {
    pub fn new(_info: CommonInfo, _user: String, _pacman_addr: SocketAddr) -> Self {
        todo!();
    }

    pub fn run(self) {}
}
