use crate::client::CommonInfo;

use super::*;

pub struct Idle {
    info: CommonInfo,
    user: String
}

impl Idle {
    pub fn new(info: CommonInfo, user: String) -> Self {
        Self { info, user }
    }
    pub fn run(self) {
        todo!()
    }
}
