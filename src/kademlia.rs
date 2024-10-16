use {
    crate::{
        constants::{rpc::Command, ALPHA, BUCKET_SIZE},
        contact::Contact,
        kademlia_id::KademliaID,
        networking::Networking,
        routing_table::RoutingTable,
        routing_table_handler::*,
        rpc::RpcMessage,
        utils,
    },
    tokio::sync::mpsc,
};

#[derive(Clone)]
pub struct Kademlia {
    pub route_table_tx: mpsc::Sender<RouteTableCMD>,
    pub own_id: KademliaID,
    pub networking: Networking,
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

        let networking = Networking::new();

        Self {
            route_table_tx: tx,
            own_id: kad_id,
            networking,
        }
    }

    pub async fn listen(&self, addr: &str) {
        let tx = self.route_table_tx.clone();
        let _ = self.networking.listen_for_rpc(tx, addr).await;
    }

    pub async fn join(&self) -> std::io::Result<()> {
        if utils::check_bn() {
            return Ok(());
        }
        let adr: String = utils::boot_node_address();
        let mut boot_node_addr: String;
        #[cfg(not(feature = "local"))]
        {
            let adr = "127.0.0.1".to_string();
            boot_node_addr = format!("{}:{}", adr, "5678");
        }
        #[cfg(feature = "local")]
        {
            boot_node_addr = format!("{}:{}", adr, "5678");
        }

        println!("Boot node address: {}", boot_node_addr);

        let own_contact = Contact::new(self.own_id.clone(), utils::get_own_address());

        self.networking
            .send_rpc_request_await(
                KademliaID::new(),
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
            ()
        }

        // print contacts [DONT DELETE]
        /*let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);

        let _ = self
            .route_table_tx
            .send(RouteTableCMD::GetClosestNodes(
                self.own_id.clone(),
                reply_tx,
            ))
            .await;

        if let Some(contacts) = reply_rx.recv().await {
            println!("{:?}", contacts);
        } else {
            println!("no conacts from routing table");
        }*/

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
            .send(RouteTableCMD::GetClosestNodes(target_id.clone(), reply_tx))
            .await;

        if let Some(initial_contacts) = reply_rx.recv().await {
            for contact in initial_contacts.into_iter().take(BUCKET_SIZE) {
                shortlist.push((contact, false));
            }
        }

        let mut closest_node_seen = None;
        let mut closest_distance = self.own_id.distance(&target_id);

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
                let networking_clone = self.networking.clone();
                let contact_clone = contact.clone();
                let rpc_id = KademliaID::new();

                let task = tokio::spawn(async move {
                    let response = networking_clone
                        .send_rpc_request_await(
                            rpc_id,
                            &target_addr,
                            Command::FINDNODE,
                            Some(target_id_copy),
                            None,
                            None,
                        )
                        .await;
                    (response, contact_clone)
                });

                tasks.push(task);
            }

            for task in tasks {
                match task.await {
                    Ok((
                        Ok(Some(RpcMessage::Response {
                            contact: Some(received_contacts),
                            ..
                        })),
                        queried_contact,
                    )) => {
                        println!(
                            "Received response from contact: {}",
                            queried_contact.id.to_hex()
                        );

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
                    Ok((Ok(Some(_)), queried_contact)) => {
                        println!(
                            "Received response from contact: {} but no contacts",
                            queried_contact.id.to_hex()
                        );
                    }
                    Ok((Ok(None), queried_contact)) => {
                        println!(
                            "No response from contact: {} within timeout. Marking as unreachable.",
                            queried_contact.id.to_hex()
                        );
                        shortlist.retain(|(contact, _)| contact.id != queried_contact.id);
                    }
                    Ok((Err(e), queried_contact)) => {
                        println!(
                            "Failed to send request to contact: {}. Error: {}. Marking as unreachable.",
                            queried_contact.id.to_hex(),
                            e
                        );
                        shortlist.retain(|(contact, _)| contact.id != queried_contact.id);
                    }
                    Err(e) => {
                        println!("Task failed with error: {}", e);
                    }
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
                /*println!(
                    "Found {} active contacts, ending lookup.",
                    active_contacts.len()
                );*/
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

        active_contacts.sort_by(|a, b| a.get_distance().cmp(&b.get_distance()));

        /*println!(
            "Finished iterative find node. Found {} active contacts.",
            active_contacts.len()
        );*/
        Ok(active_contacts)
    }

    pub async fn iterative_find_value(
        &self,
        target_id: KademliaID,
    ) -> std::io::Result<Option<String>> {
        println!(
            "Starting iterative find value for target ID: {}",
            target_id.to_hex()
        );

        let mut shortlist: Vec<(Contact, bool)> = Vec::new();

        let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);
        let _ = self
            .route_table_tx
            .send(RouteTableCMD::GetClosestNodes(target_id.clone(), reply_tx))
            .await;

        if let Some(initial_contacts) = reply_rx.recv().await {
            for contact in initial_contacts.into_iter().take(BUCKET_SIZE) {
                shortlist.push((contact, false));
            }
        }

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
                let networking_clone = self.networking.clone();
                let contact_clone = contact.clone();
                let rpc_id = KademliaID::new();

                let task = tokio::spawn(async move {
                    let response = networking_clone
                        .send_rpc_request_await(
                            rpc_id,
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

            for task in tasks {
                match task.await {
                    Ok((
                        Ok(Some(RpcMessage::Response {
                            data: Some(value), ..
                        })),
                        queried_contact,
                    )) => {
                        println!(
                            "Value found: {} from node {}",
                            value,
                            queried_contact.id.to_hex(),
                        );
                        return Ok(Some(value));
                    }
                    Ok((
                        Ok(Some(RpcMessage::Response {
                            contact: Some(received_contacts),
                            ..
                        })),
                        queried_contact,
                    )) => {
                        println!(
                            "Received contacts from contact: {}",
                            queried_contact.id.to_hex()
                        );

                        for new_contact in received_contacts {
                            if !shortlist.iter().any(|(c, _)| c.id == new_contact.id) {
                                println!(
                                    "Adding new contact: {} to shortlist",
                                    new_contact.id.to_hex()
                                );
                                shortlist.push((new_contact, false));
                            }
                        }
                    }
                    Ok((Ok(Some(_)), queried_contact)) => {
                        println!(
                            "Received response from contact: {} but no data or contacts",
                            queried_contact.id.to_hex()
                        );
                    }
                    Ok((Ok(None), queried_contact)) => {
                        println!(
                            "No response from contact: {} within timeout. Marking as unreachable.",
                            queried_contact.id.to_hex()
                        );
                        shortlist.retain(|(contact, _)| contact.id != queried_contact.id);
                    }
                    Ok((Err(e), queried_contact)) => {
                        println!(
                            "Failed to send request to contact: {}. Error: {}. Marking as unreachable.",
                            queried_contact.id.to_hex(),
                            e
                        );
                        shortlist.retain(|(contact, _)| contact.id != queried_contact.id);
                    }
                    Err(e) => {
                        println!("Task failed with error: {}", e);
                    }
                }
            }

            for contact in &mut shortlist {
                if unqueried_contacts.iter().any(|c| c.id == contact.0.id) {
                    contact.1 = true;
                }
            }

            if unqueried_contacts.is_empty() {
                println!("No more unqueried contacts and value not found. Ending lookup.");
                break;
            }
        }

        println!("Value not found in the network.");
        Ok(None)
    }

    pub async fn iterative_store(
        &self,
        target_id: KademliaID,
        data: String,
    ) -> std::io::Result<()> {
        println!(
            "Starting iterative store for target ID: {}",
            target_id.to_hex()
        );

        let closest_nodes = self.iterative_find_node(target_id.clone()).await?;

        if closest_nodes.is_empty() {
            println!("No contacts found to store data.");
            return Ok(());
        }

        for contact in closest_nodes {
            let target_addr = format!("{}:{}", contact.address, "5678");
            println!(
                "Storing data at contact: {} ({})",
                contact.id.to_hex(),
                target_addr
            );

            let store_result = self
                .networking
                .send_rpc_request_await(
                    KademliaID::new(),
                    &target_addr,
                    Command::STORE,
                    Some(target_id.clone()),
                    Some(data.clone()),
                    None,
                )
                .await;

            match store_result {
                Ok(Some(_)) => println!("Successfully stored data at {}", contact.id.to_hex()),
                Ok(None) => println!("No response from {}", contact.id.to_hex()),
                Err(e) => println!("Failed to store data at {}: {}", contact.id.to_hex(), e),
            }
        }

        Ok(())
    }
}
