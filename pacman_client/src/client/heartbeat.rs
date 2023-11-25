use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, RecvTimeoutError},
        Arc,
    },
    time::Duration,
};

use pacman_communication::{
    client_server::{self, MessageEnum},
    current_time,
    server_client::Message,
    Connection, HEARTBEAT_INTERVAL, HEARTBEAT_TIMEOUT,
};

const RECV_TIMEOUT: Duration = Duration::from_millis(33);

pub fn setup(
    server: Connection,
    connection: Connection,
    recv: Receiver<Message>,
    keep_running: Arc<AtomicBool>,
) -> Receiver<Message> {
    let (send, new_recv) = channel();
    let keep_running_watcher = keep_running.clone();
    std::thread::spawn(move || {
        let mut last_heartbeat = current_time();
        loop {
            if current_time() - last_heartbeat > HEARTBEAT_TIMEOUT {
                println!("Server heartbeat timeout!");
                keep_running_watcher.store(false, Ordering::Relaxed);
                return;
            }
            match recv.recv_timeout(RECV_TIMEOUT) {
                Ok(msg) => {
                    if let Message::Heartbeat = msg {
                        last_heartbeat = current_time();
                    } else {
                        send.send(msg)
                            .expect("Not expected for receiver channel to drop!");
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return;
                }
            }
        }
    });
    std::thread::spawn(move || {
        while keep_running.load(Ordering::Relaxed) {
            server.send(client_server::Message {
                connection: connection.clone(),
                message: MessageEnum::Heartbeat,
            });
            std::thread::sleep(HEARTBEAT_INTERVAL);
        }
    });
    new_recv
}
