#[cfg(test)]
mod tests {
    use crate::kademlia_id::KademliaID;
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
    fn xor_metric_triangle_inequality() {
        let kad_id_1 = KademliaID::new();
        let kad_id_2 = KademliaID::new();
        let kad_id_3 = KademliaID::new();
        let ab = kad_id_1.distance(&kad_id_2);
        let bc = kad_id_2.distance(&kad_id_3);
        let ac = kad_id_1.distance(&kad_id_3);
        let distance_ab_num = xor_distance_to_num(&ab);
        let distance_bc_num = xor_distance_to_num(&bc);
        let distance_ac_num = xor_distance_to_num(&ac);
        let sum_ab_bc = add_160bit_numbers(&distance_ab_num, &distance_bc_num);
        assert!(
            compare_xor_distances(&distance_ac_num, &sum_ab_bc) != std::cmp::Ordering::Greater,
            "Triangle inequality failed: d(A, C) > d(A, B) + d(B, C)"
        );
    }

    fn add_160bit_numbers(a: &[u32; 5], b: &[u32; 5]) -> [u32; 5] {
        let mut result = [0u32; 5];
        let mut carry = 0u64;

        for i in (0..5).rev() {
            let sum = a[i] as u64 + b[i] as u64 + carry;
            result[i] = sum as u32;
            carry = sum >> 32;
        }
        result
    }

    fn compare_xor_distances(a: &[u32; 5], b: &[u32; 5]) -> std::cmp::Ordering {
        for (a_part, b_part) in a.iter().zip(b.iter()) {
            match a_part.cmp(b_part) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        std::cmp::Ordering::Equal
    }

    fn xor_distance_to_num(xor_distance: &KademliaID) -> [u32; 5] {
        let mut num = [0u32; 5];
        for (i, chunk) in xor_distance.id.chunks(4).enumerate() {
            num[i] = u32::from_be_bytes(chunk.try_into().unwrap());
        }
        num
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
}
