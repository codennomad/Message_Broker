use crate::proto::{frame::{Frame, CommandType}, MessageCodec}; // Importamos nosso codec
use tokio::net::{TcpListener, TcpStream};
use futures::{StreamExt, SinkExt}; // Traits para trabalhar com Streams e Sinks
use tokio_util::codec::Framed;
use tracing::{error, info, debug, warn};
use anyhow::Result;
use tokio::sync::broadcast;

pub async fn run(tx: broadcast::Sender<Frame>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6789").await?;
    info!("Servidor escutando em 127.0.0.1:6789");

    loop {
        let (socket, addr) = listener.accept().await?;
        let tx = tx.clone();
        let rx = tx.subscribe();

        info!("Nova conexao de: {}", addr);

        tokio::spawn(async move {
            if let Err(e) = process_connection(socket, tx, rx).await {
                error!("Erro ao processar conexao de {}: {}", addr, e);
            }
        });
    }
}

/// Processa uma unica conexao TCP.
async fn process_connection(
    socket: TcpStream,
    tx: broadcast::Sender<Frame>,
    mut rx: broadcast::Receiver<Frame>
) -> Result<()> {
    let mut framed = Framed::new(socket, MessageCodec);

    loop {
        tokio::select! {
            // Ramo 1: recebe frame do cliente
            biased;

            result = framed.next() => {
                match result {
                    Some(Ok(frame)) => {
                        debug!("Recebido do cliente: {:?}", frame);
                        match frame.command_type {
                            CommandType::Pub => {
                                tx.send(frame)?;
                            },
                            CommandType::Sub => {
                                info!("Cliente se inscreveu.");
                            },
                            _ => {
                                warn!("Comando nao suportado recebido.");
                            }
                        }
                    }
                    Some(Err(e)) => {
                        error!("Erro ao ler do socket: {}", e);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }

            // Ramo 2: recebe mensagens do broadcast
            result = rx.recv() => {
                match result {
                    Ok(frame) => {
                        debug!("Enviando frame para o cliente: {:?}", frame);
                        if let Err(e) = framed.send(frame).await {
                            error!("Erro ao enviar para o socket: {}", e);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        error!("Cliente lento! {} mensagens perdidas.", n);
                    }
                    Err(e) => {
                        error!("Canal de broadcast fechado: {}", e);
                        break;
                    }
                }
            }
        }
    }

    info!("Cliente desconectado.");
    Ok(())
}
