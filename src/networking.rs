use tokio::net::UdpSocket;

pub struct Networking;

impl Networking {
    pub async fn send_ping(target_addr: &str) -> std::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let ping_msg = "PING";

        socket.send_to(ping_msg.as_bytes(), target_addr).await?;
        println!("Sent PING to {}", target_addr);

        Ok(())
    }

    pub async fn listen_for_ping(bind_addr: &str) -> std::io::Result<()> {
        let socket = UdpSocket::bind(bind_addr).await?;
        println!("Listening for PINGs on {}", bind_addr);

        let mut buf = [0u8; 1024];

        loop {
            let (len, src) = socket.recv_from(&mut buf).await?;
            let received = String::from_utf8_lossy(&buf[..len]);

            if received == "PING" {
                println!("Received PING from {}", src);

                let pong_msg = "PONG";
                socket.send_to(pong_msg.as_bytes(), &src).await?;
                println!("Sent PONG to {}", src);
            } else {
                println!("Received unexpected message from {}", src);
            }
        }
    }
}
