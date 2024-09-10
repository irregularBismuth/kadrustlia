use crate::contact::Contact;
use std::collections::LinkedList;
struct Bucket {
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
}
