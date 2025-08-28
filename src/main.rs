use anyhow::Result;
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

    // Inicia o servidor e o mantem rodando.
    net::server::run().await?;

    Ok(())
}