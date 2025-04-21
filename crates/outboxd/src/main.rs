use anyhow::{Context as _, Result};
use futures::future::select;
use std::pin::pin;
use std::process::ExitCode;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;

mod http_server;
mod sender;

#[tokio::main]
async fn main() -> Result<ExitCode> {
    let shutdown = CancellationToken::new();
    let (tx, rx) = unbounded_channel();
    let sender = sender::spawn_sender(rx, shutdown.clone());
    let server = http_server::spawn_http_server(tx.clone(), shutdown.clone())
        .context("spawn http server")?;
    sender::queue_pending_mails(tx.clone())
        .await
        .context("re-queue pending mails")?;
    wait_for_signal().await;
    shutdown.cancel();
    _ = sender.await;
    _ = server.await;
    eprintln!("Server stopped");
    Ok(ExitCode::SUCCESS)
}

async fn wait_for_signal() {
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    let interrupt = pin!(interrupt.recv());
    let mut terminate = signal(SignalKind::terminate()).unwrap();
    let terminate = pin!(terminate.recv());
    select(interrupt, terminate).await;
}
