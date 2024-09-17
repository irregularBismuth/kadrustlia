use kadrustlia::cli::Cli;
use kadrustlia::client::Client;
use kadrustlia::kademlia;
use std::net::SocketAddr;
/*
        let target_container = "kadrustlia-kademliaNodes-2:5678";

        Networking::send_ping(target_container)
            .await
            .expect("Failed to send PING");
*/
use kadrustlia::networking::Networking;
use kadrustlia::{
    contact::Contact, contact::ContactCandidates, kademlia_id::KademliaID,
    routing_table::RoutingTable,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let addr: SocketAddr = "[::1]:50051".parse()?;
    let addr: SocketAddr = "0.0.0.0:50051".parse()?;

    tokio::spawn(async move {
        kademlia::start_server(&addr).await.unwrap();
    });

    println!("Server started on {}", addr);
    let bind_addr = "0.0.0.0:5678";
    tokio::spawn(async move {
        Networking::listen_for_ping(bind_addr)
            .await
            .expect("Failed to listen for PING");
    });

    /*    let mut candidates = ContactCandidates::new();
    candidates.append(&mut vec![
        Contact::new(KademliaID::new(), "192.168.1.1".to_string()),
        Contact::new(KademliaID::new(), "192.168.2.21".to_string()),
    ]); */
    let ct = Contact::new(KademliaID::new(), "192.168.1.2".to_string());

    // let client_url = format!("http://{}", addr);
    let client_url = format!("http://bootNode:50051");
    let mut client = Client::new(client_url).await?;

    let rt = RoutingTable::new(ct);
    //let result = candidates.less(0, 1);
    //println!("{}", result);

    let cli = Cli::new();

    cli.read_input(&mut client).await;

    Ok(())
}

