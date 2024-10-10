use crate::constants::BUCKET_SIZE;
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
    pub fn add_contact(&mut self, contact: &Contact, target: KademliaID) -> &Self {
        let mut contact_clone = contact.clone(); // Create a mutable clone to modify distance
        contact_clone.calc_distance(&target);

        if self.list.len() < BUCKET_SIZE {
            // If there's still space, add the contact directly
            self.list.push_back(contact_clone);
        } else {
            // If the bucket is full, check if the new contact is closer than the furthest
            let mut contacts: Vec<Contact> = self.list.iter().cloned().collect();
            contacts.push(contact_clone);

            // Sort contacts by distance to the target
            contacts.sort_by(|a, b| a.get_distance().cmp(&b.get_distance()));

            // Retain only the closest BUCKET_SIZE contacts
            contacts.truncate(BUCKET_SIZE);

            // Rebuild the linked list with the closest contacts
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
