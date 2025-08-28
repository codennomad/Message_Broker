use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use message_broker::proto::{Frame, CommandType, MessageCodec};
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use bytes::Bytes;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:6789").await?;
    let mut framed = Framed::new(stream, MessageCodec);

    println!("Conectando ao Servidor.");

    // Envia um frame PUB
    let pub_frame = Frame {
        command_type: CommandType::Pub,
        payload: Bytes::from("hello world from client"),
    };
    framed.send(pub_frame.clone()).await?;
    println!("Enviado: {:?}", pub_frame);

    // Aguarda a resposta do echo do servidor
    if let Some(Ok(response)) = framed.next().await {
        println!("Recebido (echo): {:?}", response);
        assert_eq!(pub_frame, response);
    } else {
        println!("Servidor nao respondeu.");
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Cliente encerrando.");

    Ok(())
}