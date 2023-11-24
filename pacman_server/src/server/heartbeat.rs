use std::{sync::{Arc, Mutex}, time::Duration};

use super::{game::ConnectionTable, current_time};

const TICK: Duration = Duration::from_secs(1);

use pacman_communication::HEARTBEAT_TIMEOUT;

/// Watchs for HEARTBEAT_TIMEOUT
pub fn watch(conn_table: Arc<Mutex<ConnectionTable>>) {
    std::thread::spawn(move || {
        loop {
            let mut conn_table = conn_table.lock().unwrap();
            let mut expired = Vec::new();
            let now = current_time();
            for (conn, conn_data) in conn_table.get_connections().iter() {
                if conn_data.last_heartbeat - now > HEARTBEAT_TIMEOUT {
                    expired.push(conn.clone());
                }
            }
            for conn in expired {
                conn_table.remove(&conn);
            }
            std::thread::sleep(TICK);
        }
    });
}
