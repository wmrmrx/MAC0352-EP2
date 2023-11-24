//! Module to deal with persistent user and password data

use std::{
    fs::File,
    io::{Read, Write},
};

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
        if let Ok(file) = File::open(&user_file_path(user)) {
            return Some(file);
        } else {
            return None;
        }
    }

    pub fn user_exists(&mut self, user: &str) -> bool {
        self.open_user_file(user).is_some()
    }

    /// Returns true if created, false if otherwise
    pub fn create_user(&mut self, user: &str, password: &str) -> bool {
        if self.user_exists(&user) {
            false
        } else {
            let mut file = File::create(&format!("users/{user}")).unwrap();
            file.write_all(password.as_bytes()).unwrap();
            true
        }
    }

    /// Returns true if successful, false otherwise
    pub fn login(&mut self, user: &str, passwd: &str) -> bool {
        if let Some(mut file) = self.open_user_file(&user) {
            let mut cur_passwd = String::new();
            let _ = file.read_to_string(&mut cur_passwd).unwrap();
            passwd == cur_passwd
        } else {
            false
        }
    }

    /// Returns true if successful, false otherwise
    pub fn change_password(&self, user: &str, old_passwd: &str, new_passwd: &str) -> bool {
        if let Some(mut file) = self.open_user_file(&user) {
            let mut cur_passwd = String::new();
            let _ = file.read_to_string(&mut cur_passwd).unwrap();
            if old_passwd == cur_passwd {
                let mut file = File::create(&user_file_path(user)).unwrap();
                let _ = file.write_all(new_passwd.as_bytes()).unwrap();
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
