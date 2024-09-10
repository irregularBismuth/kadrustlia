use crate::kademlia_id::KademliaID;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct Contact {
    id: KademliaID,
    address: String,
    distance: Option<KademliaID>,
}

type Contacts = Vec<Contact>;

impl Contact {
    pub fn new(id: KademliaID, address: String) -> Self {
        Self {
            id,
            address,
            distance: None,
        }
    }

    pub fn calc_distance(&mut self, target: &KademliaID) -> &mut Self {
        self.distance = Some(target.distance(&self.id));
        self
    }

    pub fn get_distance(&self) -> KademliaID {
        self.distance
            .expect("error no distance was set for contact")
    }

    pub fn less(&self, other: Contact) -> bool {
        self.get_distance().less(&other.get_distance())
    }
}

pub struct ContactCandidates {
    contacts: Contacts,
}

impl ContactCandidates {
    pub fn new() -> Self {
        Self {
            contacts: Vec::new(),
        }
    }
    pub fn append(&mut self, contacts: &mut Contacts) -> &mut Self {
        self.contacts.append(contacts);
        self
    }

    pub fn sort(&mut self) {
        self.contacts
            .sort_by(|a, b| a.get_distance().cmp(&b.get_distance()));
    }

    pub fn len(&self) -> usize {
        self.contacts.len()
    }

    pub fn get_contacts(&mut self, count: usize) -> &mut [Contact] {
        let len = self.contacts.len();
        let end = count.min(len);
        &mut self.contacts[0..end]
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        self.contacts.swap(i, j);
    }

    pub fn less(&self, i: usize, j: usize) -> bool {
        self.contacts[i].less(self.contacts[j].clone())
    }
}
