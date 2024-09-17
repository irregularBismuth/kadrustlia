use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response, Status};
use proto::kademlia_server::{Kademlia, KademliaServer};
use proto::{LookupContactRequest, LookupContactResponse, LookupDataRequest, LookupDataResponse, StoreRequest, StoreResponse, Node};

pub mod proto {
    tonic::include_proto!("kademlia");
}

#[derive(Debug, Default)]
pub struct KademliaService;

#[tonic::async_trait]
impl Kademlia for KademliaService {
    async fn lookup_contact(
        &self,
        request: Request<LookupContactRequest>,
    ) -> Result<Response<LookupContactResponse>, Status> {
        let req = request.into_inner();
        println!("LookupContact request received for contact_id={}", req.contact_id);

        let nodes = vec![
            Node {
                node_id: "123".to_string(),
                address: "192.168.1.1:5678".to_string(),
            },
            Node {
                node_id: "456".to_string(),
                address: "192.168.1.2:5678".to_string(),
            },
        ];

        let reply = LookupContactResponse { nodes };
        Ok(Response::new(reply))
    }

    async fn lookup_data(
        &self,
        request: Request<LookupDataRequest>,
    ) -> Result<Response<LookupDataResponse>, Status> {
        let req = request.into_inner();
        println!("LookupData request received for hash={}", req.hash);

        let data = "data".to_string(); 

        let reply = LookupDataResponse { data };
        Ok(Response::new(reply))
    }

    async fn store(
        &self,
        request: Request<StoreRequest>,
    ) -> Result<Response<StoreResponse>, Status> {
        let req = request.into_inner();
        println!("Store request received with data size={}", req.data.len());

        let success = true;

        let reply = StoreResponse { success };
        Ok(Response::new(reply))
    }
}

pub async fn start_server(addr: &SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let kad = KademliaService::default();

    println!("Starting server...");

    Server::builder()
        .add_service(KademliaServer::new(kad))
        .serve(*addr)
        .await?;

    Ok(())
}
