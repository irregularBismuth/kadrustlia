use {
    crate::constants::ID_LENGTH,
    rand::Rng,
    serde::{Deserialize, Serialize},
    sha2::{Digest, Sha256},
    std::cmp::*,
    tokio::fs,
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

    pub fn generate_random_id_in_bucket(&self, bucket_index: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut id = self.id;

        let num_bits = bucket_index;
        let num_full_bytes = num_bits / 8;
        let num_bits_in_partial_byte = num_bits % 8;

        for i in 0..num_full_bytes {
            id[i] = self.id[i];
        }

        if num_bits_in_partial_byte > 0 && num_full_bytes < ID_LENGTH {
            let mask = 0xff << (8 - num_bits_in_partial_byte);
            id[num_full_bytes] = self.id[num_full_bytes] & mask;
        }

        let byte_index = num_bits / 8;
        let bit_index = num_bits % 8;
        if byte_index < ID_LENGTH {
            id[byte_index] ^= 1 << (7 - bit_index);
        }

        for bit in (bucket_index + 1)..(ID_LENGTH * 8) {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            let random_bit = rng.gen::<bool>();
            if random_bit {
                id[byte_index] |= 1 << (7 - bit_index);
            } else {
                id[byte_index] &= !(1 << (7 - bit_index));
            }
        }

        KademliaID::with_id(id)
    }

    pub fn from_hex(hex: String) -> Self {
        let id: KadId = hex
            .as_bytes()
            .chunks(2)
            .map(|chunk| {
                let high = (chunk[0] as char).to_digit(16).unwrap();
                let low = (chunk[1] as char).to_digit(16).unwrap();
                ((high << 4) | low) as u8
            })
            .collect::<Vec<u8>>()
            .try_into()
            .expect("invalid kademlia id ");
        Self { id }
    }

    pub fn with_id(id: KadId) -> Self {
        Self { id }
    }

    pub async fn store_data(&mut self, data: String) -> Self {
        let hash = Sha256::digest(data.as_bytes());
        self.id.copy_from_slice(&hash[..ID_LENGTH]);

        let dir = "data";
        let filename = format!("{}/{}.txt", dir, self.to_hex());

        match fs::create_dir_all(dir).await {
            Ok(_) => {
                eprintln!("Directory '{}' created or already exists", dir);
            }
            Err(e) => {
                eprintln!("Failed to create directory '{}': {}", dir, e);
            }
        }

        match fs::write(&filename, data).await {
            Ok(_) => {
                eprintln!("Data successfully stored in file: {}", filename);
            }
            Err(e) => {
                eprintln!("Failed to store data in '{}': {}", filename, e);
            }
        }

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
