pub mod connected;

use std::time::Duration;

use super::Info;

pub use connected::Connected;

const RECV_TIMEOUT: Duration= Duration::from_millis(33);


#[must_use]
struct Idle {
}

#[must_use]
struct Pacman;

#[must_use]
struct Ghost;

trait ClientState {}

trait IsConnected: Sized {
    fn disconnect(self, _: Info) {
        println!("Disconnecting from server!");
        drop(self);
    }
}
impl IsConnected for Connected {}
impl IsConnected for Pacman {}
impl IsConnected for Idle {}
impl IsConnected for Ghost {}

trait IsLoggedIn: Sized + IsConnected {
    fn logout(self, _: Info) {
    }
}
impl IsLoggedIn for Idle {}
impl IsLoggedIn for Pacman {}
impl IsLoggedIn for Ghost {}
