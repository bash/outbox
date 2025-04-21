use std::path::Path;

use reqwest::Client as HttpClient;

#[derive(Debug, Clone)]
pub struct Outbox(HttpClient);

impl Outbox {
    /// Creates a new [`Connection`]
    pub fn new(connection: HttpClient) -> Self {
        Outbox(connection)
    }

    pub fn new_for_path(path: impl AsRef<Path>) -> reqwest::Result<Self> {
        Ok(Self::new(
            HttpClient::builder().unix_socket(path.as_ref()).build()?,
        ))
    }

    /// Adds a new message to the queue.
    pub async fn queue(&self, message: Vec<u8>) -> reqwest::Result<()> {
        self.0
            .post("http://localhost/queue")
            .body(message)
            .send()
            .await?;
        Ok(())
    }
}
