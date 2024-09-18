use crate::{
    constants::rpc::Command, contact::Contact, kademlia_id::KademliaID, networking::Networking,
    routing_table::RoutingTable, utils,
};
pub struct Kademlia {
    route_table: RoutingTable,
}

impl Kademlia {
    pub fn new() -> Self {
        let kad_id = KademliaID::new();
        let addr = utils::get_own_address();
        let contact: Contact = Contact::new(kad_id, addr);
        Self {
            route_table: RoutingTable::new(contact),
        }
    }
    pub async fn join(&mut self) {
        if utils::check_bn() {
            return;
        }
        let boot_node_addr: String = utils::boot_node_address();
        println!(" boot node address {}", boot_node_addr);
        let boot_node_addr: String = format!("{}:{}", boot_node_addr, "5678");
        tokio::spawn(async move {
            Networking::send_ping(&boot_node_addr, Command::PING)
                .await
                .expect("failed to send PING");
        });
    }
}
