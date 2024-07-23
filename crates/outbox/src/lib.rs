use anyhow::Result;
use async_trait::async_trait;
use std::borrow::Cow;
use std::io;
use std::os::fd::{FromRawFd as _, IntoRawFd, OwnedFd};
use tokio::io::AsyncWriteExt;
use tokio_pipe::{pipe, PipeWrite};
use zbus::Connection;

pub use zbus;

#[derive(Debug, Clone)]
pub struct Outbox(Connection);

impl Outbox {
    /// Creates a new [`Connection`]
    pub fn new(connection: Connection) -> Self {
        Outbox(connection)
    }

    /// Create an [`Outbox`] connected to the session/user bus.
    pub async fn session() -> Result<Self> {
        Ok(Outbox::new(Connection::session().await?))
    }

    /// Create an [`Outbox`] connected to the system bus.
    pub async fn system() -> Result<Self> {
        Ok(Outbox::new(Connection::system().await?))
    }

    /// Adds a new message to the queue.
    pub async fn queue<M: OutboxMessage>(&self, message: M) -> Result<()> {
        let proxy = dbus::OutboxProxy::new(&self.0).await?;
        let (read_fd, mut writer) = message.into_writer().await?;
        let future = proxy.queue(read_fd.into());
        writer.flush().await?;
        future.await?;
        Ok(())
    }
}

#[async_trait]
pub trait OutboxMessage {
    type Writer: OutboxMessageWriter;
    async fn into_writer(self) -> io::Result<(OwnedFd, Self::Writer)>;
}

#[async_trait]
pub trait OutboxMessageWriter {
    async fn flush(&mut self) -> io::Result<()>;
}

#[async_trait]
impl<'a> OutboxMessage for &'a [u8] {
    type Writer = BytesWriter<'a>;
    async fn into_writer(self) -> io::Result<(OwnedFd, Self::Writer)> {
        Cow::Borrowed(self).into_writer().await
    }
}

#[async_trait]
impl OutboxMessage for Vec<u8> {
    type Writer = BytesWriter<'static>;
    async fn into_writer(self) -> io::Result<(OwnedFd, Self::Writer)> {
        Cow::<'static, [u8]>::Owned(self).into_writer().await
    }
}

#[async_trait]
impl<'a> OutboxMessage for Cow<'a, [u8]> {
    type Writer = BytesWriter<'a>;
    async fn into_writer(self) -> io::Result<(OwnedFd, Self::Writer)> {
        let (read, write) = pipe()?;
        let read_fd = unsafe { OwnedFd::from_raw_fd(read.into_raw_fd()) };
        Ok((
            read_fd,
            Self::Writer {
                writer: Some(write),
                data: self,
            },
        ))
    }
}

pub struct BytesWriter<'a> {
    data: Cow<'a, [u8]>,
    writer: Option<PipeWrite>,
}

#[async_trait]
impl<'a> OutboxMessageWriter for BytesWriter<'a> {
    async fn flush(&mut self) -> io::Result<()> {
        if let Some(mut write) = self.writer.take() {
            write.write_all(&self.data).await?;
            AsyncWriteExt::flush(&mut write).await?;
        }
        Ok(())
    }
}

mod dbus {
    use std::fmt;
    use zbus::zvariant::OwnedFd;
    use zbus::{proxy, DBusError};

    #[proxy(
        interface = "garden.tau.Outbox1",
        default_service = "garden.tau.Outbox",
        default_path = "/garden/tau/Outbox"
    )]
    pub(crate) trait Outbox {
        fn queue(&self, mail: OwnedFd) -> Result<(), QueueError>;
    }

    #[derive(Debug, DBusError)]
    #[zbus(prefix = "garden.tau.Outbox", impl_display = false)]
    pub(crate) enum QueueError {
        #[zbus(error)]
        Zbus(zbus::Error),
        Io(String),
    }

    impl fmt::Display for QueueError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = self.name();
            match self {
                Self::Zbus(e) => write!(f, "{name}: {e}"),
                _ => {
                    let description = self.description().unwrap_or("no description");
                    write!(f, "{name}: {description}")
                }
            }
        }
    }
}
