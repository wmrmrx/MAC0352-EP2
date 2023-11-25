use crate::client::CommonInfo;

use super::*;

pub struct Idle {
    info: CommonInfo,
}

impl Idle {
    pub fn new(info: CommonInfo) -> Self {
        Self { info }
    }
    pub fn run(self) {
        todo!()
    }
}
