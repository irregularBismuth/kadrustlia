use kadrustlia::test::call_func;

#[tokio::main]
async fn main() {
    let future = call_func();
    println!("Hello, world!");
    future.await;
}

