use std::{fs::read_to_string, io::BufRead};

use pacman_communication::client_server::{LoginRequest, Message, MessageEnum};
use pacman_communication::server_client::{LoginResponse, Message as ServerMessage};

use super::event::watch;
use super::CommonInfo;

pub struct Shell {
    allowed_commands: Vec<String>,
}

impl Shell {
    pub fn new<T: ToString>(allowed_commands: &[T]) -> Self {
        Shell {
            allowed_commands: allowed_commands.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn help(&self) {
        println!("Comandos:");
        for command in self.allowed_commands.iter() {
            println!(
                "- {}",
                match command.as_str() {
                    "novo" => "novo <usario> <senha>",
                    "senha" => "senha <senha antiga> <senha nova>",
                    "entra" => "entra <usuario> <senha>",
                    "lideres" => "lideres",
                    "l" => "l",
                    "inicia" => "inicia",
                    "desafio" => "desafio <oponente>",
                    "move" => "move <direcao (wasd)>",
                    "atraso" => "atraso",
                    "encerra" => "encerra",
                    "sai" => "sai",
                    "tchau" => "tchau",
                    _ => unreachable!(),
                }
            );
        }
    }

    pub fn prompt(&self) -> Vec<String> {
        let mut lock = std::io::stdin().lock();
        loop {
            print!("> ");
            let mut line = String::new();
            let _ = lock.read_line(&mut line).unwrap();
            let tokens: Vec<String> = line.split_whitespace().map(|s| s.to_owned()).collect();
            if tokens.is_empty() {
                continue;
            }
            let command = &tokens[0];
            if self.allowed_commands.iter().all(|c| c != command) {
                println!("Comando {command} não reconhecido");
                self.help();
                continue;
            }
            let len = tokens.len();
            match match command.as_str() {
                "novo" => {
                    if len == 3 {
                        Ok(())
                    } else {
                        Err("novo <usario> <senha>")
                    }
                }
                "senha" => {
                    if len == 3 {
                        Ok(())
                    } else {
                        Err("senha <senha antiga> <senha nova>")
                    }
                }
                "entra" => {
                    if len == 3 {
                        Ok(())
                    } else {
                        Err("entra <usuario> <senha>")
                    }
                }
                "lideres" => {
                    if len == 1 {
                        Ok(())
                    } else {
                        Err("lideres")
                    }
                }
                "l" => {
                    if len == 1 {
                        Ok(())
                    } else {
                        Err("l")
                    }
                }
                "inicia" => {
                    if len == 1 {
                        Ok(())
                    } else {
                        Err("inicia")
                    }
                }
                "desafio" => {
                    if len == 2 {
                        Ok(())
                    } else {
                        Err("desafio <oponente>")
                    }
                }
                "move" => {
                    if len == 2 && ["w", "a", "s", "d"].contains(&tokens[1].as_str()) {
                        Ok(())
                    } else {
                        Err("move <direcao (wasd)>")
                    }
                }
                "atraso" => {
                    if len == 1 {
                        Ok(())
                    } else {
                        Err("atraso")
                    }
                }
                "encerra" => {
                    if len == 1 {
                        Ok(())
                    } else {
                        Err("encerra")
                    }
                }
                "sai" => {
                    if len == 1 {
                        Ok(())
                    } else {
                        Err("sai")
                    }
                }
                "tchau" => {
                    if len == 1 {
                        Ok(())
                    } else {
                        Err("tchau")
                    }
                }
                _ => unreachable!(),
            } {
                Ok(()) => return tokens,
                Err(hint) => {
                    println!("Argumentos inválidos: `{hint}`");
                    return Vec::new();
                }
            }
        }
    }

    pub fn login(&self, info: &CommonInfo, user: String, passwd: String) -> bool {
        info.server.send(Message {
            connection: info.connection.clone(),
            message: MessageEnum::LoginRequest(LoginRequest { user, passwd }),
        });
        match watch(&info.recv, |msg| -> bool {
            matches!(msg, ServerMessage::LoginResponse(_))
        }) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
}
