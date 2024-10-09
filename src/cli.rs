use ::tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};

use crate::{kademlia::{self, Kademlia}, kademlia_id::KademliaID};

enum Command {
    GET(String),
    PUT(String),
    EXIT,
}
#[derive(Clone)]
pub struct Cli {}

impl Cli {
    pub fn new() -> Self {
        Cli {}
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
                        if let Command::EXIT = command {
                            println!("bombaclat node");
                            break;
                        }
                        self.execute_command(command).await;
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }

    async fn execute_command(&self, cmd: Command) {
        match cmd {
            Command::GET(hash) => {
                //let target_id = KademliaID::from_hex(hash);
                let kademlia = Kademlia::new();
                kademlia.find_value(KademliaID::new()).await.unwrap();
            }
            Command::PUT(data) => {
                let kademlia = Kademlia::new();
                kademlia.store(data).await.unwrap();

                // let data = data.as_bytes().to_vec();
                //client.store(data).await.unwrap();
            }
            Command::EXIT => {
                println!("Exiting...");
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