pub mod cli;
pub mod constants;
pub mod contact;
pub mod kademlia;
pub mod kademlia_id;
pub mod tests;
pub mod server;
pub mod client;

pub mod proto {
    tonic::include_proto!("kademlia");
}