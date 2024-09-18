use crate::{
    bucket::Bucket,
    constants::{BUCKET_SIZE, ID_LENGTH, RT_BCKT_SIZE},
    contact::Contact,
    contact::ContactCandidates,
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

    pub fn get_bucket_index(&mut self, id: KademliaID) -> usize {
        let distance: KademliaID = self.me.calc_distance(&id).get_distance();
        distance
            .id
            .iter()
            .flat_map(|&byte| (0..8).rev().map(move |i| (byte >> i) & 1))
            .position(|bit| bit != 0)
            .unwrap_or(RT_BCKT_SIZE - 1)
    }

    pub fn add_contact(&mut self, contact: Contact) {
        let index: usize = self.get_bucket_index(contact.id.clone());
        match &mut self.buckets[index] {
            Some(bucket) => {
                bucket.add_contact(contact);
            }
            None => {
                let mut bucket = Bucket::new();
                bucket.add_contact(contact);
                self.buckets[index] = Some(bucket);
            }
        }
    }

    pub fn find_closest_contacts(&mut self, target: KademliaID, count: usize) -> Vec<Contact> {
        let mut candidates = ContactCandidates::new();
        let bucket_index = self.get_bucket_index(target);
        if let Some(bucket) = self.buckets[bucket_index].as_mut() {
            let mut contacts = bucket.get_contact_and_calc_distance(target);
            candidates.append(&mut contacts);
        }
        let mut i = 0;
        while (bucket_index as isize - i as isize >= 0 || bucket_index + i < ID_LENGTH * 8)
            && candidates.len() < count
        {
            if bucket_index - i >= 0 {
                if let Some(bucket_) = self.buckets[bucket_index - 1].as_mut() {
                    let mut cntcs = bucket_.get_contact_and_calc_distance(target);
                    candidates.append(&mut cntcs);
                }
                if let Some(bucket_) = self.buckets[bucket_index + 1].as_mut() {
                    let mut cntcs = bucket_.get_contact_and_calc_distance(target);
                    candidates.append(&mut cntcs);
                }
            }
            i = i + 1;
        }
        candidates.sort();
        let mut count_ = count;
        if count_ > candidates.len() {
            count_ = candidates.len();
        }
        candidates.get_contacts(count_).to_vec()
    }
}
