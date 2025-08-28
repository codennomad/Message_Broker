use anyhow::Result;
use tokio::sync::broadcast;
use crate::proto::frame::Frame;
use tracing_subscriber;

// Declarando os modulos que criamos
mod net;
mod proto;

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializa o subscriber de tracing.
    // Permite ver logs com niveis (info!, error!, debug!) e spans.
    // RUST_LOG=info cargo run (ou debug, trace) para controlar o nivel.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Iniciando o Message Broker...");

    // Cria o canal de broadcast.
    // O `Sender` (tx) envia mensagens para o canal
    // O `Receiver` (rx) e clonado para cada assinante
    // A capacidade (32) e o numero de mensagens que o canal pode reter se nao houver
    // receivers ativos. Quando um receiver lendo perde mensagens, ele recebe um erro `Lagged`
    let(tx, _rx) =  broadcast::channel::<Frame>(32);

    // Inicia o servidor e o mantem rodando.
    net::server::run(tx).await?;

    Ok(())
}