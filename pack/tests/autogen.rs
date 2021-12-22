#![cfg(test)]

use pack::{pack_union, Pack};

#[test]
fn test1() {
    #[repr(C)]
    struct C {
        a: u8,
        b: u16,
        c: u32,
    }

    #[derive(Pack, Debug, PartialEq, Eq)]
    struct A {
        a: u8,
        b: u16,
        c: u32,
    }

    let mut a = A { a: 1, b: 2, c: 3 };
    let c = C {
        a: 1_u8.to_be(),
        b: 2_u16.to_be(),
        c: 3_u32.to_be(),
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(A::static_size(), std::mem::size_of::<C>());
    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
}

#[test]
fn test1_packed() {
    #[repr(C, packed)]
    struct C {
        a: u8,
        b: u16,
        c: u32,
    }

    #[derive(Pack, Debug, PartialEq, Eq)]
    #[packed]
    struct A {
        a: u8,
        b: u16,
        c: u32,
    }

    let mut a = A { a: 1, b: 2, c: 3 };
    let c = C {
        a: 1_u8.to_be(),
        b: 2_u16.to_be(),
        c: 3_u32.to_be(),
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(A::static_size(), std::mem::size_of::<C>());
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

    #[derive(Pack, Debug, PartialEq, Eq)]
    struct A {
        #[len(4)]
        a: String, // Fixed 4
        b: [u8; 2],
        c_len: u32,
        #[len("c_len")]
        c: Vec<u8>, // Dynamic size with c_len
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

    // assert_eq!(A::static_size(), std::mem::size_of::<C>());
    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
}

#[test]
fn test2_packed() {
    #[repr(C, packed)]
    struct C {
        a: [u8; 4],
        b: [u8; 2],
        c_len: u32,
        c: [u8; 3],
    }

    #[derive(Pack, Debug, PartialEq, Eq)]
    #[packed]
    struct A {
        #[len(4)]
        a: String, // Fixed 4
        b: [u8; 2],
        c_len: u32,
        #[len("c_len")]
        c: Vec<u8>, // Dynamic size with c_len
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

    // assert_eq!(A::static_size(), std::mem::size_of::<C>());
    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
}

#[test]
fn test3() {
    #[repr(C)]
    struct C {
        a: u16,
        b: u32,
        c: [u8; 5],
    }

    #[derive(Pack, Debug, PartialEq, Eq)]
    struct A {
        a: u16,
        b: u32,
        #[len(5)]
        c: String,
    }

    let mut a = A {
        a: 1,
        b: 2,
        c: "ab".to_string(),
    };
    let c = C {
        a: 1_u16.to_be(),
        b: 2_u32.to_be(),
        c: ['a' as u8, 'b' as u8, 0, 0, 0],
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(A::static_size(), std::mem::size_of::<C>());
    assert_eq!(a.size(), std::mem::size_of::<C>());
    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
}

#[test]
fn test3_packed() {
    #[repr(C, packed)]
    struct C {
        a: u16,
        b: u32,
        c: [u8; 5],
    }

    #[derive(Pack, Debug, PartialEq, Eq)]
    #[packed]
    struct A {
        a: u16,
        b: u32,
        #[len(5)]
        c: String,
    }

    let mut a = A {
        a: 1,
        b: 2,
        c: "ab".to_string(),
    };
    let c = C {
        a: 1_u16.to_be(),
        b: 2_u32.to_be(),
        c: ['a' as u8, 'b' as u8, 0, 0, 0],
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(A::static_size(), std::mem::size_of::<C>());
    assert_eq!(a.size(), std::mem::size_of::<C>());
    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
}

#[test]
fn test4() {
    #[derive(Pack, Debug, PartialEq, Eq)]
    #[pack_type("u32")]
    enum A {
        #[value(0)]
        A,
        #[value(1)]
        B,
        #[value(2)]
        C,
    }

    assert_eq!(A::static_size(), std::mem::size_of::<u32>());
    assert_eq!(A::A.pack_vec().unwrap(), vec![0, 0, 0, 0]);
    assert_eq!(A::B.pack_vec().unwrap(), vec![0, 0, 0, 1]);
    assert_eq!(A::C.pack_vec().unwrap(), vec![0, 0, 0, 2]);
    assert_eq!(A::unpack(&vec![0, 0, 0, 0], 0).unwrap().0, A::A);
    assert_eq!(A::unpack(&vec![0, 0, 0, 1], 0).unwrap().0, A::B);
    assert_eq!(A::unpack(&vec![0, 0, 0, 2], 0).unwrap().0, A::C);
}

#[test]
fn test5() {
    #[derive(Pack, Debug, PartialEq, Eq)]
    #[pack_type("u16")]
    enum A {
        #[value(0)]
        A,
        #[value(1)]
        B,
        #[value(2)]
        C,
        #[default]
        NotMatch(u16),
    }

    assert_eq!(A::static_size(), std::mem::size_of::<u16>());
    assert_eq!(A::A.pack_vec().unwrap(), vec![0, 0]);
    assert_eq!(A::B.pack_vec().unwrap(), vec![0, 1]);
    assert_eq!(A::C.pack_vec().unwrap(), vec![0, 2]);
    assert_eq!(A::NotMatch(10).pack_vec().unwrap(), vec![0, 10]);
    assert_eq!(A::unpack(&vec![0, 0], 0).unwrap().0, A::A);
    assert_eq!(A::unpack(&vec![0, 1], 0).unwrap().0, A::B);
    assert_eq!(A::unpack(&vec![0, 2], 0).unwrap().0, A::C);
    assert_eq!(A::unpack(&vec![0, 20], 0).unwrap().0, A::NotMatch(20));
}

#[test]
fn test6() {
    #[repr(C)]
    union C {
        ipv4: u32,
        ipv6: [u32; 4],
    }

    #[pack_union]
    #[derive(Debug, PartialEq, Eq)]
    union A {
        ipv4: u32,
        ipv6: [u32; 4],
    }

    // Test v4
    let mut a = A::from_ipv4(10);

    let c = C {
        ipv4: 10u32.to_be(),
    };
    let mut v = vec![0_u8; std::mem::size_of::<C>()];
    unsafe { std::ptr::copy(&c as *const C as *const u8, v.as_mut_ptr(), v.len()) };

    assert_eq!(A::static_size(), std::mem::size_of::<C>());
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

    assert_eq!(A::static_size(), std::mem::size_of::<C>());
    assert_eq!(a.pack_vec().unwrap(), v);
    assert_eq!(A::unpack(&v, 0).unwrap().0, a);
    assert_eq!(a.ipv6(), [1, 2, 3, 4]);
}
