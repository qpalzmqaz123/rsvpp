use super::Transport;
use crate::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        unix::{OwnedReadHalf, OwnedWriteHalf},
        UnixStream,
    },
    sync::Mutex,
};

pub struct UnixTransport {
    rd: Mutex<OwnedReadHalf>,
    wr: Mutex<OwnedWriteHalf>,
}

impl UnixTransport {
    pub async fn new(sock: &str) -> Result<Self> {
        // Create unix stream
        let stream = UnixStream::connect(sock).await?;

        // Split stream to owned read & write
        let (rd, wr) = stream.into_split();

        Ok(Self {
            rd: Mutex::new(rd),
            wr: Mutex::new(wr),
        })
    }
}

#[async_trait::async_trait]
impl Transport for UnixTransport {
    async fn write(&self, buf: &[u8]) -> Result<()> {
        self.wr.lock().await.write_all(buf).await?;

        Ok(())
    }

    async fn read(&self, nbytes: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0_u8; nbytes];
        self.rd.lock().await.read_exact(&mut buf).await?;

        Ok(buf)
    }
}
