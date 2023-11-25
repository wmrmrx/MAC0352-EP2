//! Module to deal with persistent user and password data

use std::{
    fs::File,
    io::{Read, Write},
};

use pacman_communication::LeaderboardEntry;

fn user_file_path(user: &str) -> String {
    format!("users/{user}")
}

pub struct Database;

impl Database {
    pub fn new() -> Self {
        let _ = std::fs::create_dir("users");
        Self
    }

    fn open_user_file(&mut self, user: &str) -> Option<File> {
        if let Ok(file) = File::open(user_file_path(user)) {
            Some(file)
        } else {
            None
        }
    }

    pub fn user_exists(&mut self, user: &str) -> bool {
        self.open_user_file(user).is_some()
    }

    /// Returns true if created, false if otherwise
    pub fn create_user(&mut self, user: &str, password: &str) -> bool {
        // Username must be reasonable, forbid whitespaces and limit length to 20
        if user.chars().any(|c| c.is_whitespace()) || user.chars().count() > 20  {
            false
        } else if self.user_exists(user) {
            false
        } else {
            let mut file = File::create(format!("users/{user}")).unwrap();
            file.write_all(password.as_bytes()).unwrap();
            true
        }
    }

    /// Returns true if successful, false otherwise
    pub fn login(&mut self, user: &str, passwd: &str) -> bool {
        if let Some(mut file) = self.open_user_file(user) {
            let mut cur_passwd = String::new();
            let _ = file.read_to_string(&mut cur_passwd).unwrap();
            passwd == cur_passwd
        } else {
            false
        }
    }

    /// Returns true if successful, false otherwise
    pub fn change_password(&mut self, user: &str, old_passwd: &str, new_passwd: &str) -> bool {
        if let Some(mut file) = self.open_user_file(user) {
            let mut cur_passwd = String::new();
            let _ = file.read_to_string(&mut cur_passwd).unwrap();
            if old_passwd == cur_passwd {
                let mut file = File::create(user_file_path(user)).unwrap();
                file.write_all(new_passwd.as_bytes()).unwrap();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    // Keeps top 10
    pub fn add_leaderboard_entry(&mut self, entry: LeaderboardEntry) {
        let mut leaderboard: Vec<LeaderboardEntry> = self.get_leaderboard().to_vec();
        leaderboard.push(entry);
        leaderboard.sort();
        leaderboard.reverse();
        if leaderboard.len() > 10 {
            leaderboard = leaderboard[..10].to_vec();
        }
        let mut file = File::create("leaderboard").unwrap();
        let leaderboard = leaderboard.into_boxed_slice();
        file.write_all(serde_json::to_string(&leaderboard).unwrap().as_bytes()).unwrap();
    }

    pub fn get_leaderboard(&self) -> Box<[LeaderboardEntry]> {
        let Ok(mut file) = File::open("leaderboard") else { return Box::new([]); };
        let mut leaderboard_str = String::new();
        file.read_to_string(&mut leaderboard_str).unwrap();
        serde_json::from_str(&leaderboard_str).unwrap()
    }
}
