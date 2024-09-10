use tonic::{transport::Server, Request, Response, Status};
use kadrustlia::kademlia_service_server::{KademliaService, KademliaServiceServer};
use kadrustlia::{LookupRequest, LookupResponse, Contact};
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Default)]
pub struct KademliaContact {
}

#[tonic::async_trait]
impl KademliaService for KademliaContact {
    async fn lookup_contact(
        &self,
        request: Request<LookupRequest>,
    ) -> Result<Response<LookupResponse>, Status> {
        //send request
        //get response
        //call kademlia.lookup_contact

        println!("Request: {:?}", request);

        let response = LookupResponse {};

        Ok(Response::new(response))
    }

    async fn lookup_data();
    async fn store();
}

pub async fn start_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let kademlia_contact = KademliaContact = KademliaContact::default();

    println!("Starting server...");

    Server::builder()
        .add_service(KademliaServiceServer::new(kademlia_contact))
        .serve(addr)
        .await?;

    Ok(())
}