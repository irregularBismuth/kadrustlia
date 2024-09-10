use proto::kademlia_service_client::KademliaServiceClient;
use proto::LookupRequest;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = KademliaServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(LookupRequest {
        contact_id: "BENISGOOOOOODDD".to_vec(),
    });

    let response = client.lookup_contact(request).await?;

    println!("Response: {:?}", response.into_inner());

    Ok(())
}
