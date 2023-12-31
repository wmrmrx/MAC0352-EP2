use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use pacman_communication::{current_time, game::Game};

use super::{CommonInfo, Idle, Message, MessageEnum, Shell};

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
            stream
                .set_read_timeout(Some(Duration::from_secs(60)))
                .unwrap();
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
        drop(self.stream);
        self.info.server.send(Message {
            connection: self.info.connection,
            message: MessageEnum::QuitGameRequest,
        });
        let idle_client = Idle::new(self.info, self.user);
        idle_client.run()
    }

    pub fn fail(self) {
        println!("Falha no jogo P2P!");
        drop(self.stream);
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
            println!("Aguardando pelo turno de {}....", &self.pacman_user);
            let mut game: Game;
            let Ok(amt) = self.stream.read(&mut buf) else { return self.fail(); };
            if amt == 0 {
                println!("Conexão fechada!");
            }
            if let Ok(remote_game) = serde_json::from_str(std::str::from_utf8(&buf[..amt]).unwrap())
            {
                game = remote_game;
            } else {
                return self.fail();
            }
            game.show();
            println!("Seu turno!");
            if game.game_over() {
                return self.finish();
            }
            let commands = ["move", "atraso", "encerra"];
            let shell = Shell::new(&commands, self.info.keep_running.clone());
            loop {
                let command = shell.prompt(&format!("{} - GHOST", &self.user));
                if command.is_empty() {
                    continue;
                }
                match command[0].as_str() {
                    "move" => {
                        let dir = command[1].chars().next().unwrap();
                        game.move_remote_ghost(dir);
                        break;
                    }
                    "atraso" => {
                        if self.latencies.is_empty() {
                            println!("Sem latências medidas!");
                            continue;
                        }
                        let len = self.latencies.len().min(3);
                        println!("Últimas latências:");
                        println!("{:?}", &self.latencies[self.latencies.len() - len..]);
                    }
                    "encerra" => {
                        return self.finish();
                    }
                    _ => unreachable!(),
                }
            }
            let game_str = serde_json::to_string(&game).unwrap();
            let start = current_time();
            if self.stream.write_all(game_str.as_bytes()).is_err() {
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
