use std::io::BufRead;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct Shell {
    keep_running: Arc<AtomicBool>,
    allowed_commands: Vec<String>,
}

impl Shell {
    pub fn new<T: ToString>(allowed_commands: &[T], keep_running: Arc<AtomicBool>) -> Self {
        Shell {
            allowed_commands: allowed_commands
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            keep_running,
        }
    }

    pub fn help(&self) {
        println!("Comandos:");
        for command in &self.allowed_commands {
            println!(
                "- {}",
                match command.as_str() {
                    "novo" => "novo <usuario> <senha>",
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

    #[must_use]
    pub fn prompt(&self, decoration: &str) -> Vec<String> {
        loop {
            if !self.keep_running.load(std::sync::atomic::Ordering::Relaxed) {
                println!("Encerrando shell...");
                return Vec::new();
            }
            print!("{decoration} > ");
            std::io::stdout().flush().unwrap();
            let mut lock = std::io::stdin().lock();
            let mut line = String::new();
            let _ = lock.read_line(&mut line).unwrap();
            let tokens: Vec<String> = line
                .split_whitespace()
                .map(std::borrow::ToOwned::to_owned)
                .collect();
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
                        Err("novo <usuario> <senha>")
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
}
