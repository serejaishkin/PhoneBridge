use crossbeam_channel::Receiver;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct UdpSender {
    socket: UdpSocket,
    target: SocketAddr,
    rx: Receiver<Vec<u8>>,
    seq: u16,
}

impl UdpSender {
    pub async fn new(
        bind_addr: &str,
        target: SocketAddr,
        rx: Receiver<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(bind_addr).await?;
        Ok(Self {
            socket,
            target,
            rx,
            seq: 0,
        })
    }

    pub async fn run(&mut self) {
        let mut buf = vec![0u8; 1500];
        loop {
            match self.rx.recv() {
                Ok(opus_frame) => {
                    if opus_frame.len() + 2 > buf.len() {
                        continue;
                    }
                    buf[0] = (self.seq >> 8) as u8;
                    buf[1] = (self.seq & 0xFF) as u8;
                    buf[2..2 + opus_frame.len()].copy_from_slice(&opus_frame);
                    let _ = self.socket.send_to(&buf[..2 + opus_frame.len()], self.target).await;
                    self.seq = self.seq.wrapping_add(1);
                }
                Err(_) => {
                    log::warn!("UDP sender channel closed");
                    break;
                }
            }
        }
    }
}
