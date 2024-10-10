use std::sync::Arc;

use ::tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};

use crate::{kademlia::{self, Kademlia}, kademlia_id::KademliaID};

enum Command {
    GET(String),
    PUT(String),
    EXIT,
}
#[derive(Clone)]
pub struct Cli {
    kademlia: Arc<Kademlia>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

enum CMDStatus {
    CONTINUE,
    EXIT,
}

impl Cli {
    pub fn new(kademlia: Arc<Kademlia>, shutdown_tx: tokio::sync::broadcast::Sender<()>) -> Self {
        Cli { kademlia, shutdown_tx }
    }

    pub async fn read_input(&self) {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        loop {
            io::stdout().flush().await.unwrap();

            if let Some(line) = reader.next_line().await.unwrap() {
                let input = line.trim().to_lowercase();

                match self.parse_command(&input) {
                    Ok(command) => {
                        if let CMDStatus::EXIT = self.execute_command(command).await {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }

    async fn execute_command(&self, cmd: Command) -> CMDStatus {
        match cmd {
            Command::GET(hash) => {
                //let target_id = KademliaID::from_hex(hash);
                self.kademlia.find_value(KademliaID::new()).await.unwrap();
                CMDStatus::CONTINUE
            }
            Command::PUT(data) => {
                self.kademlia.store(data).await.unwrap();
                CMDStatus::CONTINUE
            }
            Command::EXIT => {
                println!("Exiting...");
                let _ = self.shutdown_tx.send(());
                CMDStatus::EXIT
            }
        }
    }

    fn parse_command(&self, input: &str) -> Result<Command, &'static str> {
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap_or_default();

        match command {
            "get" => {
                if let Some(arg) = parts.next() {
                    Ok(Command::GET(arg.to_string()))
                } else {
                    Err("GET: missing hash argument")
                }
            }
            "put" => {
                if let Some(arg) = parts.next() {
                    Ok(Command::PUT(arg.to_string()))
                } else {
                    Err("PUT: missing data argument")
                }
            }
            "exit" => Ok(Command::EXIT),
            _ => Err("Unknown command"),
        }
    }
}