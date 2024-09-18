use {
    crate::constants::ID_LENGTH,
    rand::Rng,
    serde::{Deserialize, Serialize},
    sha2::{Digest, Sha256},
    std::cmp::*,
};

type KadId = [u8; ID_LENGTH];

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct KademliaID {
    pub id: KadId,
}

impl KademliaID {
    pub fn new() -> Self {
        let mut id: KadId = [0u8; ID_LENGTH];
        rand::thread_rng().fill(&mut id[..]);
        Self { id }
    }

    pub fn with_id(id: KadId) -> Self {
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
        KademliaID::with_id(core::array::from_fn(|i| self.id[i] ^ other.id[i]))
    }
}

impl PartialEq for KademliaID {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
impl Eq for KademliaID {}

impl PartialOrd for KademliaID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for KademliaID {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.less(other) {
            Ordering::Less
        } else if other.less(self) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
