use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{atomic::Ordering, Mutex},
    time::Duration,
};

use pacman_communication::{current_time, game::Game, LeaderboardEntry};
use rand::seq::SliceRandom;

use super::*;

pub struct Pacman {
    info: CommonInfo,
    keep_running: Arc<AtomicBool>,
    user: String,
    connection: Arc<Mutex<Option<(TcpStream, String)>>>,
    latencies: Vec<(Duration, String)>,
}

impl Pacman {
    #[must_use]
    pub fn new(info: CommonInfo, user: String, listener: TcpListener) -> Self {
        let keep_running = Arc::new(AtomicBool::new(true));
        let keep_running1 = keep_running.clone();
        let connection = Arc::new(Mutex::new(None));
        let connection1 = connection.clone();
        std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while keep_running1.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(33));
                if let Ok((mut stream, _)) = listener.accept() {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(60)))
                        .unwrap();
                    let mut buf = [0u8; 9001];
                    let Ok(amt) = stream.read(&mut buf) else {
                            continue;
                        };
                    // Start of connection: Ghost should send its user
                    let Ok(ghost_user) = std::str::from_utf8(&buf[..amt]) else { continue; };
                    let mut conn = connection1.lock().unwrap();
                    if conn.is_none() {
                        println!("Aceitando desafio de {ghost_user}");
                        *conn = Some((stream, ghost_user.to_owned()));
                    }
                    drop(conn);
                }
            }
        });
        Self {
            info,
            keep_running,
            user,
            connection,
            latencies: Vec::new(),
        }
    }

    pub fn fail(self) {
        println!("Falha no jogo P2P!");
        let mut conn = self.connection.lock().unwrap();
        if let Some((stream, _)) = conn.as_mut() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        drop(conn);
        self.info.server.send(Message {
            connection: self.info.connection,
            message: MessageEnum::QuitGameRequest,
        });
        self.keep_running.store(false, Ordering::Relaxed);
        let idle_client = Idle::new(self.info, self.user);
        idle_client.run()
    }

    pub fn finish(self, game: Game) {
        println!("Jogo P2P encerrado com pontuação {}!", game.score());
        let mut conn = self.connection.lock().unwrap();
        if let Some((stream, _)) = conn.as_mut() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        drop(conn);
        self.info.server.send(Message {
            connection: self.info.connection,
            message: MessageEnum::AddLeaderboardEntry(LeaderboardEntry {
                score: game.score(),
                user: self.user.clone(),
            }),
        });
        self.keep_running.store(false, Ordering::Relaxed);
        let idle_client = Idle::new(self.info, self.user);
        idle_client.run()
    }

    pub fn run(mut self) {
        let mut game = Game::new();
        game.show();
        loop {
            // Local ghost's turn
            let mut array = ['w', 'a', 's', 'd'];
            let mut rng = rand::thread_rng();
            array.shuffle(&mut rng);
            let random_dir = array[0];
            game.move_local_ghost(random_dir);
            if game.game_over() {
                let mut conn = self.connection.lock().unwrap();
                if let Some((stream, _)) = conn.as_mut() {
                    let _ = stream.write_all(serde_json::to_string(&game).unwrap().as_bytes());
                }
                drop(conn);
                return self.finish(game.clone());
            }

            // Remote ghost's turn
            let mut conn = self.connection.lock().unwrap();
            if let Some((stream, ghost_user)) = conn.as_mut() {
                game.add_remote_ghost();
                println!("Esperando pelo turno de {}", ghost_user);
                let game_str = serde_json::to_string(&game).unwrap();
                let mut buf = [0u8; 9001];
                let start = current_time();
                if stream.write_all(game_str.as_bytes()).is_err() {
                    println!("Erro de conexão com o usuário {}", ghost_user);
                    *conn = None;
                } else {
                    let latency = current_time() - start;
                    self.latencies.push((latency, ghost_user.to_owned()));
                    if let Ok(amt) = stream.read(&mut buf) {
                        if amt == 0 {
                            println!("Conexão fechada!");
                            *conn = None;
                        } else if let Ok(remote_game) =
                            serde_json::from_str(std::str::from_utf8(&buf[..amt]).unwrap())
                        {
                            game = remote_game
                        } else {
                            println!("Erro de conexão com o usuário {}", ghost_user);
                            *conn = None;
                        }
                    } else {
                        println!("Erro de conexão com o usuário {}", ghost_user);
                        *conn = None;
                    }
                }
            } else {
                *conn = None;
            }
            drop(conn);
            if game.game_over() {
                return self.finish(game);
            }

            // Our turn
            game.show();
            println!("Seu turno!");
            let commands = ["move", "atraso", "encerra"];
            let shell = Shell::new(&commands, self.info.keep_running.clone());
            loop {
                let command = shell.prompt(&format!("{} - PACMAN", &self.user));
                if command.is_empty() {
                    continue;
                }
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
                        return self.finish(game);
                    }
                    _ => unreachable!(),
                }
            }
            if game.game_over() {
                let mut conn = self.connection.lock().unwrap();
                if let Some((stream, _)) = conn.as_mut() {
                    let _ = stream.write_all(serde_json::to_string(&game).unwrap().as_bytes());
                }
                drop(conn);
                return self.finish(game.clone());
            }
        }
    }
}
