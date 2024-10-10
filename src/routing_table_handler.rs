use tokio::sync::mpsc;

use crate::{constants::{ALPHA, BUCKET_SIZE}, contact::Contact, kademlia_id::KademliaID, routing_table::RoutingTable};

pub enum RouteTableCMD {
    AddContact(Contact),
    RemoveContact(KademliaID),
    GetClosestNodes(KademliaID, mpsc::Sender<Vec<Contact>>),
}

pub async fn routing_table_handler(
    mut rx: mpsc::Receiver<RouteTableCMD>,
    mut routing_table: RoutingTable,
) {
    println!("route table handler");
    while let Some(cmd) = rx.recv().await {
        match cmd {
            RouteTableCMD::AddContact(contact) => {
                //let kad_id = contact.id.clone();
                routing_table.add_contact(contact);
                //println!("{:?}", routing_table.find_closest_contacts(kad_id, 5));
            }
            RouteTableCMD::RemoveContact(kad_id) => {
                println!("remove  coibntact");
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
