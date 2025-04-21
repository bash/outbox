use anyhow::Result;
use futures::future::BoxFuture;
use futures::{FutureExt, StreamExt as _};
use http_body_util::{BodyExt as _, Limited};
use hyper::service::Service;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::server::graceful::GracefulShutdown;
use listenfd::ListenFd;
use std::error::Error as StdError;
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use std::pin::pin;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::UnixListener;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::sender::PENDING_DIRECTORY;

type BoxError = Box<dyn StdError + Send + Sync>;

const BODY_SIZE_LIMIT: usize = 100 * (1 << 10);

pub(crate) fn spawn_http_server(
    tx: UnboundedSender<PathBuf>,
    shutdown: CancellationToken,
) -> Result<JoinHandle<Result<()>>> {
    let listener = if let Some(listener) = listener_from_env()? {
        listener
    } else {
        eprintln!(
            "warn: Prefer socket activation to starting the service immediately.
tip: you can start outboxd with `systemd-socket-activate --listen=/path/to/outbox.sock outboxd`"
        );
        _ = fs::remove_file("outbox.sock");
        UnixListener::bind("outbox.sock")?
    };
    let local_addr = listener.local_addr()?;
    eprintln!("Listening on {:?}", local_addr);
    Ok(tokio::spawn(run_http_server(tx, listener, shutdown)))
}

async fn run_http_server(
    tx: UnboundedSender<PathBuf>,
    listener: UnixListener,
    shutdown: CancellationToken,
) -> Result<()> {
    let graceful = GracefulShutdown::new();
    shutdown
        .run_until_cancelled(try_run_http_server(tx, listener, &graceful))
        .await
        .unwrap_or(Ok(()))?;
    graceful.shutdown().await;
    Ok(())
}

async fn try_run_http_server(
    tx: UnboundedSender<PathBuf>,
    listener: UnixListener,
    graceful: &GracefulShutdown,
) -> Result<()> {
    let server = auto::Builder::new(TokioExecutor::new());
    loop {
        let (stream, _) = listener.accept().await?;

        let stream = TokioIo::new(stream);
        let connection = graceful.watch(
            server
                .serve_connection(stream, AcceptMailService { tx: tx.clone() })
                .into_owned(),
        );

        tokio::task::spawn(async move {
            if let Err(err) = connection.await {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

struct AcceptMailService {
    tx: UnboundedSender<PathBuf>,
}

impl Service<Request<hyper::body::Incoming>> for AcceptMailService {
    type Response = Response<String>;
    type Error = BoxError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, request: Request<hyper::body::Incoming>) -> Self::Future {
        let tx: UnboundedSender<PathBuf> = self.tx.clone();
        async move {
            match (request.method(), request.uri().path()) {
                (&Method::POST, "/queue") => accept_mail(tx, request).await,
                _ => Ok(not_found()),
            }
        }
        .boxed()
    }
}

async fn accept_mail(
    tx: UnboundedSender<PathBuf>,
    request: Request<hyper::body::Incoming>,
) -> Result<Response<String>, BoxError> {
    let body = Limited::new(request.into_body(), BODY_SIZE_LIMIT);
    let path = write_pending_mail(body).await?;
    _ = tx.send(path);
    Ok(Response::new(String::from("queued")))
}

fn not_found() -> Response<String> {
    let mut response = Response::new("not found".to_string());
    *response.status_mut() = StatusCode::NOT_FOUND;
    response
}

fn listener_from_env() -> io::Result<Option<UnixListener>> {
    ListenFd::from_env()
        .take_unix_listener(0)
        .and_then(|r| r.map(from_std).transpose())
}

fn from_std(listener: std::os::unix::net::UnixListener) -> io::Result<UnixListener> {
    listener.set_nonblocking(true)?;
    UnixListener::from_std(listener)
}

async fn write_pending_mail(source: Limited<hyper::body::Incoming>) -> Result<PathBuf, BoxError> {
    let message_id = Uuid::new_v4();
    let path: PathBuf = [PENDING_DIRECTORY, &format!("{message_id}.eml")]
        .iter()
        .collect();
    create_dir_all(PENDING_DIRECTORY).await?;
    let target = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await?;
    let mut target = BufWriter::new(target);
    let mut source = pin!(source.into_data_stream());
    while let Some(bytes) = source.next().await {
        let bytes = bytes?;
        target.write_all(&bytes).await?;
    }
    target.flush().await?;
    Ok(path)
}
