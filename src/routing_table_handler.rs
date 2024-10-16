use tokio::sync::mpsc;

use crate::{
    constants::BUCKET_SIZE, contact::Contact, kademlia_id::KademliaID, routing_table::RoutingTable,
};

pub enum RouteTableCMD {
    AddContact(Contact),
    RemoveContact(KademliaID),
    GetClosestNodes(KademliaID, mpsc::Sender<Vec<Contact>>),
    GetBucketIndex(KademliaID, mpsc::Sender<usize>),
}

pub async fn routing_table_handler(
    mut rx: mpsc::Receiver<RouteTableCMD>,
    mut routing_table: RoutingTable,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            RouteTableCMD::AddContact(contact) => {
                routing_table.add_contact(contact);
            }
            RouteTableCMD::RemoveContact(_kad_id) => {
                //                println!("Remove  contact");
            }
            RouteTableCMD::GetClosestNodes(target_id, reply) => {
                let contacts = routing_table.find_closest_contacts(target_id, BUCKET_SIZE);
                let _ = reply.send(contacts).await;
            }
            RouteTableCMD::GetBucketIndex(kad_id, reply) => {
                let index = routing_table.get_bucket_index(kad_id);
                let _ = reply.send(index).await;
            }
        }
    }
}
