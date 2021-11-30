use std::mem::MaybeUninit;

pub trait PackDefault {
    fn pack_default() -> Self;
}

macro_rules! impl_base_type {
    ($type:ty) => {
        impl PackDefault for $type {
            fn pack_default() -> Self {
                Default::default()
            }
        }
    };
}

// Impl basic types
impl_base_type!(bool);
impl_base_type!(i8);
impl_base_type!(u8);
impl_base_type!(i16);
impl_base_type!(u16);
impl_base_type!(i32);
impl_base_type!(u32);
impl_base_type!(i64);
impl_base_type!(u64);
impl_base_type!(i128);
impl_base_type!(u128);
impl_base_type!(f32);
impl_base_type!(f64);
impl_base_type!(isize);
impl_base_type!(usize);
impl_base_type!(char);
impl_base_type!(String);

// Impl static array
impl<T: PackDefault, const N: usize> PackDefault for [T; N] {
    fn pack_default() -> Self {
        let mut arr: [T; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..N {
            arr[i] = T::pack_default();
        }

        arr
    }
}

// Impl dynamic array
impl<T: PackDefault> PackDefault for Vec<T> {
    fn pack_default() -> Self {
        Vec::new()
    }
}
