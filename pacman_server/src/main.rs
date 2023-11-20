mod server;

use clap::Parser;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    Config,
};
use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port: u16,
    #[arg(short, long, default_value = "pacman_server_config")]
    config_dir: PathBuf,
}

fn main() {
    // Read arguments
    let args = Args::parse();

    // Set current directory to configuration directory
    let config_dir_path = Path::new(&args.config_dir);
    if config_dir_path.exists() {
        if !config_dir_path.is_dir() {
            panic!("Configuration directory path supplied already exists and is not a directory");
        }
    } else {
        std::fs::create_dir(config_dir_path).expect("Failed to create configuration directory");
    }
    std::env::set_current_dir(config_dir_path)
        .expect("Failed to set current directory to configuration directory");

    // Setup logging
    let log_file = FileAppender::builder().build("log/log.txt").unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("log_file", Box::new(log_file)))
        .build(
            Root::builder()
                .appender("log_file")
                .build(LevelFilter::Info),
        )
        .unwrap();
    log4rs::init_config(config).unwrap();

    log::info!("New server is initialized");

    loop {
        log::info!("Starting new game");
        let keep_running = Arc::new(AtomicBool::new(true));
        server::run(args.port, keep_running);
    }
}
