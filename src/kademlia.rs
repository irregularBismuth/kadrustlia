use {
    crate::{
        constants::{rpc::Command, BUCKET_SIZE}, contact::Contact, kademlia_id::KademliaID,
        networking::Networking, routing_table::RoutingTable, utils, routing_table_handler::*,
    },
    tokio::sync::mpsc,
};

#[derive(Clone)]
pub struct Kademlia {
    route_table_tx: mpsc::Sender<RouteTableCMD>,
}

impl Kademlia {
    pub fn new() -> Self {
        let kad_id = KademliaID::new();
        let addr = utils::get_own_address();
        println!("my addr is {}", addr);
        let contact: Contact = Contact::new(kad_id, addr);
        let (tx, rx) = mpsc::channel(32);
        let initial_contact = contact.clone();
        tokio::spawn(async move {
            let routing_table = RoutingTable::new(initial_contact);
            routing_table_handler(rx, routing_table).await;
        });

        Self {
            route_table_tx: tx,
        }
    }

    pub async fn listen(&self, addr: &str) {
        let tx = self.route_table_tx.clone();
        let _ = Networking::listen_for_rpc(tx, addr).await;
    }

    pub async fn join(&self) {
        if utils::check_bn() {
            return;
        }
        let adr: String = utils::boot_node_address();
        let boot_node_addr: String = format!("{}:{}", adr, "5678");
        println!("Boot node address: {}", boot_node_addr);

        Networking::send_rpc_request(&boot_node_addr, Command::PING, None, None, None)
            .await
            .expect("failed to send PING");
    }

    pub async fn find_node(&self, target_id: KademliaID) -> std::io::Result<()> {
        // Implement find node logic
        Ok(())
    }

    pub async fn find_value(&self, target_id: KademliaID) -> std::io::Result<()> {
        let adr: String = utils::boot_node_address();
        let boot_node_addr: String = format!("{}:{}", adr, "5678");
        Networking::send_rpc_request(&boot_node_addr, Command::FINDVALUE, Some(target_id), None, None)
            .await
            .expect("failed");

        println!("Value found or contacts returned");
        Ok(())
    }

    pub async fn store(&self, data: String) -> std::io::Result<()> {
        let mut kad_id = KademliaID::new();
        kad_id.store_data(data.clone()).await;
        println!("Data stored with kademlia id: {}", kad_id.to_hex());
        Ok(())
    }
}
