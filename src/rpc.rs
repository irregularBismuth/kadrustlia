use crate::constants::rpc::Command;
use crate::contact::Contact;
use crate::kademlia_id::KademliaID;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum RpcMessage {
    Request {
        rpc_id: KademliaID,
        method: Command,
        target_id: Option<KademliaID>,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    },
    Response {
        rpc_id: KademliaID,
        result: Command,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    },
    Error {
        rpc_id: KademliaID,
        message: String,
    },
}
