use tokio::sync::mpsc;

use crate::{
    constants::{ALPHA, BUCKET_SIZE},
    contact::Contact,
    kademlia_id::KademliaID,
    routing_table::RoutingTable,
};

pub enum RouteTableCMD {
    AddContact(Contact),
    RemoveContact(KademliaID),
    GetClosestNodes(KademliaID, mpsc::Sender<Vec<Contact>>), // No changes required here, it's already used correctly
}

// Ensure the handler function is correctly implemented:
pub async fn routing_table_handler(
    mut rx: mpsc::Receiver<RouteTableCMD>,
    mut routing_table: RoutingTable,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            RouteTableCMD::AddContact(contact) => {
                routing_table.add_contact(contact);
            }
            RouteTableCMD::RemoveContact(kad_id) => {
                routing_table.remove_contact(kad_id);
            }
            RouteTableCMD::GetClosestNodes(target_id, reply) => {
                let contacts = routing_table.find_closest_contacts(target_id, BUCKET_SIZE);
                //println!("target_id: {:?}", target_id);
                //println!("contacts: {:?}", contacts);
                let _ = reply.send(contacts).await;
            }
        }
    }
}
