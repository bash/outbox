use anyhow::Result;
use std::process::ExitCode;
use tokio::signal;
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;

mod http_server;
mod sender;

#[tokio::main]
async fn main() -> Result<ExitCode> {
    let shutdown = CancellationToken::new();
    let (tx, rx) = unbounded_channel();
    sender::spawn_sender(rx, shutdown.clone());
    http_server::spawn_http_server(tx.clone(), shutdown.clone())?;
    sender::queue_pending_mails(tx.clone()).await?;
    signal::ctrl_c().await.expect("failed to listen for event");
    shutdown.cancel();

    Ok(ExitCode::SUCCESS)
}
