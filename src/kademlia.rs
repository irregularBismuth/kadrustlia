use crate::{
    cli::Cli, constants::rpc::Command, contact::Contact, kademlia_id::KademliaID,
    networking::Networking, routing_table::RoutingTable, utils,
};
pub struct Kademlia {
    route_table: RoutingTable,
    cli: Cli,
}

impl Kademlia {
    pub fn new() -> Self {
        let kad_id = KademliaID::new();
        let addr = utils::get_own_address();
        println!("my addr is {}", addr);
        let contact: Contact = Contact::new(kad_id, addr);
        Self {
            route_table: RoutingTable::new(contact),
            cli: Cli::new(),
        }
    }
    pub async fn join(&mut self) {
        if utils::check_bn() {
            return;
        }
        let boot_node_addr: String = utils::boot_node_address();
        let boot_node_addr: String = format!("{}:{}", boot_node_addr, "5678");
        println!("Boot node address: {}", boot_node_addr);

        Networking::send_ping(&boot_node_addr, Command::PING)
            .await
            .expect("failed to send PING");
    }

    pub async fn start_cli(&mut self) {
        self.cli.read_input().await;
    }
}
