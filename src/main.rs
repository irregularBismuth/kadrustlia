use kadrustlia::cli::Cli;
use kadrustlia::kademlia_id::KademliaID;
use kadrustlia::networking::Networking;

#[tokio::main]
async fn main() {
    println!("Hello  world!");

    let kad_id: KademliaID = KademliaID::with_id([0u8; 20]);
    let kad_id_2: KademliaID = KademliaID::with_id([150u8; 20]);
    println!("{}", kad_id.distance(&kad_id_2).to_hex());

    let bind_addr = "0.0.0.0:5678";
    tokio::spawn(async move {
        Networking::listen_for_ping(bind_addr)
            .await
            .expect("Failed to listen for PING");
    });
    /*
        let target_container = "kadrustlia-kademliaNodes-2:5678";

        Networking::send_ping(target_container)
            .await
            .expect("Failed to send PING");
    */
    let cli = Cli::new();
    cli.read_input().await;
}
