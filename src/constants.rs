pub const ID_LENGTH: usize = 20;
pub const BUCKET_SIZE: usize = 20;
pub const ALPHA: usize = 3;
pub const RT_BCKT_SIZE: usize = ID_LENGTH << 3;

pub mod rpc {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug)]
    pub enum Command {
        PING,
        PONG,
    }
}
