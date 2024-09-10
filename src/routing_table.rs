use crate::{
    bucket::Bucket,
    constants::{BUCKET_SIZE, RT_BCKT_SIZE},
    contact::Contact,
    kademlia_id::KademliaID,
};

#[derive(Clone)]
pub struct RoutingTable {
    me: Contact,
    buckets: [Option<Bucket>; RT_BCKT_SIZE],
}

impl RoutingTable {
    pub fn new(me: Contact) -> Self {
        Self {
            me,
            buckets: std::array::from_fn(|_| None),
        }
    }

    pub fn add_contact(contact: Contact) {}

    pub fn get_closest_neighbours(target: KademliaID, count: usize) {}
}
