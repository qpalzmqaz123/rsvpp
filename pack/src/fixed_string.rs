use std::ops::{Deref, DerefMut};

use pack_trait::{validate_buffer, Pack, PackDefault, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedString<const N: usize>(pub String);

impl<const N: usize> FixedString<N> {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<const N: usize> Deref for FixedString<N> {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl<const N: usize> DerefMut for FixedString<N> {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl<const N: usize> From<String> for FixedString<N> {
    fn from(s: String) -> Self {
        FixedString(s)
    }
}

impl<const N: usize> From<&str> for FixedString<N> {
    fn from(s: &str) -> Self {
        FixedString(s.to_string())
    }
}

impl<const N: usize> From<FixedString<N>> for String {
    fn from(ds: FixedString<N>) -> Self {
        ds.0
    }
}

impl<const N: usize> Pack for FixedString<N> {
    fn size(&self) -> usize {
        self.0.len() + 1
    }

    fn static_size() -> usize {
        N
    }

    fn align_size() -> usize {
        1
    }

    fn pack(&mut self, buf: &mut [u8]) -> Result<usize> {
        let size = self.size();
        validate_buffer!(buf, size);

        buf[0..size - 1].copy_from_slice(self.0.as_bytes());
        buf[size - 1] = 0;

        Ok(size)
    }

    fn unpack(buf: &[u8], _: usize) -> Result<(Self, usize)> {
        let mut s_buf: Vec<u8> = Vec::new();
        for c in buf {
            if *c == 0 {
                break;
            } else {
                s_buf.push(*c);
            }
        }

        if buf.len() == s_buf.len() {
            return Err("\\0 Not found".into());
        }

        let s = String::from_utf8(s_buf)?;
        let len = s.len();
        Ok((Self(s), len))
    }
}

impl<const N: usize> PackDefault for FixedString<N> {
    fn pack_default() -> Self {
        Self(String::new())
    }
}

impl<const N: usize> Default for FixedString<N> {
    fn default() -> Self {
        Self(String::new())
    }
}
