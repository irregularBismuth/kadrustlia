use {
    crate::{constants::rpc::Command, rpc::RpcMessage},
    bincode::{deserialize, serialize},
    tokio::net::lookup_host,
    tokio::net::ToSocketAddrs,
    tokio::net::UdpSocket,
};

pub struct Networking;

impl Networking {
    pub async fn send_ping(target_addr: &str, cmd: Command) -> std::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let ping_msg = cmd;
        let rpc_msg = RpcMessage::Request {
            id: 1,
            method: ping_msg,
            params: vec!["alice".to_string()],
        };
        for addr in lookup_host(target_addr).await? {
            println!("addr is {:?}", addr);
            let address = addr;
            let bin_data = bincode::serialize(&rpc_msg).expect("failed to serialize data");
            socket.send_to(&bin_data, &address).await?;
            println!("Sent PING to {}", &address);
            break;
        }
        Ok(())
    }

    pub async fn send_rpc_response(target_addr: &str, cmd: Command) -> tokio::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let rpc_msg = RpcMessage::Response { id: 2, result: cmd };
        let bin_data = bincode::serialize(&rpc_msg).expect("Failed to serialize response");
        let target = format!("{}:5678", target_addr);
        socket.send_to(&bin_data, &target).await?;
        println!("Sent response (PONG or other) to {}", target);
        Ok(())
    }

    pub async fn listen_for_rpc(bind_addr: &str) -> std::io::Result<()> {
        let socket = UdpSocket::bind(bind_addr).await?;
        println!("Listening for RPC messages on {}", bind_addr);

        let mut buf = [0u8; 1024];

        loop {
            let (len, src) = socket.recv_from(&mut buf).await?;

            let received_msg: RpcMessage =
                bincode::deserialize(&buf[..len]).expect("failed to deserialize data");

            match received_msg {
                RpcMessage::Request { id, method, params } => match method {
                    Command::PING => {
                        println!("Received PING from {} with params: {:?}", src, params);
                        let src_ip = src.ip().to_string();
                        let dest_cp = src_ip.clone();
                        tokio::spawn(async move {
                            Networking::send_rpc_response(&src_ip, Command::PONG)
                                .await
                                .expect("no response was sent");
                        });

                        println!("Sent PONG to {}", dest_cp);
                    }
                    _ => {
                        println!("Received unexpected command from {}", src);
                    }
                },
                RpcMessage::Response { id, result } => {
                    println!("Received Response with ID {} and result: {:?}", id, result);
                }
                RpcMessage::Error { id, message } => {
                    println!("Received Error with ID {}: {}", id, message);
                }
            }
        }
    }
}
