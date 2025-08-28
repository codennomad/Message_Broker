// src/bin/subscriber.rs
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use message_broker::proto::{frame::Frame, MessageCodec};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:6789").await?;
    let mut framed = Framed::new(stream, MessageCodec);
    println!("[Subscriber] Conectado ao servidor.");

    // Envia um comando SUB para registrar a intenção.
    framed.send(Frame::new_sub("meu-topico")).await?;

    // Loop para receber mensagens do servidor.
    while let Some(Ok(frame)) = framed.next().await {
        println!("[Subscriber] Recebido: {:?}", frame);
    }

    Ok(())
}