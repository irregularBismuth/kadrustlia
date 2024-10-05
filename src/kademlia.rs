use {
    crate::{
        cli::Cli, constants::rpc::Command, contact::Contact, kademlia_id::KademliaID,
        networking::Networking, routing_table::RoutingTable, utils,
    },
    tokio::sync::mpsc,
};

pub enum RouteTableCMD {
    AddContact(Contact),
    RemoveContact(KademliaID),
    GetClosestNodes(KademliaID),
}

async fn routing_table_handler(
    mut rx: mpsc::Receiver<RouteTableCMD>,
    mut routing_table: RoutingTable,
) {
    println!("route table handler");
    while let Some(cmd) = rx.recv().await {
        match cmd {
            RouteTableCMD::AddContact(contact) => {
                println!("ping  hello ");
            }
            RouteTableCMD::RemoveContact(kad_id) => {
                println!("remove  coibntact");
            }
            RouteTableCMD::GetClosestNodes(kad_id) => {
                println!("kademlia we got {}", kad_id.to_hex());
            }
        }
    }
}

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

        Networking::send_rpc_request(&boot_node_addr, Command::PING, None, None)
            .await
            .expect("failed to send PING");
    }

    pub async fn find_node(self, target_id: KademliaID) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn find_value(self, target_id: KademliaID) -> std::io::Result<()> {
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
