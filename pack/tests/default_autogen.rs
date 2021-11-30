#![cfg(test)]
use pack::PackDefault;

#[test]
fn test1() {
    #[derive(PackDefault, Debug, PartialEq, Eq)]
    struct A {
        a: u8,
        b: Vec<u16>,
        c: [u32; 2],
    }

    let a = A {
        a: 0,
        b: Vec::new(),
        c: [0, 0],
    };
    let b = A::pack_default();

    assert_eq!(a, b)
}
