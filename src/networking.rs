use {
    crate::{
        constants::rpc::Command, contact::Contact, routing_table_handler::*,
        kademlia_id::KademliaID, rpc::RpcMessage,
    },
    bincode::{deserialize, serialize},
    tokio::net::{lookup_host, ToSocketAddrs, UdpSocket},
    tokio::sync::mpsc,
};

pub struct Networking;

impl Networking {
    pub async fn send_rpc_request(
        target_addr: &str,
        cmd: Command,
        target_id: Option<KademliaID>,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    ) -> std::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let rpc_msg = RpcMessage::Request {
            id: KademliaID::new(),
            method: cmd,
            target_id,
            data,
            contact,
        };
        for addr in lookup_host(target_addr).await? {
            let bin_data = bincode::serialize(&rpc_msg).expect("failed to serialize data");
            socket.send_to(&bin_data, &addr).await?;
            println!("Sent {:?} to {}", cmd, &addr);
            break;
        }
        Ok(())
    }

    pub async fn send_rpc_response(
        target_addr: &str,
        cmd: Command,
        id: KademliaID,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    ) -> tokio::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let rpc_msg = RpcMessage::Response {
            id,
            result: cmd,
            data,
            contact,
        };
        let bin_data = bincode::serialize(&rpc_msg).expect("Failed to serialize response");

        let target = if target_addr.contains(':') {
            let parts: Vec<&str> = target_addr.split(':').collect();
            format!("{}:5678", parts[0])
        } else {
            format!("{}:5678", target_addr)
        };

        println!("Sending response to {}", target);

        let mut attempts = 0;
        while attempts < 3 {
            if let Err(e) = socket.send_to(&bin_data, &target).await {
                println!("Attempt {} to send failed: {}", attempts + 1, e);
                attempts += 1;
            } else {
                println!("Successfully sent on attempt {}", attempts + 1);
                break;
            }
        }

        Ok(())
    }

    pub async fn listen_for_rpc(
        mut tx: mpsc::Sender<RouteTableCMD>,
        bind_addr: &str,
    ) -> std::io::Result<()> {
        let socket = UdpSocket::bind(bind_addr).await?;
        println!("Listening for RPC messages on {}", bind_addr);

        let mut buf = [0u8; 1024];

        loop {
            let (len, src) = socket.recv_from(&mut buf).await?;

            let received_msg: RpcMessage =
                bincode::deserialize(&buf[..len]).expect("failed to deserialize data");

            match received_msg {
                RpcMessage::Request {
                    id,
                    method,
                    target_id,
                    data,
                    contact,
                } => match method {
                    Command::PING => {
                        println!("Recived {:?} Request from {} rpc id {}", method, src, id.to_hex());
                        let src_ip = src.ip().to_string();
                        let dest_cp = src_ip.clone();
                        let dest_cp_cp = src_ip.clone();

                        let _ = tx.send(RouteTableCMD::AddContact(Contact::new(id, dest_cp))).await;
                        //let _ = tx.send(RouteTableCMD::GetClosestNodes(id)).await;
                        tokio::spawn(async move {
                            Networking::send_rpc_response(&src_ip, Command::PONG, id, None, None)
                                .await
                                .expect("no response was sent");
                        });

                        println!("Sent PONG to {}", dest_cp_cp);
                    }
                    Command::FINDNODE => {
                        println!("Recived {:?} Request from {} rpc id {}", method, src, id.to_hex());

                        //let Some(data) = target;

                        //let target = KademliaID::from_hex(data.expect("expected valid hex string"));

                        if let Some(target_id) = target_id {
                            let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);

                            let _ = tx.send(RouteTableCMD::GetClosestNodes(target_id, reply_tx)).await;

                            if let Some(contacts) = reply_rx.recv().await {
                                let src_ip = src.to_string();
                                Networking::send_rpc_response(&src_ip, Command::FINDNODE, id, None, Some(contacts)).await?;
                            } else {
                                println!("no conacts from routing table");
                            }
                        } else {
                            println!("{:?} request missing target_id", method);
                        }
                    }
                    Command::FINDVALUE => {
                        println!("Recived {:?} Request from {} rpc id {}", method, src, id.to_hex());

                        /*let src_ip = src.to_string();
                        tokio::spawn(async move {
                            Networking::send_rpc_response(
                                &src_ip,
                                Command::FINDVALUE,
                                id,
                                None,
                                None,
                            )
                            .await
                            .expect("no response was sent");
                        });*/

                        if let Some(target_id) = target_id {
                            let dir = "data";
                            let filename = format!("{}/{}.txt", dir, target_id.to_hex());

                            if let Ok(data) = tokio::fs::read_to_string(&filename).await {
                                let src_ip = src.to_string();
                                tokio::spawn(async move {
                                    Networking::send_rpc_response(
                                        &src_ip,
                                        Command::FINDVALUE,
                                        id,
                                        Some(data),
                                        None,
                                    )
                                    .await
                                    .expect("no response was sent");
                                });
                            } else {
                                let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);

                                let _ = tx
                                    .send(RouteTableCMD::GetClosestNodes(target_id, reply_tx))
                                    .await;

                                if let Some(contacts) = reply_rx.recv().await {
                                    let contacts_cp = contacts.clone();
                                    println!("contacts: {:?}", contacts_cp);
                                    let src_ip = src.to_string();
                                    tokio::spawn(async move {
                                        Networking::send_rpc_response(
                                            &src_ip,
                                            Command::FINDVALUE,
                                            id,
                                            None,
                                            Some(contacts),
                                        )
                                        .await
                                        .expect("no response was sent");
                                    });
                                } else {
                                    println!("no contacts from routing table");
                                }
                            }
                        } else {
                            println!("{:?} request missing target_id", method);
                        }
                    }
                    Command::STORE => {
                        println!("Recived {:?} Request from {} rpc id {}", method, src, id.to_hex());
                        if let Some(data) = data {
                            let mut kad_id = KademliaID::new();
                            kad_id.store_data(data).await;

                            let src_ip = src.ip().to_string();
                            tokio::spawn(async move {
                                Networking::send_rpc_response(
                                    &src_ip,
                                    Command::STORE,
                                    kad_id,
                                    None,
                                    None,
                                )
                                .await
                                .expect("Failed to send STORE response");
                            });
                        } else {
                            println!("STORE request missing data");
                        }
                    }
                    _ => {
                        println!("Received unexpected command from {}", src);
                    }
                },
                RpcMessage::Response {
                    id,
                    result,
                    data,
                    contact,
                } => match result {
                    Command::PONG => {
                        println!("Recived {:?} Response from {} rpc id {}", result, src, id.to_hex());
                    }
                    Command::FINDNODE => {
                        println!("Recived {:?} Response from {} rpc id {}", result, src, id.to_hex());

                        if let Some(contacts) = contact {
                            for contact in &contacts {
                                let _ = tx.send(RouteTableCMD::AddContact(contact.clone())).await;
                            }
                        } else {
                            println!("{:?} missing contacts", result);
                        }
                    }
                    Command::FINDVALUE => {
                        println!("Recived {:?} Response from {} rpc id {}", result, src, id.to_hex());

                        if let Some(value) = data {
                            println!("value found: {}", value);
                        } else if let Some(contacts) = contact {
                            println!("contacts: {:?}", contacts);
                            for contact in &contacts {
                                let _ = tx.send(RouteTableCMD::AddContact(contact.clone())).await;
                            }
                        } else {
                            println!("{:?} response missing data and contacts", result);
                        }
                    }
                    Command::STORE => {
                        println!("Recived {:?} Response from {} rpc id {}", result, src, id.to_hex());
                    }
                    _ => {
                        println!("Received Response with ID {} and result: {:?}", id.to_hex(), result);
                    }
                },
                RpcMessage::Error { id, message } => {
                    println!("Received Error with ID {}: {}", id.to_hex(), message);
                }
            }
        }

        println!("------------------OUT OF LOOP------------------");
    }
}
