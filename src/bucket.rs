use crate::contact::Contact;
use crate::kademlia_id::KademliaID;
use serde::Deserialize;
use serde::Serialize;
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
    pub fn add_contact(&mut self, contact: Contact) -> &Self {
        self.list.push_back(contact);
        self
    }

    pub fn remove_contact(&mut self, kad_id: KademliaID) {
        let mut new_list = LinkedList::new();
        while let Some(contact) = self.list.pop_front() {
            if contact.id != kad_id {
                new_list.push_back(contact);
            }
        }
        self.list = new_list;
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
