// src/bin/publisher.rs
use anyhow::Result;
use bytes::Bytes;
use futures::SinkExt;
use message_broker::proto::{frame::Frame, MessageCodec};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:6789").await?;
    let mut framed = Framed::new(stream, MessageCodec);
    println!("[Publisher] Conectado ao servidor.");

    let mut i = 0;
    loop {
        let message = format!("minha mensagem {}", i);
        let pub_frame = Frame::new_pub(Bytes::from(message));

        println!("[Publisher] Enviando: {:?}", pub_frame);
        framed.send(pub_frame).await?;

        i += 1;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}