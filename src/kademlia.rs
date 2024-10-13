use {
    crate::{
        constants::{rpc::Command, ALPHA, BUCKET_SIZE, RT_BCKT_SIZE},
        contact::Contact,
        kademlia_id::KademliaID,
        networking::Networking,
        routing_table::RoutingTable,
        routing_table_handler::*,
        rpc::RpcMessage,
        utils,
    },
    std::collections::{HashMap, HashSet},
    tokio::{
        sync::mpsc::{self, Receiver, Sender},
        task,
    },
};

#[derive(Clone)]
pub struct Kademlia {
    route_table_tx: mpsc::Sender<RouteTableCMD>,
    own_id: KademliaID,
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
            own_id: kad_id,
        }
    }

    pub async fn listen(&self, addr: &str) {
        let tx = self.route_table_tx.clone();
        let _ = Networking::listen_for_rpc(tx, addr).await;
    }

    pub async fn join(&self) -> std::io::Result<()> {
        if utils::check_bn() {
            return Ok(());
        }
        let adr: String = utils::boot_node_address();
        let boot_node_addr: String = format!("{}:{}", adr, "5678");
        println!("Boot node address: {}", boot_node_addr);

        let own_contact = Contact::new(self.own_id.clone(), utils::get_own_address());

        Networking::send_rpc_request(
            self.own_id.clone(),
            &boot_node_addr,
            Command::PING,
            None,
            None,
            Some(vec![own_contact]),
        )
        .await
        .expect("Failed to send PING");

        let contacts = self.iterative_find_node(self.own_id.clone()).await?;

        if contacts.is_empty() {
            println!("No contacts found during iterative find node.");
            return Ok(());
        }

        let closest_neighbor = contacts[0].clone();

        let (index_tx, mut index_rx) = mpsc::channel(1);
        self.route_table_tx
            .send(RouteTableCMD::GetBucketIndex(
                closest_neighbor.id.clone(),
                index_tx,
            ))
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let bucket_index = index_rx.recv().await.expect("Failed to get bucket index");

        for i in (bucket_index + 1)..RT_BCKT_SIZE {
            let random_id = self.own_id.generate_random_id_in_bucket(i);
            let _ = self.iterative_find_node(random_id).await?;
        }

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
            for contact in initial_contacts.into_iter().take(BUCKET_SIZE) {
                shortlist.push((contact, false));
            }
        }

        let mut closest_node_seen = None;
        let mut closest_distance = KademliaID::new().distance(&target_id);

        while !shortlist.is_empty() {
            let unqueried_contacts: Vec<Contact> = shortlist
                .iter()
                .filter(|(_, queried)| !queried)
                .take(BUCKET_SIZE)
                .map(|(contact, _)| contact.clone())
                .collect();

            if unqueried_contacts.is_empty() {
                println!("No more unqueried contacts left in shortlist. Ending lookup.");
                break;
            }

            let mut tasks = vec![];
            for contact in unqueried_contacts.iter().take(ALPHA) {
                println!("Querying contact: {}", contact.id.to_hex());
                let target_addr = format!("{}:{}", contact.address, "5678");
                let target_id_copy = target_id.clone();
                let own_id_copy = self.own_id.clone();
                let task = tokio::spawn(async move {
                    Networking::send_rpc_request(
                        own_id_copy,
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

        let mut active_contacts: Vec<_> = shortlist
            .iter()
            .filter(|(_, queried)| *queried)
            .map(|(contact, _)| {
                let mut c = contact.clone();
                c.calc_distance(&target_id);
                c
            })
            .collect();

        // Sort the active contacts by distance
        active_contacts.sort_by(|a, b| a.get_distance().cmp(&b.get_distance()));

        println!(
            "Finished iterative find node. Found {} active contacts.",
            active_contacts.len()
        );
        Ok(active_contacts)
    }

    // pub async fn find_value(&self, target_id: KademliaID) -> std::io::Result<()> {
    //     println!(
    //         "Starting iterative find value for target ID: {}",
    //         target_id.to_hex()
    //     );

    //     let mut shortlist: Vec<(Contact, bool)> = Vec::new();

    //     let adr: String = utils::boot_node_address();
    //     let boot_node_addr: String = format!("{}:{}", adr, "5678");
    //     let own_id_copy = self.own_id.clone();
    //     Networking::send_rpc_request(
    //         own_id_copy,
    //         &boot_node_addr,
    //         Command::FINDVALUE,
    //         Some(target_id),
    //         None,
    //         None,
    //     )
    //     .await
    //     .expect("failed");

    //     println!("ben");
    //     //Networking::send_rpc_request(target_addr, cmd, data, contact);

    //     println!("Value found or contacts returned");
    //     Ok(())
    // }

    pub async fn find_value(
        &self,
        target_id: KademliaID,
    ) -> std::io::Result<Option<(String, Contact)>> {
        println!(
            "Starting iterative find value for target ID: {}",
            target_id.to_hex()
        );

        let mut shortlist: Vec<(Contact, bool)> = Vec::new();

        // Get initial contacts from the routing table
        let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);
        let _ = self
            .route_table_tx
            .send(RouteTableCMD::GetClosestNodes(target_id, reply_tx))
            .await;

        if let Some(initial_contacts) = reply_rx.recv().await {
            for contact in initial_contacts.into_iter().take(BUCKET_SIZE) {
                shortlist.push((contact, false));
            }
        }

        let queried_nodes: HashSet<KademliaID> = HashSet::new();

        while !shortlist.is_empty() {
            let unqueried_contacts: Vec<Contact> = shortlist
                .iter()
                .filter(|(contact, queried)| !*queried && !queried_nodes.contains(&contact.id))
                .take(BUCKET_SIZE)
                .map(|(contact, _)| contact.clone())
                .collect();

            if unqueried_contacts.is_empty() {
                println!("No more unqueried contacts left in shortlist. Ending lookup.");
                break;
            }

            let mut tasks = vec![];
            for contact in unqueried_contacts.iter().take(ALPHA) {
                println!("Querying contact: {}", contact.id.to_hex());
                let target_addr = format!("{}:{}", contact.address, "5678");
                let target_id_copy = target_id.clone();
                let own_id_copy = self.own_id.clone();
                let contact_clone = contact.clone();
                let task = tokio::spawn(async move {
                    let response = Networking::send_rpc_request_and_await_response(
                        own_id_copy,
                        &target_addr,
                        Command::FINDVALUE,
                        Some(target_id_copy),
                        None,
                        None,
                    )
                    .await;
                    (response, contact_clone)
                });
                tasks.push(task);
            }

            let mut found_value = None;

            for task in tasks {
                if let Ok((Ok(response), contact)) = task.await {
                    match response {
                        RpcMessage::Response {
                            id: _,
                            result,
                            data,
                            contact: contacts,
                        } => {
                            if result == Command::FINDVALUE {
                                if let Some(value) = data {
                                    // Value found
                                    println!("Value found at node {}", contact.id.to_hex());
                                    found_value = Some((value, contact.clone()));
                                    break;
                                } else if let Some(new_contacts) = contacts {
                                    // No value, but received contacts
                                    for new_contact in new_contacts {
                                        if !shortlist.iter().any(|(c, _)| c.id == new_contact.id) {
                                            shortlist.push((new_contact, false));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("Unexpected response {:?}", response);
                        }
                    }
                } else {
                    println!("Failed to receive response");
                }
            }

            if let Some((value, contact)) = found_value {
                return Ok(Some((value, contact)));
            }

            for contact in &mut shortlist {
                if unqueried_contacts.iter().any(|c| c.id == contact.0.id) {
                    contact.1 = true;
                }
            }
        }

        println!("Value not found");
        Ok(None)
    }

    pub async fn store(&self, data: String) -> std::io::Result<()> {
        let mut kad_id = KademliaID::new();
        kad_id.store_data(data.clone()).await;
        println!("Data stored with kademlia id: {}", kad_id.to_hex());
        Ok(())
    }
}
