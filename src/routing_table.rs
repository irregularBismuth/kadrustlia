use crate::{
    bucket::Bucket,
    constants::{ID_LENGTH, RT_BCKT_SIZE},
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

    pub fn get_bucket_index(&self, id: KademliaID) -> usize {
        let distance = self.me.id.distance(&id);

        if let Some(position) = distance
            .id
            .iter()
            .flat_map(|&byte| (0..8).rev().map(move |i| (byte >> i) & 1))
            .position(|bit| bit != 0)
        {
            let bucket_index = (ID_LENGTH * 8 - 1) - position;
            bucket_index
        } else {
            0
        }
    }


    pub fn add_contact(&mut self, contact: Contact) {
        let index: usize = self.get_bucket_index(contact.id.clone());
        match &mut self.buckets[index] {
            Some(bucket) => {
                bucket.add_contact(&contact, contact.id);
            }
            None => {
                let mut bucket = Bucket::new();
                bucket.add_contact(&contact, contact.id);
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

        let mut i = 1;
        while (bucket_index as isize - i as isize >= 0 || bucket_index + i < RT_BCKT_SIZE)
            && candidates.len() < count
        {
            if bucket_index >= i {
                if let Some(bucket_) = self.buckets[bucket_index - i].as_mut() {
                    let mut cntcs = bucket_.get_contact_and_calc_distance(target);
                    candidates.append(&mut cntcs);
                }
            }

            if bucket_index + i < RT_BCKT_SIZE {
                if let Some(bucket_) = self.buckets[bucket_index + i].as_mut() {
                    let mut cntcs = bucket_.get_contact_and_calc_distance(target);
                    candidates.append(&mut cntcs);
                }
            }
            i += 1;
        }
        candidates.sort();

        let mut count_ = count;
        if count_ > candidates.len() {
            count_ = candidates.len();
        }
        candidates.get_contacts(count_).to_vec()
    }
}
