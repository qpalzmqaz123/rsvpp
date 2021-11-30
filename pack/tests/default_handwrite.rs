#![cfg(test)]
use pack::PackDefault;

#[test]
fn test1() {
    #[derive(Debug, PartialEq, Eq)]
    struct A {
        a: u8,
        b: Vec<u16>,
        c: [u32; 2],
    }

    impl PackDefault for A {
        fn pack_default() -> Self {
            Self {
                a: u8::pack_default(),
                b: <Vec<u16>>::pack_default(),
                c: <[u32; 2]>::pack_default(),
            }
        }
    }

    let a = A {
        a: 0,
        b: Vec::new(),
        c: [0, 0],
    };
    let b = A::pack_default();

    assert_eq!(a, b)
}
