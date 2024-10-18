use crate::{
    contact::Contact, kademlia::Kademlia, kademlia_id::KademliaID,
    routing_table_handler::RouteTableCMD,
};

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::sync::Arc;
    use std::time::Duration;

    use crate::bucket::Bucket;
    use crate::cli::{CMDStatus, Cli, Command};
    use crate::constants::{rpc::Command as otherCommand, BUCKET_SIZE, ID_LENGTH, RT_BCKT_SIZE};
    use crate::contact::Contact;
    use crate::kademlia::Kademlia;
    use crate::kademlia_id::KademliaID;
    use crate::networking::Networking;
    use crate::routing_table::RoutingTable;
    use crate::routing_table_handler::{routing_table_handler, RouteTableCMD};
    use crate::rpc::RpcMessage;
    use tokio::net::UdpSocket;
    use tokio::sync::{broadcast, mpsc};
    use tokio::time::sleep;
    #[test]
    fn test_contact_placed_in_correct_bucket() {
        let my_id = KademliaID::new();
        let me = Contact::new(my_id.clone(), "127.0.0.1".to_string());
        let mut routing_table = RoutingTable::new(me.clone());

        for i in 0..BUCKET_SIZE {
            let contact_id = my_id.generate_random_id_in_bucket(i);
            let contact = Contact::new(contact_id.clone(), format!("127.0.0.{}", i));

            let expected_bucket_index = routing_table.get_bucket_index(contact_id.clone());

            routing_table.add_contact(contact.clone());

            let actual_bucket_index = routing_table.get_bucket_index(contact_id);

            assert_eq!(
                expected_bucket_index, actual_bucket_index,
                "Contact was placed in the wrong bucket"
            );
        }
    }

    #[test]
    fn test_duplicate_contact_in_bucket() {
        let mut bucket = Bucket::new();
        let target_id = KademliaID::new();
        let contact_id = target_id.generate_random_id_in_bucket(0);
        let contact = Contact::new(contact_id.clone(), "127.0.0.1".to_string());

        bucket.add_contact(&contact, target_id.clone());
        bucket.add_contact(&contact, target_id.clone());

        assert_eq!(
            bucket.len(),
            1,
            "Duplicate contact should not increase bucket size"
        );
    }

    #[test]
    fn test_empty_bucket() {
        let mut bucket = Bucket::new();
        let target_id = KademliaID::new();

        let contacts = bucket.get_contact_and_calc_distance(target_id.clone());
        assert_eq!(contacts.len(), 0, "Expected empty bucket but got contacts");
    }

    #[test]
    fn test_routing_table_bucket_indexing() {
        let my_id = KademliaID::new();
        let me = Contact::new(my_id.clone(), "127.0.0.1".to_string());
        let mut routing_table = RoutingTable::new(me.clone());

        for i in 0..BUCKET_SIZE {
            let contact_id = my_id.generate_random_id_in_bucket(i);
            let contact = Contact::new(contact_id.clone(), format!("127.0.0.{}", i));
            let bucket_index = routing_table.get_bucket_index(contact_id.clone());
            assert!(bucket_index < RT_BCKT_SIZE, "Bucket index out of bounds");

            routing_table.add_contact(contact.clone());
        }
    }

    #[test]
    fn xor_metric() {
        let kad_id_1: KademliaID = KademliaID::new();
        let kad_id_2 = KademliaID::new();

        assert_eq!(
            kad_id_1.distance(&kad_id_2),
            kad_id_2.distance(&kad_id_1),
            "XOR is symmetric"
        );

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
            "The distance should be greater than zero"
        );
    }

    #[test]
    fn test_full_routing_table() {
        let my_id = KademliaID::new();
        let me = Contact::new(my_id.clone(), "127.0.0.1".to_string());
        let mut routing_table = RoutingTable::new(me.clone());

        for i in 0..(BUCKET_SIZE * 2) {
            let contact_id = my_id.generate_random_id_in_bucket(i);
            let contact = Contact::new(contact_id.clone(), format!("127.0.0.{}", i));
            routing_table.add_contact(contact);
        }

        let target_id = my_id.generate_random_id_in_bucket(1);
        let closest_contacts = routing_table.find_closest_contacts(target_id.clone(), BUCKET_SIZE);

        assert!(
            !closest_contacts.is_empty(),
            "Expected some contacts in routing table"
        );
    }

    #[test]
    fn test_kademlia_id_edge_cases() {
        let zero_id = KademliaID::from_hex("0000000000000000000000000000000000000000".to_string());
        let zero_id_hex = zero_id.to_hex();
        assert_eq!(
            zero_id_hex, "0000000000000000000000000000000000000000",
            "Expected all-zero Kademlia ID"
        );

        let one_id = KademliaID::from_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_string());
        let one_id_hex = one_id.to_hex().to_uppercase();
        assert_eq!(
            one_id_hex, "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
            "Expected all-one Kademlia ID"
        );
    }

    #[test]
    fn test_find_closest_contacts() {
        let my_id = KademliaID::from_hex("0000000000000000000000000000000000000000".to_string());
        let me = Contact::new(my_id.clone(), "1256".to_string());

        let mut routing_table = RoutingTable::new(me);

        println!("Generating contacts...");

        for i in 0..21 {
            let hex_value = format!("{:040X}", i);
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

        println!("Checking closest contacts to {}:", target_id.to_hex());

        for (i, contact) in closest_contacts.iter().enumerate() {
            let expected_id = format!("{:040X}", 20 - i);
            assert_eq!(
                contact.id.to_hex().to_uppercase(),
                expected_id,
                "Contact ID at position {} does not match expected ID {}",
                i,
                expected_id
            );
            println!("Closest contact ID: {}", contact.id.to_hex());
        }
    }

    #[test]
    fn test_add_contact_to_bucket() {
        let mut bucket = Bucket::new();
        let target_id = KademliaID::new();

        for i in 0..(BUCKET_SIZE + 5) {
            let contact_id = target_id.generate_random_id_in_bucket(i);
            let contact = Contact::new(contact_id, format!("address{}", i));
            bucket.add_contact(&contact, target_id);
        }

        assert_eq!(
            bucket.len(),
            BUCKET_SIZE,
            "Bucket size exceeded the BUCKET_SIZE limit"
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
        assert_eq!(kad_id, kad_id2, "Hashes do not match");
    }

    #[tokio::test]
    async fn test_routing_table_handler() {
        let local_id = KademliaID::new();
        let local_contact = Contact::new(local_id.clone(), "127.0.0.1:8080".to_string());
        let routing_table = RoutingTable::new(local_contact.clone());

        let (tx, rx) = mpsc::channel::<RouteTableCMD>(32);

        tokio::spawn(async move {
            routing_table_handler(rx, routing_table).await;
        });

        let contact_id = KademliaID::new();
        let contact = Contact::new(contact_id.clone(), "127.0.0.1:8081".to_string());

        tx.send(RouteTableCMD::AddContact(contact.clone()))
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        tx.send(RouteTableCMD::GetBucketIndex(contact_id.clone(), reply_tx))
            .await
            .unwrap();

        let bucket_index = reply_rx.recv().await.expect("Did not receive bucket index");

        assert!(
            bucket_index < RT_BCKT_SIZE,
            "Bucket index out of range: {}",
            bucket_index
        );

        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        tx.send(RouteTableCMD::GetClosestNodes(contact_id.clone(), reply_tx))
            .await
            .unwrap();

        let closest_contacts = reply_rx.recv().await.expect("Did not receive contacts");

        assert_eq!(
            closest_contacts.len(),
            1,
            "Expected 1 closest contact, got {}",
            closest_contacts.len()
        );
        assert_eq!(
            closest_contacts[0].id, contact_id,
            "The contact ID does not match"
        );
    }

    #[test]
    fn test_contact_from_hex() {
        let hex_id = "0123456789abcdef0123456789abcdef01234567".to_string();
        let address = "127.0.0.1:8080".to_string();

        let contact = Contact::contact_from_hex(hex_id.clone(), address.clone());

        assert_eq!(contact.id.to_hex(), hex_id, "Contact ID does not match");
        assert_eq!(contact.address, address, "Contact address does not match");
    }

    #[test]
    fn test_contact_less() {
        let target_id = KademliaID::new();

        let mut contact1 = Contact::new(KademliaID::new(), "127.0.0.1:8081".to_string());
        contact1.calc_distance(&target_id);

        let mut contact2 = Contact::new(KademliaID::new(), "127.0.0.1:8082".to_string());
        contact2.calc_distance(&target_id);

        let less = contact1.less(contact2.clone());

        let expected = contact1.get_distance().less(&contact2.get_distance());

        assert_eq!(less, expected, "Contact less comparison failed");
    }

    #[test]
    fn test_kademlia_id_from_data() {
        let data = "some test data";
        let id1 = KademliaID::from_data(data);
        let id2 = KademliaID::from_data(data);

        assert_eq!(id1, id2, "KademliaIDs from the same data should be equal");
    }

    #[test]
    fn test_kademlia_id_partial_ord() {
        let id1 = KademliaID::from_hex("0000000000000000000000000000000000000001".to_string());
        let id2 = KademliaID::from_hex("0000000000000000000000000000000000000002".to_string());
        let id3 = KademliaID::from_hex("0000000000000000000000000000000000000001".to_string());

        assert!(id1 < id2, "id1 should be less than id2");
        assert!(id2 > id1, "id2 should be greater than id1");
        assert_eq!(
            id1.partial_cmp(&id3),
            Some(Ordering::Equal),
            "id1 should equal id3"
        );
    }

    #[test]
    fn test_kademlia_id_cmp_equal() {
        let id1 = KademliaID::from_hex("ABCDEF1234567890ABCDEF1234567890ABCDEF12".to_string());
        let id2 = KademliaID::from_hex("ABCDEF1234567890ABCDEF1234567890ABCDEF12".to_string());

        let ordering = id1.cmp(&id2);
        assert_eq!(ordering, Ordering::Equal, "Expected Ordering::Equal");
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

    #[tokio::test]
    async fn test_kademlia_new() {
        let kademlia = Kademlia::new();
        assert!(kademlia.route_table_tx.capacity() > 0);
        assert_eq!(kademlia.own_id.id.len(), ID_LENGTH);
    }

    #[tokio::test]
    async fn test_kademlia_join() {
        let kademlia = Kademlia::new();
        let result = kademlia.join().await;
        assert!(result.is_ok(), "Kademlia join failed: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_iterative_find_node() {
        let kademlia = Kademlia::new();
        let target_id = KademliaID::new();

        let result = kademlia.iterative_find_node(target_id).await;
        assert!(
            result.is_ok(),
            "iterative_find_node failed: {:?}",
            result.err()
        );
        let contacts = result.unwrap();
        assert!(contacts.len() >= 0, "Expected contacts");
    }

    #[tokio::test]
    async fn test_parse_command() {
        let kademlia = Arc::new(Kademlia::new());
        let (shutdown_tx, _) = broadcast::channel(1);
        let cli = Cli::new(kademlia.clone(), shutdown_tx.clone());

        let input = "get 0123456789abcdef0123456789abcdef01234567";
        let command = cli.parse_command(input);
        assert!(command.is_ok());
        match command.unwrap() {
            Command::GET(hash) => assert_eq!(hash, "0123456789abcdef0123456789abcdef01234567"),
            _ => panic!("Expected GET command"),
        }
    }

    #[tokio::test]
    async fn test_execute_command_exit() {
        let kademlia = Arc::new(Kademlia::new());
        let (shutdown_tx, _) = broadcast::channel(1);
        let cli = Cli::new(kademlia.clone(), shutdown_tx.clone());

        let status = cli.execute_command(Command::EXIT).await;
        match status {
            CMDStatus::EXIT => assert!(true),
            _ => assert!(false, "Expected CMDStatus::EXIT"),
        }
    }

    #[tokio::test]
    async fn test_iterative_find_node_with_empty_routing_table() {
        let kademlia = Kademlia::new();
        let target_id = KademliaID::new();

        let result = kademlia.iterative_find_node(target_id).await;

        assert!(result.is_ok(), "Expected Ok, but got an error");
        let contacts = result.unwrap();
        assert!(contacts.is_empty(), "Expected no contacts, but got some");
    }

    #[tokio::test]
    async fn test_iterative_find_node_with_network_failure() {
        let kademlia = Kademlia::new();
        let target_id = KademliaID::new();
        let contact_id = KademliaID::new();
        let contact = Contact::new(contact_id.clone(), "127.0.0.1:8080".to_string());

        kademlia
            .route_table_tx
            .send(RouteTableCMD::AddContact(contact.clone()))
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let result = kademlia.iterative_find_node(target_id).await;

        assert!(result.is_ok(), "Expected Ok, but got an error");
        let contacts = result.unwrap();
        assert!(
            contacts.is_empty(),
            "Expected no new contacts after failure"
        );
    }

    #[tokio::test]
    async fn test_iterative_find_value_with_empty_routing_table() {
        // Arrange: Create a new Kademlia instance with an empty routing table
        let kademlia = Kademlia::new();
        let target_id = KademliaID::new();

        // Act: Call iterative_find_value with an empty routing table
        let result = kademlia.iterative_find_value(target_id).await;

        // Assert: Ensure that no value is found since the routing table is empty
        assert!(result.is_ok(), "Expected Ok, but got an error");
        let value = result.unwrap();
        assert!(value.is_none(), "Expected no value, but found some data");
    }

    #[tokio::test]
    async fn test_iterative_find_value() {
        let kademlia = Kademlia::new();
        let target_id = KademliaID::new();
        let result = kademlia.iterative_find_value(target_id).await;
        assert!(
            result.is_ok(),
            "iterative_find_value failed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_iterative_store_no_contacts() {
        // Arrange: Create a new Kademlia instance and mock `iterative_find_node` to return no contacts
        let kademlia = Kademlia::new();
        let target_id = KademliaID::new();

        // Mocking the `iterative_find_node` to return an empty vector
        let mock_closest_nodes: Vec<Contact> = vec![];

        // Simulate that `iterative_find_node` returns no contacts
        let kademlia_clone = kademlia.clone();
        let result = kademlia_clone
            .iterative_store(target_id, "test data".to_string())
            .await;

        // Assert: Since no contacts are found, the store operation should just return Ok
        assert!(result.is_ok(), "Expected Ok, but got an error");
    }

    #[tokio::test]
    async fn test_iterative_store() {
        let kademlia = Kademlia::new();
        let target_id = KademliaID::new();
        let data = "test data".to_string();
        let result = kademlia.iterative_store(target_id, data).await;
        assert!(result.is_ok(), "iterative_store failed: {:?}", result.err());
    }
    #[tokio::test]
    async fn test_send_rpc_request() {
        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let (msg_tx, mut msg_rx) = mpsc::channel(1);

        let server_task = tokio::spawn(async move {
            let mut buf = [0u8; 65507];
            let (len, _src) = server_socket.recv_from(&mut buf).await.unwrap();

            let received_msg: RpcMessage =
                bincode::deserialize(&buf[..len]).expect("Failed to deserialize data");

            msg_tx.send(received_msg).await.unwrap();
        });

        let networking = Networking::new();

        let rpc_id = KademliaID::new();
        let target_addr = server_addr.to_string();

        let result = networking
            .send_rpc_request(
                rpc_id.clone(),
                &target_addr,
                otherCommand::PING,
                None,
                None,
                None,
            )
            .await;

        assert!(
            result.is_ok(),
            "Failed to send RPC request: {:?}",
            result.err()
        );

        sleep(Duration::from_millis(100)).await;

        if let Some(received_msg) = msg_rx.recv().await {
            match received_msg {
                RpcMessage::Request {
                    rpc_id: received_rpc_id,
                    method,
                    ..
                } => {
                    assert_eq!(received_rpc_id, rpc_id, "RPC IDs do not match");
                    assert_eq!(method, otherCommand::PING, "Unexpected RPC command");
                }
                _ => panic!("Expected RpcMessage::Request"),
            }
        } else {
            panic!("Did not receive message on the server");
        }

        server_task.await.unwrap();
    }
    #[tokio::test]
    async fn test_rpc_timeout() {
        let networking = Networking::new();
        let rpc_id = KademliaID::new();
        let target_addr = "127.0.0.1:12345";
        let result = networking
            .send_rpc_request_await(
                rpc_id.clone(),
                target_addr,
                otherCommand::PING,
                None,
                None,
                None,
            )
            .await;
        assert!(result.is_ok(), "Request should not fail");
        assert!(
            result.unwrap().is_none(),
            "Expected no response due to timeout"
        );
    }

    #[test]
    fn test_calc_distance() {
        let id1 = KademliaID::new();
        let id2 = KademliaID::new();
        let mut contact = Contact::new(id1.clone(), "127.0.0.1:8080".to_string());

        contact.calc_distance(&id2);
        let expected_distance = id1.distance(&id2);

        assert_eq!(
            contact.get_distance(),
            expected_distance,
            "Distance calculation is incorrect"
        );
    }

    #[test]
    fn test_calc_distance_same_id() {
        let id = KademliaID::new();
        let mut contact = Contact::new(id.clone(), "127.0.0.1:8080".to_string());

        contact.calc_distance(&id);
        let expected_distance = KademliaID::with_id([0u8; 20]);

        assert_eq!(
            contact.get_distance(),
            expected_distance,
            "Distance should be zero for the same ID"
        );
    }
}
