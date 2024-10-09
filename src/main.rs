use {
    axum::{http::StatusCode, routing::get, Json, Router},
    kadrustlia::{
        cli::Cli,
        constants::{rpc::Command, ALL_IPV4},
        contact::Contact,
        kademlia::Kademlia,
        kademlia_id::KademliaID,
        networking::Networking,
        routing_table::RoutingTable,
        rpc::RpcMessage,
        utils,
    },
    std::net::SocketAddr,
    std::sync::Arc,
    tokio::net::ToSocketAddrs,
    tokio::sync::Mutex,
};

async fn root() -> &'static str {
    "Hello world!"
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // REST interface ##################
    tokio::spawn(async move {
        let app = Router::new().route("/", get(root));
        let ip = format!("{}:{}", ALL_IPV4, "3000");
        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    //#################################

    // Create some KademliaIDs
    let kad_id_1 = KademliaID::new();
    let kad_id_2 = KademliaID::new();
    let kad_id_3 = KademliaID::new();

    // Create contacts
    let contact_1 = Contact::new(kad_id_1, "127.0.0.1:8001".to_string());
    let contact_2 = Contact::new(kad_id_2, "127.0.0.2:8002".to_string());
    let contact_3 = Contact::new(kad_id_3, "127.0.0.3".to_string());

    // Create a routing table and add contact_1 as the "self" node
    let mut routing_table = RoutingTable::new(contact_1.clone());

    // Add contact_2 and contact_3 to the routing table
    routing_table.add_contact(contact_2.clone());
    routing_table.add_contact(contact_3.clone());

    // Print the routing table's closest contacts to kad_id_3
    println!("Closest contacts to kad_id_3:");
    let closest_contacts = routing_table.find_closest_contacts(kad_id_3, 2);
    for contact in closest_contacts {
        println!(
            "Contact: {}, Address: {}",
            contact.id.to_hex(),
            contact.address
        );
    }

    // Remove contact_2 from the routing table
    routing_table.remove_contact(kad_id_2);
    println!("Removed contact_2");

    // Print the updated closest contacts after removal
    println!("Closest contacts to kad_id_3 after removal of contact_2:");
    let updated_contacts = routing_table.find_closest_contacts(kad_id_3, 2);
    for contact in updated_contacts {
        println!(
            "Contact: {}, Address: {}",
            contact.id.to_hex(),
            contact.address
        );
    }

    // Now continue with asynchronous tasks
    let bind_addr = format!("{}:{}", ALL_IPV4, "5678");

    let kademlia = Kademlia::new();
    let kademlia_c = kademlia.clone();
    let kademlia_c2 = kademlia.clone();

    let listen_task = tokio::spawn(async move {
        kademlia.listen(&bind_addr).await;
    });

    let join_task = tokio::spawn(async move {
        kademlia_c.join().await;
    });

    let join_task_2 = tokio::spawn(async move {
        kademlia_c2.start_cli().await;
    });

    let _ = tokio::join!(listen_task, join_task, join_task_2);

    Ok(())
}
