use ::tokio::io::{self, AsyncBufReadExt};
enum Command {
    GET,
    PUT,
    DELETE,
}

pub struct Cli {}

impl Cli {
    pub fn new() -> Self {
        Cli {}
    }

    pub async fn read_input(&self) {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();
        while let Some(line) = reader.next_line().await.unwrap() {
            let input = line.trim().to_lowercase();
            let command = self.parse_command(&input);
            self.execute_command(command);
        }
    }

    fn execute_command(&self, cmd: Command) {
        match cmd {
            Command::GET => Self::get(),
            Command::PUT => Self::put(),
            Command::DELETE => Self::delete(),
        }
    }

    fn parse_command(&self, input: &str) -> Command {
        match input {
            "get" => Command::GET,
            "put" => Command::PUT,
            "delete" => Command::DELETE,
            _ => {
                panic!("command not found ");
            }
        }
    }

    fn get() {
        println!("Executing GET command...");
    }

    fn put() {
        println!("Executing PUT command...");
    }

    fn delete() {
        println!("Executing DELETE command...");
    }
}
