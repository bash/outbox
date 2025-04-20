use anyhow::{anyhow, Context as _, Result};
use std::env::{self};
use std::ffi::{OsStr, OsString};
use std::fmt::Display;
use std::fs::remove_file;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use tokio::fs::{self, create_dir_all, rename};
use tokio::process::Command;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

pub(crate) const PENDING_DIRECTORY: &str = "pending";
const FAILED_DIRECTORY: &str = "failed";
const DEFAULT_SENDMAIL_COMMAND: &str = "msmtp";

pub(crate) fn spawn_sender(rx: UnboundedReceiver<PathBuf>, shutdown: CancellationToken) {
    tokio::spawn(run_sender(rx, shutdown));
}

pub(crate) async fn queue_pending_mails(sender: UnboundedSender<PathBuf>) -> Result<()> {
    let mut entries = match fs::read_dir(PENDING_DIRECTORY).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    while let Some(entry) = entries.next_entry().await? {
        let type_ = entry.file_type().await?;
        if type_.is_file() {
            sender.send(entry.path())?;
        }
    }
    Ok(())
}

async fn run_sender(rx: UnboundedReceiver<PathBuf>, shutdown: CancellationToken) -> Result<()> {
    shutdown
        .run_until_cancelled(try_run_sender(rx))
        .await
        .unwrap_or(Ok(()))
}

async fn try_run_sender(mut rx: UnboundedReceiver<PathBuf>) -> Result<()> {
    while let Some(path) = rx.recv().await {
        send_mail(&path).await?;
    }
    Ok(())
}

async fn send_mail(path: &Path) -> Result<()> {
    match try_send_mail(path).await {
        Ok(_) => Ok(remove_file(path)?),
        Err(error) => {
            println!("Error sending email {}:\n{:?}", path.display(), error);
            let mut failed_path = PathBuf::from(FAILED_DIRECTORY);
            create_dir_all(&failed_path).await?;
            failed_path.push(path.file_name().context("File has no file name")?);
            rename(path, failed_path).await?;
            Ok(())
        }
    }
}

async fn try_send_mail(path: &Path) -> Result<()> {
    let sendmail = get_sendmail_program();
    let file = std::fs::File::open(path)?;
    let status = Command::new(&sendmail)
        .arg("--read-recipients")
        .arg("--read-envelope-from")
        .args(env::args_os().skip(1))
        .stdin(file)
        .status()
        .await?;
    if status.success() {
        Ok(())
    } else {
        let status_code = status.code().unwrap_or(-1);
        Err(anyhow!(
            "Command {} failed with exit code {status_code}",
            display(&sendmail),
        ))
    }
}

fn get_sendmail_program() -> OsString {
    env::var_os("SENDMAIL").unwrap_or_else(|| DEFAULT_SENDMAIL_COMMAND.into())
}

fn display(s: &OsStr) -> impl Display + '_ {
    Path::new(s).display()
}
