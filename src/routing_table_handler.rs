use tokio::sync::mpsc;

use crate::{contact::Contact, kademlia_id::KademliaID, routing_table::RoutingTable};

pub enum RouteTableCMD {
    AddContact(Contact),
    RemoveContact(KademliaID),
    GetClosestNodes(KademliaID),
}

pub async fn routing_table_handler(
    mut rx: mpsc::Receiver<RouteTableCMD>,
    mut routing_table: RoutingTable,
) {
    println!("route table handler");
    while let Some(cmd) = rx.recv().await {
        match cmd {
            RouteTableCMD::AddContact(contact) => {
                println!("ping  hello ");
            }
            RouteTableCMD::RemoveContact(kad_id) => {
                println!("remove  coibntact");
            }
            RouteTableCMD::GetClosestNodes(kad_id) => {
                println!("kademlia we got {}", kad_id.to_hex());
            }
        }
    }
}
