use {
    crate::{
        constants::rpc::Command, contact::Contact, kademlia::RouteTableCMD,
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
        data: Option<String>,
        contact: Option<Vec<Contact>>,
    ) -> std::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let rpc_msg = RpcMessage::Request {
            id: KademliaID::new(),
            method: cmd,
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
        let target = format!("{}:5678", target_addr);
        socket.send_to(&bin_data, &target).await?;
        println!("Sent response ({:?}) to {}", cmd, target);
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
                    data,
                    contact,
                } => match method {
                    Command::PING => {
                        println!(
                            "Recived {:?} Request from {} rpc id {}",
                            method,
                            src,
                            id.to_hex()
                        );
                        let src_ip = src.ip().to_string();
                        let dest_cp = src_ip.clone();
                        let _ = tx.send(RouteTableCMD::GetClosestNodes(id)).await;
                        tokio::spawn(async move {
                            Networking::send_rpc_response(&src_ip, Command::PONG, id, None, None)
                                .await
                                .expect("no response was sent");
                        });

                        println!("Sent PONG to {}", dest_cp);
                    }
                    Command::FINDNODE => {
                        println!(
                            "Recived {:?} Request from {} rpc id {}",
                            method,
                            src,
                            id.to_hex()
                        );
                    }
                    Command::FINDVALUE => {
                        println!(
                            "Recived {:?} Request from {} rpc id {}",
                            method,
                            src,
                            id.to_hex()
                        );
                    }
                    Command::STORE => {
                        println!(
                            "Recived {:?} Request from {} rpc id {}",
                            method,
                            src,
                            id.to_hex()
                        );
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
                        println!(
                            "Recived {:?} Response from {} rpc id {}",
                            result,
                            src,
                            id.to_hex()
                        );
                    }
                    Command::FINDNODE => {
                        println!(
                            "Recived {:?} Response from {} rpc id {}",
                            result,
                            src,
                            id.to_hex()
                        );
                    }
                    Command::FINDVALUE => {
                        println!(
                            "Recived {:?} Response from {} rpc id {}",
                            result,
                            src,
                            id.to_hex()
                        );
                    }
                    Command::STORE => {
                        println!(
                            "Recived {:?} Response from {} rpc id {}",
                            result,
                            src,
                            id.to_hex()
                        );
                    }
                    _ => {
                        println!(
                            "Received Response with ID {} and result: {:?}",
                            id.to_hex(),
                            result
                        );
                    }
                },
                RpcMessage::Error { id, message } => {
                    println!("Received Error with ID {}: {}", id.to_hex(), message);
                }
            }
        }
    }
}
