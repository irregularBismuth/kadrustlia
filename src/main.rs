use {
    axum::{http::StatusCode, routing::get, Json, Router},
    kadrustlia::{
        cli::Cli, constants::rpc::Command, contact::Contact, kademlia::Kademlia,
        kademlia_id::KademliaID, networking::Networking, rpc::RpcMessage, utils,
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
    let bind_addr = "0.0.0.0:5678";
    tokio::spawn(async move {
        Networking::listen_for_rpc(bind_addr)
            .await
            .expect("Failed to listen for PING");
    });

    let kadid = KademliaID::new();
    let hex = kadid.to_hex();
    let kadid2 = KademliaID::from_hex(hex.clone());

    assert_eq!(kadid, kadid2);
    // REST interface ##################
    tokio::spawn(async move {
        let app = Router::new().route("/", get(root));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    //#################################
    Contact::contact_from_hex(hex, "127.0.0.1".to_string());

    #[cfg(not(feature = "local"))]
    {
        println!("addr : {:?}", utils::get_own_address());
    }

    let kademlia_instance = Arc::new(Mutex::new(Kademlia::new()));

    let kademlia_for_join = Arc::clone(&kademlia_instance);
    let kademlia_for_cli = Arc::clone(&kademlia_instance);

    tokio::join!(
        async {
            let mut instance = kademlia_for_join.lock().await;
            instance.join().await;
        },
        async {
            let mut instance = kademlia_for_cli.lock().await;
            instance.start_cli().await;
        }
    );
    Ok(())
}
