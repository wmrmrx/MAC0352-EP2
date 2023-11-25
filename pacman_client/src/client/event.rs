use std::{
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};
use thiserror::Error;

use pacman_communication::{current_time, PacmanMessage};

#[derive(Error, Debug)]
pub enum WatchErr {
    #[error("timeout waiting for server response")]
    Timeout,
    #[error("disconnected from server")]
    Disconnection,
}

const SERVER_TIMEOUT: Duration = Duration::from_secs(10);
const RECV_TIMEOUT: Duration = Duration::from_millis(33);

/// Watch for events
/// Function returns the message from which f returns true
pub fn watch<MSG: PacmanMessage, F: Fn(&MSG) -> bool>(
    recv: &Receiver<MSG>,
    f: F,
) -> Result<MSG, WatchErr> {
    let start = current_time();
    loop {
        if current_time() - start > SERVER_TIMEOUT {
            return Err(WatchErr::Timeout);
        }
        match recv.recv_timeout(RECV_TIMEOUT) {
            Ok(message) => {
                if f(&message) {
                    return Ok(message);
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                return Err(WatchErr::Disconnection);
            }
        }
    }
}
