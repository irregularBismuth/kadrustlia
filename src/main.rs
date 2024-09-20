use {
    axum::{http::StatusCode, routing::get, Json, Router},
    kadrustlia::{
        cli::Cli,
        constants::{rpc::Command, ALL_IPV4},
        contact::Contact,
        kademlia::Kademlia,
        kademlia_id::KademliaID,
        networking::Networking,
        rpc::RpcMessage,
        utils,
    },
    std::net::SocketAddr,
    std::sync::Arc,
    tokio::net::ToSocketAddrs,
    tokio::sync::Mutex,
};

async fn root() -> &'static str {
    "Hello world!"
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // REST interface ##################
    tokio::spawn(async move {
        let app = Router::new().route("/", get(root));
        let ip = format!("{}:{}", ALL_IPV4, "3000");
        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    //#################################

    let bind_addr = format!("{}:{}", ALL_IPV4, "5678");
    /*
    tokio::spawn(async move {
        Networking::listen_for_rpc(&bind_addr)
            .await
            .expect("Failed to listen for PING");
    }); */

    let kadid = KademliaID::new();
    let hex = kadid.to_hex();
    let kadid2 = KademliaID::from_hex(hex.clone());

    assert_eq!(kadid, kadid2);

    let kademlia = Kademlia::new();

    let kademlia_c = kademlia.clone();
    let listen_task = tokio::spawn(async move {
        kademlia.listen(&bind_addr).await;
    });
    let join_task = tokio::spawn(async move {
        kademlia_c.join().await;
    });
    let _ = tokio::join!(listen_task, join_task);
    Ok(())
}
