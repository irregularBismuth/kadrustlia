use kadrustlia::cli::Cli;

use kadrustlia::contact::Contact;
use kadrustlia::contact::ContactCandidates;
use kadrustlia::kademlia_id::KademliaID;
async fn run() {
    println!("Test");
}

#[tokio::main]
async fn main() {
    let fut = run();
    println!("Hello  world!");
    fut.await;

    let mut candidates = ContactCandidates::new();
    candidates.append(&mut vec![
        Contact::new(KademliaID::new(), "192.168.1.1".to_string()),
        Contact::new(KademliaID::new(), "192.168.2.21".to_string()),
    ]);

    // Compare the contacts at index 0 and 1
    let result = candidates.less(0, 1);
    println!("{}", result);
    let cli = Cli::new();
    cli.read_input().await;
}
