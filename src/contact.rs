struct Contact {
    address: String,
}

impl Contact {
    pub fn new() -> Self {
        Self {
            address: "".to_string(),
        }
    }
}
