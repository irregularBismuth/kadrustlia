async fn run() {
    println!(
        " 
        test from kademlia;
    "
    );
}

#[tokio::main]
async fn main() {
    let fut = run();
    println!("Hello  world!");
    fut.await;
    loop {}
}
