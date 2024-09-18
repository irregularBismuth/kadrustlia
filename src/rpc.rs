use crate::constants::rpc::Command;
use crate::kademlia_id::KademliaID;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum RpcMessage {
    Request {
        id: u64,
        method: Command,
        params: Vec<String>,
    },
    Response {
        id: u64,
        result: Command,
    },
    Error {
        id: u64,
        message: String,
    },
}
