pub const ID_LENGTH: usize = 20;
pub const BUCKET_SIZE: usize = 20;
pub const ALPHA: usize = 5;
pub const RT_BCKT_SIZE: usize = ID_LENGTH << 3;

pub mod rpc {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
    pub enum Command {
        PING,
        PONG,
        FINDNODE,
        FINDVALUE,
        STORE,
    }
}

pub const ALL_IPV4: &str = "0.0.0.0";
