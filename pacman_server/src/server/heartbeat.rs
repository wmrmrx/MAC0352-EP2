use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use super::game::ConnectionTable;

/// The interval where we watch if the connections expired
const WATCH_INTERVAL: Duration = Duration::from_secs(3);

use pacman_communication::{
    current_time, server_client::Message, HEARTBEAT_INTERVAL, HEARTBEAT_TIMEOUT,
};

/// Watchs for `HEARTBEAT_TIMEOUT` and also sends heartbeats every `HEARTBEAT_INTERVAL`
pub fn setup(conn_table: Arc<Mutex<ConnectionTable>>) {
    {
        // Heartbeat watcher thread
        let conn_table = conn_table.clone();
        std::thread::spawn(move || loop {
            {
                let mut conn_table = conn_table.lock().unwrap();
                let mut expired = Vec::new();
                let now = current_time();
                for (conn, conn_data) in conn_table.get_connections().iter() {
                    if now - conn_data.last_heartbeat > HEARTBEAT_TIMEOUT {
                        expired.push(*conn);
                    }
                }
                for conn in expired {
                    conn_table.remove(&conn);
                }
            }
            std::thread::sleep(WATCH_INTERVAL);
        });
    }

    std::thread::spawn(move || loop {
        {
            let conn_table = conn_table.lock().unwrap();
            for (conn, _) in conn_table.get_connections().iter() {
                conn.send(Message::Heartbeat);
            }
        }
        std::thread::sleep(HEARTBEAT_INTERVAL);
    });
}
