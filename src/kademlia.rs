use {
    crate::{
        cli::Cli,
        constants::{rpc::Command, ALPHA, BUCKET_SIZE},
        contact::Contact,
        kademlia_id::KademliaID,
        networking::Networking,
        routing_table::RoutingTable,
        routing_table_handler::*,
        utils,
    },
    tokio::sync::mpsc,
};

#[derive(Clone)]
pub struct Kademlia {
    route_table_tx: mpsc::Sender<RouteTableCMD>,
    cli: Cli,
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
            cli: Cli::new(),
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

    pub async fn find_node(self, target_id: KademliaID) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn iterative_find_node(
        &self,
        target_id: KademliaID,
    ) -> std::io::Result<Vec<Contact>> {
        println!(
            "Starting iterative find node for target ID: {}",
            target_id.to_hex()
        );

        let mut shortlist: Vec<(Contact, bool)> = Vec::new();

        let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);
        let _ = self
            .route_table_tx
            .send(RouteTableCMD::GetClosestNodes(target_id, reply_tx))
            .await;

        if let Some(initial_contacts) = reply_rx.recv().await {
            for contact in initial_contacts.into_iter().take(ALPHA) {
                println!(
                    "Adding contact {} to initial shortlist",
                    contact.id.to_hex()
                );
                shortlist.push((contact, false));
            }
        }

        let mut closest_node_seen = None;
        let mut closest_distance = KademliaID::new().distance(&target_id);

        while !shortlist.is_empty() {
            let unqueried_contacts: Vec<Contact> = shortlist
                .iter()
                .filter(|(_, queried)| !queried)
                .take(ALPHA)
                .map(|(contact, _)| contact.clone())
                .collect();

            if unqueried_contacts.is_empty() {
                println!("No more unqueried contacts left in shortlist. Ending lookup.");
                break;
            }

            let mut tasks = vec![];
            for contact in &unqueried_contacts {
                println!("Querying contact: {}", contact.id.to_hex());
                let target_addr = format!("{}:{}", contact.address, "5678");
                let target_id_copy = target_id.clone();
                let task = tokio::spawn(async move {
                    Networking::send_rpc_request(
                        &target_addr,
                        Command::FINDNODE,
                        Some(target_id_copy),
                        None,
                        None,
                    )
                    .await
                });
                tasks.push((task, contact.clone()));
            }

            for (task, queried_contact) in tasks {
                if let Ok(Ok(())) = task.await {
                    println!(
                        "Received response from contact: {}",
                        queried_contact.id.to_hex()
                    );

                    if let Some(received_contacts) = reply_rx.recv().await {
                        for new_contact in received_contacts {
                            println!(
                                "Received new contact: {} from response",
                                new_contact.id.to_hex()
                            );
                            let distance = new_contact.id.distance(&target_id);

                            if distance.less(&closest_distance) {
                                println!(
                                    "Found closer contact: {} with distance {}",
                                    new_contact.id.to_hex(),
                                    distance.to_hex()
                                );
                                closest_node_seen = Some(new_contact.clone());
                                closest_distance = distance;
                            }

                            if !shortlist.iter().any(|(c, _)| c.id == new_contact.id) {
                                println!(
                                    "Adding new contact: {} to shortlist",
                                    new_contact.id.to_hex()
                                );
                                shortlist.push((new_contact, false));
                            }
                        }
                    }
                } else {
                    println!(
                        "Failed to receive response from contact: {}. Marking as unreachable.",
                        queried_contact.id.to_hex()
                    );
                    shortlist.retain(|(contact, _)| contact.id != queried_contact.id);
                }
            }

            for contact in &mut shortlist {
                if unqueried_contacts.iter().any(|c| c.id == contact.0.id) {
                    contact.1 = true;
                }
            }

            let active_contacts: Vec<_> = shortlist
                .iter()
                .filter(|(_, queried)| *queried)
                .map(|(contact, _)| contact.clone())
                .collect();

            if active_contacts.len() >= BUCKET_SIZE {
                println!(
                    "Found {} active contacts, ending lookup.",
                    active_contacts.len()
                );
                return Ok(active_contacts);
            }

            if unqueried_contacts.is_empty() || closest_node_seen.is_none() {
                println!("No improvement found or no more unqueried contacts. Ending lookup.");
                break;
            }
        }

        let active_contacts: Vec<_> = shortlist
            .iter()
            .filter(|(_, queried)| *queried)
            .map(|(contact, _)| contact.clone())
            .collect();

        println!(
            "Finished iterative find node. Found {} active contacts.",
            active_contacts.len()
        );
        Ok(active_contacts)
    }

    pub async fn find_value(self, target_id: KademliaID) -> std::io::Result<()> {
        //let target_id = KademliaID::new();
        let adr: String = utils::boot_node_address();
        let boot_node_addr: String = format!("{}:{}", adr, "5678");
        Networking::send_rpc_request(
            &boot_node_addr,
            Command::FINDVALUE,
            Some(target_id),
            None,
            None,
        )
        .await
        .expect("failed");

        println!("ben");
        //Networking::send_rpc_request(target_addr, cmd, data, contact);

        Ok(())
    }

    pub async fn store(self, data: String) -> std::io::Result<()> {
        let mut kad_id = KademliaID::new();
        kad_id.store_data(data.clone()).await;
        println!("Data stored with kademlia id: {}", kad_id.to_hex());

        Ok(())
    }

    pub async fn start_cli(&self) {
        self.cli.read_input().await;
    }
}
