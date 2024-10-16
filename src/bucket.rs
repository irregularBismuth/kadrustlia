use crate::constants::BUCKET_SIZE;
use crate::contact::Contact;
use crate::kademlia_id::KademliaID;
use std::collections::LinkedList;

#[derive(Clone)]
pub struct Bucket {
    list: LinkedList<Contact>,
}

impl Bucket {
    pub fn new() -> Self {
        Self {
            list: LinkedList::<Contact>::new(),
        }
    }
    pub fn add_contact(&mut self, contact: &Contact, target: KademliaID) -> &Self {
        let mut contact_clone = contact.clone();
        contact_clone.calc_distance(&target);

        if self.list.len() < BUCKET_SIZE {
            self.list.push_back(contact_clone);
        } else {
            let mut contacts: Vec<Contact> = self.list.iter().cloned().collect();
            contacts.push(contact_clone);

            contacts.sort_by(|a, b| a.get_distance().cmp(&b.get_distance()));

            contacts.truncate(BUCKET_SIZE);

            self.list = contacts.into_iter().collect();
        }

        self
    }

    pub fn get_contact_and_calc_distance(&mut self, target: KademliaID) -> Vec<Contact> {
        let mut contacts: Vec<Contact> = Vec::new();
        for contact in self.list.iter_mut() {
            contact.calc_distance(&target);
            contacts.push(contact.clone());
        }
        contacts
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}
