use std::{collections::HashMap, sync::Arc, time::Instant};

use tokio::sync::{broadcast, Mutex};

use crate::{
    hard_coded_message::ApiMessageReplyHeader,
    message::{Message, MessageHeader},
    pack::Pack,
    Error, Result, Transport,
};

type RecvCacheT = Arc<Mutex<HashMap<u32, Vec<RecvEntry>>>>;

const GC_LENGTH_THRESHOLD: usize = 64;
const GC_TIME_THRESHOLD: u32 = 30; // Seconds

#[derive(Debug)]
pub struct RecvEntry {
    pub header: ApiMessageReplyHeader,
    pub data: Vec<u8>,
    pub timestamp: u32,
}

pub struct Session {
    transport: Arc<dyn Transport + Sync + Send>,
    recv_cache: RecvCacheT,
    signal_rx: broadcast::Receiver<()>,
}

impl Session {
    pub fn new(transport: Arc<dyn Transport + Sync + Send>) -> Self {
        let (signal_tx, signal_rx) = broadcast::channel::<()>(16);
        let recv_cache = Arc::new(Mutex::new(HashMap::new()));

        // Create recv task
        RecvTask::start(recv_cache.clone(), transport.clone(), signal_tx);

        Self {
            transport,
            recv_cache,
            signal_rx,
        }
    }

    pub async fn send_msg<T: Pack>(&self, mut msg: Message<T>) -> Result<()> {
        let buf = msg.encode()?;
        // TODO: Add timeout
        self.transport.write(&buf).await?;

        Ok(())
    }

    pub async fn recv_single_msg<T: Pack>(&mut self, ctx: u32, msg_id: u16) -> Result<T> {
        // Recv data
        let mut entries = self.recv(ctx).await?;

        // Verify entries length
        if entries.len() != 1 {
            return Err(Error::internal("Message length not 1"));
        }

        let entry = entries.pop().ok_or(Error::internal("Empty entry"))?;

        // Verify message id
        if entry.header._vl_msg_id != msg_id {
            return Err(Error::msg_id_mismatch(format!(
                "Message id mismatch, expect {}, received {}",
                msg_id, entry.header._vl_msg_id,
            )));
        }

        // Decode data
        let data = T::unpack(&entry.data, 0)?.0;

        Ok(data)
    }

    pub async fn recv(&mut self, ctx: u32) -> Result<Vec<RecvEntry>> {
        let entries = loop {
            // Get message from cache
            if let Some(entries) = self.recv_cache.lock().await.remove(&ctx) {
                break entries;
            }

            // Wait signal
            self.signal_rx
                .recv()
                .await
                .map_err(|e| Error::internal(format!("Recv signal error: {}", e)))?;
        };

        Ok(entries)
    }
}

struct RecvTask {
    cache: RecvCacheT,
    transport: Arc<dyn Transport + Sync + Send>,
    signal_tx: broadcast::Sender<()>,
}

impl RecvTask {
    pub fn start(
        cache: RecvCacheT,
        transport: Arc<dyn Transport + Sync + Send>,
        signal_tx: broadcast::Sender<()>,
    ) {
        let mut instance = Self {
            cache,
            transport,
            signal_tx,
        };

        tokio::spawn(async move {
            instance.run().await;
        });
    }

    async fn run(&mut self) {
        loop {
            if let Err(_) = self.recv_frame().await {
                // TODO: Log error
                tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;
            } else {
                // Send signal
                if let Err(_) = self.signal_tx.send(()) {
                    // If other error occurred, terminate task
                    break;
                }
            }
        }
    }

    async fn recv_frame(&mut self) -> Result<()> {
        // Receive header
        log::trace!("Try recv header");
        let header_size = MessageHeader::static_size();
        let header_buf = self.transport.read(header_size).await?;

        // Decode header
        let header = MessageHeader::decode(&header_buf)?;
        log::trace!("Header is: {:?}", header);

        // Receive data
        log::trace!("Try recv data");
        let data_buf = self.transport.read(header.len as usize).await?;
        log::trace!("Data length is: {:?}", data_buf.len());

        // Decode message header
        let msg_header = ApiMessageReplyHeader::unpack(&data_buf, 0)?.0;
        log::trace!("Data header is: {:?}", msg_header);

        // Lock cache & try to gc
        let mut cache = self.cache.lock().await;
        if cache.len() >= GC_LENGTH_THRESHOLD {
            Self::gc(&mut cache);
        }

        // Add message to cache
        let ctx = msg_header.context;
        let entry = RecvEntry {
            header: msg_header,
            data: data_buf,
            timestamp: Instant::now().elapsed().as_secs() as u32,
        };
        if let Some(old_vec) = cache.get_mut(&ctx) {
            log::trace!("Append to cache '{}'", entry.header.context);
            old_vec.push(entry);
        } else {
            log::trace!("New to cache '{}'", entry.header.context);
            cache.insert(ctx, vec![entry]);
        };

        Ok(())
    }

    fn gc(map: &mut HashMap<u32, Vec<RecvEntry>>) {
        log::debug!("Start gc");

        // Take map ownership
        let old_map = std::mem::replace(map, HashMap::new());

        // Replace map
        *map = old_map.into_iter().fold(HashMap::new(), |mut map, (k, v)| {
            let new_vec = v
                .into_iter()
                .filter(|entry| {
                    let now = Instant::now().elapsed().as_secs() as u32;
                    if now - entry.timestamp >= GC_TIME_THRESHOLD {
                        log::debug!("Message {:?} expired", entry.header);
                        false
                    } else {
                        true
                    }
                })
                .collect::<Vec<RecvEntry>>();

            if new_vec.len() > 0 {
                map.insert(k, new_vec);
            }
            map
        });
    }
}
