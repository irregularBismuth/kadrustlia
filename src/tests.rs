#[cfg(test)]
mod tests {
    use crate::contact::Contact;
    use crate::kademlia_id::KademliaID;
    use crate::routing_table::RoutingTable;

    #[test]
    fn xor_metric() {
        let kad_id_1: KademliaID = KademliaID::new();
        let zero_distance = KademliaID::with_id([0u8; 20]);
        assert_eq!(
            kad_id_1.distance(&kad_id_1).to_hex(),
            zero_distance.to_hex(),
            "The distance to itself should be zero"
        );

        let kad_id_2: KademliaID = KademliaID::new();
        assert_ne!(
            kad_id_1.distance(&kad_id_2),
            zero_distance,
            "The distance should be greater than zero "
        );
    }
    #[test]
    fn test_find_closest_contacts() {
        let my_id = KademliaID::from_hex("0000000000000000000000000000000000000000".to_string());
        let me = Contact::new(my_id.clone(), "1256".to_string());

        let mut routing_table = RoutingTable::new(me);

        println!("Generating contacts...");

        for i in 0..21 {
            let hex_value = format!("{:040X}", i); // Generate sequential Kademlia IDs
            println!("Generated KademliaID: {}", hex_value);

            let kad_id = KademliaID::from_hex(hex_value.clone());
            let contact = Contact::new(kad_id.clone(), "123".to_string());

            routing_table.add_contact(contact);
            println!(
                "Added contact with ID: {} to routing table.",
                kad_id.to_hex()
            );
        }

        let target_id =
            KademliaID::from_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string());

        let closest_contacts = routing_table.find_closest_contacts(target_id.clone(), 20);

        assert_eq!(
            closest_contacts.len(),
            20,
            "Expected 20 closest contacts but got {}",
            closest_contacts.len()
        );

        let excluded_id =
            KademliaID::from_hex("0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string());
        let excluded_contact = Contact::new(excluded_id.clone(), "123".to_string());

        println!("Checking closest contacts to {}:", target_id.to_hex());
        let mut found_excluded = false;

        for contact in closest_contacts.iter() {
            println!("Closest contact ID: {}", contact.id.to_hex());
            if contact.id.to_hex() == excluded_contact.id.to_hex() {
                found_excluded = true;
            }
        }

        assert!(
        found_excluded,
        "The excluded contact (0x0FFFFFFFF...) was not found but should be among the closest contacts!"
    );
    }
    #[test]
    fn xor_metric_triangle_inequality() {
        let kad_id_1 = KademliaID::new();
        let kad_id_2 = KademliaID::new();
        let kad_id_3 = KademliaID::new();

        let ab = kad_id_1.distance(&kad_id_2);
        let bc = kad_id_2.distance(&kad_id_3);
        let ac = kad_id_1.distance(&kad_id_3);

        let mut ab_and_bc = Big160::new();
        ab_and_bc.add(&ab, &bc);
        let ab_bc_hex = ab_and_bc.to_hex();
        let ac_hex = ac.to_hex();

        let padded_ac_hex = format!("{:0>80}", ac_hex);
        println!("Sum (ab + bc) in hex: {}", ab_bc_hex);
        println!("Distance ac in hex (padded): {}", padded_ac_hex);

        assert!(
            padded_ac_hex <= ab_bc_hex,
            "Triangle inequality failed: d(A, C) > d(A, B) + d(B, C)"
        );
    }

    #[tokio::test]
    async fn hash_data() {
        let kad_id = KademliaID::new()
            .store_data("test".to_string())
            .await
            .to_hex();
        let kad_id2 = KademliaID::new()
            .store_data("test".to_string())
            .await
            .to_hex();
        assert_eq!(kad_id, kad_id2, "Don't have same hash");
    }

    struct Big160 {
        parts: [u32; 10],
    }

    impl Big160 {
        fn new() -> Big160 {
            Big160 { parts: [0; 10] }
        }

        fn add(&mut self, a: &KademliaID, b: &KademliaID) {
            let mut carry = 0u64;

            for i in (0..5).rev() {
                let a_part =
                    u32::from_be_bytes(a.id[i * 4..(i + 1) * 4].try_into().unwrap()) as u64;
                let b_part =
                    u32::from_be_bytes(b.id[i * 4..(i + 1) * 4].try_into().unwrap()) as u64;
                let sum = a_part + b_part + carry;

                self.parts[i] = (sum & 0xFFFFFFFF) as u32;
                carry = sum >> 32;
            }

            for i in 5..10 {
                self.parts[i] = carry as u32;
                carry = 0;
            }
        }

        pub fn to_hex(&self) -> String {
            self.parts
                .iter()
                .map(|part| format!("{:08x}", part))
                .collect::<Vec<String>>()
                .join("")
        }
    }
}
