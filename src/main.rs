use std::vec;

use kadrustlia::cli::Cli;

use kadrustlia::contact::Contact;
use kadrustlia::kademlia_id::KademliaID;
use kadrustlia::{client, server};

/*async fn run() {
    println!("Test");
}*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    tokio::spawn(async move {
        server::start_server(&addr.to_string()).await.unwrap();
    })
    .await??;

    Ok(())

    /*let fut = run();
    println!("Hello  world!");
    fut.await;

    let kad_id: KademliaID = KademliaID::with_id([0u8; 20]);
    let kad_id_2: KademliaID = KademliaID::with_id([150u8; 20]);
    println!("{}", kad_id.distance(&kad_id_2).to_hex());

    let cli = Cli::new();
    cli.read_input().await;*/
}
