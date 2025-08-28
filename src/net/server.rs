use crate::proto::MessageCodec; // Importamos nosso codec
use tokio::net::{TcpListener, TcpStream};
use futures::{StreamExt, SinkExt}; // Traits para trabalhar com Streams e Sinks
use tokio_util::codec::Framed;
use tracing::{error, info, debug};
use anyhow::Result;

pub async fn run() -> Result<()> {
    // Listener TCP que escuta no endereco  especifico
    let listener = TcpListener::bind("127.0.0.1:6789").await?;
    info!("Servidor escutando em 127.0.0.1:6789");

    loop{
        // O metodo `accept()` aguarda uma nova conexao.
        // Retorna `Result` contendo o socket da conexao e endereco do cliente.
        let (socket, addr) = listener.accept().await?;
        info!("Nova conexao de: {}", addr);

        // Para cada conexao, cria uma nova tarefa assincrona com `tokio::spawn`
        // Isso permite que o servidor lide com multiplas conexao concorretemente,
        // sem que uma conexao lenta bloqueie as outras.
        tokio::spawn(async move {
            if let Err(e) = process_connection(socket).await {
                error!("Erro ao processar conexao de {}: {}", addr, e);
            }
        });
    }
}

/// Processa uma unica conexao TCP.
async fn process_connection(socket: TcpStream) -> Result<()> {
    // `Framed` envolve nosso socket e usa o `MessageCodec` para
    // transformar o fluxo de bytes em um fluxo de `Frame`s.
    let mut framed = Framed::new(socket, MessageCodec);

    // Ler `Frame`s completos.
    while let Some(result) = framed.next().await {
        match result {
            Ok(frame) => {
                debug!("Frame recebido: {:?}", frame);

                // Logica de `echo` com frames.
                // Responde ao cliente com o mesmo frame quem recebeu
                if let Err(e) = framed.send(frame).await {
                    error!("Erro ao enviar frame de resposta: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Error ao decodificar frame: {}", e);
                break;
            }
        }
    }
    info!("Cliente desconectado.");
    Ok(())

}