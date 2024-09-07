use crate::kademlia_id::KademliaID;
pub struct Contact {
    id: KademliaID,
    address: String,
    pub distance: usize,
}

impl Contact {
    pub fn new(id: KademliaID, address: String) -> Self {
        Self {
            id,
            address: "".to_string(),
            distance: 0,
        }
    }

    pub fn calc_distance(&mut self, target: &KademliaID) -> &mut Self {
        self.distance = target.distance(&self.id);
        self
    }
}
