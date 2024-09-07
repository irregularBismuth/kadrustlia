use kadrustlia::cli::Cli;

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
    let kad_id2 = KademliaID::new().store_data("test".to_string()).to_hex();

    println!("{}", kad_id.to_hex());
    println!("{}", kad_id.store_data("test".to_string()).to_hex());
    println!("{}", kad_id2);
    let cli = Cli::new();
    cli.read_input().await;
}
