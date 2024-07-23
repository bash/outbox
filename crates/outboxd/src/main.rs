use anyhow::{anyhow, Context as _, Result};
use serde::Serialize;
use std::env::{self, VarError};
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::remove_file;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use tokio::fs::{self, create_dir_all, rename, File, OpenOptions};
use tokio::process::Command;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use uuid::Uuid;
use zbus::zvariant::OwnedFd;
use zbus::ConnectionBuilder;
use zbus::{interface, DBusError};

const PENDING_DIRECTORY: &str = "pending";
const FAILED_DIRECTORY: &str = "failed";
const DEFAULT_SENDMAIL_COMMAND: &str = "msmtp";

#[tokio::main]
async fn main() -> Result<()> {
    let (sender, mut receiver) = unbounded_channel();

    queue_pending_mails(sender.clone()).await?;

    let _connection = connection_builder()?
        .name("garden.tau.Outbox")?
        .serve_at("/garden/tau/Outbox", Outbox(sender))?
        .build()
        .await?;

    let mut ctrl_c = signal(SignalKind::interrupt()).unwrap();

    loop {
        select!(
            _ = ctrl_c.recv() => { return Ok(()); }
            Some(path) = receiver.recv() => {
                send_mail(&path).await?;
            },
        )
    }
}

fn connection_builder() -> Result<ConnectionBuilder<'static>> {
    match env::var("DBUS_CONNECTION").as_deref() {
        Ok("system") | Err(VarError::NotPresent) => Ok(ConnectionBuilder::system()?),
        Ok("session") => Ok(ConnectionBuilder::session()?),
        Ok(value) => Err(anyhow!("Invalid bus connection name '{value}'")),
        Err(e) => Err(e.clone().into()),
    }
}

struct Outbox(UnboundedSender<PathBuf>);

#[interface(interface = "garden.tau.Outbox1")]
impl Outbox {
    async fn queue(&self, mail: OwnedFd) -> Result<(), QueueError> {
        let fd: std::os::fd::OwnedFd = mail.into();
        let mut file = File::from(std::fs::File::from(fd));
        let path = write_pending_mail(&mut file).await?;
        _ = self.0.send(path); // If the channel is closed that means we're shutting down
        Ok(())
    }
}

#[derive(Debug, DBusError, Serialize)]
#[zbus(prefix = "garden.tau.Outbox")]
enum QueueError {
    Io(String),
}

impl From<io::Error> for QueueError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

async fn write_pending_mail(source: &mut File) -> io::Result<PathBuf> {
    let message_id = Uuid::new_v4();
    let path: PathBuf = [PENDING_DIRECTORY, &format!("{message_id}.eml")]
        .iter()
        .collect();

    create_dir_all(PENDING_DIRECTORY).await?;
    let mut target = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await?;
    tokio::io::copy(source, &mut target).await?;

    Ok(path)
}

async fn queue_pending_mails(sender: UnboundedSender<PathBuf>) -> Result<()> {
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
    let file = std::fs::File::open(path)?;
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

fn display(s: &OsStr) -> impl Display + '_ {
    Path::new(s).display()
}
