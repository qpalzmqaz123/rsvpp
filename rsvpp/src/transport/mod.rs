mod unix;

use std::sync::Arc;

use crate::Result;
use unix::UnixTransport;

#[async_trait::async_trait]
pub trait Transport: Sync + Send {
    async fn write(&self, buf: &[u8]) -> Result<()>;
    async fn read(&self, nbytes: usize) -> Result<Vec<u8>>;
}

pub async fn unix(sock: &str) -> Result<Arc<dyn Transport>> {
    Ok(Arc::new(UnixTransport::new(sock).await?))
}

// TODO: Add tcp
