use kadrustlia::cli::Cli;

use kadrustlia::contact::Contact;
use kadrustlia::kademlia_id::KademliaID;
async fn run() {
    println!("Test");
}

#[tokio::main]
async fn main() {
    let fut = run();
    println!("Hello  world!");
    fut.await;

    let kad_id: KademliaID = KademliaID::new();

    let mut kad_id = KademliaID::new();
    let kad_id2 = KademliaID::new();
    let mut contact = Contact::new(kad_id, "2123".to_string());
    contact.calc_distance(&kad_id2);
    println!("Contact distance to target: {}", contact.distance);
    println!("{}", kad_id.to_hex());
    println!("{}", kad_id.store_data("test".to_string()).to_hex());
    let cli = Cli::new();
    cli.read_input().await;
}
