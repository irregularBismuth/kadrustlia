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
    futures::future::join_all,
    tokio::sync::mpsc,
};

/*pub enum RouteTableCMD {
    AddContact(Contact),
    RemoveContact(KademliaID),
    GetClosestNodes(KademliaID, mpsc::Sender<Vec<Contact>>),
}

async fn routing_table_handler(
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
                println!("target_id: {:?}", target_id);
                println!("contacts: {:?}", contacts);
                let _ = reply.send(contacts).await;
            }
        }
    }
}*/

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
        let mut shortlist = Vec::new();
        let mut queried_nodes = Vec::new();
        let mut closest_node: Option<Contact> = None;
        let mut current_alpha_contacts: Vec<Contact>;

        // Step 1: Get the initial ALPHA closest contacts from the routing table
        let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);
        let _ = self
            .route_table_tx
            .send(RouteTableCMD::GetClosestNodes(target_id.clone(), reply_tx))
            .await;

        if let Some(contacts) = reply_rx.recv().await {
            shortlist = contacts;
        }

        loop {
            // Step 2: Select ALPHA contacts from the shortlist
            current_alpha_contacts = shortlist
                .iter()
                .filter(|contact| !queried_nodes.contains(&contact.id)) // exclude already queried
                .take(ALPHA)
                .cloned()
                .collect();

            if current_alpha_contacts.is_empty() {
                break;
            }

            // Mark these nodes as queried
            queried_nodes.extend(current_alpha_contacts.iter().map(|c| c.id.clone()));

            // Step 3: Send FIND_NODE RPCs in parallel
            let mut tasks = vec![];
            for contact in current_alpha_contacts {
                let target_addr = format!("{}:5678", contact.address); // `target_addr` is created here.
                let target_id = target_id.clone(); // Clone `target_id` to avoid ownership issues.

                // Move `target_addr` into the async block so it lives as long as the task
                let rpc_task = tokio::spawn(async move {
                    Networking::send_rpc_request(
                        &target_addr, // Ownership is moved here
                        Command::FINDNODE,
                        Some(target_id), // Cloned `target_id` is moved here
                        None,
                        None,
                    )
                    .await
                });

                tasks.push(rpc_task);
            }
            let _ = futures::future::join_all(tasks).await;

            // Step 4: Update shortlist with closer nodes
            if let Some(contacts) = reply_rx.recv().await {
                for contact in contacts {
                    // Calculate distance and check if it's closer than closest_node
                    if let Some(ref closest) = closest_node {
                        if contact.less(closest.clone()) {
                            closest_node = Some(contact.clone());
                        }
                    } else {
                        closest_node = Some(contact.clone());
                    }
                    // Add contact to shortlist
                    if !shortlist.iter().any(|c| c.id == contact.id) {
                        shortlist.push(contact);
                    }
                }
            }

            // Stop if no new closest node is found
            if let Some(closest) = closest_node.as_ref() {
                if queried_nodes.contains(&closest.id) {
                    break;
                }
            }
        }

        Ok(())
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

/*
+-----------------+                   +-----------------+
|                 |                   |                 |
|     My Node     |                   |   Other Node    |
|                 |                   |                 |
+-----------------+                   +-----------------+
        |                                       |
        | find_node(target_id)                  |
        |-------------------------------------->|
        |                               listen_for_rpc()
        |                               Processes FIND_NODE request
        |                               Accesses routing table
        |<--------------------------------------|
        | Receives response with contacts       |
        | Processes response                    |

*/
