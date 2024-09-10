use crate::kademlia_id::KademliaID;

pub struct Contact {
    id: KademliaID,
    address: String,
    distance: Option<KademliaID>,
}

impl Contact {
    pub fn new(id: KademliaID, address: String) -> Self {
        Self {
            id,
            address: "".to_string(),
            distance: None,
        }
    }

    pub fn calc_distance(&mut self, target: &KademliaID) -> &mut Self {
        self.distance = Some(target.distance(&self.id));
        self
    }
}

pub struct ContactCandidates {
    contacts: Vec<Contact>,
}

impl ContactCandidates {
    pub fn append(&mut self, contacts: &mut Vec<Contact>) -> &mut Self {
        self.contacts.append(contacts);
        self
    }

    pub fn get_contacts(&mut self, count: usize) -> &mut [Contact] {
        let len = self.contacts.len();
        let end = count.min(len);
        &mut self.contacts[0..end]
    }
}
