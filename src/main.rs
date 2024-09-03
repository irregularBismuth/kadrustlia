use kadrustlia::cli::Cli;
async fn run() {
    println!("Test");
}

#[tokio::main]
async fn main() {
    let fut = run();
    println!("Hello  world!");
    fut.await;

    let cli = Cli::new();
    cli.read_input().await;
}
