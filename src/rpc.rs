use crate::constants::rpc::Command;
use crate::contact::Contact;
use crate::kademlia_id::KademliaID;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum RpcMessage {
    Request {
        id: KademliaID,
        method: Command,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    },
    Response {
        id: KademliaID,
        result: Command,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    },
    Error {
        id: KademliaID,
        message: String,
    },
}
