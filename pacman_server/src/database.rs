//! Module to deal with persistent user and password data

use std::{io::{Write, Read}, fs::{self, File}};

use pacman_communication::{CreateUserResponse, CreateUserRequest, LoginRequest, LoginResponse, ChangePasswordRequest, ChangePasswordResponse};

pub struct Database;

impl Database {
    pub fn new() -> Self {
        let _ = std::fs::create_dir("users");
        Self
    }

    fn open_user_file(&self, user: &str) -> Option<File> {
        let path = format!("users/{user}");
        if let Ok(file) = File::open(&path) {
            return Some(file);
        } else {
            return None;
        }
    }

    pub fn user_exists(&self, user: &str) -> bool {
        self.open_user_file(user).is_some()
    }

    pub fn create_user(&self, CreateUserRequest { user, password }: CreateUserRequest) -> CreateUserResponse {
        if self.user_exists(&user) {
            CreateUserResponse::Rejected
        } else {
            if let Ok(mut file) = File::create(&format!("users/{user}")) {
                file.write_all(password.as_bytes()).unwrap();
                CreateUserResponse::Created
            } else {
                CreateUserResponse::Rejected
            }
        }
    }

    pub fn login_request(&self, LoginRequest { user, password }: LoginRequest) -> LoginResponse {
        if let Some(mut file) = self.open_user_file(&user) {
            let mut current_password = String::new();
            let _ = file.read_to_string(&mut current_password).unwrap();
            if password == current_password {
                LoginResponse::LoggedIn
            } else {
                LoginResponse::Failed
            }
        } else {
            LoginResponse::Failed
        }
    }

    pub fn change_password_request(&self, user: &str, ChangePasswordRequest { old_password, new_password }: ChangePasswordRequest) -> ChangePasswordResponse {
        if let Some(mut file) = self.open_user_file(&user) {
            let mut current_password = String::new();
            let _ = file.read_to_string(&mut current_password).unwrap();
            if old_password == current_password {
                let path = format!("users/{user}");
                let mut file = File::create(&path).unwrap();
                let _ = file.write_all(new_password.as_bytes()).unwrap();
                ChangePasswordResponse::Success
            } else {
                ChangePasswordResponse::Fail
            }
        } else {
            ChangePasswordResponse::Fail
        }
    }
}
