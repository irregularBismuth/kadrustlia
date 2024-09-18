use {
    axum::{http::StatusCode, routing::get, Json, Router},
    kadrustlia::{
        cli::Cli, constants::rpc::Command, kademlia::Kademlia, networking::Networking,
        rpc::RpcMessage, utils,
    },
    std::net::SocketAddr,
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

    // REST interface ##################
    tokio::spawn(async move {
        let app = Router::new().route("/", get(root));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
    //#################################

    let cli = Cli::new();
    let mut kademlia_instance: Kademlia = Kademlia::new();
    kademlia_instance.join().await;
    cli.read_input().await;

    Ok(())
}
