use anyhow::Result;
use std::os::fd::{AsRawFd as _, FromRawFd as _};
use tokio::fs::File;
use tokio::io::AsyncWriteExt as _;
use zbus::zvariant::OwnedFd;
use zbus::Connection;

pub use zbus;

pub async fn queue(connection: &Connection, data: &[u8]) -> Result<()> {
    let proxy = dbus::OutboxProxy::new(connection).await?;
    let (pipe_reader, pipe_writer) = os_pipe::pipe()?;
    let pipe_reader = unsafe { OwnedFd::from_raw_fd(pipe_reader.as_raw_fd()) };
    let mut pipe_writer = unsafe { File::from_raw_fd(pipe_writer.as_raw_fd()) };
    let future = proxy.queue(pipe_reader);
    pipe_writer.write_all(data).await?;
    pipe_writer.flush().await?;
    drop(pipe_writer);
    future.await?;
    Ok(())
}

mod dbus {
    use zbus::zvariant::OwnedFd;
    use zbus::{dbus_proxy, DBusError};

    #[dbus_proxy(
        interface = "garden.tau.Outbox1",
        default_service = "garden.tau.Outbox",
        default_path = "/garden/tau/Outbox"
    )]
    pub(crate) trait Outbox {
        fn queue(&self, mail: OwnedFd) -> Result<(), QueueError>;
    }

    #[derive(Debug, DBusError)]
    #[dbus_error(prefix = "garden.tau.Outbox")]
    pub(crate) enum QueueError {
        #[dbus_error(zbus_error)]
        Zbus(zbus::Error),
        Io(String),
    }
}
