#![cfg(test)]
use pack::Pack;

#[test]
fn test1() {
    #[repr(C)]
    struct C {
        a: u8,
        b: u16,
        c: u32,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct A {
        a: u8,
        b: u16,
        c: u32,
    }

    impl Pack for A {
        fn size(&self) -> usize {
            let mut offset = 0;

            offset = pack::align_offset(offset, u8::align_size(), false);
            offset += self.a.size();

            offset = pack::align_offset(offset, u16::align_size(), false);
            offset += self.b.size();

            offset = pack::align_offset(offset, u32::align_size(), false);
            offset += self.c.size();

            offset = pack::align_offset(offset, Self::align_size(), false);
            offset
        }

        fn static_size() -> usize {
            let mut offset = 0;

            offset = pack::align_offset(offset, u8::align_size(), false);
            offset += u8::static_size();

            offset = pack::align_offset(offset, u16::align_size(), false);
            offset += u16::static_size();

            offset = pack::align_offset(offset, u32::align_size(), false);
            offset += u32::static_size();

            offset = pack::align_offset(offset, Self::align_size(), false);
            offset
        }

        fn align_size() -> usize {
            pack::max!(u8::align_size(), u16::align_size(), u32::align_size())
        }

        fn pack(&mut self, buf: &mut [u8]) -> pack::Result<usize> {
            let mut offset = 0;

            offset = pack::align_offset(offset, u8::align_size(), false);
            offset += self
                .a
                .pack(pack::safe_slice_mut(buf, offset, Some(self.a.size()))?)?;

            offset = pack::align_offset(offset, u16::align_size(), false);
            offset += self
                .b
                .pack(pack::safe_slice_mut(buf, offset, Some(self.b.size()))?)?;

            offset = pack::align_offset(offset, u32::align_size(), false);
            offset += self
                .c
                .pack(pack::safe_slice_mut(buf, offset, Some(self.c.size()))?)?;

            Ok(offset)
        }

        fn unpack(buf: &[u8], _: usize) -> pack::Result<(Self, usize)> {
            let mut offset = 0;

            offset = pack::align_offset(offset, u8::align_size(), false);
            let res = u8::unpack(pack::safe_slice(&buf, offset, None)?, 0)?;
            let a = res.0;
            offset += res.1;

            offset = pack::align_offset(offset, u16::align_size(), false);
            let res = u16::unpack(pack::safe_slice(&buf, offset, None)?, 0)?;
            let b = res.0;
            offset += res.1;

            offset = pack::align_offset(offset, u32::align_size(), false);
            let res = u32::unpack(pack::safe_slice(&buf, offset, None)?, 0)?;
            let c = res.0;
            offset += res.1;

            Ok((Self { a, b, c }, offset))
        }
    }

    let mut a = A { a: 1, b: 2, c: 3 };
    let c = C {
        a: 1_u8.to_be(),
        b: 2_u16.to_be(),
        c: 3_u32.to_be(),
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
}

#[test]
fn test2() {
    #[repr(C)]
    struct C {
        a: [u8; 4],
        b: [u8; 2],
        c_len: u32,
        c: [u8; 3],
    }

    #[derive(Debug, PartialEq, Eq)]
    struct A {
        a: String, // Fixed 4
        b: [u8; 2],
        c_len: u32,
        c: Vec<u8>, // Dynamic size with c_len
    }

    impl Pack for A {
        fn size(&self) -> usize {
            let mut offset = 0;

            offset = pack::align_offset(offset, String::align_size(), false);
            offset += 4;

            offset = pack::align_offset(offset, <[u8; 2]>::align_size(), false);
            offset += self.b.size();

            offset = pack::align_offset(offset, u32::align_size(), false);
            offset += self.c_len.size();

            offset = pack::align_offset(offset, <Vec<u8>>::align_size(), false);
            offset += self.c.size();

            offset = pack::align_offset(offset, Self::align_size(), false);
            offset
        }

        fn static_size() -> usize {
            let mut offset = 0;

            offset = pack::align_offset(offset, u8::align_size(), false);
            offset += String::static_size();

            offset = pack::align_offset(offset, u16::align_size(), false);
            offset += <[u8; 2]>::static_size();

            offset = pack::align_offset(offset, u32::align_size(), false);
            offset += u32::static_size();

            offset = pack::align_offset(offset, u32::align_size(), false);
            offset += <Vec<u8>>::static_size();

            offset = pack::align_offset(offset, Self::align_size(), false);
            offset
        }

        fn align_size() -> usize {
            pack::max!(
                String::align_size(),
                <[u8; 2]>::align_size(),
                u32::align_size(),
                <Vec<u8>>::align_size()
            )
        }

        fn pack(&mut self, buf: &mut [u8]) -> pack::Result<usize> {
            self.c_len = self.c.len() as u32;

            let mut offset = 0;

            offset = pack::align_offset(offset, String::align_size(), false);
            self.a
                .pack(pack::safe_slice_mut(buf, offset, Some(self.a.size()))?)?;
            offset += 4;

            offset = pack::align_offset(offset, <[u8; 2]>::align_size(), false);
            offset += self
                .b
                .pack(pack::safe_slice_mut(buf, offset, Some(self.b.size()))?)?;

            offset = pack::align_offset(offset, u32::align_size(), false);
            offset +=
                self.c_len
                    .pack(pack::safe_slice_mut(buf, offset, Some(self.c_len.size()))?)?;

            offset = pack::align_offset(offset, <Vec<u8>>::align_size(), false);
            offset += self
                .c
                .pack(pack::safe_slice_mut(buf, offset, Some(self.c.size()))?)?;

            Ok(offset)
        }

        fn unpack(buf: &[u8], _: usize) -> pack::Result<(Self, usize)> {
            let mut offset = 0;

            offset = pack::align_offset(offset, String::align_size(), false);
            let res = String::unpack(pack::safe_slice(buf, offset, None)?, 0)?;
            let a = res.0;
            offset += 4;

            offset = pack::align_offset(offset, <[u8; 2]>::align_size(), false);
            let res = <[u8; 2]>::unpack(pack::safe_slice(&buf, offset, None)?, 0)?;
            let b = res.0;
            offset += res.1;

            offset = pack::align_offset(offset, u32::align_size(), false);
            let res = u32::unpack(pack::safe_slice(&buf, offset, None)?, 0)?;
            let c_len = res.0;
            offset += res.1;

            offset = pack::align_offset(offset, <Vec<u8>>::align_size(), false);
            let res = <Vec<u8>>::unpack(pack::safe_slice(&buf, offset, None)?, c_len as usize)?;
            let c = res.0;
            offset += res.1;

            Ok((Self { a, b, c_len, c }, offset))
        }
    }

    let mut a = A {
        a: "ab".to_string(),
        b: [1, 2],
        c_len: 3,
        c: vec![4, 5, 6],
    };
    let c = C {
        a: ['a' as u8, 'b' as u8, 0, 0],
        b: [1, 2],
        c_len: 3_u32.to_be(),
        c: [4, 5, 6],
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
}

#[test]
fn test3() {
    #[derive(Debug, PartialEq, Eq)]
    enum A {
        A,
        B,
        C,
    }

    impl Pack for A {
        fn size(&self) -> usize {
            Self::align_size()
        }

        fn static_size() -> usize {
            Self::align_size()
        }

        fn align_size() -> usize {
            u32::align_size()
        }

        fn pack(&mut self, buf: &mut [u8]) -> pack::Result<usize> {
            match self {
                Self::A => (0 as u32).pack(buf),
                Self::B => (1 as u32).pack(buf),
                Self::C => (2 as u32).pack(buf),
            }
        }

        fn unpack(buf: &[u8], len: usize) -> pack::Result<(Self, usize)> {
            let (v, size) = u32::unpack(buf, len)?;
            let e = match v {
                0 => Self::A,
                1 => Self::B,
                2 => Self::C,
                _ => return Err(format!("Enum Pack received invalid number: '{}'", v).into()),
            };

            Ok((e, size))
        }
    }

    assert!(true)
}

#[test]
fn test4() {
    #[repr(C)]
    union C {
        ipv4: u32,
        ipv6: [u32; 4],
    }

    // Union <u32, [u32; 4]>
    #[derive(Debug, PartialEq, Eq)]
    struct A {
        buf: Vec<u8>,
    }

    impl A {
        pub fn ipv4(&self) -> u32 {
            u32::unpack(&self.buf, 0).expect("Unpack error").0
        }

        pub fn from_ipv4(mut ip: u32) -> Self {
            let mut buf = vec![0u8; Self::static_size()];
            // TODO: Handle error
            ip.pack(&mut buf).expect("Pack error");

            Self { buf }
        }

        pub fn ipv6(&self) -> [u32; 4] {
            <[u32; 4]>::unpack(&self.buf, 0).expect("Unpack error").0
        }

        pub fn from_ipv6(mut ip: [u32; 4]) -> Self {
            let mut buf = vec![0u8; Self::static_size()];
            // TODO: Handle error
            ip.pack(&mut buf).expect("Pack error");

            Self { buf }
        }
    }

    impl Pack for A {
        fn size(&self) -> usize {
            Self::static_size()
        }

        fn static_size() -> usize {
            pack::max!(u32::static_size(), <[u32; 4]>::static_size())
        }

        fn align_size() -> usize {
            pack::max!(u32::align_size(), <[u32; 4]>::align_size())
        }

        fn pack(&mut self, buf: &mut [u8]) -> pack::Result<usize> {
            if buf.len() < self.buf.len() {
                return Err("Buffer not enough".into());
            }

            self.buf.iter().enumerate().for_each(|(i, v)| buf[i] = *v);

            Ok(self.buf.len())
        }

        fn unpack(buf: &[u8], _: usize) -> pack::Result<(Self, usize)> {
            let len = Self::static_size();
            if buf.len() < len {
                return Err("Buffer not enough".into());
            }

            Ok((
                Self {
                    buf: (&buf[0..len]).to_vec(),
                },
                len,
            ))
        }
    }

    // Test v4
    let mut a = A::from_ipv4(10);

    let c = C {
        ipv4: 10u32.to_be(),
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
    assert_eq!(a.ipv4(), 10);

    // Test v6
    let mut a = A::from_ipv6([1, 2, 3, 4]);

    let c = C {
        ipv6: [1u32.to_be(), 2u32.to_be(), 3u32.to_be(), 4u32.to_be()],
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
    assert_eq!(a.ipv6(), [1, 2, 3, 4]);
}
