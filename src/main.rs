use std::net::SocketAddr;
use kadrustlia::kademlia;
use kadrustlia::client::Client;
use kadrustlia::cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let addr: SocketAddr = "[::1]:50051".parse()?;
    let addr: SocketAddr = "0.0.0.0:50051".parse()?;

    tokio::spawn(async move {
        kademlia::start_server(&addr).await.unwrap();
    });

    println!("Server started on {}", addr);

    let client_url = format!("http://bootNode:50051");
    let mut client = Client::new(client_url).await?;

    let cli = Cli::new();

    cli.read_input(&mut client).await;

    Ok(())
}