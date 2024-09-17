use tonic::transport::Channel;
use proto::kademlia_client::KademliaClient;
use proto::LookupContactRequest;
use proto::LookupDataRequest;
use proto::StoreRequest;

pub mod proto {
    tonic::include_proto!("kademlia");
}

pub struct Client {
    client: KademliaClient<Channel>,
}

impl Client {
    pub async fn new(server_addr: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = KademliaClient::connect(server_addr).await?;
        Ok(Client { client })
    }

    pub async fn lookup_contact(&mut self, contact_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(LookupContactRequest {
            contact_id,
        });

        let response = self.client.lookup_contact(request).await?;
        println!("LookupContact response: {:?}", response);

        Ok(())
    }

    pub async fn lookup_data(&mut self, hash: String) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(LookupDataRequest {
            hash,
        });

        let response = self.client.lookup_data(request).await?;
        println!("LookupData response: {:?}", response);

        Ok(())
    }

    pub async fn store(&mut self, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let request = tonic::Request::new(StoreRequest {
            data,
        });

        let response = self.client.store(request).await?;
        println!("Store response: {:?}", response);

        Ok(())
    }
}
