use {
    axum::{routing::get, Router},
    kadrustlia::{cli::Cli, constants::ALL_IPV4, kademlia::Kademlia},
    std::sync::Arc,
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

    let kademlia = Arc::new(Kademlia::new());

    let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);

    let kademlia_listen = Arc::clone(&kademlia);
    let mut shutdown_rx = shutdown_tx.subscribe();
    let listen_task = tokio::spawn(async move {
        tokio::select! {
            _ = kademlia_listen.listen(&bind_addr) => {},
            _ = shutdown_rx.recv() => {
                println!("shutting down listen task...");
            },
        }
    });

    let kademlia_join = Arc::clone(&kademlia);
    let join_task = tokio::spawn(async move {
        if let Err(e) = kademlia_join.join().await {
            eprintln!("Error during join: {}", e);
        }
    });

    let kademlia_cli = Arc::clone(&kademlia);
    let cli = Cli::new(kademlia_cli, shutdown_tx.clone());
    let cli_task = tokio::spawn(async move {
        cli.read_input().await;
    });

    let _ = tokio::join!(listen_task, join_task, cli_task);
    Ok(())
}
