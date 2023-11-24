use std::sync::{mpsc::{Sender, Receiver, channel}, Mutex};

use pacman_communication::PacmanMessage;

// Really inefficient if there's many subscribers since it sends a copy of all messages to everyone
// Used due to ease of implementation
pub struct Publisher<'a> {
    subscribers: &'a Mutex<Vec<Sender<PacmanMessage>>>
}

impl<'a> Publisher<'a> {
    pub fn new<'b: 'a> (subscribers: &'b Mutex<Vec<Sender<PacmanMessage>>>) -> Self {
        Self {
            subscribers
        }
    }

    pub fn subscribe(&mut self) -> Receiver<PacmanMessage> {
        let subscribers: &mut Vec<Sender<PacmanMessage>> = self.subscribers.lock().unwrap().as_mut();
        let (send, recv) = channel();
        subscribers.push(send);
        recv
    }
}
