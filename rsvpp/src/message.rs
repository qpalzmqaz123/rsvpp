use std::ops::{Deref, DerefMut};

use crate::{pack::Pack, Result};

pub trait MessageName {
    fn message_name() -> String;
}

pub trait MessageId {
    fn message_id(&self) -> u16;
    fn set_message_id(self, id: u16) -> Self;
}

pub trait MessageContext {
    fn context(&self) -> u32;
    fn set_context(self, ctx: u32) -> Self;
}

pub trait MessageClientId {
    fn client_index(&self) -> u32;
    fn set_client_index(self, idx: u32) -> Self;
}

pub trait MessageCrc {
    fn crc() -> &'static str;
}

#[derive(Debug, Default, Pack)]
pub struct MessageHeader {
    pub q: u64,
    pub len: u32,
    pub timestamp: u32,
}

impl MessageHeader {
    pub fn static_size() -> usize {
        8 + 4 + 4
    }

    pub fn encode(&mut self) -> Result<Vec<u8>> {
        Ok(self.pack_vec()?)
    }

    pub fn decode(buf: &[u8]) -> Result<Self> {
        Ok(Self::unpack(buf, 0)?.0)
    }
}

pub struct Message<T> {
    header: MessageHeader,
    _inner: T,
}

impl<T: Pack> Message<T> {
    pub fn new(msg: T) -> Self {
        Self {
            header: MessageHeader::default(),
            _inner: msg,
        }
    }

    pub fn encode(&mut self) -> Result<Vec<u8>> {
        // Encode inner
        let inner_buf = self._inner.pack_vec()?;
        let len = inner_buf.len();

        // Encode header
        self.header.len = len as u32;
        let mut buf = self.header.encode()?;

        // Encode entire buf
        buf.extend(inner_buf);

        Ok(buf)
    }

    pub fn decode(header: MessageHeader, buf: &[u8]) -> Result<Self> {
        let inner = T::unpack(buf, 0)?.0;

        Ok(Self {
            header,
            _inner: inner,
        })
    }
}

impl<T> Deref for Message<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self._inner
    }
}

impl<T> DerefMut for Message<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._inner
    }
}
