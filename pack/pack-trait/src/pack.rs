use std::{
    convert::TryInto,
    mem::{size_of, MaybeUninit},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Pack: Sized {
    /// Get pack size
    fn size(&self) -> usize;

    /// Get static size
    fn static_size() -> usize;

    /// Get align size
    fn align_size() -> usize;

    /// Pack data with given buffer
    fn pack(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Unpack data from buffer
    fn unpack(buf: &[u8], len: usize) -> Result<(Self, usize)>;

    /// Pack data to new buffer
    fn pack_vec(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![0_u8; self.size()];
        self.pack(&mut buf)?;

        Ok(buf)
    }
}

#[inline(always)]
pub fn align_offset(offset: usize, align: usize, skip: bool) -> usize {
    if skip {
        return offset;
    }

    if offset % align != 0 {
        ((offset - 1) / align + 1) * align
    } else {
        offset
    }
}

pub fn safe_slice<T>(slice: &[T], start: usize, n: Option<usize>) -> Result<&[T]> {
    if let Some(n) = n {
        if start + n > slice.len() {
            return Err("Slice out of range".into());
        }

        Ok(&slice[start..start + n])
    } else {
        if start >= slice.len() {
            return Err("Slice out of range".into());
        }

        Ok(&slice[start..])
    }
}

pub fn safe_slice_mut<T>(slice: &mut [T], start: usize, n: Option<usize>) -> Result<&mut [T]> {
    if let Some(n) = n {
        if start + n > slice.len() {
            return Err("Slice out of range".into());
        }

        Ok(&mut slice[start..start + n])
    } else {
        if start >= slice.len() {
            return Err("Slice out of range".into());
        }

        Ok(&mut slice[start..])
    }
}

#[macro_export]
macro_rules! max {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        {
            use std::cmp::max;
            max($x, $crate::max!( $($xs),+ ))
        }
    };
}

#[macro_export]
macro_rules! min {
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) => {
        {
            use std::cmp::min;
            min($x, $crate::min!( $($xs),+ ))
        }
    };
}

macro_rules! validate_buffer {
    ($buf:expr, $size:expr) => {
        if $buf.len() < $size {
            return Err(format!(
                "Out of buffer, expect size {}, remain size: {}",
                $size,
                $buf.len()
            )
            .into());
        }
    };
}

macro_rules! impl_pack_for_number {
    ($type:ty) => {
        impl Pack for $type {
            fn size(&self) -> usize {
                size_of::<$type>()
            }

            fn static_size() -> usize {
                size_of::<$type>()
            }

            fn align_size() -> usize {
                size_of::<$type>()
            }

            fn pack(&mut self, buf: &mut [u8]) -> Result<usize> {
                validate_buffer!(buf, self.size());
                for (i, v) in (*self).to_be_bytes().iter().enumerate() {
                    buf[i] = *v;
                }

                Ok(self.size())
            }

            fn unpack(buf: &[u8], _: usize) -> Result<(Self, usize)> {
                validate_buffer!(buf, Self::align_size());

                Ok((
                    <$type>::from_be_bytes((&buf[0..Self::align_size()]).try_into()?),
                    Self::align_size(),
                ))
            }
        }
    };
}

// Impl Pack for number
impl_pack_for_number!(i8);
impl_pack_for_number!(u8);
impl_pack_for_number!(i16);
impl_pack_for_number!(u16);
impl_pack_for_number!(i32);
impl_pack_for_number!(u32);
impl_pack_for_number!(i64);
impl_pack_for_number!(u64);
impl_pack_for_number!(f32);
impl_pack_for_number!(f64);

// Impl Pack for bool
impl Pack for bool {
    fn size(&self) -> usize {
        1
    }

    fn static_size() -> usize {
        1
    }

    fn align_size() -> usize {
        1
    }

    fn pack(&mut self, buf: &mut [u8]) -> Result<usize> {
        validate_buffer!(buf, 1);
        buf[0] = if *self { 1 } else { 0 };

        Ok(1)
    }

    fn unpack(buf: &[u8], _: usize) -> Result<(Self, usize)> {
        validate_buffer!(buf, 1);

        Ok((buf[0] != 0, 1))
    }
}

// Impl Pack for String
impl Pack for String {
    fn size(&self) -> usize {
        self.len() + 1
    }

    fn static_size() -> usize {
        std::mem::size_of::<*const u8>()
    }

    fn align_size() -> usize {
        1
    }

    fn pack(&mut self, buf: &mut [u8]) -> Result<usize> {
        let size = self.size();

        validate_buffer!(buf, size);
        for (i, c) in self.chars().enumerate() {
            buf[i] = c as u8;
        }
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
        Ok((s, len))
    }
}

// Impl Pack for [T; N]
impl<T: Pack, const N: usize> Pack for [T; N] {
    fn size(&self) -> usize {
        self.iter().fold(0, |sum, v| sum + v.size())
    }

    fn static_size() -> usize {
        T::static_size() * N
    }

    fn align_size() -> usize {
        T::align_size()
    }

    fn pack(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut offset = 0;
        for i in 0..N {
            validate_buffer!(buf, offset);
            offset += self[i].pack(&mut buf[offset..])?;
        }

        Ok(offset)
    }

    fn unpack(buf: &[u8], _: usize) -> Result<(Self, usize)> {
        let mut arr: [T; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut offset = 0;
        for i in 0..N {
            validate_buffer!(buf, offset);
            let res = T::unpack(&buf[offset..], 0)?;
            arr[i] = res.0;
            offset += res.1;
        }

        Ok((arr, offset))
    }
}

impl<T: Pack> Pack for Vec<T> {
    fn size(&self) -> usize {
        self.iter().fold(0, |sum, v| sum + v.size())
    }

    fn static_size() -> usize {
        std::mem::size_of::<*const u8>()
    }

    fn align_size() -> usize {
        T::align_size()
    }

    fn pack(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut offset = 0;
        for v in self {
            validate_buffer!(buf, offset);
            offset += v.pack(&mut buf[offset..])?;
        }

        Ok(offset)
    }

    fn unpack(buf: &[u8], len: usize) -> Result<(Self, usize)> {
        let mut arr: Vec<T> = Vec::new();
        let mut offset = 0;
        for _ in 0..len {
            validate_buffer!(buf, offset);
            let res = T::unpack(&buf[offset..], 0)?;
            arr.push(res.0);
            offset += res.1;
        }

        Ok((arr, offset))
    }
}
