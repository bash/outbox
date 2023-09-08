use anyhow::Context;
use anyhow::{anyhow, Result};
use std::env;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::remove_file;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::rename;
use tokio::fs::{self, create_dir_all};
use tokio::process::Command;
use tokio::select;
use tokio::signal::unix::signal;
use tokio::signal::unix::SignalKind;
use tokio::time;

const PENDING_DIRECTORY: &str = "pending";
const FAILED_DIRECTORY: &str = "failed";
const DEFAULT_SENDMAIL_COMMAND: &str = "msmtp";

#[tokio::main]
async fn main() -> Result<()> {
    let mut ctrl_c = signal(SignalKind::interrupt()).unwrap();
    let mut interval = time::interval(Duration::from_secs(5));
    loop {
        select!(
            _ = ctrl_c.recv() => { return Ok(()); }
            _ = interval.tick() => {
                send_mails().await?;
            },
        )
    }
}

async fn send_mails() -> Result<()> {
    let mut entries = match fs::read_dir(PENDING_DIRECTORY).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    while let Some(entry) = entries.next_entry().await? {
        let type_ = entry.file_type().await?;
        let path = entry.path();
        if type_.is_file() && !should_ignore_mail(&path) {
            send_mail(&path).await?;
        }
    }
    Ok(())
}

fn should_ignore_mail(path: &Path) -> bool {
    path.as_os_str()
        .to_str()
        .map(|s| s.starts_with('_'))
        .unwrap_or_default()
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
    let sendmail_command =
        env::var_os("SENDMAIL").unwrap_or_else(|| DEFAULT_SENDMAIL_COMMAND.into());
    let file = File::open(path)?;
    let status = Command::new(&sendmail_command)
        .arg("--read-recipients")
        .arg("--read-envelope-from")
        .args(env::args_os().skip(1))
        .stdin(file)
        .status()
        .await?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Command {} failed with exit code {}",
            display(&sendmail_command),
            status.code().unwrap_or(-1)
        ))
    }
}

fn display<'a>(s: &'a OsStr) -> impl Display + 'a {
    Path::new(s).display()
}
