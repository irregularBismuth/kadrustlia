use kadrustlia::test::call_func;

#[tokio::main]
async fn main() {
    call_func().await;
    println!("Hello, world!");
}
