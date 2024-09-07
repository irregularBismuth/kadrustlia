use crate::constants::ID_LENGTH;
use rand::Rng;

pub struct KademliaID {
    pub id: [u8; ID_LENGTH],
}

impl KademliaID {
    pub fn new() -> Self {
        let mut id = [0u8; ID_LENGTH];
        rand::thread_rng().fill(&mut id[..]);
        Self { id }
    }

    pub fn to_hex(&self) -> String {
        self.id.iter().map(|byte| format!("{:02x}", byte)).collect()
    }

    pub fn distance(&self, other: KademliaID) -> usize {
        self.id
            .iter()
            .zip(other.id.iter())
            .map(|(a, b)| (a ^ b) as usize)
            .sum()
    }
}
