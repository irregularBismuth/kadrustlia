#[cfg(test)]
mod tests {
    use crate::kademlia_id::KademliaID;
    #[test]
    fn xor_metric() {
        let kad_id_1: KademliaID = KademliaID::new();
        //assert_eq!(kad_id_1.distance(&kad_id_1.clone()));

        let kad_id_2: KademliaID = KademliaID::new();
        // assert_eq!(kad_id_1.distance(&kad_id_2) > 0, true);
    }

    #[test]
    fn xor_metric_triangle_inequality() {
        /*  let kad_id_1 = KademliaID::new();
            let kad_id_2 = KademliaID::new();
            let kad_id_3 = KademliaID::new();
            let ab = kad_id_1.distance(&kad_id_2);
            let bc = kad_id_2.distance(&kad_id_3);
            let ac = kad_id_1.distance(&kad_id_3);
            assert!(ab + bc >= ac, "Triangle inequality failed");
        */
    }

    #[test]
    fn hash_data() {
        let kad_id = KademliaID::new().store_data("test".to_string()).to_hex();
        let kad_id2 = KademliaID::new().store_data("test".to_string()).to_hex();
        assert_eq!(kad_id, kad_id2, "Don't have same hash");
    }
}
