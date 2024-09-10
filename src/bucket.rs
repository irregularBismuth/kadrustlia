use crate::contact::Contact;
use crate::kademlia_id::KademliaID;
use std::collections::LinkedList;

#[derive(Clone)]
pub struct Bucket {
    list: LinkedList<Contact>,
}

impl Bucket {
    fn new() -> Self {
        Self {
            list: LinkedList::<Contact>::new(),
        }
    }

    fn add_contact(&mut self, contact: Contact) -> &Self {
        self.list.push_back(contact);
        self
    }

    fn get_contact_and_calc_distance(&mut self, target: KademliaID) -> Vec<Contact> {
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
