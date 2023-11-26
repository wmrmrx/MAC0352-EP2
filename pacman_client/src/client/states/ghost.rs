use std::net::{SocketAddr, TcpStream};

use super::*;

pub struct Ghost {
    info: CommonInfo,
    user: String,
    stream: TcpStream,
}

impl Ghost {
    #[must_use]
    pub fn new(_info: CommonInfo, _user: String, _pacman_addr: SocketAddr) -> Self {
        todo!();
    }

    pub fn fail(self) {
        println!("Falha no jogo P2P!");
        self.info.server.send(Message {
            connection: self.info.connection,
            message: MessageEnum::QuitGameRequest,
        });
        let idle_client = Idle::new(self.info, self.user);
        return idle_client.run();
    }

    pub fn run(self) {}
}
