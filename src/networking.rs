use {
    crate::{
        constants::rpc::Command, contact::Contact, kademlia_id::KademliaID,
        routing_table_handler::*, rpc::RpcMessage,
    },
    bincode::{deserialize, serialize},
    std::{collections::HashMap, sync::Arc, time::Duration},
    tokio::{net::{lookup_host, ToSocketAddrs, UdpSocket}, sync::{mpsc, Mutex}, time::sleep},
};
use tokio::sync::oneshot;

type RpcMap = Arc<Mutex<HashMap<KademliaID, oneshot::Sender<RpcMessage>>>>;
#[derive(Clone)]
pub struct Networking {
    response_map: RpcMap,
}
impl Networking {
    pub fn new() -> Self {
        Self {
            response_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn send_rpc_request_await(
        &self,
        rpc_id: KademliaID,
        target_addr: &str,
        cmd: Command,
        target_id: Option<KademliaID>,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    ) -> std::io::Result<Option<RpcMessage>> {

        let (tx, rx) = oneshot::channel();

        {
            let mut map = self.response_map.lock().await;
            map.insert(rpc_id.clone(), tx);
        }

        self.send_rpc_request(rpc_id.clone(), target_addr, cmd, target_id, data, contact)
            .await?;

        let timeout_duration = Duration::from_secs(15);
        match tokio::time::timeout(timeout_duration, rx).await {
            Ok(Ok(response)) => {
                {
                    let mut map = self.response_map.lock().await;
                    map.remove(&rpc_id);
                }
                Ok(Some(response))
            }
            Ok(Err(_)) => {
                {
                    let mut map = self.response_map.lock().await;
                    map.remove(&rpc_id);
                }
                Ok(None)
            }
            Err(_) => {
                {
                    let mut map = self.response_map.lock().await;
                    map.remove(&rpc_id);
                }
                Ok(None)
            }
        }
    }

    pub async fn send_rpc_request(
        &self,
        rpc_id: KademliaID,
        target_addr: &str,
        cmd: Command,
        target_id: Option<KademliaID>,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    ) -> std::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let rpc_msg = RpcMessage::Request {
            rpc_id,
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

        //sleep(Duration::from_millis(10000)).await;
        Ok(())
    }

    pub async fn send_rpc_response(
        rpc_id: KademliaID,
        target_addr: &str,
        cmd: Command,
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    ) -> tokio::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let rpc_msg = RpcMessage::Response {
            rpc_id,
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
        &self,
        mut tx: mpsc::Sender<RouteTableCMD>,
        bind_addr: &str,
    ) -> std::io::Result<()> {
        let socket = UdpSocket::bind(bind_addr).await?;
        println!("Listening for RPC messages on {}", bind_addr);

        let mut buf = [0u8; 65507];

        loop {
            let (len, src) = socket.recv_from(&mut buf).await?;

            let received_msg: RpcMessage =
                bincode::deserialize(&buf[..len]).expect("failed to deserialize data");

            match received_msg {
                RpcMessage::Request {
                    rpc_id,
                    method,
                    target_id,
                    data,
                    contact: cntact,
                } => match method {
                    Command::PING => {
                        println!(
                            "Received {:?} Request from {} rpc id {}",
                            method,
                            src,
                            rpc_id.to_hex()
                        );
                        let contact_vec = cntact.unwrap();

                        let contact = &contact_vec[0];

                        let _ = tx.send(RouteTableCMD::AddContact(contact.clone())).await;
                        /*let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);

                        let _ = tx
                            .send(RouteTableCMD::GetClosestNodes(contact.id.clone(), reply_tx))
                            .await;

                        if let Some(contacts) = reply_rx.recv().await {
                            println!("{:?}", contacts);
                        } else {
                            println!("no conacts from routing table");
                        }*/
                        tokio::spawn(async move {
                            Networking::send_rpc_response(
                                rpc_id,
                                &src.ip().to_string(),
                                Command::PONG,
                                None,
                                None,
                            )
                            .await
                            .expect("no response was sent");
                        });

                        println!("Sent PONG to {}", &src.ip().to_string());
                    }
                    Command::FINDNODE => {
                        println!(
                            "Received {:?} Request from {} rpc id {}",
                            method,
                            src,
                            rpc_id.to_hex()
                        );

                        //let Some(data) = target;

                        //let target = KademliaID::from_hex(data.expect("expected valid hex string"));

                        if let Some(target_id) = target_id {
                            let (reply_tx, mut reply_rx) = mpsc::channel::<Vec<Contact>>(1);

                            let _ = tx
                                .send(RouteTableCMD::GetClosestNodes(target_id, reply_tx))
                                .await;

                            if let Some(contacts) = reply_rx.recv().await {
                                let src_ip = src.to_string();
                                let own_id_copy = rpc_id.clone();
                                tokio::spawn(async move {
                                    Networking::send_rpc_response(
                                        own_id_copy,
                                        &src_ip,
                                        Command::FINDNODE,
                                        None,
                                        Some(contacts),
                                    )
                                    .await
                                    .expect("no response was sent");
                                });
                            } else {
                                println!("no conacts from routing table");
                            }
                        } else {
                            println!("{:?} request missing target_id", method);
                        }
                    }
                    Command::FINDVALUE => {
                        println!(
                            "Received {:?} Request from {} rpc id {}",
                            method,
                            src,
                            rpc_id.to_hex()
                        );

                        if let Some(target_id) = target_id {
                            let dir = "data";
                            let filename = format!("{}/{}.txt", dir, target_id.to_hex());

                            if let Ok(data) = tokio::fs::read_to_string(&filename).await {
                                let src_ip = src.to_string();
                                let own_id_copy = rpc_id.clone();
                                tokio::spawn(async move {
                                    Networking::send_rpc_response(
                                        own_id_copy,
                                        &src_ip,
                                        Command::FINDVALUE,
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
                                    let id_hex = rpc_id.to_hex();
                                    let own_id_copy = rpc_id.clone();
                                    println!("id_hex: {}", id_hex);

                                    for i in contacts.iter() {
                                        let kadid = i.id.to_hex();
                                        println!("kadid: {}", kadid);
                                    }

                                    tokio::spawn(async move {
                                        Networking::send_rpc_response(
                                            own_id_copy,
                                            &src_ip,
                                            Command::FINDVALUE,
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
                        println!(
                            "Received {:?} Request from {} rpc id {}",
                            method,
                            src,
                            rpc_id.to_hex()
                        );
                        if let Some(data) = data {
                            let mut kad_id = KademliaID::new();
                            kad_id.store_data(data).await;
                            let src_ip = src.ip().to_string();
                            let own_id_copy = rpc_id.clone();
                            tokio::spawn(async move {
                                Networking::send_rpc_response(
                                    own_id_copy,
                                    &src_ip,
                                    Command::STORE,
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
                    rpc_id,
                    result,
                    data,
                    contact,
                } => {
                    let sender_opt = {
                        let mut map = self.response_map.lock().await;
                        map.remove(&rpc_id)
                    };

                    if let Some(sender) = sender_opt {

                        let response_message = RpcMessage::Response {
                            rpc_id: rpc_id.clone(),
                            result,
                            data: data.clone(),
                            contact: contact.clone(),
                        };

                        let _ = sender.send(response_message);
                    } else {
                        println!("No waiting sender for rpc_id {}", rpc_id.to_hex());
                    }

                    match result {
                        Command::PONG => {
                            println!(
                                "Received {:?} Response from {} rpc id {}",
                                result,
                                src,
                                rpc_id.to_hex()
                            );
                            let src_ip = src.ip().to_string();
                            let contact = Contact::new(rpc_id, src_ip.clone());
                            let _ = tx.send(RouteTableCMD::AddContact(contact)).await;
                        }
                        Command::FINDNODE => {
                            println!(
                                "Received {:?} Response from {} rpc id {}",
                                result,
                                src,
                                rpc_id.to_hex()
                            );

                            if let Some(contacts) = contact {
                                for contact in &contacts {
                                    let _ = tx.send(RouteTableCMD::AddContact(contact.clone())).await;
                                }
                            } else {
                                println!("{:?} missing contacts", result);
                            }
                        }
                        Command::FINDVALUE => {
                            println!(
                                "Received {:?} Response from {} rpc id {}",
                                result,
                                src,
                                rpc_id.to_hex()
                            );

                            if let Some(value) = data {
                                println!("value found: {}", value);
                            } else if let Some(contacts) = contact {
                                println!("contacts: {:?}", contacts);
                                let id_hex = &rpc_id.to_hex();
                                println!("id_hex: {}", id_hex);

                                for i in contacts.iter() {
                                    let kadid = i.id.to_hex();
                                    println!("kadid: {}", kadid);
                                }

                                for contact in &contacts {
                                    let _ = tx.send(RouteTableCMD::AddContact(contact.clone())).await;
                                }
                            } else {
                                println!("{:?} response missing data and contacts", result);
                            }
                        }
                        Command::STORE => {
                            println!(
                                "Received {:?} Response from {} rpc id {}",
                                result,
                                src,
                                rpc_id.to_hex()
                            );
                        }
                        _ => {
                            println!(
                                "Received Response with ID {} and result: {:?}",
                                rpc_id.to_hex(),
                                result
                            );
                        }
                    }
                },
                RpcMessage::Error { rpc_id, message } => {
                    println!("Received Error with ID {}: {}", rpc_id.to_hex(), message);
                }
            }
        }
    }
}
