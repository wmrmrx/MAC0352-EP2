use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use pacman_communication::{current_time, game::Game};

use super::*;

pub struct Ghost {
    info: CommonInfo,
    user: String,
    pacman_user: String,
    stream: TcpStream,
    latencies: Vec<(Duration, String)>,
}

impl Ghost {
    pub fn new_and_run(
        info: CommonInfo,
        user: String,
        pacman_addr: SocketAddr,
        pacman_user: String,
    ) {
        if let Ok(mut stream) = TcpStream::connect(pacman_addr) {
            stream.write_all(user.as_bytes()).unwrap();
            println!("Conectado ao Pacman com sucesso!");
            Self {
                info,
                user,
                stream,
                pacman_user,
                latencies: Vec::new(),
            }
            .run()
        } else {
            println!("Conexão ao Pacman não foi bem sucedida!");
            info.server.send(Message {
                connection: info.connection,
                message: MessageEnum::QuitGameRequest,
            });
            let idle_client = Idle::new(info, user);
            idle_client.run()
        }
    }

    pub fn finish(self) {
        println!("Saindo do jogo!");
        self.info.server.send(Message {
            connection: self.info.connection,
            message: MessageEnum::QuitGameRequest,
        });
        let idle_client = Idle::new(self.info, self.user);
        idle_client.run()
    }

    pub fn fail(self) {
        println!("Falha no jogo P2P!");
        self.info.server.send(Message {
            connection: self.info.connection,
            message: MessageEnum::QuitGameRequest,
        });
        let idle_client = Idle::new(self.info, self.user);
        idle_client.run()
    }

    fn run(mut self) {
        let mut buf = [0u8; 9001];
        loop {
            let Ok(amt) = self.stream.read(&mut buf) else { return self.fail(); };
            let mut game: Game =
                serde_json::from_str(std::str::from_utf8(&buf[..amt]).unwrap()).unwrap();
            game.show();
            println!("Seu turno!");
            if game.game_over() {
                return self.finish();
            }
            let commands = ["move", "atraso", "encerra"];
            let shell = Shell::new(&commands);
            loop {
                let command = shell.prompt(&format!("{} - PACMAN", &self.user));
                match command[0].as_str() {
                    "move" => {
                        let dir = command[1].chars().next().unwrap();
                        game.move_pacman(dir);
                        break;
                    }
                    "atraso" => {
                        if self.latencies.is_empty() {
                            println!("Sem latências medidas!");
                            continue;
                        }
                        let len = self.latencies.len().min(3);
                        println!("Últimas latências:");
                        println!("{:?}", &self.latencies[self.latencies.len() - 1 - len..]);
                    }
                    "encerra" => {
                        return self.finish();
                    }
                    _ => unreachable!(),
                }
            }
            let start = current_time();
            if self
                .stream
                .write_all(serde_json::to_string(&game).unwrap().as_bytes())
                .is_err()
            {
                return self.fail();
            }
            self.latencies
                .push((current_time() - start, self.pacman_user.clone()));
            if game.game_over() {
                return self.finish();
            }
        }
    }
}
