use std::ops::{Deref, DerefMut};

use pack_trait::{validate_buffer, Pack, PackDefault, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynamicString(pub String);

impl DynamicString {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Deref for DynamicString {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl DerefMut for DynamicString {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl From<String> for DynamicString {
    fn from(s: String) -> Self {
        DynamicString(s)
    }
}

impl From<&str> for DynamicString {
    fn from(s: &str) -> Self {
        DynamicString(s.to_string())
    }
}

impl From<DynamicString> for String {
    fn from(ds: DynamicString) -> Self {
        ds.0
    }
}

impl Pack for DynamicString {
    fn size(&self) -> usize {
        4 + self.0.len()
    }

    fn static_size() -> usize {
        4
    }

    fn align_size() -> usize {
        4
    }

    fn pack(&mut self, buf: &mut [u8]) -> Result<usize> {
        let size = self.size();
        validate_buffer!(buf, size);

        (self.0.len() as u32).pack(buf)?;
        buf[4..].copy_from_slice(self.0.as_bytes());

        Ok(size)
    }

    fn unpack(buf: &[u8], len: usize) -> Result<(Self, usize)> {
        let (len, offset) = u32::unpack(buf, len)?;
        let s = String::from_utf8(buf[offset..offset + len as usize].to_vec())?;

        Ok((Self(s), offset + len as usize))
    }
}

impl PackDefault for DynamicString {
    fn pack_default() -> Self {
        Self(String::new())
    }
}

impl Default for DynamicString {
    fn default() -> Self {
        Self(String::new())
    }
}
