use crate::client::CommonInfo;



pub struct Idle {
    info: CommonInfo,
    user: String
}

impl Idle {
    pub fn new(info: CommonInfo, user: String) -> Self {
        Self { info, user }
    }
    pub fn run(self) {
        println!("You are idle and logged in as {}", &self.user);
        todo!()
    }
}
