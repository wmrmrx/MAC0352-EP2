use serde::{Deserialize, Serialize};

const H: usize = 5;
const W: usize = 27;

const WALL: u8 = b'*';
const PACDOT: u8 = b'.';
const EMPTY: u8 = b' ';

const INITIAL_BOARD: [&str; H] = [
    "******.**... .....**.******",
    "******.**.*******.**.******",
    "******.**.*.. ..*.**.******",
    "..... ....*.....*..........",
    "******.**.*.. ..*.**.******",
];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    board: [[u8; W]; H],
    pacman: (usize, usize),
    score: u64,
    local_ghost: (usize, usize),
    remote_ghost: Option<(usize, usize)>,
    ended: bool,
}

impl Game {
    pub fn new() -> Self {
        let board: [[u8; W]; H] = [
            INITIAL_BOARD[0].as_bytes().try_into().unwrap(),
            INITIAL_BOARD[1].as_bytes().try_into().unwrap(),
            INITIAL_BOARD[2].as_bytes().try_into().unwrap(),
            INITIAL_BOARD[3].as_bytes().try_into().unwrap(),
            INITIAL_BOARD[4].as_bytes().try_into().unwrap(),
        ];
        let pacman = (2, 13);
        let score = 0;
        let local_ghost = (3, 24); // totally random initial position
        let remote_ghost = None;
        Self {
            board,
            pacman,
            score,
            local_ghost,
            remote_ghost,
            ended: false,
        }
    }

    pub fn show(&self) {
        println!("Estado do jogo:");
        let mut copy = self.board;
        let (x, y) = self.pacman;
        copy[x][y] = b'P';
        let (x, y) = self.local_ghost;
        copy[x][y] = b'F';
        if let Some((x, y)) = self.remote_ghost {
            copy[x][y] = b'f';
        }
        for line in copy {
            println!("{}", std::str::from_utf8(&line).unwrap());
        }
    }

    pub fn game_over(&self) -> bool {
        self.ended
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    fn dir_vec(dir: char) -> Option<(isize, isize)> {
        match dir {
            'w' => Some((-1, 0)),
            'a' => Some((0, -1)),
            's' => Some((1, 0)),
            'd' => Some((0, 1)),
            _ => None,
        }
    }

    fn update_game_state(&mut self) {
        if self.pacman == self.local_ghost {
            self.ended = true;
        }
        let Some(remote_ghost) = self.remote_ghost else { return; };
        if self.pacman == remote_ghost {
            self.ended = true;
        }
    }

    fn new_position((x, y): (usize, usize), dir: char) -> (usize, usize) {
        let Some((dx, dy)) = Self::dir_vec(dir) else { return (x, y); };
        let (mut nx, mut ny) = (
            (x as isize + dx) % H as isize,
            (y as isize + dy) % W as isize,
        );
        if nx < 0 {
            nx += H as isize;
        }
        if ny < 0 {
            ny += W as isize;
        }
        (nx as usize, ny as usize)
    }

    pub fn move_pacman(&mut self, dir: char) {
        let (nx, ny) = Self::new_position(self.pacman, dir);
        match self.board[nx][ny] {
            WALL => {
                return;
            }
            PACDOT => {
                self.score += 1;
                self.board[nx][ny] = EMPTY;
            }
            EMPTY => {}
            _ => panic!("Invalid byte!"),
        }
        self.pacman = (nx, ny);
        self.update_game_state();
    }

    pub fn move_local_ghost(&mut self, dir: char) {
        let (nx, ny) = Self::new_position(self.local_ghost, dir);
        match self.board[nx][ny] {
            WALL => {
                return;
            }
            PACDOT => {}
            EMPTY => {}
            _ => panic!("Invalid byte!"),
        }
        self.local_ghost = (nx, ny);
        self.update_game_state();
    }

    pub fn move_remote_ghost(&mut self, dir: char) {
        let Some(remote_ghost) = &mut self.remote_ghost else { return; };
        let (nx, ny) = Self::new_position(*remote_ghost, dir);
        match self.board[nx][ny] {
            WALL => {
                return;
            }
            PACDOT => {}
            EMPTY => {}
            _ => panic!("Invalid byte!"),
        }
        *remote_ghost = (nx, ny);
        self.update_game_state();
    }

    pub fn add_remote_ghost(&mut self) {
        if self.remote_ghost.is_none() {
            self.remote_ghost = Some((3, 3)); // totally random starting position
        }
    }

    pub fn remove_remote_ghost(&mut self) {
        self.remote_ghost = None;
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
