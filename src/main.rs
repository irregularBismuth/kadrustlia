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
    println!("{:?}", kad_id.id);

    println!("{:?}", kad_id.to_hex());

    let kad_id2: KademliaID = KademliaID::new();
    println!("{:?}", kad_id2.to_hex());
    println!("xor distance {:?}", kad_id.distance(&kad_id));
    println!("less {}", kad_id.less(&kad_id2));
    let cli = Cli::new();
    cli.read_input().await;
}
