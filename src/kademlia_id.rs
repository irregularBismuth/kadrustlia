use crate::constants::ID_LENGTH;
use rand::Rng;
use sha2::{Digest, Sha256};

type KadId = [u8; ID_LENGTH];

#[derive(Clone, Copy)]
pub struct KademliaID {
    pub id: KadId,
}

impl KademliaID {
    pub fn new() -> Self {
        let mut id = [0u8; ID_LENGTH];
        rand::thread_rng().fill(&mut id[..]);
        Self { id }
    }

    pub fn store_data(&mut self, data: String) -> Self {
        let hash = Sha256::digest(data.as_bytes());
        self.id.copy_from_slice(&hash[..ID_LENGTH]);
        *self
    }

    pub fn to_hex(&self) -> String {
        self.id.iter().map(|byte| format!("{:02x}", byte)).collect()
    }

    pub fn less(&self, other: &KademliaID) -> bool {
        self.id
            .iter()
            .zip(other.id.iter())
            .find(|(a, b)| a != b)
            .map_or(false, |(a, b)| a < b)
    }

    pub fn equals(&self, other: &KademliaID) -> bool {
        self.id.iter().zip(other.id.iter()).all(|(a, b)| a == b)
    }

    pub fn distance(&self, other: &KademliaID) -> KademliaID {
        let dist: KadId = self
            .id
            .iter()
            .zip(other.id.iter())
            .map(|(a, b)| (a ^ b))
            .collect::<Vec<u8>>()
            .try_into()
            .expect("invalid length");
        KademliaID { id: dist }
    }
}
