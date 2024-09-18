use kadrustlia::cli::Cli;
use kadrustlia::kademlia;
use std::net::SocketAddr;
/*
        let target_container = "kadrustlia-kademliaNodes-2:5678";

        Networking::send_ping(target_container)
            .await
            .expect("Failed to send PING");
*/
use kadrustlia::networking::Networking;
use kadrustlia::rpc::RpcMessage;
use kadrustlia::{
    contact::Contact, contact::ContactCandidates, kademlia_id::KademliaID,
    routing_table::RoutingTable,
};

use kadrustlia::constants::rpc::Command;
use kadrustlia::utils;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let addr: SocketAddr = "[::1]:50051".parse()?;

    let bind_addr = "0.0.0.0:5678";
    tokio::spawn(async move {
        Networking::listen_for_rpc(bind_addr)
            .await
            .expect("Failed to listen for PING");
    });
    /*
        let message = RpcMessage::Request {
            id: 1,
            method: Command::PING,
            params: vec!["Alice".to_string()],
        };
        println!("{:?}", message);
        let data = bincode::serialize(&message).expect("Failed to serialize message");

        println!("{:?}", data);

        let readable: RpcMessage = bincode::deserialize(&data).expect("Failed to deserialize message");

        println!("{:?}", readable);
    */
    /*    let mut candidates = ContactCandidates::new();
    candidates.append(&mut vec![
        Contact::new(KademliaID::new(), "192.168.1.1".to_string()),
        Contact::new(KademliaID::new(), "192.168.2.21".to_string()),
    ]); */
    let ct = Contact::new(KademliaID::new(), "192.168.1.2".to_string());

    let rt = RoutingTable::new(ct);
    let bootNode: bool = utils::check_bn();
    if !bootNode {
        let boot_node_addr: String = utils::boot_node_address();
        println!(" boot node address {}", boot_node_addr);
        let boot_node_addr: String = format!("{}:{}", boot_node_addr, "5678");
        tokio::spawn(async move {
            Networking::send_ping(&boot_node_addr, Command::PING)
                .await
                .expect("failed to send PING");
        });
    }

    let cli = Cli::new();

    cli.read_input().await;

    Ok(())
}
