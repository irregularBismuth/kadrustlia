use crate::constants::rpc::Command;
use crate::kademlia_id::KademliaID;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum RpcMessage {
    Request {
        id: KademliaID,
        method: Command,
        params: Vec<String>,
    },
    Response {
        id: KademliaID,
        result: Command,
    },
    Error {
        id: KademliaID,
        message: String,
    },
}
