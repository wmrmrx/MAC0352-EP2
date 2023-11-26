use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::{atomic::Ordering, Mutex},
    time::Duration,
};

use pacman_communication::LeaderboardEntry;

use crate::client::game::Game;

use super::*;

pub struct Pacman {
    info: CommonInfo,
    keep_running: Arc<AtomicBool>,
    user: String,
    connection: Arc<Mutex<Option<(TcpStream, String)>>>,
}

impl Pacman {
    #[must_use]
    pub fn new(info: CommonInfo, user: String, listener: TcpListener) -> Self {
        let keep_running = Arc::new(AtomicBool::new(true));
        let kr = keep_running.clone();
        let connection = Arc::new(Mutex::new(None));
        let conn = connection.clone();
        std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while kr.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(33));
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let mut buf = [0u8; 9001];
                        let Ok(amt) = stream.read(&mut buf) else {
                            continue;
                        };
                        // Start of connection: Ghost should send its user
                        let Ok(ghost_user) = std::str::from_utf8(&buf[..amt]) else { continue; };
                        let mut conn = conn.lock().unwrap();
                        if conn.is_none() {
                            println!("Aceitando desafio de {ghost_user}");
                            *conn = Some((stream, ghost_user.to_owned()));
                        }
                        drop(conn);
                    }
                    Err(_) => {}
                }
            }
        });
        Self {
            info,
            keep_running,
            user,
            connection,
        }
    }

    pub fn fail(self) {
        println!("Falha no jogo P2P!");
        self.info.server.send(Message {
            connection: self.info.connection,
            message: MessageEnum::QuitGameRequest,
        });
        self.keep_running.store(false, Ordering::Relaxed);
        let idle_client = Idle::new(self.info, self.user);
        return idle_client.run();
    }

    pub fn finish_game(self, game: Game) {
        println!("Jogo P2P encerrado com pontuação {}!", game.score());
        self.info.server.send(Message {
            connection: self.info.connection,
            message: MessageEnum::AddLeaderboardEntry(LeaderboardEntry {
                score: game.score(),
                user: self.user.clone(),
            }),
        });
        self.keep_running.store(false, Ordering::Relaxed);
        let idle_client = Idle::new(self.info, self.user);
        return idle_client.run();
    }

    pub fn run(self) {
        let game = Game::new();
        loop {
        }
    }
}
