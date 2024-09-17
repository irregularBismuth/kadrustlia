use std::net::SocketAddr;
use kadrustlia::kademlia;
use kadrustlia::client::Client;
use kadrustlia::cli::Cli;


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