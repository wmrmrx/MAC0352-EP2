use crate::Connection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub connection: Connection,
    pub message: MessageEnum,
}

#[derive(Serialize, Deserialize)]
pub enum MessageEnum {
    LatencyCheck,
    RespondLatencyCheck,
}
