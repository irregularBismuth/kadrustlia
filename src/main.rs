use kadrustlia::test::call_func;

#[tokio::main]
async fn main() {
    call_func().await;
    println!("Hello, world!");
    loop {}
}

// docker build -t kademlia .
// docker compose up --build -d
// docker ps -a
// docker exec -it kadrustlia-node-1 /bin/sh
// ping kadrustlia-node-2
